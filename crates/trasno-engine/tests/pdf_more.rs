//! Limpeza, índice, tapar de verdade e formularios en PDF.

use lopdf::{dictionary, Document, Object, ObjectId, Stream, StringFormat};
use std::path::{Path, PathBuf};
use trasno_engine::pdf_more::*;

fn s(t: &str) -> Object {
    Object::String(t.as_bytes().to_vec(), StringFormat::Literal)
}

/// PDF de `n` páxinas, cada unha co texto «Segredo i». Devolve o documento aberto para engadirlle cousas e o catálogo.
fn base(n: usize) -> (Document, ObjectId, Vec<ObjectId>) {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" });
    let mut kids: Vec<Object> = Vec::new();
    let mut ids = Vec::new();
    for i in 0..n {
        let c = doc.add_object(Stream::new(dictionary! {}, format!("BT /F1 24 Tf 50 700 Td (Segredo {i}) Tj ET").into_bytes()));
        let p = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Contents" => c,
            "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
        });
        kids.push(p.into());
        ids.push(p);
    }
    doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => n as i64 }));
    let cat = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", cat);
    (doc, cat, ids)
}

fn save(mut doc: Document, dir: &Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    doc.save(&p).unwrap();
    p
}

#[test]
fn limpar_quita_so_o_pedido_e_conta_o_que_quitou() {
    let dir = tempfile::tempdir().unwrap();
    let (mut doc, cat, pages) = base(2);
    let info = doc.add_object(dictionary! { "Author" => s("Marta"), "Title" => s("Informe"), "Producer" => s("X") });
    doc.trailer.set("Info", info);
    let js = doc.add_object(dictionary! { "S" => "JavaScript", "JS" => s("app.alert(1)") });
    let link_web = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Link", "Rect" => vec![0.into(), 0.into(), 10.into(), 10.into()], "A" => dictionary! { "S" => "URI", "URI" => s("https://example.com") } });
    let link_in = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Link", "Rect" => vec![0.into(), 0.into(), 10.into(), 10.into()], "Dest" => vec![Object::Reference(pages[1]), "Fit".into()] });
    let note = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Text", "Rect" => vec![0.into(), 0.into(), 10.into(), 10.into()], "Contents" => s("comentario") });
    let attach = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "FileAttachment", "Rect" => vec![0.into(), 0.into(), 10.into(), 10.into()] });
    doc.get_dictionary_mut(pages[0]).unwrap().set("Annots", Object::Array(vec![link_web.into(), link_in.into(), note.into(), attach.into()]));
    let emb = doc.add_object(dictionary! { "Names" => vec![s("a.txt"), Object::Null] });
    let names = doc.add_object(dictionary! { "EmbeddedFiles" => Object::Reference(emb), "JavaScript" => Object::Reference(emb) });
    let c = doc.get_dictionary_mut(cat).unwrap();
    c.set("OpenAction", Object::Reference(js));
    c.set("Names", Object::Reference(names));
    let src = save(doc, dir.path(), "a.pdf");

    // Só metadatos: o resto queda.
    let out = dir.path().join("meta.pdf");
    let r = sanitize_pdf(&src, &out, false, &Sanitize { metadata: true, scripts: false, attachments: false, links: false, comments: false }).unwrap();
    assert!(r.metadata >= 3 && r.scripts == 0 && r.links == 0);
    let d = Document::load(&out).unwrap();
    assert!(d.trailer.get(b"Info").is_err(), "sen Info");
    assert_eq!(d.get_page_annotations(*d.get_pages().get(&1).unwrap()).unwrap().len(), 4, "as anotacións seguen");
    assert!(d.catalog().unwrap().has(b"OpenAction"));

    // Todo.
    let out = dir.path().join("todo.pdf");
    let r = sanitize_pdf(&src, &out, false, &Sanitize { metadata: true, scripts: true, attachments: true, links: true, comments: true }).unwrap();
    assert_eq!((r.links, r.comments, r.attachments >= 2, r.scripts >= 2), (1, 1, true, true), "{r:?}");
    let d = Document::load(&out).unwrap();
    let ann = d.get_page_annotations(*d.get_pages().get(&1).unwrap()).unwrap();
    assert_eq!(ann.len(), 1, "só queda a ligazón interna");
    assert!(ann[0].has(b"Dest"));
    let cat = d.catalog().unwrap();
    assert!(!cat.has(b"OpenAction"));
    let raw = std::fs::read(&out).unwrap();
    for needle in ["Marta", "app.alert", "example.com", "a.txt"] {
        assert!(!raw.windows(needle.len()).any(|w| w == needle.as_bytes()), "{needle} segue no ficheiro");
    }
    // O texto das páxinas segue lendose.
    assert!(d.extract_text(&[1]).unwrap().contains("Segredo 0"));
    // O orixinal non se tocou.
    assert!(Document::load(&src).unwrap().trailer.get(b"Info").is_ok());
}

