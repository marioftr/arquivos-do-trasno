//! Transcrición de audio e vídeo con whisper.cpp (extensión opcional: o programa e o modelo descárganse a petición, ver `tools_install`).
//!
//! Todo se fai no equipo: FFmpeg saca o son a un WAV de 16 kHz en mono (o que lle fai falta ao modelo), `whisper-cli` transcríbeo
//! e o resultado (subtítulos SRT ou VTT, ou texto) escríbese sempre nun ficheiro novo, co mesmo cuidado có resto do motor: primeiro
//! nun temporal e só despois ao seu nome. O avance (0 a 1) chega á interface como o de FFmpeg.

use crate::error::{EngineError, Result};
use crate::ffmpeg::{self, check_success, run_ffmpeg};
use crate::integrity::{ensure_distinct, AtomicOutput};
use crate::tools_install::{self, Tool};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// O programa de whisper.cpp: `TRASNO_WHISPER_PATH`, o PATH ou o que descargou a app (con `TRASNO_ONLY_OWN_TOOLS`, só isto último).
pub fn whisper_path() -> Option<PathBuf> {
    let own = || tools_install::installed(Tool::Whisper, Tool::Whisper.exe());
    if std::env::var_os("TRASNO_ONLY_OWN_TOOLS").is_some() {
        return own();
    }
    if let Some(p) = std::env::var_os("TRASNO_WHISPER_PATH").map(PathBuf::from).filter(|p| p.is_file()) {
        return Some(p);
    }
    which::which("whisper-cli").ok().or_else(own)
}

