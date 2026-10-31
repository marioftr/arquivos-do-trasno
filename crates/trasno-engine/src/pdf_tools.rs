//! Ferramentas de PDF máis alá de unir/extraer/rotar: marcas de auga, números
//! de páxina, texto e imaxes (firma), recorte, compresión, contrasinal,
//! reordenar e reparar. Sempre se escribe un ficheiro novo, de forma atómica.

use crate::error::{EngineError, Result};
use crate::integrity::ensure_distinct;
use crate::pdf::{flatten_inherited_page_attributes, load, save_atomically};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::path::{Path, PathBuf};

fn pdf_err(msg: &str) -> EngineError {
    EngineError::Pdf(msg.to_string())
}

fn num(o: &Object) -> f32 {
    match o {
        Object::Integer(i) => *i as f32,
        Object::Real(r) => *r,
        _ => 0.0,
    }
}

fn resolve<'a>(doc: &'a Document, o: &'a Object) -> &'a Object {
    match o {
        Object::Reference(id) => doc.get_object(*id).unwrap_or(o),
        _ => o,
    }
}

/// (esquerda, abaixo, dereita, arriba) da área visible da páxina.
fn page_box(doc: &Document, page: ObjectId) -> (f32, f32, f32, f32) {
    let dict = doc.get_object(page).ok().and_then(|o| o.as_dict().ok());
    for key in [&b"CropBox"[..], &b"MediaBox"[..]] {
        if let Some(Object::Array(a)) = dict.and_then(|d| d.get(key).ok()).map(|o| resolve(doc, o)) {
            if a.len() == 4 {
                let v: Vec<f32> = a.iter().map(|x| num(resolve(doc, x))).collect();
                return (v[0].min(v[2]), v[1].min(v[3]), v[0].max(v[2]), v[1].max(v[3]));
            }
        }
    }
    (0.0, 0.0, 595.0, 842.0)
}

fn select_pages(doc: &Document, pages: &Option<Vec<u32>>) -> Result<Vec<(u32, ObjectId)>> {
    let all = doc.get_pages();
    let total = all.len() as u32;
    match pages {
        None => Ok(all.into_iter().collect()),
        Some(list) => {
            let mut out = Vec::new();
            for &n in list {
                match all.get(&n) {
                    Some(id) => out.push((n, *id)),
                    None => return Err(EngineError::PageMissing { page: n, total }),
                }
            }
            Ok(out)
        }
    }
}

