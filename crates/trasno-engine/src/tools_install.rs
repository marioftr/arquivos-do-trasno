//! Descarga e instalación de FFmpeg e MKVToolNix desde as súas webs oficiais, a petición do usuario.
//!
//! Cada descarga compróbase co SHA-256 que publica a propia fonte antes de extraela, e extráese nunha
//! carpeta da app (`%LOCALAPPDATA%\ArquivosDoTrasno\tools\<programa>`): non se necesitan permisos de
//! administrador nin se toca o sistema. Se o usuario xa ten o programa no PATH, úsase ese.

use crate::error::{EngineError, Result};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Ffmpeg,
    MkvToolNix,
}

impl Tool {
    pub fn parse(s: &str) -> Option<Tool> {
        match s {
            "ffmpeg" => Some(Tool::Ffmpeg),
            "mkvmerge" | "mkvtoolnix" => Some(Tool::MkvToolNix),
            _ => None,
        }
    }
    fn dir_name(self) -> &'static str {
        match self {
            Tool::Ffmpeg => "ffmpeg",
            Tool::MkvToolNix => "mkvtoolnix",
        }
    }
}

/// Fase actual, para que a interface a poida contar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Download,
    Verify,
    Extract,
}

/// Carpeta onde a app garda os programas que descarga.
pub fn tools_dir() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))?;
    Some(base.join("ArquivosDoTrasno").join("tools"))
}

/// Busca `file` dentro do que se instalou para `tool` (ata tres niveis de subcartafol).
pub fn installed(tool: Tool, file: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, file: &str, depth: u8) -> Option<PathBuf> {
        let direct = dir.join(file);
        if direct.is_file() {
            return Some(direct);
        }
        if depth == 0 {
            return None;
        }
        for e in fs::read_dir(dir).ok()?.flatten() {
            if e.path().is_dir() {
                if let Some(p) = walk(&e.path(), file, depth - 1) {
                    return Some(p);
                }
            }
        }
        None
    }
    walk(&tools_dir()?.join(tool.dir_name()), file, 3)
}

/// Quita o que descargou a app para `tool`.
pub fn remove(tool: Tool) -> Result<()> {
    if let Some(d) = tools_dir().map(|d| d.join(tool.dir_name())) {
        if d.exists() {
            fs::remove_dir_all(d)?;
        }
    }
    Ok(())
}

fn fail<T>(msg: impl Into<String>) -> Result<T> {
    Err(EngineError::IntegrityCheck(msg.into()))
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .user_agent("ArquivosDoTrasno")
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(30))
        .redirects(5)
        .build()
}

fn net_err(what: &str, e: ureq::Error) -> EngineError {
    EngineError::IntegrityCheck(format!("Non puiden {what}: {e}. Comproba a conexión a internet e téntao de novo."))
}

/// Primeiro valor hexadecimal de 64 caracteres dun ficheiro `.sha256`.
fn parse_sha256(text: &str) -> Option<String> {
    text.split_whitespace().find(|w| w.len() == 64 && w.chars().all(|c| c.is_ascii_hexdigit())).map(|w| w.to_ascii_lowercase())
}

/// Onde descargar `tool`: (url do zip, url do SHA-256).
fn sources(a: &ureq::Agent, tool: Tool) -> Result<(String, String)> {
    match tool {
        Tool::Ffmpeg => {
            let zip = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip".to_string();
            Ok((zip.clone(), format!("{zip}.sha256")))
        }
        Tool::MkvToolNix => {
            let version = a
                .get("https://mkvtoolnix.download/latest-release.json")
                .call()
                .ok()
                .and_then(|r| r.into_json::<serde_json::Value>().ok())
                .and_then(|j| j["mkvtoolnix-releases"]["latest-source"]["version"].as_str().map(str::to_string))
                .unwrap_or_else(|| "103.0".to_string());
            if !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
                return fail("A versión de MKVToolNix que anuncia a web non é válida.");
            }
            let zip = format!("https://mkvtoolnix.download/windows/releases/{version}/mkvtoolnix-64-bit-{version}.zip");
            Ok((zip.clone(), format!("{zip}.sha256")))
        }
    }
}

