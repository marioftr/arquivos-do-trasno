//! Redimensionar imaxes (soas ou en lote): dentro dun cadro, por porcentaxe ou a un tamaño exacto. Sempre se garda un ficheiro novo
//! no mesmo formato que o orixinal; a orientación EXIF aplícase antes de medir, así que as fotos do móbil miden o que se ve.

use crate::error::{EngineError, Result};
use crate::image_convert::{convert_image, ImageConvertOptions};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Lado máximo e total de píxeles que se aceptan no resultado: máis dá memoria de sobra para estragar o equipo.
pub const MAX_SIDE: u32 = 30_000;
pub const MAX_PIXELS: u64 = 300_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Fit {
    /// Cabe dentro dun cadro de ancho × alto (calquera dos dous pode faltar) sen deformar. Con `no_upscale`, o que xa é máis pequeno non se amplía.
    Within { width: Option<u32>, height: Option<u32>, no_upscale: bool },
    /// Un tanto por cento do tamaño orixinal.
    Percent { percent: f32 },
    /// Ancho e alto exactos: ou se estica a imaxe, ou se escala ata encher e se recorta o que sobra polo centro.
    Exact { width: u32, height: u32, crop: bool },
}

#[derive(Debug, Clone, Serialize)]
pub struct ResizeReport {
    pub output_path: PathBuf,
    pub from: (u32, u32),
    pub to: (u32, u32),
    pub bytes_written: u64,
}

fn bad<T>(msg: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(msg.to_string()))
}

/// Tamaño final dunha imaxe de `from` (ancho, alto) segundo o axuste. Función pura: sen tocar ficheiros.
pub fn target_size(from: (u32, u32), fit: &Fit) -> Result<(u32, u32)> {
    let (sw, sh) = from;
    if sw == 0 || sh == 0 {
        return bad("A imaxe non ten tamaño.");
    }
    let to = match *fit {
        Fit::Within { width, height, no_upscale } => {
            let w = width.filter(|v| *v > 0);
            let h = height.filter(|v| *v > 0);
            if w.is_none() && h.is_none() {
                return bad("Indica polo menos un ancho ou un alto.");
            }
            let fw = w.map(|v| v as f64 / sw as f64);
            let fh = h.map(|v| v as f64 / sh as f64);
            let mut f = match (fw, fh) {
                (Some(a), Some(b)) => a.min(b),
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (None, None) => unreachable!(),
            };
            if no_upscale && f > 1.0 {
                f = 1.0;
            }
            let mut nw = ((sw as f64 * f).round() as u32).max(1);
            let mut nh = ((sh as f64 * f).round() as u32).max(1);
            // O lado que manda queda exacto (o redondeo non o pode pasar nin quedar un píxel curto).
            if let (Some(a), Some(bw)) = (fw, w) {
                if f == a {
                    nw = bw;
                }
            }
            if let (Some(b), Some(bh)) = (fh, h) {
                if f == b {
                    nh = bh;
                }
            }
            if let Some(bw) = w {
                nw = nw.min(bw);
            }
            if let Some(bh) = h {
                nh = nh.min(bh);
            }
            (nw, nh)
        }
        Fit::Percent { percent } => {
            if !percent.is_finite() || percent <= 0.0 || percent > 1000.0 {
                return bad("A porcentaxe ten que estar entre 1 e 1000.");
            }
            let k = percent as f64 / 100.0;
            (((sw as f64 * k).round() as u32).max(1), ((sh as f64 * k).round() as u32).max(1))
        }
        Fit::Exact { width, height, .. } => {
            if width == 0 || height == 0 {
                return bad("O ancho e o alto teñen que ser polo menos 1.");
            }
            (width, height)
        }
    };
    if to.0 > MAX_SIDE || to.1 > MAX_SIDE || to.0 as u64 * to.1 as u64 > MAX_PIXELS {
        return bad("Ese tamaño é demasiado grande: o resultado non cabería na memoria.");
    }
    Ok(to)
}

/// Tamaño da imaxe tal como se ve (coa orientación EXIF aplicada), sen decodificar os píxeles.
pub fn oriented_dimensions(path: &Path) -> Result<(u32, u32)> {
    use image::metadata::Orientation::*;
    use image::ImageDecoder;
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    let readable = crate::image_ffmpeg::readable(path)?;
    let mut dec = image::ImageReader::open(&readable)?.with_guessed_format()?.into_decoder()?;
    let (w, h) = dec.dimensions();
    Ok(match dec.orientation().unwrap_or(NoTransforms) {
        Rotate90 | Rotate270 | Rotate90FlipH | Rotate270FlipH => (h, w),
        _ => (w, h),
    })
}

