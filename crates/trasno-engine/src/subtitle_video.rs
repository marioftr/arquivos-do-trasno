//! Subtítulos dentro do vídeo: incrustalos como pista (soft), queimalos na imaxe ou sacar unha pista a ficheiro.
//! Os subtítulos léense co noso lector (calquera codificación) e pásanse a UTF-8 antes de dárllelos a FFmpeg.

use crate::error::{EngineError, Result};
use crate::ffmpeg;
use crate::subtitles::{self, Format};
use serde::Serialize;
use std::path::{Path, PathBuf};

fn bad<T>(m: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(m.into()))
}

/// Unha pista de subtítulos dun vídeo.
#[derive(Debug, Clone, Serialize)]
pub struct SubtitleTrack {
    /// Posición entre as pistas de subtítulos (0 é a primeira): vale para `extract_track`.
    pub position: usize,
    pub codec: String,
    pub language: Option<String>,
    pub title: Option<String>,
    /// As pistas de imaxe (PGS, DVD) non se poden pasar a texto.
    pub text_based: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubtitleVideoReport {
    pub output: PathBuf,
    /// Falas dos subtítulos que entraron (ou que saíron, ao extraer).
    pub cues: usize,
}

const IMAGE_CODECS: [&str; 4] = ["hdmv_pgs_subtitle", "dvd_subtitle", "dvb_subtitle", "xsub"];

/// Pistas de subtítulos dun ficheiro de vídeo.
pub fn list_tracks(input: &Path) -> Result<Vec<SubtitleTrack>> {
    let info = ffmpeg::probe(input)?;
    let streams = info["streams"].as_array().cloned().unwrap_or_default();
    Ok(streams
        .iter()
        .filter(|s| s["codec_type"] == "subtitle")
        .enumerate()
        .map(|(position, s)| {
            let codec = s["codec_name"].as_str().unwrap_or("").to_string();
            SubtitleTrack {
                position,
                text_based: !IMAGE_CODECS.contains(&codec.as_str()),
                codec,
                language: s["tags"]["language"].as_str().filter(|l| *l != "und").map(String::from),
                title: s["tags"]["title"].as_str().map(String::from),
            }
        })
        .collect())
}

/// Código de idioma de tres letras (glg, spa, eng…) ou baleiro.
fn valid_lang(l: &str) -> bool {
    l.is_empty() || (l.len() == 3 && l.chars().all(|c| c.is_ascii_lowercase()))
}

/// Cartafol temporal propio (co ficheiro de subtítulos en UTF-8) que se borra ao rematar.
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self> {
        let d = std::env::temp_dir().join(format!("trasno-subs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&d)?;
        Ok(Scratch(d))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write_utf8(sub: &subtitles::Subtitle, to: Format, dir: &Path, stem: &str) -> Result<PathBuf> {
    let ext = match to {
        Format::Srt => "srt",
        Format::Vtt => "vtt",
        Format::Ass => "ass",
    };
    let p = dir.join(format!("{stem}.{ext}"));
    std::fs::write(&p, subtitles::render(sub, to))?;
    Ok(p)
}

fn out_ext(output: &Path) -> String {
    crate::formats::extension_of(output).unwrap_or_default()
}

/// Mete uns subtítulos no vídeo como unha pista máis (pódese acender e apagar no reprodutor). A imaxe e o son cópianse sen recodificar.
pub fn embed(video: &Path, subs: &Path, output: &Path, overwrite: bool, language: &str) -> Result<SubtitleVideoReport> {
    if !valid_lang(language) {
        return bad("O código de idioma non é válido (por exemplo: glg, spa, eng).");
    }
    let sub = subtitles::read_subtitle(subs)?;
    let ext = out_ext(output);
    let (to, codec) = match ext.as_str() {
        "mp4" | "m4v" | "mov" => (Format::Srt, "mov_text"),
        "webm" => (Format::Vtt, "webvtt"),
        "mkv" => {
            if sub.format == Format::Ass {
                (Format::Ass, "ass")
            } else {
                (Format::Srt, "subrip")
            }
        }
        _ => return bad("Este formato non admite subtítulos como pista. Usa MP4, MKV ou WebM."),
    };
    let info = ffmpeg::probe(video)?;
    let streams = info["streams"].as_array().cloned().unwrap_or_default();
    if !streams.iter().any(|s| s["codec_type"] == "video") {
        return bad("Este ficheiro non ten imaxe: os subtítulos van nun vídeo.");
    }
    let existing = streams.iter().filter(|s| s["codec_type"] == "subtitle").count();
    let scratch = Scratch::new()?;
    let file = write_utf8(&sub, to, &scratch.0, "subtitulos")?;

    let mut args: Vec<String> = ["-map", "0:v", "-map", "0:a?", "-map", "0:s?", "-map", "1:0", "-c", "copy"].iter().map(|s| s.to_string()).collect();
    // O códec novo só vale para a pista que se engade: as que o vídeo xa tiña (tamén as de imaxe, PGS) pasan tal cal.
    args.extend([format!("-c:s:{existing}"), codec.to_string()]);
    if !language.is_empty() {
        args.extend([format!("-metadata:s:s:{existing}"), format!("language={language}")]);
    }
    if existing == 0 {
        args.extend([format!("-disposition:s:{existing}"), "default".into()]);
    }
    // Algúns contedores (MPEG, AVI) non traen marcas de tempo en todos os paquetes: FFmpeg ten que as inventar para poder copialos.
    let pre: Vec<String> = vec!["-fflags".into(), "+genpts".into()];
    let out = ffmpeg::convert_media_inputs(&[video, &file], output, overwrite, &pre, &args)?;
    Ok(SubtitleVideoReport { output: out, cues: sub.cues.len() })
}

/// Escapa unha ruta para o filtro `subtitles` de FFmpeg (barras normais e «:» escapado).
fn filter_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/").replace(':', "\\:").replace('\'', "\\\\\\'")
}

/// Queima os subtítulos na imaxe: saen sempre, en calquera reprodutor, pero xa non se poden apagar. Recodifica a imaxe (calidade alta).
pub fn burn(video: &Path, subs: &Path, output: &Path, overwrite: bool) -> Result<SubtitleVideoReport> {
    let sub = subtitles::read_subtitle(subs)?;
    if crate::silence::video_encoder_args(output).is_empty() {
        return bad("Para queimar os subtítulos usa un vídeo en MP4, MKV, MOV ou WebM.");
    }
    let info = ffmpeg::probe(video)?;
    let streams = info["streams"].as_array().cloned().unwrap_or_default();
    if !streams.iter().any(|s| s["codec_type"] == "video" && s["disposition"]["attached_pic"] != 1) {
        return bad("Este ficheiro non ten imaxe: os subtítulos van nun vídeo.");
    }
    let has_audio = streams.iter().any(|s| s["codec_type"] == "audio");
    let scratch = Scratch::new()?;
    let to = if sub.format == Format::Ass { Format::Ass } else { Format::Srt };
    let file = write_utf8(&sub, to, &scratch.0, "subtitulos")?;

    let mut args: Vec<String> = vec!["-map".into(), "0:v:0".into()];
    if has_audio {
        args.extend(["-map".into(), "0:a?".into()]);
    }
    args.extend(["-vf".into(), format!("subtitles='{}'", filter_path(&file))]);
    args.extend(crate::silence::video_encoder_args(output));
    if has_audio {
        args.extend(ffmpeg::keep_audio_args(video, output));
    }
    match ffmpeg::convert_media(video, output, overwrite, &args) {
        Ok(out) => Ok(SubtitleVideoReport { output: out, cues: sub.cues.len() }),
        Err(EngineError::FfmpegFailed { stderr, .. }) if stderr.contains("No such filter") => {
            bad("Este FFmpeg non sabe queimar subtítulos (falta a biblioteca libass).")
        }
        Err(e) => Err(e),
    }
}

/// Saca unha pista de subtítulos a un ficheiro SRT, WebVTT ou ASS (segundo a extensión de `output`).
pub fn extract_track(video: &Path, position: usize, output: &Path, overwrite: bool) -> Result<SubtitleVideoReport> {
    let tracks = list_tracks(video)?;
    let track = tracks.get(position).ok_or_else(|| EngineError::IntegrityCheck("Esa pista de subtítulos non existe.".into()))?;
    if !track.text_based {
        return bad("Esta pista de subtítulos son imaxes (PGS ou DVD): non as podo pasar a texto.");
    }
    let codec = match out_ext(output).as_str() {
        "srt" => "subrip",
        "vtt" => "webvtt",
        "ass" | "ssa" => "ass",
        _ => return bad("Os subtítulos sacan a SRT, WebVTT ou ASS."),
    };
    let args: Vec<String> = ["-map".to_string(), format!("0:s:{position}"), "-c:s".into(), codec.into()].to_vec();
    let out = ffmpeg::convert_media(video, output, overwrite, &args)?;
    match subtitles::read_subtitle(&out) {
        Ok(s) => Ok(SubtitleVideoReport { output: out, cues: s.cues.len() }),
        Err(_) => {
            let _ = std::fs::remove_file(&out);
            bad("Esta pista non ten ningunha fala.")
        }
    }
}