/// Descarga, verifica e instala `tool`. `progress(fase, 0..1)` avisa do avance dentro de cada fase;
/// `cancelled()` consúltase mentres traballa. Devolve a ruta do executable principal.
pub fn install(tool: Tool, progress: &dyn Fn(Phase, f64), cancelled: &dyn Fn() -> bool) -> Result<PathBuf> {
    if !cfg!(windows) {
        return fail("A descarga automática só está dispoñible en Windows.");
    }
    let root = tools_dir().ok_or_else(|| EngineError::IntegrityCheck("Non atopo onde gardar os programas.".into()))?;
    fs::create_dir_all(&root)?;
    let a = agent();
    let (zip_url, sum_url) = sources(&a, tool)?;

    let expected = parse_sha256(&a.get(&sum_url).call().map_err(|e| net_err("obter a suma de comprobación", e))?.into_string()?)
        .ok_or_else(|| EngineError::IntegrityCheck("A suma de comprobación publicada non ten o formato esperado.".into()))?;

    // 1. Descarga a un ficheiro temporal, calculando o hash ao mesmo tempo.
    let tmp = root.join(format!("{}.download", tool.dir_name()));
    let result = (|| -> Result<PathBuf> {
        let resp = a.get(&zip_url).call().map_err(|e| net_err("descargar o ficheiro", e))?;
        let total: u64 = resp.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
        let mut reader = resp.into_reader();
        let mut out = File::create(&tmp)?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 64 * 1024];
        let mut done: u64 = 0;
        let mut last = -1.0f64;
        loop {
            if cancelled() {
                return Err(EngineError::Cancelled);
            }
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            out.write_all(&buf[..n])?;
            done += n as u64;
            if total > 0 {
                let f = done as f64 / total as f64;
                if f - last >= 0.005 {
                    last = f;
                    progress(Phase::Download, f.min(1.0));
                }
            }
        }
        out.flush()?;
        drop(out);
        progress(Phase::Download, 1.0);

        // 2. Comprobación do hash publicado pola fonte.
        progress(Phase::Verify, 0.0);
        let got = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>();
        if got != expected {
            return fail("O ficheiro descargado non coincide coa suma de comprobación publicada, así que non o instalo.");
        }
        progress(Phase::Verify, 1.0);

        // 3. Extracción a unha carpeta nova; só se substitúe a anterior se saíu ben.
        let stage = root.join(format!("{}.new", tool.dir_name()));
        if stage.exists() {
            fs::remove_dir_all(&stage)?;
        }
        if let Err(e) = extract_zip(&tmp, &stage, progress, cancelled) {
            let _ = fs::remove_dir_all(&stage);
            return Err(e);
        }
        let target = root.join(tool.dir_name());
        if target.exists() {
            fs::remove_dir_all(&target)?;
        }
        fs::rename(&stage, &target)?;
        let main = if tool == Tool::Ffmpeg { "ffmpeg.exe" } else { "mkvmerge.exe" };
        installed(tool, main).ok_or_else(|| EngineError::IntegrityCheck(format!("Instaleino, pero non atopo {main} dentro do paquete.")))
    })();
    let _ = fs::remove_file(&tmp);
    result
}

fn extract_zip(zip_path: &Path, dest: &Path, progress: &dyn Fn(Phase, f64), cancelled: &dyn Fn() -> bool) -> Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(zip_path)?).map_err(|e| EngineError::IntegrityCheck(format!("O paquete descargado está danado: {e}")))?;
    let total: u64 = (0..zip.len()).filter_map(|i| zip.by_index(i).ok().map(|f| f.size())).sum::<u64>().max(1);
    let mut done: u64 = 0;
    let mut last = -1.0f64;
    fs::create_dir_all(dest)?;
    for i in 0..zip.len() {
        if cancelled() {
            return Err(EngineError::Cancelled);
        }
        let mut f = zip.by_index(i).map_err(|e| EngineError::IntegrityCheck(format!("Paquete danado: {e}")))?;
        // `enclosed_name` rexeita rutas que saian da carpeta (zip-slip).
        let Some(rel) = f.enclosed_name() else { continue };
        let out_path = dest.join(rel);
        if f.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(p) = out_path.parent() {
            fs::create_dir_all(p)?;
        }
        let mut out = File::create(&out_path)?;
        let size = f.size();
        std::io::copy(&mut f, &mut out)?;
        done += size;
        let p = done as f64 / total as f64;
        if p - last >= 0.01 {
            last = p;
            progress(Phase::Extract, p.min(1.0));
        }
    }
    progress(Phase::Extract, 1.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_o_hash_de_ficheiros_sha256() {
        let h = "60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba";
        assert_eq!(parse_sha256(h).as_deref(), Some(h));
        assert_eq!(parse_sha256(&format!("{}  ficheiro.zip\n", h.to_uppercase())).as_deref(), Some(h));
        assert!(parse_sha256("nada").is_none());
    }

    #[test]
    fn extrae_un_zip_e_rexeita_rutas_que_saen_da_carpeta() {
        let dir = tempfile::tempdir().unwrap();
        let zp = dir.path().join("a.zip");
        {
            let mut w = zip::ZipWriter::new(File::create(&zp).unwrap());
            let o = zip::write::SimpleFileOptions::default();
            w.start_file("pkg/bin/ffmpeg.exe", o).unwrap();
            w.write_all(b"x").unwrap();
            w.start_file("../fuxe.txt", o).unwrap();
            w.write_all(b"y").unwrap();
            w.finish().unwrap();
        }
        let dest = dir.path().join("out");
        extract_zip(&zp, &dest, &|_, _| {}, &|| false).unwrap();
        assert!(dest.join("pkg/bin/ffmpeg.exe").is_file());
        assert!(!dir.path().join("fuxe.txt").exists());
    }
}

#[cfg(test)]
mod red {
    use super::*;

    /// Descarga real (uns 100 MB): `cargo test -p trasno-engine --lib red -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn instala_mkvtoolnix_de_verdade() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("LOCALAPPDATA", dir.path());
        let seen = std::sync::Mutex::new(Vec::new());
        let exe = install(Tool::MkvToolNix, &|p, f| { if f >= 1.0 { seen.lock().unwrap().push(p); } }, &|| false).unwrap();
        assert!(exe.is_file());
        assert_eq!(seen.lock().unwrap().len(), 3);
        let out = std::process::Command::new(&exe).arg("--version").output().unwrap();
        assert!(String::from_utf8_lossy(&out.stdout).contains("mkvmerge"));
        assert!(installed(Tool::MkvToolNix, "mkvmerge.exe").is_some());
        remove(Tool::MkvToolNix).unwrap();
        assert!(installed(Tool::MkvToolNix, "mkvmerge.exe").is_none());
    }
}
