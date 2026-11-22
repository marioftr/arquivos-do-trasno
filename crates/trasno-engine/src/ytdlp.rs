//! Descarga de vídeo e audio por ligazón con yt-dlp (extensión opcional: o programa descárgase a petición, ver `tools_install`).
//!
//! Unha ligazón pode ser un vídeo ou unha lista: a lista amósase enteira e descárganse os vídeos que se marquen, un tras outro.
//! O que baixa vai primeiro a un cartafol temporal dentro do de destino e só o resultado final se move a el, así que unha
//! descarga cancelada ou fallida non deixa ficheiros a medias. FFmpeg (o da app) únese e converte o son cando fai falta.

use crate::error::{EngineError, Result};
use crate::ffmpeg::{self, run_external_watch};
use crate::tools_install::{self, Tool};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Máximo de vídeos dunha lista que se amosan.
const MAX_ENTRIES: usize = 500;

/// O yt-dlp: `TRASNO_YTDLP_PATH`, o PATH ou o que descargou a app (con `TRASNO_ONLY_OWN_TOOLS`, só isto último: así o usan as probas).
pub fn ytdlp_path() -> Option<PathBuf> {
    let own = || tools_install::installed(Tool::YtDlp, Tool::YtDlp.exe());
    if std::env::var_os("TRASNO_ONLY_OWN_TOOLS").is_some() {
        return own();
    }
    if let Some(p) = std::env::var_os("TRASNO_YTDLP_PATH").map(PathBuf::from).filter(|p| p.is_file()) {
        return Some(p);
    }
    which::which("yt-dlp").ok().or_else(own)
}

pub fn is_available() -> bool {
    ytdlp_path().is_some()
}

/// Comproba que é unha ligazón web normal (http ou https, sen espazos nin caracteres de control).
pub fn clean_url(url: &str) -> Result<String> {
    let u = url.trim();
    let ok = (u.starts_with("https://") || u.starts_with("http://")) && u.len() <= 2048 && u.len() > 10 && !u.chars().any(|c| c.is_whitespace() || c.is_control());
    if ok {
        Ok(u.to_string())
    } else {
        Err(EngineError::IntegrityCheck("Iso non parece unha ligazón web: ten que empezar por http:// ou https://.".into()))
    }
}

fn base_command(exe: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(exe);
    // Sen o que teña o usuario configurado por el: a app controla o que pasa. Cada vídeo descárgase pola súa propia ligazón:
    // `--no-playlist` evita que unha ligazón de vídeo traia a lista á que pertence.
    cmd.args(["--ignore-config", "--no-warnings", "--no-playlist", "--socket-timeout", "30"]);
    cmd
}

/// Unha calidade de vídeo dispoñible.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct VideoOption {
    pub height: u32,
    pub fps: Option<f64>,
    /// Códec que se descargaría a esa altura: «H.264», «VP9», «AV1»…
    pub codec: String,
    /// Tamaño aproximado (vídeo e audio), se o sitio o di.
    pub size: Option<u64>,
}

/// Un vídeo dunha lista.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Entry {
    pub url: String,
    pub title: String,
    pub duration: Option<f64>,
    pub uploader: Option<String>,
}

/// O que se sabe dunha ligazón antes de descargala.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MediaInfo {
    pub title: String,
    /// Duración en segundos, se se coñece.
    pub duration: Option<f64>,
    pub uploader: Option<String>,
    /// Sitio web (YouTube, Vimeo…).
    pub platform: Option<String>,
    /// Data de subida, «AAAAMMDD».
    pub upload_date: Option<String>,
    pub views: Option<u64>,
    /// Ligazón da miniatura, se o sitio ten unha.
    pub thumbnail: Option<String>,
    /// Alturas de vídeo que hai (de maior a menor), por exemplo 1080, 720, 360.
    pub heights: Vec<u32>,
    /// Calidades de vídeo con detalle (de maior a menor).
    pub video: Vec<VideoOption>,
    /// Calidade do mellor audio, en kbps, e o seu códec.
    pub audio_kbps: Option<u32>,
    pub audio_codec: Option<String>,
    /// A ligazón é unha lista de reprodución: `entries` ten os seus vídeos.
    pub playlist: bool,
    pub entries: Vec<Entry>,
    /// Vídeos que ten a lista (pode haber máis dos que se amosan).
    pub total: u32,
}

