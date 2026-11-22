//! Probas das extensións opcionais (transcrición, quitar fondos e descarga por ligazón) con descargas e modelos de verdade.
//! Fan falta rede e un cartafol onde gardar o que se descarga, así que están ignoradas por defecto:
//!
//! ```text
//! TRASNO_TOOLS_DIR=<cartafol> TRASNO_TEST_VOICE=<voz.wav> TRASNO_TEST_IMAGE=<foto.png> TRASNO_TEST_URL=<ligazón> \
//!   cargo test -p trasno-engine --test extensiones -- --ignored --nocapture
//! ```

use std::path::PathBuf;
use trasno_engine::tools_install::{self, Tool};
use trasno_engine::{cutout, transcribe, ytdlp};

fn env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var(name).unwrap_or_else(|_| panic!("falta a variable {name}")))
}

fn quiet() -> (impl Fn(f64), impl Fn() -> bool) {
    (|_f: f64| {}, || false)
}

#[test]
#[ignore]
fn transcribe_de_verdade() {
    assert!(std::env::var_os("TRASNO_TOOLS_DIR").is_some(), "usa TRASNO_TOOLS_DIR para non tocar o cartafol real");
    let voice = env_path("TRASNO_TEST_VOICE");
    if !transcribe::is_available() {
        tools_install::install(Tool::Whisper, &|_, _| {}, &|| false).expect("instalar whisper.cpp");
    }
    if tools_install::model_path("whisper-base").is_none() {
        let (p, c) = quiet();
        tools_install::install_model("whisper-base", &p, &c).expect("descargar o modelo");
    }
    assert!(tools_install::models_installed().contains(&"whisper-base".to_string()));
    let dir = tempfile::tempdir().unwrap();
    let opts = |format: &str, short: bool| transcribe::TranscribeOptions { model: "whisper-base".into(), language: "en".into(), translate: false, format: format.into(), short_lines: short };

    let srt = dir.path().join("voz.srt");
    let r = transcribe::transcribe(&voice, &srt, false, &opts("srt", true)).expect("transcribir a SRT");
    assert!(r.segments >= 1 && r.duration > 3.0);
    let text = std::fs::read_to_string(&srt).unwrap().to_lowercase();
    println!("{text}");
    assert!(text.contains("-->") && text.contains("fox"), "o SRT debe ter tempos e a frase");

    let txt = dir.path().join("voz.txt");
    transcribe::transcribe(&voice, &txt, false, &opts("txt", false)).expect("transcribir a texto");
    assert!(std::fs::read_to_string(&txt).unwrap().to_lowercase().contains("transcription"));

    // Con «auto» o modelo di que idioma detectou.
    let auto = dir.path().join("auto.txt");
    let detected = transcribe::transcribe(&voice, &auto, false, &transcribe::TranscribeOptions { language: "auto".into(), ..opts("txt", false) }).expect("transcribir con idioma automático");
    assert!(detected.language.is_some(), "debería informar do idioma detectado");

    // Non pisa un ficheiro que xa existe sen permiso.
    assert!(transcribe::transcribe(&voice, &txt, false, &opts("txt", false)).is_err());
    // Cancelar deixa todo limpo: nada a medias no cartafol.
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.starts_with('.')).collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
#[ignore]
fn quita_o_fondo_de_verdade() {
    assert!(std::env::var_os("TRASNO_TOOLS_DIR").is_some());
    let image = env_path("TRASNO_TEST_IMAGE");
    if tools_install::model_path("cutout").is_none() {
        let (p, c) = quiet();
        tools_install::install_model("cutout", &p, &c).expect("descargar o modelo");
    }
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("sen-fondo.png");
    let started = std::time::Instant::now();
    let r = cutout::remove_background(&image, &out, false, &cutout::CutoutOptions::default()).expect("quitar o fondo");
    println!("tardou {:?}; queda un {:.0} %", started.elapsed(), r.kept * 100.0);
    let img = image::open(&out).unwrap().to_rgba8();
    let (mut clear, mut solid) = (0u64, 0u64);
    for p in img.pixels() {
        match p[3] {
            0..=10 => clear += 1,
            245..=255 => solid += 1,
            _ => {}
        }
    }
    assert!(clear > 1000 && solid > 1000, "debe haber fondo transparente e obxecto opaco ({clear} / {solid})");
    assert_eq!(r.size, (img.width(), img.height()));

    let white = dir.path().join("branco.jpg");
    cutout::remove_background(&image, &white, false, &cutout::CutoutOptions { background: Some([255, 255, 255]), ..Default::default() }).expect("fondo branco");
    assert!(image::open(&white).is_ok());
}