fn parse_color(hex: &str) -> (f32, f32, f32) {
    let h = hex.trim_start_matches('#');
    if h.len() == 6 {
        if let Ok(v) = u32::from_str_radix(h, 16) {
            return (((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0);
        }
    }
    (0.0, 0.0, 0.0)
}

/// Obxectos compartidos por todas as sobreposicións dun documento.
struct Ctx {
    font: ObjectId,
    q: ObjectId,
    qq: ObjectId,
}

impl Ctx {
    fn new(doc: &mut Document) -> Self {
        let font = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding" });
        let q = doc.add_object(Stream::new(dictionary! {}, b"q".to_vec()));
        let qq = doc.add_object(Stream::new(dictionary! {}, b"Q".to_vec()));
        Ctx { font, q, qq }
    }
}

fn sub_dict(doc: &Document, res: &Dictionary, key: &[u8]) -> Dictionary {
    match res.get(key) {
        Ok(o) => match resolve(doc, o) {
            Object::Dictionary(d) => d.clone(),
            _ => Dictionary::new(),
        },
        Err(_) => Dictionary::new(),
    }
}

/// Engade `content` por riba do contido da páxina (illado con q/Q para non
/// herdar estados gráficos) e rexistra os recursos que usa.
fn add_overlay(
    doc: &mut Document,
    ctx: &Ctx,
    page: ObjectId,
    content: String,
    alpha: Option<f32>,
    image: Option<ObjectId>,
) -> Result<()> {
    let page_dict = doc.get_object(page).map_err(|e| EngineError::Pdf(e.to_string()))?.as_dict().map_err(|e| EngineError::Pdf(e.to_string()))?.clone();

    let mut res = match page_dict.get(b"Resources") {
        Ok(o) => match resolve(doc, o) {
            Object::Dictionary(d) => d.clone(),
            _ => Dictionary::new(),
        },
        Err(_) => Dictionary::new(),
    };
    let mut fonts = sub_dict(doc, &res, b"Font");
    fonts.set("FTrasno", Object::Reference(ctx.font));
    res.set("Font", Object::Dictionary(fonts));
    if let Some(a) = alpha {
        let gs = doc.add_object(dictionary! { "Type" => "ExtGState", "ca" => a, "CA" => a });
        let mut states = sub_dict(doc, &res, b"ExtGState");
        states.set("GSTrasno", Object::Reference(gs));
        res.set("ExtGState", Object::Dictionary(states));
    }
    if let Some(img) = image {
        let mut xs = sub_dict(doc, &res, b"XObject");
        xs.set("ImTrasno", Object::Reference(img));
        res.set("XObject", Object::Dictionary(xs));
    }

    let mine = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
    let mut parts: Vec<Object> = vec![Object::Reference(ctx.q)];
    match page_dict.get(b"Contents") {
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(Object::Array(a)) => parts.extend(a.iter().cloned()),
            _ => parts.push(Object::Reference(*id)),
        },
        Ok(Object::Array(a)) => parts.extend(a.iter().cloned()),
        _ => {}
    }
    parts.push(Object::Reference(ctx.qq));
    parts.push(Object::Reference(mine));

    let d = doc.get_object_mut(page).map_err(|e| EngineError::Pdf(e.to_string()))?.as_dict_mut().map_err(|e| EngineError::Pdf(e.to_string()))?;
    d.set("Resources", Object::Dictionary(res));
    d.set("Contents", Object::Array(parts));
    Ok(())
}

fn approx_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.52
}

fn esc(text: &str) -> String {
    // Latin-1: o resto dos caracteres vai como «?» (ver `escape_pdf_line`).
    String::from_utf8_lossy(&crate::pdf::escape_pdf_line(text).iter().map(|b| *b).collect::<Vec<u8>>()).to_string()
}

fn text_stream(text: &str, size: f32, color: (f32, f32, f32), alpha: Option<f32>, angle_deg: f32, x: f32, y: f32) -> String {
    let (s, c) = angle_deg.to_radians().sin_cos();
    let gs = if alpha.is_some() { "/GSTrasno gs " } else { "" };
    // O texto vai en bytes Latin-1: gárdase como «latin-1 → char» para que o String conserve os bytes.
    let bytes = crate::pdf::escape_pdf_line(text);
    let latin: String = bytes.iter().map(|b| *b as char).collect();
    let _ = esc;
    format!(
        "{gs}BT /FTrasno {size} Tf {r} {g} {b} rg {c} {s} {ns} {c} {x} {y} Tm ({latin}) Tj ET",
        r = color.0,
        g = color.1,
        b = color.2,
        ns = -s,
    )
}

/// Convirte o `String` con bytes Latin-1 en bytes reais ao gardar o fluxo.
fn latin1_bytes(s: &str) -> Vec<u8> {
    s.chars().map(|c| if (c as u32) < 256 { c as u32 as u8 } else { b'?' }).collect()
}

fn overlay_latin1(doc: &mut Document, ctx: &Ctx, page: ObjectId, content: String, alpha: Option<f32>, image: Option<ObjectId>) -> Result<()> {
    add_overlay(doc, ctx, page, String::new(), alpha, image)?;
    // add_overlay deixou un fluxo baleiro como último elemento: énchese con bytes Latin-1.
    let id = {
        let d = doc.get_object(page).map_err(|e| EngineError::Pdf(e.to_string()))?.as_dict().map_err(|e| EngineError::Pdf(e.to_string()))?;
        match d.get(b"Contents") {
            Ok(Object::Array(a)) => match a.last() {
                Some(Object::Reference(id)) => *id,
                _ => return Err(pdf_err("contido inesperado")),
            },
            _ => return Err(pdf_err("contido inesperado")),
        }
    };
    if let Ok(Object::Stream(s)) = doc.get_object_mut(id) {
        s.set_content(latin1_bytes(&content));
    }
    Ok(())
}

