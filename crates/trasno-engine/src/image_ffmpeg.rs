//! Formatos de imaxe que a crate `image` non le nin escribe (AVIF, JPEG 2000, JPEG XL, HEIC...).
//! Para eles o motor axúdase de FFmpeg: lense pasándoos a un PNG temporal e escríbense a partir dun PNG.
//! Se FFmpeg non está instalado ou a súa compilación non leva o códec, o erro di o que falta.

use crate::error::Result;
use crate::ffmpeg;
use std::path::{Path, PathBuf};

/// Extensións que se len con FFmpeg (a crate `image` non as entende).
pub const DECODE_EXTS: &[&str] = &["avif", "jp2", "jpx", "j2k", "jxl", "heic", "heif", "dds", "exr", "pcx", "hdr"];

/// Extensións que se poden escribir con FFmpeg.
pub const ENCODE_EXTS: &[&str] = &["avif", "jp2", "jxl"];

pub fn is_decode_ext(ext: &str) -> bool {
    DECODE_EXTS.contains(&ext.to_ascii_lowercase().as_str())
}

pub fn is_encode_ext(ext: &str) -> bool {
    ENCODE_EXTS.contains(&ext.to_ascii_lowercase().as_str())
}

pub fn cache_dir() -> PathBuf {
    std::env::temp_dir().join("arquivos-do-trasno-imaxes")
}

/// Devolve unha ruta que a crate `image` sabe ler: a propia se xa é lexible, ou un PNG temporal (feito con FFmpeg e gardado
/// mentres o ficheiro non cambie) se é un formato que só le FFmpeg.
pub fn readable(input: &Path) -> Result<PathBuf> {
    let ext = crate::formats::extension_of(input).unwrap_or_default();
    if !is_decode_ext(&ext) {
        return Ok(input.to_path_buf());
    }
    let meta = std::fs::metadata(input)?;
    let stamp = meta.modified().ok().and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
    let key = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        input.hash(&mut h);
        meta.len().hash(&mut h);
        stamp.hash(&mut h);
        format!("{:x}", h.finish())
    };
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)?;
    let out = dir.join(format!("{key}.png"));
    if out.is_file() {
        return Ok(out);
    }
    let attempts = decode_attempts(input);
    let mut last = None;
    for args in attempts {
        match ffmpeg::background(|| ffmpeg::convert_media(input, &out, true, &args)) {
            Ok(_) => return Ok(out),
            Err(e) => last = Some(e),
        }
    }
    Err(last.expect("polo menos hai un intento"))
}

/// Formas de pasar a PNG o primeiro que se atope, da máis completa á máis simple:
/// HEIC en cuadrícula de teselas (iPhone), AVIF con transparencia e, por último, a imaxe sen máis.
fn decode_attempts(input: &Path) -> Vec<Vec<String>> {
    let plain: Vec<String> = ["-frames:v", "1", "-update", "1"].iter().map(|s| s.to_string()).collect();
    let mut v = Vec::new();
    if let Ok(info) = ffmpeg::probe_with_groups(input) {
        if let Some(fc) = grid_filter(&info) {
            v.push(vec!["-filter_complex".into(), fc, "-map".into(), "[o]".into(), "-frames:v".into(), "1".into(), "-update".into(), "1".into()]);
        } else if has_alpha_stream(&info) {
            v.push(vec![
                "-filter_complex".into(),
                "[0:v:0][0:v:1]alphamerge[o]".into(),
                "-map".into(),
                "[o]".into(),
                "-frames:v".into(),
                "1".into(),
                "-update".into(),
                "1".into(),
            ]);
        }
    }
    v.push(plain);
    v
}

/// AVIF con transparencia: un segundo fluxo de vídeo, só con luminancia, leva o canal alfa.
fn has_alpha_stream(info: &serde_json::Value) -> bool {
    let streams = match info["streams"].as_array() {
        Some(s) => s,
        None => return false,
    };
    let video: Vec<&serde_json::Value> = streams.iter().filter(|s| s["codec_type"] == "video").collect();
    video.len() == 2 && video[1]["pix_fmt"].as_str().map(|p| p.starts_with("gray")).unwrap_or(false)
}

