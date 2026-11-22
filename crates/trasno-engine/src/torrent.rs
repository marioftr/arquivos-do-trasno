//! Descargar torrents (ligazóns magnet ou ficheiros .torrent) co cliente BitTorrent integrado (librqbit, Apache-2.0): sen programas externos.
//! Só se descarga: ao rematar o torrent déixase de compartir, e non se abre ningún porto de entrada (só conexións de saída).
//! Todo baixa a un cartafol temporal dentro do destino e, cando está completo, móvese ao destino sen pisar nada.

use crate::error::{EngineError, Result};
use crate::ffmpeg;
use librqbit::api::TorrentIdOrHash;
use librqbit::{AddTorrent, AddTorrentOptions, AddTorrentResponse, Session, SessionOptions};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Canto se agarda a que apareza alguén que teña o torrent (e os seus datos de ficheiros) antes de rendirse.
const FIND_TIMEOUT: Duration = Duration::from_secs(90);
/// Se non chega un só byte en tanto tempo, o torrent considérase morto.
const STALL_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// Tamaño máximo dun ficheiro .torrent que se le.
const MAX_TORRENT_FILE: u64 = 10 * 1024 * 1024;

fn bad<T>(msg: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(msg.to_string()))
}

#[derive(Debug, Clone, Serialize)]
pub struct TorrentFile {
    pub index: usize,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TorrentInfo {
    pub name: String,
    pub info_hash: String,
    pub total: u64,
    pub files: Vec<TorrentFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TorrentReport {
    pub name: String,
    /// O que quedou no destino: un cartafol (torrent de varios ficheiros) ou un ficheiro.
    pub outputs: Vec<PathBuf>,
    pub bytes_written: u64,
}

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Builder::new_multi_thread().worker_threads(2).thread_name("trasno-torrent").enable_all().build().expect("non puiden arrancar o motor de torrents"))
}

async fn session() -> Result<Arc<Session>> {
    static SESSION: tokio::sync::Mutex<Option<Arc<Session>>> = tokio::sync::Mutex::const_new(None);
    let mut guard = SESSION.lock().await;
    if let Some(s) = guard.as_ref() {
        return Ok(s.clone());
    }
    let dir = std::env::temp_dir().join("arquivos-do-trasno-torrents");
    std::fs::create_dir_all(&dir)?;
    // Sen `listen`: non se abre ningún porto (sen avisos do firewall); sen persistencia: nada queda gardado entre execucións.
    let s = Session::new_with_opts(dir, SessionOptions::default()).await.map_err(|e| EngineError::IntegrityCheck(format!("Non puiden arrancar o cliente de torrents: {e}")))?;
    *guard = Some(s.clone());
    Ok(s)
}

/// De onde vén un torrent.
enum Source {
    Magnet(String),
    Url(String),
    File(PathBuf),
}

fn classify(source: &str) -> Result<Source> {
    let s = source.trim();
    if s.starts_with("magnet:?") {
        if !s.contains("xt=urn:btih:") && !s.contains("xt=urn:btmh:") {
            return bad("Esta ligazón magnet non ten identificador de torrent.");
        }
        return Ok(Source::Magnet(s.to_string()));
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        let path = s.split(['?', '#']).next().unwrap_or(s).to_ascii_lowercase();
        if !path.ends_with(".torrent") {
            return bad("Para un torrent, pega unha ligazón magnet ou unha ligazón a un ficheiro .torrent.");
        }
        return Ok(Source::Url(s.to_string()));
    }
    let p = PathBuf::from(s);
    if p.is_file() && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("torrent")) {
        return Ok(Source::File(p));
    }
    bad("Para un torrent, pega unha ligazón magnet, unha ligazón a un .torrent ou elixe un ficheiro .torrent.")
}

/// Ligazóns magnet, ligazóns a un .torrent ou ficheiros .torrent: o que acepta esta función.
pub fn is_torrent_source(source: &str) -> bool {
    classify(source).is_ok()
}

/// Os ficheiros .torrent xa lidos (cos datos dos ficheiros) por ligazón: así descargar non repite a busca.
fn cache() -> &'static Mutex<HashMap<String, Vec<u8>>> {
    static C: OnceLock<Mutex<HashMap<String, Vec<u8>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn add_for(source: &str) -> Result<AddTorrent<'static>> {
    if let Some(bytes) = cache().lock().unwrap_or_else(|e| e.into_inner()).get(source.trim()) {
        return Ok(AddTorrent::from_bytes(bytes.clone()));
    }
    Ok(match classify(source)? {
        Source::Magnet(m) => AddTorrent::from_url(m),
        Source::Url(u) => AddTorrent::from_url(u),
        Source::File(p) => {
            if std::fs::metadata(&p)?.len() > MAX_TORRENT_FILE {
                return bad("Ese ficheiro .torrent é demasiado grande.");
            }
            AddTorrent::from_bytes(std::fs::read(&p)?)
        }
    })
}

fn fail(e: impl std::fmt::Display) -> EngineError {
    EngineError::IntegrityCheck(format!("O torrent deu un erro: {e}"))
}

