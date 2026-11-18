//! Cortar en partes iguais con ficheiros reais: tamaños, píxeles exactos, numeración, nomes sen pisar e límites.

use image::{Rgb, RgbImage};
use std::path::{Path, PathBuf};
use trasno_engine::slice::{part_number, piece_size, plan_outputs, slice_image, SliceOptions};

/// Imaxe na que cada píxel di onde está: así se comproba que cada parte é exactamente o seu anaco.
fn grid_image(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| Rgb([(x % 256) as u8, (y % 256) as u8, ((x / 256) * 40 + (y / 256) * 90) as u8]))
}

fn make(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let p = dir.join(name);
    grid_image(w, h).save(&p).unwrap();
    p
}

fn opts(rect: (u32, u32, u32, u32), cols: u32, rows: u32) -> SliceOptions {
    SliceOptions { rect, cols, rows, by_columns: false, quality: Some(92), background: [255, 255, 255] }
}

#[test]
fn tres_cadrados_dun_panorama_saen_exactos() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "pano.png", 900, 300);
    let before = std::fs::read(&src).unwrap();
    let r = slice_image(&src, d.path(), "pano", "png", &opts((0, 0, 900, 300), 3, 1)).unwrap();
    assert_eq!(r.piece, (300, 300));
    assert_eq!(r.outputs.len(), 3);
    let whole = grid_image(900, 300);
    for (i, p) in r.outputs.iter().enumerate() {
        assert_eq!(p.file_name().unwrap().to_str().unwrap(), format!("pano-{}.png", i + 1));
        let img = image::open(p).unwrap().to_rgb8();
        assert_eq!(img.dimensions(), (300, 300));
        for (x, y, px) in img.enumerate_pixels() {
            assert_eq!(px, whole.get_pixel(i as u32 * 300 + x, y), "parte {} en ({x},{y})", i + 1);
        }
    }
    assert_eq!(std::fs::read(&src).unwrap(), before, "o orixinal queda igual, byte a byte");
}

#[test]
fn unha_zona_dentro_da_imaxe_e_o_resto_non_se_corta() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "foto.png", 1000, 800);
    let r = slice_image(&src, d.path(), "foto", "png", &opts((100, 50, 600, 400), 3, 2)).unwrap();
    assert_eq!(r.piece, (200, 200));
    assert_eq!(r.outputs.len(), 6);
    let whole = grid_image(1000, 800);
    // A parte 5 (fila 2, columna 2) empeza en (100 + 200, 50 + 200).
    let img = image::open(&r.outputs[4]).unwrap().to_rgb8();
    assert_eq!(img.get_pixel(0, 0), whole.get_pixel(300, 250));
    assert_eq!(img.get_pixel(199, 199), whole.get_pixel(499, 449));
}

#[test]
fn o_que_sobra_menos_dun_pixel_por_parte_queda_fora() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "foto.png", 1000, 500);
    let r = slice_image(&src, d.path(), "foto", "png", &opts((0, 0, 1000, 500), 3, 1)).unwrap();
    assert_eq!(r.piece, (333, 500));
}

#[test]
fn numerar_por_columnas_cambia_a_orde_dos_ficheiros() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "foto.png", 600, 400);
    let mut o = opts((0, 0, 600, 400), 3, 2);
    o.by_columns = true;
    let r = slice_image(&src, d.path(), "foto", "png", &o).unwrap();
    let whole = grid_image(600, 400);
    // Por columnas: 1 e 2 son a primeira columna (arriba e abaixo), 3 e 4 a segunda…
    let second = image::open(&r.outputs[1]).unwrap().to_rgb8();
    assert_eq!(second.get_pixel(0, 0), whole.get_pixel(0, 200));
    let third = image::open(&r.outputs[2]).unwrap().to_rgb8();
    assert_eq!(third.get_pixel(0, 0), whole.get_pixel(200, 0));
    assert_eq!(part_number(2, 1, 3, 2, true), 6);
    assert_eq!(part_number(2, 1, 3, 2, false), 6);
    assert_eq!(part_number(1, 0, 3, 2, true), 3);
    assert_eq!(part_number(1, 0, 3, 2, false), 2);
}

#[test]
fn con_mais_de_nove_partes_os_nomes_levan_ceros() {
    let d = tempfile::tempdir().unwrap();
    let names = plan_outputs(d.path(), "a", "png", 12).unwrap();
    assert_eq!(names[0].file_name().unwrap().to_str().unwrap(), "a-01.png");
    assert_eq!(names[11].file_name().unwrap().to_str().unwrap(), "a-12.png");
}

