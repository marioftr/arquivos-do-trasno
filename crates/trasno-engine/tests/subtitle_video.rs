//! Subtítulos dentro do vídeo: incrustar, queimar e extraer, sobre ficheiros reais.

use std::path::{Path, PathBuf};
use trasno_engine::ffmpeg;
use trasno_engine::subtitle_video::{burn, embed, extract_track, list_tracks};

fn ff(args: &[&str]) {
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-y"]).args(args).status().unwrap();
    assert!(st.success());
}

fn video(dir: &Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    ff(&["-f", "lavfi", "-i", "testsrc=s=640x360:r=25:d=4", "-f", "lavfi", "-i", "sine=f=440:r=44100:d=4", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest", p.to_str().unwrap()]);
    p
}

/// Subtítulos antigos en Windows-1252 («Niño» co ñ = 0xF1).
fn srt_latin1(dir: &Path) -> PathBuf {
    let p = dir.join("a.srt");
    let bytes = b"1\n00:00:00,500 --> 00:00:03,000\nOl\xE1, Ni\xF1o\n\n2\n00:00:03,100 --> 00:00:03,900\nAdeus\n\n".to_vec();
    std::fs::write(&p, bytes).unwrap();
    p
}

fn gray_frame(p: &Path, at: &str) -> Vec<u8> {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-ss", at, "-i", p.to_str().unwrap(), "-frames:v", "1", "-pix_fmt", "gray", "-f", "rawvideo", "-"])
        .output()
        .unwrap();
    o.stdout
}

#[test]
fn incrustar_e_sacar_de_volta_conserva_o_texto_en_utf8_e_non_toca_a_imaxe() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let v = video(dir.path(), "v.mp4");
    let s = srt_latin1(dir.path());
    let out = dir.path().join("v-sub.mp4");
    let r = embed(&v, &s, &out, false, "glg").unwrap();
    assert_eq!(r.cues, 2);
    let tracks = list_tracks(&out).unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].codec, "mov_text");
    assert_eq!(tracks[0].language.as_deref(), Some("glg"));
    // A imaxe copiouse: mesmos fotogramas e mesmo códec.
    let (a, b) = (ffmpeg::probe(&v).unwrap(), ffmpeg::probe(&out).unwrap());
    let vid = |j: &serde_json::Value| j["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == "video").unwrap().clone();
    assert_eq!(vid(&a)["nb_frames"], vid(&b)["nb_frames"]);
    // Sacalos: saen en UTF-8 co «ñ» ben.
    let back = dir.path().join("fora.srt");
    let e = extract_track(&out, 0, &back, false).unwrap();
    assert_eq!(e.cues, 2);
    let text = std::fs::read_to_string(&back).unwrap();
    assert!(text.contains("Olá, Niño"), "UTF-8 correcto: {text}");
    // En MKV con ASS mantense o formato.
    let k = dir.path().join("v.mkv");
    ff(&["-i", v.to_str().unwrap(), "-c", "copy", k.to_str().unwrap()]);
    let ass = dir.path().join("a.ass");
    trasno_engine::subtitles::convert_subtitle(&s, &ass, false).unwrap();
    let ko = dir.path().join("k-sub.mkv");
    embed(&k, &ass, &ko, false, "").unwrap();
    assert_eq!(list_tracks(&ko).unwrap()[0].codec, "ass");
    // Unha segunda pista engádese á primeira.
    let ko2 = dir.path().join("k-sub2.mkv");
    embed(&ko, &s, &ko2, false, "spa").unwrap();
    let t2 = list_tracks(&ko2).unwrap();
    assert_eq!(t2.len(), 2);
    assert_eq!(t2[1].language.as_deref(), Some("spa"));
}

#[test]
fn queimar_cambia_a_imaxe_onde_hai_fala_e_non_onde_non_hai_e_conserva_o_son() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let v = video(dir.path(), "v.mp4");
    let s = srt_latin1(dir.path());
    let out = dir.path().join("v-queimado.mp4");
    let r = match burn(&v, &s, &out, false) {
        Ok(r) => r,
        // Un FFmpeg sen libass non pode queimar: o erro ten que ser claro.
        Err(e) => {
            assert!(format!("{e}").contains("libass"), "{e}");
            return;
        }
    };
    assert_eq!(r.cues, 2);
    assert!(list_tracks(&out).unwrap().is_empty(), "queimados: non queda pista que apagar");
    let (orig, burned) = (gray_frame(&v, "1"), gray_frame(&out, "1"));
    let diff = orig.iter().zip(&burned).filter(|(a, b)| (**a as i16 - **b as i16).abs() > 40).count();
    assert!(diff > 300, "co subtítulo a imaxe cambia ({diff} píxeles)");
    let (orig2, burned2) = (gray_frame(&v, "3.5"), gray_frame(&out, "3.5"));
    let diff2 = orig2.iter().zip(&burned2).filter(|(a, b)| (**a as i16 - **b as i16).abs() > 40).count();
    assert!(diff2 > 100, "a segunda fala tamén se ve ({diff2})");
    let probe = ffmpeg::probe(&out).unwrap();
    assert!(probe["streams"].as_array().unwrap().iter().any(|s| s["codec_type"] == "audio"), "o son queda");
}

#[test]
fn os_erros_son_claros_e_non_deixan_ficheiros_a_medias() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let v = video(dir.path(), "v.mp4");
    let s = srt_latin1(dir.path());
    let out = dir.path().join("x.mp4");
    assert!(embed(&v, &s, &out, false, "galego").is_err(), "idioma de tres letras");
    assert!(embed(&v, &s, &dir.path().join("x.avi"), false, "").is_err(), "AVI non admite pistas");
    let junk = dir.path().join("lixo.srt");
    std::fs::write(&junk, "isto non son subtítulos").unwrap();
    assert!(embed(&v, &junk, &out, false, "").is_err());
    assert!(extract_track(&v, 0, &dir.path().join("y.srt"), false).is_err(), "o vídeo non ten pistas");
    assert!(extract_track(&v, 0, &dir.path().join("y.doc"), false).is_err());
    let audio = dir.path().join("a.wav");
    ff(&["-f", "lavfi", "-i", "sine=d=1", audio.to_str().unwrap()]);
    assert!(burn(&audio, &s, &dir.path().join("z.mp4"), false).is_err(), "sen imaxe");
    // Non existe ningún ficheiro a medias.
    for n in ["x.mp4", "x.avi", "y.srt", "y.doc", "z.mp4"] {
        assert!(!dir.path().join(n).exists(), "{n}");
    }
    assert!(std::fs::read_dir(dir.path()).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().contains(".part.")));
}
