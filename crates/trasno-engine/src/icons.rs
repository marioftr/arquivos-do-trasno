//! Paquete de iconas a partir dunha imaxe: `.ico` de Windows, `.icns` de macOS, favicons e PNG soltos de varios tamaños.
//! Sempre se crea un cartafol novo (nunca se pisa nada) e a imaxe de orixe non se toca.

use crate::error::{EngineError, Result};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::{Path, PathBuf};

fn bad<T>(msg: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(msg.to_string()))
}

#[derive(Debug, Clone, Deserialize)]
pub struct IconOptions {
    /// `icon.ico` con 16–256 px.
    pub windows: bool,
    /// `icon.icns` con 16–1024 px.
    pub mac: bool,
    /// favicon.ico, favicons PNG, apple-touch-icon, iconas de Android e `site.webmanifest`.
    pub web: bool,
    /// PNG soltos de 16 a 1024 px en `png/`.
    pub png: bool,
    /// Se a imaxe non é cadrada: `false` recorta polo centro (enche), `true` encaixa enteira con marxes transparentes.
    pub fit: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct IconReport {
    pub folder: PathBuf,
    pub files: Vec<String>,
    pub bytes_written: u64,
    /// Lado da imaxe cadrada de orixe: se é menor ca 1024, as iconas grandes saen ampliadas.
    pub source_side: u32,
}

const ICO_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];
const ICNS: [(&[u8; 4], u32); 7] = [(b"icp4", 16), (b"icp5", 32), (b"icp6", 64), (b"ic07", 128), (b"ic08", 256), (b"ic09", 512), (b"ic10", 1024)];
const PNG_SIZES: [u32; 8] = [16, 32, 48, 64, 128, 256, 512, 1024];

/// A imaxe pasada a cadrada: recortada polo centro ou encaixada con marxes transparentes.
fn square(img: &DynamicImage, fit: bool) -> RgbaImage {
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    if w == h {
        return rgba;
    }
    if fit {
        let side = w.max(h);
        let mut out = RgbaImage::new(side, side);
        image::imageops::overlay(&mut out, &rgba, ((side - w) / 2) as i64, ((side - h) / 2) as i64);
        out
    } else {
        let side = w.min(h);
        image::imageops::crop_imm(&rgba, (w - side) / 2, (h - side) / 2, side, side).to_image()
    }
}

fn png_at(src: &RgbaImage, size: u32) -> Result<Vec<u8>> {
    let scaled = if src.width() == size { src.clone() } else { image::imageops::resize(src, size, size, FilterType::Lanczos3) };
    let mut buf = Vec::new();
    DynamicImage::ImageRgba8(scaled).write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
    Ok(buf)
}

/// `.ico` con una entrada PNG por tamaño (válido desde Windows Vista).
pub fn ico_bytes(src: &RgbaImage, sizes: &[u32]) -> Result<Vec<u8>> {
    let blobs: Vec<(u32, Vec<u8>)> = sizes.iter().map(|s| Ok((*s, png_at(src, *s)?))).collect::<Result<_>>()?;
    let mut out = vec![0, 0, 1, 0];
    out.extend((blobs.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * blobs.len() as u32;
    for (s, b) in &blobs {
        let d = if *s >= 256 { 0 } else { *s as u8 };
        out.extend([d, d, 0, 0, 1, 0, 32, 0]);
        out.extend((b.len() as u32).to_le_bytes());
        out.extend(offset.to_le_bytes());
        offset += b.len() as u32;
    }
    for (_, b) in blobs {
        out.extend(b);
    }
    Ok(out)
}

/// `.icns` de macOS: unha entrada PNG por tamaño.
pub fn icns_bytes(src: &RgbaImage) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    for (kind, size) in ICNS {
        let png = png_at(src, size)?;
        body.extend(kind);
        body.extend((png.len() as u32 + 8).to_be_bytes());
        body.extend(png);
    }
    let mut out = b"icns".to_vec();
    out.extend((body.len() as u32 + 8).to_be_bytes());
    out.extend(body);
    Ok(out)
}

/// Crea o paquete en `dir/<stem>-iconas` (ou co « (2)» ao final se xa existe). Se algo falla, non queda nada a medias.
pub fn make_icon_pack(input: &Path, dir: &Path, stem: &str, o: &IconOptions) -> Result<IconReport> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    if !(o.windows || o.mac || o.web || o.png) {
        return bad("Escolle polo menos un tipo de iconas.");
    }
    if crate::animated::is_animated(input) {
        return bad("Esta imaxe é animada: unha icona só leva un fotograma, así que non o fago.");
    }
    let (img, _) = crate::image_meta::open_oriented(input, crate::image_meta::Metadata::Strip)?;
    let src = square(&img, o.fit);
    let side = src.width();
    if side < 16 {
        return bad("A imaxe é demasiado pequena para facer iconas (mínimo 16 × 16).");
    }

    // Todo vai a un cartafol temporal e, ao final, renómease: nunca queda un paquete a medias.
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    if o.windows {
        files.push(("icon.ico".into(), ico_bytes(&src, &ICO_SIZES)?));
    }
    if o.mac {
        files.push(("icon.icns".into(), icns_bytes(&src)?));
    }
    if o.web {
        files.push(("favicon.ico".into(), ico_bytes(&src, &[16, 32, 48])?));
        for (name, s) in [("favicon-16x16.png", 16), ("favicon-32x32.png", 32), ("apple-touch-icon.png", 180), ("android-chrome-192x192.png", 192), ("android-chrome-512x512.png", 512)] {
            files.push((name.into(), png_at(&src, s)?));
        }
        let manifest = "{\n  \"icons\": [\n    { \"src\": \"android-chrome-192x192.png\", \"sizes\": \"192x192\", \"type\": \"image/png\" },\n    { \"src\": \"android-chrome-512x512.png\", \"sizes\": \"512x512\", \"type\": \"image/png\" }\n  ]\n}\n";
        files.push(("site.webmanifest".into(), manifest.as_bytes().to_vec()));
    }
    if o.png {
        for s in PNG_SIZES {
            files.push((format!("png/icon-{s}.png"), png_at(&src, s)?));
        }
    }

    std::fs::create_dir_all(dir)?;
    let mut target = dir.join(format!("{stem}-iconas"));
    let mut k = 2;
    while target.exists() {
        target = dir.join(format!("{stem}-iconas ({k})"));
        k += 1;
        if k > 1000 {
            return bad("Hai demasiados cartafoles co mesmo nome.");
        }
    }
    let tmp = dir.join(format!(".trasno-iconas-{}.part", uuid::Uuid::new_v4()));
    let write = || -> Result<u64> {
        let mut total = 0u64;
        for (name, data) in &files {
            let p = tmp.join(name);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&p, data)?;
            total += data.len() as u64;
        }
        Ok(total)
    };
    let total = match write() {
        Ok(t) => t,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&tmp);
            return Err(e);
        }
    };
    if let Err(e) = std::fs::rename(&tmp, &target) {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(e.into());
    }
    Ok(IconReport { folder: target, files: files.into_iter().map(|(n, _)| n).collect(), bytes_written: total, source_side: side })
}
