//! Subtítulos en ficheiro: SRT, WebVTT e ASS/SSA. Léense e escríbense sen FFmpeg, así que a conversión, o cambio de
//! codificación (os subtítulos vellos adoitan vir en Windows-1252 e saen con «Ã±» nos reprodutores) e a resincronización fanse con
//! todo o control. Sempre se escribe un ficheiro novo en UTF-8.

use crate::error::{EngineError, Result};
use crate::integrity::{ensure_distinct, AtomicOutput};
use std::path::{Path, PathBuf};

/// Formatos de subtítulos que se len e se escriben.
pub const SUBTITLE_EXTS: &[&str] = &["srt", "vtt", "ass", "ssa"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Srt,
    Vtt,
    Ass,
}

impl Format {
    pub fn from_ext(ext: &str) -> Option<Format> {
        match ext.to_ascii_lowercase().as_str() {
            "srt" => Some(Format::Srt),
            "vtt" => Some(Format::Vtt),
            "ass" | "ssa" => Some(Format::Ass),
            _ => None,
        }
    }
}

/// Unha fala. O texto vai en forma «canónica»: liñas separadas por `\n` e só as marcas `<i>`, `<b>` e `<u>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub start: i64,
    pub end: i64,
    pub text: String,
    /// Só en ASS: o resto dos campos da liña `Dialogue:` (capa, estilo, nome, marxes, efecto) e o texto tal cal co que viña,
    /// para non perder estilos nin posicións ao pasar de ASS a ASS.
    ass: Option<AssLine>,
}

#[derive(Debug, Clone, PartialEq)]
struct AssLine {
    layer: String,
    style: String,
    name: String,
    margins: [String; 3],
    effect: String,
    raw_text: String,
}

#[derive(Debug, Clone)]
pub struct Subtitle {
    pub format: Format,
    pub cues: Vec<Cue>,
    /// Só en ASS: cabeceira (scripts, estilos e a liña `Format:` dos eventos) tal como vén no ficheiro.
    ass_header: Option<String>,
    /// Codificación en que viña o ficheiro (para avisar ao usuario).
    pub encoding: &'static str,
}

// --------------------------------------------------------------------------------------------- codificación

/// Texto do ficheiro e a codificación que tiña. UTF-8 (con ou sen BOM) e UTF-16 con BOM; o resto vale como Windows-1252, que é o
/// que levan case todos os subtítulos antigos en galego, español, portugués e francés (ISO-8859-1 incluído).
pub fn decode(bytes: &[u8]) -> (String, &'static str) {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return (String::from_utf8_lossy(rest).into_owned(), "UTF-8");
    }
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        let enc = if bytes[0] == 0xFF { encoding_rs::UTF_16LE } else { encoding_rs::UTF_16BE };
        let (t, _, _) = enc.decode(bytes);
        return (t.into_owned(), "UTF-16");
    }
    match std::str::from_utf8(bytes) {
        Ok(t) => (t.to_string(), "UTF-8"),
        Err(_) => (encoding_rs::WINDOWS_1252.decode(bytes).0.into_owned(), "Windows-1252"),
    }
}

// ---------------------------------------------------------------------------------------------- tempos

fn parse_clock(s: &str) -> Option<i64> {
    // «H:MM:SS,mmm», «HH:MM:SS.mmm» ou «MM:SS.mmm» (WebVTT sen horas); tamén «H:MM:SS.cc» de ASS (centésimas).
    let s = s.trim();
    let (main, frac) = match s.find([',', '.']) {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    };
    let parts: Vec<&str> = main.split(':').collect();
    let nums: Option<Vec<i64>> = parts.iter().map(|p| p.trim().parse::<i64>().ok()).collect();
    let nums = nums?;
    let (h, m, sec) = match nums.as_slice() {
        [h, m, s] => (*h, *m, *s),
        [m, s] => (0, *m, *s),
        _ => return None,
    };
    if h < 0 || !(0..60).contains(&m) || !(0..60).contains(&sec) || !frac.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    // A parte decimal son fraccións de segundo, tantas cifras como haxa («5» = 0,5; «50» = 0,50; «500» = 0,500).
    let ms = if frac.is_empty() {
        0
    } else {
        let digits: String = frac.chars().take(3).collect();
        let v: i64 = digits.parse().ok()?;
        v * 10_i64.pow(3 - digits.len() as u32)
    };
    Some(((h * 60 + m) * 60 + sec) * 1000 + ms)
}