#[test]
#[ignore]
fn descarga_por_ligazon_de_verdade() {
    assert!(std::env::var_os("TRASNO_TOOLS_DIR").is_some());
    let url = std::env::var("TRASNO_TEST_URL").expect("falta TRASNO_TEST_URL");
    if !ytdlp::is_available() {
        tools_install::install(Tool::YtDlp, &|_, _| {}, &|| false).expect("instalar yt-dlp");
    }
    let info = ytdlp::info(&url).expect("ler a ligazón");
    println!("{info:?}");
    assert!(!info.title.is_empty());
    let dir = tempfile::tempdir().unwrap();
    let o = ytdlp::DownloadOptions { kind: "audio".into(), max_height: None, container: "mp4".into(), audio_format: "mp3".into(), audio_kbps: Some(128) };
    let r = ytdlp::download(&url, dir.path(), &o).expect("descargar o audio");
    println!("{r:?}");
    assert!(r.output_path.is_file() && r.output_path.extension().is_some_and(|e| e == "mp3"));
    // O cartafol temporal da descarga non queda atrás.
    let hidden: Vec<_> = std::fs::read_dir(dir.path()).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with('.')).collect();
    assert!(hidden.is_empty());
    // Unha ligazón que non é unha web normal rexéitase sen lanzar nada.
    assert!(ytdlp::download("file:///C:/Windows/notepad.exe", dir.path(), &o).is_err());
}

#[test]
#[ignore]
fn cancelar_a_transcricion_deixa_todo_limpo() {
    assert!(std::env::var_os("TRASNO_TOOLS_DIR").is_some());
    let voice = env_path("TRASNO_TEST_VOICE");
    // Unha gravación longa: a de proba repetida moitas veces (WAV de PCM: a cabeceira, ata o chunk «data», e os datos).
    let wav = std::fs::read(&voice).unwrap();
    let at = wav.windows(4).position(|w| w == b"data").expect("chunk data") + 8;
    let (head, data) = wav.split_at(at);
    let mut long = head.to_vec();
    let times = 40usize;
    for _ in 0..times {
        long.extend_from_slice(data);
    }
    let total = (data.len() * times) as u32;
    long[4..8].copy_from_slice(&(total + at as u32 - 8).to_le_bytes());
    long[at - 4..at].copy_from_slice(&total.to_le_bytes());
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("longo.wav");
    std::fs::write(&input, &long).unwrap();

    trasno_engine::ffmpeg::begin_job();
    let canceller = std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(4));
        trasno_engine::ffmpeg::cancel_current();
    });
    let o = transcribe::TranscribeOptions { model: "whisper-base".into(), language: "en".into(), translate: false, format: "srt".into(), short_lines: false };
    let out = dir.path().join("longo.srt");
    let started = std::time::Instant::now();
    let r = transcribe::transcribe(&input, &out, false, &o);
    canceller.join().unwrap();
    assert!(matches!(r, Err(trasno_engine::EngineError::Cancelled)), "debería saír cancelado: {r:?}");
    assert!(started.elapsed() < std::time::Duration::from_secs(20), "cancelar tardou demasiado");
    assert!(!out.exists(), "non debe quedar o resultado a medias");
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n != "longo.wav").collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    trasno_engine::ffmpeg::begin_job();
}

#[test]
#[ignore]
fn recorte_con_axustes_de_verdade() {
    assert!(std::env::var_os("TRASNO_TOOLS_DIR").is_some());
    let image = env_path("TRASNO_TEST_IMAGE");
    let out_dir = env_path("TRASNO_TEST_OUT");
    let started = std::time::Instant::now();
    let size = cutout::analyze(&image).expect("analizar");
    println!("analizar: {:?} ({size:?})", started.elapsed());
    for (name, detail, soft, edge) in [("d0", 0u8, 70u8, 50u8), ("d60", 60, 70, 50), ("d100", 100, 70, 50), ("duro", 60, 0, 50), ("corta", 60, 70, 80)] {
        let o = cutout::CutoutOptions { background: None, edge, soft, detail };
        let t = std::time::Instant::now();
        let p = cutout::preview(&image, &out_dir.join(format!("prev-{name}.png")), &o).expect("vista previa");
        println!("vista previa {name}: {:?} -> {}", t.elapsed(), p.display());
    }
    let t = std::time::Instant::now();
    let r = cutout::remove_background(&image, &out_dir.join("final.png"), true, &cutout::CutoutOptions::default()).expect("final");
    println!("final con máscara en memoria: {:?}; queda {:.0} %", t.elapsed(), r.kept * 100.0);
}
