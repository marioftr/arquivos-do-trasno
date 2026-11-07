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

pub mod animated;
pub mod compress;
pub mod error;
pub mod ffmpeg;
pub mod fileinfo;
pub mod fonts;
pub mod formats;
pub mod image_convert;
pub mod image_ffmpeg;
pub mod integrity;
pub mod mkv;
pub mod model3d;
pub mod office;
pub mod pdf;
pub mod pdf_tools;
pub mod tools_install;

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
    /// Sen perdas: JPEG XL e JPEG 2000 en imaxes; H.264 sen compresión con perdas en MP4, MKV e MOV.
    #[serde(default)]
    pub lossless: bool,
    /// Fondo (RGB) para a transparencia ao pasar a un formato que non a admite (JPG, BMP, PNM). Por defecto, branco.
    #[serde(default)]
    pub background: Option<[u8; 3]>,
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
                lossless: options.lossless,
                background: options.background,
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
            let dst_ext = formats::extension_of(output).unwrap_or_default();
            if options.lossless && src_category == Category::Video && dst_category == Category::Video && matches!(dst_ext.as_str(), "mp4" | "mkv" | "mov") {
                // H.264 con QP 0: sen perdas respecto dos fotogramas do orixinal; o son vai en ALAC (MP4/MOV) ou FLAC (MKV).
                let audio = if dst_ext == "mkv" { "flac" } else { "alac" };
                for a in ["-c:v", "libx264", "-qp", "0", "-preset", "medium", "-c:a", audio] {
                    extra_args.push(a.to_string());
                }
            } else if let Some(kbps) = options.bitrate_kbps.filter(|_| dst_category == Category::Audio && dst_ext == "ogg") {
                // libvorbis non acepta calquera bitrate fixo (a 32 kbps estéreo ou a 320 kbps en mono de 8 kHz nin arranca):
                // tradúcese ao nivel de calidade equivalente, que sempre vale.
                const Q: [(u32, &str); 11] = [(45, "-1"), (64, "0"), (80, "1"), (96, "2"), (112, "3"), (128, "4"), (160, "5"), (192, "6"), (224, "7"), (256, "8"), (320, "9")];
                let q = Q.iter().rev().find(|(k, _)| kbps >= *k).map(|(_, q)| *q).unwrap_or("-1");
                for a in ["-c:a", "libvorbis", "-q:a", q] {
                    extra_args.push(a.to_string());
                }
            } else if let Some(kbps) = options.bitrate_kbps {
                let flag = if dst_category == Category::Audio { "-b:a" } else { "-b:v" };
                extra_args.push(flag.to_string());
                extra_args.push(format!("{kbps}k"));
            }
            // MPEG-1/2 só admite unhas cadencias concretas: unha de 10 ou 15 fps ou unha variable fan fallar a codificación.
            if dst_category == Category::Video && matches!(dst_ext.as_str(), "mpg" | "mpeg") {
                if let Some(r) = mpeg_rate(input) {
                    extra_args.push("-r".to_string());
                    extra_args.push(r);
                }
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

/// Cadencia para MPEG-1/2: a do orixinal se é unha das admitidas; se non, a admitida máis próxima.
fn mpeg_rate(input: &Path) -> Option<String> {
    const OK: [(f64, &str); 8] = [(24000.0 / 1001.0, "24000/1001"), (24.0, "24"), (25.0, "25"), (30000.0 / 1001.0, "30000/1001"), (30.0, "30"), (50.0, "50"), (60000.0 / 1001.0, "60000/1001"), (60.0, "60")];
    let info = ffmpeg::probe(input).ok()?;
    let fps = info["streams"].as_array()?.iter().find(|s| s["codec_type"] == "video").and_then(|s| {
        let r = s["avg_frame_rate"].as_str().filter(|r| *r != "0/0").or_else(|| s["r_frame_rate"].as_str())?;
        let (n, d) = r.split_once('/')?;
        let (n, d): (f64, f64) = (n.parse().ok()?, d.parse().ok()?);
        (d > 0.0 && n > 0.0).then_some(n / d)
    })?;
    // Se xa é admitida (con marxe mínima), non se toca.
    if OK.iter().any(|(v, _)| (v - fps).abs() < 0.01) {
        return None;
    }
    OK.iter().min_by(|a, b| (a.0 - fps).abs().partial_cmp(&(b.0 - fps).abs()).unwrap()).map(|(_, s)| s.to_string())
}

/// Formatos de destino razoables para un ficheiro de entrada dado,
/// pensado para que a UI só ofreza opcións que teñan sentido.
pub fn suggested_targets(input: &Path) -> Vec<&'static str> {
    match formats::category_of_path(input) {
        Category::Imaxe => {
            let mut v = formats::IMAGE_ENCODE.to_vec();
            v.extend(image_ffmpeg::ENCODE_EXTS.iter().copied());
            v
        }
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
