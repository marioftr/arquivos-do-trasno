//! Restaurar imaxes sen IA xerativa: axustes de luz, cor, ruído e nitidez feitos píxel a píxel. Non se inventa detalle: só se corrixe o que xa hai.
//! A web fai o mesmo (`web/src/lib/enhance.ts`) cos mesmos nomes e a mesma orde.

use crate::error::{EngineError, Result};
use image::{DynamicImage, RgbaImage};
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Tipo de resultado: a cor da foto, unha páxina fotografada (fondo branco, texto negro) ou branco e negro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
    Photo,
    Document,
    Bw,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestoreOptions {
    #[serde(default)]
    pub kind: Kind,
    /// Estira a luz: o 0,5 % máis escuro pasa a negro e o 0,5 % máis claro a branco.
    #[serde(default)]
    pub auto_levels: bool,
    /// Acerca cada canle de cor á media (quita dominantes de cor).
    #[serde(default)]
    pub color_balance: bool,
    /// Levanta as sombras, de 0 a 100.
    #[serde(default)]
    pub shadows: u8,
    /// De -100 (gris) a 100 (cores moi vivas); 0 non cambia nada.
    #[serde(default)]
    pub saturation: i32,
    /// Quitar ruído: 0 nada, 1 suave, 2 normal, 3 forte.
    #[serde(default)]
    pub denoise: u8,
    /// Nitidez: 0 nada, 1 suave, 2 normal, 3 forte.
    #[serde(default)]
    pub sharpen: u8,
}

impl Default for RestoreOptions {
    fn default() -> Self {
        RestoreOptions { kind: Kind::Photo, auto_levels: false, color_balance: false, shadows: 0, saturation: 0, denoise: 0, sharpen: 0 }
    }
}

impl RestoreOptions {
    /// Non cambiaría nada.
    pub fn is_noop(&self) -> bool {
        *self == RestoreOptions::default()
    }

    pub fn validate(&self) -> Result<()> {
        if self.shadows > 100 || self.denoise > 3 || self.sharpen > 3 || !(-100..=100).contains(&self.saturation) {
            return Err(EngineError::IntegrityCheck("Algún axuste está fóra do seu rango.".into()));
        }
        Ok(())
    }
}

/// Máximo de píxeles que se tratan (por riba, a memoria dunha copia ou dúas sería de máis).
pub const MAX_PIXELS: u64 = 60_000_000;

fn luma(r: f32, g: f32, b: f32) -> f32 {
    0.299 * r + 0.587 * g + 0.114 * b
}

fn clamp(v: f32) -> u8 {
    v.clamp(0.0, 255.0).round() as u8
}

/// Desenfoque de caixa de 3×3 (separable) nas tres canles de cor; o alfa non se toca.
fn blur3(px: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut tmp = vec![0u8; px.len()];
    let mut out = vec![0u8; px.len()];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            let a = (y * w + x.saturating_sub(1)) * 4;
            let c = (y * w + (x + 1).min(w - 1)) * 4;
            for k in 0..3 {
                tmp[i + k] = ((px[a + k] as u16 + px[i + k] as u16 + px[c + k] as u16) / 3) as u8;
            }
            tmp[i + 3] = px[i + 3];
        }
    }
    for y in 0..h {
        let up = y.saturating_sub(1);
        let dn = (y + 1).min(h - 1);
        for x in 0..w {
            let i = (y * w + x) * 4;
            let a = (up * w + x) * 4;
            let c = (dn * w + x) * 4;
            for k in 0..3 {
                out[i + k] = ((tmp[a + k] as u16 + tmp[i + k] as u16 + tmp[c + k] as u16) / 3) as u8;
            }
            out[i + 3] = tmp[i + 3];
        }
    }
    out
}

/// Límites de luz (percentís 0,5 e 99,5) dunha lista de valores 0–255.
fn percentiles(hist: &[u32; 256], n: u64) -> (usize, usize) {
    let cut = (n as f64 * 0.005) as u64;
    let (mut lo, mut acc) = (0usize, 0u64);
    while lo < 255 {
        acc += hist[lo] as u64;
        if acc > cut {
            break;
        }
        lo += 1;
    }
    let (mut hi, mut acc) = (255usize, 0u64);
    while hi > 0 {
        acc += hist[hi] as u64;
        if acc > cut {
            break;
        }
        hi -= 1;
    }
    if hi < lo + 40 {
        hi = (lo + 40).min(255);
    }
    (lo, hi)
}

