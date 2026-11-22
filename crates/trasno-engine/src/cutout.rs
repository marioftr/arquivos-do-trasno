//! Quitar o fondo dunha imaxe (extensión opcional: o modelo IS-Net «general use» do proxecto DIS, en ONNX, descárgase a petición).
//!
//! O modelo corre no propio equipo con `tract` (inferencia en Rust puro, sen DLL nin servizos): a imaxe redimensiónase a 1024 × 1024,
//! o modelo devolve unha máscara (canto de «obxecto» é cada punto) e esa máscara escálase ao tamaño orixinal e úsase de canle alfa.
//! O resultado vai sempre a un ficheiro novo, en PNG ou WebP con transparencia ou, se se pide un fondo de cor, tamén en JPG.

use crate::error::{EngineError, Result};
use crate::ffmpeg;
use crate::image_convert::write_verified;
use crate::integrity::ensure_distinct;
use crate::tools_install;
use image::{DynamicImage, GrayImage, ImageFormat, Rgba, RgbaImage};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tract_onnx::prelude::*;

/// Lado do cadrado que espera o modelo.
const SIDE: usize = 1024;
/// Máximo de píxeles da imaxe de entrada (a máscara escálase ao seu tamaño).
const MAX_PIXELS: u64 = 120_000_000;

/// Axustes do recorte. Todos van de 0 a 100.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CutoutOptions {
    /// Cor do fondo novo. Sen ela, o fondo queda transparente.
    pub background: Option<[u8; 3]>,
    /// Onde se corta: máis alto quita máis (o obxecto encolle); máis baixo deixa máis (o obxecto medra).
    pub edge: u8,
    /// Suavidade da beira: 0 corta seco, 100 deixa a transición que dá o modelo.
    pub soft: u8,
    /// Detalle das beiras: axusta a máscara ás liñas da imaxe (pelo, ramas…). 0 non a axusta.
    pub detail: u8,
}

impl Default for CutoutOptions {
    fn default() -> Self {
        CutoutOptions { background: None, edge: 50, soft: 70, detail: 60 }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CutoutReport {
    pub output_path: PathBuf,
    pub size: (u32, u32),
    /// Que parte da imaxe quedou (0 a 1): o resto é fondo.
    pub kept: f64,
    pub bytes_written: u64,
}

fn fails(detail: impl std::fmt::Display) -> EngineError {
    EngineError::ExternalFailed { program: "O modelo de imaxe", detail: detail.to_string() }
}

/// Máscara que devolve o modelo (1024 × 1024; 0 = fondo, 255 = obxecto).
fn infer(model: &Path, rgb: &image::RgbImage) -> Result<GrayImage> {
    let plan = tract_onnx::onnx()
        .model_for_path(model)
        .and_then(|m| m.with_input_fact(0, f32::fact([1, 3, SIDE, SIDE]).into()))
        .and_then(|m| m.into_optimized())
        .and_then(|m| m.into_runnable())
        .map_err(fails)?;
    ffmpeg::report_progress(0.3);
    if ffmpeg::cancelled_here() {
        return Err(EngineError::Cancelled);
    }

    let small = image::imageops::resize(rgb, SIDE as u32, SIDE as u32, image::imageops::FilterType::Lanczos3);
    let mean = [0.485f32, 0.456, 0.406];
    let input: Tensor = tract_ndarray::Array4::from_shape_fn((1, 3, SIDE, SIDE), |(_, c, y, x)| small.get_pixel(x as u32, y as u32)[c] as f32 / 255.0 - mean[c]).into();

    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(1, 8);
    let run = |input: Tensor| plan.run(tvec!(input.into()));
    let out = if threads > 1 {
        tract_linalg::multithread::multithread_tract_scope(tract_linalg::multithread::Executor::multithread(threads), || run(input))
    } else {
        run(input)
    }
    .map_err(fails)?;
    if ffmpeg::cancelled_here() {
        return Err(EngineError::Cancelled);
    }
    ffmpeg::report_progress(0.85);

    // A primeira saída é a máscara final (as outras son pasos intermedios).
    let view = out[0].to_array_view::<f32>().map_err(fails)?;
    let values: Vec<f32> = view.iter().copied().collect();
    if values.len() != SIDE * SIDE {
        return Err(fails("a saída do modelo non ten o tamaño esperado"));
    }
    let (lo, hi) = values.iter().fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    let span = (hi - lo).max(1e-6);
    let mut mask = GrayImage::new(SIDE as u32, SIDE as u32);
    for (i, v) in values.iter().enumerate() {
        mask.put_pixel((i % SIDE) as u32, (i / SIDE) as u32, image::Luma([(((v - lo) / span) * 255.0).round().clamp(0.0, 255.0) as u8]));
    }
    Ok(mask)
}

// ---------------------------------------------------------------- memoria da última imaxe analizada

/// O que queda en memoria da última imaxe analizada: a máscara do modelo e unha copia pequena para a vista previa.
/// Así mover os axustes non repite o traballo do modelo (que é o que tarda).
struct Analysis {
    key: (PathBuf, u64, u64),
    raw: Arc<GrayImage>,
    small: Arc<RgbaImage>,
    size: (u32, u32),
}

static LAST: Mutex<Option<Analysis>> = Mutex::new(None);
/// Lado maior da vista previa.
const PREVIEW_SIDE: u32 = 1400;

fn key_of(input: &Path) -> Result<(PathBuf, u64, u64)> {
    let meta = std::fs::metadata(input)?;
    let mtime = meta.modified().ok().and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos() as u64).unwrap_or(0);
    Ok((input.to_path_buf(), mtime, meta.len()))
}

fn check_input(input: &Path) -> Result<()> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    if crate::animated::is_animated(input) {
        return Err(EngineError::IntegrityCheck("Esta imaxe é animada: só se tratería un fotograma, así que non o fago.".into()));
    }
    let (w, h) = crate::resize::oriented_dimensions(input)?;
    if w as u64 * h as u64 > MAX_PIXELS {
        return Err(EngineError::IntegrityCheck("A imaxe é demasiado grande para quitarlle o fondo: non cabería na memoria.".into()));
    }
    Ok(())
}