// ------------------------------------------------------------ marca de auga

pub struct WatermarkText {
    pub text: String,
    pub size: f32,
    pub opacity: f32,
    pub angle: f32,
    pub color: String,
}

pub fn watermark_text(input: &Path, output: &Path, overwrite: bool, w: &WatermarkText, pages: &Option<Vec<u32>>) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    if w.text.trim().is_empty() {
        return Err(pdf_err("Escribe o texto da marca de auga."));
    }
    let mut doc = load(input)?;
    flatten_inherited_page_attributes(&mut doc);
    let ctx = Ctx::new(&mut doc);
    for (_, id) in select_pages(&doc, pages)? {
        let (l, b, r, t) = page_box(&doc, id);
        let (cx, cy) = ((l + r) / 2.0, (b + t) / 2.0);
        let width = approx_width(&w.text, w.size);
        let (s, c) = w.angle.to_radians().sin_cos();
        let x = cx - (width / 2.0) * c + (w.size * 0.35) * s;
        let y = cy - (width / 2.0) * s - (w.size * 0.35) * c;
        let content = text_stream(&w.text, w.size, parse_color(&w.color), Some(w.opacity.clamp(0.02, 1.0)), w.angle, x, y);
        overlay_latin1(&mut doc, &ctx, id, content, Some(w.opacity.clamp(0.02, 1.0)), None)?;
    }
    save_atomically(&mut doc, output, overwrite)
}

// ---------------------------------------------------------- números de páxina

pub struct PageNumbers {
    /// "bl", "bc", "br", "tl", "tc", "tr"
    pub position: String,
    /// Admite `{n}` e `{total}`.
    pub format: String,
    pub start: i64,
    pub size: f32,
    pub margin: f32,
    pub color: String,
}

pub fn page_numbers(input: &Path, output: &Path, overwrite: bool, o: &PageNumbers, pages: &Option<Vec<u32>>) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let mut doc = load(input)?;
    flatten_inherited_page_attributes(&mut doc);
    let total = doc.get_pages().len() as i64;
    let ctx = Ctx::new(&mut doc);
    for (n, id) in select_pages(&doc, pages)? {
        let (l, b, r, t) = page_box(&doc, id);
        let label = o.format.replace("{n}", &(o.start + n as i64 - 1).to_string()).replace("{total}", &(o.start + total - 1).to_string());
        let width = approx_width(&label, o.size);
        let x = match o.position.as_bytes().get(1) {
            Some(b'l') => l + o.margin,
            Some(b'r') => r - o.margin - width,
            _ => (l + r) / 2.0 - width / 2.0,
        };
        let y = if o.position.starts_with('t') { t - o.margin - o.size } else { b + o.margin };
        let content = text_stream(&label, o.size, parse_color(&o.color), None, 0.0, x, y);
        overlay_latin1(&mut doc, &ctx, id, content, None, None)?;
    }
    save_atomically(&mut doc, output, overwrite)
}

// ---------------------------------------------------------------- engadir texto

pub struct AddText {
    pub text: String,
    /// Posición da esquina inferior esquerda do texto, en % da páxina (0-100).
    pub x_pct: f32,
    pub y_pct: f32,
    pub size: f32,
    pub color: String,
}

pub fn add_text(input: &Path, output: &Path, overwrite: bool, o: &AddText, pages: &Option<Vec<u32>>) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    if o.text.trim().is_empty() {
        return Err(pdf_err("Escribe o texto que queres engadir."));
    }
    let mut doc = load(input)?;
    flatten_inherited_page_attributes(&mut doc);
    let ctx = Ctx::new(&mut doc);
    for (_, id) in select_pages(&doc, pages)? {
        let (l, b, r, t) = page_box(&doc, id);
        let x = l + (r - l) * o.x_pct / 100.0;
        let y = b + (t - b) * o.y_pct / 100.0;
        let content = text_stream(&o.text, o.size, parse_color(&o.color), None, 0.0, x, y);
        overlay_latin1(&mut doc, &ctx, id, content, None, None)?;
    }
    save_atomically(&mut doc, output, overwrite)
}

