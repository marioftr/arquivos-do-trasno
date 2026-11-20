//! Xuntar con ficheiros reais: capturas de desprazamento con cabeceira e pé fixos, sen solapamento, de lado, con anchos distintos,
//! e que nunca se toquen os orixinais.

use image::{Rgb, RgbImage};
use std::path::{Path, PathBuf};
use trasno_engine::stitch::{stitch_images, Align, Direction, StitchOptions};

/// Páxina «larga» con contido que non se repite: bloques de cores e liñas dependentes da fila, como un texto.
fn page(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| {
        let block = (y / 7) as u64;
        let seed = block.wrapping_mul(2654435761).wrapping_add((x as u64 / 9).wrapping_mul(40503)) ^ (block << 7);
        let v = (seed % 251) as u8;
        let line = if y % 7 == 0 { 40 } else { 0 };
        Rgb([v.saturating_sub(line), (v / 2).wrapping_add((y / 70) as u8), ((x / 9) as u8).wrapping_mul(7).wrapping_add(v / 3)])
    })
}

/// Captura de desprazamento: cabeceira fixa, `view` filas da páxina desde `off` e pé fixo.
fn shot(page: &RgbImage, off: u32, view: u32, head: &RgbImage, foot: &RgbImage) -> RgbImage {
    let (w, hh, fh) = (page.width(), head.height(), foot.height());
    RgbImage::from_fn(w, hh + view + fh, |x, y| {
        if y < hh {
            *head.get_pixel(x, y)
        } else if y < hh + view {
            *page.get_pixel(x, off + y - hh)
        } else {
            *foot.get_pixel(x, y - hh - view)
        }
    })
}

fn save(dir: &Path, name: &str, img: &RgbImage) -> PathBuf {
    let p = dir.join(name);
    img.save(&p).unwrap();
    p
}

fn opts() -> StitchOptions {
    StitchOptions { quality: None, ..Default::default() }
}

fn sha(p: &Path) -> Vec<u8> {
    std::fs::read(p).unwrap()
}

#[test]
fn capturas_de_desprazamento_con_cabeceira_e_pe_fixos_quedan_como_a_paxina() {
    let d = tempfile::tempdir().unwrap();
    let (w, view, hh, fh) = (480, 840, 90, 70);
    let pg = page(w, 3100);
    let head = RgbImage::from_fn(w, hh, |x, y| Rgb([30 + (x % 40) as u8, 60, 200 - (y % 50) as u8]));
    let foot = RgbImage::from_fn(w, fh, |x, y| Rgb([200, 30 + (y % 40) as u8, (x % 60) as u8]));
    // Desprazamentos de 700 filas con 840 visibles: se solapan 140.
    let offs = [0u32, 700, 1400, 2100, 2260];
    let shots: Vec<PathBuf> = offs.iter().enumerate().map(|(i, o)| save(d.path(), &format!("c{i}.png"), &shot(&pg, *o, view, &head, &foot))).collect();
    let before: Vec<_> = shots.iter().map(|p| sha(p)).collect();
    let out = d.path().join("larga.png");
    let r = stitch_images(&shots, &out, false, &opts()).unwrap();
    assert!(r.joins.iter().all(|j| j.is_some()), "atopou o solapamento en todas as unións: {:?}", r.joins);
    // Esperado: cabeceira + páxina ata a última fila vista + pé.
    let last = 2260 + view;
    let want = {
        let mut img = RgbImage::new(w, hh + last + fh);
        for y in 0..hh + last + fh {
            for x in 0..w {
                let px = if y < hh { *head.get_pixel(x, y) } else if y < hh + last { *pg.get_pixel(x, y - hh) } else { *foot.get_pixel(x, y - hh - last) };
                img.put_pixel(x, y, px);
            }
        }
        img
    };
    assert_eq!(r.size, (w, want.height()), "a altura é a da páxina, sen filas repetidas nin cabeceiras no medio");
    let got = image::open(&out).unwrap().to_rgb8();
    assert_eq!(got.as_raw(), want.as_raw(), "píxel a píxel igual á páxina orixinal");
    for (p, b) in shots.iter().zip(before) {
        assert_eq!(&sha(p), &b, "as capturas orixinais non se tocan");
    }
}

