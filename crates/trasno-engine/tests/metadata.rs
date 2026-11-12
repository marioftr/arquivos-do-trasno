//! Metadatos das fotos ao converter e comprimir: por defecto non se levan; opcionalmente van sen ubicación ou completos.
//! O perfil de cor consérvase sempre e a orientación aplícase aos píxeles.

use image::{ImageDecoder, ImageEncoder};
use std::path::{Path, PathBuf};
use trasno_engine::compress::{compress_image, ImageCompress};
use trasno_engine::image_meta::Metadata;
use trasno_engine::{convert_file, ConvertOptions};

/// EXIF con cámara, data, ubicación e orientación (6 = hai que xirar a foto 90° para velo ben).
fn exif_bytes(orientation: u16) -> Vec<u8> {
    use exif::{Field, In, Rational, Tag, Value};
    let ascii = |s: &str| Value::Ascii(vec![s.as_bytes().to_vec()]);
    let fields = vec![
        Field { tag: Tag::Make, ifd_num: In::PRIMARY, value: ascii("Trasno") },
        Field { tag: Tag::Model, ifd_num: In::PRIMARY, value: ascii("Proba 1") },
        Field { tag: Tag::Orientation, ifd_num: In::PRIMARY, value: Value::Short(vec![orientation]) },
        Field { tag: Tag::DateTimeOriginal, ifd_num: In::PRIMARY, value: ascii("2024:05:06 07:08:09") },
        Field { tag: Tag::GPSLatitudeRef, ifd_num: In::PRIMARY, value: ascii("N") },
        Field { tag: Tag::GPSLatitude, ifd_num: In::PRIMARY, value: Value::Rational(vec![Rational { num: 42, denom: 1 }, Rational { num: 52, denom: 1 }, Rational { num: 0, denom: 1 }]) },
    ];
    let mut w = exif::experimental::Writer::new();
    for f in &fields {
        w.push_field(f);
    }
    let mut buf = std::io::Cursor::new(Vec::new());
    w.write(&mut buf, true).unwrap();
    buf.into_inner()
}

/// Perfil «RGB » falso pero con cabeceira de tamaño razoable: abonda para comprobar que viaxa.
fn fake_icc(space: &[u8; 4]) -> Vec<u8> {
    let mut p = vec![0u8; 300];
    p[16..20].copy_from_slice(space);
    p[36..40].copy_from_slice(b"acsp");
    p
}

fn picture(w: u32, h: u32) -> image::DynamicImage {
    image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x * 255 / w) as u8, (y * 255 / h) as u8, 120])))
}

/// Foto «de móbil»: 40×20 con orientación 6, cámara, data, ubicación e perfil de cor.
fn photo(dir: &Path, ext: &str, icc: &[u8; 4]) -> PathBuf {
    let path = dir.join(format!("foto.{ext}"));
    let img = picture(40, 20);
    let file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
    match ext {
        "jpg" => {
            let mut e = image::codecs::jpeg::JpegEncoder::new_with_quality(file, 90);
            e.set_exif_metadata(exif_bytes(6)).unwrap();
            e.set_icc_profile(fake_icc(icc)).unwrap();
            img.write_with_encoder(e).unwrap();
        }
        "png" => {
            let mut e = image::codecs::png::PngEncoder::new(file);
            e.set_exif_metadata(exif_bytes(6)).unwrap();
            e.set_icc_profile(fake_icc(icc)).unwrap();
            img.write_with_encoder(e).unwrap();
        }
        _ => unreachable!(),
    }
    path
}

fn exif_tags(path: &Path) -> Option<Vec<String>> {
    let mut r = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let e = exif::Reader::new().read_from_container(&mut r).ok()?;
    Some(e.fields().map(|f| f.tag.to_string()).collect())
}

fn orientation(path: &Path) -> Option<u32> {
    let mut r = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let e = exif::Reader::new().read_from_container(&mut r).ok()?;
    e.get_field(exif::Tag::Orientation, exif::In::PRIMARY).and_then(|f| f.value.get_uint(0))
}

