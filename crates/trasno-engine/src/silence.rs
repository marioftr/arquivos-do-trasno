//! Cortar silencios dun audio ou vídeo (entrevistas, podcasts, clases gravadas…).
//!
//! Dous pasos, como a normalización: primeiro mídese (FFmpeg `silencedetect` busca os tramos por baixo dun volume durante un mínimo
//! de tempo) e despois úsase ese mesmo resultado para quedar só cos tramos con son. A análise pódese ensinar ao usuario antes de
//! cortar nada. Os silencios non se quitan enteiros: queda un pouco (`padding`) á beira de cada corte para que as palabras non soen
//! cortadas de golpe. Nos vídeos os cortes van aliñados cos fotogramas, así que imaxe e son quedan sempre sincronizados.

use crate::error::{EngineError, Result};
use crate::ffmpeg::{self, check_success, run_ffmpeg};
use crate::integrity::ensure_distinct;
use std::path::{Path, PathBuf};

/// Que se considera un silencio e canto se deixa á beira.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SilenceOptions {
    /// Volume por baixo do cal hai silencio, en dB (−70 a −10). Máis preto de 0 = quita máis (tamén o son baixo).
    pub threshold_db: f64,
    /// Duración mínima dun silencio para cortalo, en segundos (0,1 a 30).
    pub min_silence: f64,
    /// Silencio que se deixa a cada lado de cada corte, en segundos (0 a 2).
    pub padding: f64,
}

impl Default for SilenceOptions {
    fn default() -> Self {
        SilenceOptions { threshold_db: -35.0, min_silence: 0.5, padding: 0.1 }
    }
}

fn check(o: &SilenceOptions) -> Result<()> {
    let ok = o.threshold_db.is_finite()
        && o.min_silence.is_finite()
        && o.padding.is_finite()
        && (-70.0..=-10.0).contains(&o.threshold_db)
        && (0.1..=30.0).contains(&o.min_silence)
        && (0.0..=2.0).contains(&o.padding);
    if ok {
        Ok(())
    } else {
        Err(EngineError::IntegrityCheck("Os valores para cortar silencios non son válidos: o volume vai de −70 a −10 dB, o silencio mínimo de 0,1 a 30 s e o que se deixa á beira de 0 a 2 s.".into()))
    }
}

/// O que se atopou nun ficheiro.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SilenceAnalysis {
    pub duration: f64,
    /// Tramos que se cortarían (xa sen o `padding`): `(inicio, fin)` en segundos.
    pub cuts: Vec<(f64, f64)>,
    /// Canto durará o resultado.
    pub result_duration: f64,
}

/// Tramos de silencio que lle saen a FFmpeg (`silence_start` / `silence_end`). Un silencio que chega ata o final non ten «end».
fn parse_silences(log: &str, duration: f64) -> Vec<(f64, f64)> {
    let num = |line: &str, key: &str| -> Option<f64> { line.split(key).nth(1)?.split_whitespace().next()?.trim_end_matches('|').parse::<f64>().ok() };
    let mut out = Vec::new();
    let mut open: Option<f64> = None;
    for line in log.lines() {
        if line.contains("silence_start:") {
            open = num(line, "silence_start:").map(|s| s.max(0.0));
        } else if line.contains("silence_end:") {
            if let (Some(s), Some(e)) = (open.take(), num(line, "silence_end:")) {
                out.push((s, e.min(duration)));
            }
        }
    }
    if let Some(s) = open {
        out.push((s, duration));
    }
    out.retain(|(s, e)| e > s);
    out
}

/// De silencios a cortes: cada silencio perde `padding` en cada punta (queda ese silencio ao carón do son) e os que quedan sen
/// nada que cortar desaparecen. Se hai un `grid` (fotogramas por segundo), os cortes aliñanse a el cara dentro, para que o que se
/// quede sexa sempre fotogramas enteiros.
fn cuts_from(silences: &[(f64, f64)], padding: f64, duration: f64, grid: Option<f64>) -> Vec<(f64, f64)> {
    let mut cuts: Vec<(f64, f64)> = Vec::new();
    for &(s, e) in silences {
        // O silencio que toca o principio ou o final non ten son por ese lado: non hai nada que deixar.
        let a = if s <= 0.001 { 0.0 } else { s + padding };
        let b = if e >= duration - 0.001 { duration } else { e - padding };
        let (a, b) = match grid {
            // Cara dentro do corte: o inicio sobe ao fotograma seguinte e o fin baixa ao anterior.
            Some(fps) if a > 0.0 || b < duration => (if a > 0.0 { (a * fps).ceil() / fps } else { 0.0 }, if b < duration { (b * fps).floor() / fps } else { duration }),
            _ => (a, b),
        };
        if b - a > 0.02 {
            cuts.push((a, b));
        }
    }
    cuts.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
    // Cortes que se tocan ou se solapan, un só.
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for c in cuts {
        match merged.last_mut() {
            Some(last) if c.0 <= last.1 + 1e-9 => last.1 = last.1.max(c.1),
            _ => merged.push(c),
        }
    }
    merged
}