fn codec_name(c: &str) -> String {
    let l = c.to_lowercase();
    if l.starts_with("avc") || l.starts_with("h264") {
        "H.264".into()
    } else if l.starts_with("vp9") || l.starts_with("vp09") {
        "VP9".into()
    } else if l.starts_with("av01") || l.starts_with("av1") {
        "AV1".into()
    } else if l.starts_with("hev") || l.starts_with("hvc") || l.starts_with("h265") {
        "H.265".into()
    } else if l.starts_with("vp8") {
        "VP8".into()
    } else if l.starts_with("mp4a") || l.starts_with("aac") {
        "AAC".into()
    } else if l.starts_with("opus") {
        "Opus".into()
    } else if l.starts_with("mp3") {
        "MP3".into()
    } else if l.starts_with("vorbis") {
        "Vorbis".into()
    } else {
        c.split('.').next().unwrap_or(c).to_string()
    }
}

fn size_of(f: &serde_json::Value) -> Option<u64> {
    f["filesize"].as_u64().or_else(|| f["filesize_approx"].as_f64().map(|v| v as u64)).filter(|n| *n > 0)
}

/// Calidades de vídeo (a mellor de cada altura, preferindo H.264) e a calidade do mellor audio.
fn read_formats(item: &serde_json::Value) -> (Vec<VideoOption>, Option<(u32, String, Option<u64>)>) {
    let formats = item["formats"].as_array().cloned().unwrap_or_default();
    let is_video = |f: &&serde_json::Value| f["vcodec"].as_str().is_some_and(|c| c != "none") && f["height"].as_u64().unwrap_or(0) > 0;
    let is_audio = |f: &&serde_json::Value| f["acodec"].as_str().is_some_and(|c| c != "none") && f["vcodec"].as_str().is_none_or(|c| c == "none");
    let audio = formats
        .iter()
        .filter(is_audio)
        .max_by(|a, b| a["abr"].as_f64().unwrap_or(0.0).total_cmp(&b["abr"].as_f64().unwrap_or(0.0)))
        .map(|f| (f["abr"].as_f64().unwrap_or(0.0).round() as u32, codec_name(f["acodec"].as_str().unwrap_or("")), size_of(f)));
    let mut heights: Vec<u32> = formats.iter().filter(is_video).filter_map(|f| f["height"].as_u64()).map(|h| h as u32).collect();
    heights.sort_unstable_by(|a, b| b.cmp(a));
    heights.dedup();
    let video = heights
        .into_iter()
        .filter_map(|h| {
            let same: Vec<&serde_json::Value> = formats.iter().filter(is_video).filter(|f| f["height"].as_u64() == Some(h as u64)).collect();
            let tbr = |f: &&serde_json::Value| f["tbr"].as_f64().unwrap_or(0.0);
            let family = |f: &serde_json::Value| codec_name(f["vcodec"].as_str().unwrap_or(""));
            // A esa altura prefírese H.264, que reproduce todo; se non hai, o de máis bitrate.
            let fam = if same.iter().any(|f| family(f) == "H.264") { "H.264".to_string() } else { family(same.iter().max_by(|a, b| tbr(a).total_cmp(&tbr(b)))?) };
            let best = *same.iter().filter(|f| family(f) == fam).max_by(|a, b| tbr(a).total_cmp(&tbr(b)))?;
            let size = size_of(best).map(|v| v + audio.as_ref().and_then(|a| a.2).unwrap_or(0));
            Some(VideoOption { height: h, fps: best["fps"].as_f64().filter(|f| *f > 0.0), codec: fam, size })
        })
        .collect();
    (video, audio)
}