// --------------------------------------------------------- imaxe (firma, selo)

pub struct StampImage {
    pub image: PathBuf,
    /// Centro da imaxe, en % da páxina.
    pub x_pct: f32,
    pub y_pct: f32,
    /// Ancho da imaxe en % do ancho da páxina.
    pub width_pct: f32,
    pub opacity: f32,
}

fn image_object(doc: &mut Document, path: &Path) -> Result<(ObjectId, f32)> {
    let img = image::open(path)?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    let mut alpha = Vec::with_capacity((w * h) as usize);
    let mut has_alpha = false;
    for p in rgba.pixels() {
        rgb.extend_from_slice(&p.0[..3]);
        alpha.push(p[3]);
        if p[3] != 255 {
            has_alpha = true;
        }
    }
    let mut dict = dictionary! { "Type" => "XObject", "Subtype" => "Image", "Width" => w as i64, "Height" => h as i64, "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8 };
    if has_alpha {
        let mut mask = Stream::new(
            dictionary! { "Type" => "XObject", "Subtype" => "Image", "Width" => w as i64, "Height" => h as i64, "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8 },
            alpha,
        );
        let _ = mask.compress();
        let mask_id = doc.add_object(mask);
        dict.set("SMask", Object::Reference(mask_id));
    }
    let mut s = Stream::new(dict, rgb);
    let _ = s.compress();
    Ok((doc.add_object(s), h as f32 / w as f32))
}

pub fn stamp_image(input: &Path, output: &Path, overwrite: bool, o: &StampImage, pages: &Option<Vec<u32>>) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let mut doc = load(input)?;
    flatten_inherited_page_attributes(&mut doc);
    let (img, ratio) = image_object(&mut doc, &o.image)?;
    let ctx = Ctx::new(&mut doc);
    let alpha = Some(o.opacity.clamp(0.02, 1.0));
    for (_, id) in select_pages(&doc, pages)? {
        let (l, b, r, t) = page_box(&doc, id);
        let w = (r - l) * o.width_pct / 100.0;
        let h = w * ratio;
        let cx = l + (r - l) * o.x_pct / 100.0;
        let cy = b + (t - b) * o.y_pct / 100.0;
        let content = format!("/GSTrasno gs q {w} 0 0 {h} {x} {y} cm /ImTrasno Do Q", x = cx - w / 2.0, y = cy - h / 2.0);
        add_overlay(&mut doc, &ctx, id, content, alpha, Some(img))?;
    }
    save_atomically(&mut doc, output, overwrite)
}

// ---------------------------------------------------------------------- recorte

pub fn crop_pages(input: &Path, output: &Path, overwrite: bool, left: f32, bottom: f32, right: f32, top: f32, pages: &Option<Vec<u32>>) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let mut doc = load(input)?;
    flatten_inherited_page_attributes(&mut doc);
    for (_, id) in select_pages(&doc, pages)? {
        let (l, b, r, t) = page_box(&doc, id);
        let (nl, nb, nr, nt) = (l + left.max(0.0), b + bottom.max(0.0), r - right.max(0.0), t - top.max(0.0));
        if nr - nl < 10.0 || nt - nb < 10.0 {
            return Err(pdf_err("Os márxenes que queres quitar deixan a páxina sen espazo."));
        }
        let arr = Object::Array(vec![nl.into(), nb.into(), nr.into(), nt.into()]);
        let d = doc.get_object_mut(id).map_err(|e| EngineError::Pdf(e.to_string()))?.as_dict_mut().map_err(|e| EngineError::Pdf(e.to_string()))?;
        d.set("CropBox", arr.clone());
        d.set("MediaBox", arr);
    }
    save_atomically(&mut doc, output, overwrite)
}

// ------------------------------------------------------------------ compresión

pub struct PdfCompress {
    /// Calidade JPEG das imaxes (1-100).
    pub quality: u8,
    /// Escala das imaxes en % (100 = non se reducen).
    pub scale_percent: u32,
}

