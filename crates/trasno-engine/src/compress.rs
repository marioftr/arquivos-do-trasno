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

/// Formatos de imaxe que se comprimen mantendo o formato. Os demais (BMP, TIFF, HEIC...) non teñen compresión propia
/// ou o trasno non os sabe escribir: a interface bloquea a compresión e di que hai que convertelos antes.
pub const COMPRESSIBLE: &[&str] = &["jpg", "jpeg", "png", "webp", "avif", "jxl", "jp2", "gif"];

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

/// Número de cores da paleta para esta calidade (10 → 8 cores, 95 → 256).
pub(crate) fn palette_colors(quality: u8) -> usize {
    let q = quality.clamp(10, 95) as f32;
    (8.0 + (q - 10.0) * (248.0 / 85.0)).round().clamp(8.0, 256.0) as usize
}

/// PNG con perdas: reduce a imaxe a unha paleta de cores (NeuQuant) con tramado Floyd–Steinberg e garda un PNG indexado
/// (conserva a transparencia). É o que fan ferramentas como pngquant ou TinyPNG; se non dá un ficheiro máis pequeno que o sen
/// perdas, devólvese este último.
fn png_quantized(img: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    let colors = palette_colors(quality);
    let pixels = w * h;
    let samplefac = match pixels {
        0..=1_000_000 => 1,
        1_000_001..=4_000_000 => 4,
        4_000_001..=16_000_000 => 12,
        _ => 25,
    };
    let nq = color_quant::NeuQuant::new(samplefac, colors, rgba.as_raw());
    let map = nq.color_map_rgba();
    let mut idx = vec![0u8; pixels];
    let mut cur = vec![[0f32; 3]; w + 2];
    let mut next = vec![[0f32; 3]; w + 2];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let p = &rgba.as_raw()[i * 4..i * 4 + 4];
            let e = cur[x + 1];
            let mut adj = [0f32; 3];
            for c in 0..3 {
                adj[c] = (p[c] as f32 + e[c]).clamp(0.0, 255.0);
            }
            let k = nq.index_of(&[adj[0] as u8, adj[1] as u8, adj[2] as u8, p[3]]);
            idx[i] = k as u8;
            if p[3] == 0 {
                continue;
            }
            let pal = &map[k * 4..k * 4 + 3];
            for c in 0..3 {
                let d = (adj[c] - pal[c] as f32) * 0.85;
                cur[x + 2][c] += d * 7.0 / 16.0;
                next[x][c] += d * 3.0 / 16.0;
                next[x + 1][c] += d * 5.0 / 16.0;
                next[x + 2][c] += d / 16.0;
            }
        }
        std::mem::swap(&mut cur, &mut next);
        next.iter_mut().for_each(|e| *e = [0.0; 3]);
    }
    let n = map.len() / 4;
    let mut palette = Vec::with_capacity(n * 3);
    let mut alpha = Vec::with_capacity(n);
    for c in map.chunks_exact(4) {
        palette.extend_from_slice(&c[..3]);
        alpha.push(c[3]);
    }
    let mut buf = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut buf, w as u32, h as u32);
        enc.set_color(png::ColorType::Indexed);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_palette(palette);
        if alpha.iter().any(|a| *a != 255) {
            enc.set_trns(alpha);
        }
        enc.set_compression(png::Compression::High);
        let mut wr = enc.write_header().map_err(|e| std::io::Error::other(e.to_string()))?;
        wr.write_image_data(&idx).map_err(|e| std::io::Error::other(e.to_string()))?;
    }
    let lossless = png_bytes(img)?;
    Ok(if buf.len() < lossless.len() { buf } else { lossless })
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