/// A máscara do modelo para `input` (a de memoria se xa se analizou esta mesma imaxe).
fn analysis_of(input: &Path) -> Result<(Arc<GrayImage>, Arc<RgbaImage>, (u32, u32))> {
    let key = key_of(input)?;
    if let Some(a) = LAST.lock().unwrap().as_ref().filter(|a| a.key == key) {
        return Ok((a.raw.clone(), a.small.clone(), a.size));
    }
    check_input(input)?;
    let model = tools_install::model_path("cutout").ok_or(EngineError::ExtensionMissing("cutout-model"))?;
    ffmpeg::report_progress(0.05);
    let (img, _) = crate::image_meta::open_oriented(input, crate::image_meta::Metadata::Strip)?;
    ffmpeg::report_progress(0.1);
    let raw = Arc::new(infer(&model, &img.to_rgb8())?);
    let size = (img.width(), img.height());
    let longest = img.width().max(img.height());
    let small = if longest > PREVIEW_SIDE {
        let f = PREVIEW_SIDE as f32 / longest as f32;
        img.resize_exact(((img.width() as f32 * f).round() as u32).max(1), ((img.height() as f32 * f).round() as u32).max(1), image::imageops::FilterType::Triangle).to_rgba8()
    } else {
        img.to_rgba8()
    };
    let small = Arc::new(small);
    *LAST.lock().unwrap() = Some(Analysis { key, raw: raw.clone(), small: small.clone(), size });
    Ok((raw, small, size))
}

/// Analiza a imaxe co modelo (a primeira vez tarda) e deixa a máscara en memoria para a vista previa e o resultado.
/// Devolve o tamaño da imaxe.
pub fn analyze(input: &Path) -> Result<(u32, u32)> {
    analysis_of(input).map(|a| a.2)
}

/// Esquece a imaxe analizada (ao pechar a ferramenta).
pub fn forget() {
    *LAST.lock().unwrap() = None;
}

// ---------------------------------------------------------------- axuste da máscara

/// Beira: de máscara do modelo a opacidade final segundo `edge` e `soft`.
fn edge_curve(edge: u8, soft: u8) -> impl Fn(u8) -> u8 {
    let t = edge.min(100) as f32 / 100.0;
    let s = 0.06 + soft.min(100) as f32 / 100.0 * 0.94;
    let lo = t - s / 2.0;
    let lut: Vec<u8> = (0..256).map(|v| (((v as f32 / 255.0 - lo) / s).clamp(0.0, 1.0) * 255.0).round() as u8).collect();
    move |v| lut[v as usize]
}

