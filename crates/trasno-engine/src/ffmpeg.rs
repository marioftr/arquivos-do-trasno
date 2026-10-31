//! Wrapper para ffmpeg/ffprobe. Na fase actual búscanse os binarios no
//! PATH do sistema (ou en `TRASNO_FFMPEG_PATH` / `TRASNO_FFPROBE_PATH`);
//! nunha fase posterior empaquetaranse como "sidecars" de Tauri para non
//! depender de que o usuario os instale á parte.
//!
//! Todo o audio e vídeo (edición, mestura, recorte, montaxe, compresión,
//! cambio de formato, xuntar clips) pasa por aquí: o motor propio non
//! reimplementa códecs, delega en ffmpeg, que é o estándar de facto.

use crate::error::{EngineError, Result};
use crate::integrity::{assert_non_empty_file, ensure_distinct};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Busca un binario de ffmpeg: variable de contorno, despois ao carón do
/// executable da app (`ffmpeg.exe` ou `ffmpeg/ffmpeg.exe`, para levalo
/// portable) e por último o PATH do sistema.
fn find_tool(name: &str, env: &str) -> Result<PathBuf> {
    if let Ok(p) = std::env::var(env) {
        return Ok(PathBuf::from(p));
    }
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())) {
        let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
        for candidate in [dir.join(&file), dir.join("ffmpeg").join(&file), dir.join("bin").join(&file)] {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    which::which(name).map_err(|_| EngineError::FfmpegNotFound)
}

pub fn ffmpeg_path() -> Result<PathBuf> {
    find_tool("ffmpeg", "TRASNO_FFMPEG_PATH")
}

pub fn ffprobe_path() -> Result<PathBuf> {
    find_tool("ffprobe", "TRASNO_FFPROBE_PATH")
}

pub fn is_available() -> bool {
    ffmpeg_path().is_ok()
}

/// En Windows evita que salte unha xanela de consola ao lanzar ffmpeg.
fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    #[cfg(not(windows))]
    let _ = cmd;
}

fn run_ffmpeg(args: &[std::ffi::OsString]) -> Result<Output> {
    let bin = ffmpeg_path()?;
    let mut cmd = Command::new(bin);
    hide_console(&mut cmd);
    let output = cmd
        .arg("-hide_banner")
        .arg("-nostdin")
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
    ensure_distinct(&[input], output_final)?;
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
    let temp_path = dir.join(format!(".trasno-{}.part.{}", uuid::Uuid::new_v4(), ext));

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
        let mut probe_cmd = Command::new(probe_bin);
        hide_console(&mut probe_cmd);
        let probe = probe_cmd
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
    if !start_seconds.is_finite() || start_seconds < 0.0 {
        return Err(EngineError::IntegrityCheck("o inicio do recorte non é válido".into()));
    }
    if let Some(end) = end_seconds {
        if !end.is_finite() || end <= start_seconds {
            return Err(EngineError::IntegrityCheck(
                "o fin do recorte ten que ser posterior ao inicio".into(),
            ));
        }
    }
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
        // O ficheiro de lista de concat é liña a liña: un salto de liña no
        // nome permitiría inxectar directivas alleas.
        if i.to_string_lossy().contains(['\n', '\r']) {
            return Err(EngineError::IntegrityCheck(
                "un nome de ficheiro contén saltos de liña".into(),
            ));
        }
    }
    let refs: Vec<&Path> = inputs.iter().map(|p| p.as_path()).collect();
    ensure_distinct(&refs, output_final)?;
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
        .map(|p| {
            // Rutas absolutas: as relativas resolveríanse contra o directorio
            // da lista, non contra o do usuario.
            let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone());
            format!("file '{}'", abs.to_string_lossy().replace('\'', "'\\''"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&list_path, list_contents)?;

    let ext = output_final
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("out");
    let temp_path = dir.join(format!(".trasno-{}.part.{}", uuid::Uuid::new_v4(), ext));

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

/// Operacións sinxelas de edición sobre audio/vídeo, todas cara un ficheiro
/// novo (nunca se toca o orixinal).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaOp {
    /// Garda un só fotograma (imaxe) no segundo indicado.
    Frame(f64),
    /// Factor de velocidade (0.25 a 4.0); 2.0 é o dobre de rápido.
    Speed(f64),
    /// Quita a pista de audio dun vídeo sen recodificar a imaxe.
    Mute,
    /// Multiplicador de volume (0.0 a 4.0).
    Volume(f64),
    /// Cambia a altura do vídeo (en píxeles, par) mantendo a proporción.
    ScaleHeight(u32),
}

pub fn media_edit(input: &Path, output: &Path, overwrite: bool, op: MediaOp, has_video: bool) -> Result<PathBuf> {
    let bad = |m: &str| Err(EngineError::IntegrityCheck(m.into()));
    let mut args: Vec<String> = Vec::new();
    match op {
        MediaOp::Frame(at) => {
            if !at.is_finite() || at < 0.0 {
                return bad("o instante do fotograma non é válido");
            }
            args.extend(["-ss".into(), at.to_string(), "-frames:v".into(), "1".into()]);
        }
        MediaOp::Speed(f) => {
            if !f.is_finite() || !(0.25..=4.0).contains(&f) {
                return bad("a velocidade ten que estar entre 0,25 e 4");
            }
            if has_video {
                args.extend(["-filter:v".into(), format!("setpts=PTS/{f}")]);
            }
            // atempo só admite 0,5 a 2: encadéase para chegar ao resto.
            let mut chain = Vec::new();
            let mut rest = f;
            while rest > 2.0 {
                chain.push("atempo=2.0".to_string());
                rest /= 2.0;
            }
            while rest < 0.5 {
                chain.push("atempo=0.5".to_string());
                rest /= 0.5;
            }
            chain.push(format!("atempo={rest}"));
            args.extend(["-filter:a".into(), chain.join(",")]);
        }
        MediaOp::Mute => args.extend(["-an".into(), "-c:v".into(), "copy".into()]),
        MediaOp::Volume(v) => {
            if !v.is_finite() || !(0.0..=4.0).contains(&v) {
                return bad("o volume ten que estar entre 0 e 4");
            }
            args.extend(["-af".into(), format!("volume={v}")]);
            if has_video {
                args.extend(["-c:v".into(), "copy".into()]);
            }
        }
        MediaOp::ScaleHeight(h) => {
            if !(16..=8640).contains(&h) {
                return bad("a altura non é válida");
            }
            args.extend(["-vf".into(), format!("scale=-2:{}", h - h % 2)]);
        }
    }
    convert_media(input, output, overwrite, &args)
}

/// Información de ffprobe (formato e pistas) como JSON.
pub fn probe(path: &Path) -> Result<serde_json::Value> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    let mut cmd = Command::new(ffprobe_path()?);
    hide_console(&mut cmd);
    let out = cmd
        .args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(path)
        .output()
        .map_err(EngineError::Io)?;
    check_success(&out)?;
    serde_json::from_slice(&out.stdout).map_err(|e| EngineError::IntegrityCheck(e.to_string()))
}
