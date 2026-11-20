//! Xuntar nunha soa imaxe, de arriba abaixo ou de esquerda a dereita. Con «quitar o repetido», busca onde cada captura
//! continúa a anterior (as de desprazamento repiten unha franxa, e ás veces cabeceira e pé fixos) e quita o que sobra.

use crate::error::{EngineError, Result};
use crate::image_convert::{flatten_on, write_verified};
use image::{DynamicImage, ImageFormat, RgbImage};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MAX_SIDE: u32 = 65_000;
pub const MAX_PIXELS: u64 = 300_000_000;
/// WebP non admite máis de 16383 píxeles por lado.
const WEBP_SIDE: u32 = 16_383;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Unha debaixo doutra.
    Vertical,
    /// Unha ao carón doutra.
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    /// Todas ao ancho (ou alto, en horizontal) da máis estreita.
    Narrowest,
    /// Todas ao ancho da máis ancha.
    Widest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StitchOptions {
    pub direction: Direction,
    pub align: Align,
    /// Busca onde cada captura continúa a anterior e quita o repetido.
    pub remove_overlap: bool,
    /// Espazo entre capturas que non se solapan (px).
    pub gap: u32,
    /// Fondo do espazo e da transparencia.
    pub background: [u8; 3],
    pub quality: Option<u8>,
}

impl Default for StitchOptions {
    fn default() -> Self {
        StitchOptions { direction: Direction::Vertical, align: Align::Narrowest, remove_overlap: true, gap: 0, background: [255, 255, 255], quality: Some(92) }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StitchReport {
    pub output_path: PathBuf,
    pub size: (u32, u32),
    /// Por cada unión (unha menos ca capturas): píxeles repetidos que se quitaron, ou `None` se non se atopou onde se solapan.
    pub joins: Vec<Option<u32>>,
    pub bytes_written: u64,
}

fn bad<T>(msg: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(msg.to_string()))
}

const BINS: usize = 48;

/// Resumo de cada fila: o gris medio de `BINS` tramos. Compárase ese resumo en vez de todos os píxeles: chega para recoñecer a mesma
/// franxa de contido e aguanta o suavizado e o ruído de compresión.
fn row_features(img: &RgbImage) -> Vec<[u8; BINS]> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let raw = img.as_raw();
    let mut out = Vec::with_capacity(h);
    for y in 0..h {
        let row = &raw[y * w * 3..(y + 1) * w * 3];
        let mut f = [0u8; BINS];
        for (b, slot) in f.iter_mut().enumerate() {
            let x0 = b * w / BINS;
            let x1 = ((b + 1) * w / BINS).max(x0 + 1).min(w);
            let mut sum = 0u32;
            for px in row[x0 * 3..x1 * 3].chunks_exact(3) {
                sum += (px[0] as u32 * 30 + px[1] as u32 * 59 + px[2] as u32 * 11) / 100;
            }
            *slot = (sum / (x1 - x0) as u32) as u8;
        }
        out.push(f);
    }
    out
}

/// Diferenza media permitida entre dúas franxas (de 0 a 255) para darlles por iguais.
const MATCH_TOLERANCE: f64 = 3.0;
/// Contraste mínimo dunha franxa para fiarse dela (unha franxa lisa coincide con calquera lugar).
const MIN_CONTRAST: f64 = 6.0;

fn band_contrast(f: &[[u8; BINS]]) -> f64 {
    // Desvío típico dos grises da franxa: se case todo é igual, non serve para atopar nada.
    let n = (f.len() * BINS) as f64;
    let mean = f.iter().flat_map(|r| r.iter()).map(|v| *v as f64).sum::<f64>() / n;
    (f.iter().flat_map(|r| r.iter()).map(|v| (*v as f64 - mean).powi(2)).sum::<f64>() / n).sqrt()
}