#[test]
fn indice_con_niveis_e_titulos_con_acentos() {
    let dir = tempfile::tempdir().unwrap();
    let (doc, _, _) = base(4);
    let src = save(doc, dir.path(), "a.pdf");
    let out = dir.path().join("indice.pdf");
    let e = |t: &str, page: u32, level: u32| OutlineEntry { title: t.into(), page, level };
    set_outline(&src, &out, false, &[e("Introdución", 1, 0), e("Contexto", 1, 1), e("Método ☺", 2, 1), e("Anexo", 4, 0)]).unwrap();
    let d = Document::load(&out).unwrap();
    let cat = d.catalog().unwrap();
    let root = d.get_dictionary(cat.get(b"Outlines").unwrap().as_reference().unwrap()).unwrap();
    assert_eq!(root.get(b"Count").unwrap().as_i64().unwrap(), 4);
    let first = d.get_dictionary(root.get(b"First").unwrap().as_reference().unwrap()).unwrap();
    assert_eq!(first.get(b"Count").unwrap().as_i64().unwrap(), 2, "«Introdución» ten dous fillos");
    let second = d.get_dictionary(first.get(b"Next").unwrap().as_reference().unwrap()).unwrap();
    assert!(second.get(b"Title").is_ok(), "o segundo do nivel 0 é «Anexo»");
    let toc = d.get_toc().unwrap();
    assert_eq!(toc.toc.len(), 4, "{toc:?}");
    assert_eq!(toc.toc[3].page, 4);
    // Un nivel que salta (de 0 a 3) queda un máis que o anterior; un título baleiro e unha páxina que non existe dan erro.
    let out2 = dir.path().join("indice2.pdf");
    set_outline(&src, &out2, false, &[e("A", 1, 0), e("B", 2, 3)]).unwrap();
    assert!(set_outline(&src, &dir.path().join("x.pdf"), false, &[e("  ", 1, 0)]).is_err());
    assert!(set_outline(&src, &dir.path().join("y.pdf"), false, &[e("A", 9, 0)]).is_err());
    // Lista baleira: quita o índice.
    let out3 = dir.path().join("sen.pdf");
    set_outline(&out, &out3, false, &[]).unwrap();
    assert!(!Document::load(&out3).unwrap().catalog().unwrap().has(b"Outlines"));
}

#[test]
fn tapar_quita_o_texto_da_paxina_e_deixa_as_outras() {
    let dir = tempfile::tempdir().unwrap();
    let (mut doc, cat, pages) = base(3);
    // Un campo de formulario na páxina 1 (vai desaparecer) e outro na 3 (queda).
    let w1 = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Widget", "FT" => "Tx", "T" => s("dni"), "V" => s("12345678Z"), "Rect" => vec![0.into(), 0.into(), 50.into(), 20.into()] });
    let w3 = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Widget", "FT" => "Tx", "T" => s("nota"), "V" => s("ola"), "Rect" => vec![0.into(), 0.into(), 50.into(), 20.into()] });
    doc.get_dictionary_mut(pages[0]).unwrap().set("Annots", Object::Array(vec![w1.into()]));
    doc.get_dictionary_mut(pages[2]).unwrap().set("Annots", Object::Array(vec![w3.into()]));
    doc.get_dictionary_mut(cat).unwrap().set("AcroForm", dictionary! { "Fields" => vec![Object::Reference(w1), Object::Reference(w3)] });
    let src = save(doc, dir.path(), "a.pdf");
    let jpeg = {
        let mut buf = Vec::new();
        let img = image::RgbImage::from_pixel(120, 170, image::Rgb([250, 250, 250]));
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 90).encode_image(&img).unwrap();
        buf
    };
    let out = dir.path().join("tapado.pdf");
    let r = redact_pages(&src, &out, false, &[RedactPage { page: 1, jpeg }]).unwrap();
    assert_eq!((r.pages, r.fields_removed), (1, 1));
    let d = Document::load(&out).unwrap();
    assert_eq!(d.get_pages().len(), 3);
    assert!(d.extract_text(&[1]).unwrap().trim().is_empty(), "a páxina 1 xa non ten texto");
    assert!(d.extract_text(&[2]).unwrap().contains("Segredo 1"), "a 2 queda igual");
    let raw = std::fs::read(&out).unwrap();
    for needle in ["Segredo 0", "12345678Z"] {
        assert!(!raw.windows(needle.len()).any(|w| w == needle.as_bytes()), "{needle} segue no ficheiro");
    }
    assert!(raw.windows(9).any(|w| w == b"Segredo 1"), "o texto das outras páxinas segue");
    // A páxina 1 mide o mesmo (595 × 842) e leva a imaxe.
    let p1 = *d.get_pages().get(&1).unwrap();
    let mb = d.get_dictionary(p1).unwrap().get(b"MediaBox").unwrap().as_array().unwrap();
    assert_eq!(mb[2].as_float().unwrap().round() as i32, 595);
    assert!(String::from_utf8_lossy(&d.get_page_content(p1).unwrap()).contains("Do"));
    assert!(redact_pages(&src, &dir.path().join("z.pdf"), false, &[]).is_err());
}

