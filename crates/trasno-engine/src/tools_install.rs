//! Descarga e instalación de FFmpeg, MKVToolNix, whisper.cpp e yt-dlp desde as súas webs oficiais, e dos modelos que usan
//! a transcrición e quitar fondos, sempre a petición do usuario.
//!
//! Cada descarga compróbase co SHA-256 (o que publica a propia fonte ou, nos modelos e en whisper.cpp, o que ten fixado a app)
//! antes de usala, e vai a unha carpeta da app (`%LOCALAPPDATA%\ArquivosDoTrasno\tools\<programa>`): non se necesitan permisos
//! de administrador nin se toca o sistema. Se o usuario xa ten o programa no PATH, úsase ese. A variable `TRASNO_TOOLS_DIR`
//! cambia esa carpeta (probas e versións portables).

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
    /// whisper.cpp (`whisper-cli`): transcrición de audio.
    Whisper,
    /// yt-dlp: descarga de vídeo e audio por ligazón.
    YtDlp,
}

impl Tool {
    pub fn parse(s: &str) -> Option<Tool> {
        match s {
            "ffmpeg" => Some(Tool::Ffmpeg),
            "mkvmerge" | "mkvtoolnix" => Some(Tool::MkvToolNix),
            "whisper" | "whisper-cli" => Some(Tool::Whisper),
            "ytdlp" | "yt-dlp" => Some(Tool::YtDlp),
            _ => None,
        }
    }
    fn dir_name(self) -> &'static str {
        match self {
            Tool::Ffmpeg => "ffmpeg",
            Tool::MkvToolNix => "mkvtoolnix",
            Tool::Whisper => "whisper",
            Tool::YtDlp => "yt-dlp",
        }
    }
    /// O executable principal do paquete.
    pub fn exe(self) -> &'static str {
        match self {
            Tool::Ffmpeg => "ffmpeg.exe",
            Tool::MkvToolNix => "mkvmerge.exe",
            Tool::Whisper => "whisper-cli.exe",
            Tool::YtDlp => "yt-dlp.exe",
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
    if let Some(d) = std::env::var_os("TRASNO_TOOLS_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d));
    }
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

/// Como se comproba unha descarga.
enum Checksum {
    /// Un ficheiro `.sha256` á beira.
    File(String),
    /// Un SHA-256 fixado na app (a descarga é dunha versión concreta).
    Pinned(&'static str),
    /// Un ficheiro de sumas da release (`<hash> *<nome>` por liña) e o nome que se busca nel.
    Sums { url: String, name: &'static str },
}

/// A versión de whisper.cpp que se instala (a release de GitHub e o seu SHA-256 fixado).
const WHISPER_URL: &str = "https://github.com/ggml-org/whisper.cpp/releases/download/b5907/whisper-bin-x64.zip";
const WHISPER_SHA256: &str = "ecca07b277d943e9b9f4712856a144b857271a6b333104e511e1988e80db582b";

/// O hash de `name` nun ficheiro de sumas tipo `sha256sum` (`<hash> *<nome>` ou `<hash>  <nome>`).
fn parse_sums(text: &str, name: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let mut it = l.split_whitespace();
        let hash = it.next()?;
        let file = it.next()?.trim_start_matches('*');
        (file == name && hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())).then(|| hash.to_ascii_lowercase())
    })
}

