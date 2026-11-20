//! Máis ferramentas de PDF: limpeza (PDF seguro), índice (marcadores), tapar zonas de verdade e formularios.
//! Sempre se escribe un ficheiro novo, de forma atómica, e nunca se toca o orixinal.

use crate::error::{EngineError, Result};
use crate::integrity::ensure_distinct;
use crate::pdf::{flatten_inherited_page_attributes, load, save_atomically};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream, StringFormat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

fn pdf_err(msg: &str) -> EngineError {
    EngineError::Pdf(msg.to_string())
}
fn lerr(e: lopdf::Error) -> EngineError {
    EngineError::Pdf(e.to_string())
}

fn open_plain(input: &Path, output: &Path) -> Result<Document> {
    ensure_distinct(&[input], output)?;
    let doc = load(input)?;
    if doc.is_encrypted() {
        return Err(pdf_err("O PDF está protexido con contrasinal: quítalle a protección antes."));
    }
    Ok(doc)
}

/// Segue unha referencia ata o obxecto (ou devolve o propio).
pub(crate) fn deref<'a>(doc: &'a Document, o: &'a Object) -> &'a Object {
    match o {
        Object::Reference(id) => doc.get_object(*id).unwrap_or(o),
        _ => o,
    }
}
pub(crate) fn dict_at(doc: &Document, o: &Object) -> Option<Dictionary> {
    match deref(doc, o) {
        Object::Dictionary(d) => Some(d.clone()),
        Object::Stream(s) => Some(s.dict.clone()),
        _ => None,
    }
}
fn name_of(o: Option<&Object>) -> Vec<u8> {
    match o {
        Some(Object::Name(n)) => n.clone(),
        _ => Vec::new(),
    }
}
pub(crate) fn num(o: &Object) -> f32 {
    match o {
        Object::Integer(i) => *i as f32,
        Object::Real(r) => *r,
        _ => 0.0,
    }
}
pub(crate) fn rect_of(doc: &Document, d: &Dictionary) -> Option<[f32; 4]> {
    let a = d.get(b"Rect").ok().map(|o| deref(doc, o))?.as_array().ok()?;
    if a.len() != 4 {
        return None;
    }
    let v: Vec<f32> = a.iter().map(|x| num(deref(doc, x))).collect();
    Some([v[0].min(v[2]), v[1].min(v[3]), v[0].max(v[2]), v[1].max(v[3])])
}

/// Texto de PDF (PDFDocEncoding ≈ Latin-1, ou UTF-16 con BOM) a `String`.
fn pdf_text(b: &[u8]) -> String {
    if b.len() >= 2 && b[0] == 0xFE && b[1] == 0xFF {
        let u: Vec<u16> = b[2..].chunks(2).filter(|c| c.len() == 2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&u)
    } else if b.len() >= 2 && b[0] == 0xFF && b[1] == 0xFE {
        let u: Vec<u16> = b[2..].chunks(2).filter(|c| c.len() == 2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&u)
    } else {
        b.iter().map(|c| *c as char).collect()
    }
}
/// `String` a texto de PDF: ASCII tal cal; o resto en UTF-16 con BOM.
fn text_obj(s: &str) -> Object {
    if s.is_ascii() {
        Object::String(s.as_bytes().to_vec(), StringFormat::Literal)
    } else {
        let mut v = vec![0xFE, 0xFF];
        for u in s.encode_utf16() {
            v.extend(u.to_be_bytes());
        }
        Object::String(v, StringFormat::Hexadecimal)
    }
}

fn page_ids(doc: &Document) -> Vec<(u32, ObjectId)> {
    doc.get_pages().into_iter().collect()
}

/// Lista de anotacións dunha páxina (a lista pode ser directa ou indirecta).
pub(crate) fn annots_of(doc: &Document, page: ObjectId) -> Vec<Object> {
    let Ok(d) = doc.get_dictionary(page) else { return Vec::new() };
    match d.get(b"Annots").ok().map(|o| deref(doc, o)) {
        Some(Object::Array(a)) => a.clone(),
        _ => Vec::new(),
    }
}
pub(crate) fn set_annots(doc: &mut Document, page: ObjectId, list: Vec<Object>) {
    if let Ok(d) = doc.get_dictionary_mut(page) {
        if list.is_empty() {
            d.remove(b"Annots");
        } else {
            d.set("Annots", Object::Array(list));
        }
    }
}

// ----------------------------------------------------------------- PDF seguro

/// Que se quita nunha limpeza.
#[derive(Debug, Clone, Deserialize)]
pub struct Sanitize {
    /// Autor, título, programa, datas e demais datos do documento (incluído XMP).
    pub metadata: bool,
    /// JavaScript e accións automáticas (abrir, lanzar programas, enviar formularios).
    pub scripts: bool,
    /// Ficheiros anexados dentro do PDF.
    pub attachments: bool,
    /// Ligazóns a páxinas web, a outros ficheiros ou a programas.
    pub links: bool,
    /// Comentarios, resaltados e notas.
    pub comments: bool,
}

/// Cantas cousas se quitaron de cada tipo.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SanitizeReport {
    pub output_path: PathBuf,
    pub metadata: u32,
    pub scripts: u32,
    pub attachments: u32,
    pub links: u32,
    pub comments: u32,
}

const DANGEROUS: [&[u8]; 7] = [b"JavaScript", b"Launch", b"ImportData", b"SubmitForm", b"Rendition", b"Movie", b"GoToE"];
const COMMENT_KINDS: [&[u8]; 16] = [
    b"Text", b"Highlight", b"Underline", b"StrikeOut", b"Squiggly", b"FreeText", b"Ink", b"Stamp", b"Line", b"Square", b"Circle", b"Polygon", b"PolyLine", b"Caret", b"Popup", b"Redact",
];

/// Quita un valor do dicionario `Names` do catálogo (que pode ser indirecto). Devolve se había algo.
fn remove_from_names(doc: &mut Document, key: &[u8]) -> bool {
    let target = match doc.catalog().ok().and_then(|c| c.get(b"Names").ok()).cloned() {
        Some(Object::Reference(id)) => Some(id),
        Some(Object::Dictionary(_)) => None,
        _ => return false,
    };
    match target {
        Some(id) => doc.get_dictionary_mut(id).map(|d| d.remove(key).is_some()).unwrap_or(false),
        None => doc.catalog_mut().ok().and_then(|c| c.get_mut(b"Names").ok().and_then(|n| n.as_dict_mut().ok()).map(|d| d.remove(key).is_some())).unwrap_or(false),
    }
}