fn icc_of(path: &Path) -> Option<Vec<u8>> {
    let mut dec = image::ImageReader::open(path).unwrap().with_guessed_format().unwrap().into_decoder().unwrap();
    dec.icc_profile().unwrap()
}

fn convert(src: &Path, out: &Path, mode: Metadata, quality: Option<u8>) {
    let o = ConvertOptions { quality, metadata: mode, ..Default::default() };
    convert_file(src, out, &o).unwrap_or_else(|e| panic!("{} -> {}: {e}", src.display(), out.display()));
}

fn check(out: &Path, mode: Metadata, icc: bool) {
    let name = out.file_name().unwrap().to_string_lossy().to_string();
    let tags = exif_tags(out);
    match mode {
        Metadata::Strip => assert!(tags.is_none(), "{name}: sen metadatos non debe levar EXIF: {tags:?}"),
        Metadata::NoLocation => {
            let t = tags.unwrap_or_else(|| panic!("{name}: faltan os metadatos"));
            assert!(t.iter().any(|x| x == "Make") && t.iter().any(|x| x == "DateTimeOriginal"), "{name}: {t:?}");
            assert!(!t.iter().any(|x| x.starts_with("GPS")), "{name}: a ubicación non debe ir: {t:?}");
        }
        Metadata::All => {
            let t = tags.unwrap_or_else(|| panic!("{name}: faltan os metadatos"));
            assert!(t.iter().any(|x| x == "GPSLatitude"), "{name}: con «todo» vai a ubicación: {t:?}");
        }
    }
    if mode != Metadata::Strip {
        assert_eq!(orientation(out), Some(1), "{name}: os píxeles xa están xirados, a etiqueta ten que ser 1");
    }
    assert_eq!(icc_of(out).is_some(), icc, "{name}: perfil de cor");
    // Sempre co lado longo en vertical: a foto estaba deitada con orientación 6.
    let (w, h) = image::image_dimensions(out).unwrap();
    assert_eq!((w, h), (20, 40), "{name}: a orientación aplícase aos píxeles");
}

#[test]
fn converter_leva_os_metadatos_que_se_piden_e_sempre_o_perfil_de_cor() {
    for src_ext in ["jpg", "png"] {
        let dir = tempfile::tempdir().unwrap();
        let src = photo(dir.path(), src_ext, b"RGB ");
        for mode in [Metadata::Strip, Metadata::NoLocation, Metadata::All] {
            for (ext, quality) in [("jpg", Some(85)), ("jpg", None), ("png", None), ("webp", None), ("webp", Some(85))] {
                // O WebP con perdas escríbeo FFmpeg.
                if ext == "webp" && quality.is_some() && !trasno_engine::ffmpeg::is_available() {
                    continue;
                }
                let out = dir.path().join(format!("{mode:?}-{}.{ext}", quality.map_or("d".into(), |q: u8| q.to_string())));
                convert(&src, &out, mode, quality);
                check(&out, mode, true);
            }
        }
        // O orixinal non se toca.
        assert!(exif_tags(&src).unwrap().iter().any(|x| x == "GPSLatitude"));
    }
}

#[test]
fn un_perfil_que_non_e_rgb_non_se_leva() {
    let dir = tempfile::tempdir().unwrap();
    let src = photo(dir.path(), "jpg", b"CMYK");
    let out = dir.path().join("o.png");
    convert(&src, &out, Metadata::Strip, None);
    assert!(icc_of(&out).is_none(), "un perfil de CMYK sobre píxeles RGB deixaría a foto mal");
    assert_eq!(image::image_dimensions(&out).unwrap(), (20, 40));
}

