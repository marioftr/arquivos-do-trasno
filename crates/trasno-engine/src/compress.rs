//! Compresión de imaxes e vídeos, ben por calidade ou ben forzando un tamaño
//! máximo ("que pese menos de X MB"). Escríbese sempre nun ficheiro novo, de
//! forma atómica, e nunca se toca o orixinal.

use crate::error::{EngineError, Result};
use crate::ffmpeg::{self, check_success, run_ffmpeg};
use crate::integrity::{assert_non_empty_file, ensure_distinct, AtomicOutput};
use image::DynamicImage;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------- imaxes

#[derive(Debug, Clone, Default)]
pub struct ImageCompress {
    /// Calidade JPEG fixa (1-100). Ignórase se hai `target_bytes`.
    pub quality: Option<u8>,
    /// Tamaño máximo do resultado en bytes: busca a mellor calidade que o cumpra
    /// e, se non chega, reduce tamén as dimensións.
    pub target_bytes: Option<u64>,
    /// Lado máximo en píxeles (só reduce, nunca amplía).
    pub max_side: Option<u32>,
    /// Fondo para a transparencia se o resultado é JPG (por defecto, branco).
    pub background: Option<[u8; 3]>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImageCompressReport {
    pub output_path: PathBuf,
    pub bytes_written: u64,
    pub quality: u8,
    /// Porcentaxe do tamaño orixinal en cada lado (100 = sen redimensionar).
    pub scale_percent: u32,
}

/// Formatos de imaxe que se comprimen mantendo o formato. Os demais (BMP, TIFF, GIF...) saen en JPG.
pub const COMPRESSIBLE: &[&str] = &["jpg", "jpeg", "png", "webp", "avif", "jxl", "jp2"];

/// Extensión do resultado ao comprimir un ficheiro con esta extensión: a mesma se se pode, se non `jpg`.
pub fn output_ext_for(ext: &str) -> String {
    let e = ext.to_ascii_lowercase();
    if COMPRESSIBLE.contains(&e.as_str()) {
        e
    } else {
        "jpg".to_string()
    }
}

/// JPEG non ten transparencia: compónse sobre fondo branco (en vez de negro).
fn flatten(img: &DynamicImage, bg: Option<[u8; 3]>) -> image::RgbImage {
    crate::image_convert::flatten_on(img, bg.unwrap_or(crate::image_convert::WHITE))
}

fn jpeg_bytes(img: &DynamicImage, quality: u8, bg: Option<[u8; 3]>) -> Result<Vec<u8>> {
    let rgb = flatten(img, bg);
    let mut buf = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality.clamp(1, 100));
    enc.encode_image(&rgb)?;
    Ok(buf)
}

/// PNG sen perdas co mellor nivel de compresión e filtro adaptativo.
fn png_bytes(img: &DynamicImage) -> Result<Vec<u8>> {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    let mut buf = Vec::new();
    img.write_with_encoder(PngEncoder::new_with_quality(&mut buf, CompressionType::Best, FilterType::Adaptive))?;
    Ok(buf)
}

/// WebP, AVIF, JPEG XL e JPEG 2000 escríbense con FFmpeg a partir dun PNG temporal.
fn ffmpeg_bytes(img: &DynamicImage, ext: &str, quality: u8) -> Result<Vec<u8>> {
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("in.png");
    img.save(&src)?;
    let dst = dir.path().join(format!("out.{ext}"));
    crate::image_ffmpeg::encode(&src, &dst, true, Some(quality), false)?;
    Ok(std::fs::read(&dst)?)
}

/// Os bytes que daría a imaxe nese formato con esa calidade (a calidade non conta en PNG, que é sen perdas).
fn encode_bytes(img: &DynamicImage, ext: &str, quality: u8, bg: Option<[u8; 3]>) -> Result<Vec<u8>> {
    match ext {
        "jpg" | "jpeg" => jpeg_bytes(img, quality, bg),
        "png" => png_bytes(img),
        _ => ffmpeg_bytes(img, ext, quality),
    }
}

fn scaled(base: &DynamicImage, percent: u32) -> DynamicImage {
    if percent >= 100 {
        return base.clone();
    }
    let w = (base.width() * percent / 100).max(1);
    let h = (base.height() * percent / 100).max(1);
    base.resize_exact(w, h, image::imageops::FilterType::Lanczos3)
}

fn load_base(input: &Path, max_side: Option<u32>) -> Result<DynamicImage> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    let mut img = image::open(crate::image_ffmpeg::readable(input)?)?;
    if let Some(m) = max_side.filter(|m| *m > 0) {
        if img.width().max(img.height()) > m {
            img = img.resize(m, m, image::imageops::FilterType::Lanczos3);
        }
    }
    Ok(img)
}