/// Filtro que une as teselas dunha cuadrícula HEIF/AVIF nunha soa imaxe, recorta o sobrante e xira segundo a orientación gardada.
fn grid_filter(info: &serde_json::Value) -> Option<String> {
    let comp = info["stream_groups"].as_array()?.iter().find(|g| g["type"] == "Tile Grid")?["components"].as_array()?.first()?.clone();
    let subs = comp["subcomponents"].as_array()?;
    if subs.is_empty() {
        return None;
    }
    let mut inputs = String::new();
    let mut layout = Vec::new();
    for s in subs {
        inputs.push_str(&format!("[0:{}]", s["stream_index"].as_i64()?));
        layout.push(format!("{}_{}", s["tile_horizontal_offset"].as_i64()?, s["tile_vertical_offset"].as_i64()?));
    }
    let mut f = format!(
        "{inputs}xstack=inputs={}:layout={},crop={}:{}:{}:{}",
        subs.len(),
        layout.join("|"),
        comp["width"].as_i64()?,
        comp["height"].as_i64()?,
        comp["horizontal_offset"].as_i64().unwrap_or(0),
        comp["vertical_offset"].as_i64().unwrap_or(0)
    );
    // A rotación vai na matriz de visualización (graos en sentido antihorario). Se as teselas non a levan,
    // vale a do primeiro fluxo que a teña (a miniatura leva a mesma que a imaxe principal).
    let rotation = info["streams"]
        .as_array()
        .and_then(|st| st.iter().find_map(|s| s["side_data_list"].as_array()?.iter().find_map(|d| d["rotation"].as_f64())))
        .unwrap_or(0.0);
    match (((-rotation).round() as i64) % 360 + 360) % 360 {
        90 => f.push_str(",transpose=1"),
        180 => f.push_str(",hflip,vflip"),
        270 => f.push_str(",transpose=2"),
        _ => {}
    }
    f.push_str("[o]");
    Some(f)
}

/// Escribe `output` (AVIF, JPEG 2000 ou JPEG XL) a partir dunha imaxe que a crate `image` sabe ler.
/// `quality` vai de 1 a 100 (por defecto, 80).
pub fn encode(source: &Path, output: &Path, overwrite: bool, quality: Option<u8>, lossless: bool) -> Result<PathBuf> {
    let ext = crate::formats::extension_of(output).unwrap_or_default();
    let q = quality.unwrap_or(80).clamp(1, 100) as f64;
    let s = |v: &str| v.to_string();
    let args: Vec<String> = match ext.as_str() {
        "avif" => avif_args(source, q, lossless),
        "jxl" => vec![s("-frames:v"), s("1"), s("-c:v"), s("libjxl"), s("-distance"), if lossless { s("0") } else { format!("{:.2}", (100.0 - q) / 10.0) }],
        // «-pred 1»: transformada reversible 5/3, sen perdas.
        "jp2" if lossless => vec![s("-frames:v"), s("1"), s("-c:v"), s("jpeg2000"), s("-pred"), s("1")],
        _ => vec![s("-frames:v"), s("1")],
    };
    ffmpeg::convert_media(source, output, overwrite, &args)
}

/// Opcións de AVIF. Con perdas: YUV 4:2:0 con matriz BT.709 e rango completo (o habitual); sen perdas: RGB tal cal
/// (matriz de identidade) e `lossless=1`, que é o que fai o códec exacto. Se a imaxe ten transparencia, o canal alfa
/// vai nun segundo fluxo en gris.
fn avif_args(source: &Path, q: f64, lossless: bool) -> Vec<String> {
    let s = |v: &str| v.to_string();
    let alpha = image_has_alpha(source);
    let crf = if lossless { 0 } else { ((100.0 - q) * 0.63).round() as u32 };
    let mut a: Vec<String> = Vec::new();
    let color = if lossless { "format=gbrp" } else { "scale=out_range=full:out_color_matrix=bt709,format=yuv420p" };
    if alpha {
        a.extend([
            s("-filter_complex"),
            format!("[0:v]format=rgba,split[c][a];[a]alphaextract,format=gray,setparams=colorspace=bt709:range=pc[al];[c]{color}[cc]"),
            s("-map"),
            s("[cc]"),
            s("-map"),
            s("[al]"),
        ]);
    } else if !lossless {
        a.extend([s("-vf"), s(color)]);
    }
    a.extend([s("-frames:v"), s("1"), s("-c:v"), s("libaom-av1"), s("-still-picture"), s("1"), s("-crf"), crf.to_string(), s("-b:v"), s("0"), s("-cpu-used"), s("6")]);
    if lossless {
        a.extend([s("-aom-params"), s("lossless=1")]);
    } else {
        a.extend([s("-colorspace"), s("bt709"), s("-color_range"), s("pc")]);
    }
    a
}

