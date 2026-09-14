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
}
