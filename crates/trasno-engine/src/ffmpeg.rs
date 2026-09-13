//! Wrapper para ffmpeg/ffprobe. Na fase actual búscanse os binarios no
//! PATH do sistema (ou en `TRASNO_FFMPEG_PATH` / `TRASNO_FFPROBE_PATH`);
//! nunha fase posterior empaquetaranse como "sidecars" de Tauri para non
//! depender de que o usuario os instale á parte.
//!
//! Todo o audio e vídeo (edición, mestura, recorte, montaxe, compresión,
//! cambio de formato, xuntar clips) pasa por aquí: o motor propio non
//! reimplementa códecs, delega en ffmpeg, que é o estándar de facto.

use crate::error::{EngineError, Result};
use crate::integrity::assert_non_empty_file;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn ffmpeg_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("TRASNO_FFMPEG_PATH") {
        return Ok(PathBuf::from(p));
    }
    which::which("ffmpeg").map_err(|_| EngineError::FfmpegNotFound)
}

pub fn ffprobe_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("TRASNO_FFPROBE_PATH") {
        return Ok(PathBuf::from(p));
    }
    which::which("ffprobe").map_err(|_| EngineError::FfmpegNotFound)
}

pub fn is_available() -> bool {
    ffmpeg_path().is_ok()
}

fn run_ffmpeg(args: &[std::ffi::OsString]) -> Result<Output> {
    let bin = ffmpeg_path()?;
    let output = Command::new(bin)
        .arg("-hide_banner")
        .arg("-y") // sobrescribe SÓ o noso ficheiro temporal, nunca o do usuario
        .args(args)
        .output()
        .map_err(EngineError::Io)?;
    Ok(output)
}

fn check_success(output: &Output) -> Result<()> {
    if !output.status.success() {
        return Err(EngineError::FfmpegFailed {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(())
}

/// Conversión xenérica de audio/vídeo dun formato a outro. `extra_args`
/// permite pasar opcións específicas (bitrate, códec, filtros de recorte...)
/// sen ter que abrir unha función nova por cada caso de uso.
pub fn convert_media(
    input: &Path,
    output_final: &Path,
    overwrite: bool,
    extra_args: &[String],
) -> Result<PathBuf> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    if output_final.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output_final.to_path_buf()));
    }

    let dir = output_final
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;

    // ffmpeg necesita escribir directamente nun ficheiro real (non nun
    // NamedTempFile xa aberto), así que xeramos un nome temporal propio,
    // deixamos que ffmpeg o cree, e só ao final facemos o rename atómico.
    let ext = output_final
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("out");
    let temp_path = dir.join(format!(".trasno-{}.{}.part", uuid::Uuid::new_v4(), ext));

    let mut args: Vec<std::ffi::OsString> = vec![
        std::ffi::OsString::from("-i"),
        input.as_os_str().to_owned(),
    ];
    args.extend(extra_args.iter().map(std::ffi::OsString::from));
    args.push(temp_path.as_os_str().to_owned());

    let result = run_ffmpeg(&args);

    let output = match result {
        Ok(o) => o,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }
    };

    if let Err(e) = check_success(&output) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    if let Err(e) = assert_non_empty_file(&temp_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    // Verificación de coherencia: que ffprobe sexa quen de ler o ficheiro
    // resultante antes de darlle o visto e prace e movelo ao destino final.
    if let Ok(probe_bin) = ffprobe_path() {
        let probe = Command::new(probe_bin)
            .args(["-v", "error", "-show_entries", "format=duration"])
            .arg(&temp_path)
            .output();
        if let Ok(probe) = probe {
            if !probe.status.success() {
                let _ = std::fs::remove_file(&temp_path);
                return Err(EngineError::IntegrityCheck(
                    "ffprobe non puido ler o ficheiro convertido".into(),
                ));
            }
        }
    }

    std::fs::rename(&temp_path, output_final).map_err(EngineError::Io)?;
    Ok(output_final.to_path_buf())
}

/// Recorta un intervalo [inicio, fin) dun ficheiro de audio ou vídeo sen
/// recodificar cando é posible (copia de stream), o que é case instantáneo
/// e non perde calidade.
pub fn trim_media(
    input: &Path,
    output_final: &Path,
    overwrite: bool,
    start_seconds: f64,
    end_seconds: Option<f64>,
) -> Result<PathBuf> {
    let mut args = vec!["-ss".to_string(), start_seconds.to_string()];
    if let Some(end) = end_seconds {
        args.push("-to".to_string());
        args.push(end.to_string());
    }
    args.push("-c".to_string());
    args.push("copy".to_string());
    convert_media(input, output_final, overwrite, &args)
}

/// Xunta varios ficheiros de vídeo/audio do MESMO formato/códec nun só,
/// usando o demuxer "concat" de ffmpeg (sen recodificar).
pub fn concat_media(inputs: &[PathBuf], output_final: &Path, overwrite: bool) -> Result<PathBuf> {
    if inputs.is_empty() {
        return Err(EngineError::IntegrityCheck(
            "non se indicou ningún ficheiro para xuntar".into(),
        ));
    }
    for i in inputs {
        if !i.is_file() {
            return Err(EngineError::InputNotFound(i.clone()));
        }
    }
    if output_final.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output_final.to_path_buf()));
    }

    let dir = output_final
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;

    let list_path = dir.join(format!(".trasno-concat-{}.txt", uuid::Uuid::new_v4()));
    let list_contents = inputs
        .iter()
        .map(|p| format!("file '{}'", p.to_string_lossy().replace('\'', "'\\''")))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&list_path, list_contents)?;

    let ext = output_final
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("out");
    let temp_path = dir.join(format!(".trasno-{}.{}.part", uuid::Uuid::new_v4(), ext));

    let args: Vec<std::ffi::OsString> = vec![
        "-f".into(),
        "concat".into(),
        "-safe".into(),
        "0".into(),
        "-i".into(),
        list_path.as_os_str().to_owned(),
        "-c".into(),
        "copy".into(),
        temp_path.as_os_str().to_owned(),
    ];

    let run_result = run_ffmpeg(&args);
    let _ = std::fs::remove_file(&list_path);

    let output = match run_result {
        Ok(o) => o,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }
    };
    if let Err(e) = check_success(&output) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }
    if let Err(e) = assert_non_empty_file(&temp_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    std::fs::rename(&temp_path, output_final).map_err(EngineError::Io)?;
    Ok(output_final.to_path_buf())
}