pub fn sanitize_pdf(input: &Path, output: &Path, overwrite: bool, o: &Sanitize) -> Result<SanitizeReport> {
    let mut doc = open_plain(input, output)?;
    let mut r = SanitizeReport::default();

    if o.metadata {
        if let Some(info) = doc.trailer.remove(b"Info") {
            r.metadata += dict_at(&doc, &info).map(|d| d.len() as u32).unwrap_or(1).max(1);
        }
        if let Ok(c) = doc.catalog_mut() {
            for k in [&b"Metadata"[..], b"PieceInfo"] {
                if c.remove(k).is_some() {
                    r.metadata += 1;
                }
            }
        }
        for (_, p) in page_ids(&doc) {
            if let Ok(d) = doc.get_dictionary_mut(p) {
                for k in [&b"Metadata"[..], b"PieceInfo", b"Thumb"] {
                    if d.remove(k).is_some() {
                        r.metadata += 1;
                    }
                }
            }
        }
    }

    if o.scripts {
        let open_is_action = doc.catalog().ok().and_then(|c| c.get(b"OpenAction").ok()).and_then(|a| dict_at(&doc, a)).is_some();
        if let Ok(c) = doc.catalog_mut() {
            if c.remove(b"AA").is_some() {
                r.scripts += 1;
            }
            if open_is_action && c.remove(b"OpenAction").is_some() {
                r.scripts += 1;
            }
        }
        if remove_from_names(&mut doc, b"JavaScript") {
            r.scripts += 1;
        }
        // XFA (formularios con scripts propios).
        let form_id = doc.catalog().ok().and_then(|c| c.get(b"AcroForm").ok()).and_then(|a| a.as_reference().ok());
        let removed_xfa = match form_id {
            Some(id) => doc.get_dictionary_mut(id).map(|d| d.remove(b"XFA").is_some()).unwrap_or(false),
            None => doc.catalog_mut().ok().and_then(|c| c.get_mut(b"AcroForm").ok().and_then(|f| f.as_dict_mut().ok()).map(|d| d.remove(b"XFA").is_some())).unwrap_or(false),
        };
        if removed_xfa {
            r.scripts += 1;
        }
        for (_, p) in page_ids(&doc) {
            if let Ok(d) = doc.get_dictionary_mut(p) {
                if d.remove(b"AA").is_some() {
                    r.scripts += 1;
                }
            }
        }
    }

    if o.attachments {
        if remove_from_names(&mut doc, b"EmbeddedFiles") {
            r.attachments += 1;
        }
        if let Ok(c) = doc.catalog_mut() {
            if c.remove(b"AF").is_some() {
                r.attachments += 1;
            }
        }
    }

    for (_, p) in page_ids(&doc) {
        let list = annots_of(&doc, p);
        if list.is_empty() {
            continue;
        }
        let mut keep: Vec<Object> = Vec::new();
        for item in list {
            let Some(ad) = dict_at(&doc, &item) else {
                keep.push(item);
                continue;
            };
            let sub = name_of(ad.get(b"Subtype").ok());
            let action = ad.get(b"A").ok().and_then(|a| dict_at(&doc, a));
            let kind = action.as_ref().map(|a| name_of(a.get(b"S").ok())).unwrap_or_default();
            if o.attachments && sub == b"FileAttachment" {
                r.attachments += 1;
                continue;
            }
            if o.comments && COMMENT_KINDS.iter().any(|k| *k == sub.as_slice()) {
                r.comments += 1;
                continue;
            }
            if o.links && sub == b"Link" && (kind == b"URI" || kind == b"GoToR" || kind == b"Launch") {
                r.links += 1;
                continue;
            }
            if o.scripts {
                let has_aa = ad.has(b"AA");
                let bad = DANGEROUS.iter().any(|k| *k == kind.as_slice());
                if bad || has_aa {
                    if let Object::Reference(id) = &item {
                        if let Ok(d) = doc.get_dictionary_mut(*id) {
                            if bad {
                                d.remove(b"A");
                            }
                            d.remove(b"AA");
                        }
                    }
                    r.scripts += 1;
                }
            }
            keep.push(item);
        }
        set_annots(&mut doc, p, keep);
    }

    doc.prune_objects();
    r.output_path = save_atomically(&mut doc, output, overwrite)?;
    Ok(r)
}

// ----------------------------------------------------------------------- índice

/// Unha entrada do índice: título, páxina (desde 1) e nivel (0 = principal).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutlineEntry {
    pub title: String,
    pub page: u32,
    pub level: u32,
}

