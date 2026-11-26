//! Vídeo en MPEG-TS (.ts), o que gardan Ace Stream e outros reproductores de emisións en directo.
//! Gravación realista: H.264 + AAC, cortada no medio dun paquete (como queda ao pechar o reproductor) e comezando fóra dun fotograma clave.
//! `cargo test -p trasno-engine --test ts -- --nocapture`

use std::path::{Path, PathBuf};
use trasno_engine::edit_plan::{apply_plan, EditPlan};
use trasno_engine::formats::{self, Category};
use trasno_engine::{convert_file, ffmpeg, ConvertOptions};

fn ff(args: &[&str]) {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-y"]).args(args).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

/// Mostra de 8 s en .ts, gardada como un reproductor: só os primeiros 600 KB menos uns bytes (último paquete a medias).
fn recording(dir: &Path, truncated: bool) -> PathBuf {
    let full = dir.join("full.ts");
    ff(&["-f", "lavfi", "-i", "testsrc=s=640x360:r=25:d=8", "-f", "lavfi", "-i", "sine=f=440:r=48000:d=8", "-c:v", "libx264", "-g", "50", "-pix_fmt", "yuv420p", "-c:a", "aac", "-f", "mpegts", full.to_str().unwrap()]);
    if !truncated {
        return full;
    }
    let bytes = std::fs::read(&full).unwrap();
    let cut = (bytes.len() * 3 / 4) / 188 * 188 + 77; // a metade dun paquete
    let p = dir.join("gravacion.ts");
    // Tamén comeza no medio dun paquete: lixo de 100 bytes diante.
    let mut out = vec![0x55u8; 100];
    out.extend_from_slice(&bytes[..cut]);
    std::fs::write(&p, out).unwrap();
    p
}

fn decoded_secs(p: &Path) -> f64 {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-stats", "-i", p.to_str().unwrap(), "-f", "null", "-"]).output().unwrap();
    let log = String::from_utf8_lossy(&o.stderr).replace('\r', "\n");
    log.rsplit("time=").next().and_then(|t| t.split_whitespace().next()).map(|t| t.split(':').fold(0.0, |a, x| a * 60.0 + x.parse::<f64>().unwrap_or(0.0))).unwrap_or(0.0)
}

#[test]
fn ts_e_video_e_non_se_ofrece_como_destino() {
    assert_eq!(formats::category_of_extension("ts"), Category::Video);
    assert_eq!(formats::category_of_extension("TS"), Category::Video);
    let t = trasno_engine::suggested_targets(Path::new("directo.ts"));
    assert!(t.contains(&"mp4") && t.contains(&"mkv") && !t.contains(&"ts"), "{t:?}");
    // Os demais vídeos non cambian: seguen ofrecendo todo o que ofrecían.
    assert!(trasno_engine::suggested_targets(Path::new("a.mp4")).contains(&"avi"));
}

#[test]
fn ts_convertese_a_mp4_mkv_mp3_e_gif() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for truncated in [false, true] {
        let src = recording(dir.path(), truncated);
        for ext in ["mp4", "mkv", "mov", "webm", "avi", "mp3", "gif"] {
            let out = dir.path().join(format!("o{}.{ext}", truncated as u8));
            let r = convert_file(&src, &out, &ConvertOptions { overwrite: true, ..Default::default() }).unwrap_or_else(|e| panic!("{ext} (cortado: {truncated}): {e}"));
            assert!(r.bytes_written > 1000, "{ext}: ficheiro baleiro");
            if ext != "gif" {
                let secs = decoded_secs(&out);
                let want = if truncated { 4.0 } else { 7.5 };
                assert!(secs >= want, "{ext} (cortado: {truncated}) só dura {secs} s");
            }
        }
    }
}

#[test]
fn ts_edita_e_garda_como_ts_ou_mp4() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = recording(dir.path(), false);
    for ext in ["ts", "mp4"] {
        let out = dir.path().join(format!("cut.{ext}"));
        apply_plan(&src, &out, true, &EditPlan { start: 2.0, end: Some(6.0), ..Default::default() }).unwrap_or_else(|e| panic!("{ext}: {e}"));
        let secs = decoded_secs(&out);
        assert!((secs - 4.0).abs() < 0.5, "{ext}: dura {secs} s e agardábase 4");
    }
}