/// Niveis automáticos e equilibrio de cor.
fn levels_and_balance(px: &mut [u8], auto_levels: bool, balance: bool) {
    let n = (px.len() / 4) as u64;
    if n == 0 {
        return;
    }
    let mut hist = [0u32; 256];
    let (mut sr, mut sg, mut sb) = (0u64, 0u64, 0u64);
    for p in px.chunks_exact(4) {
        hist[luma(p[0] as f32, p[1] as f32, p[2] as f32).round() as usize] += 1;
        sr += p[0] as u64;
        sg += p[1] as u64;
        sb += p[2] as u64;
    }
    let (lo, hi) = if auto_levels { percentiles(&hist, n) } else { (0, 255) };
    let mean = (sr + sg + sb) as f32 / (3.0 * n as f32);
    let gain = |s: u64| if balance { (1.0 + (mean / (s as f32 / n as f32).max(1.0) - 1.0) * 0.5).clamp(0.88, 1.12) } else { 1.0 };
    let bal = [gain(sr), gain(sg), gain(sb)];
    let span = (hi - lo) as f32;
    let mut lut = [[0u8; 256]; 3];
    for (c, t) in lut.iter_mut().enumerate() {
        for (v, o) in t.iter_mut().enumerate() {
            *o = clamp(((v as f32 * bal[c] - lo as f32) / span) * 255.0);
        }
    }
    for p in px.chunks_exact_mut(4) {
        for c in 0..3 {
            p[c] = lut[c][p[c] as usize];
        }
    }
}

/// Levanta as sombras con unha curva gamma (de 1,0 a 0,6) sen queimar as luces.
fn shadows(px: &mut [u8], amount: u8) {
    let gamma = 1.0 - amount as f32 / 100.0 * 0.4;
    let mut lut = [0u8; 256];
    for (v, o) in lut.iter_mut().enumerate() {
        *o = clamp(255.0 * (v as f32 / 255.0).powf(gamma));
    }
    for p in px.chunks_exact_mut(4) {
        for c in 0..3 {
            p[c] = lut[p[c] as usize];
        }
    }
}

/// Saturación: afasta (ou achega) cada canle da súa luminosidade.
fn saturate(px: &mut [u8], amount: i32) {
    let k = 1.0 + amount as f32 / 100.0 * if amount > 0 { 0.8 } else { 1.0 };
    for p in px.chunks_exact_mut(4) {
        let l = luma(p[0] as f32, p[1] as f32, p[2] as f32);
        for c in 0..3 {
            p[c] = clamp(l + (p[c] as f32 - l) * k);
        }
    }
}

/// Pasa a gris e, se `flat`, iguala a iluminación dividindo pola luz de fondo: unha páxina fotografada con sombras queda branca e o texto negro.
fn gray(px: &mut [u8], w: usize, h: usize, flat: bool) {
    let g: Vec<f32> = px.chunks_exact(4).map(|p| luma(p[0] as f32, p[1] as f32, p[2] as f32)).collect();
    let out: Vec<f32> = if flat { divide_by_background(&g, w, h) } else { g };
    let (lo, hi) = if flat {
        (85.0, 225.0)
    } else {
        let mut hist = [0u32; 256];
        for v in &out {
            hist[v.round().clamp(0.0, 255.0) as usize] += 1;
        }
        let (l, h) = percentiles(&hist, out.len() as u64);
        (l as f32, h as f32)
    };
    for (p, v) in px.chunks_exact_mut(4).zip(out) {
        let o = clamp(((v - lo) / (hi - lo)) * 255.0);
        p[0] = o;
        p[1] = o;
        p[2] = o;
    }
}