/// Substitúe o índice (os marcadores do panel lateral do lector) polo que se pasa. Cunha lista baleira quita o índice.
pub fn set_outline(input: &Path, output: &Path, overwrite: bool, entries: &[OutlineEntry]) -> Result<PathBuf> {
    let mut doc = open_plain(input, output)?;
    let pages = doc.get_pages();
    let total = pages.len() as u32;
    for e in entries {
        if e.title.trim().is_empty() {
            return Err(pdf_err("Hai unha entrada do índice sen título."));
        }
        if !pages.contains_key(&e.page) {
            return Err(EngineError::PageMissing { page: e.page, total });
        }
    }
    if entries.is_empty() {
        let c = doc.catalog_mut().map_err(lerr)?;
        c.remove(b"Outlines");
        doc.prune_objects();
        return save_atomically(&mut doc, output, overwrite);
    }

    // Niveis: o primeiro é 0 e cada un só pode ser un máis que o anterior.
    let mut levels: Vec<usize> = Vec::with_capacity(entries.len());
    for (i, e) in entries.iter().enumerate() {
        let max = if i == 0 { 0 } else { levels[i - 1] + 1 };
        levels.push((e.level as usize).min(max));
    }
    let ids: Vec<ObjectId> = entries.iter().map(|_| doc.new_object_id()).collect();
    let root = doc.new_object_id();

    // Pai de cada entrada e fillos de cada pai (None = raíz).
    let mut parent: Vec<Option<usize>> = vec![None; entries.len()];
    let mut stack: Vec<usize> = Vec::new();
    for i in 0..entries.len() {
        while stack.last().map(|&j| levels[j] >= levels[i]).unwrap_or(false) {
            stack.pop();
        }
        parent[i] = stack.last().copied();
        stack.push(i);
    }
    let mut kids: HashMap<Option<usize>, Vec<usize>> = HashMap::new();
    for i in 0..entries.len() {
        kids.entry(parent[i]).or_default().push(i);
    }
    fn descendants(i: usize, kids: &HashMap<Option<usize>, Vec<usize>>) -> i64 {
        kids.get(&Some(i)).map(|k| k.iter().map(|&c| 1 + descendants(c, kids)).sum()).unwrap_or(0)
    }
    for i in 0..entries.len() {
        let mut d = Dictionary::new();
        d.set("Title", text_obj(entries[i].title.trim()));
        d.set("Parent", Object::Reference(parent[i].map(|p| ids[p]).unwrap_or(root)));
        let page_ref = pages[&entries[i].page];
        d.set("Dest", Object::Array(vec![Object::Reference(page_ref), Object::Name(b"Fit".to_vec())]));
        let sibs = &kids[&parent[i]];
        let at = sibs.iter().position(|&s| s == i).unwrap_or(0);
        if at > 0 {
            d.set("Prev", Object::Reference(ids[sibs[at - 1]]));
        }
        if at + 1 < sibs.len() {
            d.set("Next", Object::Reference(ids[sibs[at + 1]]));
        }
        if let Some(own) = kids.get(&Some(i)) {
            d.set("First", Object::Reference(ids[own[0]]));
            d.set("Last", Object::Reference(ids[*own.last().unwrap()]));
            d.set("Count", Object::Integer(descendants(i, &kids)));
        }
        doc.set_object(ids[i], Object::Dictionary(d));
    }
    let top = &kids[&None];
    let outlines = dictionary! {
        "Type" => "Outlines",
        "First" => Object::Reference(ids[top[0]]),
        "Last" => Object::Reference(ids[*top.last().unwrap()]),
        "Count" => Object::Integer(entries.len() as i64),
    };
    doc.set_object(root, Object::Dictionary(outlines));
    let c = doc.catalog_mut().map_err(lerr)?;
    c.set("Outlines", Object::Reference(root));
    c.set("PageMode", Object::Name(b"UseOutlines".to_vec()));
    doc.prune_objects();
    save_atomically(&mut doc, output, overwrite)
}

// ------------------------------------------------------------- tapar de verdade

/// Unha páxina xa debuxada (JPEG) co negro por riba do que hai que tapar. O texto e as imaxes de debaixo desaparecen do ficheiro.
pub struct RedactPage {
    pub page: u32,
    pub jpeg: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedactReport {
    pub output_path: PathBuf,
    /// Páxinas tapadas.
    pub pages: u32,
    pub fields_removed: u32,
    /// Das tapadas, as que conservan o seu contido (só se quitou o de debaixo das zonas)…
    pub cleaned: u32,
    /// …e as que pasaron a ser unha imaxe.
    pub rasterized: u32,
}

fn page_view_box(doc: &Document, page: ObjectId) -> (f32, f32) {
    let d = doc.get_dictionary(page).ok();
    for key in [&b"CropBox"[..], &b"MediaBox"[..]] {
        if let Some(a) = d.and_then(|d| d.get(key).ok()).map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()) {
            if a.len() == 4 {
                let v: Vec<f32> = a.iter().map(|x| num(deref(doc, x))).collect();
                return ((v[2] - v[0]).abs(), (v[3] - v[1]).abs());
            }
        }
    }
    (595.0, 842.0)
}

/// Cambia unha páxina por unha soa imaxe a tamaño de páxina (sen texto, sen anotacións e sen o contido orixinal).
/// Devolve as anotacións que quitou.
fn rasterize_page(doc: &mut Document, id: ObjectId, jpeg: &[u8]) -> Result<Vec<ObjectId>> {
    let (w_px, h_px) = {
        let r = image::ImageReader::new(std::io::Cursor::new(jpeg)).with_guessed_format().map_err(|e| pdf_err(&e.to_string()))?;
        r.into_dimensions().map_err(|e| pdf_err(&e.to_string()))?
    };
    let (mut w, mut h) = page_view_box(doc, id);
    let rotate = doc.get_dictionary(id).ok().and_then(|d| d.get(b"Rotate").ok()).map(|o| num(deref(doc, o)) as i64).unwrap_or(0).rem_euclid(360);
    if rotate % 180 != 0 {
        std::mem::swap(&mut w, &mut h);
    }
    let mut gone = Vec::new();
    for a in annots_of(doc, id) {
        if let Object::Reference(r) = a {
            gone.push(r);
        }
    }
    let mut img = Stream::new(
        dictionary! { "Type" => "XObject", "Subtype" => "Image", "Width" => w_px as i64, "Height" => h_px as i64, "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8, "Filter" => "DCTDecode" },
        jpeg.to_vec(),
    );
    img.allows_compression = false;
    let img_id = doc.add_object(img);
    let content = doc.add_object(Stream::new(dictionary! {}, format!("q {w} 0 0 {h} 0 0 cm /ImTrasnoTapar Do Q").into_bytes()));
    let boxarr = Object::Array(vec![0.into(), 0.into(), w.into(), h.into()]);
    let d = doc.get_dictionary_mut(id).map_err(lerr)?;
    d.set("Contents", Object::Reference(content));
    d.set("Resources", Object::Dictionary(dictionary! { "XObject" => dictionary! { "ImTrasnoTapar" => Object::Reference(img_id) } }));
    d.set("MediaBox", boxarr.clone());
    d.set("CropBox", boxarr);
    for k in [&b"Rotate"[..], b"Annots", b"Thumb", b"Metadata", b"PieceInfo", b"AA", b"B", b"StructParents", b"Group"] {
        d.remove(k);
    }
    Ok(gone)
}

/// Quita os campos de formulario cuxos widgets desapareceron, e a estrutura etiquetada (pode levar o texto), e garda.
fn finish_redaction(doc: &mut Document, removed_widgets: &HashSet<ObjectId>, output: &Path, overwrite: bool) -> Result<(PathBuf, u32)> {
    let mut fields_removed = 0;
    let form = doc.catalog().ok().and_then(|c| c.get(b"AcroForm").ok()).cloned();
    if let Some(form) = form {
        let fd = dict_at(doc, &form);
        let fields: Vec<Object> = fd.as_ref().and_then(|d| d.get(b"Fields").ok()).map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()).cloned().unwrap_or_default();
        fn widgets(doc: &Document, id: ObjectId, out: &mut Vec<ObjectId>, depth: u32) {
            if depth > 16 {
                return;
            }
            let Ok(d) = doc.get_dictionary(id) else { return };
            match d.get(b"Kids").ok().map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()) {
                Some(k) => {
                    for c in k {
                        if let Object::Reference(cid) = c {
                            widgets(doc, *cid, out, depth + 1);
                        }
                    }
                }
                None => out.push(id),
            }
        }
        let mut keep = Vec::new();
        for f in fields {
            let Object::Reference(fid) = f else {
                keep.push(f);
                continue;
            };
            let mut ws = Vec::new();
            widgets(doc, fid, &mut ws, 0);
            if !ws.is_empty() && ws.iter().all(|w| removed_widgets.contains(w)) {
                fields_removed += 1;
            } else {
                keep.push(f);
            }
        }
        if fields_removed > 0 {
            let id = form.as_reference().ok();
            let target: Option<&mut Dictionary> = match id {
                Some(id) => doc.get_dictionary_mut(id).ok(),
                None => doc.catalog_mut().ok().and_then(|c| c.get_mut(b"AcroForm").ok()).and_then(|f| f.as_dict_mut().ok()),
            };
            if let Some(t) = target {
                t.set("Fields", Object::Array(keep));
            }
        }
    }
    if let Ok(c) = doc.catalog_mut() {
        c.remove(b"StructTreeRoot");
        c.remove(b"MarkInfo");
    }
    doc.prune_objects();
    let path = save_atomically(doc, output, overwrite)?;
    Ok((path, fields_removed))
}

