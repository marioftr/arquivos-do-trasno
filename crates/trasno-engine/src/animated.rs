//! Imaxes animadas (GIF, WebP e PNG animados). A crate `image` só le o primeiro fotograma, así que calquera ferramenta que
//! pase por ela deixaría unha imaxe fixa. Aquí detéctase se unha imaxe é animada e, para o GIF, trabállase con FFmpeg
//! fotograma a fotograma (anotar, comprimir, pasar a WebP animado), conservando os tempos de cada cadro.

use crate::error::{EngineError, Result};
use crate::ffmpeg;
use image::AnimationDecoder;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// A imaxe ten máis dun fotograma? Só mira GIF, WebP e PNG (APNG); calquera outro formato devolve `false`.
pub fn is_animated(path: &Path) -> bool {
    let ext = crate::formats::extension_of(path).unwrap_or_default();
    let Ok(file) = std::fs::File::open(path) else { return false };
    let reader = BufReader::new(file);
    match ext.as_str() {
        "gif" => image::codecs::gif::GifDecoder::new(reader)
            .map(|d| d.into_frames().take(2).filter(|f| f.is_ok()).count() > 1)
            .unwrap_or(false),
        "webp" => image::codecs::webp::WebPDecoder::new(reader).map(|d| d.has_animation()).unwrap_or(false),
        "png" | "apng" => image::codecs::png::PngDecoder::new(reader).and_then(|d| d.is_apng()).unwrap_or(false),
        _ => false,
    }
}

/// Número de colores da paleta para esta calidade (100 → 256).
fn colors_for(quality: u8) -> u32 {
    crate::compress::palette_colors(quality) as u32
}

/// Redución opcional do lado maior (nunca amplía).
fn scale_part(max_side: Option<u32>) -> String {
    match max_side.filter(|m| *m > 0) {
        Some(m) => format!("scale=w=min(iw\\,{m}):h=min(ih\\,{m}):force_original_aspect_ratio=decrease:flags=lanczos,"),
        None => String::new(),
    }
}

/// Tramo final dun filtro: crea a paleta e aplícaa. `animated` axuda a que só se garde o que cambia entre cadros.
fn palette_part(colors: u32, animated: bool) -> String {
    let (stats, diff) = if animated { ("stats_mode=diff:", ":diff_mode=rectangle") } else { ("", "") };
    let dither = if colors >= 128 { "sierra2_4a" } else { "bayer:bayer_scale=3" };
    format!("split[a][b];[a]palettegen=max_colors={colors}:{stats}reserve_transparent=1[p];[b][p]paletteuse=dither={dither}:alpha_threshold=128{diff}")
}

fn gif_args(filter: String) -> Vec<String> {
    ["-filter_complex".to_string(), filter, "-fps_mode".into(), "vfr".into(), "-loop".into(), "0".into()].to_vec()
}

/// Comproba que o resultado segue sendo animado se o orixinal o era (se non, descártase).
fn check_still_animated(input: &Path, out: &Path) -> Result<()> {
    if is_animated(input) && !is_animated(out) {
        let _ = std::fs::remove_file(out);
        return Err(EngineError::IntegrityCheck("O resultado perdeu a animación; descarteino para non deixarche un ficheiro roto.".into()));
    }
    Ok(())
}

/// Superpón `overlay` (un PNG con transparencia, do mesmo tamaño que o GIF) en todos os fotogramas do GIF.
pub fn annotate_gif(input: &Path, overlay: &Path, output: &Path, overwrite: bool) -> Result<PathBuf> {
    let (w, h) = crate::image_convert::image_dimensions(input)?;
    let (ow, oh) = crate::image_convert::image_dimensions(overlay)?;
    if (w, h) != (ow, oh) {
        return Err(EngineError::IntegrityCheck("As anotacións non teñen o mesmo tamaño ca o GIF.".into()));
    }
    let animated = is_animated(input);
    let filter = format!("[0:v][1:v]overlay=0:0:format=auto,{}", palette_part(256, animated));
    let out = ffmpeg::convert_media_inputs(&[input, overlay], output, overwrite, &[], &gif_args(filter))?;
    check_still_animated(input, &out)?;
    Ok(out)
}

/// Escribe un GIF (animado ou non) con esa calidade (100 = paleta completa de 256 cores) e, se se pide, co lado maior reducido.
pub fn compress_gif(input: &Path, output: &Path, overwrite: bool, quality: u8, max_side: Option<u32>) -> Result<PathBuf> {
    gif_with_colors(input, output, overwrite, colors_for(quality), max_side)
}