fn is_dct(dict: &Dictionary) -> bool {
    match dict.get(b"Filter") {
        Ok(Object::Name(n)) => n == b"DCTDecode",
        Ok(Object::Array(a)) => a.len() == 1 && matches!(&a[0], Object::Name(n) if n == b"DCTDecode"),
        _ => false,
    }
}

/// Reduce o peso: reencóde as imaxes JPEG, comprime o resto dos fluxos e
/// quita obxectos sen usar. Devolve a ruta de saída.
pub fn compress_pdf(input: &Path, output: &Path, overwrite: bool, o: &PdfCompress) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let mut doc = load(input)?;
    let ids: Vec<ObjectId> = doc.objects.keys().copied().collect();
    for id in ids {
        let Some(Object::Stream(s)) = doc.objects.get_mut(&id) else { continue };
        let is_image = matches!(s.dict.get(b"Subtype"), Ok(Object::Name(n)) if n == b"Image");
        if !is_image || !is_dct(&s.dict) {
            continue;
        }
        let gray = matches!(s.dict.get(b"ColorSpace"), Ok(Object::Name(n)) if n == b"DeviceGray");
        let Ok(img) = image::load_from_memory_with_format(&s.content, image::ImageFormat::Jpeg) else { continue };
        let mut img = img;
        if o.scale_percent < 100 {
            let w = (img.width() * o.scale_percent / 100).max(1);
            let h = (img.height() * o.scale_percent / 100).max(1);
            img = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
        }
        let mut buf = Vec::new();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, o.quality.clamp(10, 100));
        let ok = if gray { enc.encode_image(&img.to_luma8()).is_ok() } else { enc.encode_image(&img.to_rgb8()).is_ok() };
        // Só se substitúe se de verdade pesa menos.
        if ok && buf.len() < s.content.len() {
            s.dict.set("Width", Object::Integer(img.width() as i64));
            s.dict.set("Height", Object::Integer(img.height() as i64));
            s.dict.set("Length", Object::Integer(buf.len() as i64));
            s.content = buf;
        }
    }
    doc.prune_objects();
    doc.renumber_objects();
    doc.compress();
    save_atomically(&mut doc, output, overwrite)
}

// ------------------------------------------------------------------ contrasinal

pub fn protect_pdf(input: &Path, output: &Path, overwrite: bool, user_pw: &str, owner_pw: &str, allow_print: bool, allow_copy: bool) -> Result<PathBuf> {
    use lopdf::{EncryptionState, EncryptionVersion, Permissions};
    ensure_distinct(&[input], output)?;
    if user_pw.is_empty() {
        return Err(pdf_err("Escribe un contrasinal."));
    }
    let mut doc = load(input)?;
    if doc.is_encrypted() {
        return Err(pdf_err("Este PDF xa ten contrasinal."));
    }
    if doc.trailer.get(b"ID").is_err() {
        let id = uuid::Uuid::new_v4().as_bytes().to_vec();
        doc.trailer.set("ID", Object::Array(vec![Object::string_literal(id.clone()), Object::string_literal(id)]));
    }
    let mut perms = Permissions::empty();
    if allow_print {
        perms |= Permissions::PRINTABLE | Permissions::PRINTABLE_IN_HIGH_QUALITY;
    }
    if allow_copy {
        perms |= Permissions::COPYABLE | Permissions::COPYABLE_FOR_ACCESSIBILITY;
    }
    let owner = if owner_pw.is_empty() { user_pw } else { owner_pw };
    let state = EncryptionState::try_from(EncryptionVersion::V2 { document: &doc, owner_password: owner, user_password: user_pw, key_length: 128, permissions: perms })
        .map_err(|e| EngineError::Pdf(e.to_string()))?;
    doc.encrypt(&state).map_err(|e| EngineError::Pdf(e.to_string()))?;
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    let dir = output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(".trasno-{}.pdf.part", uuid::Uuid::new_v4()));
    doc.save(&tmp).map_err(|e| EngineError::Pdf(e.to_string()))?;
    std::fs::rename(&tmp, output).map_err(EngineError::Io)?;
    Ok(output.to_path_buf())
}