/// Os tramos que se conservan: o complementario dos cortes.
fn keeps_from(cuts: &[(f64, f64)], duration: f64) -> Vec<(f64, f64)> {
    let mut keeps = Vec::new();
    let mut at = 0.0;
    for &(a, b) in cuts {
        if a > at + 1e-9 {
            keeps.push((at, a));
        }
        at = b;
    }
    if duration > at + 1e-9 {
        keeps.push((at, duration));
    }
    keeps
}

fn duration_of(info: &serde_json::Value) -> f64 {
    info["format"]["duration"].as_str().and_then(|d| d.parse::<f64>().ok()).filter(|d| d.is_finite() && *d > 0.0).unwrap_or(0.0)
}

/// Fotogramas por segundo do vídeo (para aliñar os cortes), se o ficheiro leva imaxe.
fn video_fps(info: &serde_json::Value) -> Option<f64> {
    let v = info["streams"].as_array()?.iter().find(|s| s["codec_type"] == "video" && s["disposition"]["attached_pic"] != 1)?;
    let rate = |k: &str| -> Option<f64> {
        let (n, d) = v[k].as_str()?.split_once('/')?;
        let (n, d): (f64, f64) = (n.parse().ok()?, d.parse().ok()?);
        (d > 0.0 && n > 0.0).then_some(n / d)
    };
    rate("avg_frame_rate").or_else(|| rate("r_frame_rate")).filter(|f| (1.0..=240.0).contains(f))
}

