//! Batería de conversións: xera mostras variadas (imaxes con alfa, de 16 bits, de 1 píxel, GIF animado; audio mono e
//! estéreo; vídeo de medidas impares, sen son, con varias pistas, 10 bits...) e proba cada conversión que a app
//! ofrece (`suggested_targets`). Cada resultado ten que existir, non estar baleiro e poder lerse de novo.
//! Se falta FFmpeg ou o seu códec, esa mostra salta (non é un fallo do motor).

use std::path::{Path, PathBuf};
use std::process::Command;
use trasno_engine::{convert_file, ffmpeg, suggested_targets, ConvertOptions};

fn ff(args: &[&str]) -> bool {
    let Ok(bin) = ffmpeg::ffmpeg_path() else { return false };
    Command::new(bin).args(["-y", "-v", "error"]).args(args).status().map(|s| s.success()).unwrap_or(false)
}

struct Sample {
    name: &'static str,
    path: PathBuf,
}

fn samples(dir: &Path) -> Vec<Sample> {
    let mut v = Vec::new();
    let mut add = |name: &'static str, file: &str, ok: bool| {
        if ok && dir.join(file).is_file() {
            v.push(Sample { name, path: dir.join(file) });
        } else {
            eprintln!("(mostra {name} non dispoñible)");
        }
    };
    // Imaxes feitas co propio motor
    image::RgbImage::from_fn(64, 48, |x, y| image::Rgb([(x * 4) as u8, (y * 5) as u8, 90])).save(dir.join("rgb.png")).unwrap();
    image::RgbaImage::from_fn(64, 48, |x, y| image::Rgba([200, (x * 4) as u8, 30, if y < 24 { 255 } else { 90 }])).save(dir.join("rgba.png")).unwrap();
    image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_fn(40, 30, |x, y| image::Rgb([(x * 1500) as u16, (y * 2000) as u16, 40000])).save(dir.join("rgb16.png")).unwrap();
    image::RgbImage::from_pixel(1, 1, image::Rgb([9, 9, 9])).save(dir.join("one.png")).unwrap();
    image::RgbImage::from_fn(63, 47, |x, y| image::Rgb([x as u8, y as u8, 7])).save(dir.join("odd.png")).unwrap();
    image::GrayImage::from_fn(50, 50, |x, y| image::Luma([(x * y) as u8])).save(dir.join("gray.png")).unwrap();
    image::RgbImage::from_fn(64, 48, |x, y| image::Rgb([(x * 4) as u8, 80, (y * 5) as u8])).save(dir.join("photo.jpg")).unwrap();
    for (n, f) in [("png rgb", "rgb.png"), ("png alfa", "rgba.png"), ("png 16 bits", "rgb16.png"), ("png 1x1", "one.png"), ("png impar", "odd.png"), ("png gris", "gray.png"), ("jpg", "photo.jpg")] {
        add(n, f, true);
    }
    // Resto con FFmpeg
    let g = ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=5:duration=1", &dir.join("anim.gif").to_string_lossy()]);
    add("gif animado", "anim.gif", g);
    let p = |f: &str| dir.join(f).to_string_lossy().to_string();
    let t = ff(&["-i", &p("rgb.png"), &p("pic.webp")]);
    add("webp", "pic.webp", t);
    let t = ff(&["-i", &p("rgb.png"), &p("pic.bmp")]);
    add("bmp", "pic.bmp", t);
    let t = ff(&["-i", &p("rgb.png"), &p("pic.tiff")]);
    add("tiff", "pic.tiff", t);
    // Audio
    let t = ff(&["-f", "lavfi", "-i", "sine=frequency=440:duration=1", "-ac", "2", &p("st.wav")]);
    add("wav estéreo", "st.wav", t);
    let t = ff(&["-f", "lavfi", "-i", "sine=frequency=330:duration=1", "-ac", "1", "-ar", "8000", &p("mono8k.wav")]);
    add("wav mono 8 kHz", "mono8k.wav", t);
    let t = ff(&["-i", &p("st.wav"), &p("a.mp3")]);
    add("mp3", "a.mp3", t);
    let t = ff(&["-i", &p("st.wav"), &p("a.flac")]);
    add("flac", "a.flac", t);
    let t = ff(&["-i", &p("st.wav"), &p("a.ogg")]);
    add("ogg", "a.ogg", t);
    // Vídeo
    let t = ff(&["-f", "lavfi", "-i", "testsrc=size=66x50:rate=10:duration=1", "-f", "lavfi", "-i", "sine=duration=1", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest", &p("v_audio.mp4")]);
    add("mp4 con son", "v_audio.mp4", t);
    let t = ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=10:duration=1", "-c:v", "libx264", "-pix_fmt", "yuv420p", &p("v_mute.mp4")]);
    add("mp4 sen son", "v_mute.mp4", t);
    let t = ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=10:duration=1", "-f", "lavfi", "-i", "sine=frequency=300:duration=1", "-f", "lavfi", "-i", "sine=frequency=600:duration=1", "-map", "0", "-map", "1", "-map", "2", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", &p("v_2audio.mkv")]);
    add("mkv 2 pistas de audio", "v_2audio.mkv", t);
    let t = ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=10:duration=1", "-c:v", "libvpx-vp9", "-b:v", "200k", &p("v.webm")]);
    add("webm vp9", "v.webm", t);
    let t = ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=10:duration=1", "-f", "lavfi", "-i", "sine=duration=1", "-c:v", "mpeg4", "-c:a", "mp3", &p("v.avi")]);
    add("avi mpeg4", "v.avi", t);
    let t = ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=10:duration=1", "-c:v", "libx265", "-pix_fmt", "yuv420p10le", "-x265-params", "log-level=none", &p("v10.mp4")]);
    add("mp4 HEVC 10 bits", "v10.mp4", t);
    let t = ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=10:duration=1", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-metadata:s:v", "rotate=90", &p("v.mov")]);
    add("mov", "v.mov", t);
    v
}

