//! trasno-engine: motor de conversión e manipulación de ficheiros
//! multimedia da aplicación Trasno.
//!
//! Deseño xeral:
//! - Nunca se escribe directamente sobre o ficheiro de destino: sempre se
//!   pasa por un temporal no mesmo directorio e faise un `rename` atómico
//!   ao final (ver `integrity`), tras validar o resultado.
//! - O motor non asume unha GUI concreta: expón funcións simples que
//!   devolven `Result<PathBuf>` ou erros claros en galego, pensadas para
//!   chamarse tanto desde comandos de Tauri coma desde un futuro proceso
//!   de fondo (bandexa do sistema / menú contextual).
//! - Imaxes: Rust puro (crate `image`), sen dependencias externas.
//! - Audio/vídeo: delega en ffmpeg/ffprobe do sistema (ou empaquetado máis
//!   adiante como sidecar).
//! - PDF: operacións estruturais con `lopdf` (unir, dividir, rotar). A
//!   sinatura dixital e a conversión doc↔pdf quedan para outra fase.

pub mod error;
pub mod ffmpeg;
pub mod formats;
pub mod image_convert;
pub mod integrity;
pub mod model3d;
pub mod pdf;

pub use error::{EngineError, Result};
pub use formats::Category;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConvertOptions {
    pub overwrite: bool,
    pub quality: Option<u8>,
    pub resize: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConvertReport {
    pub output_path: PathBuf,
    pub bytes_written: u64,
    pub category: Category,
}

/// Punto de entrada único do conversor universal: mira as extensións de
/// orixe e destino, decide que backend usar (imaxe / ffmpeg) e executa a
/// conversión coas garantías de integridade do módulo `integrity`.
pub fn convert_file(input: &Path, output: &Path, options: &ConvertOptions) -> Result<ConvertReport> {
    let src_category = formats::category_of_path(input);
    let dst_category = formats::category_of_path(output);

    let output_path = match (src_category, dst_category) {
        (Category::Imaxe, Category::Imaxe) => image_convert::convert_image(
            input,
            output,
            options.overwrite,
            &image_convert::ImageConvertOptions {
                quality: options.quality,
                resize: options.resize,
            },
        )?,
        (Category::Audio, Category::Audio)
        | (Category::Video, Category::Video)
        | (Category::Video, Category::Audio)
        | (Category::Audio, Category::Video)
        | (Category::Imaxe, Category::Video)
        | (Category::Video, Category::Imaxe) => {
            ffmpeg::convert_media(input, output, options.overwrite, &[])?
        }
        (Category::Modelo3d, Category::Modelo3d) => {
            model3d::convert_model3d(input, output, options.overwrite)?
        }
        (from, to) => {
            return Err(EngineError::UnsupportedConversion {
                from: format!("{from:?}"),
                to: format!("{to:?}"),
            })
        }
    };

    let bytes_written = integrity::assert_non_empty_file(&output_path)?;

    Ok(ConvertReport {
        output_path,
        bytes_written,
        category: dst_category,
    })
}

/// Formatos de destino razoables para un ficheiro de entrada dado,
/// pensado para que a UI só ofreza opcións que teñan sentido.
pub fn suggested_targets(input: &Path) -> Vec<&'static str> {
    match formats::category_of_path(input) {
        Category::Imaxe => formats::IMAGE_ENCODE.to_vec(),
        Category::Audio => formats::AUDIO_EXTS.to_vec(),
        Category::Video => formats::VIDEO_EXTS.to_vec(),
        Category::Documento => formats::DOCUMENT_EXTS.to_vec(),
        Category::Modelo3d => formats::MODEL3D_ENGINE_SUPPORTED.to_vec(),
        Category::Descoñecido => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_imaxe_a_imaxe() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::new(10, 10))
            .save(&input)
            .unwrap();
        let output = dir.path().join("b.bmp");
        let report = convert_file(&input, &output, &ConvertOptions::default()).unwrap();
        assert!(report.bytes_written > 0);
        assert_eq!(report.category, Category::Imaxe);
    }

    #[test]
    fn combinacion_non_soportada_da_erro_claro() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::new(4, 4))
            .save(&input)
            .unwrap();
        let output = dir.path().join("b.pdf");
        let err = convert_file(&input, &output, &ConvertOptions::default()).unwrap_err();
        assert!(matches!(err, EngineError::UnsupportedConversion { .. }));
    }
}