#[test]
fn comprimir_leva_os_metadatos_que_se_piden_e_xira_a_foto() {
    for src_ext in ["jpg", "png"] {
        let dir = tempfile::tempdir().unwrap();
        let src = photo(dir.path(), src_ext, b"RGB ");
        for mode in [Metadata::Strip, Metadata::NoLocation, Metadata::All] {
            // JPG, PNG sen perdas (calidade 100) e PNG con perdas (paleta): todos levan o mesmo.
            for (ext, quality) in [("jpg", 70u8), ("png", 100), ("png", 60), ("webp", 70)] {
                if ext == "webp" && !trasno_engine::ffmpeg::is_available() {
                    continue;
                }
                let out = dir.path().join(format!("c-{mode:?}-{quality}.{ext}"));
                compress_image(&src, &out, false, &ImageCompress { quality: Some(quality), metadata: mode, ..Default::default() }).unwrap_or_else(|e| panic!("{src_ext}->{ext} {quality}: {e}"));
                check(&out, mode, true);
            }
            // E co tamaño á forza: os metadatos contan para o tope.
            let out = dir.path().join(format!("t-{mode:?}.jpg"));
            let r = compress_image(&src, &out, false, &ImageCompress { target_bytes: Some(3000), metadata: mode, ..Default::default() }).unwrap();
            assert!(r.bytes_written <= 3000, "{} bytes", r.bytes_written);
            check_oriented(&out);
        }
    }
}

fn check_oriented(out: &Path) {
    let (w, h) = image::image_dimensions(out).unwrap();
    assert!(h > w, "a foto ten que quedar en vertical: {w}x{h}");
}

#[test]
fn un_exif_roto_no_orixinal_non_fai_fallar_nada() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rota.jpg");
    let file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
    let mut e = image::codecs::jpeg::JpegEncoder::new_with_quality(file, 90);
    e.set_exif_metadata(b"isto non e un exif".to_vec()).unwrap();
    picture(32, 16).write_with_encoder(e).unwrap();
    let out = dir.path().join("o.jpg");
    convert(&path, &out, Metadata::All, Some(80));
    assert!(exif_tags(&out).is_none());
    assert_eq!(image::image_dimensions(&out).unwrap(), (32, 16));
    let out = dir.path().join("c.jpg");
    compress_image(&path, &out, false, &ImageCompress { quality: Some(80), metadata: Metadata::NoLocation, ..Default::default() }).unwrap();
    assert!(image::open(&out).is_ok());
}

#[test]
fn sen_pedir_nada_o_resultado_non_cambia_de_tamano_nin_de_formato() {
    // Regresión: o comportamento por defecto segue sendo «sen metadatos» e a foto sae igual.
    let dir = tempfile::tempdir().unwrap();
    let src = photo(dir.path(), "jpg", b"RGB ");
    let out = dir.path().join("d.png");
    convert_file(&src, &out, &ConvertOptions::default()).unwrap();
    assert!(exif_tags(&out).is_none());
    assert_eq!(image::image_dimensions(&out).unwrap(), (20, 40));
}

#[test]
fn webp_con_perdas_e_transparencia_leva_metadatos_sen_perder_o_alfa() {
    if !trasno_engine::ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("alfa.png");
    {
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(40, 20, |x, _| image::Rgba([200, 50, 50, if x < 20 { 0 } else { 255 }])));
        let mut e = image::codecs::png::PngEncoder::new(std::io::BufWriter::new(std::fs::File::create(&src).unwrap()));
        e.set_exif_metadata(exif_bytes(1)).unwrap();
        e.set_icc_profile(fake_icc(b"RGB ")).unwrap();
        img.write_with_encoder(e).unwrap();
    }
    let out = dir.path().join("alfa.webp");
    convert(&src, &out, Metadata::NoLocation, Some(80));
    let t = exif_tags(&out).expect("o WebP leva EXIF");
    assert!(t.iter().any(|x| x == "Make") && !t.iter().any(|x| x.starts_with("GPS")), "{t:?}");
    assert!(icc_of(&out).is_some());
    let back = image::open(&out).unwrap().to_rgba8();
    assert_eq!(back.dimensions(), (40, 20));
    assert_eq!(back.get_pixel(2, 5)[3], 0, "a parte transparente segue transparente");
    assert_eq!(back.get_pixel(30, 5)[3], 255);
    // O contedor segue sendo un WebP válido: o tamaño do RIFF cadra co ficheiro.
    let raw = std::fs::read(&out).unwrap();
    assert_eq!(u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]) as usize, raw.len() - 8);
}
