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

pub mod compress;
pub mod error;
pub mod ffmpeg;
pub mod formats;
pub mod image_convert;
pub mod integrity;
pub mod model3d;
pub mod pdf;
pub mod pdf_tools;

pub use error::{EngineError, Result};
pub use formats::Category;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConvertOptions {
    pub overwrite: bool,
    pub quality: Option<u8>,
    pub resize: Option<(u32, u32)>,
    /// Só para audio/vídeo (delegado en ffmpeg): bitrate obxectivo en kbps.
    /// `None` deixa que ffmpeg escolla o seu bitrate por defecto.
    pub bitrate_kbps: Option<u32>,
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
    integrity::ensure_distinct(&[input], output)?;
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
            let mut extra_args: Vec<String> = Vec::new();
            if dst_category == Category::Audio && src_category == Category::Video {
                // Sen isto, ffmpeg tenta meter tamén o stream de vídeo nun
                // contedor que só admite audio (mp3, flac...) e falla.
                extra_args.push("-vn".to_string());
            }
            if let Some(kbps) = options.bitrate_kbps {
                let flag = if dst_category == Category::Audio { "-b:a" } else { "-b:v" };
                extra_args.push(flag.to_string());
                extra_args.push(format!("{kbps}k"));
            }
            ffmpeg::convert_media(input, output, options.overwrite, &extra_args)?
        }
        (Category::Modelo3d, Category::Modelo3d) => {
            model3d::convert_model3d(input, output, options.overwrite)?
        }
        (Category::Documento, Category::Documento) => {
            let in_ext = formats::extension_of(input).unwrap_or_default();
            let out_ext = formats::extension_of(output).unwrap_or_default();
            match (in_ext.as_str(), out_ext.as_str()) {
                (e, "pdf") if formats::PLAIN_TEXT_EXTS.contains(&e) => pdf::text_to_pdf(input, output, options.overwrite)?,
                ("pdf", "txt") => pdf::pdf_to_text(input, output, options.overwrite)?,
                _ => {
                    return Err(EngineError::UnsupportedConversion {
                        from: in_ext,
                        to: out_ext,
                    })
                }
            }
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
        Category::Documento => match formats::extension_of(input).as_deref() {
            Some("pdf") => vec!["txt"],
            Some(e) if formats::PLAIN_TEXT_EXTS.contains(&e) => vec!["pdf"],
            _ => Vec::new(),
        },
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
    fn dispatch_texto_a_pdf_e_pdf_a_texto() {
        let dir = tempfile::tempdir().unwrap();
        let txt = dir.path().join("nota.txt");
        std::fs::write(&txt, "Ola dende o trasno!").unwrap();

        let pdf = dir.path().join("nota.pdf");
        let report = convert_file(&txt, &pdf, &ConvertOptions::default()).unwrap();
        assert_eq!(report.category, Category::Documento);

        let volta = dir.path().join("volta.txt");
        convert_file(&pdf, &volta, &ConvertOptions::default()).unwrap();
        let contido = std::fs::read_to_string(&volta).unwrap();
        assert!(contido.contains("Ola dende o trasno!"));
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