/// Cada páxina tapada pasa a ser unha soa imaxe a tamaño de páxina: sen texto, sen anotacións e sen o contido orixinal.
/// As outras páxinas quedan tal cal. (Sempre vale, pero cambia o aspecto do ficheiro: `redact_zones` só o fai cando non queda outro remedio.)
pub fn redact_pages(input: &Path, output: &Path, overwrite: bool, pages: &[RedactPage]) -> Result<RedactReport> {
    if pages.is_empty() {
        return Err(pdf_err("Non hai ningunha zona que tapar."));
    }
    let mut doc = open_plain(input, output)?;
    flatten_inherited_page_attributes(&mut doc);
    let ids = doc.get_pages();
    let total = ids.len() as u32;
    let mut gone: HashSet<ObjectId> = HashSet::new();
    for rp in pages {
        let id = *ids.get(&rp.page).ok_or(EngineError::PageMissing { page: rp.page, total })?;
        gone.extend(rasterize_page(&mut doc, id, &rp.jpeg)?);
    }
    let (path, fields_removed) = finish_redaction(&mut doc, &gone, output, overwrite)?;
    Ok(RedactReport { output_path: path, pages: pages.len() as u32, fields_removed, cleaned: 0, rasterized: pages.len() as u32 })
}

/// Páxinas con zonas que non se poden tapar sen convertelas en imaxe (e por que). As demais tápanse tocando só o que está debaixo.
pub fn redact_plan(input: &Path, zones: &[crate::pdf_redact::Zone]) -> Result<Vec<(u32, String)>> {
    let mut doc = load(input)?;
    if doc.is_encrypted() {
        return Err(pdf_err("O PDF está protexido con contrasinal: quítalle a protección antes."));
    }
    flatten_inherited_page_attributes(&mut doc);
    let ids = doc.get_pages();
    let mut need = Vec::new();
    for page in zone_pages(zones) {
        let id = *ids.get(&page).ok_or(EngineError::PageMissing { page, total: ids.len() as u32 })?;
        let here: Vec<&crate::pdf_redact::Zone> = zones.iter().filter(|z| z.page == page).collect();
        let rects = crate::pdf_redact::zone_rects(&doc, id, &here);
        if let Err(why) = crate::pdf_redact::clean_page(&mut doc, id, &rects) {
            need.push((page, why));
        }
    }
    Ok(need)
}

fn zone_pages(zones: &[crate::pdf_redact::Zone]) -> Vec<u32> {
    let mut p: Vec<u32> = zones.iter().map(|z| z.page).collect();
    p.sort_unstable();
    p.dedup();
    p
}

/// Tapa as zonas. As páxinas que veñen en `rasters` (xa debuxadas co negro por riba) pasan a ser unha imaxe; as demais
/// consérvanse como estaban e só se quita o texto e a parte das imaxes que cae baixo cada zona.
pub fn redact_zones(input: &Path, output: &Path, overwrite: bool, zones: &[crate::pdf_redact::Zone], rasters: &[RedactPage]) -> Result<RedactReport> {
    if zones.is_empty() {
        return Err(pdf_err("Non hai ningunha zona que tapar."));
    }
    let mut doc = open_plain(input, output)?;
    flatten_inherited_page_attributes(&mut doc);
    let ids = doc.get_pages();
    let total = ids.len() as u32;
    let mut gone: HashSet<ObjectId> = HashSet::new();
    let (mut cleaned, mut rasterized) = (0u32, 0u32);
    for page in zone_pages(zones) {
        let id = *ids.get(&page).ok_or(EngineError::PageMissing { page, total })?;
        if let Some(r) = rasters.iter().find(|r| r.page == page) {
            gone.extend(rasterize_page(&mut doc, id, &r.jpeg)?);
            rasterized += 1;
            continue;
        }
        let here: Vec<&crate::pdf_redact::Zone> = zones.iter().filter(|z| z.page == page).collect();
        let rects = crate::pdf_redact::zone_rects(&doc, id, &here);
        match crate::pdf_redact::clean_page(&mut doc, id, &rects) {
            Ok(removed) => {
                gone.extend(removed);
                cleaned += 1;
            }
            Err(why) => return Err(pdf_err(&format!("A páxina {page} non se pode tapar sen convertela en imaxe ({why}).")))
        }
    }
    let (path, fields_removed) = finish_redaction(&mut doc, &gone, output, overwrite)?;
    Ok(RedactReport { output_path: path, pages: cleaned + rasterized, fields_removed, cleaned, rasterized })
}

// ------------------------------------------------------------------ formularios

#[derive(Debug, Clone, Serialize)]
pub struct FieldOption {
    pub value: String,
    pub label: String,
}