/// Tamaño aproximado (bytes) do resultado en `ext` con esta calidade.
pub fn estimate_image_bytes(input: &Path, ext: &str, quality: u8, max_side: Option<u32>, bg: Option<[u8; 3]>) -> Result<u64> {
    let ext = output_ext_for(ext);
    let img = load_base(input, max_side)?;
    // A estimación non é un traballo do usuario: non lle afecta «Cancelar».
    let bytes = ffmpeg::background(|| encode_bytes(&img, &ext, quality, bg))?;
    Ok(bytes.len() as u64)
}

pub fn compress_image(
    input: &Path,
    output: &Path,
    overwrite: bool,
    o: &ImageCompress,
) -> Result<ImageCompressReport> {
    ensure_distinct(&[input], output)?;
    let ext = crate::formats::extension_of(output).unwrap_or_default();
    if !COMPRESSIBLE.contains(&ext.as_str()) {
        return Err(EngineError::UnsupportedConversion {
            from: "imaxe".into(),
            to: ext,
        });
    }
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    let base = load_base(input, o.max_side)?;
    // En PNG non hai calidade que buscar: só se reducen as dimensións se hai un tamaño obxectivo.
    let lossy = ext != "png";

    let (bytes, quality, scale) = match o.target_bytes {
        None => {
            let q = o.quality.unwrap_or(80).clamp(1, 100);
            (encode_bytes(&base, &ext, q, o.background)?, if lossy { q } else { 100 }, 100)
        }
        Some(target) => {
            let mut found = None;
            let mut percent = 100u32;
            for _ in 0..10 {
                let img = scaled(&base, percent);
                if lossy {
                    // Busca binaria da mellor calidade que cabe no tamaño obxectivo.
                    let (mut lo, mut hi) = (5u8, 95u8);
                    let mut best: Option<(Vec<u8>, u8)> = None;
                    while lo <= hi {
                        let mid = lo + (hi - lo) / 2;
                        let b = encode_bytes(&img, &ext, mid, o.background)?;
                        if b.len() as u64 <= target {
                            best = Some((b, mid));
                            lo = mid + 1;
                        } else {
                            if mid == 0 {
                                break;
                            }
                            hi = mid - 1;
                        }
                    }
                    if let Some((b, q)) = best {
                        found = Some((b, q, percent));
                        break;
                    }
                } else {
                    let b = png_bytes(&img)?;
                    if b.len() as u64 <= target {
                        found = Some((b, 100, percent));
                        break;
                    }
                }
                percent = percent * 85 / 100;
                if percent < 5 {
                    break;
                }
            }
            found.ok_or_else(|| {
                EngineError::IntegrityCheck("Non hai maneira de que a imaxe caiba en tan pouco espazo; proba cun tamaño maior.".into())
            })?
        }
    };

    let mut out = AtomicOutput::new(output, overwrite)?;
    out.write_all(&bytes)?;
    let path = out.commit()?;
    // Comprobación: ten que reabrirse (os formatos que só le FFmpeg xa os comprobou el ao escribilos).
    if matches!(ext.as_str(), "jpg" | "jpeg" | "png") && image::open(&path).is_err() {
        let _ = std::fs::remove_file(&path);
        return Err(EngineError::EmptyOrCorruptOutput);
    }
    Ok(ImageCompressReport {
        output_path: path,
        bytes_written: bytes.len() as u64,
        quality,
        scale_percent: scale,
    })
}

// ----------------------------------------------------------------- vídeo

#[derive(Debug, Clone, Default)]
pub struct VideoCompress {
    /// CRF de x264 (18 case sen perdas ... 40 moi comprimido). Ignórase se hai `target_bytes`.
    pub crf: u8,
    /// Tamaño máximo do resultado en bytes (dúas pasadas).
    pub target_bytes: Option<u64>,
    /// Altura máxima en píxeles (só reduce).
    pub max_height: Option<u32>,
    pub audio_kbps: u32,
    /// Bitrate fixo da imaxe (kbps), nunha soa pasada. Ignórase se hai `target_bytes`.
    pub video_kbps: Option<u32>,
    /// Desentrelazar: 0 non, 1 só os cadros entrelazados (mantén a cadencia), 2 todos (dobra a cadencia, como o DV orixinal en «yadif=1»).
    pub deinterlace: u8,
    /// Preset lento de x264 e perfil High 4.1: máis eficiente e compatible con case todo.
    pub compat: bool,
    /// Relación de aspecto de pantalla forzada (p. ex. «4/3» ou «16/9»), útil nos vídeos DV anamórficos.
    pub dar: Option<String>,
}