/// Onde segue `b` a `a`: devolve `(p, y0)` tal que a franxa de `b` que comeza na fila `y0` coincide con `a` desde a fila `p`
/// (con `p > y0`: `b` está máis abaixo). Unir cortando `a` en `p` e collendo `b` desde `y0` quita o repetido e tamén a cabeceira
/// fixa de `b` e o pé fixo de `a`. `a_from` é a primeira fila de `a` que aínda se usa.
fn find_join(fa: &[[u8; BINS]], a_from: usize, fb: &[[u8; BINS]]) -> Option<(usize, usize)> {
    let (ha, hb) = (fa.len(), fb.len());
    if hb < 48 || ha < 48 {
        return None;
    }
    let band = (hb / 12).clamp(24, 120);
    let step = (band / 2).max(1);
    let mut best: Option<(f64, usize, usize)> = None; // (erro, p, y0)
    let mut y0 = hb / 25;
    while y0 + band <= hb {
        let t = &fb[y0..y0 + band];
        if band_contrast(t) >= MIN_CONTRAST {
            let limit = MATCH_TOLERANCE * (band * BINS) as f64;
            let mut local: Option<(f64, usize)> = None;
            // `a` ten que ter a franxa máis abaixo ca en `b` (p > y0) e deixar algo de `a` antes (p > a_from).
            let lo = (y0 + 1).max(a_from + 1);
            if lo + band <= ha {
                for p in lo..=ha - band {
                    let cap = local.map(|l| l.0).unwrap_or(limit).min(limit);
                    let mut sum = 0.0;
                    'rows: for (ra, rb) in fa[p..p + band].iter().zip(t) {
                        for k in 0..BINS {
                            sum += (ra[k] as i32 - rb[k] as i32).unsigned_abs() as f64;
                        }
                        if sum > cap {
                            sum = f64::INFINITY;
                            break 'rows;
                        }
                    }
                    if sum.is_finite() && sum <= cap && local.map(|l| sum < l.0).unwrap_or(true) {
                        local = Some((sum, p));
                    }
                }
            }
            if let Some((e, p)) = local {
                // Gaña o erro máis pequeno; a igualdade, a franxa máis alta (a que deixa máis de `b`).
                if best.map(|b| e < b.0 - 1e-9).unwrap_or(true) {
                    best = Some((e, p, y0));
                }
            }
        }
        y0 += step;
    }
    best.map(|(_, p, y0)| (p, y0))
}

/// Un tramo de filas dunha imaxe que acaba no resultado.
struct Piece {
    img: usize,
    from: usize,
    to: usize,
}

fn side_limit(ext: &str) -> u32 {
    if ext == "webp" {
        WEBP_SIDE
    } else {
        MAX_SIDE
    }
}