/// Un campo de formulario tal como o ve quen o enche.
#[derive(Debug, Clone, Serialize)]
pub struct FormField {
    /// Nome completo (cos nomes dos campos pai, separados por «.»).
    pub name: String,
    /// «text», «check», «radio» ou «choice».
    pub kind: String,
    /// Texto; en «check» «true» ou «false»; en «radio» o valor elixido ou baleiro.
    pub value: String,
    /// Opcións dunha lista, ou valores dun grupo de opcións (radio). En «check», a opción 0 é o valor de marcado.
    pub options: Vec<FieldOption>,
    pub page: u32,
    pub rect: [f32; 4],
    pub read_only: bool,
    pub multiline: bool,
}

struct Leaf {
    id: ObjectId,
    name: String,
    ft: Vec<u8>,
    ff: i64,
    widgets: Vec<ObjectId>,
}

fn walk_fields(doc: &Document, id: ObjectId, prefix: &str, ft: &[u8], ff: i64, out: &mut Vec<Leaf>, depth: u32) {
    if depth > 16 {
        return;
    }
    let Ok(d) = doc.get_dictionary(id) else { return };
    let name = match d.get(b"T").ok().map(|o| deref(doc, o)) {
        Some(Object::String(b, _)) => {
            let t = pdf_text(b);
            if prefix.is_empty() { t } else { format!("{prefix}.{t}") }
        }
        _ => prefix.to_string(),
    };
    let ft: Vec<u8> = d.get(b"FT").ok().map(|o| name_of(Some(deref(doc, o)))).filter(|n| !n.is_empty()).unwrap_or_else(|| ft.to_vec());
    let ff = d.get(b"Ff").ok().map(|o| num(deref(doc, o)) as i64).unwrap_or(ff);
    let kids: Vec<Object> = d.get(b"Kids").ok().map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()).cloned().unwrap_or_default();
    if kids.is_empty() {
        out.push(Leaf { id, name, ft, ff, widgets: vec![id] });
        return;
    }
    let mut widgets = Vec::new();
    for k in &kids {
        let Object::Reference(kid) = k else { continue };
        let named = doc.get_dictionary(*kid).map(|kd| kd.has(b"T")).unwrap_or(false);
        if named {
            walk_fields(doc, *kid, &name, &ft, ff, out, depth + 1);
        } else {
            widgets.push(*kid);
        }
    }
    if !widgets.is_empty() {
        out.push(Leaf { id, name, ft, ff, widgets });
    }
}

fn leaves(doc: &Document) -> Vec<Leaf> {
    let mut out = Vec::new();
    let Some(form) = doc.catalog().ok().and_then(|c| c.get(b"AcroForm").ok()).and_then(|f| dict_at(doc, f)) else { return out };
    let fields: Vec<Object> = form.get(b"Fields").ok().map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()).cloned().unwrap_or_default();
    for f in fields {
        if let Object::Reference(id) = f {
            walk_fields(doc, id, "", b"", 0, &mut out, 0);
        }
    }
    out
}

/// Número de páxina (desde 1) de cada widget.
fn widget_pages(doc: &Document) -> HashMap<ObjectId, u32> {
    let mut m = HashMap::new();
    for (n, p) in page_ids(doc) {
        for a in annots_of(doc, p) {
            if let Object::Reference(id) = a {
                m.insert(id, n);
            }
        }
    }
    m
}

/// O nome do estado «marcado» dun widget: a chave de `AP/N` que non é «Off».
fn on_state(doc: &Document, w: ObjectId) -> Option<String> {
    let d = doc.get_dictionary(w).ok()?;
    let ap = dict_at(doc, d.get(b"AP").ok()?)?;
    let n = dict_at(doc, ap.get(b"N").ok()?)?;
    n.iter().map(|(k, _)| String::from_utf8_lossy(k).to_string()).find(|k| k != "Off")
}

fn value_text(doc: &Document, d: &Dictionary) -> String {
    match d.get(b"V").ok().map(|o| deref(doc, o)) {
        Some(Object::String(b, _)) => pdf_text(b),
        Some(Object::Name(n)) => String::from_utf8_lossy(n).to_string(),
        _ => String::new(),
    }
}