#[test]
fn sen_quitar_o_repetido_vai_todo_tal_cal_cun_espazo() {
    let d = tempfile::tempdir().unwrap();
    let pg = page(300, 1000);
    let a = save(d.path(), "a.png", &shot(&pg, 0, 400, &RgbImage::new(300, 0), &RgbImage::new(300, 0)));
    let b = save(d.path(), "b.png", &shot(&pg, 300, 400, &RgbImage::new(300, 0), &RgbImage::new(300, 0)));
    let out = d.path().join("x.png");
    let o = StitchOptions { remove_overlap: false, gap: 10, background: [10, 20, 30], quality: None, ..Default::default() };
    let r = stitch_images(&[a, b], &out, false, &o).unwrap();
    assert_eq!(r.size, (300, 810));
    assert_eq!(r.joins, vec![None]);
    let img = image::open(&out).unwrap().to_rgb8();
    assert_eq!(*img.get_pixel(5, 405), Rgb([10, 20, 30]), "o espazo leva o fondo");
    assert_eq!(*img.get_pixel(5, 410), *pg.get_pixel(5, 300));
}

#[test]
fn capturas_sen_relacion_xúntanse_sen_inventar_solapamentos() {
    let d = tempfile::tempdir().unwrap();
    let a = save(d.path(), "a.png", &page(300, 500));
    let b = save(d.path(), "b.png", &RgbImage::from_fn(300, 400, |x, y| Rgb([(x * 3 % 255) as u8, 200, (y * 5 % 255) as u8])));
    let out = d.path().join("x.png");
    let r = stitch_images(&[a, b], &out, false, &opts()).unwrap();
    assert_eq!(r.joins, vec![None], "se non hai onde coincidan, unense tal cal");
    assert_eq!(r.size, (300, 900));
}

#[test]
fn fondos_lisos_non_se_confunden_co_solapamento() {
    let d = tempfile::tempdir().unwrap();
    let a = save(d.path(), "a.png", &RgbImage::from_pixel(300, 500, Rgb([255, 255, 255])));
    let b = save(d.path(), "b.png", &RgbImage::from_pixel(300, 500, Rgb([255, 255, 255])));
    let out = d.path().join("x.png");
    let r = stitch_images(&[a, b], &out, false, &opts()).unwrap();
    assert_eq!(r.joins, vec![None]);
    assert_eq!(r.size, (300, 1000));
}

#[test]
fn de_lado_a_lado_tamén_quita_o_repetido() {
    let d = tempfile::tempdir().unwrap();
    // Páxina ancha de 1500×300, vista en dúas fiestras de 900 con 300 de solapamento.
    let tall = page(300, 1500);
    let wide = RgbImage::from_fn(1500, 300, |x, y| *tall.get_pixel(y, x));
    let view = |off: u32| RgbImage::from_fn(900, 300, |x, y| *wide.get_pixel(off + x, y));
    let a = save(d.path(), "a.png", &view(0));
    let b = save(d.path(), "b.png", &view(600));
    let out = d.path().join("ancha.png");
    let o = StitchOptions { direction: Direction::Horizontal, quality: None, ..Default::default() };
    let r = stitch_images(&[a, b], &out, false, &o).unwrap();
    assert_eq!(r.size, (1500, 300));
    assert!(r.joins[0].is_some());
    assert_eq!(image::open(&out).unwrap().to_rgb8().as_raw(), wide.as_raw());
}