/// Media sobre unha ventá cadrada de radio `r` (a ventá recórtase nas beiras).
fn box_mean(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut tmp = vec![0f32; w * h];
    let mut pre = vec![0f64; w.max(h) + 1];
    for y in 0..h {
        for x in 0..w {
            pre[x + 1] = pre[x] + src[y * w + x] as f64;
        }
        for x in 0..w {
            let (a, b) = (x.saturating_sub(r), (x + r + 1).min(w));
            tmp[y * w + x] = ((pre[b] - pre[a]) / (b - a) as f64) as f32;
        }
    }
    let mut out = vec![0f32; w * h];
    for x in 0..w {
        for y in 0..h {
            pre[y + 1] = pre[y] + tmp[y * w + x] as f64;
        }
        for y in 0..h {
            let (a, b) = (y.saturating_sub(r), (y + r + 1).min(h));
            out[y * w + x] = ((pre[b] - pre[a]) / (b - a) as f64) as f32;
        }
    }
    out
}

/// Reduce un plano `f` veces (media de cada bloque).
fn shrink(src: impl Fn(usize, usize) -> f32, w: usize, h: usize, f: usize) -> (Vec<f32>, usize, usize) {
    let (lw, lh) = (w.div_ceil(f), h.div_ceil(f));
    let mut out = vec![0f32; lw * lh];
    for ly in 0..lh {
        for lx in 0..lw {
            let (mut sum, mut n) = (0f32, 0f32);
            for y in ly * f..((ly + 1) * f).min(h) {
                for x in lx * f..((lx + 1) * f).min(w) {
                    sum += src(x, y);
                    n += 1.0;
                }
            }
            out[ly * lw + lx] = sum / n;
        }
    }
    (out, lw, lh)
}

/// Filtro guiado (He, Sun e Tang): axusta a máscara `mask` ás beiras da imaxe `guide` (a súa luminosidade).
/// Os coeficientes calcúlanse nunha versión reducida e aplícanse á imaxe enteira, así custa case o mesmo en calquera tamaño.
fn guided(guide: &RgbaImage, mask: &GrayImage, radius: usize, eps: f32) -> GrayImage {
    let (w, h) = (guide.width() as usize, guide.height() as usize);
    let luma = |x: usize, y: usize| {
        let p = guide.get_pixel(x as u32, y as u32);
        ((p[0] as u32 * 54 + p[1] as u32 * 183 + p[2] as u32 * 19) >> 8) as u8
    };
    let luma_f = |x: usize, y: usize| luma(x, y) as f32 / 255.0;
    let f = ((w * h) as f64 / 1_500_000.0).sqrt().ceil().max(1.0) as usize;
    let (i_s, lw, lh) = shrink(luma_f, w, h, f);
    let (p_s, _, _) = shrink(|x, y| mask.get_pixel(x as u32, y as u32)[0] as f32 / 255.0, w, h, f);
    let r = (radius / f).max(1);
    let mean_i = box_mean(&i_s, lw, lh, r);
    let mean_p = box_mean(&p_s, lw, lh, r);
    let ii: Vec<f32> = i_s.iter().map(|v| v * v).collect();
    let ip: Vec<f32> = i_s.iter().zip(&p_s).map(|(a, b)| a * b).collect();
    let corr_i = box_mean(&ii, lw, lh, r);
    let corr_ip = box_mean(&ip, lw, lh, r);
    let n = lw * lh;
    let (mut a, mut b) = (vec![0f32; n], vec![0f32; n]);
    for k in 0..n {
        let var = corr_i[k] - mean_i[k] * mean_i[k];
        let cov = corr_ip[k] - mean_i[k] * mean_p[k];
        a[k] = cov / (var.max(0.0) + eps);
        b[k] = mean_p[k] - a[k] * mean_i[k];
    }
    let (a, b) = (box_mean(&a, lw, lh, r), box_mean(&b, lw, lh, r));
    // Interpolación bilineal dos coeficientes ata o tamaño enteiro.
    let at = |g: &[f32], fx: f32, fy: f32| {
        let (x0, y0) = (fx.floor().max(0.0) as usize, fy.floor().max(0.0) as usize);
        let (x1, y1) = ((x0 + 1).min(lw - 1), (y0 + 1).min(lh - 1));
        let (x0, y0) = (x0.min(lw - 1), y0.min(lh - 1));
        let (tx, ty) = ((fx - x0 as f32).clamp(0.0, 1.0), (fy - y0 as f32).clamp(0.0, 1.0));
        let top = g[y0 * lw + x0] * (1.0 - tx) + g[y0 * lw + x1] * tx;
        let bot = g[y1 * lw + x0] * (1.0 - tx) + g[y1 * lw + x1] * tx;
        top * (1.0 - ty) + bot * ty
    };
    let mut out = GrayImage::new(w as u32, h as u32);
    for y in 0..h {
        let fy = (y as f32 + 0.5) / f as f32 - 0.5;
        for x in 0..w {
            let fx = (x as f32 + 0.5) / f as f32 - 0.5;
            let i = luma(x, y) as f32 / 255.0;
            let q = at(&a, fx, fy) * i + at(&b, fx, fy);
            out.put_pixel(x as u32, y as u32, image::Luma([(q.clamp(0.0, 1.0) * 255.0).round() as u8]));
        }
    }
    out
}

