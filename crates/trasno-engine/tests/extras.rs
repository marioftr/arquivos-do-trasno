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
fn efectos_de_audio_funcionan_con_ffmpeg_real() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("a.wav");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "sine=frequency=440:duration=2"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let ops = [
        ffmpeg::MediaOp::Normalize,
        ffmpeg::MediaOp::Reverse,
        ffmpeg::MediaOp::FadeIn(0.5),
        ffmpeg::MediaOp::FadeOut(0.5),
        ffmpeg::MediaOp::Echo(200.0),
        ffmpeg::MediaOp::Bass(6.0),
        ffmpeg::MediaOp::Treble(-6.0),
        ffmpeg::MediaOp::Denoise,
    ];
    for (i, op) in ops.into_iter().enumerate() {
        let out = dir.path().join(format!("o{i}.wav"));
        ffmpeg::media_edit(&src, &out, false, op, false).unwrap_or_else(|e| panic!("{op:?}: {e}"));
        assert!(std::fs::metadata(&out).unwrap().len() > 0, "{op:?}");
    }
    assert!(ffmpeg::media_edit(&src, &dir.path().join("x.wav"), false, ffmpeg::MediaOp::Bass(99.0), false).is_err());
}

#[test]
fn mkv_quita_pistas_e_cambia_nomes_sen_recodificar() {
    use trasno_engine::mkv;
    if !ffmpeg::is_available() || !mkv::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("a.mkv");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc=size=160x120:rate=10:duration=1"])
        .args(["-f", "lavfi", "-i", "sine=duration=1", "-shortest", "-pix_fmt", "yuv420p"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let before = mkv::tracks(&src).unwrap();
    assert_eq!(before["tracks"].as_array().unwrap().len(), 2);

    let out = dir.path().join("so-video.mkv");
    let edits = vec![mkv::TrackEdit { id: 0, name: Some("Principal".into()), language: Some("glg".into()), default: Some(true) }];
    let one = |keep: Vec<u64>, edits: Vec<mkv::TrackEdit>| [mkv::MuxInput { path: src.clone(), keep, edits }];
    mkv::mux(&one(vec![0], edits), &[], &out, false).unwrap();
    let after = mkv::tracks(&out).unwrap();
    let list = after["tracks"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["properties"]["track_name"], "Principal");
    assert_eq!(list[0]["properties"]["language"], "glg");
    // Sen pistas non hai nada que gardar, e o orixinal queda igual.
    assert!(mkv::mux(&one(vec![], vec![]), &[], &dir.path().join("x.mkv"), false).is_err());
    // Idioma inválido: rexeitado antes de lanzar nada.
    let bad = vec![mkv::TrackEdit { id: 0, name: None, language: Some("--x".into()), default: None }];
    assert!(mkv::mux(&one(vec![0], bad), &[], &dir.path().join("y.mkv"), false).is_err());
    // Mestura de dous ficheiros coa orde cambiada: o audio de b antes do vídeo de a.
    let b = dir.path().join("b.mkv");
    std::fs::copy(&src, &b).unwrap();
    let both = [
        mkv::MuxInput { path: src.clone(), keep: vec![0], edits: vec![] },
        mkv::MuxInput { path: b.clone(), keep: vec![1], edits: vec![] },
    ];
    let order = [mkv::TrackRef { input: 1, id: 1 }, mkv::TrackRef { input: 0, id: 0 }];
    let mixed = dir.path().join("mix.mkv");
    mkv::mux(&both, &order, &mixed, false).unwrap();
    let t = mkv::tracks(&mixed).unwrap();
    let list = t["tracks"].as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["type"], "audio");
    assert_eq!(list[1]["type"], "video");
    assert_eq!(mkv::tracks(&src).unwrap()["tracks"].as_array().unwrap().len(), 2);
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
    // Formatos sen compresión propia (BMP...) non valen como saída, e nunca pisa nada.
    assert!(compress::compress_image(&src, &dir.path().join("a.bmp"), false, &Default::default()).is_err());
    assert!(compress::compress_image(&src, &out, false, &Default::default()).is_err());
    // A estimación vai en liña co resultado real á mesma calidade.
    let est = compress::estimate_image_bytes(&src, "jpg", 80, None, None).unwrap();
    let real = compress::compress_image(&src, &dir.path().join("b.jpg"), false, &compress::ImageCompress { quality: Some(80), ..Default::default() }).unwrap();
    assert_eq!(est, real.bytes_written);
}

/// Comprimir mantén o formato: PNG sen perdas (reabre igual), e WebP/AVIF/JPEG XL se hai FFmpeg.
#[test]
fn comprimir_imaxe_no_mesmo_formato() {
    assert_eq!(compress::output_ext_for("PNG"), "png");
    assert_eq!(compress::output_ext_for("webp"), "webp");
    assert_eq!(compress::output_ext_for("bmp"), "jpg");
    assert_eq!(compress::output_ext_for("gif"), "jpg");
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("a.png");
    image::RgbImage::from_fn(300, 200, |x, y| image::Rgb([(x / 4) as u8, (y / 4) as u8, 120])).save(&src).unwrap();
    let out = dir.path().join("a-c.png");
    let r = compress::compress_image(&src, &out, false, &Default::default()).unwrap();
    assert_eq!(image::open(&out).unwrap().to_rgb8(), image::open(&src).unwrap().to_rgb8(), "PNG sen perdas");
    assert_eq!(r.bytes_written, std::fs::metadata(&out).unwrap().len());
    // Con tamaño obxectivo en PNG só se reducen as dimensións.
    let small = dir.path().join("a-s.png");
    let target = r.bytes_written / 3;
    let r2 = compress::compress_image(&src, &small, false, &compress::ImageCompress { target_bytes: Some(target), ..Default::default() }).unwrap();
    assert!(r2.bytes_written <= target && r2.scale_percent < 100);
    if ffmpeg::is_available() {
        for ext in ["webp", "avif", "jxl"] {
            let o = dir.path().join(format!("a-c.{ext}"));
            match compress::compress_image(&src, &o, false, &compress::ImageCompress { quality: Some(60), ..Default::default() }) {
                Ok(rep) => assert!(rep.bytes_written > 0 && image_dimensions_ok(&o), "{ext}"),
                Err(e) => eprintln!("{ext}: {e}"),
            }
        }
    }
}

fn image_dimensions_ok(p: &std::path::Path) -> bool {
    trasno_engine::image_convert::image_dimensions(p).map(|d| d == (300, 200)).unwrap_or(false)
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
    compress::compress_video(&src, &by_crf, false, &compress::VideoCompress { crf: 32, target_bytes: None, max_height: Some(240), audio_kbps: 64, video_kbps: None, ..Default::default() }).unwrap();
    let crf_size = std::fs::metadata(&by_crf).unwrap().len();
    assert!(crf_size < src_size, "{crf_size} !< {src_size}");
    let info = ffmpeg::probe(&by_crf).unwrap();
    assert_eq!(info["streams"][0]["height"], 240);

    let target = 250_000u64;
    let by_size = dir.path().join("size.mp4");
    compress::compress_video(&src, &by_size, false, &compress::VideoCompress { crf: 28, target_bytes: Some(target), max_height: None, audio_kbps: 64, video_kbps: None, ..Default::default() }).unwrap();
    let got = std::fs::metadata(&by_size).unwrap().len();
    assert!(got <= target, "{got} > {target}");
    assert!(got > target / 3, "demasiado lonxe do obxectivo: {got}");
    // Non deixa restos temporais.
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with(".trasno")).collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");

    // Obxectivo imposible: erro claro, sen ficheiro.
    let tiny = dir.path().join("tiny.mp4");
    assert!(compress::compress_video(&src, &tiny, false, &compress::VideoCompress { crf: 28, target_bytes: Some(2_000), max_height: None, audio_kbps: 64, video_kbps: None, ..Default::default() }).is_err());
    assert!(!tiny.exists());

    // Estimación razoable do modo por calidade.
    let est = compress::estimate_video_bytes(&src, &compress::VideoCompress { crf: 32, max_height: Some(240), audio_kbps: 64, ..Default::default() }).unwrap();
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
    compress::compress_video(&src, &out, false, &compress::VideoCompress { crf: 28, target_bytes: None, max_height: None, audio_kbps: 64, video_kbps: Some(300), ..Default::default() }).unwrap();
    let bytes = std::fs::metadata(&out).unwrap().len() as f64;
    // ~300 kbps durante 6 s = ~225 KB (con marxe).
    assert!(bytes < 400_000.0 && bytes > 50_000.0, "{bytes}");
    assert!(compress::compress_video(&src, &dir.path().join("c.mp4"), false, &compress::VideoCompress { crf: 28, target_bytes: None, max_height: None, audio_kbps: 64, video_kbps: Some(5), ..Default::default() }).is_err());
}