#[test]
fn nunca_pisa_e_o_xogo_enteiro_pasa_a_outro_nome() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "foto.png", 600, 200);
    let first = slice_image(&src, d.path(), "foto", "png", &opts((0, 0, 600, 200), 3, 1)).unwrap();
    let kept: Vec<Vec<u8>> = first.outputs.iter().map(|p| std::fs::read(p).unwrap()).collect();
    let second = slice_image(&src, d.path(), "foto", "png", &opts((0, 0, 600, 200), 3, 1)).unwrap();
    assert_eq!(second.outputs[0].file_name().unwrap().to_str().unwrap(), "foto (2)-1.png");
    for (p, bytes) in first.outputs.iter().zip(kept) {
        assert_eq!(std::fs::read(p).unwrap(), bytes, "as partes anteriores non se tocan");
    }
    // Se só existe a parte 2, tamén pasa todo o xogo a outro nome (non se mestura).
    std::fs::remove_file(&first.outputs[0]).unwrap();
    std::fs::remove_file(&first.outputs[2]).unwrap();
    let third = slice_image(&src, d.path(), "foto", "png", &opts((0, 0, 600, 200), 3, 1)).unwrap();
    assert_eq!(third.outputs[0].file_name().unwrap().to_str().unwrap(), "foto (3)-1.png");
}

#[test]
fn cada_formato_de_saida_garda_partes_que_se_len() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "foto.png", 400, 200);
    for ext in ["png", "jpg", "webp", "bmp", "tiff"] {
        let r = slice_image(&src, d.path(), &format!("saida-{ext}"), ext, &opts((0, 0, 400, 200), 2, 1)).unwrap();
        for p in &r.outputs {
            assert_eq!(image::image_dimensions(p).unwrap(), (200, 200), "{ext}");
        }
    }
}

#[test]
fn rexeita_o_que_non_ten_sentido_e_non_deixa_partes_soltas() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "foto.png", 400, 200);
    let count = || std::fs::read_dir(d.path()).unwrap().count();
    let base = count();
    // Zona fóra da imaxe.
    assert!(slice_image(&src, d.path(), "x", "png", &opts((300, 0, 200, 200), 2, 1)).is_err());
    // Demasiadas partes, e partes de menos dun píxel.
    assert!(slice_image(&src, d.path(), "x", "png", &opts((0, 0, 400, 200), 21, 1)).is_err());
    assert!(slice_image(&src, d.path(), "x", "png", &opts((0, 0, 400, 200), 20, 6)).is_err());
    assert!(piece_size((0, 0, 3, 3), 4, 1).is_err());
    assert!(piece_size((0, 0, 100, 100), 0, 1).is_err());
    // Formato que non se sabe escribir aquí.
    assert!(slice_image(&src, d.path(), "x", "avif", &opts((0, 0, 400, 200), 2, 1)).is_err());
    assert!(slice_image(&src, d.path(), "x", "xyz", &opts((0, 0, 400, 200), 2, 1)).is_err());
    // Orixe que non existe.
    assert!(slice_image(&d.path().join("non-hai.png"), d.path(), "x", "png", &opts((0, 0, 400, 200), 2, 1)).is_err());
    assert_eq!(count(), base, "ningún fallo deixa ficheiros");
}

#[test]
fn unha_foto_xirada_corta_sobre_a_imaxe_tal_como_se_ve() {
    // Unha JPG de 400×200 con orientación 6 (xirar 90° horario) vese como 200×400: a zona vai sobre esas medidas.
    let d = tempfile::tempdir().unwrap();
    let src = d.path().join("movil.jpg");
    let img = RgbImage::from_fn(400, 200, |x, _| if x < 200 { Rgb([230, 20, 20]) } else { Rgb([20, 20, 230]) });
    let mut buf = Vec::new();
    {
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 95);
        enc.encode_image(&img).unwrap();
    }
    // Insire un segmento EXIF mínimo (orientación 6) tras o SOI.
    let exif: Vec<u8> = vec![
        0xFF, 0xE1, 0x00, 0x22, b'E', b'x', b'i', b'f', 0, 0, b'M', b'M', 0, 0x2A, 0, 0, 0, 8, 0, 1, 0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, 6, 0, 0, 0, 0, 0, 0,
    ];
    let mut out = vec![0xFF, 0xD8];
    out.extend(exif);
    out.extend(&buf[2..]);
    std::fs::write(&src, out).unwrap();
    assert_eq!(trasno_engine::resize::oriented_dimensions(&src).unwrap(), (200, 400));
    let r = slice_image(&src, d.path(), "movil", "png", &opts((0, 0, 200, 400), 1, 2)).unwrap();
    assert_eq!(r.piece, (200, 200));
    // Xirada 90° horario: a metade esquerda vermella pasa arriba.
    let top = image::open(&r.outputs[0]).unwrap().to_rgb8();
    let bottom = image::open(&r.outputs[1]).unwrap().to_rgb8();
    assert!(top.get_pixel(100, 100)[0] > 150, "arriba vermello");
    assert!(bottom.get_pixel(100, 100)[2] > 150, "abaixo azul");
}
