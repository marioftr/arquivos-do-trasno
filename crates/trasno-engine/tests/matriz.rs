//! Matriz de formatos: cada operación de edición, silencios e subtítulos sobre cada contedor de audio e vídeo que admite a app.
//! Comproba que o resultado se decodifica ata o final sen erros, que dura o agardado e que o orixinal non cambia nin un byte.
//! Lenta (varios minutos): `cargo test -p trasno-engine --test matriz -- --include-ignored --nocapture`.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use trasno_engine::edit_plan::{apply_plan, EditPlan};
use trasno_engine::ffmpeg;
use trasno_engine::silence::{cut_silence, SilenceOptions};
use trasno_engine::subtitle_video as sv;

fn ff(args: &[&str]) -> Result<(), String> {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-y"]).args(args).output().unwrap();
    if o.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).lines().last().unwrap_or("").to_string())
    }
}

/// Decodifica o ficheiro enteiro: (ben, queixas de FFmpeg, segundos realmente decodificados).
fn decode(p: &Path) -> (bool, String, f64) {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-stats", "-i", p.to_str().unwrap(), "-f", "null", "-"]).output().unwrap();
    let log = String::from_utf8_lossy(&o.stderr).replace(char::from(13), "
");
    let complaints: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty() && !l.contains("time=") && !l.starts_with("size=") && !l.starts_with("frame=")).collect();
    let secs = log.rsplit("time=").next().and_then(|t| t.split_whitespace().next()).map(|t| t.split(':').fold(0.0, |a, x| a * 60.0 + x.parse::<f64>().unwrap_or(0.0))).unwrap_or(0.0);
    (o.status.success(), complaints.join(" | "), secs)
}

/// O resultado decodifica limpo (se a orixe tamén o facía: algunhas mostras de FFmpeg xa traen avisos propios).
fn decodes(src: &Path, out: &Path) -> Result<(), String> {
    let (ok, complaints, _) = decode(out);
    let (_, src_complaints, _) = decode(src);
    if ok && (complaints.is_empty() || !src_complaints.is_empty()) {
        Ok(())
    } else {
        Err(format!("non decodifica limpo: {}", complaints.chars().take(120).collect::<String>()))
    }
}

/// Duración do que se decodifica de verdade (a das cabeceiras pode mentir, sobre todo no AAC en bruto).
fn duration(p: &Path) -> f64 {
    decode(p).2
}

fn has(p: &Path, kind: &str) -> bool {
    ffmpeg::probe(p).unwrap()["streams"].as_array().unwrap().iter().any(|s| s["codec_type"] == kind)
}

fn sha(p: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(p).unwrap()))
}

/// Son de 10 s con dous silencios (de 2 a 4 e de 7 a 9).
const TONE: &str = "sine=f=440:r=44100:d=10,volume='if(between(t,2,4)+between(t,7,9),0,1)':eval=frame";

const SRT: &str = "1\n00:00:01,000 --> 00:00:03,000\nOlá, Galicia: ñ á é í ó ú\n\n2\n00:00:04,000 --> 00:00:06,000\nSegunda fala\n";

struct Matrix {
    rows: Vec<(String, Result<(), String>)>,
}
impl Matrix {
    fn add(&mut self, name: String, r: Result<(), String>) {
        println!("  {} {}{}", if r.is_ok() { "✓" } else { "✗" }, name, r.as_ref().err().map(|e| format!(" → {e}")).unwrap_or_default());
        self.rows.push((name, r));
    }
    fn finish(self) {
        let bad: Vec<_> = self.rows.iter().filter(|r| r.1.is_err()).map(|r| format!("{}: {}", r.0, r.1.as_ref().unwrap_err())).collect();
        assert!(bad.is_empty(), "{} de {} casos fallan:\n{}", bad.len(), self.rows.len(), bad.join("\n"));
    }
}

/// Executa `op` e comproba o resultado: decodifica, dura o agardado, ten o que se pide e a orixe non cambiou.
fn verify(src: &Path, out: &Path, want_dur: Option<f64>, want_video: bool, want_audio: bool, op: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    let before = sha(src);
    op()?;
    if sha(src) != before {
        return Err("o orixinal cambiou".into());
    }
    if !out.exists() {
        return Err("non saíu ficheiro".into());
    }
    decodes(src, out)?;
    if let Some(d) = want_dur {
        let got = duration(out);
        if (got - d).abs() > 0.45 {
            return Err(format!("dura {got:.2} s e agardábase {d:.2} s"));
        }
    }
    if want_video && !has(out, "video") {
        return Err("perdeu a imaxe".into());
    }
    if want_audio && !has(out, "audio") {
        return Err("perdeu o son".into());
    }
    Ok(())
}