#[test]
fn anchos_distintos_igualanse_ao_estreito_ou_ao_ancho() {
    let d = tempfile::tempdir().unwrap();
    let a = save(d.path(), "a.png", &page(400, 300));
    let b = save(d.path(), "b.png", &page(200, 100));
    let out = d.path().join("x.png");
    let r = stitch_images(&[a.clone(), b.clone()], &out, false, &StitchOptions { remove_overlap: false, quality: None, ..Default::default() }).unwrap();
    assert_eq!(r.size, (200, 150 + 100), "ao máis estreito: a primeira encolle a 200×150");
    let out2 = d.path().join("y.png");
    let r = stitch_images(&[a, b], &out2, false, &StitchOptions { align: Align::Widest, remove_overlap: false, quality: None, ..Default::default() }).unwrap();
    assert_eq!(r.size, (400, 300 + 200), "ao máis ancho: a segunda amplíase a 400×200");
}

#[test]
fn varios_formatos_de_entrada_e_de_saida() {
    let d = tempfile::tempdir().unwrap();
    let pg = page(200, 600);
    let a = d.path().join("a.jpg");
    let b = d.path().join("b.webp");
    let c = d.path().join("c.bmp");
    image::DynamicImage::ImageRgb8(shot(&pg, 0, 250, &RgbImage::new(200, 0), &RgbImage::new(200, 0))).save(&a).unwrap();
    image::DynamicImage::ImageRgb8(shot(&pg, 200, 250, &RgbImage::new(200, 0), &RgbImage::new(200, 0))).save(&b).unwrap();
    image::DynamicImage::ImageRgb8(shot(&pg, 400, 200, &RgbImage::new(200, 0), &RgbImage::new(200, 0))).save(&c).unwrap();
    for ext in ["png", "jpg", "webp", "bmp", "tiff"] {
        let out = d.path().join(format!("todo.{ext}"));
        let r = stitch_images(&[a.clone(), b.clone(), c.clone()], &out, false, &StitchOptions { quality: Some(90), ..Default::default() }).unwrap();
        assert_eq!(image::image_dimensions(&out).unwrap(), r.size, "{ext}");
        assert_eq!(r.size.0, 200, "{ext}");
        // Co JPG de entrada hai un pouco de ruído, pero o solapamento atópase igual.
        assert!(r.joins.iter().all(|j| j.is_some()), "{ext}: {:?}", r.joins);
        assert!(r.size.1 >= 590 && r.size.1 <= 610, "{ext}: {:?}", r.size);
    }
}

#[test]
fn rexeita_o_que_non_vale_e_non_deixa_nada_a_medias() {
    let d = tempfile::tempdir().unwrap();
    let a = save(d.path(), "a.png", &page(100, 100));
    let b = save(d.path(), "b.png", &page(100, 100));
    let out = d.path().join("x.png");
    // Unha soa imaxe.
    assert!(stitch_images(&[a.clone()], &out, false, &opts()).is_err());
    // Sobre un dos orixinais.
    let before = sha(&a);
    assert!(stitch_images(&[a.clone(), b.clone()], &a, true, &opts()).is_err());
    assert_eq!(sha(&a), before);
    // Destino que existe.
    std::fs::write(&out, b"non tocar").unwrap();
    assert!(stitch_images(&[a.clone(), b.clone()], &out, false, &opts()).is_err());
    assert_eq!(std::fs::read(&out).unwrap(), b"non tocar");
    // Formato de saída que non vale.
    assert!(stitch_images(&[a.clone(), b.clone()], &d.path().join("x.gif"), false, &opts()).is_err());
    assert!(stitch_images(&[a.clone(), b.clone()], &d.path().join("x.avif"), false, &opts()).is_err());
    // Unha imaxe estragada: erro e sen ficheiro.
    let rota = d.path().join("rota.png");
    std::fs::write(&rota, b"isto non e un png").unwrap();
    let o2 = d.path().join("y.png");
    assert!(stitch_images(&[a.clone(), rota], &o2, false, &opts()).is_err());
    assert!(!o2.exists());
    // Demasiado grande para WebP.
    let tall = save(d.path(), "alta.png", &RgbImage::new(20, 9000));
    let o3 = d.path().join("z.webp");
    let e = stitch_images(&[tall.clone(), tall], &o3, false, &opts()).unwrap_err();
    assert!(e.to_string().contains("demasiado grande"), "{e}");
    assert!(!o3.exists());
}