/// Os bytes que daría a imaxe nese formato con esa calidade (en PNG, 100 é sen perdas e menos reduce a paleta de cores).
fn encode_bytes(img: &DynamicImage, ext: &str, quality: u8, bg: Option<[u8; 3]>) -> Result<Vec<u8>> {
    match ext {
        "jpg" | "jpeg" => jpeg_bytes(img, quality, bg),
        "png" if quality >= 100 => png_bytes(img),
        "png" => png_quantized(img, quality),
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
    if ext == "gif" {
        // Un GIF (sobre todo se é animado) non se pode estimar cunha soa imaxe: faise de verdade nun temporal.
        let tmp = tempfile::Builder::new().suffix(".gif").tempfile()?.into_temp_path();
        let path = tmp.to_path_buf();
        ffmpeg::background(|| crate::animated::compress_gif(input, &path, true, quality, max_side))?;
        return Ok(std::fs::metadata(&path)?.len());
    }
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
    if ext == "gif" {
        return compress_gif_file(input, output, overwrite, o);
    }
    if crate::animated::is_animated(input) {
        return Err(EngineError::IntegrityCheck("Esta imaxe é animada e comprimila só gardaría un fotograma: o trasno só sabe comprimir sen perder a animación os GIF.".into()));
    }
    let base = load_base(input, o.max_side)?;
    // En PNG, calidade 100 (a de por defecto) é sen perdas; por baixo reduce a paleta de cores (como pngquant).
    let lossy = true;

    let (bytes, quality, scale) = match o.target_bytes {
        None => {
            let q = o.quality.unwrap_or(if ext == "png" { 100 } else { 80 }).clamp(1, 100);
            (encode_bytes(&base, &ext, q, o.background)?, q, 100)
        }
        Some(target) => {
            let mut found = None;
            let mut percent = 100u32;
            for _ in 0..10 {
                let img = scaled(&base, percent);
                if ext == "png" {
                    // Primeiro sen perdas: se xa cabe, non se toca nin unha cor.
                    let b = png_bytes(&img)?;
                    if b.len() as u64 <= target {
                        found = Some((b, 100, percent));
                        break;
                    }
                }
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

/// GIF (animado ou non): ffmpeg reduce a paleta de cores e, se hai que cumprir un tamaño, tamén as dimensións, sen tocar os fotogramas.
fn compress_gif_file(input: &Path, output: &Path, overwrite: bool, o: &ImageCompress) -> Result<ImageCompressReport> {
    let size_of = |p: &Path| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    let done = |path: PathBuf, colors: u32, percent: u32| ImageCompressReport {
        bytes_written: size_of(&path),
        output_path: path,
        quality: (colors * 100 / 256).clamp(1, 100) as u8,
        scale_percent: percent,
    };
    let side = |percent: u32| -> Option<u32> {
        let (w, h) = crate::image_convert::image_dimensions(input).ok()?;
        let base = w.max(h);
        let base = o.max_side.map(|m| m.min(base)).unwrap_or(base);
        (percent < 100 || o.max_side.is_some()).then(|| (base * percent / 100).max(1))
    };
    let Some(target) = o.target_bytes else {
        let q = o.quality.unwrap_or(100).clamp(1, 100);
        let out = crate::animated::compress_gif(input, output, overwrite, q, o.max_side)?;
        return Ok(done(out, crate::compress::palette_colors(q) as u32, 100));
    };
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    // Primeiro a paleta completa; despois menos cores; se aínda non chega, redúcese o tamaño.
    let mut percent = 100u32;
    for _ in 0..10 {
        for colors in [256u32, 128, 64, 32] {
            let tmp = tempfile::Builder::new().suffix(".gif").tempfile()?.into_temp_path();
            let path = tmp.to_path_buf();
            crate::animated::gif_with_colors(input, &path, true, colors, side(percent))?;
            if size_of(&path) <= target {
                let out = crate::animated::gif_with_colors(input, output, overwrite, colors, side(percent))?;
                return Ok(done(out, colors, percent));
            }
        }
        percent = percent * 80 / 100;
        if percent < 5 {
            break;
        }
    }
    Err(EngineError::IntegrityCheck("Non hai maneira de que a imaxe caiba en tan pouco espazo; proba cun tamaño maior.".into()))
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