/// Onde descargar `tool`: (url do paquete, como se comproba).
fn sources(a: &ureq::Agent, tool: Tool) -> Result<(String, Checksum)> {
    match tool {
        Tool::Ffmpeg => {
            let zip = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip".to_string();
            Ok((zip.clone(), Checksum::File(format!("{zip}.sha256"))))
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
            let zip = format!("https://mkvtoolnix.download/windows/releases/{version}/mkvtoolnix-64-bit-{version}.7z");
            Ok((zip.clone(), Checksum::File(format!("{zip}.sha256"))))
        }
        Tool::Whisper => Ok((WHISPER_URL.to_string(), Checksum::Pinned(WHISPER_SHA256))),
        Tool::YtDlp => Ok((
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe".to_string(),
            Checksum::Sums { url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS".to_string(), name: "yt-dlp.exe" },
        )),
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
    let (zip_url, sum) = sources(&a, tool)?;

    let bad_sum = || EngineError::IntegrityCheck("A suma de comprobación publicada non ten o formato esperado.".into());
    let expected = match sum {
        Checksum::Pinned(h) => h.to_string(),
        Checksum::File(url) => parse_sha256(&a.get(&url).call().map_err(|e| net_err("obter a suma de comprobación", e))?.into_string()?).ok_or_else(bad_sum)?,
        Checksum::Sums { url, name } => parse_sums(&a.get(&url).call().map_err(|e| net_err("obter a suma de comprobación", e))?.into_string()?, name).ok_or_else(bad_sum)?,
    };

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
        let extracted = if zip_url.ends_with(".7z") {
            extract_7z(&tmp, &stage, progress, cancelled)
        } else if zip_url.ends_with(".exe") {
            // Un programa solto (yt-dlp): non hai que extraer nada, só poñelo no seu cartafol.
            fs::create_dir_all(&stage).and_then(|_| fs::copy(&tmp, stage.join(tool.exe())).map(|_| ())).map_err(EngineError::Io)
        } else {
            extract_zip(&tmp, &stage, progress, cancelled)
        };
        if let Err(e) = extracted {
            let _ = fs::remove_dir_all(&stage);
            return Err(e);
        }
        let target = root.join(tool.dir_name());
        // Primeiro aparta a instalación vella; se algo falla, devólvese ao seu sitio e non se perde o que funcionaba.
        let old = root.join(format!("{}.old", tool.dir_name()));
        let _ = fs::remove_dir_all(&old);
        let had_old = target.exists();
        if had_old {
            fs::rename(&target, &old).map_err(|e| EngineError::IntegrityCheck(format!("Non puiden substituír a versión instalada (está en uso?): {e}")))?;
        }
        if let Err(e) = fs::rename(&stage, &target) {
            if had_old {
                let _ = fs::rename(&old, &target);
            }
            let _ = fs::remove_dir_all(&stage);
            return Err(e.into());
        }
        let _ = fs::remove_dir_all(&old);
        let main = tool.exe();
        installed(tool, main).ok_or_else(|| EngineError::IntegrityCheck(format!("Instaleino, pero non atopo {main} dentro do paquete.")))
    })();
    let _ = fs::remove_file(&tmp);
    result
}

/// Idiomas de Tesseract que a app ofrece (código de tessdata).
pub const OCR_LANGS: [&str; 8] = ["glg", "spa", "eng", "por", "cat", "fra", "deu", "ita"];

/// Carpeta dos paquetes de idioma do OCR.
pub fn tessdata_dir() -> Option<PathBuf> {
    tools_dir().map(|d| d.join("tessdata"))
}

/// Idiomas xa descargados.
pub fn ocr_installed() -> Vec<String> {
    let Some(dir) = tessdata_dir() else { return vec![] };
    OCR_LANGS.iter().filter(|l| dir.join(format!("{l}.traineddata")).is_file()).map(|l| l.to_string()).collect()
}

/// Descarga o paquete de idioma `lang` (modelos «fast» do proxecto Tesseract, en GitHub).
/// Non hai suma de comprobación publicada: valese do HTTPS de GitHub e Tesseract rexeita un ficheiro que non sexa un modelo.
pub fn install_ocr_language(lang: &str, progress: &dyn Fn(f64), cancelled: &dyn Fn() -> bool) -> Result<PathBuf> {
    if !OCR_LANGS.contains(&lang) {
        return fail("Idioma descoñecido.");
    }
    let dir = tessdata_dir().ok_or_else(|| EngineError::IntegrityCheck("Non atopo onde gardar os programas.".into()))?;
    fs::create_dir_all(&dir)?;
    let dest = dir.join(format!("{lang}.traineddata"));
    let part = dir.join(format!("{lang}.traineddata.part"));
    let url = format!("https://github.com/tesseract-ocr/tessdata_fast/raw/main/{lang}.traineddata");
    let a = agent();
    let result = (|| -> Result<()> {
        let resp = a.get(&url).call().map_err(|e| net_err("descargar o paquete de idioma", e))?;
        let total: u64 = resp.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
        let mut reader = resp.into_reader();
        let mut out = File::create(&part)?;
        let mut buf = vec![0u8; 64 * 1024];
        let mut done: u64 = 0;
        loop {
            if cancelled() {
                return Err(EngineError::Cancelled);
            }
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])?;
            done += n as u64;
            if total > 0 {
                progress((done as f64 / total as f64).min(1.0));
            }
        }
        out.flush()?;
        drop(out);
        if done < 200_000 {
            return fail("O paquete de idioma descargado non é válido.");
        }
        fs::rename(&part, &dest)?;
        progress(1.0);
        Ok(())
    })();
    let _ = fs::remove_file(&part);
    result.map(|_| dest)
}