/// Opacidade final de cada punto de `img`: a máscara do modelo (escalada), axustada ás beiras e coa curva de `o`.
fn alpha_for(img: &RgbaImage, raw: &GrayImage, o: &CutoutOptions) -> GrayImage {
    let (w, h) = img.dimensions();
    let mut mask = image::imageops::resize(raw, w, h, image::imageops::FilterType::CatmullRom);
    let d = o.detail.min(100) as f32 / 100.0;
    if d > 0.0 {
        let radius = edge_radius(w, h);
        let eps = 0.0004 + (1.0 - d) * (1.0 - d) * 0.02;
        let fine = guided(img, &mask, radius, eps);
        let mix = (d * 1.6).min(1.0);
        for (m, f) in mask.pixels_mut().zip(fine.pixels()) {
            m[0] = (m[0] as f32 * (1.0 - mix) + f[0] as f32 * mix).round() as u8;
        }
    }
    let curve = edge_curve(o.edge, o.soft);
    for m in mask.pixels_mut() {
        m[0] = curve(m[0]);
    }
    mask
}

/// Limpa a cor das beiras: onde o recorte é parcial, os puntos conservan algo da cor do fondo vello (o halo escuro ou claro arredor do pelo).
/// Mestúrase coa cor que ten o obxecto un pouco máis adentro, estimada con medias ponderadas pola opacidade.
fn clean_edges(img: &RgbaImage, alpha: &GrayImage, radius: usize) -> RgbaImage {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let weight = |x: usize, y: usize| {
        let a = alpha.get_pixel(x as u32, y as u32)[0] as f32 / 255.0;
        a * a * a * a
    };
    let f = ((w * h) as f64 / 1_500_000.0).sqrt().ceil().max(1.0) as usize;
    let r = (radius / f).max(1);
    let (den, lw, lh) = shrink(weight, w, h, f);
    let den = box_mean(&den, lw, lh, r);
    let num: Vec<Vec<f32>> = (0..3)
        .map(|c| {
            let (m, _, _) = shrink(|x, y| weight(x, y) * img.get_pixel(x as u32, y as u32)[c] as f32 / 255.0, w, h, f);
            box_mean(&m, lw, lh, r)
        })
        .collect();
    let at = |g: &[f32], fx: f32, fy: f32| {
        let (x0, y0) = (fx.floor().max(0.0) as usize, fy.floor().max(0.0) as usize);
        let (x1, y1) = ((x0 + 1).min(lw - 1), (y0 + 1).min(lh - 1));
        let (x0, y0) = (x0.min(lw - 1), y0.min(lh - 1));
        let (tx, ty) = ((fx - x0 as f32).clamp(0.0, 1.0), (fy - y0 as f32).clamp(0.0, 1.0));
        let top = g[y0 * lw + x0] * (1.0 - tx) + g[y0 * lw + x1] * tx;
        let bot = g[y1 * lw + x0] * (1.0 - tx) + g[y1 * lw + x1] * tx;
        top * (1.0 - ty) + bot * ty
    };
    let mut out = img.clone();
    for y in 0..h {
        let fy = (y as f32 + 0.5) / f as f32 - 0.5;
        for x in 0..w {
            let a = alpha.get_pixel(x as u32, y as u32)[0] as f32 / 255.0;
            if a >= 0.98 || a <= 0.0 {
                continue;
            }
            let fx = (x as f32 + 0.5) / f as f32 - 0.5;
            let d = at(&den, fx, fy);
            if d < 0.02 {
                continue;
            }
            let mix = (1.0 - a * a).clamp(0.0, 1.0);
            let p = out.get_pixel_mut(x as u32, y as u32);
            for c in 0..3 {
                let fg = (at(&num[c], fx, fy) / d).clamp(0.0, 1.0) * 255.0;
                p[c] = (p[c] as f32 * (1.0 - mix) + fg * mix).round() as u8;
            }
        }
    }
    out
}

