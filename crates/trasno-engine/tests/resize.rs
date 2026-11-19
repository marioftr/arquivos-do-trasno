//! Redimensionar imaxes con ficheiros reais: tamaños, formatos, orientación das fotos do móbil, recorte, animacións e que o orixinal non se toque.

use image::{ImageEncoder, Rgb, RgbImage};
use std::path::{Path, PathBuf};
use trasno_engine::image_meta::Metadata;
use trasno_engine::resize::{oriented_dimensions, resize_image, Fit};

fn sha(p: &Path) -> Vec<u8> {
    std::fs::read(p).unwrap()
}

/// Mitade esquerda vermella e dereita azul: serve para ver que fai o recorte e o esticado.
fn halves(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, _| if x < w / 2 { Rgb([230, 20, 20]) } else { Rgb([20, 20, 230]) })
}

fn make(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let p = dir.join(name);
    halves(w, h).save(&p).unwrap();
    p
}

fn dims(p: &Path) -> (u32, u32) {
    image::image_dimensions(p).unwrap()
}

fn within(w: u32, h: u32) -> Fit {
    Fit::Within { width: Some(w), height: Some(h), no_upscale: true }
}

fn run(input: &Path, out: &Path, fit: &Fit) -> trasno_engine::resize::ResizeReport {
    resize_image(input, out, false, fit, Some(90), None, Metadata::default()).unwrap()
}

#[test]
fn cada_formato_encolle_ao_tamaño_pedido_e_non_toca_o_orixinal() {
    let d = tempfile::tempdir().unwrap();
    for ext in ["png", "jpg", "webp", "bmp", "tiff", "gif", "qoi", "tga"] {
        let src = make(d.path(), &format!("foto.{ext}"), 400, 300);
        let before = sha(&src);
        let out = d.path().join(format!("foto-pequena.{ext}"));
        let r = run(&src, &out, &within(100, 100));
        assert_eq!(r.from, (400, 300), "{ext}");
        assert_eq!(r.to, (100, 75), "{ext}");
        assert_eq!(dims(&out), (100, 75), "{ext}: o ficheiro garda o tamaño que di o informe");
        assert_eq!(r.bytes_written, std::fs::metadata(&out).unwrap().len(), "{ext}");
        assert_eq!(sha(&src), before, "{ext}: o orixinal queda igual, byte a byte");
    }
}

#[test]
fn non_amplia_o_que_xa_cabe_e_si_cando_se_permite() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "pequena.png", 80, 60);
    let r = run(&src, &d.path().join("a.png"), &within(1000, 1000));
    assert_eq!(r.to, (80, 60));
    let up = Fit::Within { width: Some(160), height: None, no_upscale: false };
    let r = run(&src, &d.path().join("b.png"), &up);
    assert_eq!(r.to, (160, 120));
    assert_eq!(dims(&d.path().join("b.png")), (160, 120));
    let r = run(&src, &d.path().join("c.png"), &Fit::Percent { percent: 250.0 });
    assert_eq!(r.to, (200, 150));
}

#[test]
fn as_fotos_do_movil_miden_o_que_se_ve() {
    // 400×300 gardada «deitada» con orientación 6: na pantalla é 300×400.
    let d = tempfile::tempdir().unwrap();
    let src = d.path().join("movil.jpg");
    {
        use exif::{Field, In, Tag, Value};
        let mut w = exif::experimental::Writer::new();
        let f = Field { tag: Tag::Orientation, ifd_num: In::PRIMARY, value: Value::Short(vec![6]) };
        w.push_field(&f);
        let mut buf = std::io::Cursor::new(Vec::new());
        w.write(&mut buf, true).unwrap();
        let file = std::io::BufWriter::new(std::fs::File::create(&src).unwrap());
        let mut e = image::codecs::jpeg::JpegEncoder::new_with_quality(file, 92);
        e.set_exif_metadata(buf.into_inner()).unwrap();
        image::DynamicImage::ImageRgb8(halves(400, 300)).write_with_encoder(e).unwrap();
    }
    assert_eq!(oriented_dimensions(&src).unwrap(), (300, 400));
    let out = d.path().join("movil-pequena.jpg");
    let r = run(&src, &out, &within(150, 150));
    assert_eq!(r.from, (300, 400));
    assert_eq!(r.to, (113, 150), "o cadro mídese coa foto de pé, non deitada");
    assert_eq!(dims(&out), (113, 150));
}