fn image_has_alpha(path: &Path) -> bool {
    use image::ImageDecoder;
    image::ImageReader::open(path)
        .ok()
        .and_then(|r| r.with_guessed_format().ok())
        .and_then(|r| r.into_decoder().ok())
        .map(|d| d.color_type().has_alpha())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image_convert::{convert_image, image_dimensions, ImageConvertOptions};

    /// Se hai FFmpeg: PNG -> AVIF/JP2/JXL -> PNG, e as medidas se conservan.
    #[test]
    fn roundtrip_with_ffmpeg() {
        if !ffmpeg::is_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("a.png");
        image::RgbImage::from_fn(64, 48, |x, y| image::Rgb([(x * 4) as u8, (y * 5) as u8, 90])).save(&png).unwrap();
        for ext in ["avif", "jp2", "jxl"] {
            let out = dir.path().join(format!("b.{ext}"));
            match convert_image(&png, &out, true, &ImageConvertOptions::default()) {
                Ok(_) => {
                    assert_eq!(image_dimensions(&out).unwrap(), (64, 48), "{ext}");
                    let back = dir.path().join(format!("c-{ext}.png"));
                    convert_image(&out, &back, true, &ImageConvertOptions::default()).unwrap();
                    assert_eq!(image::image_dimensions(&back).unwrap(), (64, 48), "{ext}");
                }
                // A compilación de FFmpeg pode non levar este códec: non é un fallo do motor.
                Err(e) => eprintln!("{ext}: {e}"),
            }
        }
    }

    /// JPEG XL e JPEG 2000 sen perdas: o PNG de volta ten exactamente os mesmos píxeles.
    #[test]
    fn lossless_keeps_every_pixel() {
        if !ffmpeg::is_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("a.png");
        let src = image::RgbImage::from_fn(64, 48, |x, y| image::Rgb([((x * 37 + y * 11) % 256) as u8, ((x * 5 + y * 91) % 256) as u8, ((x * y) % 256) as u8]));
        src.save(&png).unwrap();
        let opts = ImageConvertOptions { lossless: true, ..Default::default() };
        for ext in ["jxl", "jp2", "avif"] {
            let out = dir.path().join(format!("b.{ext}"));
            if let Err(e) = convert_image(&png, &out, true, &opts) {
                eprintln!("{ext}: {e}"); // a compilación de FFmpeg pode non levar este códec
                continue;
            }
            let back = dir.path().join(format!("c-{ext}.png"));
            convert_image(&out, &back, true, &ImageConvertOptions::default()).unwrap();
            assert_eq!(image::open(&back).unwrap().to_rgb8(), src, "{ext}");
        }
    }

    /// AVIF coa transparencia: o PNG de volta segue sendo medio transparente.
    #[test]
    fn avif_keeps_transparency() {
        if !ffmpeg::is_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("a.png");
        image::RgbaImage::from_fn(64, 48, |x, _| image::Rgba([200, 30, 30, if x < 32 { 255 } else { 0 }])).save(&png).unwrap();
        let out = dir.path().join("b.avif");
        if let Err(e) = convert_image(&png, &out, true, &ImageConvertOptions::default()) {
            eprintln!("avif: {e}");
            return;
        }
        let back = dir.path().join("c.png");
        convert_image(&out, &back, true, &ImageConvertOptions::default()).unwrap();
        let img = image::open(&back).unwrap().to_rgba8();
        assert!(img.get_pixel(5, 5)[3] > 200, "esquerda opaca");
        assert!(img.get_pixel(60, 5)[3] < 50, "dereita transparente");
    }
}