/// Redimensiona unha imaxe e gárdaa en `output` (co formato da súa extensión). Nunca toca o orixinal.
pub fn resize_image(input: &Path, output: &Path, overwrite: bool, fit: &Fit, quality: Option<u8>, background: Option<[u8; 3]>, metadata: crate::image_meta::Metadata) -> Result<ResizeReport> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    crate::integrity::ensure_distinct(&[input], output)?;
    let from = oriented_dimensions(input)?;
    let to = target_size(from, fit)?;
    let crop = matches!(fit, Fit::Exact { crop: true, .. });
    let dst_ext = crate::formats::extension_of(output).unwrap_or_default();
    if crate::animated::is_animated(input) {
        if !crate::animated::can_keep_animation(input, &dst_ext) {
            return bad("Esta imaxe é animada: ao redimensionala só quedaría un fotograma, así que non o fago.");
        }
        if crop {
            return bad("Non podo recortar unha imaxe animada: escolle estirar ou outro modo.");
        }
    }
    let opts = ImageConvertOptions { quality, resize: Some(to), fill: crop, lossless: false, background, metadata, ..Default::default() };
    let done = convert_image(input, output, overwrite, &opts)?;
    let bytes_written = std::fs::metadata(&done)?.len();
    Ok(ResizeReport { output_path: done, from, to, bytes_written })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn within(w: Option<u32>, h: Option<u32>, no_upscale: bool) -> Fit {
        Fit::Within { width: w, height: h, no_upscale }
    }

    #[test]
    fn dentro_dun_cadro_mantén_a_proporción() {
        assert_eq!(target_size((4000, 3000), &within(Some(1000), Some(1000), false)).unwrap(), (1000, 750));
        assert_eq!(target_size((3000, 4000), &within(Some(1000), Some(1000), false)).unwrap(), (750, 1000));
        assert_eq!(target_size((4000, 3000), &within(Some(1920), None, false)).unwrap(), (1920, 1440));
        assert_eq!(target_size((4000, 3000), &within(None, Some(600), false)).unwrap(), (800, 600));
        // O lado que manda queda exacto aínda que o redondeo non o sería (1000/3 non é enteiro).
        assert_eq!(target_size((3, 7), &within(Some(1000), None, false)).unwrap().0, 1000);
        assert_eq!(target_size((7, 3), &within(None, Some(1000), false)).unwrap().1, 1000);
    }

    #[test]
    fn non_ampliar_deixa_as_pequenas_como_están() {
        assert_eq!(target_size((400, 300), &within(Some(1000), Some(1000), true)).unwrap(), (400, 300));
        assert_eq!(target_size((400, 300), &within(Some(1000), Some(1000), false)).unwrap(), (1000, 750));
        // Só o lado que pasa do cadro se encolle.
        assert_eq!(target_size((2000, 100), &within(Some(1000), Some(1000), true)).unwrap(), (1000, 50));
    }

    #[test]
    fn porcentaxe_e_tamaño_exacto() {
        assert_eq!(target_size((1001, 501), &Fit::Percent { percent: 50.0 }).unwrap(), (501, 251));
        assert_eq!(target_size((10, 10), &Fit::Percent { percent: 1.0 }).unwrap(), (1, 1));
        assert_eq!(target_size((10, 10), &Fit::Percent { percent: 300.0 }).unwrap(), (30, 30));
        assert_eq!(target_size((100, 50), &Fit::Exact { width: 64, height: 64, crop: true }).unwrap(), (64, 64));
    }

    #[test]
    fn rexeita_o_que_non_ten_sentido() {
        assert!(target_size((10, 10), &within(None, None, false)).is_err());
        assert!(target_size((10, 10), &within(Some(0), Some(0), false)).is_err());
        assert!(target_size((10, 10), &Fit::Percent { percent: 0.0 }).is_err());
        assert!(target_size((10, 10), &Fit::Percent { percent: f32::NAN }).is_err());
        assert!(target_size((10, 10), &Fit::Percent { percent: 5000.0 }).is_err());
        assert!(target_size((10, 10), &Fit::Exact { width: 0, height: 5, crop: false }).is_err());
        assert!(target_size((10, 10), &Fit::Exact { width: 40_000, height: 5, crop: false }).is_err());
        assert!(target_size((10, 10), &Fit::Exact { width: 20_000, height: 20_000, crop: false }).is_err());
        assert!(target_size((0, 10), &Fit::Percent { percent: 50.0 }).is_err());
    }
}
