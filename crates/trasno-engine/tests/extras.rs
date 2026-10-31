use trasno_engine::{ffmpeg, image_convert, pdf};

#[test]
fn imaxes_a_pdf_fai_unha_paxina_por_imaxe() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.png");
    let b = dir.path().join("b.jpg");
    image::DynamicImage::ImageRgb8(image::RgbImage::new(40, 30)).save(&a).unwrap();
    image::DynamicImage::ImageRgb8(image::RgbImage::new(20, 20)).save(&b).unwrap();
    let out = dir.path().join("todo.pdf");
    pdf::images_to_pdf(&[a.clone(), b], &out, false).unwrap();
    assert_eq!(pdf::page_count(&out).unwrap(), 2);
    // Non pisa un destino existente nin o orixe.
    assert!(pdf::images_to_pdf(&[a.clone()], &out, false).is_err());
    assert!(pdf::images_to_pdf(&[a.clone()], &a, true).is_err());
}

#[test]
fn edicions_de_imaxe_non_tocan_o_orixinal() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.png");
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(8, 4, image::Rgb([200, 10, 10])))
        .save(&a)
        .unwrap();
    let before = std::fs::read(&a).unwrap();
    let out = dir.path().join("b.png");
    let edits = image_convert::ImageEdits {
        rotate: 90,
        grayscale: true,
        flip_h: true,
        brightness: 20,
        contrast: 10.0,
        blur: 1.0,
        invert: true,
        ..Default::default()
    };
    image_convert::edit_image(&a, &out, false, &edits).unwrap();
    let d = image::open(&out).unwrap();
    assert_eq!((d.width(), d.height()), (4, 8));
    assert_eq!(std::fs::read(&a).unwrap(), before);
    let bad = image_convert::ImageEdits { rotate: 45, ..Default::default() };
    assert!(image_convert::edit_image(&a, &dir.path().join("c.png"), false, &bad).is_err());
}

#[test]
fn operacions_de_media_validan_os_valores() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("x.mp4");
    std::fs::write(&f, b"nada").unwrap();
    let out = dir.path().join("y.mp4");
    for op in [
        ffmpeg::MediaOp::Speed(10.0),
        ffmpeg::MediaOp::Volume(-1.0),
        ffmpeg::MediaOp::Frame(-3.0),
        ffmpeg::MediaOp::ScaleHeight(2),
    ] {
        let e = ffmpeg::media_edit(&f, &out, false, op, true).unwrap_err();
        assert!(e.to_string().contains("verificación"), "{e}");
    }
}

#[test]
fn operacions_de_media_funcionan_con_ffmpeg_real() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.mp4");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc=size=160x120:rate=10:duration=2"])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=2", "-shortest", "-pix_fmt", "yuv420p"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let cases: Vec<(ffmpeg::MediaOp, &str)> = vec![
        (ffmpeg::MediaOp::Frame(0.5), "f.png"),
        (ffmpeg::MediaOp::Speed(2.0), "s.mp4"),
        (ffmpeg::MediaOp::Speed(0.25), "s2.mp4"),
        (ffmpeg::MediaOp::Mute, "m.mp4"),
        (ffmpeg::MediaOp::Volume(0.5), "v.mp4"),
        (ffmpeg::MediaOp::ScaleHeight(60), "e.mp4"),
    ];
    for (op, name) in cases {
        let out = dir.path().join(name);
        ffmpeg::media_edit(&src, &out, false, op, true).unwrap_or_else(|e| panic!("{op:?}: {e}"));
        assert!(std::fs::metadata(&out).unwrap().len() > 0, "{op:?}");
    }
    let info = ffmpeg::probe(&dir.path().join("e.mp4")).unwrap();
    assert_eq!(info["streams"][0]["height"], 60);
}

#[test]
fn xuntar_e_convertir_media_con_ffmpeg_real() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.mp4");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc=size=160x120:rate=10:duration=1"])
        .args(["-f", "lavfi", "-i", "sine=duration=1", "-shortest", "-pix_fmt", "yuv420p"])
        .arg(&a)
        .status()
        .unwrap();
    assert!(st.success());
    let b = dir.path().join("b.mp4");
    std::fs::copy(&a, &b).unwrap();
    let joined = dir.path().join("j.mp4");
    ffmpeg::concat_media(&[a.clone(), b], &joined, false).unwrap();
    let dur: f64 = ffmpeg::probe(&joined).unwrap()["format"]["duration"].as_str().unwrap().parse().unwrap();
    assert!(dur > 1.8, "{dur}");
    let mp3 = dir.path().join("a.mp3");
    trasno_engine::convert_file(&a, &mp3, &Default::default()).unwrap();
    assert!(mp3.exists());
    let webm = dir.path().join("a.webm");
    trasno_engine::convert_file(&a, &webm, &Default::default()).unwrap();
    assert!(webm.exists());
}