/// Quita o contrasinal (hai que saber o contrasinal, non se "rompe" nada).
pub fn unprotect_pdf(input: &Path, output: &Path, overwrite: bool, password: &str) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let mut doc = load(input)?;
    if doc.is_encrypted() {
        doc.decrypt(password).map_err(|e| match e {
            lopdf::Error::Decryption(lopdf::encryption::DecryptionError::IncorrectPassword) => pdf_err("O contrasinal non é o correcto."),
            other => EngineError::Pdf(other.to_string()),
        })?;
    }
    doc.trailer.remove(b"Encrypt");
    save_atomically(&mut doc, output, overwrite)
}

pub fn is_encrypted(input: &Path) -> Result<bool> {
    let doc = load(input)?;
    Ok(doc.is_encrypted())
}

// ---------------------------------------------------------- ordenar e reparar

/// Deixa só as páxinas indicadas, na orde dada, cun xiro extra (graos) cada unha.
pub fn arrange_pages(input: &Path, output: &Path, overwrite: bool, order: &[(u32, i64)]) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    if order.is_empty() {
        return Err(pdf_err("Indica polo menos unha páxina."));
    }
    let mut doc = load(input)?;
    flatten_inherited_page_attributes(&mut doc);
    let pages = doc.get_pages();
    let total = pages.len() as u32;
    let root = doc
        .catalog()
        .and_then(|c| c.get(b"Pages"))
        .and_then(|p| p.as_reference())
        .map_err(|e| EngineError::Pdf(e.to_string()))?;
    let mut kids = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for &(n, extra) in order {
        let id = *pages.get(&n).ok_or(EngineError::PageMissing { page: n, total })?;
        if !seen.insert(n) {
            continue;
        }
        let d = doc.get_object_mut(id).map_err(|e| EngineError::Pdf(e.to_string()))?.as_dict_mut().map_err(|e| EngineError::Pdf(e.to_string()))?;
        let current = d.get(b"Rotate").ok().map(num).unwrap_or(0.0) as i64;
        d.set("Rotate", Object::Integer((current + extra).rem_euclid(360)));
        d.set("Parent", Object::Reference(root));
        kids.push(Object::Reference(id));
    }
    let count = kids.len() as i64;
    let pages_dict = doc.get_object_mut(root).map_err(|e| EngineError::Pdf(e.to_string()))?.as_dict_mut().map_err(|e| EngineError::Pdf(e.to_string()))?;
    pages_dict.set("Kids", Object::Array(kids));
    pages_dict.set("Count", Object::Integer(count));
    doc.prune_objects();
    doc.renumber_objects();
    doc.compress();
    save_atomically(&mut doc, output, overwrite)
}