pub fn form_fields(input: &Path) -> Result<Vec<FormField>> {
    let doc = load(input)?;
    if doc.is_encrypted() {
        return Err(pdf_err("O PDF está protexido con contrasinal: quítalle a protección antes."));
    }
    let pages = widget_pages(&doc);
    let mut out = Vec::new();
    for l in leaves(&doc) {
        let Ok(d) = doc.get_dictionary(l.id) else { continue };
        let kind = match l.ft.as_slice() {
            b"Tx" => "text",
            b"Ch" => "choice",
            b"Btn" if l.ff & (1 << 16) != 0 => continue, // botón de acción
            b"Btn" if l.ff & (1 << 15) != 0 => "radio",
            b"Btn" => "check",
            _ => continue, // firmas e descoñecidos
        };
        let first = l.widgets.first().copied().unwrap_or(l.id);
        let wd = doc.get_dictionary(first).ok();
        let rect = wd.and_then(|w| rect_of(&doc, w)).unwrap_or([0.0; 4]);
        let raw = value_text(&doc, d);
        let mut options: Vec<FieldOption> = Vec::new();
        let value = match kind {
            "check" => {
                let on = l.widgets.iter().find_map(|w| on_state(&doc, *w)).unwrap_or_else(|| "Yes".to_string());
                options.push(FieldOption { value: on.clone(), label: on.clone() });
                (raw == on || (!raw.is_empty() && raw != "Off")).to_string()
            }
            "radio" => {
                for w in &l.widgets {
                    if let Some(s) = on_state(&doc, *w) {
                        options.push(FieldOption { value: s.clone(), label: s });
                    }
                }
                if raw == "Off" { String::new() } else { raw }
            }
            _ => {
                if kind == "choice" {
                    if let Some(a) = d.get(b"Opt").ok().map(|o| deref(&doc, o)).and_then(|o| o.as_array().ok()) {
                        for o in a {
                            match deref(&doc, o) {
                                Object::String(b, _) => {
                                    let t = pdf_text(b);
                                    options.push(FieldOption { value: t.clone(), label: t });
                                }
                                Object::Array(p) if p.len() == 2 => {
                                    let s = |x: &Object| match deref(&doc, x) {
                                        Object::String(b, _) => pdf_text(b),
                                        _ => String::new(),
                                    };
                                    options.push(FieldOption { value: s(&p[0]), label: s(&p[1]) });
                                }
                                _ => {}
                            }
                        }
                    }
                }
                raw
            }
        };
        out.push(FormField {
            name: l.name.clone(),
            kind: kind.to_string(),
            value,
            options,
            page: pages.get(&first).copied().unwrap_or(1),
            rect,
            read_only: l.ff & 1 != 0,
            multiline: kind == "text" && l.ff & (1 << 12) != 0,
        });
    }
    Ok(out)
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldValue {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FillReport {
    pub output_path: PathBuf,
    pub filled: u32,
    pub flattened: bool,
}

fn latin1(s: &str) -> Vec<u8> {
    crate::pdf::escape_pdf_line(s)
}

/// Tamaño de letra do campo: o do seu `DA`, ou o que caiba se é 0 («automático»).
fn font_size(doc: &Document, widget: &Dictionary, field: &Dictionary, form: &Dictionary, h: f32, multiline: bool) -> f32 {
    let da = [widget, field, form].iter().find_map(|d| match d.get(b"DA").ok().map(|o| deref(doc, o)) {
        Some(Object::String(b, _)) => Some(pdf_text(b)),
        _ => None,
    });
    let size = da
        .and_then(|s| {
            let t: Vec<&str> = s.split_whitespace().collect();
            t.iter().position(|x| *x == "Tf").and_then(|i| i.checked_sub(1)).and_then(|i| t[i].parse::<f32>().ok())
        })
        .unwrap_or(0.0);
    if size > 0.0 { size } else if multiline { 12.0 } else { (h - 4.0).clamp(6.0, 12.0) }
}

/// Aparencia dun campo de texto: unha pequena páxina co texto dentro da caixa.
fn text_appearance(doc: &mut Document, font: ObjectId, size: f32, w: f32, h: f32, text: &str, multiline: bool, quad: i64) -> ObjectId {
    let mut c: Vec<u8> = b"/Tx BMC q BT /FTrasnoForm ".to_vec();
    c.extend(format!("{size} Tf 0 g ").into_bytes());
    let lines: Vec<&str> = if multiline { text.lines().collect() } else { vec![text.lines().next().unwrap_or("")] };
    let lh = size * 1.15;
    for (i, line) in lines.iter().enumerate() {
        let y = if multiline { h - 2.0 - size - i as f32 * lh } else { (h - size) / 2.0 + size * 0.22 };
        let tw = line.chars().count() as f32 * size * 0.5;
        let x = match quad {
            1 => ((w - tw) / 2.0).max(2.0),
            2 => (w - tw - 2.0).max(2.0),
            _ => 2.0,
        };
        // Td é relativo á posición anterior: cada liña vai con Tm absoluto.
        c.extend(format!("1 0 0 1 {x} {y} Tm (").into_bytes());
        c.extend(latin1(line));
        c.extend(b") Tj ");
    }
    c.extend(b"ET Q EMC");
    let mut s = Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Form",
            "BBox" => vec![0.into(), 0.into(), w.into(), h.into()],
            "Resources" => dictionary! { "Font" => dictionary! { "FTrasnoForm" => Object::Reference(font) } },
        },
        c,
    );
    s.allows_compression = false;
    doc.add_object(s)
}

/// Engade `content` por riba da páxina e rexistra os obxectos de formulario (`XObject`) que usa.
fn append_content(doc: &mut Document, page: ObjectId, content: Vec<u8>, xobjs: &[(String, ObjectId)]) -> Result<()> {
    let pd = doc.get_dictionary(page).map_err(lerr)?.clone();
    let mut res = pd.get(b"Resources").ok().and_then(|o| dict_at(doc, o)).unwrap_or_default();
    let mut xs = res.get(b"XObject").ok().and_then(|o| dict_at(doc, o)).unwrap_or_default();
    for (n, id) in xobjs {
        xs.set(n.as_bytes().to_vec(), Object::Reference(*id));
    }
    res.set("XObject", Object::Dictionary(xs));
    let q = doc.add_object(Stream::new(dictionary! {}, b"q".to_vec()));
    let qq = doc.add_object(Stream::new(dictionary! {}, b"Q".to_vec()));
    let mine = doc.add_object(Stream::new(dictionary! {}, content));
    let mut parts: Vec<Object> = vec![Object::Reference(q)];
    match pd.get(b"Contents") {
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(Object::Array(a)) => parts.extend(a.iter().cloned()),
            _ => parts.push(Object::Reference(*id)),
        },
        Ok(Object::Array(a)) => parts.extend(a.iter().cloned()),
        _ => {}
    }
    parts.push(Object::Reference(qq));
    parts.push(Object::Reference(mine));
    let d = doc.get_dictionary_mut(page).map_err(lerr)?;
    d.set("Resources", Object::Dictionary(res));
    d.set("Contents", Object::Array(parts));
    Ok(())
}