fn clock(ms: i64, sep: char) -> String {
    let ms = ms.max(0);
    format!("{:02}:{:02}:{:02}{}{:03}", ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, sep, ms % 1000)
}

fn ass_clock(ms: i64) -> String {
    // Centésimas, redondeando.
    let cs = (ms.max(0) + 5) / 10;
    format!("{}:{:02}:{:02}.{:02}", cs / 360_000, cs / 6000 % 60, cs / 100 % 60, cs % 100)
}

// -------------------------------------------------------------------------------------------- texto inline

/// Fala en forma canónica a partir do texto dun SRT ou WebVTT: deixa `<i>`, `<b>` e `<u>`, e quita o resto de marcas
/// (cores, `<c.clase>`, `<v Nome>`, `{\an8}`).
fn canonical_html(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(['<', '{']) {
        out.push_str(&rest[..i]);
        let open = rest.as_bytes()[i];
        let close = if open == b'<' { '>' } else { '}' };
        match rest[i..].find(close) {
            Some(j) => {
                let tag = &rest[i..i + j + 1];
                if open == b'<' {
                    let t = tag.to_ascii_lowercase();
                    if matches!(t.as_str(), "<i>" | "</i>" | "<b>" | "</b>" | "<u>" | "</u>") {
                        out.push_str(&t);
                    }
                }
                rest = &rest[i + j + 1..];
            }
            None => {
                out.push_str(&rest[i..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// Texto dunha liña ASS a forma canónica: `\N` é salto de liña, `\h` espazo duro e as marcas `{\i1}` pasan a `<i>`.
fn canonical_ass(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                let mut block = String::new();
                for d in chars.by_ref() {
                    if d == '}' {
                        break;
                    }
                    block.push(d);
                }
                for tag in block.split('\\') {
                    match tag.trim() {
                        "i1" => out.push_str("<i>"),
                        "i0" => out.push_str("</i>"),
                        "b1" => out.push_str("<b>"),
                        "b0" => out.push_str("</b>"),
                        "u1" => out.push_str("<u>"),
                        "u0" => out.push_str("</u>"),
                        _ => {}
                    }
                }
            }
            '\\' => match chars.peek() {
                Some('N') | Some('n') => {
                    chars.next();
                    out.push('\n');
                }
                Some('h') => {
                    chars.next();
                    out.push(' ');
                }
                _ => out.push('\\'),
            },
            _ => out.push(c),
        }
    }
    out
}

fn ass_text(canonical: &str) -> String {
    canonical
        .replace("<i>", "{\\i1}")
        .replace("</i>", "{\\i0}")
        .replace("<b>", "{\\b1}")
        .replace("</b>", "{\\b0}")
        .replace("<u>", "{\\u1}")
        .replace("</u>", "{\\u0}")
        .replace("\r\n", "\\N")
        .replace('\n', "\\N")
}

// --------------------------------------------------------------------------------------------------- ler

fn parse_timing_line(line: &str) -> Option<(i64, i64)> {
    let (a, b) = line.split_once("-->")?;
    // Despois do fin pode haber axustes de posición (WebVTT) ou nada.
    let end = b.trim().split_whitespace().next()?;
    Some((parse_clock(a.trim())?, parse_clock(end)?))
}

/// SRT e WebVTT: bloques separados por liñas en branco; o tempo está na liña con «-->».
fn parse_blocks(text: &str, format: Format) -> Vec<Cue> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut cues = Vec::new();
    for block in text.split("\n\n") {
        let lines: Vec<&str> = block.lines().collect();
        let Some(at) = lines.iter().position(|l| l.contains("-->")) else { continue };
        // As notas, estilos e rexións do WebVTT non son falas.
        if lines.first().is_some_and(|l| l.starts_with("NOTE") || l.starts_with("STYLE") || l.starts_with("REGION")) {
            continue;
        }
        let Some((start, end)) = parse_timing_line(lines[at]) else { continue };
        let body = lines[at + 1..].join("\n");
        let body = canonical_html(body.trim_end_matches('\n'));
        if body.trim().is_empty() {
            continue;
        }
        let _ = format;
        cues.push(Cue { start, end, text: body, ass: None });
    }
    cues
}

fn parse_ass(text: &str) -> (Option<String>, Vec<Cue>) {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut header = String::new();
    let mut cues = Vec::new();
    let mut in_events = false;
    // Posicións das columnas segundo a liña «Format:» dos eventos.
    let mut cols: Vec<String> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("[events]") {
            in_events = true;
            header.push_str(line);
            header.push('\n');
            continue;
        }
        if t.starts_with('[') && t.ends_with(']') {
            in_events = false;
        }
        if !in_events {
            header.push_str(line);
            header.push('\n');
            continue;
        }
        if let Some(f) = t.strip_prefix("Format:") {
            cols = f.split(',').map(|c| c.trim().to_ascii_lowercase()).collect();
            header.push_str(line);
            header.push('\n');
        } else if let Some(d) = t.strip_prefix("Dialogue:") {
            if cols.is_empty() {
                continue;
            }
            let fields: Vec<&str> = d.trim_start().splitn(cols.len(), ',').collect();
            if fields.len() < cols.len() {
                continue;
            }
            let get = |name: &str| cols.iter().position(|c| c == name).and_then(|i| fields.get(i)).map(|s| s.trim().to_string());
            let (Some(start), Some(end), Some(raw)) = (get("start").and_then(|s| parse_clock(&s)), get("end").and_then(|s| parse_clock(&s)), get("text")) else { continue };
            // O texto vai tal cal (pode levar comas); os espazos do final non importan.
            let raw_text = fields[cols.iter().position(|c| c == "text").unwrap_or(fields.len() - 1)].to_string();
            let _ = raw;
            let body = canonical_ass(&raw_text);
            if body.trim().is_empty() {
                continue;
            }
            cues.push(Cue {
                start,
                end,
                text: body,
                ass: Some(AssLine {
                    layer: get("layer").or_else(|| get("marked")).unwrap_or_else(|| "0".into()),
                    style: get("style").unwrap_or_else(|| "Default".into()),
                    name: get("name").or_else(|| get("actor")).unwrap_or_default(),
                    margins: [get("marginl").unwrap_or_else(|| "0".into()), get("marginr").unwrap_or_else(|| "0".into()), get("marginv").unwrap_or_else(|| "0".into())],
                    effect: get("effect").unwrap_or_default(),
                    raw_text,
                }),
            });
        }
        // As liñas «Comment:» e demais non son falas: non entran.
    }
    (if cues.is_empty() && !text.contains("[Events]") { None } else { Some(header) }, cues)
}

/// Le uns subtítulos (SRT, WebVTT ou ASS/SSA), en calquera das codificacións habituais.
pub fn read_subtitle(path: &Path) -> Result<Subtitle> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    let ext = crate::formats::extension_of(path).unwrap_or_default();
    let format = Format::from_ext(&ext).ok_or_else(|| EngineError::UnknownSourceFormat(path.to_path_buf()))?;
    // Uns subtítulos de máis de 20 MB non son uns subtítulos.
    if std::fs::metadata(path)?.len() > 20_000_000 {
        return Err(EngineError::IntegrityCheck("Este ficheiro é demasiado grande para ser de subtítulos.".into()));
    }
    let (text, encoding) = decode(&std::fs::read(path)?);
    let (cues, ass_header) = match format {
        Format::Ass => {
            let (h, c) = parse_ass(&text);
            (c, h)
        }
        f => (parse_blocks(&text, f), None),
    };
    if cues.is_empty() {
        return Err(EngineError::IntegrityCheck("Non atopei ningunha fala neste ficheiro: non parecen uns subtítulos válidos.".into()));
    }
    Ok(Subtitle { format, cues, ass_header, encoding })
}