#[test]
fn tamaño_exacto_estica_ou_recorta_polo_centro() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "ancha.png", 400, 200);
    // Esticar: 100×100 con toda a imaxe apertada: a metade esquerda vermella e a dereita azul.
    let st = d.path().join("estica.png");
    let r = run(&src, &st, &Fit::Exact { width: 100, height: 100, crop: false, no_upscale: false });
    assert_eq!(r.to, (100, 100));
    let img = image::open(&st).unwrap().to_rgb8();
    assert!(img.get_pixel(5, 50)[0] > 200 && img.get_pixel(95, 50)[2] > 200, "esticar leva a imaxe enteira");
    // Recortar: 100×100 do centro (200×200 de 400×200): aínda hai vermello á esquerda e azul á dereita, pero só a metade de cada un.
    let cr = d.path().join("recorta.png");
    let r = run(&src, &cr, &Fit::Exact { width: 100, height: 100, crop: true, no_upscale: false });
    assert_eq!(r.to, (100, 100));
    assert_eq!(dims(&cr), (100, 100));
    let img = image::open(&cr).unwrap().to_rgb8();
    assert!(img.get_pixel(5, 50)[0] > 200 && img.get_pixel(95, 50)[2] > 200);
    // Recortar unha imaxe máis alta ca ancha: a franxa central, sen deformar (o círculo non se volve óvalo).
    let alta = d.path().join("alta.png");
    RgbImage::from_fn(100, 300, |_, y| if (100..200).contains(&y) { Rgb([10, 200, 10]) } else { Rgb([200, 10, 10]) }).save(&alta).unwrap();
    let out = d.path().join("alta-cadrada.png");
    run(&alta, &out, &Fit::Exact { width: 100, height: 100, crop: true, no_upscale: false });
    let img = image::open(&out).unwrap().to_rgb8();
    assert!(img.pixels().all(|p| p[1] > 150 && p[0] < 60), "o recorte colle o centro, que é todo verde");
}

#[test]
fn os_metadatos_non_se_levan_por_defecto() {
    let d = tempfile::tempdir().unwrap();
    let src = d.path().join("con-exif.jpg");
    {
        use exif::{Field, In, Tag, Value};
        let mut w = exif::experimental::Writer::new();
        let f = Field { tag: Tag::Make, ifd_num: In::PRIMARY, value: Value::Ascii(vec![b"Trasno".to_vec()]) };
        w.push_field(&f);
        let mut buf = std::io::Cursor::new(Vec::new());
        w.write(&mut buf, true).unwrap();
        let file = std::io::BufWriter::new(std::fs::File::create(&src).unwrap());
        let mut e = image::codecs::jpeg::JpegEncoder::new_with_quality(file, 90);
        e.set_exif_metadata(buf.into_inner()).unwrap();
        image::DynamicImage::ImageRgb8(halves(200, 100)).write_with_encoder(e).unwrap();
    }
    let out = d.path().join("sen.jpg");
    resize_image(&src, &out, false, &within(100, 100), Some(90), None, Metadata::Strip).unwrap();
    use image::ImageDecoder;
    let mut dec = image::ImageReader::open(&out).unwrap().into_decoder().unwrap();
    assert!(dec.exif_metadata().unwrap().is_none(), "sen datos da cámara");
}