use trasno_engine::compress;

fn noisy_png(path: &std::path::Path, w: u32, h: u32) {
    let img = image::RgbImage::from_fn(w, h, |x, y| {
        let v = (x.wrapping_mul(2654435761) ^ y.wrapping_mul(40503)) as u8;
        image::Rgb([v, v.wrapping_add((x / 3) as u8), (y % 251) as u8])
    });
    image::DynamicImage::ImageRgb8(img).save(path).unwrap();
}

#[test]
fn comprimir_imaxe_a_un_tamano_maximo() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("a.png");
    noisy_png(&src, 800, 600);
    let before = std::fs::read(&src).unwrap();
    let out = dir.path().join("a.jpg");
    let target = 60_000u64;
    let r = compress::compress_image(
        &src,
        &out,
        false,
        &compress::ImageCompress { target_bytes: Some(target), ..Default::default() },
    )
    .unwrap();
    assert!(r.bytes_written <= target, "{} > {}", r.bytes_written, target);
    assert_eq!(std::fs::metadata(&out).unwrap().len(), r.bytes_written);
    assert_eq!(std::fs::read(&src).unwrap(), before);
    // Só JPG como saída, e nunca pisa nada.
    assert!(compress::compress_image(&src, &dir.path().join("a.png2.png"), false, &Default::default()).is_err());
    assert!(compress::compress_image(&src, &out, false, &Default::default()).is_err());
    // A estimación vai en liña co resultado real á mesma calidade.
    let est = compress::estimate_image_bytes(&src, 80, None).unwrap();
    let real = compress::compress_image(&src, &dir.path().join("b.jpg"), false, &compress::ImageCompress { quality: Some(80), ..Default::default() }).unwrap();
    assert_eq!(est, real.bytes_written);
}

#[test]
fn comprimir_video_por_calidade_e_por_tamano_maximo() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.mp4");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=640x360:rate=25:duration=6"])
        .args(["-f", "lavfi", "-i", "sine=frequency=300:duration=6", "-shortest", "-c:v", "libx264", "-b:v", "3000k", "-pix_fmt", "yuv420p"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let src_size = std::fs::metadata(&src).unwrap().len();

    let by_crf = dir.path().join("crf.mp4");
    compress::compress_video(&src, &by_crf, false, &compress::VideoCompress { crf: 32, target_bytes: None, max_height: Some(240), audio_kbps: 64, video_kbps: None }).unwrap();
    let crf_size = std::fs::metadata(&by_crf).unwrap().len();
    assert!(crf_size < src_size, "{crf_size} !< {src_size}");
    let info = ffmpeg::probe(&by_crf).unwrap();
    assert_eq!(info["streams"][0]["height"], 240);

    let target = 250_000u64;
    let by_size = dir.path().join("size.mp4");
    compress::compress_video(&src, &by_size, false, &compress::VideoCompress { crf: 28, target_bytes: Some(target), max_height: None, audio_kbps: 64, video_kbps: None }).unwrap();
    let got = std::fs::metadata(&by_size).unwrap().len();
    assert!(got <= target, "{got} > {target}");
    assert!(got > target / 3, "demasiado lonxe do obxectivo: {got}");
    // Non deixa restos temporais.
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with(".trasno")).collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");

    // Obxectivo imposible: erro claro, sen ficheiro.
    let tiny = dir.path().join("tiny.mp4");
    assert!(compress::compress_video(&src, &tiny, false, &compress::VideoCompress { crf: 28, target_bytes: Some(2_000), max_height: None, audio_kbps: 64, video_kbps: None }).is_err());
    assert!(!tiny.exists());

    // Estimación razoable do modo por calidade.
    let est = compress::estimate_video_bytes(&src, 32, Some(240), 64).unwrap();
    assert!(est > crf_size / 3 && est < crf_size * 3, "est {est} real {crf_size}");
}

#[test]
fn comprimir_video_por_bitrate_da_imaxe() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.mp4");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=640x360:rate=25:duration=6"])
        .args(["-c:v", "libx264", "-b:v", "4000k", "-pix_fmt", "yuv420p"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let out = dir.path().join("b.mp4");
    compress::compress_video(&src, &out, false, &compress::VideoCompress { crf: 28, target_bytes: None, max_height: None, audio_kbps: 64, video_kbps: Some(300) }).unwrap();
    let bytes = std::fs::metadata(&out).unwrap().len() as f64;
    // ~300 kbps durante 6 s = ~225 KB (con marxe).
    assert!(bytes < 400_000.0 && bytes > 50_000.0, "{bytes}");
    assert!(compress::compress_video(&src, &dir.path().join("c.mp4"), false, &compress::VideoCompress { crf: 28, target_bytes: None, max_height: None, audio_kbps: 64, video_kbps: Some(5) }).is_err());
}