fn tmp_out(dir: &Path, tag: &str, ext: &str) -> PathBuf {
    dir.join(format!("{tag}.{ext}"))
}

#[test]
#[ignore = "lenta: probas con todos os contedores"]
fn audio_todos_os_formatos() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let mut m = Matrix { rows: vec![] };
    for ext in ["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "wma", "aiff"] {
        let src = dir.path().join(format!("src.{ext}"));
        if let Err(e) = ff(&["-f", "lavfi", "-i", TONE, src.to_str().unwrap()]) {
            m.add(format!("{ext}: crear mostra"), Err(e));
            continue;
        }
        let o = tmp_out(dir.path(), &format!("cut-{ext}"), ext);
        m.add(format!("{ext}: cortar 2-8 s"), verify(&src, &o, Some(6.0), false, true, || apply_plan(&src, &o, true, &EditPlan { start: 2.0, end: Some(8.0), ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())));
        let o = tmp_out(dir.path(), &format!("fx-{ext}"), ext);
        m.add(
            format!("{ext}: velocidade 1,5 + volume + graves + esvaecer"),
            verify(&src, &o, Some(10.0 / 1.5), false, true, || apply_plan(&src, &o, true, &EditPlan { speed: 1.5, volume: 0.7, bass: 3.0, fade_out: 0.5, ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())),
        );
        let o = tmp_out(dir.path(), &format!("rev-{ext}"), ext);
        m.add(format!("{ext}: ao revés + ruído"), verify(&src, &o, Some(10.0), false, true, || apply_plan(&src, &o, true, &EditPlan { reverse: true, denoise: true, ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())));
        let o = tmp_out(dir.path(), &format!("sil-{ext}"), ext);
        m.add(format!("{ext}: cortar silencios"), verify(&src, &o, Some(6.4), false, true, || cut_silence(&src, &o, true, &SilenceOptions::default()).map(|_| ()).map_err(|e| e.to_string())));
    }
    m.finish();
}

/// Mostra de vídeo no contedor, deixando a FFmpeg escoller os códecs habituais (ou os dados).
fn make_video(dir: &Path, ext: &str) -> Result<PathBuf, String> {
    let p = dir.join(format!("v.{ext}"));
    let mut a: Vec<&str> = vec!["-f", "lavfi", "-i", "testsrc=s=320x240:r=25:d=10", "-f", "lavfi", "-i", TONE, "-shortest", "-pix_fmt", "yuv420p"];
    match ext {
        "mp4" | "mov" | "mkv" => a.extend(["-c:v", "libx264", "-c:a", "aac"]),
        "webm" => a.extend(["-c:v", "libvpx-vp9", "-deadline", "realtime", "-cpu-used", "8", "-c:a", "libopus"]),
        "avi" => a.extend(["-c:v", "mpeg4", "-c:a", "mp3"]),
        "flv" => a.extend(["-c:v", "flv", "-c:a", "mp3"]),
        "wmv" => a.extend(["-c:v", "wmv2", "-c:a", "wmav2"]),
        "mpg" | "mpeg" => a.extend(["-c:v", "mpeg2video", "-c:a", "mp2"]),
        _ => {}
    }
    a.push(p.to_str().unwrap());
    ff(&a)?;
    Ok(p)
}

#[test]
#[ignore = "lenta: probas con todos os contedores"]
fn video_edicion_e_silencios_en_todos_os_contedores() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let mut m = Matrix { rows: vec![] };
    for ext in ["mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "mpg", "mpeg"] {
        let src = match make_video(dir.path(), ext) {
            Ok(p) => p,
            Err(e) => {
                m.add(format!("{ext}: crear mostra"), Err(e));
                continue;
            }
        };
        let o = tmp_out(dir.path(), &format!("cut-{ext}"), ext);
        m.add(format!("{ext}: cortar 2-8 s"), verify(&src, &o, Some(6.0), true, true, || apply_plan(&src, &o, true, &EditPlan { start: 2.0, end: Some(8.0), ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())));
        let o = tmp_out(dir.path(), &format!("spd-{ext}"), ext);
        m.add(format!("{ext}: velocidade 2x + volume"), verify(&src, &o, Some(5.0), true, true, || apply_plan(&src, &o, true, &EditPlan { speed: 2.0, volume: 0.6, ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())));
        let o = tmp_out(dir.path(), &format!("snd-{ext}"), ext);
        m.add(format!("{ext}: só son (graves + eco)"), verify(&src, &o, Some(10.0), true, true, || apply_plan(&src, &o, true, &EditPlan { bass: 4.0, echo_ms: 200.0, ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())));
        let o = tmp_out(dir.path(), &format!("mute-{ext}"), ext);
        m.add(format!("{ext}: mudo"), verify(&src, &o, Some(10.0), true, false, || apply_plan(&src, &o, true, &EditPlan { mute: true, ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())));
        let o = tmp_out(dir.path(), &format!("sil-{ext}"), ext);
        m.add(format!("{ext}: cortar silencios"), verify(&src, &o, Some(6.4), true, true, || cut_silence(&src, &o, true, &SilenceOptions::default()).map(|_| ()).map_err(|e| e.to_string())));
    }
    m.finish();
}

#[test]
#[ignore = "lenta: probas con todos os contedores"]
fn subtitulos_en_todos_os_contedores() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let subs = dir.path().join("s.srt");
    std::fs::write(&subs, SRT).unwrap();
    let mut m = Matrix { rows: vec![] };
    const KEEPS: [&str; 5] = ["mp4", "m4v", "mov", "mkv", "webm"];
    for ext in ["mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "mpg"] {
        let src = match make_video(dir.path(), ext) {
            Ok(p) => p,
            Err(e) => {
                m.add(format!("{ext}: crear mostra"), Err(e));
                continue;
            }
        };
        // Igual ca na app: o contedor orixinal se admite subtítulos; se non, MKV para incrustar e MP4 para queimar.
        let embed_ext = if KEEPS.contains(&ext) { ext } else { "mkv" };
        let o = tmp_out(dir.path(), &format!("emb-{ext}"), embed_ext);
        m.add(
            format!("{ext}: incrustar → {embed_ext}"),
            verify(&src, &o, Some(10.0), true, true, || sv::embed(&src, &subs, &o, true, "glg").map(|_| ()).map_err(|e| e.to_string())).and_then(|_| {
                let tracks = sv::list_tracks(&o).map_err(|e| e.to_string())?;
                if tracks.len() != 1 || !tracks[0].text_based {
                    return Err(format!("pistas: {tracks:?}"));
                }
                let ex = tmp_out(dir.path(), &format!("ext-{ext}"), "srt");
                sv::extract_track(&o, tracks[0].position, &ex, true).map_err(|e| e.to_string())?;
                let txt = std::fs::read_to_string(&ex).map_err(|e| e.to_string())?;
                if txt.contains("Olá, Galicia: ñ á é í ó ú") && txt.contains("Segunda fala") {
                    Ok(())
                } else {
                    Err(format!("o texto extraído non coincide: {txt:?}"))
                }
            }),
        );
        let burn_ext = if KEEPS.contains(&ext) { ext } else { "mp4" };
        let o = tmp_out(dir.path(), &format!("burn-{ext}"), burn_ext);
        m.add(format!("{ext}: queimar → {burn_ext}"), verify(&src, &o, Some(10.0), true, true, || sv::burn(&src, &subs, &o, true).map(|_| ()).map_err(|e| e.to_string())));
    }
    m.finish();
}

#[test]
#[ignore = "lenta: probas con todos os contedores"]
fn rutas_con_acentos_apostrofos_e_corchetes() {
    if !ffmpeg::is_available() {
        return;
    }
    let base = tempfile::tempdir().unwrap();
    let dir = base.path().join("Meus vídeos (copia) [1] & máis");
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("o vídeo d'Ana, 100% ñ.mp4");
    ff(&["-f", "lavfi", "-i", "testsrc=s=320x240:r=25:d=6", "-f", "lavfi", "-i", "sine=f=440:r=44100:d=6", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", src.to_str().unwrap()]).unwrap();
    let subs = dir.join("lendas ñ's & máis [galego].srt");
    std::fs::write(&subs, SRT).unwrap();
    let mut m = Matrix { rows: vec![] };
    let o = dir.join("queimado d'Ana [1].mp4");
    m.add("queimar con rutas raras".into(), verify(&src, &o, Some(6.0), true, true, || sv::burn(&src, &subs, &o, true).map(|_| ()).map_err(|e| e.to_string())));
    let o = dir.join("incrustado d'Ana [1].mkv");
    m.add("incrustar con rutas raras".into(), verify(&src, &o, Some(6.0), true, true, || sv::embed(&src, &subs, &o, true, "glg").map(|_| ()).map_err(|e| e.to_string())));
    let o = dir.join("editado d'Ana (1).mp4");
    m.add("editar con rutas raras".into(), verify(&src, &o, Some(3.0), true, true, || apply_plan(&src, &o, true, &EditPlan { start: 1.0, end: Some(4.0), ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())));
    m.finish();
}

#[test]
#[ignore = "lenta: probas con todos os contedores"]
fn subtitulos_de_entrada_en_varios_formatos_e_codificacions() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = make_video(dir.path(), "mp4").unwrap();
    let mkv = make_video(dir.path(), "mkv").unwrap();
    let mut m = Matrix { rows: vec![] };
    let vtt = "WEBVTT\n\n00:00:01.000 --> 00:00:03.000\nOlá ñ\n\n00:00:04.000 --> 00:00:06.000\nOutra\n";
    let ass = "[Script Info]\nScriptType: v4.00+\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,28,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,Ola ñ\n";
    // SRT en Windows-1252 e en UTF-16 con BOM (como os deixan algúns programas antigos).
    let latin1: Vec<u8> = SRT.chars().map(|c| if (c as u32) < 256 { c as u8 } else { b'?' }).collect();
    let mut utf16 = vec![0xFF, 0xFE];
    for u in SRT.encode_utf16() {
        utf16.extend(u.to_le_bytes());
    }
    let inputs: Vec<(&str, Vec<u8>)> = vec![("a.vtt", vtt.as_bytes().to_vec()), ("b.ass", ass.as_bytes().to_vec()), ("c.srt", latin1), ("d.srt", utf16), ("e.srt", [&[0xEF, 0xBB, 0xBF][..], SRT.as_bytes()].concat())];
    for (name, bytes) in inputs {
        let subs = dir.path().join(name);
        std::fs::write(&subs, bytes).unwrap();
        for (video, ext) in [(&src, "mp4"), (&mkv, "mkv")] {
            let o = dir.path().join(format!("o-{name}.{ext}"));
            m.add(
                format!("incrustar {name} en {ext}"),
                verify(video, &o, Some(10.0), true, true, || sv::embed(video, &subs, &o, true, "").map(|_| ()).map_err(|e| e.to_string())).and_then(|_| {
                    let t = sv::list_tracks(&o).map_err(|e| e.to_string())?;
                    if t.len() == 1 && t[0].text_based {
                        Ok(())
                    } else {
                        Err(format!("pistas {t:?}"))
                    }
                }),
            );
        }
        let o = dir.path().join(format!("q-{name}.mp4"));
        m.add(format!("queimar {name}"), verify(&src, &o, Some(10.0), true, true, || sv::burn(&src, &subs, &o, true).map(|_| ()).map_err(|e| e.to_string())));
    }
    // Un MKV que xa leva unha pista: ao incrustar outra, a primeira segue alí.
    let first = dir.path().join("p.srt");
    std::fs::write(&first, SRT).unwrap();
    let one = dir.path().join("un.mkv");
    sv::embed(&mkv, &first, &one, true, "spa").unwrap();
    let two = dir.path().join("dous.mkv");
    m.add(
        "incrustar unha segunda pista".into(),
        verify(&one, &two, Some(10.0), true, true, || sv::embed(&one, &first, &two, true, "eng").map(|_| ()).map_err(|e| e.to_string())).and_then(|_| {
            let t = sv::list_tracks(&two).map_err(|e| e.to_string())?;
            if t.len() == 2 && t[0].language.as_deref() == Some("spa") && t[1].language.as_deref() == Some("eng") {
                Ok(())
            } else {
                Err(format!("pistas {t:?}"))
            }
        }),
    );
    m.finish();
}

#[test]
#[ignore = "lenta: probas con todos os contedores"]
fn igualar_volume_en_audio_e_video() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let mut m = Matrix { rows: vec![] };
    let target = ffmpeg::Loudness { lufs: -16.0, peak: -1.5, range: 11.0 };
    for ext in ["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "wma", "aiff", "mp4", "mkv", "webm", "avi", "mov"] {
        let src = if ["mp4", "mkv", "webm", "avi", "mov"].contains(&ext) {
            match make_video(dir.path(), ext) {
                Ok(p) => p,
                Err(e) => {
                    m.add(format!("{ext}: crear mostra"), Err(e));
                    continue;
                }
            }
        } else {
            let p = dir.path().join(format!("src.{ext}"));
            if let Err(e) = ff(&["-f", "lavfi", "-i", TONE, p.to_str().unwrap()]) {
                m.add(format!("{ext}: crear mostra"), Err(e));
                continue;
            }
            p
        };
        let o = dir.path().join(format!("norm-{ext}.{ext}"));
        let video = ["mp4", "mkv", "webm", "avi", "mov"].contains(&ext);
        m.add(
            format!("{ext}: igualar a −16 LUFS"),
            verify(&src, &o, Some(10.0), video, true, || apply_plan(&src, &o, true, &EditPlan { normalize: Some(target), ..Default::default() }).map(|_| ()).map_err(|e| e.to_string())).and_then(|_| {
                let l = ffmpeg::measure_loudness(&o, target).map_err(|e| e.to_string())?;
                // Os 20 % de silencio dixital baixan a media: o que importa é que non se disparou nin quedou en silencio.
                if l.lufs < -24.0 || l.lufs > -10.0 || l.peak > -0.5 {
                    Err(format!("volume fóra de rango: {} LUFS, pico {} dB", l.lufs, l.peak))
                } else {
                    Ok(())
                }
            }),
        );
    }
    m.finish();
}