/// Radio (en puntos da imaxe) co que se axustan as beiras e se limpa a súa cor.
fn edge_radius(w: u32, h: u32) -> usize {
    ((w.max(h) as f32 * 0.004).round() as usize).max(2)
}

/// Pon a máscara de canle alfa (ou mestura co fondo de cor se se pediu).
fn apply_mask(img: &DynamicImage, mask: &GrayImage, background: Option<[u8; 3]>) -> RgbaImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = rgba.get_pixel(x, y);
            // Se a imaxe xa tiña transparencia, non se lle devolve opacidade a nada.
            let a = (mask.get_pixel(x, y)[0] as u16 * p[3] as u16 / 255) as u8;
            out.put_pixel(
                x,
                y,
                match background {
                    None => Rgba([p[0], p[1], p[2], a]),
                    Some(c) => {
                        let mix = |f: u8, b: u8| ((f as u16 * a as u16 + b as u16 * (255 - a as u16) + 127) / 255) as u8;
                        Rgba([mix(p[0], c[0]), mix(p[1], c[1]), mix(p[2], c[2]), 255])
                    }
                },
            );
        }
    }
    out
}

/// Vista previa: a imaxe sen fondo (transparente) en pequeno, nun PNG en `output`. O fondo de cor ponse por detrás na interface.
pub fn preview(input: &Path, output: &Path, o: &CutoutOptions) -> Result<PathBuf> {
    let (raw, small, _) = analysis_of(input)?;
    let alpha = alpha_for(&small, &raw, o);
    let clean = clean_edges(&small, &alpha, edge_radius(small.width(), small.height()) * 2);
    let result = apply_mask(&DynamicImage::ImageRgba8(clean), &alpha, None);
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

/// Quita o fondo de `input` e garda o resultado en `output` (o formato sae da súa extensión: PNG, WebP ou, con fondo de cor, JPG).
pub fn remove_background(input: &Path, output: &Path, overwrite: bool, o: &CutoutOptions) -> Result<CutoutReport> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    ensure_distinct(&[input], output)?;
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    let ext = crate::formats::extension_of(output).unwrap_or_default();
    let format = match ext.as_str() {
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::WebP,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        _ => return Err(EngineError::IntegrityCheck("O resultado gárdase en PNG, WebP ou JPG.".into())),
    };
    if format == ImageFormat::Jpeg && o.background.is_none() {
        return Err(EngineError::IntegrityCheck("O JPG non admite transparencia: escolle PNG ou WebP, ou pon un fondo de cor.".into()));
    }
    check_input(input)?;
    if tools_install::model_path("cutout").is_none() {
        return Err(EngineError::ExtensionMissing("cutout-model"));
    }
    let (raw, _, _) = analysis_of(input)?;
    ffmpeg::report_progress(0.88);
    let (img, _) = crate::image_meta::open_oriented(input, crate::image_meta::Metadata::Strip)?;
    let rgba = img.to_rgba8();
    let mask = alpha_for(&rgba, &raw, o);
    let kept = mask.pixels().map(|p| p[0] as u64).sum::<u64>() as f64 / (mask.width() as u64 * mask.height() as u64 * 255) as f64;
    let clean = clean_edges(&rgba, &mask, edge_radius(rgba.width(), rgba.height()) * 2);
    drop(rgba);
    let result = DynamicImage::ImageRgba8(apply_mask(&DynamicImage::ImageRgba8(clean), &mask, o.background));
    let quality = (format == ImageFormat::Jpeg).then_some(92u8);
    let output_path = write_verified(&result, output, overwrite, format, quality, o.background)?;
    ffmpeg::report_progress(1.0);
    let bytes_written = std::fs::metadata(&output_path)?.len();
    Ok(CutoutReport { output_path, size: (result.width(), result.height()), kept, bytes_written })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mascara_vira_canle_alfa_e_respecta_a_transparencia_previa() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(2, 1, |x, _| if x == 0 { Rgba([200, 10, 10, 255]) } else { Rgba([10, 200, 10, 100]) }));
        let mut mask = GrayImage::new(2, 1);
        mask.put_pixel(0, 0, image::Luma([255]));
        mask.put_pixel(1, 0, image::Luma([255]));
        let out = apply_mask(&img, &mask, None);
        assert_eq!(out.get_pixel(0, 0).0, [200, 10, 10, 255]);
        assert_eq!(out.get_pixel(1, 0).0[3], 100, "unha imaxe xa transparente non gaña opacidade");
        let mut half = mask.clone();
        half.put_pixel(0, 0, image::Luma([0]));
        assert_eq!(apply_mask(&img, &half, None).get_pixel(0, 0).0[3], 0);
    }

    #[test]
    fn co_fondo_de_cor_mestura_e_queda_opaca() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([100, 100, 100, 255])));
        let mut mask = GrayImage::new(1, 1);
        mask.put_pixel(0, 0, image::Luma([0]));
        let out = apply_mask(&img, &mask, Some([255, 255, 255]));
        assert_eq!(out.get_pixel(0, 0).0, [255, 255, 255, 255]);
        mask.put_pixel(0, 0, image::Luma([255]));
        assert_eq!(apply_mask(&img, &mask, Some([255, 255, 255])).get_pixel(0, 0).0, [100, 100, 100, 255]);
    }

    #[test]
    fn rexeita_jpg_sen_fondo_e_avisa_se_falta_o_modelo() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.png");
        image::RgbImage::from_pixel(4, 4, image::Rgb([1, 2, 3])).save(&input).unwrap();
        assert!(remove_background(&input, &dir.path().join("a.jpg"), false, &CutoutOptions::default()).is_err());
        assert!(remove_background(&input, &dir.path().join("a.gif"), false, &CutoutOptions::default()).is_err());
        if tools_install::model_path("cutout").is_none() {
            assert!(matches!(remove_background(&input, &dir.path().join("o.png"), false, &CutoutOptions::default()), Err(EngineError::ExtensionMissing("cutout-model"))));
        }
    }

    #[test]
    fn a_curva_da_beira_respecta_o_corte_e_a_suavidade() {
        let id = edge_curve(50, 100);
        assert!(id(0) == 0 && id(255) == 255 && (id(128) as i32 - 128).abs() <= 1, "co valor neutro non cambia a máscara");
        let hard = edge_curve(50, 0);
        assert!(hard(100) == 0 && hard(160) == 255, "sen suavidade corta seco");
        let more = edge_curve(80, 100);
        let less = edge_curve(20, 100);
        assert!(more(128) < id(128) && less(128) > id(128), "máis corte quita máis e menos deixa máis");
    }

    #[test]
    fn o_filtro_guiado_leva_a_beira_da_mascara_ata_a_beira_da_imaxe() {
        // Imaxe: metade esquerda branca, dereita negra, beira en x = 32. Máscara borrosa arredor de x = 36.
        let img = RgbaImage::from_fn(64, 16, |x, _| if x < 32 { Rgba([255, 255, 255, 255]) } else { Rgba([0, 0, 0, 255]) });
        let mask = GrayImage::from_fn(64, 16, |x, _| image::Luma([(255.0 - ((x as f32 - 28.0) / 16.0).clamp(0.0, 1.0) * 255.0) as u8]));
        let out = guided(&img, &mask, 6, 0.0005);
        let jump = |g: &GrayImage| g.get_pixel(31, 8)[0] as i32 - g.get_pixel(32, 8)[0] as i32;
        assert!(jump(&mask) < 25, "a máscara orixinal non ten beira en x = 32 ({})", jump(&mask));
        assert!(jump(&out) > 60, "o resultado ten a beira onde a ten a imaxe ({})", jump(&out));
    }

    #[test]
    fn a_media_da_ventana_e_a_esperada() {
        let v: Vec<f32> = (0..16).map(|i| i as f32).collect();
        let m = box_mean(&v, 4, 4, 1);
        assert!((m[5] - 5.0).abs() < 1e-4, "a media dos nove veciños do punto central é el mesmo ({})", m[5]);
        assert!((m[0] - 2.5).abs() < 1e-4);
    }
}