/// Un modelo que se descarga a parte (transcrición e quitar fondos): URL oficial, tamaño e SHA-256 fixados na app.
#[derive(Debug, Clone, Copy)]
pub struct Model {
    pub id: &'static str,
    pub file: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub size: u64,
}

/// Modelos de whisper.cpp (de OpenAI, licenza MIT, cuantizados a 5 bits no repositorio de Hugging Face de whisper.cpp: pesan
/// unha terceira parte e a súa precisión é case a mesma) e o de quitar fondos
/// (IS-Net «general use» de DIS, Apache-2.0, na release v0.0.0 de rembg).
pub const MODELS: &[Model] = &[
    Model {
        id: "whisper-base",
        file: "ggml-base-q5_1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin",
        sha256: "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898",
        size: 59_707_625,
    },
    Model {
        id: "whisper-small",
        file: "ggml-small-q5_1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin",
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
        size: 190_085_487,
    },
    Model {
        id: "whisper-medium",
        file: "ggml-medium-q5_0.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin",
        sha256: "19fea4b380c3a618ec4723c3eef2eb785ffba0d0538cf43f8f235e7b3b34220f",
        size: 539_212_467,
    },
    Model {
        id: "cutout",
        file: "isnet-general-use.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-general-use.onnx",
        sha256: "60920e99c45464f2ba57bee2ad08c919a52bbf852739e96947fbb4358c0d964a",
        size: 178_648_008,
    },
];

pub fn model(id: &str) -> Option<&'static Model> {
    MODELS.iter().find(|m| m.id == id)
}

/// Carpeta dos modelos descargados.
pub fn models_dir() -> Option<PathBuf> {
    tools_dir().map(|d| d.join("models"))
}

/// Ruta dun modelo xa descargado e completo.
pub fn model_path(id: &str) -> Option<PathBuf> {
    let m = model(id)?;
    let p = models_dir()?.join(m.file);
    (fs::metadata(&p).ok()?.len() == m.size).then_some(p)
}

/// Ids dos modelos xa descargados.
pub fn models_installed() -> Vec<String> {
    MODELS.iter().filter(|m| model_path(m.id).is_some()).map(|m| m.id.to_string()).collect()
}

/// Quita un modelo descargado.
pub fn remove_model(id: &str) -> Result<()> {
    let m = model(id).ok_or_else(|| EngineError::IntegrityCheck("Modelo descoñecido.".into()))?;
    if let Some(dir) = models_dir() {
        for name in [m.file.to_string(), format!("{}.part", m.file)] {
            let p = dir.join(name);
            if p.exists() {
                fs::remove_file(p)?;
            }
        }
    }
    Ok(())
}

/// Descarga un modelo e só o deixa no seu sitio se o SHA-256 coincide co fixado. `progress` vai de 0 a 1.
pub fn install_model(id: &str, progress: &dyn Fn(f64), cancelled: &dyn Fn() -> bool) -> Result<PathBuf> {
    let m = model(id).ok_or_else(|| EngineError::IntegrityCheck("Modelo descoñecido.".into()))?;
    let dir = models_dir().ok_or_else(|| EngineError::IntegrityCheck("Non atopo onde gardar os programas.".into()))?;
    fs::create_dir_all(&dir)?;
    let dest = dir.join(m.file);
    let part = dir.join(format!("{}.part", m.file));
    let a = agent();
    let result = (|| -> Result<()> {
        let resp = a.get(m.url).call().map_err(|e| net_err("descargar o modelo", e))?;
        let total: u64 = resp.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(m.size).max(1);
        let mut reader = resp.into_reader();
        let mut out = File::create(&part)?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 256 * 1024];
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
            let f = (done as f64 / total as f64).min(1.0);
            if f - last >= 0.004 {
                last = f;
                progress(f);
            }
        }
        out.flush()?;
        drop(out);
        let got = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>();
        if done != m.size || got != m.sha256 {
            return fail("O modelo descargado non coincide co esperado, así que non o instalo.");
        }
        crate::integrity::rename_retry(&part, &dest)?;
        progress(1.0);
        Ok(())
    })();
    let _ = fs::remove_file(&part);
    result.map(|_| dest)
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