// ------------------------------------------------------------------------------------------------ escribir

const ASS_DEFAULT_HEADER: &str = "[Script Info]\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\nWrapStyle: 0\nScaledBorderAndShadow: yes\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,56,&H00FFFFFF,&H000000FF,&H00000000,&H64000000,0,0,0,0,100,100,0,0,1,2,1,2,20,20,40,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

/// Os subtítulos no formato pedido, como texto (sempre se grava en UTF-8).
pub fn render(sub: &Subtitle, to: Format) -> String {
    let mut out = String::new();
    match to {
        Format::Srt => {
            for (i, c) in sub.cues.iter().enumerate() {
                out.push_str(&format!("{}\n{} --> {}\n{}\n\n", i + 1, clock(c.start, ','), clock(c.end, ','), c.text));
            }
        }
        Format::Vtt => {
            out.push_str("WEBVTT\n\n");
            for c in &sub.cues {
                out.push_str(&format!("{} --> {}\n{}\n\n", clock(c.start, '.'), clock(c.end, '.'), c.text));
            }
        }
        Format::Ass => {
            // De ASS a ASS consérvase a cabeceira e o estilo de cada fala; desde SRT ou VTT, un estilo sinxelo.
            match (&sub.ass_header, sub.format) {
                (Some(h), Format::Ass) => out.push_str(h),
                _ => out.push_str(ASS_DEFAULT_HEADER),
            }
            for c in &sub.cues {
                let (layer, style, name, m, effect, text) = match &c.ass {
                    Some(a) if sub.format == Format::Ass => (a.layer.as_str(), a.style.as_str(), a.name.as_str(), [a.margins[0].as_str(), a.margins[1].as_str(), a.margins[2].as_str()], a.effect.as_str(), a.raw_text.clone()),
                    _ => ("0", "Default", "", ["0", "0", "0"], "", ass_text(&c.text)),
                };
                out.push_str(&format!("Dialogue: {layer},{},{},{style},{name},{},{},{},{effect},{text}\n", ass_clock(c.start), ass_clock(c.end), m[0], m[1], m[2]));
            }
        }
    }
    out
}