pub(crate) fn gif_with_colors(input: &Path, output: &Path, overwrite: bool, colors: u32, max_side: Option<u32>) -> Result<PathBuf> {
    let animated = is_animated(input);
    let filter = format!("[0:v]{}{}", scale_part(max_side), palette_part(colors.clamp(2, 256), animated));
    let out = ffmpeg::convert_media_inputs(&[input], output, overwrite, &[], &gif_args(filter))?;
    check_still_animated(input, &out)?;
    Ok(out)
}

/// Imaxe animada (GIF ou PNG animado) a outro formato animado (WebP ou GIF): consérvanse todos os fotogramas e os seus tempos.
/// `quality` só afecta ao WebP (100 = sen perdas).
pub fn convert_animated(input: &Path, output: &Path, overwrite: bool, quality: Option<u8>, resize: Option<(u32, u32)>) -> Result<PathBuf> {
    let dst = crate::formats::extension_of(output).unwrap_or_default();
    let scale = resize.map(|(w, h)| format!("scale={w}:{h}:flags=lanczos")).unwrap_or_default();
    let out = match dst.as_str() {
        "webp" => {
            let q = quality.unwrap_or(80).clamp(1, 100);
            let mut args: Vec<String> = Vec::new();
            if !scale.is_empty() {
                args.extend(["-vf".into(), scale]);
            }
            args.extend(["-c:v".into(), "libwebp_anim".into(), "-loop".into(), "0".into(), "-fps_mode".into(), "vfr".into()]);
            if q >= 100 {
                args.extend(["-lossless".into(), "1".into()]);
            } else {
                args.extend(["-quality".into(), q.to_string()]);
            }
            ffmpeg::convert_media_inputs(&[input], output, overwrite, &[], &args)?
        }
        "gif" => {
            let head = if scale.is_empty() { String::new() } else { format!("{scale},") };
            ffmpeg::convert_media_inputs(&[input], output, overwrite, &[], &gif_args(format!("[0:v]{head}{}", palette_part(256, true))))?
        }
        other => return Err(EngineError::UnsupportedConversion { from: "imaxe animada".into(), to: other.into() }),
    };
    check_still_animated(input, &out)?;
    Ok(out)
}