/// Formulario con texto, casilla (con aparencia «Yes») e un grupo de opcións.
fn form_pdf(dir: &Path) -> PathBuf {
    let (mut doc, cat, pages) = base(1);
    let rect = |y: i32| vec![50.into(), y.into(), 250.into(), (y + 20).into()];
    let nome = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Widget", "FT" => "Tx", "T" => s("nome"), "V" => s(""), "Rect" => rect(600), "DA" => s("/Helv 0 Tf 0 g") });
    let on = doc.add_object(Stream::new(dictionary! { "Type" => "XObject", "Subtype" => "Form", "BBox" => vec![0.into(), 0.into(), 12.into(), 12.into()] }, b"0 0 12 12 re S".to_vec()));
    let acepto = doc.add_object(dictionary! {
        "Type" => "Annot", "Subtype" => "Widget", "FT" => "Btn", "T" => s("acepto"), "V" => "Off", "AS" => "Off", "Rect" => vec![50.into(), 560.into(), 62.into(), 572.into()],
        "AP" => dictionary! { "N" => dictionary! { "Yes" => Object::Reference(on) } },
    });
    let r1 = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Widget", "Rect" => vec![50.into(), 520.into(), 62.into(), 532.into()], "AS" => "Off", "AP" => dictionary! { "N" => dictionary! { "A" => Object::Reference(on) } } });
    let r2 = doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Widget", "Rect" => vec![80.into(), 520.into(), 92.into(), 532.into()], "AS" => "Off", "AP" => dictionary! { "N" => dictionary! { "B" => Object::Reference(on) } } });
    let grupo = doc.add_object(dictionary! { "FT" => "Btn", "Ff" => 1 << 15, "T" => s("opcion"), "V" => "Off", "Kids" => vec![Object::Reference(r1), Object::Reference(r2)] });
    for r in [r1, r2] {
        doc.get_dictionary_mut(r).unwrap().set("Parent", Object::Reference(grupo));
    }
    doc.get_dictionary_mut(pages[0]).unwrap().set("Annots", Object::Array(vec![nome.into(), acepto.into(), r1.into(), r2.into()]));
    doc.get_dictionary_mut(cat).unwrap().set("AcroForm", dictionary! { "Fields" => vec![Object::Reference(nome), Object::Reference(acepto), Object::Reference(grupo)] });
    save(doc, dir, "form.pdf")
}

#[test]
fn formulario_le_enche_e_aplana() {
    let dir = tempfile::tempdir().unwrap();
    let src = form_pdf(dir.path());
    let f = form_fields(&src).unwrap();
    let kinds: Vec<(&str, &str)> = f.iter().map(|x| (x.name.as_str(), x.kind.as_str())).collect();
    assert_eq!(kinds, vec![("nome", "text"), ("acepto", "check"), ("opcion", "radio")]);
    assert_eq!(f[1].value, "false");
    assert_eq!(f[2].options.iter().map(|o| o.value.as_str()).collect::<Vec<_>>(), vec!["A", "B"]);

    let vals = vec![FieldValue { name: "nome".into(), value: "María Núñez".into() }, FieldValue { name: "acepto".into(), value: "true".into() }, FieldValue { name: "opcion".into(), value: "B".into() }, FieldValue { name: "non_existe".into(), value: "x".into() }];
    let out = dir.path().join("cheo.pdf");
    let r = fill_form(&src, &out, false, &vals, false).unwrap();
    assert_eq!(r.filled, 3);
    let g = form_fields(&out).unwrap();
    assert_eq!(g[0].value, "María Núñez");
    assert_eq!(g[1].value, "true");
    assert_eq!(g[2].value, "B");
    // O orixinal segue baleiro.
    assert_eq!(form_fields(&src).unwrap()[0].value, "");

    // Aplanado: sen campos e coa aparencia debuxada na páxina.
    let flat = dir.path().join("plano.pdf");
    let r = fill_form(&src, &flat, false, &vals, true).unwrap();
    assert!(r.flattened);
    assert!(form_fields(&flat).unwrap().is_empty(), "xa non hai formulario");
    let d = Document::load(&flat).unwrap();
    let p = *d.get_pages().get(&1).unwrap();
    assert!(d.get_page_annotations(p).unwrap().is_empty(), "sen widgets");
    let content = String::from_utf8_lossy(&d.get_page_content(p).unwrap()).to_string();
    assert!(content.contains("FmTrasno0") && content.matches(" Do").count() >= 3, "{content}");
    // Sen valores que se poidan encher, erro.
    assert!(fill_form(&src, &dir.path().join("e.pdf"), false, &[FieldValue { name: "nada".into(), value: "x".into() }], false).is_err());
}