/// A luz de fondo: media por bloques nunha imaxe pequena, o máximo local (o texto fino non conta como fondo) e un suavizado, ampliada con interpolación.
fn divide_by_background(g: &[f32], w: usize, h: usize) -> Vec<f32> {
    let f = (w.max(h) as f32 / 240.0).ceil().max(1.0) as usize;
    let (sw, sh) = (w.div_ceil(f), h.div_ceil(f));
    let mut small = vec![0f32; sw * sh];
    let mut cnt = vec![0u32; sw * sh];
    for y in 0..h {
        for x in 0..w {
            let i = (y / f) * sw + x / f;
            small[i] += g[y * w + x];
            cnt[i] += 1;
        }
    }
    for (s, c) in small.iter_mut().zip(&cnt) {
        *s /= (*c).max(1) as f32;
    }
    let neighbours = |s: &[f32], x: usize, y: usize, f: &mut dyn FnMut(f32)| {
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let yy = (y as i32 + dy).clamp(0, sh as i32 - 1) as usize;
                let xx = (x as i32 + dx).clamp(0, sw as i32 - 1) as usize;
                f(s[yy * sw + xx]);
            }
        }
    };
    for _ in 0..2 {
        let mut next = vec![0f32; small.len()];
        for y in 0..sh {
            for x in 0..sw {
                let mut m = 0f32;
                neighbours(&small, x, y, &mut |v| m = m.max(v));
                next[y * sw + x] = m;
            }
        }
        small = next;
    }
    for _ in 0..3 {
        let mut next = vec![0f32; small.len()];
        for y in 0..sh {
            for x in 0..sw {
                let mut s = 0f32;
                neighbours(&small, x, y, &mut |v| s += v);
                next[y * sw + x] = s / 9.0;
            }
        }
        small = next;
    }
    let mut out = vec![0f32; w * h];
    for y in 0..h {
        let fy = (y as f32 / f as f32 - 0.5).clamp(0.0, (sh - 1) as f32);
        let y0 = fy.floor() as usize;
        let y1 = (y0 + 1).min(sh - 1);
        let ty = fy - y0 as f32;
        for x in 0..w {
            let fx = (x as f32 / f as f32 - 0.5).clamp(0.0, (sw - 1) as f32);
            let x0 = fx.floor() as usize;
            let x1 = (x0 + 1).min(sw - 1);
            let tx = fx - x0 as f32;
            let top = small[y0 * sw + x0] * (1.0 - tx) + small[y0 * sw + x1] * tx;
            let bottom = small[y1 * sw + x0] * (1.0 - tx) + small[y1 * sw + x1] * tx;
            let bg = top * (1.0 - ty) + bottom * ty;
            out[y * w + x] = (g[y * w + x] * 255.0 / bg.max(24.0)).min(255.0);
        }
    }
    out
}

/// Quita ruído suavizando só onde a diferenza co contorno é pequena: as beiras e o detalle fino quedan.
fn denoise(px: &mut [u8], w: usize, h: usize, level: u8) {
    let (limit, passes) = match level {
        1 => (14.0, 1),
        2 => (26.0, 1),
        _ => (40.0, 2),
    };
    for _ in 0..passes {
        let soft = blur3(px, w, h);
        for (p, s) in px.chunks_exact_mut(4).zip(soft.chunks_exact(4)) {
            for c in 0..3 {
                let diff = (p[c] as f32 - s[c] as f32).abs();
                let wt = if diff >= limit { 0.0 } else { 1.0 - diff / limit };
                p[c] = clamp(p[c] as f32 + (s[c] as f32 - p[c] as f32) * wt);
            }
        }
    }
}

/// Máscara de enfoque: realza a diferenza co desenfoque, sen tocar as variacións mínimas (ruído).
fn sharpen(px: &mut [u8], w: usize, h: usize, level: u8) {
    let k = match level {
        1 => 0.5,
        2 => 0.9,
        _ => 1.5,
    };
    let soft = blur3(&blur3(px, w, h), w, h);
    for (p, s) in px.chunks_exact_mut(4).zip(soft.chunks_exact(4)) {
        for c in 0..3 {
            let diff = p[c] as f32 - s[c] as f32;
            if diff.abs() > 3.0 {
                p[c] = clamp(p[c] as f32 + diff * k);
            }
        }
    }
}

/// Aplica os axustes a unha imaxe (en RGBA; o alfa queda como está). Orde: ruído, luz e cor, tipo, nitidez.
pub fn apply(img: &DynamicImage, o: &RestoreOptions) -> Result<RgbaImage> {
    o.validate()?;
    let mut rgba = img.to_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    if w as u64 * h as u64 > MAX_PIXELS {
        return Err(EngineError::IntegrityCheck("A imaxe é demasiado grande para restaurala: non cabería na memoria.".into()));
    }
    let px: &mut [u8] = &mut rgba;
    if o.denoise > 0 {
        denoise(px, w, h, o.denoise);
    }
    match o.kind {
        Kind::Photo => {
            if o.auto_levels || o.color_balance {
                levels_and_balance(px, o.auto_levels, o.color_balance);
            }
            if o.shadows > 0 {
                shadows(px, o.shadows);
            }
            if o.saturation != 0 {
                saturate(px, o.saturation);
            }
        }
        Kind::Document => gray(px, w, h, true),
        Kind::Bw => {
            gray(px, w, h, false);
            if o.shadows > 0 {
                shadows(px, o.shadows);
            }
        }
    }
    if o.sharpen > 0 {
        sharpen(px, w, h, o.sharpen);
    }
    Ok(rgba)
}

