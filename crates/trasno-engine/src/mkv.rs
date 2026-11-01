//! MKVToolNix (`mkvmerge`): edición sen perdas de contedores Matroska
//! (quitar pistas, cambiar nomes, idiomas e a pista por defecto).
//! Igual que ffmpeg, non vai empaquetado: úsase o que haxa instalado.

use crate::error::{EngineError, Result};
use crate::integrity::{assert_non_empty_file, ensure_distinct};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn mkvmerge_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("TRASNO_MKVMERGE_PATH") {
        return Ok(PathBuf::from(p));
    }
    let file = format!("mkvmerge{}", std::env::consts::EXE_SUFFIX);
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())) {
        for c in [dir.join(&file), dir.join("mkvtoolnix").join(&file), dir.join("bin").join(&file)] {
            if c.is_file() {
                return Ok(c);
            }
        }
    }
    // O instalador de Windows non o engade ao PATH.
    for var in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(base) = std::env::var_os(var) {
            let c = Path::new(&base).join("MKVToolNix").join(&file);
            if c.is_file() {
                return Ok(c);
            }
        }
    }
    which::which("mkvmerge").map_err(|_| EngineError::IntegrityCheck("Falta MKVToolNix (mkvmerge): instálao desde a xanela de extensións.".into()))
}

pub fn is_available() -> bool {
    mkvmerge_path().is_ok()
}

fn run(args: &[std::ffi::OsString]) -> Result<std::process::Output> {
    let mut cmd = Command::new(mkvmerge_path()?);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let out = cmd.args(args).output().map_err(EngineError::Io)?;
    // 0 = ben, 1 = avisos (o resultado vale), 2 = erro.
    if out.status.code().map_or(true, |c| c >= 2) {
        let text = String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
        return Err(EngineError::IntegrityCheck(format!("mkvmerge: {}", text.trim())));
    }
    Ok(out)
}

/// Pistas do ficheiro (JSON de `mkvmerge -J`).
pub fn tracks(path: &Path) -> Result<serde_json::Value> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    let out = run(&["-J".into(), path.as_os_str().to_owned()])?;
    serde_json::from_slice(&out.stdout).map_err(|e| EngineError::IntegrityCheck(e.to_string()))
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackEdit {
    pub id: u64,
    pub name: Option<String>,
    pub language: Option<String>,
    pub default: Option<bool>,
}

/// Garda unha copia con só as pistas indicadas e os cambios de nome, idioma
/// e pista por defecto. Non se recodifica nada. O orixinal non se toca.
pub fn remux(input: &Path, output: &Path, overwrite: bool, keep: &[u64], edits: &[TrackEdit]) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    let info = tracks(input)?;
    let list = info["tracks"].as_array().cloned().unwrap_or_default();
    let mut by_type: std::collections::BTreeMap<&str, Vec<String>> = Default::default();
    for t in &list {
        let (Some(id), Some(ty)) = (t["id"].as_u64(), t["type"].as_str()) else { continue };
        if keep.contains(&id) {
            let key = match ty {
                "video" => "video",
                "audio" => "audio",
                "subtitles" => "subtitle",
                _ => "other",
            };
            by_type.entry(key).or_default().push(id.to_string());
        }
    }
    if by_type.is_empty() {
        return Err(EngineError::IntegrityCheck("Hai que deixar polo menos unha pista.".into()));
    }

    let dir = output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let ext = output.extension().and_then(|e| e.to_str()).unwrap_or("mkv");
    let temp = dir.join(format!(".trasno-{}.part.{}", uuid::Uuid::new_v4(), ext));

    let mut args: Vec<std::ffi::OsString> = vec!["-o".into(), temp.as_os_str().to_owned()];
    for (opt, none) in [("video", "--no-video"), ("audio", "--no-audio"), ("subtitle", "--no-subtitles")] {
        match by_type.get(opt) {
            Some(ids) => args.extend([format!("--{opt}-tracks").into(), ids.join(",").into()]),
            None => args.push(none.into()),
        }
    }
    args.push("--no-attachments".into());
    for e in edits.iter().filter(|e| keep.contains(&e.id)) {
        if let Some(n) = &e.name {
            args.extend(["--track-name".into(), format!("{}:{}", e.id, n).into()]);
        }
        if let Some(l) = &e.language {
            if !l.is_empty() {
                args.extend(["--language".into(), format!("{}:{}", e.id, l).into()]);
            }
        }
        if let Some(d) = e.default {
            args.extend(["--default-track-flag".into(), format!("{}:{}", e.id, if d { 1 } else { 0 }).into()]);
        }
    }
    args.push(input.as_os_str().to_owned());

    if let Err(e) = run(&args) {
        let _ = std::fs::remove_file(&temp);
        return Err(e);
    }
    if let Err(e) = assert_non_empty_file(&temp) {
        let _ = std::fs::remove_file(&temp);
        return Err(e);
    }
    std::fs::rename(&temp, output).map_err(EngineError::Io)?;
    Ok(output.to_path_buf())
}