pub fn is_available() -> bool {
    whisper_path().is_some()
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TranscribeOptions {
    /// Id do modelo (`whisper-base`, `whisper-small`, `whisper-medium`).
    pub model: String,
    /// Código do idioma falado («gl», «en»…) ou «auto» para que o detecte.
    pub language: String,
    /// Traducir ao inglés en vez de transcribir no idioma orixinal.
    #[serde(default)]
    pub translate: bool,
    /// «srt», «vtt» ou «txt».
    pub format: String,
    /// Subtítulos de liñas curtas (ata uns 42 caracteres) en vez de frases enteiras.
    #[serde(default)]
    pub short_lines: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscribeReport {
    pub output_path: PathBuf,
    /// Idioma que detectou o modelo (só con «auto»).
    pub language: Option<String>,
    /// Canto dura o son, en segundos.
    pub duration: f64,
    /// Cantas liñas ou frases saíron.
    pub segments: usize,
}

fn check(o: &TranscribeOptions) -> Result<()> {
    let lang_ok = o.language == "auto" || (o.language.len() >= 2 && o.language.len() <= 3 && o.language.chars().all(|c| c.is_ascii_lowercase()));
    if !lang_ok || !matches!(o.format.as_str(), "srt" | "vtt" | "txt") || !matches!(o.model.as_str(), "whisper-base" | "whisper-small" | "whisper-medium") {
        return Err(EngineError::IntegrityCheck("Os valores para transcribir non son válidos.".into()));
    }
    Ok(())
}

/// «whisper_print_progress_callback: progress = 40%» → 0,4.
fn parse_progress(line: &str) -> Option<f64> {
    let rest = line.split("progress = ").nth(1)?.trim_start();
    let n: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    n.parse::<f64>().ok().map(|p| (p / 100.0).clamp(0.0, 1.0))
}

/// «auto-detected language: gl (p = 0.97)» → «gl».
fn parse_language(line: &str) -> Option<String> {
    let rest = line.split("auto-detected language: ").nth(1)?;
    let code: String = rest.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    (!code.is_empty()).then_some(code)
}

/// Cantas frases ten o resultado (liñas con tempo en SRT e VTT; liñas con texto no TXT).
fn count_segments(text: &str, format: &str) -> usize {
    if format == "txt" {
        text.lines().filter(|l| !l.trim().is_empty()).count()
    } else {
        text.lines().filter(|l| l.contains("-->")).count()
    }
}

/// Transcribe `input` (audio ou vídeo) e escribe o resultado en `output`.
pub fn transcribe(input: &Path, output: &Path, overwrite: bool, o: &TranscribeOptions) -> Result<TranscribeReport> {
    check(o)?;
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    ensure_distinct(&[input], output)?;
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    let exe = whisper_path().ok_or(EngineError::ExtensionMissing("whisper"))?;
    let model = tools_install::model_path(&o.model).ok_or(EngineError::ExtensionMissing("whisper-model"))?;
    let info = ffmpeg::probe(input)?;
    if !info["streams"].as_array().is_some_and(|l| l.iter().any(|s| s["codec_type"] == "audio")) {
        return Err(EngineError::IntegrityCheck("Este ficheiro non ten son, así que non hai nada que transcribir.".into()));
    }

    let tmp = tempfile::tempdir()?;
    let wav = tmp.path().join("audio.wav");
    // O 5 % inicial é sacar o son; o resto, a transcrición.
    ffmpeg::set_progress_phase(0.0, 0.05);
    let prepared = (|| {
        let args: Vec<std::ffi::OsString> = vec![
            "-i".into(),
            input.as_os_str().to_owned(),
            "-map".into(),
            "0:a:0".into(),
            "-vn".into(),
            "-ac".into(),
            "1".into(),
            "-ar".into(),
            "16000".into(),
            "-c:a".into(),
            "pcm_s16le".into(),
            wav.as_os_str().to_owned(),
        ];
        check_success(&run_ffmpeg(&args)?)
    })();
    ffmpeg::set_progress_phase(0.05, 0.95);
    let result = prepared.and_then(|_| run_whisper(&exe, &model, &wav, tmp.path(), o));
    ffmpeg::set_progress_phase(0.0, 1.0);
    let (text, language) = result?;

    let seconds = std::fs::metadata(&wav).map(|m| m.len().saturating_sub(44) as f64 / 32_000.0).unwrap_or(0.0);
    let segments = count_segments(&text, &o.format);
    if segments == 0 {
        return Err(EngineError::NoSpeech);
    }
    let mut out = AtomicOutput::new(output, overwrite)?;
    out.write_all(text.as_bytes())?;
    let output_path = out.commit()?;
    ffmpeg::report_progress(1.0);
    Ok(TranscribeReport { output_path, language, duration: seconds, segments })
}

/// Lanza `whisper-cli` sobre o WAV e devolve o texto do resultado e o idioma detectado.
fn run_whisper(exe: &Path, model: &Path, wav: &Path, dir: &Path, o: &TranscribeOptions) -> Result<(String, Option<String>)> {
    let stem = dir.join("resultado");
    let mut cmd = std::process::Command::new(exe);
    // O son e o resultado van por nome relativo, desde o cartafol temporal: así non dependen de acentos na ruta do usuario.
    cmd.current_dir(dir);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 8);
    let wav_name = wav.file_name().unwrap_or(wav.as_os_str());
    cmd.arg("-m").arg(model).arg("-f").arg(wav_name).arg("-of").arg("resultado").arg("-pp").arg("-t").arg(threads.to_string()).arg("-l").arg(&o.language);
    cmd.arg(match o.format.as_str() {
        "srt" => "-osrt",
        "vtt" => "-ovtt",
        _ => "-otxt",
    });
    if o.translate {
        cmd.arg("-tr");
    }
    if o.short_lines && o.format != "txt" {
        cmd.args(["-ml", "42", "-sow"]);
    }
    let language = Arc::new(std::sync::Mutex::new(None::<String>));
    let lang2 = language.clone();
    let watch: ffmpeg::Watch = Arc::new(move |line: &str| {
        if let Some(p) = parse_progress(line) {
            ffmpeg::report_progress(p);
        } else if let Some(l) = parse_language(line) {
            *lang2.lock().unwrap_or_else(|e| e.into_inner()) = Some(l);
        }
    });
    let out = ffmpeg::run_external_watch(cmd, std::time::Duration::from_secs(12 * 3600), Some(watch))?;
    if out.code != Some(0) {
        let tail: Vec<&str> = out.stderr.lines().filter(|l| !l.trim().is_empty()).rev().take(3).collect();
        let detail = tail.into_iter().rev().collect::<Vec<_>>().join(" ");
        return Err(EngineError::ExternalFailed { program: "whisper.cpp", detail });
    }
    let ext = match o.format.as_str() {
        "srt" => "srt",
        "vtt" => "vtt",
        _ => "txt",
    };
    let text = std::fs::read_to_string(stem.with_extension(ext)).map_err(|_| EngineError::EmptyOrCorruptOutput)?;
    let lang = language.lock().unwrap_or_else(|e| e.into_inner()).clone();
    Ok((text, lang))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_o_avance_e_o_idioma_do_que_di_whisper() {
        assert_eq!(parse_progress("whisper_print_progress_callback: progress =  40%"), Some(0.4));
        assert_eq!(parse_progress("whisper_print_progress_callback: progress = 100%"), Some(1.0));
        assert_eq!(parse_progress("outra cousa"), None);
        assert_eq!(parse_language("whisper_full_with_state: auto-detected language: gl (p = 0.97)"), Some("gl".to_string()));
        assert_eq!(parse_language("sen idioma"), None);
    }

    #[test]
    fn conta_as_frases() {
        let srt = "1\n00:00:00,000 --> 00:00:03,000\n Ola\n\n2\n00:00:03,320 --> 00:00:06,000\n Adeus\n";
        assert_eq!(count_segments(srt, "srt"), 2);
        assert_eq!(count_segments(" Ola\n\n Adeus\n", "txt"), 2);
        assert_eq!(count_segments("", "srt"), 0);
    }

    #[test]
    fn rexeita_valores_non_validos() {
        let ok = TranscribeOptions { model: "whisper-base".into(), language: "gl".into(), translate: false, format: "srt".into(), short_lines: true };
        assert!(check(&ok).is_ok());
        assert!(check(&TranscribeOptions { language: "auto".into(), ..ok.clone() }).is_ok());
        assert!(check(&TranscribeOptions { language: "../x".into(), ..ok.clone() }).is_err());
        assert!(check(&TranscribeOptions { format: "doc".into(), ..ok.clone() }).is_err());
        assert!(check(&TranscribeOptions { model: "../../evil".into(), ..ok }).is_err());
    }

    #[test]
    fn sen_extension_instalada_avisa_con_claridade() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.wav");
        std::fs::write(&input, b"RIFF").unwrap();
        let o = TranscribeOptions { model: "whisper-base".into(), language: "gl".into(), translate: false, format: "srt".into(), short_lines: false };
        let r = transcribe(&input, &dir.path().join("a.srt"), false, &o);
        // Sen whisper (ou sen o modelo) debe dicir que falta unha extensión, non fallar con outra cousa.
        if !is_available() || tools_install::model_path("whisper-base").is_none() {
            assert!(matches!(r, Err(EngineError::ExtensionMissing(_))));
        }
    }
}
