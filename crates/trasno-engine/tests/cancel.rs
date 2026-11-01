//! Cancelar un traballo de ffmpeg a medias: para o proceso, devolve `Cancelled` e non deixa nada atrás.
//! Vai nun ficheiro propio porque a cancelación é global ao proceso e non pode mesturarse con outros tests.

use std::time::Duration;
use trasno_engine::{error::EngineError, ffmpeg};

#[test]
fn cancelar_para_ffmpeg_e_non_deixa_temporais() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("grande.mp4");
    // Un vídeo chega longo para dar tempo a cancelar (a codificación posterior tarda máis que isto).
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=1280x720:rate=30:duration=60", "-pix_fmt", "yuv420p", "-preset", "ultrafast"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());

    ffmpeg::begin_job();
    let out = dir.path().join("saida.mkv");
    let (src2, out2) = (src.clone(), out.clone());
    let worker = std::thread::spawn(move || ffmpeg::convert_media(&src2, &out2, false, &["-c:v".into(), "libx264".into(), "-preset".into(), "veryslow".into()]));
    std::thread::sleep(Duration::from_millis(800));
    assert!(ffmpeg::cancel_current(), "había un ffmpeg en marcha");
    let result = worker.join().unwrap();
    assert!(matches!(result, Err(EngineError::Cancelled)), "{result:?}");
    assert!(!out.exists(), "o destino non debe existir");
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.starts_with(".trasno-")).collect();
    assert!(leftovers.is_empty(), "restos: {leftovers:?}");

    // Despois de comezar un traballo novo, volve funcionar.
    ffmpeg::begin_job();
    let ok = dir.path().join("curto.mkv");
    ffmpeg::convert_media(&src, &ok, false, &["-t".into(), "1".into(), "-c".into(), "copy".into()]).unwrap();
    assert!(ok.exists());
}