/// Primeiro paso: busca os silencios sen escribir nada.
pub fn analyze_silence(input: &Path, o: &SilenceOptions) -> Result<SilenceAnalysis> {
    check(o)?;
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    let info = ffmpeg::probe(input)?;
    let duration = duration_of(&info);
    if duration <= 0.0 {
        return Err(EngineError::IntegrityCheck("Non puiden saber canto dura o ficheiro.".into()));
    }
    if !info["streams"].as_array().is_some_and(|l| l.iter().any(|s| s["codec_type"] == "audio")) {
        return Err(EngineError::IntegrityCheck("Este ficheiro non ten son: non hai silencios que cortar.".into()));
    }
    let sink = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let args: Vec<std::ffi::OsString> = vec![
        "-i".into(),
        input.as_os_str().to_owned(),
        "-map".into(),
        "0:a:0".into(),
        "-af".into(),
        format!("silencedetect=noise={}dB:d={}", o.threshold_db, o.min_silence).into(),
        "-f".into(),
        "null".into(),
        sink.into(),
    ];
    let out = run_ffmpeg(&args)?;
    check_success(&out)?;
    let silences = parse_silences(&String::from_utf8_lossy(&out.stderr), duration);
    let cuts = cuts_from(&silences, o.padding, duration, video_fps(&info));
    let removed: f64 = cuts.iter().map(|(a, b)| b - a).sum();
    Ok(SilenceAnalysis { duration, result_duration: (duration - removed).max(0.0), cuts })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SilenceReport {
    pub output_path: PathBuf,
    pub before: f64,
    pub after: f64,
    /// Cantos silencios se cortaron.
    pub cuts: usize,
}

/// Corta os silencios: o resultado só leva os tramos con son, un detrás doutro. Escríbese sempre nun ficheiro novo.
pub fn cut_silence(input: &Path, output: &Path, overwrite: bool, o: &SilenceOptions) -> Result<SilenceReport> {
    check(o)?;
    ensure_distinct(&[input], output)?;
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    ffmpeg::set_progress_phase(0.0, 0.15);
    let analysis = analyze_silence(input, o);
    ffmpeg::set_progress_phase(0.15, 0.85);
    let result = analysis.and_then(|a| {
        if a.cuts.is_empty() {
            return Err(EngineError::IntegrityCheck("Con estes valores non hai silencios que cortar. Proba a subir o volume de corte ou a baixar o silencio mínimo.".into()));
        }
        if a.result_duration < 0.2 {
            return Err(EngineError::IntegrityCheck("Con estes valores todo o ficheiro conta como silencio: non quedaría nada. Baixa o volume de corte.".into()));
        }
        let info = ffmpeg::probe(input)?;
        let fps = video_fps(&info);
        let keeps = keeps_from(&a.cuts, a.duration);
        // O filtro vai nun ficheiro: con centos de cortes a liña de ordes non chegaría.
        // Os cortes caen xusto sobre o tempo dun fotograma, onde os decimais poden facer que un fotograma entre ou non: córrese medio
        // fotograma cara atrás para que a comparación caia sempre no medio entre dous.
        let half = fps.map_or(0.0, |f| 0.5 / f);
        let expr = keeps.iter().map(|(s, e)| format!("gte(t,{:.6})*lt(t,{:.6})", s - half, e - half)).collect::<Vec<_>>().join("+");
        let dir = output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(dir)?;
        let script = dir.join(format!(".trasno-silence-{}.txt", uuid::Uuid::new_v4()));
        // Hai ficheiros cuxos tempos non empezan en 0 (gravacións de TV, algúns MP4): contámolos desde o principio do ficheiro, que é
        // desde onde mide `silencedetect`.
        let start = info["format"]["start_time"].as_str().and_then(|v| v.parse::<f64>().ok()).filter(|v| v.is_finite()).unwrap_or(0.0);
        let audio = audio_graph(&keeps, a.duration, start);
        let graph = match fps {
            Some(f) => format!("[0:v:0]setpts=PTS-{start:.6}/TB,select='{expr}',setpts=N/({f})/TB[v];{audio}"),
            None => audio,
        };
        std::fs::write(&script, graph)?;
        let mut args: Vec<String> = vec!["-filter_complex_script".into(), script.to_string_lossy().to_string()];
        if fps.is_some() {
            args.extend(["-map".into(), "[v]".into()]);
            args.extend(video_encoder_args(output));
        }
        args.extend(["-map".into(), "[a]".into()]);
        args.extend(ffmpeg::keep_audio_args(input, output));
        // Sen isto FFmpeg usa a barra con canto dura o orixinal, e acabaría antes de tempo.
        ffmpeg::set_expected_duration(Some(a.result_duration));
        let made = ffmpeg::convert_media(input, output, overwrite, &args);
        ffmpeg::set_expected_duration(None);
        let _ = std::fs::remove_file(&script);
        let made = made?;
        Ok(SilenceReport { output_path: made, before: a.duration, after: a.result_duration, cuts: a.cuts.len() })
    });
    ffmpeg::set_progress_phase(0.0, 1.0);
    result
}

/// O son: `asegment` parte o fluxo nos puntos de corte (sen ser exacto só ao fotograma: corta á mostra) e `concat` xunta só os
/// anacos que se conservan; os outros van a `anullsink`. Vai en fluxo, sen gardar nada en memoria. (`aselect` non serve: non
/// quita nada nalgunhas versións de FFmpeg.)
fn audio_graph(keeps: &[(f64, f64)], duration: f64, start: f64) -> String {
    // Puntos onde se parte: os extremos dos tramos que se conservan, menos o principio e o final do ficheiro.
    let mut cuts: Vec<f64> = keeps.iter().flat_map(|&(a, b)| [a, b]).filter(|t| *t > 1e-6 && *t < duration - 1e-6).collect();
    cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let mut bounds = vec![0.0];
    bounds.extend(&cuts);
    bounds.push(duration);
    // Cada anaco entre dous puntos conséntase se cae dentro dun tramo que se conserva.
    let kept = |i: usize| {
        let mid = (bounds[i] + bounds[i + 1]) / 2.0;
        keeps.iter().any(|&(a, b)| mid >= a && mid <= b)
    };
    let n = bounds.len() - 1;
    let labels: String = (0..n).map(|i| format!("[s{i}]")).collect();
    let times = cuts.iter().map(|t| format!("{t:.6}")).collect::<Vec<_>>().join("|");
    let zero = format!("[0:a:0]asetpts=PTS-{start:.6}/TB");
    let mut g = if n == 1 { format!("{zero},anull[s0]") } else { format!("{zero},asegment=timestamps={times}{labels}") };
    let mut joined = String::new();
    let mut count = 0;
    for i in 0..n {
        if kept(i) {
            // Cada anaco leva aínda o seu tempo orixinal: `concat` agarda que cada un comece en 0.
            g.push_str(&format!(";[s{i}]asetpts=PTS-STARTPTS[k{i}]"));
            joined.push_str(&format!("[k{i}]"));
            count += 1;
        } else {
            g.push_str(&format!(";[s{i}]anullsink"));
        }
    }
    g.push_str(&format!(";{joined}concat=n={count}:v=0:a=1[a]"));
    g
}

/// Como se recodifica a imaxe: cortar fotogramas obriga a recodificar, así que sae con moita calidade (H.264 con CRF 18, ou VP9 en WebM).
pub(crate) fn video_encoder_args(output: &Path) -> Vec<String> {
    let ext = crate::formats::extension_of(output).unwrap_or_default();
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    match ext.as_str() {
        "webm" => s(&["-c:v", "libvpx-vp9", "-crf", "24", "-b:v", "0"]),
        "mp4" | "m4v" | "mov" | "mkv" => s(&["-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p"]),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_os_silencios_do_log_de_ffmpeg() {
        let log = "[silencedetect @ 0x1] silence_start: 2\n[silencedetect @ 0x1] silence_end: 4.5 | silence_duration: 2.5\n[silencedetect @ 0x1] silence_start: -0.0023\n[silencedetect @ 0x1] silence_end: 0.6 | silence_duration: 0.6\n[silencedetect @ 0x1] silence_start: 7.25\n";
        let v = parse_silences(log, 8.0);
        assert_eq!(v, vec![(2.0, 4.5), (0.0, 0.6), (7.25, 8.0)], "o último chega ata o final");
    }

    #[test]
    fn os_cortes_deixan_o_padding_e_non_o_deixan_nas_puntas() {
        // Silencio no medio: queda 0,1 a cada lado.
        let c = cuts_from(&[(2.0, 4.0)], 0.1, 10.0, None);
        assert_eq!(c, vec![(2.1, 3.9)]);
        // Silencio que toca o principio e o final: córtase ata a punta.
        let c = cuts_from(&[(0.0, 1.0), (9.0, 10.0)], 0.1, 10.0, None);
        assert_eq!(c, vec![(0.0, 0.9), (9.1, 10.0)]);
        // Un silencio tan curto que o padding o come, non se corta.
        assert!(cuts_from(&[(2.0, 2.15)], 0.1, 10.0, None).is_empty());
        // Co padding a 0 córtase enteiro.
        assert_eq!(cuts_from(&[(2.0, 4.0)], 0.0, 10.0, None), vec![(2.0, 4.0)]);
    }

    #[test]
    fn nos_videos_os_cortes_aliñanse_cara_dentro_aos_fotogramas() {
        let c = cuts_from(&[(2.0, 4.0)], 0.1, 10.0, Some(25.0));
        let (a, b) = c[0];
        assert!(a >= 2.1 && b <= 3.9, "cara dentro: {a} {b}");
        assert!(((a * 25.0).round() - a * 25.0).abs() < 1e-6 && ((b * 25.0).round() - b * 25.0).abs() < 1e-6, "sobre a grella de fotogramas");
    }

    #[test]
    fn os_tramos_que_quedan_son_o_complementario() {
        let k = keeps_from(&[(2.0, 4.0), (6.0, 7.0)], 10.0);
        assert_eq!(k, vec![(0.0, 2.0), (4.0, 6.0), (7.0, 10.0)]);
        assert_eq!(keeps_from(&[(0.0, 3.0)], 10.0), vec![(3.0, 10.0)]);
        assert_eq!(keeps_from(&[(5.0, 10.0)], 10.0), vec![(0.0, 5.0)]);
        // Cortes que se tocan, un só.
        assert_eq!(cuts_from(&[(1.0, 2.0), (2.0, 3.0)], 0.0, 10.0, None), vec![(1.0, 3.0)]);
    }

    #[test]
    fn o_filtro_de_son_parte_nos_cortes_e_xunta_so_o_que_queda() {
        let g = audio_graph(&[(0.0, 2.1), (3.9, 7.1), (8.9, 10.0)], 10.0, 0.0);
        assert!(g.contains("asegment=timestamps=2.100000|3.900000|7.100000|8.900000"), "{g}");
        assert!(g.contains("[k0][k2][k4]concat=n=3:v=0:a=1[a]") && g.contains("[s2]asetpts=PTS-STARTPTS[k2]"), "{g}");
        assert!(g.contains(";[s1]anullsink") && g.contains(";[s3]anullsink"), "{g}");
        // Corte ao principio: o primeiro anaco non se conserva.
        let g = audio_graph(&[(3.0, 10.0)], 10.0, 0.0);
        assert!(g.contains("[s0]anullsink") && g.contains("[k1]concat=n=1"), "{g}");
    }

    #[test]
    fn valores_malos_rexeitanse() {
        for bad in [
            SilenceOptions { threshold_db: 0.0, ..Default::default() },
            SilenceOptions { threshold_db: f64::NAN, ..Default::default() },
            SilenceOptions { min_silence: 0.0, ..Default::default() },
            SilenceOptions { padding: -1.0, ..Default::default() },
            SilenceOptions { padding: 5.0, ..Default::default() },
        ] {
            assert!(check(&bad).is_err(), "{bad:?}");
        }
        assert!(check(&SilenceOptions::default()).is_ok());
    }
}
