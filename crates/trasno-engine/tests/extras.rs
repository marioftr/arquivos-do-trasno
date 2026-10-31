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