/// Agarda a `fut` ata `limit`, pero deixa cancelar mentres tanto (buscar xente que teña un magnet pode tardar).
async fn with_cancel<T>(fut: impl std::future::Future<Output = T>, limit: Duration) -> Result<T> {
    tokio::pin!(fut);
    let start = std::time::Instant::now();
    loop {
        tokio::select! {
            r = &mut fut => return Ok(r),
            _ = tokio::time::sleep(Duration::from_millis(300)) => {
                if ffmpeg::cancelled_here() {
                    return Err(EngineError::Cancelled);
                }
                if start.elapsed() > limit {
                    return Err(nobody());
                }
            }
        }
    }
}

fn nobody() -> EngineError {
    EngineError::IntegrityCheck("Non atopei ninguén que teña este torrent. Pode estar morto ou non haber conexión; proba máis tarde.".into())
}

/// Mira un torrent (nome e ficheiros) sen descargar nada. Cunha ligazón magnet hai que atopar antes xente que o teña (precisa internet).
pub fn inspect(source: &str) -> Result<TorrentInfo> {
    let add = add_for(source)?;
    let key = source.trim().to_string();
    runtime().block_on(async move {
        let s = session().await?;
        let opts = AddTorrentOptions { list_only: true, ..Default::default() };
        let resp = tokio::time::timeout(FIND_TIMEOUT, s.add_torrent(add, Some(opts))).await.map_err(|_| nobody())?.map_err(fail)?;
        let AddTorrentResponse::ListOnly(r) = resp else { return bad("Este torrent xa se está a descargar.") };
        let files: Vec<TorrentFile> = r.info.iter_file_details().enumerate().filter(|(_, f)| !f.attrs().padding).map(|(index, f)| TorrentFile { index, path: f.filename.to_vec().join("/"), size: f.len }).collect();
        if files.is_empty() {
            return bad("Este torrent non ten ficheiros.");
        }
        let total = files.iter().map(|f| f.size).sum();
        let name = clean_name(&r.info.name_or_else(|| "torrent").to_string());
        {
            let mut c = cache().lock().unwrap_or_else(|e| e.into_inner());
            if c.len() >= 8 {
                c.clear();
            }
            c.insert(key, r.torrent_bytes.to_vec());
        }
        Ok(TorrentInfo { name, info_hash: r.info_hash.as_string(), total, files })
    })
}

/// Nome seguro para usar de ficheiro ou cartafol (sen separadores nin caracteres que Windows non admite).
fn clean_name(name: &str) -> String {
    let n: String = name.chars().map(|c| if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() { '_' } else { c }).collect();
    let n = n.trim().trim_matches('.').trim().to_string();
    if n.is_empty() {
        "torrent".to_string()
    } else {
        n
    }
}

/// Un nome libre en `dir`: «nome», «nome (2)»… (a extensión, se hai, queda ao final).
fn free_name(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let p = Path::new(name);
    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| name.to_string());
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    for n in 2..10_000 {
        let c = dir.join(format!("{stem} ({n}){ext}"));
        if !c.exists() {
            return c;
        }
    }
    first
}

/// Quita os cartafoles baleiros de dentro de `dir` (non `dir`).
fn remove_empty_dirs(dir: &Path) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        if e.path().is_dir() {
            remove_empty_dirs(&e.path());
            let _ = std::fs::remove_dir(e.path());
        }
    }
}

fn dir_size(p: &Path) -> u64 {
    match std::fs::metadata(p) {
        Ok(m) if m.is_file() => m.len(),
        Ok(m) if m.is_dir() => std::fs::read_dir(p).map(|rd| rd.flatten().map(|e| dir_size(&e.path())).sum()).unwrap_or(0),
        _ => 0,
    }
}

/// Descarga o torrent a `dir` (todos os ficheiros ou só os de `files`, polo número que dá `inspect`). O avance (0 a 1) vai á interface.
pub fn download(source: &str, dir: &Path, files: Option<Vec<usize>>) -> Result<TorrentReport> {
    download_with(source, dir, files, &[])
}