/// Lado maior da vista previa.
pub const PREVIEW_SIDE: u32 = 1600;

#[derive(Debug, Clone, Serialize)]
pub struct RestoreReport {
    pub output_path: std::path::PathBuf,
    pub size: (u32, u32),
    pub bytes_written: u64,
}

fn check_input(input: &Path) -> Result<()> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    if crate::animated::is_animated(input) {
        return Err(EngineError::IntegrityCheck("Esta imaxe é animada: restaurala só gardaría un fotograma, así que non o fago.".into()));
    }
    Ok(())
}

/// Vista previa: a imaxe restaurada, en pequeno, nun PNG en `output`.
pub fn preview(input: &Path, output: &Path, o: &RestoreOptions) -> Result<std::path::PathBuf> {
    check_input(input)?;
    o.validate()?;
    let (img, _) = crate::image_meta::open_oriented(input, crate::image_meta::Metadata::Strip)?;
    let longest = img.width().max(img.height());
    let small = if longest > PREVIEW_SIDE {
        let f = PREVIEW_SIDE as f32 / longest as f32;
        img.resize_exact(((img.width() as f32 * f).round() as u32).max(1), ((img.height() as f32 * f).round() as u32).max(1), image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let result = apply(&small, o)?;
    if let Some(dir) = output.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = output.with_extension("tmp");
    {
        let file = std::fs::File::create(&tmp)?;
        let enc = image::codecs::png::PngEncoder::new_with_quality(std::io::BufWriter::new(file), image::codecs::png::CompressionType::Fast, image::codecs::png::FilterType::Sub);
        use image::ImageEncoder;
        enc.write_image(result.as_raw(), result.width(), result.height(), image::ExtendedColorType::Rgba8).map_err(|e| EngineError::IntegrityCheck(e.to_string()))?;
    }
    std::fs::rename(&tmp, output)?;
    Ok(output.to_path_buf())
}

/// Restaura `input` e gárdaa en `output` (o formato sae da súa extensión). Nunca toca o orixinal.
pub fn restore_image(input: &Path, output: &Path, overwrite: bool, o: &RestoreOptions, quality: Option<u8>, background: Option<[u8; 3]>, metadata: crate::image_meta::Metadata) -> Result<RestoreReport> {
    check_input(input)?;
    o.validate()?;
    crate::integrity::ensure_distinct(&[input], output)?;
    let opts = crate::image_convert::ImageConvertOptions { quality, background, metadata, restore: Some(o.clone()), ..Default::default() };
    let done = crate::image_convert::convert_image(input, output, overwrite, &opts)?;
    let size = crate::image_convert::image_dimensions(&done)?;
    let bytes_written = std::fs::metadata(&done)?.len();
    Ok(RestoreReport { output_path: done, size, bytes_written })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn flat(w: u32, h: u32, c: [u8; 4]) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba(c)))
    }

    #[test]
    fn sen_axustes_non_cambia_nada() {
        let img = flat(8, 8, [10, 120, 200, 255]);
        let out = apply(&img, &RestoreOptions::default()).unwrap();
        assert_eq!(out, img.to_rgba8());
        assert!(RestoreOptions::default().is_noop());
    }

    #[test]
    fn os_niveis_estiran_unha_imaxe_apagada() {
        let mut im = RgbaImage::from_pixel(10, 10, Rgba([100, 100, 100, 255]));
        for x in 0..5 {
            for y in 0..10 {
                im.put_pixel(x, y, Rgba([140, 140, 140, 255]));
            }
        }
        let o = RestoreOptions { auto_levels: true, ..Default::default() };
        let out = apply(&DynamicImage::ImageRgba8(im), &o).unwrap();
        assert!(out.get_pixel(0, 0)[0] > 240, "o claro vai a branco");
        assert!(out.get_pixel(9, 0)[0] < 20, "o escuro vai a negro");
    }

    #[test]
    fn as_sombras_levantan_o_escuro_e_o_alfa_queda() {
        let img = flat(4, 4, [30, 30, 30, 128]);
        let out = apply(&img, &RestoreOptions { shadows: 100, ..Default::default() }).unwrap();
        assert!(out.get_pixel(0, 0)[0] > 30);
        assert_eq!(out.get_pixel(0, 0)[3], 128);
    }

    #[test]
    fn o_equilibrio_de_cor_acerca_as_canles() {
        let img = flat(4, 4, [200, 100, 100, 255]);
        let out = apply(&img, &RestoreOptions { color_balance: true, ..Default::default() }).unwrap();
        let p = out.get_pixel(0, 0);
        assert!(p[0] < 200 && p[1] > 100, "o vermello baixa e o resto sobe: {p:?}");
    }

    #[test]
    fn documento_deixa_o_papel_branco_aínda_con_sombra() {
        // Páxina cun degradado de luz (de 120 a 220) e unha liña negra.
        let mut im = RgbaImage::from_fn(200, 100, |x, _| {
            let v = (120 + x * 100 / 199) as u8;
            Rgba([v, v, v, 255])
        });
        for x in 20..180 {
            im.put_pixel(x, 50, Rgba([20, 20, 20, 255]));
        }
        let out = apply(&DynamicImage::ImageRgba8(im), &RestoreOptions { kind: Kind::Document, ..Default::default() }).unwrap();
        assert!(out.get_pixel(10, 10)[0] > 235 && out.get_pixel(190, 10)[0] > 235, "o papel queda branco nos dous extremos");
        assert!(out.get_pixel(100, 50)[0] < 60, "o texto queda escuro");
    }

    #[test]
    fn menos_ruido_suaviza_o_gran_pero_non_a_beira() {
        let img = RgbaImage::from_fn(40, 40, |x, y| {
            let base = if x < 20 { 40i32 } else { 210 };
            let n = if (x + y) % 2 == 0 { 6 } else { -6 };
            let v = (base + n) as u8;
            Rgba([v, v, v, 255])
        });
        let o = RestoreOptions { denoise: 2, ..Default::default() };
        let out = apply(&DynamicImage::ImageRgba8(img), &o).unwrap();
        let (a, b) = (out.get_pixel(5, 5)[0] as i32, out.get_pixel(5, 6)[0] as i32);
        assert!((a - b).abs() < 6, "o gran baixa");
        assert!(out.get_pixel(5, 5)[0] < 60 && out.get_pixel(35, 5)[0] > 190, "a beira queda");
    }

    #[test]
    fn a_nitidez_aumenta_o_contraste_dunha_beira() {
        let img = RgbaImage::from_fn(20, 20, |x, _| if x < 10 { Rgba([100, 100, 100, 255]) } else { Rgba([150, 150, 150, 255]) });
        let out = apply(&DynamicImage::ImageRgba8(img), &RestoreOptions { sharpen: 3, ..Default::default() }).unwrap();
        assert!(out.get_pixel(9, 5)[0] < 100 && out.get_pixel(10, 5)[0] > 150);
    }

    #[test]
    fn rexeita_valores_fora_de_rango() {
        assert!(RestoreOptions { denoise: 4, ..Default::default() }.validate().is_err());
        assert!(RestoreOptions { saturation: 101, ..Default::default() }.validate().is_err());
        assert!(RestoreOptions { shadows: 101, ..Default::default() }.validate().is_err());
    }

    #[test]
    fn restaura_un_ficheiro_e_non_toca_o_orixinal() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.png");
        RgbaImage::from_fn(30, 20, |x, _| if x < 15 { Rgba([90, 90, 90, 255]) } else { Rgba([130, 130, 130, 255]) }).save(&input).unwrap();
        let before = std::fs::read(&input).unwrap();
        let out = dir.path().join("a-restaurado.png");
        let o = RestoreOptions { auto_levels: true, ..Default::default() };
        let r = restore_image(&input, &out, false, &o, None, None, crate::image_meta::Metadata::Strip).unwrap();
        assert_eq!(r.size, (30, 20));
        assert!(image::open(&out).unwrap().to_rgba8().get_pixel(0, 0)[0] < 20);
        assert_eq!(std::fs::read(&input).unwrap(), before);
        assert!(restore_image(&input, &out, false, &o, None, None, crate::image_meta::Metadata::Strip).is_err(), "non pisa un ficheiro que xa existe");
        let p = preview(&input, &dir.path().join("p.png"), &o).unwrap();
        assert!(p.is_file());
    }
}