/// Enche campos polo seu nome. Con `flatten`, o que se escribiu pasa a ser parte da páxina e o formulario desaparece.
pub fn fill_form(input: &Path, output: &Path, overwrite: bool, values: &[FieldValue], flatten: bool) -> Result<FillReport> {
    let mut doc = open_plain(input, output)?;
    flatten_inherited_page_attributes(&mut doc);
    let all = leaves(&doc);
    if all.is_empty() {
        return Err(pdf_err("Este PDF non ten campos de formulario."));
    }
    let wanted: BTreeMap<&str, &str> = values.iter().map(|v| (v.name.as_str(), v.value.as_str())).collect();
    let form = doc.catalog().ok().and_then(|c| c.get(b"AcroForm").ok()).and_then(|f| dict_at(&doc, f)).unwrap_or_default();
    let font = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding" });
    let mut filled = 0;

    for l in &all {
        let Some(&val) = wanted.get(l.name.as_str()) else { continue };
        let field = doc.get_dictionary(l.id).map_err(lerr)?.clone();
        if l.ff & 1 != 0 {
            continue; // só lectura
        }
        match l.ft.as_slice() {
            b"Tx" | b"Ch" => {
                let multiline = l.ft == b"Tx" && l.ff & (1 << 12) != 0;
                let shown = if l.ft == b"Ch" {
                    // Mostra a etiqueta da opción elixida.
                    field.get(b"Opt").ok().map(|o| deref(&doc, o)).and_then(|o| o.as_array().ok()).and_then(|a| {
                        a.iter().find_map(|o| match deref(&doc, o) {
                            Object::Array(p) if p.len() == 2 => match (deref(&doc, &p[0]), deref(&doc, &p[1])) {
                                (Object::String(k, _), Object::String(lb, _)) if pdf_text(k) == val => Some(pdf_text(lb)),
                                _ => None,
                            },
                            _ => None,
                        })
                    }).unwrap_or_else(|| val.to_string())
                } else {
                    val.to_string()
                };
                doc.get_dictionary_mut(l.id).map_err(lerr)?.set("V", text_obj(val));
                let quad = field.get(b"Q").ok().map(|o| num(deref(&doc, o)) as i64).unwrap_or(0);
                for w in &l.widgets {
                    let wd = doc.get_dictionary(*w).map_err(lerr)?.clone();
                    let Some(r) = rect_of(&doc, &wd) else { continue };
                    let (bw, bh) = (r[2] - r[0], r[3] - r[1]);
                    let size = font_size(&doc, &wd, &field, &form, bh, multiline);
                    let ap = text_appearance(&mut doc, font, size, bw, bh, &shown, multiline, quad);
                    doc.get_dictionary_mut(*w).map_err(lerr)?.set("AP", Object::Dictionary(dictionary! { "N" => Object::Reference(ap) }));
                }
                filled += 1;
            }
            b"Btn" if l.ff & (1 << 16) == 0 => {
                let radio = l.ff & (1 << 15) != 0;
                let ons: Vec<Option<String>> = l.widgets.iter().map(|w| on_state(&doc, *w)).collect();
                let target: Option<String> = if radio {
                    if val.is_empty() { None } else { Some(val.to_string()) }
                } else if val == "true" {
                    ons.iter().flatten().next().cloned().or(Some("Yes".to_string()))
                } else {
                    None
                };
                doc.get_dictionary_mut(l.id).map_err(lerr)?.set("V", Object::Name(target.clone().unwrap_or_else(|| "Off".into()).into_bytes()));
                for (w, on) in l.widgets.iter().zip(&ons) {
                    let state = match (&target, on) {
                        (Some(t), Some(o)) if t == o => t.clone(),
                        (Some(t), None) if !radio => t.clone(),
                        _ => "Off".to_string(),
                    };
                    doc.get_dictionary_mut(*w).map_err(lerr)?.set("AS", Object::Name(state.into_bytes()));
                }
                filled += 1;
            }
            _ => {}
        }
    }
    if filled == 0 {
        return Err(pdf_err("Ningún dos campos indicados se pode encher."));
    }

    if flatten {
        let pages = widget_pages(&doc);
        let mut all_widgets: HashSet<ObjectId> = HashSet::new();
        let mut per_page: BTreeMap<u32, Vec<(ObjectId, ObjectId, [f32; 4])>> = BTreeMap::new(); // (aparencia, widget, rect)
        for l in &all {
            for w in &l.widgets {
                all_widgets.insert(*w);
                let wd = doc.get_dictionary(*w).map_err(lerr)?.clone();
                let hidden = wd.get(b"F").ok().map(|o| num(deref(&doc, o)) as i64).unwrap_or(0) & 2 != 0;
                let Some(r) = rect_of(&doc, &wd) else { continue };
                let Some(ap) = wd.get(b"AP").ok().and_then(|o| dict_at(&doc, o)) else { continue };
                let n = ap.get(b"N").ok();
                let stream = match n {
                    Some(Object::Reference(id)) if matches!(doc.get_object(*id), Ok(Object::Stream(_))) => Some(*id),
                    Some(o) => dict_at(&doc, o).and_then(|states| {
                        let st = name_of(wd.get(b"AS").ok());
                        states.get(&st).ok().and_then(|o| o.as_reference().ok())
                    }),
                    None => None,
                };
                if let (Some(s), false) = (stream, hidden) {
                    per_page.entry(*pages.get(w).unwrap_or(&1)).or_default().push((s, *w, r));
                }
            }
        }
        let ids = doc.get_pages();
        for (pn, items) in per_page {
            let Some(&pid) = ids.get(&pn) else { continue };
            let mut content: Vec<u8> = Vec::new();
            let mut xo: Vec<(String, ObjectId)> = Vec::new();
            for (i, (ap, _, r)) in items.iter().enumerate() {
                let bbox = doc.get_object(*ap).ok().and_then(|o| if let Object::Stream(s) = o { s.dict.get(b"BBox").ok().cloned() } else { None });
                let b: Vec<f32> = bbox.as_ref().and_then(|o| deref(&doc, o).as_array().ok()).map(|a| a.iter().map(|x| num(deref(&doc, x))).collect()).unwrap_or_else(|| vec![0.0, 0.0, r[2] - r[0], r[3] - r[1]]);
                if b.len() != 4 || (b[2] - b[0]).abs() < 0.01 || (b[3] - b[1]).abs() < 0.01 {
                    continue;
                }
                let (sx, sy) = ((r[2] - r[0]) / (b[2] - b[0]), (r[3] - r[1]) / (b[3] - b[1]));
                let (tx, ty) = (r[0] - b[0] * sx, r[1] - b[1] * sy);
                let name = format!("FmTrasno{i}");
                content.extend(format!("q {sx} 0 0 {sy} {tx} {ty} cm /{name} Do Q\n").into_bytes());
                xo.push((name, *ap));
            }
            if !xo.is_empty() {
                append_content(&mut doc, pid, content, &xo)?;
            }
        }
        // Os widgets e o formulario fóra.
        for (_, p) in page_ids(&doc) {
            let keep: Vec<Object> = annots_of(&doc, p).into_iter().filter(|a| !matches!(a, Object::Reference(id) if all_widgets.contains(id))).collect();
            set_annots(&mut doc, p, keep);
        }
        doc.catalog_mut().map_err(lerr)?.remove(b"AcroForm");
    } else {
        let id = doc.catalog().ok().and_then(|c| c.get(b"AcroForm").ok()).and_then(|a| a.as_reference().ok());
        let t: Option<&mut Dictionary> = match id {
            Some(id) => doc.get_dictionary_mut(id).ok(),
            None => doc.catalog_mut().ok().and_then(|c| c.get_mut(b"AcroForm").ok()).and_then(|f| f.as_dict_mut().ok()),
        };
        if let Some(t) = t {
            t.set("NeedAppearances", Object::Boolean(true));
        }
    }
    doc.prune_objects();
    let path = save_atomically(&mut doc, output, overwrite)?;
    Ok(FillReport { output_path: path, filled, flattened: flatten })
}