fn write_text(output: &Path, overwrite: bool, text: &str) -> Result<PathBuf> {
    let mut out = AtomicOutput::new(output, overwrite)?;
    out.write_all(text.as_bytes())?;
    out.commit()
}

/// Pasa uns subtítulos a outro formato (SRT, WebVTT ou ASS). Pasar un SRT a SRT serve para gardalo en UTF-8.
pub fn convert_subtitle(input: &Path, output: &Path, overwrite: bool) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let to = Format::from_ext(&crate::formats::extension_of(output).unwrap_or_default())
        .ok_or_else(|| EngineError::UnsupportedConversion { from: "subtítulos".into(), to: crate::formats::extension_of(output).unwrap_or_default() })?;
    let sub = read_subtitle(input)?;
    write_text(output, overwrite, &render(&sub, to))
}

// ---------------------------------------------------------------------------------------- resincronizar

/// Cambio de tempo: `novo = tempo × factor + desprazamento`.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Retime {
    /// Milisegundos que se adianta (+) ou atrasa (−) todo.
    pub shift_ms: i64,
    /// Factor de velocidade dos tempos (1 = igual). Sirve para pasar de 23,976 a 25 fotogramas por segundo.
    pub factor: f64,
}

impl Retime {
    /// Os subtítulos foron feitos para un vídeo a `from_fps` e agora vaise usar nun a `to_fps`: o factor é `from / to`.
    pub fn from_fps(from_fps: f64, to_fps: f64) -> Result<Retime> {
        if !(from_fps.is_finite() && to_fps.is_finite() && (1.0..=240.0).contains(&from_fps) && (1.0..=240.0).contains(&to_fps)) {
            return Err(EngineError::IntegrityCheck("As cadencias non son válidas: teñen que estar entre 1 e 240 fotogramas por segundo.".into()));
        }
        Ok(Retime { shift_ms: 0, factor: from_fps / to_fps })
    }

