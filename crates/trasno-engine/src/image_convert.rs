//! Conversión de imaxes usando a crate `image` (decodificación e
//! codificación en Rust puro, sen procesos externos). É a parte do
//! conversor universal que funciona sempre, en calquera equipo, sen
//! instalar nada máis.

use crate::error::{EngineError, Result};
use crate::formats::{self, Category};
use crate::integrity::AtomicOutput;
use image::ImageFormat;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct ImageConvertOptions {
    /// Calidade JPEG/WebP con perdas, de 1 a 100. `None` usa o valor por defecto.
    pub quality: Option<u8>,
    /// Redimensionar antes de gardar ao ancho×alto exactos indicados (pode
    /// deformar a imaxe se a proporción non coincide coa orixinal; a UI
    /// ofrece "manter proporción" calculando o lado que falta primeiro).
    pub resize: Option<(u32, u32)>,
}

/// Le só as dimensións dunha imaxe (cabeceira, sen decodificar os píxeles),
/// para que a UI poida ofrecer "manter proporción" ao redimensionar.
pub fn image_dimensions(path: &Path) -> Result<(u32, u32)> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    image::image_dimensions(path).map_err(EngineError::Image)
}

fn format_from_ext(ext: &str) -> Result<ImageFormat> {
    ImageFormat::from_extension(ext).ok_or_else(|| EngineError::UnsupportedConversion {
        from: ext.to_string(),
        to: "?".to_string(),
    })
}

pub fn convert_image(
    input: &Path,
    output: &Path,
    overwrite: bool,
    options: &ImageConvertOptions,
) -> Result<PathBuf> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }

    let src_ext = formats::extension_of(input).ok_or_else(|| {
        EngineError::UnknownSourceFormat(input.to_path_buf())
    })?;
    if formats::category_of_extension(&src_ext) != Category::Imaxe {
        return Err(EngineError::UnsupportedConversion {
            from: src_ext,
            to: "imaxe".to_string(),
        });
    }

    let dst_ext = formats::extension_of(output).ok_or_else(|| {
        EngineError::UnsupportedConversion {
            from: src_ext.clone(),
            to: "(sen extensión)".to_string(),
        }
    })?;
    let dst_format = format_from_ext(&dst_ext)?;

    // Decodificar por completo antes de tocar o destino: se a imaxe de
    // orixe está corrompida, fallamos aquí e nunca chegamos a crear nada.
    let mut img = image::open(input)?;

    if let Some((w, h)) = options.resize {
        if w == 0 || h == 0 {
            return Err(EngineError::EmptyOrCorruptOutput);
        }
        img = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
    }

    if img.width() == 0 || img.height() == 0 {
        return Err(EngineError::EmptyOrCorruptOutput);
    }

    let mut out = AtomicOutput::new(output, overwrite)?;

    match (dst_format, options.quality) {
        (ImageFormat::Jpeg, Some(q)) => {
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(out.as_file_mut(), q.clamp(1, 100));
            encoder.encode_image(&img)?;
        }
        _ => {
            img.write_to(&mut std::io::BufWriter::new(out.as_file_mut()), dst_format)?;
        }
    }

    let final_path = out.commit()?;

    // Verificación final: reabrir o ficheiro escrito e comprobar que
    // decodifica correctamente e ten as dimensións agardadas. Se non, non
    // deixamos o ficheiro corrompido en pé de cara ao usuario.
    match image::open(&final_path) {
        Ok(check) if check.width() == img.width() && check.height() == img.height() => Ok(final_path),
        _ => {
            let _ = std::fs::remove_file(&final_path);
            Err(EngineError::EmptyOrCorruptOutput)
        }
    }
}

/// Edicións non destrutivas dunha imaxe. Sempre se escribe nun ficheiro novo.
#[derive(Debug, Clone, Default)]
pub struct ImageEdits {
    /// Xiro en sentido horario: 0, 90, 180 ou 270 graos.
    pub rotate: u16,
    pub flip_h: bool,
    pub flip_v: bool,
    pub grayscale: bool,
    pub invert: bool,
    /// De -100 a 100 (0 non cambia nada).
    pub brightness: i32,
    /// De -100 a 100 (0 non cambia nada).
    pub contrast: f32,
    /// Desenfoque gaussiano (sigma); 0 non desenfoca.
    pub blur: f32,
    /// Recorte (x, y, ancho, alto) en píxeles da imaxe orixinal.
    pub crop: Option<(u32, u32, u32, u32)>,
    pub quality: Option<u8>,
    pub resize: Option<(u32, u32)>,
}