fn scale_filter(max_height: Option<u32>) -> Option<String> {
    max_height
        .filter(|h| *h >= 16)
        .map(|h| format!("scale=-2:trunc(min(ih\\,{h})/2)*2"))
}

struct Streams {
    duration: f64,
    has_audio: bool,
}

fn inspect(input: &Path) -> Result<Streams> {
    let info = ffmpeg::probe(input)?;
    let duration = info["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|d| *d > 0.0)
        .ok_or_else(|| EngineError::IntegrityCheck("Non souben ler a duración do vídeo.".into()))?;
    let has_audio = info["streams"]
        .as_array()
        .map(|s| s.iter().any(|x| x["codec_type"] == "audio"))
        .unwrap_or(false);
    Ok(Streams { duration, has_audio })
}

fn x264_common(crf: Option<u8>, vkbps: Option<u32>, vf: &Option<String>) -> Vec<String> {
    let mut a: Vec<String> = vec!["-c:v".into(), "libx264".into(), "-preset".into(), "medium".into(), "-pix_fmt".into(), "yuv420p".into()];
    if let Some(c) = crf {
        a.extend(["-crf".into(), c.clamp(16, 45).to_string()]);
    }
    if let Some(v) = vkbps {
        a.extend(["-b:v".into(), format!("{v}k")]);
    }
    if let Some(f) = vf {
        a.extend(["-vf".into(), f.clone()]);
    }
    a
}

fn null_sink() -> &'static str {
    if cfg!(windows) { "NUL" } else { "/dev/null" }
}

/// Tamaño aproximado (bytes) do resultado: codifica unha mostra (ata 10 s do
/// medio do vídeo) cos mesmos axustes e extrapola á duración total.
pub fn estimate_video_bytes(input: &Path, o: &VideoCompress) -> Result<u64> {
    let (crf, audio_kbps) = (o.crf, o.audio_kbps);
    let s = inspect(input)?;
    let len = s.duration.min(10.0);
    let start = ((s.duration - len) / 2.0).max(0.0);
    let tmp = std::env::temp_dir().join(format!("arquivos-do-trasno-estimacion-{}.mp4", uuid::Uuid::new_v4()));
    let mut args: Vec<std::ffi::OsString> = vec!["-ss".into(), start.to_string().into(), "-i".into(), input.as_os_str().to_owned(), "-t".into(), len.to_string().into()];
    args.extend(x264_common(Some(crf), None, &filters(o)).into_iter().map(Into::into));
    args.extend(compat_args(o).into_iter().map(Into::into));
    if s.has_audio {
        args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), format!("{audio_kbps}k").into()]);
    } else {
        args.push("-an".into());
    }
    args.push(tmp.as_os_str().to_owned());
    let out = run_ffmpeg(&args);
    let size = match out {
        Ok(o) => check_success(&o).and_then(|_| assert_non_empty_file(&tmp)),
        Err(e) => Err(e),
    };
    let _ = std::fs::remove_file(&tmp);
    let sample = size? as f64;
    Ok((sample / len * s.duration) as u64)
}

/// Filtros de vídeo do perfil: desentrelazar, escalar e fixar a relación de aspecto, por esa orde.
fn filters(o: &VideoCompress) -> Option<String> {
    let mut f: Vec<String> = Vec::new();
    match o.deinterlace {
        1 => f.push("yadif=0:-1:1".into()),
        2 => f.push("yadif=1".into()),
        _ => {}
    }
    if let Some(s) = scale_filter(o.max_height) {
        f.push(s);
    }
    if let Some(d) = o.dar.as_deref().filter(|d| d.chars().all(|c| c.is_ascii_digit() || c == '/' || c == ':')) {
        f.push(format!("setdar={}", d.replace(':', "/")));
    }
    if f.is_empty() { None } else { Some(f.join(",")) }
}

