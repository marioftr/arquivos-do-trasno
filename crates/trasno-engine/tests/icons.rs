//! Paquete de iconas: ficheiros que se len, tamaños, formas non cadradas e nome do cartafol.

use std::path::{Path, PathBuf};
use trasno_engine::icons::*;

fn make(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let p = dir.join(name);
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(w, h, |x, y| image::Rgba([(x * 255 / w) as u8, (y * 255 / h) as u8, 120, 255]))).save(&p).unwrap();
    p
}
fn all() -> IconOptions {
    IconOptions { windows: true, mac: true, web: true, png: true, fit: false }
}

#[test]
fn crea_todo_e_cada_ficheiro_le() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "logo.png", 600, 600);
    let r = make_icon_pack(&src, d.path(), "logo", &all()).unwrap();
    assert_eq!(r.folder.file_name().unwrap(), "logo-iconas");
    for f in ["icon.ico", "icon.icns", "favicon.ico", "favicon-32x32.png", "apple-touch-icon.png", "android-chrome-512x512.png", "site.webmanifest", "png/icon-16.png", "png/icon-1024.png"] {
        assert!(r.folder.join(f).is_file(), "falta {f}");
    }
    // O ICO ten sete entradas e o último é 256 (0 na cabeceira); as imaxes dentro len.
    let ico = std::fs::read(r.folder.join("icon.ico")).unwrap();
    assert_eq!(&ico[..4], &[0, 0, 1, 0]);
    assert_eq!(u16::from_le_bytes([ico[4], ico[5]]), 7);
    let first_off = u32::from_le_bytes([ico[18], ico[19], ico[20], ico[21]]) as usize;
    let decoded = image::load_from_memory(&ico[first_off..]).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (16, 16));
    // O ICNS empeza por «icns» e o tamaño da cabeceira coincide co ficheiro.
    let icns = std::fs::read(r.folder.join("icon.icns")).unwrap();
    assert_eq!(&icns[..4], b"icns");
    assert_eq!(u32::from_be_bytes([icns[4], icns[5], icns[6], icns[7]]) as usize, icns.len());
    let png = image::open(r.folder.join("png/icon-1024.png")).unwrap();
    assert_eq!((png.width(), png.height()), (1024, 1024));
    assert_eq!(r.source_side, 600);
    // Segunda vez: cartafol novo co « (2)» ao final.
    let r2 = make_icon_pack(&src, d.path(), "logo", &all()).unwrap();
    assert_eq!(r2.folder.file_name().unwrap(), "logo-iconas (2)");
    assert!(r.folder.join("icon.ico").is_file(), "o primeiro non se tocou");
}

#[test]
fn imaxe_non_cadrada_recorta_ou_encaixa() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "ancha.png", 400, 200);
    let only_png = |fit| IconOptions { windows: false, mac: false, web: false, png: true, fit };
    let cut = make_icon_pack(&src, d.path(), "cortada", &only_png(false)).unwrap();
    assert_eq!(cut.source_side, 200, "recorta ao lado menor");
    let c = image::open(cut.folder.join("png/icon-16.png")).unwrap().to_rgba8();
    assert_eq!(c.get_pixel(8, 8)[3], 255, "sen transparencia");
    let fit = make_icon_pack(&src, d.path(), "encaixada", &only_png(true)).unwrap();
    assert_eq!(fit.source_side, 400, "encaixa ao lado maior");
    let f = image::open(fit.folder.join("png/icon-64.png")).unwrap().to_rgba8();
    assert_eq!(f.get_pixel(32, 1)[3], 0, "marxe transparente arriba");
    assert_eq!(f.get_pixel(32, 32)[3], 255, "a imaxe no medio");
}

#[test]
fn erros_claros_e_nada_a_medias() {
    let d = tempfile::tempdir().unwrap();
    let src = make(d.path(), "p.png", 10, 10);
    assert!(make_icon_pack(&src, d.path(), "p", &all()).is_err(), "menos de 16 px");
    let ok = make(d.path(), "ok.png", 64, 64);
    assert!(make_icon_pack(&ok, d.path(), "ok", &IconOptions { windows: false, mac: false, web: false, png: false, fit: false }).is_err());
    assert!(make_icon_pack(&d.path().join("non.png"), d.path(), "x", &all()).is_err());
    let leftovers = std::fs::read_dir(d.path()).unwrap().filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().starts_with(".trasno")).count();
    assert_eq!(leftovers, 0);
}
