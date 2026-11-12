//! Capa de texto invisible en PDF dixitalizados: a imaxe queda intacta e o texto pódese buscar e copiar.

use lopdf::{Document, Object};
use std::path::{Path, PathBuf};
use trasno_engine::pdf::images_to_pdf;
use trasno_engine::pdf_tools::{add_text_layer, OcrPage, OcrWord};

fn scan(dir: &Path, pages: usize) -> PathBuf {
    let mut imgs = Vec::new();
    for i in 0..pages {
        let p = dir.join(format!("p{i}.png"));
        image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(200, 300, |x, y| image::Rgb([(x + i as u32 * 40) as u8, y as u8, 200]))).save(&p).unwrap();
        imgs.push(p);
    }
    let out = dir.join("scan.pdf");
    images_to_pdf(&imgs, &out, false).unwrap();
    out
}

fn word(text: &str, x: f32, y: f32, w: f32, h: f32) -> OcrWord {
    OcrWord { text: text.into(), x, y, w, h, angle: 0.0 }
}

/// Bytes da imaxe de cada páxina (para comprobar que non se tocan).
fn images(doc: &Document) -> Vec<Vec<u8>> {
    doc.objects.values().filter_map(|o| if let Object::Stream(s) = o { if s.dict.get(b"Subtype").ok().and_then(|v| v.as_name().ok()) == Some(b"Image") { Some(s.content.clone()) } else { None } } else { None }).collect()
}

#[test]
fn engade_o_texto_sen_tocar_a_imaxe_e_con_todos_os_caracteres() {
    let dir = tempfile::tempdir().unwrap();
    let src = scan(dir.path(), 3);
    let before = Document::load(&src).unwrap();
    let pages = vec![
        OcrPage { page: 1, words: vec![word("Olá", 20.0, 200.0, 40.0, 12.0), word("mundo,", 70.0, 200.0, 60.0, 12.0), word("niño", 20.0, 180.0, 35.0, 12.0)] },
        OcrPage { page: 3, words: vec![word("€100", 20.0, 200.0, 40.0, 12.0), word("😀x", 70.0, 200.0, 30.0, 12.0), word("   ", 1.0, 1.0, 5.0, 5.0)] },
    ];
    let out = dir.path().join("ocr.pdf");
    let r = add_text_layer(&src, &out, false, &pages).unwrap();
    assert_eq!((r.pages, r.words), (2, 5), "o que só ten espazos non conta");

    let after = Document::load(&out).unwrap();
    assert_eq!(after.get_pages().len(), 3, "mesmas páxinas");
    assert_eq!(images(&after), images(&before), "a imaxe queda byte a byte igual");
    // Só as páxinas con texto levan capa; a 2 non se tocou.
    let content = |n: u32| after.get_page_content(*after.get_pages().get(&n).unwrap()).unwrap();
    assert!(String::from_utf8_lossy(&content(1)).contains("3 Tr"));
    assert!(!String::from_utf8_lossy(&content(2)).contains("3 Tr"));
    // O texto sácase ben (lopdf lee o ToUnicode da letra).
    let text = after.extract_text(&[1]).unwrap();
    assert!(text.contains("Olá") && text.contains("mundo,") && text.contains("niño"), "{text:?}");
    let text3 = after.extract_text(&[3]).unwrap();
    assert!(text3.contains("€100") && text3.contains("😀x"), "{text3:?}");
    if let Ok(keep) = std::env::var("TRASNO_KEEP_OCR_PDF") {
        std::fs::copy(&out, keep).unwrap();
    }
    // O orixinal non cambiou.
    assert_eq!(std::fs::read(&src).unwrap(), std::fs::read(dir.path().join("scan.pdf")).unwrap());
}

#[test]
fn rexeita_o_que_non_vale_e_non_pisa_nada() {
    let dir = tempfile::tempdir().unwrap();
    let src = scan(dir.path(), 1);
    let out = dir.path().join("o.pdf");
    // Sen texto, ou só espazos ou números rotos.
    assert!(add_text_layer(&src, &out, false, &[]).is_err());
    assert!(add_text_layer(&src, &out, false, &[OcrPage { page: 1, words: vec![word("  ", 1.0, 1.0, 5.0, 5.0)] }]).is_err());
    assert!(add_text_layer(&src, &out, false, &[OcrPage { page: 1, words: vec![word("a", f32::NAN, 1.0, 5.0, 5.0), word("b", 1.0, 1.0, 0.0, 5.0), word("c", 1.0, 1.0, 5.0, -1.0)] }]).is_err());
    assert!(!out.exists(), "sen resultado non queda ficheiro");
    // Páxina que non existe.
    assert!(add_text_layer(&src, &out, false, &[OcrPage { page: 9, words: vec![word("a", 1.0, 1.0, 5.0, 5.0)] }]).is_err());
    assert!(!out.exists());
    // Non pisa o orixinal nin un ficheiro que xa existe.
    assert!(add_text_layer(&src, &src, true, &[OcrPage { page: 1, words: vec![word("a", 1.0, 1.0, 5.0, 5.0)] }]).is_err());
    std::fs::write(&out, b"x").unwrap();
    assert!(add_text_layer(&src, &out, false, &[OcrPage { page: 1, words: vec![word("a", 1.0, 1.0, 5.0, 5.0)] }]).is_err());
    assert_eq!(std::fs::read(&out).unwrap(), b"x");
}

#[test]
fn funciona_con_paxinas_xiradas_e_con_recursos_herdados() {
    let dir = tempfile::tempdir().unwrap();
    let src = scan(dir.path(), 1);
    // Páxina xirada 90° e coa letra xa existente no recurso.
    let mut doc = Document::load(&src).unwrap();
    let id = *doc.get_pages().get(&1).unwrap();
    doc.get_object_mut(id).unwrap().as_dict_mut().unwrap().set("Rotate", 90);
    let rot = dir.path().join("rot.pdf");
    doc.save(&rot).unwrap();
    let out = dir.path().join("o.pdf");
    let w = OcrWord { text: "xirado".into(), x: 100.0, y: 50.0, w: 50.0, h: 12.0, angle: 90.0 };
    add_text_layer(&rot, &out, false, &[OcrPage { page: 1, words: vec![w] }]).unwrap();
    let after = Document::load(&out).unwrap();
    assert!(after.extract_text(&[1]).unwrap().contains("xirado"));
    let c = String::from_utf8_lossy(&after.get_page_content(*after.get_pages().get(&1).unwrap()).unwrap()).to_string();
    assert!(c.contains("0.00000 1.00000 -1.00000 0.00000"), "a liña vai a 90°: {c}");
}
