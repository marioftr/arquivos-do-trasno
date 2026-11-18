//! Cortar unha imaxe en partes iguais (columnas × filas) dentro dunha zona, por exemplo para repartir unha foto en cadrados
//! e subila como carrusel. Sempre se escriben ficheiros novos: `nome-1.png`, `nome-2.png`…

use crate::error::{EngineError, Result};
use crate::image_convert::write_verified;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Máximo de columnas ou de filas.
pub const MAX_SIDE_PARTS: u32 = 20;
/// Máximo de partes en total.
pub const MAX_PARTS: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceOptions {
    /// Zona que se reparte (x, y, ancho, alto), en píxeles da imaxe tal como se ve.
    pub rect: (u32, u32, u32, u32),
    pub cols: u32,
    pub rows: u32,
    /// Numerar de arriba abaixo por columnas (se non, de esquerda a dereita por filas).
    pub by_columns: bool,
    /// Calidade para JPG.
    pub quality: Option<u8>,
    /// Fondo se a imaxe ten transparencia e o formato non a admite.
    pub background: [u8; 3],
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceReport {
    pub outputs: Vec<PathBuf>,
    /// Tamaño de cada parte.
    pub piece: (u32, u32),
    pub bytes_written: u64,
}

fn bad<T>(msg: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(msg.to_string()))
}

/// Tamaño de cada parte: a zona dividida entre columnas e filas (o que sobra, menos dun píxel por parte, queda fóra).
pub fn piece_size(rect: (u32, u32, u32, u32), cols: u32, rows: u32) -> Result<(u32, u32)> {
    if cols == 0 || rows == 0 || cols > MAX_SIDE_PARTS || rows > MAX_SIDE_PARTS {
        return bad("Escolle entre 1 e 20 columnas e filas.");
    }
    if cols * rows > MAX_PARTS {
        return bad("O máximo son 100 partes: baixa as columnas ou as filas.");
    }
    let (pw, ph) = (rect.2 / cols, rect.3 / rows);
    if pw == 0 || ph == 0 {
        return bad("Cada parte sairía de menos dun píxel: amplía a zona ou fai menos partes.");
    }
    Ok((pw, ph))
}

/// Número (desde 1) da parte da columna `c` e fila `r`.
pub fn part_number(c: u32, r: u32, cols: u32, rows: u32, by_columns: bool) -> u32 {
    if by_columns {
        c * rows + r + 1
    } else {
        r * cols + c + 1
    }
}

/// Nomes das partes: `nome-1.ext`… (con ceros á esquerda se hai máis de 9). Se algún xa existe, todo o xogo pasa a `nome (2)-1.ext`…,
/// para que as partes dun mesmo corte non se mesturen con outras.
pub fn plan_outputs(dir: &Path, stem: &str, ext: &str, parts: u32) -> Result<Vec<PathBuf>> {
    let width = parts.to_string().len();
    for k in 1..=1000u32 {
        let base = if k == 1 { stem.to_string() } else { format!("{stem} ({k})") };
        let names: Vec<PathBuf> = (1..=parts).map(|n| dir.join(format!("{base}-{n:0width$}.{ext}"))).collect();
        if names.iter().all(|p| !p.exists()) {
            return Ok(names);
        }
    }
    bad("Hai demasiados ficheiros co mesmo nome nese cartafol.")
}

pub fn slice_image(input: &Path, dir: &Path, stem: &str, ext: &str, o: &SliceOptions) -> Result<SliceReport> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    let ext = ext.to_ascii_lowercase();
    let format = ImageFormat::from_extension(&ext).ok_or_else(|| EngineError::UnsupportedConversion { from: "imaxe".into(), to: ext.clone() })?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP | ImageFormat::Bmp | ImageFormat::Tiff) || crate::image_ffmpeg::is_encode_ext(&ext) {
        return bad("As partes gárdanse en PNG, JPG, WebP, BMP ou TIFF.");
    }
    let (x0, y0, rw, rh) = o.rect;
    let (pw, ph) = piece_size(o.rect, o.cols, o.rows)?;
    if crate::animated::is_animated(input) {
        return bad("Esta imaxe é animada: ao cortala só se gardaría un fotograma, así que non o fago.");
    }
    let (w, h) = crate::resize::oriented_dimensions(input)?;
    if rw == 0 || rh == 0 || x0 as u64 + rw as u64 > w as u64 || y0 as u64 + rh as u64 > h as u64 {
        return bad("A zona queda fóra da imaxe.");
    }
    let (img, _) = crate::image_meta::open_oriented(input, crate::image_meta::Metadata::Strip)?;
    let outputs = plan_outputs(dir, stem, &ext, o.cols * o.rows)?;

    let mut done: Vec<PathBuf> = Vec::new();
    let mut bytes = 0u64;
    for r in 0..o.rows {
        for c in 0..o.cols {
            let n = part_number(c, r, o.cols, o.rows, o.by_columns);
            let part = img.crop_imm(x0 + c * pw, y0 + r * ph, pw, ph);
            let result = write_verified(&part, &outputs[(n - 1) as usize], false, format, o.quality, Some(o.background)).and_then(|p| {
                bytes += std::fs::metadata(&p)?.len();
                Ok(p)
            });
            match result {
                Ok(p) => done.push(p),
                Err(e) => {
                    // Sen partes soltas: se algunha falla, quítanse as que xa estaban escritas.
                    for p in &done {
                        let _ = std::fs::remove_file(p);
                    }
                    return Err(e);
                }
            }
        }
    }
    Ok(SliceReport { outputs, piece: (pw, ph), bytes_written: bytes })
}