/// Como `download`, con compañeiros coñecidos de partida (para probas e redes locais).
pub fn download_with(source: &str, dir: &Path, files: Option<Vec<usize>>, peers: &[SocketAddr]) -> Result<TorrentReport> {
    if let Some(f) = &files {
        if f.is_empty() {
            return bad("Elixe polo menos un ficheiro do torrent.");
        }
    }
    let add = add_for(source)?;
    std::fs::create_dir_all(dir)?;
    if !dir.is_dir() {
        return bad("O cartafol de destino non existe.");
    }
    let work = tempfile::Builder::new().prefix(".trasno-torrent-").tempdir_in(dir)?;
    let work_path = work.path().to_path_buf();
    let peers = peers.to_vec();

    let (name, multi, skipped, outcome) = runtime().block_on(async {
        let s = session().await?;
        let opts = AddTorrentOptions { output_folder: Some(work_path.to_string_lossy().to_string()), only_files: files.clone(), initial_peers: (!peers.is_empty()).then_some(peers), ..Default::default() };
        let resp = with_cancel(s.add_torrent(add, Some(opts)), FIND_TIMEOUT).await?.map_err(fail)?;
        let (id, handle) = match resp {
            AddTorrentResponse::Added(id, h) => (id, h),
            _ => return bad("Este torrent xa se está a descargar."),
        };
        let name = handle.name().map(|n| clean_name(&n)).unwrap_or_else(|| "torrent".to_string());
        // Un torrent de varios ficheiros baixa solto no cartafol: despois agrúpase nun cartafol co nome do torrent.
        let multi = handle.with_metadata(|m| m.info.info().length.is_none()).unwrap_or(false);
        // As pezas das beiras poden traer un cacho dos ficheiros que non se elixiron: ao rematar bórranse.
        let skipped: Vec<PathBuf> = match &files {
            Some(keep) => handle.with_metadata(|m| m.info.iter_file_details().enumerate().filter(|(i, _)| !keep.contains(i)).map(|(_, f)| f.filename.to_pathbuf()).collect()).unwrap_or_default(),
            None => Vec::new(),
        };
        let mut last_bytes = 0u64;
        let mut last_move = std::time::Instant::now();
        let outcome: Result<()> = loop {
            if ffmpeg::cancelled_here() {
                break Err(EngineError::Cancelled);
            }
            let st = handle.stats();
            if let Some(e) = &st.error {
                break Err(fail(e));
            }
            if st.finished && st.total_bytes > 0 {
                break Ok(());
            }
            if st.total_bytes > 0 {
                ffmpeg::report_progress((st.progress_bytes as f64 / st.total_bytes as f64 * 0.98).min(0.98));
            }
            if st.progress_bytes != last_bytes {
                last_bytes = st.progress_bytes;
                last_move = std::time::Instant::now();
            } else if last_move.elapsed() > STALL_TIMEOUT {
                break Err(EngineError::IntegrityCheck("O torrent levou moito tempo sen avanzar: non hai ninguén que o comparta agora.".into()));
            }
            tokio::time::sleep(Duration::from_millis(400)).await;
        };
        // Pase o que pase, déixase de compartir e pechanse os ficheiros (bórrase o torrent da sesión, non os datos).
        let _ = s.delete(TorrentIdOrHash::Id(id), false).await;
        Ok((name, multi, skipped, outcome))
    })?;
    outcome?;
    for p in &skipped {
        let _ = std::fs::remove_file(work_path.join(p));
    }
    remove_empty_dirs(&work_path);

    // Todo chegou ao cartafol temporal: móvese ao destino co nome que dea o torrent, sen pisar o que haxa.
    let mut outputs = Vec::new();
    let mut bytes = 0u64;
    let mut entries: Vec<_> = std::fs::read_dir(&work_path)?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    if multi {
        let target = free_name(dir, &name);
        std::fs::create_dir(&target)?;
        for e in entries {
            bytes += dir_size(&e.path());
            std::fs::rename(e.path(), target.join(e.file_name()))?;
        }
        outputs.push(target);
    } else {
        for e in entries {
            let target = free_name(dir, &clean_name(&e.file_name().to_string_lossy()));
            bytes += dir_size(&e.path());
            std::fs::rename(e.path(), &target)?;
            outputs.push(target);
        }
    }
    if outputs.is_empty() {
        return Err(EngineError::EmptyOrCorruptOutput);
    }
    ffmpeg::report_progress(1.0);
    Ok(TorrentReport { name, outputs, bytes_written: bytes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn so_acepta_magnet_torrent_e_ficheiros_torrent() {
        assert!(is_torrent_source("magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=exemplo"));
        assert!(is_torrent_source("https://exemplo.org/a/b.torrent"));
        assert!(is_torrent_source("https://exemplo.org/a/b.torrent?x=1"));
        assert!(!is_torrent_source("https://exemplo.org/video"));
        assert!(!is_torrent_source("magnet:?dn=sen-hash"));
        assert!(!is_torrent_source("C:\\nunca\\existe.torrent"));
        assert!(!is_torrent_source("ftp://exemplo.org/a.torrent"));
    }

    #[test]
    fn os_nomes_do_torrent_quedan_seguros() {
        assert_eq!(clean_name("a/b\\c:d*e?f"), "a_b_c_d_e_f");
        assert_eq!(clean_name("..."), "torrent");
        assert_eq!(clean_name("  Peli (2024)  "), "Peli (2024)");
    }

    #[test]
    fn un_nome_ocupado_pasa_a_ter_numero() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("x.mkv"), b"1").unwrap();
        std::fs::create_dir(d.path().join("pasta")).unwrap();
        assert_eq!(free_name(d.path(), "x.mkv"), d.path().join("x (2).mkv"));
        assert_eq!(free_name(d.path(), "pasta"), d.path().join("pasta (2)"));
        assert_eq!(free_name(d.path(), "libre.txt"), d.path().join("libre.txt"));
    }
}