/// A conversión desta imaxe animada pode facerse conservando a animación?
/// (FFmpeg só decodifica GIF e APNG; o WebP animado non, así que dese só se le o primeiro fotograma.)
pub fn can_keep_animation(input: &Path, dst_ext: &str) -> bool {
    let src = crate::formats::extension_of(input).unwrap_or_default();
    matches!(src.as_str(), "gif" | "png" | "apng") && matches!(dst_ext, "gif" | "webp") && ffmpeg::is_available() && is_animated(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(path: &Path) -> u32 {
        let bin = ffmpeg::ffprobe_path().unwrap();
        let o = std::process::Command::new(bin)
            .args(["-v", "error", "-count_frames", "-select_streams", "v", "-show_entries", "stream=nb_read_frames", "-of", "csv=p=0"])
            .arg(path)
            .output()
            .unwrap();
        String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(0)
    }

    fn duration(path: &Path) -> f64 {
        let bin = ffmpeg::ffprobe_path().unwrap();
        let o = std::process::Command::new(bin)
            .args(["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0"])
            .arg(path)
            .output()
            .unwrap();
        String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(0.0)
    }

    /// GIF animado de proba: 8 cadros de cores distintas con ruído, 4 por segundo.
    fn sample(dir: &Path) -> PathBuf {
        let gif = dir.join("a.gif");
        let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc=size=160x120:rate=4:duration=2", "-vf", "split[a][b];[a]palettegen[p];[b][p]paletteuse"])
            .arg(&gif)
            .status()
            .unwrap();
        assert!(st.success());
        gif
    }

    #[test]
    fn anotar_gif_conserva_todos_os_fotogramas_e_o_tempo() {
        if !ffmpeg::is_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let gif = sample(dir.path());
        assert!(is_animated(&gif));
        let overlay = dir.path().join("ov.png");
        image::RgbaImage::from_fn(160, 120, |x, y| if x < 40 && y < 30 { image::Rgba([255, 0, 0, 255]) } else { image::Rgba([0, 0, 0, 0]) }).save(&overlay).unwrap();
        let out = annotate_gif(&gif, &overlay, &dir.path().join("b.gif"), false).unwrap();
        assert!(is_animated(&out));
        assert_eq!(frames(&out), frames(&gif));
        assert!((duration(&out) - duration(&gif)).abs() < 0.05, "{} vs {}", duration(&out), duration(&gif));
        // A marca vermella aparece no primeiro e no último cadro.
        let first = image::open(&out).unwrap().to_rgb8();
        assert!(first.get_pixel(5, 5)[0] > 200 && first.get_pixel(5, 5)[1] < 60);
        let dec = image::codecs::gif::GifDecoder::new(BufReader::new(std::fs::File::open(&out).unwrap())).unwrap();
        let all: Vec<_> = dec.into_frames().collect::<std::result::Result<_, _>>().unwrap();
        assert_eq!(all.len() as u32, frames(&gif));
        for (i, f) in all.iter().enumerate() {
            let p = f.buffer().get_pixel(5, 5);
            assert!(p[0] > 200 && p[1] < 60, "cadro {i}: {p:?}");
        }
    }

    #[test]
    fn comprimir_gif_conserva_animacion() {
        if !ffmpeg::is_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let gif = sample(dir.path());
        let small = compress_gif(&gif, &dir.path().join("c.gif"), false, 40, Some(80)).unwrap();
        assert!(is_animated(&small));
        assert_eq!(frames(&small), frames(&gif));
        assert_eq!(image::image_dimensions(&small).unwrap().0, 80);
        let full = compress_gif(&gif, &dir.path().join("d.gif"), false, 100, None).unwrap();
        assert_eq!(image::image_dimensions(&full).unwrap(), (160, 120));
        assert_eq!(frames(&full), frames(&gif));
        assert!((duration(&full) - duration(&gif)).abs() < 0.05);
    }

    #[test]
    fn gif_a_webp_animado() {
        if !ffmpeg::is_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let gif = sample(dir.path());
        let webp = convert_animated(&gif, &dir.path().join("e.webp"), false, Some(80), None).unwrap();
        assert!(is_animated(&webp));
        let small = convert_animated(&gif, &dir.path().join("f.webp"), false, Some(80), Some((80, 60))).unwrap();
        assert_eq!(image::image_dimensions(&small).unwrap(), (80, 60));
    }

    /// Pasar un GIF animado a WebP desde o conversor xeral non o deixa fixo.
    #[test]
    fn o_conversor_xeral_conserva_a_animacion_de_gif_a_webp() {
        if !ffmpeg::is_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let gif = sample(dir.path());
        let out = dir.path().join("g.webp");
        crate::convert_file(&gif, &out, &crate::ConvertOptions { overwrite: true, ..Default::default() }).unwrap();
        assert!(is_animated(&out));
        // A un formato fixo, só o primeiro fotograma (a interface avisa antes).
        let png = dir.path().join("g.png");
        crate::convert_file(&gif, &png, &crate::ConvertOptions { overwrite: true, ..Default::default() }).unwrap();
        assert!(!is_animated(&png));
    }

    /// Comprimir un GIF a un tamaño máximo cumpre o tamaño e segue sendo animado.
    #[test]
    fn comprimir_gif_a_un_tamano_maximo() {
        if !ffmpeg::is_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let gif = sample(dir.path());
        let before = std::fs::metadata(&gif).unwrap().len();
        let target = before / 2;
        let out = dir.path().join("h.gif");
        let o = crate::compress::ImageCompress { target_bytes: Some(target), ..Default::default() };
        let r = crate::compress::compress_image(&gif, &out, false, &o).unwrap();
        assert!(r.bytes_written <= target, "{} > {}", r.bytes_written, target);
        assert!(is_animated(&out));
        assert_eq!(frames(&out), frames(&gif));
    }

    /// Editar unha imaxe animada falla en vez de deixala fixa; comprimir un GIF animado si se pode e segue animado.
    #[test]
    fn editar_unha_animacion_da_erro_e_comprimir_un_gif_non_a_perde() {
        let dir = tempfile::tempdir().unwrap();
        // GIF animado mínimo de dous fotogramas.
        let gif = dir.path().join("a.gif");
        {
            use image::codecs::gif::GifEncoder;
            let mut enc = GifEncoder::new(std::fs::File::create(&gif).unwrap());
            for c in [10u8, 200] {
                let f = image::Frame::new(image::RgbaImage::from_pixel(8, 8, image::Rgba([c, c, c, 255])));
                enc.encode_frame(f).unwrap();
            }
        }
        assert!(is_animated(&gif));
        let e = crate::image_convert::edit_image(&gif, &dir.path().join("e.gif"), false, &crate::image_convert::ImageEdits { rotate: 90, ..Default::default() });
        assert!(e.is_err(), "editar un GIF animado non pode deixalo fixo");
        assert!(!dir.path().join("e.gif").exists());
        // Comprimilo, en cambio, si se pode (GIF) e segue animado.
        let c = crate::compress::compress_image(&gif, &dir.path().join("c.gif"), false, &crate::compress::ImageCompress { quality: Some(60), ..Default::default() });
        if ffmpeg::is_available() {
            c.unwrap();
            assert!(is_animated(&dir.path().join("c.gif")));
        }
    }

    #[test]
    fn imaxe_fixa_non_e_animada() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.png");
        image::RgbImage::new(4, 4).save(&p).unwrap();
        assert!(!is_animated(&p));
    }
}