#[test]
fn perfil_minidv_desentrelaza_fixa_aspecto_e_usa_high() {
    if ffmpeg::ffmpeg_path().is_err() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("cinta.avi");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=720x576:rate=25:duration=2"])
        .args(["-f", "lavfi", "-i", "sine=frequency=300:duration=2", "-shortest", "-c:v", "mpeg4", "-q:v", "3", "-flags", "+ilme+ildct"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let out = dir.path().join("cinta.mp4");
    let o = compress::VideoCompress { crf: 18, audio_kbps: 192, deinterlace: 2, compat: true, dar: Some("16/9".into()), ..Default::default() };
    compress::compress_video(&src, &out, false, &o).unwrap();
    let info = ffmpeg::probe(&out).unwrap();
    let v = &info["streams"][0];
    assert_eq!(v["codec_name"], "h264");
    assert_eq!(v["profile"], "High");
    assert_eq!(v["display_aspect_ratio"], "16:9");
    // yadif=1 dobra a cadencia: 25 -> 50 cadros por segundo.
    assert_eq!(v["r_frame_rate"], "50/1");
}

#[test]
fn un_tramo_de_vista_previa_empeza_onde_se_pide_e_dura_o_pedido() {
    if ffmpeg::ffmpeg_path().is_err() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("cinta.avi");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=320x240:rate=25:duration=20"])
        .args(["-f", "lavfi", "-i", "sine=frequency=300:duration=20", "-shortest", "-c:v", "mpeg4", "-q:v", "4"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let out = dir.path().join("tramo.mp4");
    let pre = vec!["-ss".to_string(), "10.000".to_string()];
    let args: Vec<String> = ["-t", "5.000", "-vf", "yadif=0:-1:1", "-c:v", "libx264", "-preset", "ultrafast", "-crf", "17", "-c:a", "aac", "-pix_fmt", "yuv420p"].iter().map(|s| s.to_string()).collect();
    ffmpeg::convert_media_seek(&src, &out, true, &pre, &args).unwrap();
    let info = ffmpeg::probe(&out).unwrap();
    let d: f64 = info["format"]["duration"].as_str().unwrap().parse().unwrap();
    assert!((d - 5.0).abs() < 0.3, "duración {d}");
    assert_eq!(info["streams"][0]["width"], 320);
}

#[test]
fn elixir_pista_de_audio_e_sacar_subtitulos_a_webvtt() {
    if ffmpeg::ffmpeg_path().is_err() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let srt = dir.path().join("s.srt");
    std::fs::write(&srt, "1\n00:00:01,000 --> 00:00:02,500\nOla <i>trasno</i>\n\n").unwrap();
    let src = dir.path().join("peli.mkv");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=160x90:rate=10:duration=3"])
        .args(["-f", "lavfi", "-i", "sine=frequency=300:duration=3", "-f", "lavfi", "-i", "sine=frequency=900:duration=3"])
        .arg("-i").arg(&srt)
        .args(["-map", "0:v", "-map", "1:a", "-map", "2:a", "-map", "3:s", "-c:v", "libx264", "-c:a", "aac", "-c:s", "srt", "-metadata:s:a:1", "language=eng"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    // Segunda pista de audio, copiada a un MP4 sen recodificar o vídeo.
    let out = dir.path().join("a2.mp4");
    let args: Vec<String> = ["-map", "0:v:0?", "-map", "0:a:1?", "-c", "copy"].iter().map(|s| s.to_string()).collect();
    ffmpeg::convert_media(&src, &out, true, &args).unwrap();
    let info = ffmpeg::probe(&out).unwrap();
    let audios: Vec<_> = info["streams"].as_array().unwrap().iter().filter(|s| s["codec_type"] == "audio").collect();
    assert_eq!(audios.len(), 1);
    // Subtítulos a WebVTT.
    let vtt = dir.path().join("s.vtt");
    let args: Vec<String> = ["-map", "0:s:0", "-c:s", "webvtt"].iter().map(|s| s.to_string()).collect();
    ffmpeg::convert_media(&src, &vtt, true, &args).unwrap();
    let text = std::fs::read_to_string(&vtt).unwrap();
    assert!(text.contains("-->") && text.contains("trasno"), "{text}");
}

#[test]
fn unha_saida_sen_extension_da_un_erro_claro() {
    if ffmpeg::ffmpeg_path().is_err() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("a.wav");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "sine=frequency=300:duration=1"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let err = ffmpeg::convert_media(&src, &dir.path().join("sen_extension"), true, &[]).unwrap_err();
    assert!(err.to_string().contains("extensión"), "{err}");
}

#[test]
fn un_avi_dv_de_cinta_con_dous_audios_pasase_a_mp4_co_perfil_minidv() {
    if ffmpeg::ffmpeg_path().is_err() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("Sin título_0001.avi");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=720x576:rate=25:duration=2", "-f", "lavfi", "-i", "sine=frequency=300:sample_rate=32000:duration=2"])
        .args(["-f", "lavfi", "-i", "sine=frequency=700:sample_rate=32000:duration=2", "-map", "0:v", "-map", "1:a", "-map", "2:a", "-c:v", "dvvideo", "-pix_fmt", "yuv420p", "-c:a", "pcm_s16le", "-shortest"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let out = dir.path().join("Sin título_0001-comprimido.mp4");
    let o = compress::VideoCompress { crf: 18, audio_kbps: 192, deinterlace: 2, compat: true, ..Default::default() };
    compress::compress_video(&src, &out, false, &o).unwrap();
    let info = ffmpeg::probe(&out).unwrap();
    assert_eq!(info["streams"][0]["codec_name"], "h264");
    assert!(std::fs::metadata(&out).unwrap().len() > 1000);
}
