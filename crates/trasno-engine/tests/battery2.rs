//! Segunda batería: edición de imaxes, ferramentas de PDF e edición de son/vídeo sobre mostras variadas.
//! Un erro limpo ante unha entrada imposible vale; un fallo ante unha entrada razoable, non.

use std::path::{Path, PathBuf};
use std::process::Command;
use trasno_engine::compress::{compress_image, compress_video, ImageCompress, VideoCompress};
use trasno_engine::ffmpeg::{self, MediaOp};
use trasno_engine::image_convert::{edit_image, ImageEdits};
use trasno_engine::{pdf, pdf_tools};

fn ff(args: &[&str]) -> bool {
    let Ok(bin) = ffmpeg::ffmpeg_path() else { return false };
    Command::new(bin).args(["-y", "-v", "error"]).args(args).status().map(|s| s.success()).unwrap_or(false)
}

fn image_samples(dir: &Path) -> Vec<(&'static str, PathBuf)> {
    let p = |f: &str| dir.join(f);
    image::RgbaImage::from_fn(64, 48, |x, y| image::Rgba([200, (x * 4) as u8, 30, if y < 24 { 255 } else { 90 }])).save(p("rgba.png")).unwrap();
    image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_fn(40, 30, |x, y| image::Rgb([(x * 1500) as u16, (y * 2000) as u16, 40000])).save(p("rgb16.png")).unwrap();
    image::RgbImage::from_pixel(2, 2, image::Rgb([9, 9, 9])).save(p("tiny.png")).unwrap();
    image::RgbImage::from_fn(63, 47, |x, y| image::Rgb([x as u8, y as u8, 7])).save(p("odd.png")).unwrap();
    image::GrayImage::from_fn(50, 50, |x, y| image::Luma([(x * y) as u8])).save(p("gray.png")).unwrap();
    image::RgbImage::from_fn(64, 48, |x, y| image::Rgb([(x * 4) as u8, 80, (y * 5) as u8])).save(p("photo.jpg")).unwrap();
    vec![("rgba", p("rgba.png")), ("16 bits", p("rgb16.png")), ("2x2", p("tiny.png")), ("impar", p("odd.png")), ("gris", p("gray.png")), ("jpg", p("photo.jpg"))]
}

fn open_ok(p: &Path) -> Result<(u32, u32), String> {
    trasno_engine::image_convert::image_dimensions(p).map_err(|e| e.to_string())
}

#[test]
fn edicion_de_imaxes_en_todos_os_formatos() {
    let dir = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    let variants: Vec<(&str, ImageEdits)> = vec![
        ("xirar 90", ImageEdits { rotate: 90, ..Default::default() }),
        ("xirar 180 + espello", ImageEdits { rotate: 180, flip_h: true, flip_v: true, ..Default::default() }),
        ("xirar 270", ImageEdits { rotate: 270, ..Default::default() }),
        ("grises + inverter", ImageEdits { grayscale: true, invert: true, ..Default::default() }),
        ("brillo + contraste", ImageEdits { brightness: 60, contrast: 40.0, ..Default::default() }),
        ("brillo - contraste -", ImageEdits { brightness: -60, contrast: -40.0, ..Default::default() }),
        ("desenfoque", ImageEdits { blur: 3.0, ..Default::default() }),
        ("recorte 1x1", ImageEdits { crop: Some((0, 0, 1, 1)), ..Default::default() }),
        ("recorte medio", ImageEdits { crop: Some((1, 1, 1, 1)), ..Default::default() }),
        ("redimensionar", ImageEdits { resize: Some((7, 5)), ..Default::default() }),
        ("redimensionar grande", ImageEdits { resize: Some((500, 300)), ..Default::default() }),
        ("todo", ImageEdits { rotate: 90, flip_h: true, grayscale: true, brightness: 10, contrast: 10.0, blur: 1.0, crop: Some((0, 0, 2, 2)), resize: Some((9, 9)), quality: Some(50), ..Default::default() }),
    ];
    for (name, src) in image_samples(dir.path()) {
        for ext in ["png", "jpg", "webp", "bmp", "tiff", "gif", "ico", "tga", "qoi"] {
            for (vn, e) in &variants {
                let out = dir.path().join(format!("e-{}-{}.{ext}", name.replace(' ', "_"), vn.replace(' ', "_")));
                let r = edit_image(&src, &out, true, e).map_err(|e| e.to_string()).and_then(|_| open_ok(&out).map(|_| ()));
                if let Err(m) = r {
                    failures.push(format!("{name} -> {ext} [{vn}]: {}", m.lines().last().unwrap_or("")));
                }
                let _ = std::fs::remove_file(&out);
            }
        }
    }
    for f in &failures {
        eprintln!("FALLO {f}");
    }
    assert!(failures.is_empty(), "{} edicións fallan:\n{}", failures.len(), failures.join("\n"));
}

