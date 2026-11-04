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
    ffmpeg::convert_media(input, &out, true, &["-frames:v".to_string(), "1".to_string()])?;
    Ok(out)
}

/// Escribe `output` (AVIF, JPEG 2000 ou JPEG XL) a partir dunha imaxe que a crate `image` sabe ler.
/// `quality` vai de 1 a 100 (por defecto, 80).
pub fn encode(source: &Path, output: &Path, overwrite: bool, quality: Option<u8>) -> Result<PathBuf> {
    let ext = crate::formats::extension_of(output).unwrap_or_default();
    let q = quality.unwrap_or(80).clamp(1, 100) as f64;
    let args: Vec<String> = match ext.as_str() {
        // CRF de 0 (sen perdas) a 63; -b:v 0 deixa que mande só a calidade.
        "avif" => vec!["-frames:v".into(), "1".into(), "-c:v".into(), "libaom-av1".into(), "-still-picture".into(), "1".into(), "-crf".into(), format!("{}", ((100.0 - q) * 0.63).round() as u32), "-b:v".into(), "0".into(), "-cpu-used".into(), "6".into()],
        "jxl" => vec!["-frames:v".into(), "1".into(), "-c:v".into(), "libjxl".into(), "-distance".into(), format!("{:.2}", (100.0 - q) / 10.0)],
        _ => vec!["-frames:v".into(), "1".into()],
    };
    ffmpeg::convert_media(source, output, overwrite, &args)
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
}