/// Comprobación do resultado: ficheiro non baleiro e lexible polo motor ou por ffprobe.
fn readable(out: &Path) -> Result<(), String> {
    let len = std::fs::metadata(out).map_err(|e| e.to_string())?.len();
    if len == 0 {
        return Err("baleiro".into());
    }
    let ext = out.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let image_like = ["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "ico", "avif", "jxl", "jp2", "tga", "qoi", "pnm"].contains(&ext.as_str());
    if image_like {
        return trasno_engine::image_convert::image_dimensions(out).map(|_| ()).map_err(|e| e.to_string());
    }
    ffmpeg::probe(out).map(|_| ()).map_err(|e| e.to_string())
}

#[test]
fn todas_as_conversions_ofrecidas_funcionan() {
    if !ffmpeg::is_available() {
        eprintln!("sen FFmpeg: batería omitida");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let all = samples(dir.path());
    let mut failures: Vec<String> = Vec::new();
    let mut total = 0;
    for s in &all {
        let in_ext = s.path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
        for target in suggested_targets(&s.path) {
            if target == in_ext {
                continue;
            }
            total += 1;
            let out = dir.path().join(format!("out-{}-{}.{}", s.name.replace(' ', "_"), target, target));
            let res = convert_file(&s.path, &out, &ConvertOptions { overwrite: true, ..Default::default() }).map_err(|e| e.to_string()).and_then(|_| readable(&out));
            if let Err(e) = res {
                failures.push(format!("{} -> {}: {}", s.name, target, e.lines().last().unwrap_or("")));
            }
            let _ = std::fs::remove_file(&out);
        }
    }
    eprintln!("{} conversións probadas, {} con fallo", total, failures.len());
    for f in &failures {
        eprintln!("FALLO {f}");
    }
    assert!(failures.is_empty(), "{} conversións fallan:\n{}", failures.len(), failures.join("\n"));
}

/// Opcións habituais (calidade, redimensionar, bitrate) sobre todos os destinos: tampouco poden fallar.
#[test]
fn opcions_de_conversion_funcionan() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let all = samples(dir.path());
    let mut failures: Vec<String> = Vec::new();
    for s in &all {
        let in_ext = s.path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
        let is_image = matches!(trasno_engine::formats::category_of_path(&s.path), trasno_engine::formats::Category::Imaxe);
        // Só unhas mostras representativas para non eternizar a proba.
        if !["png alfa", "png 16 bits", "jpg", "wav estéreo", "mp4 con son", "mp4 sen son", "avi mpeg4"].contains(&s.name) {
            continue;
        }
        let variants: Vec<ConvertOptions> = if is_image {
            vec![
                ConvertOptions { overwrite: true, quality: Some(1), ..Default::default() },
                ConvertOptions { overwrite: true, quality: Some(100), resize: Some((33, 17)), ..Default::default() },
                ConvertOptions { overwrite: true, resize: Some((300, 200)), ..Default::default() },
            ]
        } else {
            vec![
                ConvertOptions { overwrite: true, bitrate_kbps: Some(32), ..Default::default() },
                ConvertOptions { overwrite: true, bitrate_kbps: Some(320), ..Default::default() },
            ]
        };
        for (i, o) in variants.iter().enumerate() {
            for target in suggested_targets(&s.path) {
                if target == in_ext {
                    continue;
                }
                // A calidade/redimensión só vale para imaxes; os vídeos e audios con bitrate.
                let out = dir.path().join(format!("o-{}-{}-{}.{}", s.name.replace(' ', "_"), i, target, target));
                let res = convert_file(&s.path, &out, o).map_err(|e| e.to_string()).and_then(|_| readable(&out));
                if let Err(e) = res {
                    failures.push(format!("{} -> {} (variante {}): {}", s.name, target, i, e.lines().last().unwrap_or("")));
                }
                let _ = std::fs::remove_file(&out);
            }
        }
    }
    for f in &failures {
        eprintln!("FALLO {f}");
    }
    assert!(failures.is_empty(), "{} fallan:
{}", failures.len(), failures.join("
"));
}

/// Un PNG con zonas transparentes en JPG non queda con fondo negro: queda branco.
#[test]
fn transparencia_a_jpg_sobre_branco() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("t.png");
    image::RgbaImage::from_pixel(20, 20, image::Rgba([0, 0, 0, 0])).save(&src).unwrap();
    let out = dir.path().join("t.jpg");
    convert_file(&src, &out, &ConvertOptions::default()).unwrap();
    let px = image::open(&out).unwrap().to_rgb8().get_pixel(5, 5).0;
    assert!(px.iter().all(|c| *c > 240), "{px:?}");
}