// ---------------------------------------------------------------------- anexos

/// Un ficheiro metido dentro do PDF.
#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    pub name: String,
    pub size: u64,
}

/// Parella (clave da árbore de nomes, obxecto do ficheiro) de todos os anexos, mesmo se a árbore ten niveis.
fn embedded_entries(doc: &Document) -> Vec<(Vec<u8>, Object)> {
    fn walk(doc: &Document, node: &Dictionary, out: &mut Vec<(Vec<u8>, Object)>, depth: u32) {
        if depth > 16 {
            return;
        }
        if let Some(a) = node.get(b"Names").ok().map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()) {
            for pair in a.chunks(2) {
                if let [k, v] = pair {
                    if let Object::String(b, _) = deref(doc, k) {
                        out.push((b.clone(), v.clone()));
                    }
                }
            }
        }
        if let Some(kids) = node.get(b"Kids").ok().map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()) {
            for k in kids {
                if let Some(d) = dict_at(doc, k) {
                    walk(doc, &d, out, depth + 1);
                }
            }
        }
    }
    let mut out = Vec::new();
    let root = doc.catalog().ok().and_then(|c| c.get(b"Names").ok()).and_then(|n| dict_at(doc, n)).and_then(|n| n.get(b"EmbeddedFiles").ok().and_then(|e| dict_at(doc, e)));
    if let Some(r) = root {
        walk(doc, &r, &mut out, 0);
    }
    out
}

pub fn list_attachments(input: &Path) -> Result<Vec<Attachment>> {
    let doc = load(input)?;
    if doc.is_encrypted() {
        return Err(pdf_err("O PDF está protexido con contrasinal: quítalle a protección antes."));
    }
    Ok(embedded_entries(&doc)
        .into_iter()
        .map(|(k, v)| {
            let size = dict_at(&doc, &v)
                .and_then(|d| d.get(b"EF").ok().and_then(|e| dict_at(&doc, e)))
                .and_then(|ef| ef.get(b"F").ok().cloned())
                .and_then(|f| match deref(&doc, &f) {
                    Object::Stream(s) => Some(s.dict.get(b"Params").ok().and_then(|p| dict_at(&doc, p)).and_then(|p| p.get(b"Size").ok().map(|n| num(n) as u64)).unwrap_or(s.content.len() as u64)),
                    _ => None,
                })
                .unwrap_or(0);
            Attachment { name: pdf_text(&k), size }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachReport {
    pub output_path: PathBuf,
    pub added: u32,
    pub removed: u32,
}

/// Un anexo non pode pasar disto: o PDF cargaríao enteiro en memoria.
pub const MAX_ATTACHMENT: u64 = 200 * 1024 * 1024;

fn text_obj_bytes(s: &str) -> Vec<u8> {
    match text_obj(s) {
        Object::String(b, _) => b,
        _ => Vec::new(),
    }
}

/// Engade ficheiros como anexos e/ou quita os que teñen eses nomes. O resto do PDF queda igual.
pub fn edit_attachments(input: &Path, output: &Path, overwrite: bool, add: &[PathBuf], remove: &[String]) -> Result<AttachReport> {
    if add.is_empty() && remove.is_empty() {
        return Err(pdf_err("Non hai ningún anexo que engadir nin quitar."));
    }
    let mut doc = open_plain(input, output)?;
    let mut entries: Vec<(Vec<u8>, Object)> = embedded_entries(&doc);
    let before = entries.len();
    entries.retain(|(k, _)| !remove.iter().any(|r| *r == pdf_text(k)));
    let removed = (before - entries.len()) as u32;
    let mut added = 0u32;
    for path in add {
        if !path.is_file() {
            return Err(EngineError::InputNotFound(path.clone()));
        }
        if std::fs::metadata(path)?.len() > MAX_ATTACHMENT {
            return Err(pdf_err("Un anexo non pode pasar de 200 MB."));
        }
        let data = std::fs::read(path)?;
        let base = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "anexo".into());
        // Sen repetir nome dentro do PDF: « (2)» ao final, antes da extensión.
        let (stem, ext) = match base.rfind('.') {
            Some(i) if i > 0 => (base[..i].to_string(), base[i..].to_string()),
            _ => (base.clone(), String::new()),
        };
        let mut name = base.clone();
        let mut k = 2;
        while entries.iter().any(|(key, _)| pdf_text(key) == name) {
            name = format!("{stem} ({k}){ext}");
            k += 1;
        }
        let mut file = Stream::new(dictionary! { "Type" => "EmbeddedFile", "Params" => dictionary! { "Size" => data.len() as i64 } }, data);
        let _ = file.compress();
        let file_id = doc.add_object(file);
        let spec = doc.add_object(dictionary! {
            "Type" => "Filespec",
            "F" => text_obj(&name),
            "UF" => text_obj(&name),
            "EF" => dictionary! { "F" => Object::Reference(file_id) },
        });
        entries.push((text_obj_bytes(&name), Object::Reference(spec)));
        added += 1;
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Árbore de nomes plana nova co que queda.
    let mut flat: Vec<Object> = Vec::new();
    for (k, v) in &entries {
        flat.push(Object::String(k.clone(), StringFormat::Literal));
        flat.push(v.clone());
    }
    let ef = if flat.is_empty() { None } else { Some(Object::Dictionary(dictionary! { "Names" => Object::Array(flat) })) };
    let names_ref = doc.catalog().ok().and_then(|c| c.get(b"Names").ok()).and_then(|n| n.as_reference().ok());
    let target: Option<&mut Dictionary> = match names_ref {
        Some(id) => doc.get_dictionary_mut(id).ok(),
        None => {
            let c = doc.catalog_mut().map_err(lerr)?;
            if !c.has(b"Names") {
                c.set("Names", Object::Dictionary(Dictionary::new()));
            }
            c.get_mut(b"Names").ok().and_then(|n| n.as_dict_mut().ok())
        }
    };
    if let Some(t) = target {
        match ef {
            Some(e) => t.set("EmbeddedFiles", e),
            None => {
                t.remove(b"EmbeddedFiles");
            }
        }
    }
    doc.prune_objects();
    let path = save_atomically(&mut doc, output, overwrite)?;
    Ok(AttachReport { output_path: path, added, removed })
}