/// Axustes de x264 que se engaden ao final (o último valor de cada opción gaña).
fn compat_args(o: &VideoCompress) -> Vec<String> {
    if o.compat {
        ["-preset", "slow", "-profile:v", "high", "-level", "4.1"].iter().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}

pub fn compress_video(input: &Path, output: &Path, overwrite: bool, o: &VideoCompress) -> Result<PathBuf> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    ensure_distinct(&[input], output)?;
    let vf = filters(o);
    let audio_kbps = o.audio_kbps.clamp(32, 320);

    if o.target_bytes.is_none() {
        if let Some(vk) = o.video_kbps {
            if !(50..=100_000).contains(&vk) {
                return Err(EngineError::IntegrityCheck("O bitrate do vídeo non é válido.".into()));
            }
            let mut args = x264_common(None, Some(vk), &vf);
            args.extend(compat_args(o));
            args.extend(["-maxrate".into(), format!("{}k", vk * 3 / 2), "-bufsize".into(), format!("{}k", vk * 2)]);
            args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), format!("{audio_kbps}k"), "-movflags".into(), "+faststart".into()]);
            return ffmpeg::convert_media(input, output, overwrite, &args);
        }
    }
    let Some(target) = o.target_bytes else {
        let mut args = x264_common(Some(o.crf), None, &vf);
        args.extend(compat_args(o));
        args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), format!("{audio_kbps}k"), "-movflags".into(), "+faststart".into()]);
        return ffmpeg::convert_media(input, output, overwrite, &args);
    };

    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    let s = inspect(input)?;
    let dir = output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let ext = output.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
    let id = uuid::Uuid::new_v4();
    let temp = dir.join(format!(".trasno-{id}.part.{ext}"));
    let log = dir.join(format!(".trasno-{id}-pass"));
    let cleanup = || {
        let _ = std::fs::remove_file(&temp);
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                if e.file_name().to_string_lossy().starts_with(&format!(".trasno-{id}-pass")) {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
    };

    let a_kbps = if s.has_audio { audio_kbps as f64 } else { 0.0 };
    // 3 % de marxe para o contedor.
    let mut v_kbps = (target as f64 * 8.0 * 0.97 / s.duration) / 1000.0 - a_kbps;
    let mut result: Result<()> = Ok(());
    for attempt in 0..2 {
        if v_kbps < 30.0 {
            cleanup();
            return Err(EngineError::IntegrityCheck(
                "Ese tamaño é demasiado pequeno para a duración do vídeo; proba cun valor maior.".into(),
            ));
        }
        let vk = v_kbps as u32;
        let mut p1: Vec<std::ffi::OsString> = vec!["-i".into(), input.as_os_str().to_owned()];
        p1.extend(x264_common(None, Some(vk), &vf).into_iter().map(Into::into));
        p1.extend(compat_args(o).into_iter().map(Into::into));
        p1.extend(["-pass".into(), "1".into(), "-passlogfile".into(), log.as_os_str().to_owned(), "-an".into(), "-f".into(), "null".into(), null_sink().into()]);
        let mut p2: Vec<std::ffi::OsString> = vec!["-i".into(), input.as_os_str().to_owned()];
        p2.extend(x264_common(None, Some(vk), &vf).into_iter().map(Into::into));
        p2.extend(compat_args(o).into_iter().map(Into::into));
        p2.extend(["-pass".into(), "2".into(), "-passlogfile".into(), log.as_os_str().to_owned()]);
        if s.has_audio {
            p2.extend(["-c:a".into(), "aac".into(), "-b:a".into(), format!("{audio_kbps}k").into()]);
        } else {
            p2.push("-an".into());
        }
        if ext == "mp4" || ext == "mov" {
            p2.extend(["-movflags".into(), "+faststart".into()]);
        }
        p2.push(temp.as_os_str().to_owned());

        ffmpeg::set_progress_phase(0.0, 0.5);
        result = run_ffmpeg(&p1).and_then(|o| check_success(&o)).and_then(|_| {
            ffmpeg::set_progress_phase(0.5, 0.5);
            run_ffmpeg(&p2)
        }).and_then(|o| check_success(&o));
        ffmpeg::set_progress_phase(0.0, 1.0);
        if result.is_err() {
            break;
        }
        let size = assert_non_empty_file(&temp).unwrap_or(0);
        if size <= target || attempt == 1 {
            break;
        }
        // Pasouse: segunda tentativa cun pouco menos de bitrate.
        v_kbps *= (target as f64 / size as f64) * 0.97;
    }
    if let Err(e) = result {
        cleanup();
        return Err(e);
    }
    let size = match assert_non_empty_file(&temp) {
        Ok(s) => s,
        Err(e) => {
            cleanup();
            return Err(e);
        }
    };
    if size > target {
        cleanup();
        return Err(EngineError::IntegrityCheck(
            "Non conseguín baixar ese tamaño; proba con menos resolución ou cun límite maior.".into(),
        ));
    }
    if let Ok(bin) = ffmpeg::ffprobe_path() {
        let mut cmd = std::process::Command::new(bin);
        let ok = cmd.args(["-v", "error", "-show_entries", "format=duration"]).arg(&temp).output().map(|p| p.status.success()).unwrap_or(true);
        if !ok {
            cleanup();
            return Err(EngineError::IntegrityCheck("O ficheiro comprimido non se pode ler; descarteino para non deixarche un ficheiro roto.".into()));
        }
    }
    let renamed = std::fs::rename(&temp, output);
    cleanup();
    renamed.map_err(EngineError::Io)?;
    Ok(output.to_path_buf())
}