/// Extrae un `.7z` (LZMA/LZMA2) co mesmo criterio de seguridade que o zip: nada pode saír da carpeta de destino.
fn extract_7z(path: &Path, dest: &Path, progress: &dyn Fn(Phase, f64), cancelled: &dyn Fn() -> bool) -> Result<()> {
    use std::path::Component;
    let bad = |e: sevenz_rust::Error| EngineError::IntegrityCheck(format!("O paquete descargado está danado: {e}"));
    let mut reader = sevenz_rust::SevenZReader::open(path, sevenz_rust::Password::empty()).map_err(bad)?;
    let total: u64 = reader.archive().files.iter().map(|f| f.size()).sum::<u64>().max(1);
    fs::create_dir_all(dest)?;
    let mut done: u64 = 0;
    let mut last = -1.0f64;
    let mut stopped = false;
    let res = reader.for_each_entries(|entry, data| {
        if cancelled() {
            stopped = true;
            return Ok(false);
        }
        let rel = Path::new(entry.name());
        if !rel.components().all(|c| matches!(c, Component::Normal(_))) {
            // Ruta absoluta ou con «..»: sáltase, pero hai que consumir os datos.
            std::io::copy(data, &mut std::io::sink())?;
            return Ok(true);
        }
        let out_path = dest.join(rel);
        if entry.is_directory() {
            fs::create_dir_all(&out_path)?;
            return Ok(true);
        }
        if let Some(p) = out_path.parent() {
            fs::create_dir_all(p)?;
        }
        let mut out = File::create(&out_path)?;
        std::io::copy(data, &mut out)?;
        done += entry.size();
        let p = done as f64 / total as f64;
        if p - last >= 0.01 {
            last = p;
            progress(Phase::Extract, p.min(1.0));
        }
        Ok(true)
    });
    if stopped {
        return Err(EngineError::Cancelled);
    }
    res.map_err(bad)?;
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
    fn busca_un_ficheiro_nunha_lista_de_sumas() {
        let a = "a".repeat(64);
        let b = "B".repeat(64);
        let text = format!("{a} *yt-dlp\n{b} *yt-dlp.exe\n{a}  yt-dlp_x86.exe\n");
        assert_eq!(parse_sums(&text, "yt-dlp.exe"), Some("b".repeat(64)));
        assert_eq!(parse_sums(&text, "yt-dlp"), Some(a));
        assert!(parse_sums(&text, "outro.exe").is_none());
        assert!(parse_sums("corto yt-dlp.exe", "yt-dlp.exe").is_none());
    }

    #[test]
    fn os_modelos_teñen_datos_validos() {
        for m in MODELS {
            assert_eq!(m.sha256.len(), 64, "{}", m.id);
            assert!(m.sha256.chars().all(|c| c.is_ascii_hexdigit()), "{}", m.id);
            assert!(m.url.starts_with("https://"), "{}", m.id);
            assert!(m.size > 1_000_000, "{}", m.id);
            assert!(model(m.id).is_some());
        }
        assert!(model("non-existe").is_none());
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
        let mut phases = seen.lock().unwrap().clone();
        phases.dedup();
        assert_eq!(phases, vec![Phase::Download, Phase::Verify, Phase::Extract]);
        let out = std::process::Command::new(&exe).arg("--version").output().unwrap();
        assert!(String::from_utf8_lossy(&out.stdout).contains("mkvmerge"));
        assert!(installed(Tool::MkvToolNix, "mkvmerge.exe").is_some());
        remove(Tool::MkvToolNix).unwrap();
        assert!(installed(Tool::MkvToolNix, "mkvmerge.exe").is_none());
    }
}