#[test]
fn unha_animacion_mantense_animada() {
    use image::codecs::gif::{GifEncoder, Repeat};
    use image::{Delay, Frame};
    if !trasno_engine::ffmpeg::is_available() {
        return;
    }
    let d = tempfile::tempdir().unwrap();
    let src = d.path().join("anim.gif");
    {
        let mut e = GifEncoder::new(std::fs::File::create(&src).unwrap());
        e.set_repeat(Repeat::Infinite).unwrap();
        for i in 0..4u8 {
            let f = image::RgbaImage::from_fn(120, 80, |x, _| if (x / 10 + i as u32) % 2 == 0 { image::Rgba([220, 30, 30, 255]) } else { image::Rgba([30, 30, 220, 255]) });
            e.encode_frame(Frame::from_parts(f, 0, 0, Delay::from_numer_denom_ms(100, 1))).unwrap();
        }
    }
    let out = d.path().join("anim-pequena.gif");
    let r = run(&src, &out, &within(60, 60));
    assert_eq!(r.to, (60, 40));
    assert_eq!(dims(&out), (60, 40));
    assert!(trasno_engine::animated::is_animated(&out), "segue sendo animada");
    // Recortar unha animación non se pode: avísase, non se deixa un ficheiro a medias.
    let e = resize_image(&src, &d.path().join("c.gif"), false, &Fit::Exact { width: 50, height: 50, crop: true, no_upscale: false }, None, None, Metadata::default()).unwrap_err();
    assert!(e.to_string().contains("animada"), "{e}");
    assert!(!d.path().join("c.gif").exists());
}

#[test]
fn nunca_pisa_un_ficheiro_nin_o_orixinal() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "foto.png", 100, 100);
    let before = sha(&src);
    // Sobre o propio orixinal: erro, e o ficheiro queda igual.
    assert!(resize_image(&src, &src, true, &within(50, 50), None, None, Metadata::default()).is_err());
    assert_eq!(sha(&src), before);
    // Un destino que xa existe: non se pisa sen permiso.
    let out = d.path().join("xa-existe.png");
    std::fs::write(&out, b"non tocar").unwrap();
    assert!(resize_image(&src, &out, false, &within(50, 50), None, None, Metadata::default()).is_err());
    assert_eq!(std::fs::read(&out).unwrap(), b"non tocar");
    // Un tamaño absurdo: erro claro e nada escrito.
    let big = d.path().join("enorme.png");
    assert!(resize_image(&src, &big, false, &Fit::Exact { width: 29_000, height: 29_000, crop: false, no_upscale: false }, None, None, Metadata::default()).is_err());
    assert!(!big.exists());
    // Unha imaxe estragada: erro e nada escrito.
    let broken = d.path().join("rota.png");
    std::fs::write(&broken, b"isto non e un png").unwrap();
    let o = d.path().join("rota-pequena.png");
    assert!(resize_image(&broken, &o, false, &within(50, 50), None, None, Metadata::default()).is_err());
    assert!(!o.exists());
}

#[test]
fn a_calidade_cambia_o_peso_dun_jpg_e_o_png_non_perde_cores() {
    let d = tempfile::tempdir().unwrap();
    let noisy = d.path().join("ruido.jpg");
    RgbImage::from_fn(600, 400, |x, y| Rgb([((x * 7 + y * 13) % 251) as u8, ((x * 3 + y * 5) % 241) as u8, ((x ^ y) % 256) as u8])).save(&noisy).unwrap();
    let hi = d.path().join("alta.jpg");
    let lo = d.path().join("baixa.jpg");
    resize_image(&noisy, &hi, false, &Fit::Percent { percent: 50.0 }, Some(95), None, Metadata::default()).unwrap();
    resize_image(&noisy, &lo, false, &Fit::Percent { percent: 50.0 }, Some(30), None, Metadata::default()).unwrap();
    assert!(std::fs::metadata(&lo).unwrap().len() < std::fs::metadata(&hi).unwrap().len());
    // PNG con transparencia: a transparencia segue ahí.
    let t = d.path().join("trans.png");
    image::RgbaImage::from_fn(60, 40, |x, _| if x < 30 { image::Rgba([255, 0, 0, 255]) } else { image::Rgba([0, 0, 255, 0]) }).save(&t).unwrap();
    let out = d.path().join("trans-pequena.png");
    resize_image(&t, &out, false, &Fit::Percent { percent: 50.0 }, None, None, Metadata::default()).unwrap();
    let img = image::open(&out).unwrap().to_rgba8();
    assert_eq!(img.get_pixel(2, 10)[3], 255);
    assert_eq!(img.get_pixel(28, 10)[3], 0, "o transparente segue transparente");
}