/// Recortes fóra da imaxe ou de tamaño cero: erro limpo (ou recorte axustado), nunca un pánico nin un ficheiro roto.
#[test]
fn recortes_imposibles_dan_erro_limpo() {
    let dir = tempfile::tempdir().unwrap();
    let (_, src) = image_samples(dir.path()).remove(3);
    for (i, c) in [(0u32, 0u32, 0u32, 0u32), (100, 100, 5, 5), (60, 40, 100, 100), (0, 0, 1000, 1000)].into_iter().enumerate() {
        let out = dir.path().join(format!("c{i}.png"));
        let r = std::panic::catch_unwind(|| edit_image(&src, &out, true, &ImageEdits { crop: Some(c), ..Default::default() }));
        let r = r.unwrap_or_else(|_| panic!("pánico co recorte {c:?}"));
        match r {
            Ok(_) => assert!(open_ok(&out).is_ok(), "recorte {c:?} deixou un ficheiro roto"),
            Err(_) => assert!(!out.exists(), "recorte {c:?} fallou pero deixou ficheiro"),
        }
    }
}

#[test]
fn compresion_de_imaxes_sobre_todas_as_mostras() {
    let dir = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    for (name, src) in image_samples(dir.path()) {
        for ext in ["jpg", "png", "webp"] {
            for o in [
                ImageCompress { quality: Some(10), ..Default::default() },
                ImageCompress { quality: Some(100), max_side: Some(20), ..Default::default() },
                ImageCompress { target_bytes: Some(400), ..Default::default() },
                ImageCompress { target_bytes: Some(4000), max_side: Some(1), ..Default::default() },
            ] {
                let out = dir.path().join(format!("c-{}.{ext}", name.replace(' ', "_")));
                let r = compress_image(&src, &out, true, &o);
                match r {
                    Ok(_) => {
                        if let Err(m) = open_ok(&out) {
                            failures.push(format!("{name} {ext} {o:?}: saída ilegible {m}"));
                        }
                    }
                    // «non cabe en tan pouco espazo» é unha resposta lexítima; o resto non.
                    Err(e) if e.to_string().contains("caiba en tan pouco") => {}
                    Err(e) => failures.push(format!("{name} {ext} {o:?}: {e}")),
                }
                let _ = std::fs::remove_file(&out);
            }
        }
    }
    for f in &failures {
        eprintln!("FALLO {f}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn ferramentas_de_pdf_encadeadas() {
    let dir = tempfile::tempdir().unwrap();
    let imgs: Vec<PathBuf> = image_samples(dir.path()).into_iter().take(4).map(|(_, p)| p).collect();
    let base = dir.path().join("base.pdf");
    pdf::images_to_pdf(&imgs, &base, true).expect("imaxes a PDF (RGBA, 16 bits, 2x2, impar)");
    assert_eq!(pdf::page_count(&base).unwrap(), 4);
    let mut failures: Vec<String> = Vec::new();
    let mut check = |name: &str, r: trasno_engine::error::Result<PathBuf>, expect_pages: Option<u32>| match r {
        Ok(p) => match pdf::page_count(&p) {
            Ok(n) => {
                if let Some(e) = expect_pages {
                    if n != e {
                        failures.push(format!("{name}: {n} páxinas, agardábanse {e}"));
                    }
                }
            }
            Err(e) => failures.push(format!("{name}: saída ilegible: {e}")),
        },
        Err(e) => failures.push(format!("{name}: {e}")),
    };
    let o = |n: &str| dir.path().join(n);
    check("unir", pdf::merge_pdfs(&[base.clone(), base.clone()], &o("m.pdf"), true), Some(8));
    check("extraer 1,3", pdf::extract_pages(&base, &[3, 1], &o("x.pdf"), true), Some(2));
    check("xirar todas", pdf::rotate_pages(&base, None, 90, &o("r.pdf"), true), Some(4));
    check("xirar unha", pdf::rotate_pages(&base, Some(2), -90, &o("r2.pdf"), true), Some(4));
    let wm = pdf_tools::WatermarkText { text: "Mostra".into(), size: 40.0, opacity: 0.4, angle: 30.0, color: "#ff0000".into(), font: None };
    check("marca de auga", pdf_tools::watermark_text(&base, &o("w.pdf"), true, &wm, &None), Some(4));
    check("marca de auga só páxina 2", pdf_tools::watermark_text(&base, &o("w2.pdf"), true, &wm, &Some(vec![2])), Some(4));
    let pn = pdf_tools::PageNumbers { position: "bc".into(), format: "{n}/{total}".into(), start: 1, size: 10.0, margin: 20.0, color: "#000000".into(), font: None };
    check("numeración", pdf_tools::page_numbers(&base, &o("n.pdf"), true, &pn, &None), Some(4));
    let it = vec![pdf_tools::TextItem { page: 1, text: "Ola ñ á é".into(), x_pct: 10.0, y_pct: 80.0, size: 14.0, color: "#0000ff".into(), font: None, angle: 15.0 }];
    check("texto engadido", pdf_tools::add_texts(&base, &o("t.pdf"), true, &it), Some(4));
    let stamp = pdf_tools::StampImage { image: imgs[0].clone(), x_pct: 50.0, y_pct: 50.0, width_pct: 30.0, opacity: 0.8 };
    check("selo con imaxe RGBA", pdf_tools::stamp_image(&base, &o("s.pdf"), true, &stamp, &None), Some(4));
    check("recorte de páxinas", pdf_tools::crop_pages(&base, &o("cr.pdf"), true, 3.0, 3.0, 3.0, 3.0, &Some(vec![1, 2, 4])), Some(4));
    check("comprimir", pdf_tools::compress_pdf(&base, &o("cp.pdf"), true, &pdf_tools::PdfCompress { quality: 40, scale_percent: 50 }), Some(4));
    check("protexer", pdf_tools::protect_pdf(&base, &o("p.pdf"), true, "abc", "owner", true, false), None);
    assert!(pdf_tools::is_encrypted(&o("p.pdf")).unwrap_or(false), "o PDF protexido ten que estar cifrado");
    check("desprotexer", pdf_tools::unprotect_pdf(&o("p.pdf"), &o("u.pdf"), true, "abc"), Some(4));
    assert!(pdf_tools::unprotect_pdf(&o("p.pdf"), &o("u2.pdf"), true, "mal").is_err(), "un contrasinal errado non pode desprotexer");
    check("reordenar", pdf_tools::arrange_pages(&base, &o("a.pdf"), true, &[(4, 0), (1, 90), (2, 0)]), Some(3));
    check("reparar", pdf_tools::repair_pdf(&base, &o("rep.pdf"), true), Some(4));
    for f in &failures {
        eprintln!("FALLO {f}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn edicion_de_son_e_video() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let p = |f: &str| dir.path().join(f);
    let s = |f: &str| p(f).to_string_lossy().to_string();
    let mut have: Vec<(&str, PathBuf, bool)> = Vec::new();
    if ff(&["-f", "lavfi", "-i", "sine=frequency=440:duration=2", "-ac", "2", &s("a.wav")]) {
        have.push(("wav", p("a.wav"), false));
    }
    if ff(&["-f", "lavfi", "-i", "sine=duration=2", "-ac", "1", "-ar", "8000", &s("m.mp3")]) {
        have.push(("mp3 mono 8k", p("m.mp3"), false));
    }
    if ff(&["-f", "lavfi", "-i", "testsrc=size=66x50:rate=10:duration=2", "-f", "lavfi", "-i", "sine=duration=2", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", &s("v.mp4")]) {
        have.push(("mp4 con son", p("v.mp4"), true));
    }
    if ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=10:duration=2", "-c:v", "libx264", "-pix_fmt", "yuv420p", &s("mute.mp4")]) {
        have.push(("mp4 sen son", p("mute.mp4"), true));
    }
    let ops: Vec<(&str, MediaOp)> = vec![
        ("fotograma", MediaOp::Frame(0.5)),
        ("fotograma tras o final", MediaOp::Frame(99.0)),
        ("velocidade 0.25", MediaOp::Speed(0.25)),
        ("velocidade 4", MediaOp::Speed(4.0)),
        ("mudo", MediaOp::Mute),
        ("volume 0", MediaOp::Volume(0.0)),
        ("volume 4", MediaOp::Volume(4.0)),
        ("altura 24", MediaOp::ScaleHeight(24)),
        ("normalizar", MediaOp::Normalize),
        ("reverso", MediaOp::Reverse),
        ("fade in", MediaOp::FadeIn(1.0)),
        ("fade out", MediaOp::FadeOut(1.0)),
        ("eco", MediaOp::Echo(300.0)),
        ("graves", MediaOp::Bass(6.0)),
        ("agudos", MediaOp::Treble(-6.0)),
        ("ruído", MediaOp::Denoise),
    ];
    let mut failures = Vec::new();
    for (name, path, has_video) in &have {
        let ext = path.extension().unwrap().to_str().unwrap().to_string();
        for (on, op) in &ops {
            // As operacións que non teñen sentido nese tipo (efectos de audio nun vídeo sen son, altura nun audio...) poden dar erro limpo;
            // o que non pode pasar é un ficheiro baleiro ou roto no caso en que dean ben.
            let target_ext = if matches!(op, MediaOp::Frame(_)) { "png".to_string() } else { ext.clone() };
            let out = p(&format!("o-{}-{}.{target_ext}", name.replace(' ', "_"), on.replace(' ', "_")));
            let r = ffmpeg::media_edit(path, &out, true, *op, *has_video);
            match r {
                Ok(_) => {
                    let ok = std::fs::metadata(&out).map(|m| m.len() > 0).unwrap_or(false) && (target_ext == "png" || ffmpeg::probe(&out).is_ok());
                    if !ok {
                        failures.push(format!("{name} [{on}]: saída baleira ou ilegible"));
                    }
                }
                Err(e) => {
                    // Erros esperables: operación de audio sen pista de audio, altura/fotograma en audio...
                    let t = e.to_string();
                    let expected = (matches!(op, MediaOp::Frame(t) if *t > 10.0) && t.contains("baleiro")) || (!has_video && matches!(op, MediaOp::ScaleHeight(_) | MediaOp::Frame(_) | MediaOp::Mute))
                        || (*has_video && name.contains("sen son") && matches!(op, MediaOp::Volume(_) | MediaOp::Normalize | MediaOp::Reverse | MediaOp::FadeIn(_) | MediaOp::FadeOut(_) | MediaOp::Echo(_) | MediaOp::Bass(_) | MediaOp::Treble(_) | MediaOp::Denoise | MediaOp::Mute));
                    eprintln!("(erro {}: {name} [{on}]: {})", if expected { "esperable" } else { "INESPERADO" }, t.lines().last().unwrap_or(""));
                    if !expected {
                        failures.push(format!("{name} [{on}]: {}", t.lines().last().unwrap_or("")));
                    }
                }
            }
            let _ = std::fs::remove_file(&out);
        }
    }
    for f in &failures {
        eprintln!("FALLO {f}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn cortar_e_comprimir_video_en_casos_difíciles() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let p = |f: &str| dir.path().join(f);
    let s = |f: &str| p(f).to_string_lossy().to_string();
    assert!(ff(&["-f", "lavfi", "-i", "testsrc=size=66x50:rate=10:duration=3", "-f", "lavfi", "-i", "sine=duration=3", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", &s("v.mp4")]));
    assert!(ff(&["-f", "lavfi", "-i", "testsrc=size=64x48:rate=10:duration=3", "-c:v", "libx264", "-pix_fmt", "yuv420p", &s("mute.mp4")]));
    let mut failures = Vec::new();
    // Cortes
    for (n, a, b) in [("normal", 0.5, Some(2.0)), ("ata o final", 1.0, None), ("fin tras a duración", 1.0, Some(99.0)), ("inicio tras a duración", 50.0, None)] {
        let out = p("t.mp4");
        match ffmpeg::trim_media(&p("v.mp4"), &out, true, a, b) {
            Ok(_) => {
                if ffmpeg::probe(&out).is_err() {
                    failures.push(format!("corte {n}: saída ilegible"));
                }
            }
            Err(e) => eprintln!("(corte {n}: erro: {})", e.to_string().lines().last().unwrap_or("")),
        }
        let _ = std::fs::remove_file(&out);
    }
    // Unir
    let c = p("cat.mp4");
    if let Err(e) = ffmpeg::concat_media(&[p("v.mp4"), p("v.mp4")], &c, true) {
        failures.push(format!("unir dous MP4 iguais: {e}"));
    } else if let Ok(info) = ffmpeg::probe(&c) {
        let d: f64 = info["format"]["duration"].as_str().and_then(|x| x.parse().ok()).unwrap_or(0.0);
        if d < 5.5 {
            failures.push(format!("unir: duración {d}, agardábase uns 6 s"));
        }
    }
    // Comprimir con perfís habituais, con e sen son e con medidas impares
    for src in ["v.mp4", "mute.mp4"] {
        for (n, o) in [
            ("calidade", VideoCompress { crf: 28, audio_kbps: 96, ..Default::default() }),
            ("compat + 720", VideoCompress { crf: 22, audio_kbps: 128, max_height: Some(720), compat: true, ..Default::default() }),
            ("altura 32", VideoCompress { crf: 30, audio_kbps: 64, max_height: Some(32), ..Default::default() }),
            ("bitrate", VideoCompress { crf: 28, audio_kbps: 64, video_kbps: Some(200), ..Default::default() }),
            ("desentrelazar", VideoCompress { crf: 28, audio_kbps: 64, deinterlace: 2, ..Default::default() }),
            ("DAR 16/9", VideoCompress { crf: 28, audio_kbps: 64, dar: Some("16/9".into()), ..Default::default() }),
            ("tamaño obxectivo", VideoCompress { crf: 28, audio_kbps: 64, target_bytes: Some(60_000), ..Default::default() }),
        ] {
            let out = p("cv.mp4");
            match compress_video(&p(src), &out, true, &o) {
                Ok(_) => {
                    if ffmpeg::probe(&out).is_err() {
                        failures.push(format!("{src} [{n}]: saída ilegible"));
                    }
                }
                Err(e) => failures.push(format!("{src} [{n}]: {}", e.to_string().lines().last().unwrap_or(""))),
            }
            let _ = std::fs::remove_file(&out);
        }
    }
    for f in &failures {
        eprintln!("FALLO {f}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