pub fn stitch_images(inputs: &[PathBuf], output: &Path, overwrite: bool, o: &StitchOptions) -> Result<StitchReport> {
    if inputs.len() < 2 {
        return bad("Preciso polo menos dúas imaxes para xuntalas.");
    }
    let refs: Vec<&Path> = inputs.iter().map(|p| p.as_path()).collect();
    crate::integrity::ensure_distinct(&refs, output)?;
    let ext = crate::formats::extension_of(output).unwrap_or_default();
    let format = ImageFormat::from_extension(&ext).ok_or_else(|| EngineError::UnsupportedConversion { from: "imaxe".into(), to: ext.clone() })?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP | ImageFormat::Bmp | ImageFormat::Tiff) || crate::image_ffmpeg::is_encode_ext(&ext) {
        return bad("As capturas xúntanse en PNG, JPG, WebP, BMP ou TIFF.");
    }
    let horizontal = o.direction == Direction::Horizontal;

    // Medidas primeiro (sen decodificar): o resultado ten que caber antes de gastar memoria.
    let mut sizes = Vec::new();
    for p in inputs {
        if crate::animated::is_animated(p) {
            return bad("Unha das imaxes é animada: só colleríase un fotograma, así que non as xunto.");
        }
        let (w, h) = crate::resize::oriented_dimensions(p)?;
        sizes.push(if horizontal { (h, w) } else { (w, h) });
    }
    let target_w = match o.align {
        Align::Narrowest => sizes.iter().map(|s| s.0).min().unwrap(),
        Align::Widest => sizes.iter().map(|s| s.0).max().unwrap(),
    };
    let scaled_h: Vec<u64> = sizes.iter().map(|&(w, h)| ((h as f64 * target_w as f64 / w as f64).round() as u64).max(1)).collect();
    let rough_h: u64 = scaled_h.iter().sum::<u64>() + o.gap as u64 * (inputs.len() as u64 - 1);
    let (rw, rh) = if horizontal { (rough_h, target_w as u64) } else { (target_w as u64, rough_h) };
    let limit = side_limit(&ext) as u64;
    if rw > limit || rh > limit || rw * rh > MAX_PIXELS {
        return bad("O resultado sería demasiado grande para este formato. Proba con menos capturas, noutra dirección ou en PNG.");
    }

    // Todas á mesma anchura, en vertical (en horizontal xíranse e desfaise ao final).
    let mut imgs: Vec<RgbImage> = Vec::new();
    for p in inputs {
        let (img, _) = crate::image_meta::open_oriented(p, crate::image_meta::Metadata::Strip)?;
        let mut rgb = flatten_on(&DynamicImage::from(img), o.background);
        if horizontal {
            rgb = image::imageops::rotate90(&rgb);
        }
        if rgb.width() != target_w {
            let nh = ((rgb.height() as f64 * target_w as f64 / rgb.width() as f64).round() as u32).max(1);
            rgb = image::imageops::resize(&rgb, target_w, nh, image::imageops::FilterType::Lanczos3);
        }
        imgs.push(rgb);
    }

    // Tramos e unións.
    let feats: Vec<Vec<[u8; BINS]>> = if o.remove_overlap { imgs.iter().map(row_features).collect() } else { Vec::new() };
    let mut pieces: Vec<Piece> = imgs.iter().enumerate().map(|(i, im)| Piece { img: i, from: 0, to: im.height() as usize }).collect();
    let mut joins: Vec<Option<u32>> = Vec::new();
    for i in 1..imgs.len() {
        let found = if o.remove_overlap { find_join(&feats[i - 1], pieces[i - 1].from, &feats[i]) } else { None };
        match found {
            Some((p, y0)) => {
                let shift = p - y0;
                let overlap = (imgs[i - 1].height() as usize).saturating_sub(shift);
                pieces[i - 1].to = pieces[i - 1].to.min(p);
                pieces[i].from = y0;
                joins.push(Some(overlap as u32));
            }
            None => joins.push(None),
        }
    }

    let gaps = joins.iter().filter(|j| j.is_none()).count() as u32;
    let total_h: u32 = pieces.iter().map(|p| (p.to - p.from) as u32).sum::<u32>() + gaps * o.gap;
    let mut canvas = RgbImage::from_pixel(target_w, total_h.max(1), image::Rgb(o.background));
    let row_bytes = target_w as usize * 3;
    let mut y = 0usize;
    for (i, piece) in pieces.iter().enumerate() {
        let raw = imgs[piece.img].as_raw();
        let dst = canvas.as_mut();
        for r in piece.from..piece.to {
            dst[y * row_bytes..(y + 1) * row_bytes].copy_from_slice(&raw[r * row_bytes..(r + 1) * row_bytes]);
            y += 1;
        }
        if i < joins.len() && joins[i].is_none() {
            y += o.gap as usize;
        }
    }
    drop(imgs);
    let result = if horizontal { image::imageops::rotate270(&canvas) } else { canvas };
    let size = (result.width(), result.height());
    let done = write_verified(&DynamicImage::ImageRgb8(result), output, overwrite, format, o.quality, Some(o.background))?;
    let bytes_written = std::fs::metadata(&done)?.len();
    Ok(StitchReport { output_path: done, size, joins, bytes_written })
}