/// Volve gardar o PDF limpo (reconstrúe a táboa de referencias e quita lixo).
pub fn repair_pdf(input: &Path, output: &Path, overwrite: bool) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let mut doc = load(input)?;
    doc.prune_objects();
    doc.renumber_objects();
    doc.compress();
    save_atomically(&mut doc, output, overwrite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf;

    fn sample(dir: &Path, name: &str, pages: usize) -> PathBuf {
        let p = dir.join(name);
        let text: String = (1..=pages).map(|i| format!("Pagina {i}\n")).collect::<Vec<_>>().join("\n");
        let t = dir.join(format!("{name}.txt"));
        std::fs::write(&t, text.repeat(1)).unwrap();
        // 60 liñas por páxina => cada "Pagina" ocupa unha só se hai chea; forzamos con saltos
        let long: String = (1..=pages).map(|i| format!("Pagina {i}{}", "\n".repeat(60))).collect();
        std::fs::write(&t, long).unwrap();
        pdf::text_to_pdf(&t, &p, true).unwrap();
        p
    }

    #[test]
    fn marcas_numeros_texto_e_recorte() {
        let dir = tempfile::tempdir().unwrap();
        let src = sample(dir.path(), "a.pdf", 3);
        assert!(pdf::page_count(&src).unwrap() >= 3);
        let out = dir.path().join("w.pdf");
        watermark_text(&src, &out, false, &WatermarkText { text: "BORRADOR".into(), size: 60.0, opacity: 0.3, angle: 45.0, color: "#ff0000".into() }, &None).unwrap();
        assert!(Document::load(&out).is_ok());
        let out2 = dir.path().join("n.pdf");
        page_numbers(&out, &out2, false, &PageNumbers { position: "bc".into(), format: "{n} / {total}".into(), start: 1, size: 10.0, margin: 20.0, color: "#000000".into() }, &None).unwrap();
        let out3 = dir.path().join("t.pdf");
        add_text(&out2, &out3, false, &AddText { text: "Revisado".into(), x_pct: 10.0, y_pct: 90.0, size: 14.0, color: "#0000ff".into() }, &Some(vec![1])).unwrap();
        let out4 = dir.path().join("c.pdf");
        crop_pages(&out3, &out4, false, 10.0, 10.0, 10.0, 10.0, &None).unwrap();
        assert_eq!(pdf::page_count(&out4).unwrap(), pdf::page_count(&src).unwrap());
        assert!(crop_pages(&src, &dir.path().join("x.pdf"), false, 400.0, 0.0, 400.0, 0.0, &None).is_err());
        // A marca é texto visible no PDF.
        let txt = dir.path().join("o.txt");
        pdf::pdf_to_text(&out3, &txt, false).unwrap();
        let t = std::fs::read_to_string(txt).unwrap();
        assert!(t.contains("BORRADOR") && t.contains("Revisado") && t.contains("1 / "), "{t}");
    }

    #[test]
    fn contrasinal_ida_e_volta_e_ordenar() {
        let dir = tempfile::tempdir().unwrap();
        let src = sample(dir.path(), "a.pdf", 3);
        let prot = dir.path().join("p.pdf");
        protect_pdf(&src, &prot, false, "segredo", "", true, true).unwrap();
        assert!(is_encrypted(&prot).unwrap());
        assert!(unprotect_pdf(&prot, &dir.path().join("bad.pdf"), false, "mal").is_err());
        let open = dir.path().join("u.pdf");
        unprotect_pdf(&prot, &open, false, "segredo").unwrap();
        assert!(!is_encrypted(&open).unwrap());
        let ord = dir.path().join("o.pdf");
        arrange_pages(&open, &ord, false, &[(3, 90), (1, 0)]).unwrap();
        assert_eq!(pdf::page_count(&ord).unwrap(), 2);
        assert!(arrange_pages(&open, &dir.path().join("e.pdf"), false, &[(99, 0)]).is_err());
        repair_pdf(&ord, &dir.path().join("r.pdf"), false).unwrap();
    }

    #[test]
    fn comprimir_pdf_con_imaxe_jpeg() {
        let dir = tempfile::tempdir().unwrap();
        let img = dir.path().join("i.jpg");
        let buf = image::RgbImage::from_fn(600, 600, |x, y| image::Rgb([(x ^ y) as u8, (x * 3) as u8, (y * 5) as u8]));
        image::DynamicImage::ImageRgb8(buf).save(&img).unwrap();
        let src = dir.path().join("s.pdf");
        pdf::images_to_pdf(&[img.clone()], &src, false).unwrap();
        // images_to_pdf usa RGB comprimido (Flate): para probar JPEG enchemos unha imaxe DCT á man.
        let mut doc = Document::load(&src).unwrap();
        let jpg = std::fs::read(&img).unwrap();
        for (_, o) in doc.objects.iter_mut() {
            if let Object::Stream(s) = o {
                if matches!(s.dict.get(b"Subtype"), Ok(Object::Name(n)) if n == b"Image") {
                    s.dict.set("Filter", "DCTDecode");
                    s.dict.remove(b"DecodeParms");
                    s.content = jpg.clone();
                }
            }
        }
        let dct = dir.path().join("d.pdf");
        doc.save(&dct).unwrap();
        let small = dir.path().join("small.pdf");
        compress_pdf(&dct, &small, false, &PdfCompress { quality: 30, scale_percent: 50 }).unwrap();
        let a = std::fs::metadata(&dct).unwrap().len();
        let b = std::fs::metadata(&small).unwrap().len();
        assert!(b < a, "{b} !< {a}");
    }
}