    /// Dous puntos de referencia: a primeira fala (`first`, en ms, como está agora) debe quedar en `want_first` e a última (`last`) en
    /// `want_last`. Calcula o factor e o desprazamento que o fan.
    pub fn from_two_points(first: i64, last: i64, want_first: i64, want_last: i64) -> Result<Retime> {
        if last <= first {
            return Err(EngineError::IntegrityCheck("Fan falta polo menos dúas falas en momentos distintos para resincronizar con dous puntos.".into()));
        }
        if want_last <= want_first {
            return Err(EngineError::IntegrityCheck("A última fala ten que quedar despois da primeira.".into()));
        }
        let factor = (want_last - want_first) as f64 / (last - first) as f64;
        let shift = want_first as f64 - first as f64 * factor;
        Ok(Retime { shift_ms: shift.round() as i64, factor })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RetimeReport {
    pub output_path: PathBuf,
    pub cues: usize,
    /// Falas que quedaron antes do 0 e se descartaron.
    pub dropped: usize,
}

/// Aplica un cambio de tempo. As falas que acaban antes do principio descártanse; as que empezan antes quedan en 0.
pub fn apply_retime(sub: &mut Subtitle, r: &Retime) -> usize {
    let map = |t: i64| (t as f64 * r.factor).round() as i64 + r.shift_ms;
    let before = sub.cues.len();
    for c in &mut sub.cues {
        c.start = map(c.start);
        c.end = map(c.end);
    }
    sub.cues.retain(|c| c.end > 0);
    for c in &mut sub.cues {
        c.start = c.start.max(0);
    }
    before - sub.cues.len()
}

pub fn retime_subtitle(input: &Path, output: &Path, overwrite: bool, r: &Retime) -> Result<RetimeReport> {
    ensure_distinct(&[input], output)?;
    if !r.factor.is_finite() || !(0.5..=2.0).contains(&r.factor) || r.shift_ms.abs() > 86_400_000 {
        return Err(EngineError::IntegrityCheck("O cambio de tempo non é válido: o factor ten que estar entre 0,5 e 2 e o desprazamento nun día.".into()));
    }
    let to = Format::from_ext(&crate::formats::extension_of(output).unwrap_or_default()).ok_or_else(|| EngineError::UnknownSourceFormat(output.to_path_buf()))?;
    let mut sub = read_subtitle(input)?;
    let dropped = apply_retime(&mut sub, r);
    if sub.cues.is_empty() {
        return Err(EngineError::IntegrityCheck("Con ese desprazamento non quedaría ningunha fala: todas acabarían antes do principio.".into()));
    }
    let path = write_text(output, overwrite, &render(&sub, to))?;
    Ok(RetimeReport { output_path: path, cues: sub.cues.len(), dropped })
}

// ---------------------------------------------------------------------------------------------------- info

#[derive(Debug, Clone, serde::Serialize)]
pub struct SubtitleInfo {
    pub format: Format,
    pub cues: usize,
    /// Inicio da primeira fala e da última, e fin da última, en milisegundos.
    pub first_start: i64,
    pub last_start: i64,
    pub last_end: i64,
    pub encoding: &'static str,
}

pub fn subtitle_info(path: &Path) -> Result<SubtitleInfo> {
    let s = read_subtitle(path)?;
    let first = s.cues.iter().map(|c| c.start).min().unwrap_or(0);
    let last = s.cues.iter().map(|c| c.start).max().unwrap_or(0);
    let end = s.cues.iter().map(|c| c.end).max().unwrap_or(0);
    Ok(SubtitleInfo { format: s.format, cues: s.cues.len(), first_start: first, last_start: last, last_end: end, encoding: s.encoding })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRT: &str = "1\n00:00:01,000 --> 00:00:03,500\nOlá, mundo!\nSegunda liña con <i>cursiva</i>\n\n2\n00:00:05,250 --> 00:00:07,000\nNiño, ñoño, çà, €\n\n3\n00:01:00,000 --> 00:01:02,001\n<b>Última</b> fala\n";

    fn parse(text: &str, f: Format) -> Subtitle {
        let cues = match f {
            Format::Ass => parse_ass(text).1,
            f => parse_blocks(text, f),
        };
        Subtitle { format: f, cues, ass_header: if f == Format::Ass { parse_ass(text).0 } else { None }, encoding: "UTF-8" }
    }

    fn texts(s: &Subtitle) -> Vec<(i64, i64, String)> {
        s.cues.iter().map(|c| (c.start, c.end, c.text.clone())).collect()
    }

    #[test]
    fn os_tempos_le_se_con_calquera_separador_e_cantidade_de_cifras() {
        assert_eq!(parse_clock("00:00:01,000"), Some(1000));
        assert_eq!(parse_clock("01:02:03.456"), Some(3_723_456));
        assert_eq!(parse_clock("02:03.5"), Some(123_500));
        assert_eq!(parse_clock("0:00:01.50"), Some(1500));
        assert_eq!(parse_clock("00:00:61,000"), None);
        assert_eq!(parse_clock("basura"), None);
        assert_eq!(clock(3_723_456, ','), "01:02:03,456");
        assert_eq!(ass_clock(1999), "0:00:02.00");
    }

    #[test]
    fn srt_a_vtt_a_ass_e_volta_non_perde_nada() {
        let srt = parse(SRT, Format::Srt);
        assert_eq!(srt.cues.len(), 3);
        let vtt_text = render(&srt, Format::Vtt);
        assert!(vtt_text.starts_with("WEBVTT\n\n00:00:01.000 --> 00:00:03.500\n"), "{vtt_text}");
        let vtt = parse(&vtt_text, Format::Vtt);
        assert_eq!(texts(&vtt), texts(&srt));
        let ass_text = render(&vtt, Format::Ass);
        assert!(ass_text.contains("Dialogue: 0,0:00:01.00,0:00:03.50,Default,,0,0,0,,Olá, mundo!\\NSegunda liña con {\\i1}cursiva{\\i0}"), "{ass_text}");
        let ass = parse(&ass_text, Format::Ass);
        // ASS só garda centésimas: 62,001 s pasa a 62,00.
        let want: Vec<_> = texts(&srt).into_iter().map(|(a, b, t)| ((a + 5) / 10 * 10, (b + 5) / 10 * 10, t)).collect();
        assert_eq!(texts(&ass), want);
        let back = render(&ass, Format::Srt);
        assert!(back.contains("1\n00:00:01,000 --> 00:00:03,500\nOlá, mundo!\nSegunda liña con <i>cursiva</i>"), "{back}");
    }

    #[test]
    fn webvtt_con_notas_axustes_e_marcas_estrañas() {
        let vtt = "WEBVTT - cabeceira\n\nNOTE isto é unha nota\n\nSTYLE\n::cue { color: red }\n\nintro\n00:01.000 --> 00:02.500 line:90% align:center\n<v Ana>Ola <c.vermello>mundo</c>\n\n00:03.000 --> 00:04.000\n<i>Segunda</i>\n";
        let s = parse(vtt, Format::Vtt);
        assert_eq!(texts(&s), vec![(1000, 2500, "Ola mundo".to_string()), (3000, 4000, "<i>Segunda</i>".to_string())]);
    }

    #[test]
    fn ass_con_estilos_comas_no_texto_e_comentarios_consérvase_ao_gardar_ass() {
        let ass = "[Script Info]\nTitle: Proba\n\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Cartel,Impact\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nComment: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,isto non é unha fala\nDialogue: 1,0:00:02.00,0:00:04.00,Cartel,Ana,10,20,30,,{\\an8}Ola, {\\i1}mundo{\\i0}\\Nsegunda\n";
        let mut s = parse(ass, Format::Ass);
        assert_eq!(texts(&s), vec![(2000, 4000, "Ola, <i>mundo</i>\nsegunda".to_string())]);
        apply_retime(&mut s, &Retime { shift_ms: 1000, factor: 1.0 });
        let out = render(&s, Format::Ass);
        assert!(out.contains("Style: Cartel,Impact"), "{out}");
        assert!(out.contains("Dialogue: 1,0:00:03.00,0:00:05.00,Cartel,Ana,10,20,30,,{\\an8}Ola, {\\i1}mundo{\\i0}\\Nsegunda"), "{out}");
        assert!(!out.contains("Comment:"), "as notas non son falas");
        // A SRT vai sen posicións nin estilos.
        assert!(render(&s, Format::Srt).contains("Ola, <i>mundo</i>\nsegunda"));
    }

    #[test]
    fn detecta_a_codificacion_e_recupera_os_acentos() {
        let latin: Vec<u8> = "1\n00:00:01,000 --> 00:00:02,000\nNiño e ação, €\n".chars().map(|c| match c { 'ñ' => 0xF1, 'ç' => 0xE7, 'ã' => 0xE3, 'o' => b'o', '€' => 0x80, c => c as u8 }).collect();
        let (t, enc) = decode(&latin);
        assert_eq!(enc, "Windows-1252");
        assert!(t.contains("Niño e ação, €"), "{t}");
        let mut u16le = vec![0xFF, 0xFE];
        for u in "Olá niño".encode_utf16() {
            u16le.extend(u.to_le_bytes());
        }
        assert_eq!(decode(&u16le), ("Olá niño".to_string(), "UTF-16"));
        let mut bom = vec![0xEF, 0xBB, 0xBF];
        bom.extend("Olá".as_bytes());
        assert_eq!(decode(&bom), ("Olá".to_string(), "UTF-8"));
        assert_eq!(decode("Olá".as_bytes()).1, "UTF-8");
    }

    #[test]
    fn resincronizar_desprazando_cambiando_a_cadencia_e_con_dous_puntos() {
        let mut s = parse(SRT, Format::Srt);
        assert_eq!(apply_retime(&mut s, &Retime { shift_ms: -2000, factor: 1.0 }), 0);
        assert_eq!((s.cues[0].start, s.cues[0].end), (0, 1500), "o que empeza antes do 0 queda en 0");
        // Atrasar tanto que a primeira fala desaparece.
        let mut s = parse(SRT, Format::Srt);
        assert_eq!(apply_retime(&mut s, &Retime { shift_ms: -4000, factor: 1.0 }), 1);
        // De 23,976 a 25: os tempos encollen un 4,1 %.
        let r = Retime::from_fps(23.976, 25.0).unwrap();
        assert!((r.factor - 0.95904).abs() < 1e-4);
        let mut s = parse(SRT, Format::Srt);
        apply_retime(&mut s, &r);
        assert_eq!(s.cues[2].start, (60_000.0 * r.factor).round() as i64);
        // Dous puntos: a primeira fala (1 s) pasa a 3 s e a última (60 s) a 70 s.
        let r = Retime::from_two_points(1000, 60_000, 3000, 70_000).unwrap();
        let mut s = parse(SRT, Format::Srt);
        apply_retime(&mut s, &r);
        assert_eq!(s.cues[0].start, 3000);
        assert_eq!(s.cues[2].start, 70_000);
        assert!(Retime::from_two_points(5, 5, 1, 2).is_err() && Retime::from_two_points(1, 5, 9, 9).is_err());
        assert!(Retime::from_fps(0.0, 25.0).is_err() && Retime::from_fps(25.0, f64::NAN).is_err());
    }

    #[test]
    fn un_ficheiro_que_non_e_de_subtitulos_dá_erro_claro_e_non_escribe_nada() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("a.srt");
        std::fs::write(&bad, "isto non son uns subtítulos\nnin unha fala").unwrap();
        assert!(convert_subtitle(&bad, &dir.path().join("o.vtt"), false).is_err());
        assert!(!dir.path().join("o.vtt").exists());
        let ok = dir.path().join("b.srt");
        std::fs::write(&ok, SRT).unwrap();
        assert!(convert_subtitle(&ok, &ok, true).is_err(), "non pisa o orixinal");
        std::fs::write(dir.path().join("xa.vtt"), "x").unwrap();
        assert!(convert_subtitle(&ok, &dir.path().join("xa.vtt"), false).is_err());
        assert_eq!(std::fs::read(dir.path().join("xa.vtt")).unwrap(), b"x");
        assert!(retime_subtitle(&ok, &dir.path().join("r.srt"), false, &Retime { shift_ms: 0, factor: 5.0 }).is_err());
        assert!(retime_subtitle(&ok, &dir.path().join("r.srt"), false, &Retime { shift_ms: -999_999, factor: 1.0 }).is_err());
        let info = subtitle_info(&ok).unwrap();
        assert_eq!((info.cues, info.first_start, info.last_start, info.last_end), (3, 1000, 60_000, 62_001));
    }
}