/// Aplica as edicións en orde fixa (recorte, xiro/volteo, cor, desenfoque,
/// tamaño) e garda o resultado cas mesmas garantías que `convert_image`.
pub fn edit_image(
    input: &Path,
    output: &Path,
    overwrite: bool,
    edits: &ImageEdits,
) -> Result<PathBuf> {
    use image::imageops;
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    crate::integrity::ensure_distinct(&[input], output)?;
    let dst_ext = formats::extension_of(output).ok_or_else(|| EngineError::UnsupportedConversion {
        from: "imaxe".into(),
        to: "(sen extensión)".into(),
    })?;
    let dst_format = format_from_ext(&dst_ext)?;
    let mut img = image::open(input)?;

    if let Some((x, y, w, h)) = edits.crop {
        if w == 0 || h == 0 || x >= img.width() || y >= img.height() {
            return Err(EngineError::IntegrityCheck("O recorte queda fóra da imaxe.".into()));
        }
        let w = w.min(img.width() - x);
        let h = h.min(img.height() - y);
        img = img.crop_imm(x, y, w, h);
    }
    img = match edits.rotate % 360 {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        0 => img,
        _ => return Err(EngineError::IntegrityCheck("O xiro só pode ser de 0°, 90°, 180° ou 270°.".into())),
    };
    if edits.flip_h {
        img = img.fliph();
    }
    if edits.flip_v {
        img = img.flipv();
    }
    if edits.grayscale {
        img = image::DynamicImage::ImageLuma8(img.to_luma8()).to_rgba8().into();
    }
    if edits.brightness != 0 {
        img = img.brighten(edits.brightness.clamp(-100, 100) * 255 / 100);
    }
    if edits.contrast != 0.0 {
        img = img.adjust_contrast(edits.contrast.clamp(-100.0, 100.0));
    }
    if edits.invert {
        img.invert();
    }
    if edits.blur > 0.0 {
        img = img.blur(edits.blur.min(50.0));
    }
    if let Some((w, h)) = edits.resize {
        if w == 0 || h == 0 {
            return Err(EngineError::EmptyOrCorruptOutput);
        }
        img = image::DynamicImage::from(imageops::resize(&img, w, h, imageops::FilterType::Lanczos3));
    }
    write_verified(&img, output, overwrite, dst_format, edits.quality)
}

/// Garda `img` de forma atómica e comproba que se pode reabrir.
fn write_verified(
    img: &image::DynamicImage,
    output: &Path,
    overwrite: bool,
    format: ImageFormat,
    quality: Option<u8>,
) -> Result<PathBuf> {
    // JPEG non admite canle alfa.
    let img = if format == ImageFormat::Jpeg && img.color().has_alpha() {
        image::DynamicImage::ImageRgb8(img.to_rgb8())
    } else {
        img.clone()
    };
    let mut out = AtomicOutput::new(output, overwrite)?;
    match (format, quality) {
        (ImageFormat::Jpeg, Some(q)) => {
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(out.as_file_mut(), q.clamp(1, 100));
            enc.encode_image(&img)?;
        }
        _ => img.write_to(&mut std::io::BufWriter::new(out.as_file_mut()), format)?,
    }
    let final_path = out.commit()?;
    match image::open(&final_path) {
        Ok(check) if check.width() == img.width() && check.height() == img.height() => Ok(final_path),
        _ => {
            let _ = std::fs::remove_file(&final_path);
            Err(EngineError::EmptyOrCorruptOutput)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converte_png_a_jpeg_e_verifica() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.png");
        let img = image::RgbImage::from_fn(16, 16, |x, y| {
            image::Rgb([x as u8 * 10, y as u8 * 10, 128])
        });
        image::DynamicImage::ImageRgb8(img).save(&input).unwrap();

        let output = dir.path().join("destino.jpg");
        let opts = ImageConvertOptions { quality: Some(85), resize: None };
        let result = convert_image(&input, &output, false, &opts).unwrap();

        assert!(result.exists());
        assert!(std::fs::metadata(&result).unwrap().len() > 0);
        let decoded = image::open(&result).unwrap();
        assert_eq!(decoded.width(), 16);
        assert_eq!(decoded.height(), 16);
    }

    #[test]
    fn converte_a_ppm() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::new(8, 8))
            .save(&input)
            .unwrap();
        let output = dir.path().join("destino.ppm");
        let result = convert_image(&input, &output, false, &Default::default()).unwrap();
        assert!(result.exists());
    }

    #[test]
    fn redimensiona_a_medidas_exactas_ainda_que_deforme() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.png");
        // Orixe cadrada (10x10): pedir 20x10 non mantén proporción, así que
        // isto comproba que se respectan as medidas EXACTAS pedidas.
        image::DynamicImage::ImageRgb8(image::RgbImage::new(10, 10))
            .save(&input)
            .unwrap();
        let output = dir.path().join("destino.png");
        let opts = ImageConvertOptions { quality: None, resize: Some((20, 10)) };
        let result = convert_image(&input, &output, false, &opts).unwrap();

        let decoded = image::open(&result).unwrap();
        assert_eq!(decoded.width(), 20);
        assert_eq!(decoded.height(), 10);
    }

    #[test]
    fn le_dimensions_sen_decodificar_todo() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::new(32, 18))
            .save(&input)
            .unwrap();
        assert_eq!(image_dimensions(&input).unwrap(), (32, 18));
    }

    #[test]
    fn non_sobrescribe_sen_permiso() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::new(4, 4))
            .save(&input)
            .unwrap();
        let output = dir.path().join("destino.png");
        std::fs::write(&output, b"xa existo").unwrap();
        let err = convert_image(&input, &output, false, &Default::default()).unwrap_err();
        assert!(matches!(err, EngineError::DestinationExists(_)));
    }

    #[test]
    fn edita_xira_e_recorta() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::new(20, 10)).save(&input).unwrap();
        let output = dir.path().join("editada.png");
        let edits = ImageEdits { rotate: 90, crop: Some((0, 0, 10, 10)), ..Default::default() };
        let r = edit_image(&input, &output, false, &edits).unwrap();
        let d = image::open(r).unwrap();
        assert_eq!((d.width(), d.height()), (10, 10));
        assert!(edit_image(&input, &input, true, &edits).is_err());
    }
}