/// Le o JSON que devolve `yt-dlp -J`.
fn parse_info(json: &str) -> Result<MediaInfo> {
    let j: serde_json::Value = serde_json::from_str(json.trim()).map_err(|_| EngineError::ExternalFailed { program: "yt-dlp", detail: "non entendín a resposta".into() })?;
    let playlist = j["_type"] == "playlist";
    let text = |v: &serde_json::Value, k: &str| v[k].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let entries: Vec<Entry> = if playlist {
        j["entries"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(|e| {
                        let url = e["url"].as_str().or_else(|| e["webpage_url"].as_str())?.to_string();
                        if !(url.starts_with("http://") || url.starts_with("https://")) {
                            return None;
                        }
                        Some(Entry { url, title: text(e, "title").unwrap_or_default(), duration: e["duration"].as_f64(), uploader: text(e, "uploader").or_else(|| text(e, "channel")) })
                    })
                    .take(MAX_ENTRIES)
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let total = if playlist { j["playlist_count"].as_u64().map(|n| n as u32).unwrap_or(entries.len() as u32).max(entries.len() as u32) } else { 1 };
    let (video, audio) = if playlist { (Vec::new(), None) } else { read_formats(&j) };
    Ok(MediaInfo {
        title: text(&j, "title").unwrap_or_default(),
        duration: j["duration"].as_f64(),
        uploader: text(&j, "uploader").or_else(|| text(&j, "channel")),
        platform: text(&j, "extractor_key").or_else(|| text(&j, "extractor")),
        upload_date: text(&j, "upload_date"),
        views: j["view_count"].as_u64(),
        thumbnail: text(&j, "thumbnail").filter(|u| u.starts_with("https://")),
        heights: video.iter().map(|v| v.height).collect(),
        video,
        audio_kbps: audio.as_ref().map(|a| a.0).filter(|k| *k > 0),
        audio_codec: audio.map(|a| a.1),
        playlist,
        entries,
        total,
    })
}

/// A última frase de erro que dixo yt-dlp, sen o prefixo «ERROR:».
fn error_detail(stderr: &str) -> String {
    let line = stderr.lines().rev().find(|l| l.contains("ERROR:")).or_else(|| stderr.lines().rev().find(|l| !l.trim().is_empty())).unwrap_or("sen detalle");
    let line = line.split("ERROR:").last().unwrap_or(line).trim();
    // Quítase o «[sitio] id:» do principio, que non axuda.
    let line = match line.strip_prefix('[').and_then(|r| r.split_once("] ")) {
        Some((_, rest)) => rest.split_once(": ").map(|(_, m)| m).unwrap_or(rest),
        None => line,
    };
    line.trim().chars().take(300).collect()
}

/// O que non deixou facer yt-dlp, dito en galego cando é un motivo coñecido (DRM, privado, sesión, país, borrado…); doutro xeito,
/// o detalle técnico tal como o deu.
fn failure(stderr: &str) -> EngineError {
    let low = stderr.to_lowercase();
    let has = |needles: &[&str]| needles.iter().any(|n| low.contains(n));
    let why = if has(&["drm"]) {
        "drm"
    } else if has(&["private video", "video is private"]) {
        "private"
    } else if has(&["age-restricted", "confirm your age", "inappropriate for some users"]) {
        "age"
    } else if has(&["sign in", "log in", "login required", "not a bot", "cookies"]) {
        "login"
    } else if has(&["not available in your country", "geo restrict", "geo-restrict", "blocked it in your country"]) {
        "geo"
    } else if has(&["live event will begin", "premieres in", "will begin in"]) {
        "live"
    } else if has(&["video unavailable", "has been removed", "does not exist", "http error 404", "no longer available", "been terminated", "was deleted"]) {
        "gone"
    } else if has(&["unsupported url", "no video formats found", "no video could be found"]) {
        "unsupported"
    } else if has(&["getaddrinfo", "timed out", "name resolution", "network is unreachable", "connection reset", "unable to download webpage", "connection refused"]) {
        "network"
    } else {
        return EngineError::ExternalFailed { program: "yt-dlp", detail: error_detail(stderr) };
    };
    EngineError::Unavailable(why)
}

/// Comproba unha ligazón: que é, as calidades dispoñibles e, se é unha lista, os seus vídeos.
pub fn info(url: &str) -> Result<MediaInfo> {
    let url = clean_url(url)?;
    let exe = ytdlp_path().ok_or(EngineError::ExtensionMissing("yt-dlp"))?;
    let mut cmd = base_command(&exe);
    cmd.args(["--flat-playlist", "--playlist-end", &MAX_ENTRIES.to_string(), "--skip-download", "-J", "--", &url]);
    let out = run_external_watch(cmd, std::time::Duration::from_secs(180), None)?;
    if out.code != Some(0) {
        return Err(failure(&out.stderr));
    }
    parse_info(&out.stdout)
}

/// Baixa a miniatura dun vídeo a `dir` (ata 4 MB, só https) e devolve a ruta, para que a interface a amose sen abrir a rede ao webview.
pub fn fetch_thumbnail(url: &str, dir: &Path) -> Result<PathBuf> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let url = clean_url(url)?;
    if !url.starts_with("https://") {
        return Err(EngineError::ExternalFailed { program: "yt-dlp", detail: "a miniatura ten que vir por https".into() });
    }
    let name: String = Sha256::digest(url.as_bytes()).iter().take(8).map(|b| format!("{b:02x}")).collect();
    let out = dir.join(format!("miniatura-{name}.img"));
    if out.is_file() {
        return Ok(out);
    }
    let resp = ureq::AgentBuilder::new()
        .user_agent("ArquivosDoTrasno")
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout_read(std::time::Duration::from_secs(15))
        .redirects(3)
        .build()
        .get(&url)
        .call()
        .map_err(|e| EngineError::ExternalFailed { program: "yt-dlp", detail: format!("non puiden baixar a miniatura: {e}") })?;
    let mut bytes = Vec::new();
    resp.into_reader().take(4 * 1024 * 1024).read_to_end(&mut bytes)?;
    if bytes.len() < 64 {
        return Err(EngineError::ExternalFailed { program: "yt-dlp", detail: "a miniatura chegou baleira".into() });
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(&out, bytes)?;
    Ok(out)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DownloadOptions {
    /// «video» ou «audio».
    pub kind: String,
    /// Altura máxima do vídeo (480, 720, 1080…). Sen ela, a mellor.
    #[serde(default)]
    pub max_height: Option<u32>,
    /// Contedor do vídeo: «mp4» (o que reproduce todo) ou «mkv» (garda calquera códec sen perder nada).
    #[serde(default = "default_container")]
    pub container: String,
    /// Formato do audio: mp3, m4a, opus, flac ou wav.
    #[serde(default = "default_audio")]
    pub audio_format: String,
    /// Bitrate do audio en kbps (en MP3, M4A e Opus). Sen el, a mellor calidade.
    #[serde(default)]
    pub audio_kbps: Option<u32>,
}

fn default_audio() -> String {
    "mp3".to_string()
}

fn default_container() -> String {
    "mp4".to_string()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadReport {
    pub output_path: PathBuf,
    pub title: String,
    pub bytes_written: u64,
}

/// Un vídeo dunha lista que non se puido descargar.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Failure {
    pub url: String,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchReport {
    pub done: Vec<DownloadReport>,
    pub failed: Vec<Failure>,
}

/// Os argumentos que dependen do que se pide (vídeo ou audio).
fn kind_args(o: &DownloadOptions) -> Result<Vec<String>> {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    match o.kind.as_str() {
        "video" => {
            let cap = match o.max_height {
                Some(h) if (100..=8640).contains(&h) => format!("[height<={h}]"),
                Some(_) => return Err(EngineError::IntegrityCheck("A calidade que pediches non é válida.".into())),
                None => String::new(),
            };
            let mut v = s(&["-f"]);
            v.push(format!("bv*{cap}+ba/b{cap}"));
            match o.container.as_str() {
                // A mellor resolución e, a igualdade, H.264 e AAC, que reproduce todo.
                "mp4" => v.extend(s(&["-S", "res,vcodec:h264,acodec:m4a", "--merge-output-format", "mp4"])),
                // Sen preferencia de códec: o que máis calidade dea, sen recodificar.
                "mkv" => v.extend(s(&["-S", "res,fps", "--merge-output-format", "mkv"])),
                _ => return Err(EngineError::IntegrityCheck("O formato de vídeo que pediches non é válido.".into())),
            }
            Ok(v)
        }
        "audio" => {
            if !matches!(o.audio_format.as_str(), "mp3" | "m4a" | "opus" | "flac" | "wav") {
                return Err(EngineError::IntegrityCheck("O formato de audio que pediches non é válido.".into()));
            }
            // Os formatos sen perdas non teñen bitrate: a calidade é a da fonte.
            let quality = match (o.audio_kbps, o.audio_format.as_str()) {
                (Some(k), "mp3" | "m4a" | "opus") if (32..=320).contains(&k) => format!("{k}K"),
                (Some(_), "mp3" | "m4a" | "opus") => return Err(EngineError::IntegrityCheck("A calidade de audio que pediches non é válida.".into())),
                _ => "0".to_string(),
            };
            Ok(s(&["-f", "ba/b", "-x", "--audio-format", &o.audio_format, "--audio-quality", &quality, "--embed-metadata"]))
        }
        _ => Err(EngineError::IntegrityCheck("Escolle descargar vídeo ou audio.".into())),
    }
}

/// «TRASNO-PROGRESO  45.3%» → 0,453.
fn parse_percent(line: &str) -> Option<f64> {
    let rest = line.strip_prefix("TRASNO-PROGRESO")?.trim();
    let n: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    n.parse::<f64>().ok().map(|p| (p / 100.0).clamp(0.0, 1.0))
}

/// Descarga a ligazón a `dir` e devolve o ficheiro que quedou.
pub fn download(url: &str, dir: &Path, o: &DownloadOptions) -> Result<DownloadReport> {
    download_part(url, dir, o, 0.0, 1.0)
}

/// Como `download`, pero o avance que se informa vai só de `base` a `base + span` (para as listas).
fn download_part(url: &str, dir: &Path, o: &DownloadOptions, base: f64, span: f64) -> Result<DownloadReport> {
    let url = clean_url(url)?;
    let exe = ytdlp_path().ok_or(EngineError::ExtensionMissing("yt-dlp"))?;
    let ffmpeg_bin = ffmpeg::ffmpeg_path()?;
    let extra = kind_args(o)?;
    std::fs::create_dir_all(dir)?;
    if !dir.is_dir() {
        return Err(EngineError::IntegrityCheck("O cartafol de destino non existe.".into()));
    }
    // Todo o que se baixa vai a un cartafol temporal dentro do destino (mesma unidade: mover é instantáneo).
    let work = tempfile::Builder::new().prefix(".trasno-descarga-").tempdir_in(dir)?;
    let listing = work.path().join("resultado.txt");

    let mut cmd = base_command(&exe);
    cmd.arg("--no-mtime").arg("--windows-filenames").arg("--newline").arg("--retries").arg("5").arg("--ffmpeg-location").arg(&ffmpeg_bin);
    cmd.arg("-P").arg(format!("home:{}", dir.display())).arg("-P").arg(format!("temp:{}", work.path().display()));
    cmd.arg("-o").arg("%(title).150B [%(id)s].%(ext)s");
    cmd.arg("--progress-template").arg("download:TRASNO-PROGRESO %(progress._percent_str)s");
    cmd.arg("--print-to-file").arg("after_move:%(filepath)s\t%(title)s").arg(&listing);
    cmd.args(&extra);
    cmd.arg("--").arg(&url);

    // O vídeo baixa en dúas partes (imaxe e son) que se unen; o audio, nunha.
    let parts: f64 = if o.kind == "video" { 2.0 } else { 1.0 };
    let state = Arc::new(std::sync::Mutex::new((0.0f64, 0.0f64))); // (partes completas, última porcentaxe)
    let st = state.clone();
    let watch: ffmpeg::Watch = Arc::new(move |line: &str| {
        let Some(p) = parse_percent(line) else { return };
        let mut s = st.lock().unwrap_or_else(|e| e.into_inner());
        // Se a porcentaxe volve atrás, empezou outra parte.
        if p + 0.5 < s.1 {
            s.0 += 1.0;
        }
        s.1 = p;
        ffmpeg::report_progress(base + span * ((s.0 + p) / parts).min(0.97));
    });
    let out = run_external_watch(cmd, std::time::Duration::from_secs(6 * 3600), Some(watch))?;
    if out.code != Some(0) {
        return Err(failure(&out.stderr));
    }

    let text = std::fs::read_to_string(&listing).unwrap_or_default();
    let last = text.lines().rev().find(|l| !l.trim().is_empty()).ok_or(EngineError::EmptyOrCorruptOutput)?;
    let (path, title) = last.split_once('\t').unwrap_or((last, ""));
    let output_path = PathBuf::from(path.trim());
    // O resultado ten que estar dentro do cartafol pedido.
    let inside = std::fs::canonicalize(&output_path).ok().zip(std::fs::canonicalize(dir).ok()).is_some_and(|(f, d)| f.starts_with(d));
    if !inside {
        return Err(EngineError::EmptyOrCorruptOutput);
    }
    let bytes_written = crate::integrity::assert_non_empty_file(&output_path)?;
    ffmpeg::report_progress(base + span);
    Ok(DownloadReport { output_path, title: title.trim().to_string(), bytes_written })
}

/// Descarga varias ligazóns (os vídeos marcados dunha lista), unha tras outra, a `dir`. Se un falla, seguen os demais e queda
/// anotado; se se cancela, para (o que xa baixou queda). O avance vai de 0 a 1 sobre o total.
pub fn download_many(urls: &[String], dir: &Path, o: &DownloadOptions) -> Result<BatchReport> {
    if urls.is_empty() {
        return Err(EngineError::IntegrityCheck("Non hai nada que descargar.".into()));
    }
    let n = urls.len() as f64;
    let mut report = BatchReport { done: Vec::new(), failed: Vec::new() };
    for (i, url) in urls.iter().enumerate() {
        if ffmpeg::is_cancelled() {
            return Err(EngineError::Cancelled);
        }
        match download_part(url, dir, o, i as f64 / n, 1.0 / n) {
            Ok(r) => report.done.push(r),
            Err(EngineError::Cancelled) => return Err(EngineError::Cancelled),
            // Sen o programa ou o FFmpeg non vale a pena seguir.
            Err(e @ EngineError::ExtensionMissing(_)) => return Err(e),
            Err(e) if urls.len() == 1 => return Err(e),
            Err(e) => report.failed.push(Failure { url: url.clone(), detail: e.wire() }),
        }
    }
    if report.done.is_empty() {
        let detail = report.failed.first().map(|f| f.detail.clone()).unwrap_or_default();
        return Err(EngineError::ExternalFailed { program: "yt-dlp", detail });
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn so_acepta_ligazons_web_normais() {
        assert_eq!(clean_url("  https://example.com/v?id=1 ").unwrap(), "https://example.com/v?id=1");
        assert!(clean_url("http://example.com/a").is_ok());
        for bad in ["", "ola", "file:///C:/a.mp4", "ftp://example.com/a", "https://exa mple.com", "https://example.com/\nmais", "--exec calc", "javascript:alert(1)"] {
            assert!(clean_url(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn le_a_informacion_dun_video() {
        let one = r#"{"title":"Un vídeo","duration":19.5,"uploader":"Alguén","extractor_key":"Youtube","upload_date":"20050423","view_count":1234,
          "formats":[
            {"vcodec":"avc1.4d401e","height":360,"fps":30,"tbr":400,"filesize":1000000},
            {"vcodec":"vp9","height":720,"fps":30,"tbr":900,"filesize":5000000},
            {"vcodec":"avc1.64001f","height":720,"fps":30,"tbr":800,"filesize":4000000},
            {"vcodec":"none","acodec":"mp4a.40.2","abr":129.5,"filesize":300000},
            {"vcodec":"none","acodec":"opus","abr":64,"filesize":100000}]}"#;
        let i = parse_info(one).unwrap();
        assert_eq!(i.title, "Un vídeo");
        assert_eq!(i.duration, Some(19.5));
        assert_eq!(i.platform.as_deref(), Some("Youtube"));
        assert_eq!(i.upload_date.as_deref(), Some("20050423"));
        assert_eq!(i.views, Some(1234));
        assert_eq!(i.heights, vec![720, 360]);
        // A 720p prefírese H.264 aínda que haxa un VP9 con máis bitrate; o tamaño suma o do audio.
        assert_eq!(i.video[0].codec, "H.264");
        assert_eq!(i.video[0].size, Some(4_300_000));
        assert_eq!(i.audio_kbps, Some(130));
        assert_eq!(i.audio_codec.as_deref(), Some("AAC"));
        assert!(!i.playlist && i.entries.is_empty() && i.total == 1);
        assert!(parse_info("non é json").is_err());
    }

    #[test]
    fn le_unha_lista_enteira() {
        let list = r#"{"_type":"playlist","title":"Lista","playlist_count":3,"entries":[
            {"url":"https://e.com/1","title":"Primeiro","duration":3.0,"uploader":"A"},
            {"url":"non é ligazón","title":"Raro"},
            {"webpage_url":"https://e.com/2","title":"Segundo","channel":"B"}]}"#;
        let l = parse_info(list).unwrap();
        assert!(l.playlist);
        assert_eq!(l.title, "Lista");
        assert_eq!(l.entries.len(), 2, "só as que son ligazóns");
        assert_eq!(l.entries[0], Entry { url: "https://e.com/1".into(), title: "Primeiro".into(), duration: Some(3.0), uploader: Some("A".into()) });
        assert_eq!(l.entries[1].uploader.as_deref(), Some("B"));
        assert_eq!(l.total, 3, "a lista di cantos vídeos ten aínda que se amosen menos");
        assert!(l.video.is_empty());
    }

    #[test]
    fn extrae_a_frase_do_erro() {
        assert_eq!(error_detail("WARNING: x\nERROR: [youtube] abc: Video unavailable\n"), "Video unavailable");
        assert_eq!(error_detail("ERROR: Unsupported URL: https://a.b"), "Unsupported URL: https://a.b");
        assert_eq!(error_detail(""), "sen detalle");
    }

    #[test]
    fn os_motivos_coñecidos_saen_en_galego() {
        let why = |t: &str| match failure(t) {
            EngineError::Unavailable(w) => w,
            _ => "outro",
        };
        assert_eq!(why("ERROR: [generic] The requested site is known to use DRM protection. It will NOT be supported."), "drm");
        assert_eq!(why("ERROR: [youtube] x: Private video. Sign in if you've been granted access"), "private");
        assert_eq!(why("ERROR: [youtube] x: Sign in to confirm you're not a bot"), "login");
        assert_eq!(why("ERROR: [youtube] x: Sign in to confirm your age. This video may be inappropriate for some users."), "age");
        assert_eq!(why("ERROR: Unsupported URL: https://a.b"), "unsupported");
        assert_eq!(why("ERROR: Unable to download webpage: HTTP Error 404: Not Found"), "gone");
        assert_eq!(why("ERROR: Unable to download webpage: <urlopen error [Errno 11001] getaddrinfo failed>"), "network");
        assert_eq!(why("ERROR: algo raro que ninguén prevé"), "outro");
        // O texto que ve o usuario é a frase en galego, non a de yt-dlp.
        assert!(failure("DRM protection").wire().contains("DRM"));
        assert!(!failure("DRM protection").wire().contains("NOT be supported"));
    }

    #[test]
    fn le_a_porcentaxe_e_valida_as_opcions() {
        assert_eq!(parse_percent("TRASNO-PROGRESO  45.5%"), Some(0.455));
        assert_eq!(parse_percent("[download] 10%"), None);
        let v = |h, c: &str| DownloadOptions { kind: "video".into(), max_height: h, container: c.into(), audio_format: "mp3".into(), audio_kbps: None };
        assert!(kind_args(&v(Some(720), "mp4")).unwrap().iter().any(|a| a == "bv*[height<=720]+ba/b[height<=720]"));
        assert!(kind_args(&v(None, "mp4")).unwrap().iter().any(|a| a == "bv*+ba/b"));
        assert!(kind_args(&v(None, "mkv")).unwrap().iter().any(|a| a == "mkv"));
        assert!(kind_args(&v(Some(5), "mp4")).is_err());
        assert!(kind_args(&v(None, "avi")).is_err());
        let a = |f: &str, k: Option<u32>| DownloadOptions { kind: "audio".into(), max_height: None, container: "mp4".into(), audio_format: f.into(), audio_kbps: k };
        assert!(kind_args(&a("mp3", None)).is_ok());
        assert!(kind_args(&a("mp3", Some(192))).unwrap().iter().any(|x| x == "192K"));
        assert!(kind_args(&a("flac", Some(192))).unwrap().iter().any(|x| x == "0"), "o formato sen perdas ignora o bitrate");
        assert!(kind_args(&a("mp3", Some(9999))).is_err());
        assert!(kind_args(&a("--exec", None)).is_err());
        assert!(kind_args(&DownloadOptions { kind: "outra".into(), max_height: None, container: "mp4".into(), audio_format: "mp3".into(), audio_kbps: None }).is_err());
    }
}
