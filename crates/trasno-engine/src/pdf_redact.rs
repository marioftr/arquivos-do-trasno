//! Tapar zonas dunha páxina de PDF sen convertela en imaxe: quítase o texto cuxas letras tocan a zona, borran os
//! píxeles das imaxes que caen debaixo e debúxase un rectángulo negro. O resto da páxina queda como estaba.
//!
//! Só se fai cando se pode garantir. Se a páxina leva algo que non se sabe tratar con certeza (unha tipografía sen anchos
//! coñecidos, un formulario con contido debaixo da zona, unha imaxe en formato raro…) devólvese un erro co motivo e quen
//! chama decide: normalmente a páxina pasa a ser unha imaxe, que tapa por completo.

use crate::pdf_more::{annots_of, deref, dict_at, num, rect_of, set_annots};
use lopdf::content::{Content, Operation};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream, StringFormat};
use serde::Deserialize;
use std::collections::HashMap;

/// Unha zona a tapar, en porcentaxe da páxina tal como se ve (orixe arriba á esquerda).
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Zone {
    pub page: u32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

type M = [f32; 6];
const IDENT: M = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
type Rect = [f32; 4];

/// `p` primeiro e despois `q` (os puntos son filas: [x y 1] × M).
fn mul(p: &M, q: &M) -> M {
    [
        p[0] * q[0] + p[1] * q[2],
        p[0] * q[1] + p[1] * q[3],
        p[2] * q[0] + p[3] * q[2],
        p[2] * q[1] + p[3] * q[3],
        p[4] * q[0] + p[5] * q[2] + q[4],
        p[4] * q[1] + p[5] * q[3] + q[5],
    ]
}
fn apply(m: &M, x: f32, y: f32) -> (f32, f32) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}
fn aabb(points: &[(f32, f32)]) -> Rect {
    let mut r = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for (x, y) in points {
        r[0] = r[0].min(*x);
        r[1] = r[1].min(*y);
        r[2] = r[2].max(*x);
        r[3] = r[3].max(*y);
    }
    r
}
fn hits(b: Rect, zones: &[Rect]) -> bool {
    zones.iter().any(|z| b[0] < z[2] && b[2] > z[0] && b[1] < z[3] && b[3] > z[1])
}

// ------------------------------------------------------------------ xeometría da páxina

struct Geo {
    x0: f32,
    y0: f32,
    w: f32,
    h: f32,
    rotate: i64,
}

fn geometry(doc: &Document, page: ObjectId) -> Geo {
    let d = doc.get_dictionary(page).ok();
    let mut b = [0.0, 0.0, 595.0, 842.0];
    for key in [&b"CropBox"[..], &b"MediaBox"[..]] {
        if let Some(a) = d.and_then(|d| d.get(key).ok()).map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()) {
            if a.len() == 4 {
                let v: Vec<f32> = a.iter().map(|x| num(deref(doc, x))).collect();
                b = [v[0].min(v[2]), v[1].min(v[3]), v[0].max(v[2]), v[1].max(v[3])];
                break;
            }
        }
    }
    let rotate = d.and_then(|d| d.get(b"Rotate").ok()).map(|o| num(deref(doc, o)) as i64).unwrap_or(0).rem_euclid(360);
    Geo { x0: b[0], y0: b[1], w: b[2] - b[0], h: b[3] - b[1], rotate: if rotate % 90 == 0 { rotate } else { 0 } }
}

/// As zonas (en % da páxina vista) pasadas ao espazo da páxina sen xirar, onde está o contido.
pub(crate) fn zone_rects(doc: &Document, page: ObjectId, zones: &[&Zone]) -> Vec<Rect> {
    let g = geometry(doc, page);
    let (wd, hd) = if g.rotate % 180 == 0 { (g.w, g.h) } else { (g.h, g.w) };
    let to_user = |px: f32, py: f32| {
        let du = px / 100.0 * wd;
        let dv = py / 100.0 * hd;
        let (a, b) = match g.rotate {
            90 => (dv, du),
            180 => (g.w - du, dv),
            270 => (g.w - dv, g.h - du),
            _ => (du, g.h - dv),
        };
        (g.x0 + a, g.y0 + b)
    };
    zones
        .iter()
        .map(|z| {
            let r = aabb(&[to_user(z.x, z.y), to_user(z.x + z.w, z.y + z.h)]);
            // Un chisco máis grande, para non deixar un fío de letra polo borde.
            [r[0] - 0.4, r[1] - 0.4, r[2] + 0.4, r[3] + 0.4]
        })
        .collect()
}

// ------------------------------------------------------------------ tipografías

enum Widths {
    Simple { first: i64, list: Vec<f32>, missing: f32 },
    Fixed(f32),
    Cid { dw: f32, w: HashMap<u32, f32> },
}
struct FontInfo {
    two_byte: bool,
    asc: f32,
    desc: f32,
    widths: Widths,
}
impl FontInfo {
    /// Ancho do glifo en «em» (1 = o tamaño da letra).
    fn width(&self, code: u32) -> f32 {
        match &self.widths {
            Widths::Fixed(w) => *w,
            Widths::Simple { first, list, missing } => {
                let i = code as i64 - first;
                let w = if i >= 0 { list.get(i as usize).copied().unwrap_or(0.0) } else { 0.0 };
                (if w > 0.0 { w } else { *missing }) / 1000.0
            }
            Widths::Cid { dw, w } => w.get(&code).copied().unwrap_or(*dw) / 1000.0,
        }
    }
}

fn load_font(doc: &Document, d: &Dictionary) -> std::result::Result<FontInfo, String> {
    let sub = match d.get(b"Subtype").ok().map(|o| deref(doc, o)) {
        Some(Object::Name(n)) => n.clone(),
        _ => Vec::new(),
    };
    let descriptor_of = |fd: &Dictionary| fd.get(b"FontDescriptor").ok().and_then(|o| dict_at(doc, o));
    let metrics = |fd: Option<&Dictionary>| -> (f32, f32) {
        let a = fd.and_then(|f| f.get(b"Ascent").ok()).map(|o| num(deref(doc, o))).filter(|v| *v > 0.0).unwrap_or(900.0);
        let b = fd.and_then(|f| f.get(b"Descent").ok()).map(|o| num(deref(doc, o)).abs()).filter(|v| *v > 0.0).unwrap_or(220.0);
        (a.clamp(700.0, 1100.0) / 1000.0, -b.clamp(150.0, 400.0) / 1000.0)
    };
    match sub.as_slice() {
        b"Type0" => {
            match d.get(b"Encoding").ok().map(|o| deref(doc, o)) {
                Some(Object::Name(n)) if n == b"Identity-H" => {}
                Some(Object::Name(n)) if n == b"Identity-V" => return Err("tipografía vertical".into()),
                _ => return Err("codificación de tipografía descoñecida".into()),
            }
            let desc = d
                .get(b"DescendantFonts")
                .ok()
                .map(|o| deref(doc, o))
                .and_then(|o| o.as_array().ok())
                .and_then(|a| a.first())
                .and_then(|o| dict_at(doc, o))
                .ok_or("tipografía sen descendente")?;
            let dw = desc.get(b"DW").ok().map(|o| num(deref(doc, o))).unwrap_or(1000.0);
            let mut w = HashMap::new();
            if let Some(arr) = desc.get(b"W").ok().map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()) {
                let mut i = 0;
                while i < arr.len() {
                    let first = num(deref(doc, &arr[i])) as i64;
                    match arr.get(i + 1).map(|o| deref(doc, o)) {
                        Some(Object::Array(list)) => {
                            for (k, v) in list.iter().enumerate() {
                                w.insert((first + k as i64) as u32, num(deref(doc, v)));
                            }
                            i += 2;
                        }
                        Some(_) => {
                            let last = num(deref(doc, &arr[i + 1])) as i64;
                            let v = arr.get(i + 2).map(|o| num(deref(doc, o))).unwrap_or(dw);
                            if last < first || last - first > 70000 {
                                return Err("anchos de tipografía raros".into());
                            }
                            for c in first..=last {
                                w.insert(c as u32, v);
                            }
                            i += 3;
                        }
                        None => break,
                    }
                }
            }
            let (asc, dsc) = metrics(descriptor_of(&desc).as_ref());
            Ok(FontInfo { two_byte: true, asc, desc: dsc, widths: Widths::Cid { dw, w } })
        }
        b"Type3" => Err("tipografía Type3".into()),
        _ => {
            let fd = descriptor_of(d);
            let (asc, dsc) = metrics(fd.as_ref());
            let missing = fd.as_ref().and_then(|f| f.get(b"MissingWidth").ok()).map(|o| num(deref(doc, o))).unwrap_or(0.0);
            let list: Option<Vec<f32>> = d.get(b"Widths").ok().map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()).map(|a| a.iter().map(|x| num(deref(doc, x))).collect());
            let widths = match list {
                Some(list) if !list.is_empty() => Widths::Simple { first: d.get(b"FirstChar").ok().map(|o| num(deref(doc, o)) as i64).unwrap_or(0), list, missing },
                _ => {
                    let base = match d.get(b"BaseFont").ok().map(|o| deref(doc, o)) {
                        Some(Object::Name(n)) => String::from_utf8_lossy(n).to_lowercase(),
                        _ => String::new(),
                    };
                    if base.contains("courier") {
                        Widths::Fixed(0.6)
                    } else {
                        return Err("tipografía sen anchos".into());
                    }
                }
            };
            Ok(FontInfo { two_byte: false, asc, desc: dsc, widths })
        }
    }
}

// ------------------------------------------------------------------ estado gráfico

#[derive(Clone)]
struct Gs {
    ctm: M,
    tc: f32,
    tw: f32,
    th: f32,
    tl: f32,
    rise: f32,
    font: Option<Vec<u8>>,
    fs: f32,
}

fn floats(ops: &[Object]) -> Option<Vec<f32>> {
    ops.iter()
        .map(|o| match o {
            Object::Integer(i) => Some(*i as f32),
            Object::Real(r) => Some(*r),
            _ => None,
        })
        .collect()
}

fn hex(b: &[u8]) -> Object {
    Object::String(b.to_vec(), StringFormat::Hexadecimal)
}

// ------------------------------------------------------------------ imaxes

/// Cantos canais ten un espazo de cor que se sabe ler (1 gris, 3 RGB).
fn channels(doc: &Document, cs: Option<&Object>) -> Option<usize> {
    match deref(doc, cs?) {
        Object::Name(n) => match n.as_slice() {
            b"DeviceGray" | b"CalGray" | b"G" => Some(1),
            b"DeviceRGB" | b"CalRGB" | b"RGB" => Some(3),
            _ => None,
        },
        Object::Array(a) => match a.first().map(|o| deref(doc, o)) {
            Some(Object::Name(n)) if n == b"ICCBased" => {
                let st = a.get(1).map(|o| deref(doc, o))?.as_stream().ok()?;
                match st.dict.get(b"N").ok().map(|o| num(deref(doc, o)) as i64) {
                    Some(1) => Some(1),
                    Some(3) => Some(3),
                    _ => None,
                }
            }
            Some(Object::Name(n)) if n == b"CalRGB" => Some(3),
            Some(Object::Name(n)) if n == b"CalGray" => Some(1),
            _ => None,
        },
        _ => None,
    }
}

/// Copia unha imaxe co negro pintado onde a tocan as zonas. Devolve o obxecto novo (o orixinal non se toca: pode usarse noutras páxinas).
fn paint_image(doc: &mut Document, id: ObjectId, ctm: &M, zones: &[Rect]) -> std::result::Result<ObjectId, String> {
    let st = doc.get_object(id).and_then(Object::as_stream).map_err(|_| "imaxe ilexible")?.clone();
    let d = &st.dict;
    let int = |k: &[u8]| d.get(k).ok().map(|o| num(deref(doc, o)) as i64);
    let (w, h) = (int(b"Width").ok_or("imaxe sen tamaño")? as usize, int(b"Height").ok_or("imaxe sen tamaño")? as usize);
    if w == 0 || h == 0 || w.checked_mul(h).map(|n| n > 120_000_000).unwrap_or(true) {
        return Err("imaxe demasiado grande".into());
    }
    if d.has(b"Decode") || d.get(b"ImageMask").ok().map(|o| matches!(deref(doc, o), Object::Boolean(true))).unwrap_or(false) {
        return Err("imaxe con máscara ou Decode".into());
    }
    if int(b"BitsPerComponent").unwrap_or(8) != 8 {
        return Err("imaxe de menos de 8 bits".into());
    }
    let n = channels(doc, d.get(b"ColorSpace").ok()).ok_or("espazo de cor da imaxe non soportado")?;
    let filters: Vec<Vec<u8>> = match st.filters() {
        Ok(f) => f.into_iter().map(|x| x.to_vec()).collect(),
        Err(_) => Vec::new(),
    };
    let jpeg = filters.len() == 1 && filters[0] == b"DCTDecode";
    if !jpeg && !(filters.is_empty() || (filters.len() == 1 && filters[0] == b"FlateDecode")) {
        return Err("filtro de imaxe non soportado".into());
    }
    let mut px: Vec<u8> = if jpeg {
        let img = image::load_from_memory_with_format(&st.content, image::ImageFormat::Jpeg).map_err(|_| "JPEG ilexible")?;
        if img.width() as usize != w || img.height() as usize != h {
            return Err("JPEG de tamaño distinto do declarado".into());
        }
        if n == 1 { img.to_luma8().into_raw() } else { img.to_rgb8().into_raw() }
    } else if filters.is_empty() {
        st.content.clone()
    } else {
        st.decompressed_content().map_err(|_| "imaxe ilexible")?
    };
    if px.len() != w * h * n {
        return Err("datos de imaxe de tamaño raro".into());
    }

    // Que píxeles tocan as zonas: cada centro de píxel pásase ao espazo da páxina.
    let det = ctm[0] * ctm[3] - ctm[1] * ctm[2];
    if det.abs() < 1e-9 {
        return Err("imaxe sen superficie".into());
    }
    let inv: M = [ctm[3] / det, -ctm[1] / det, -ctm[2] / det, ctm[0] / det, (ctm[2] * ctm[5] - ctm[3] * ctm[4]) / det, (ctm[1] * ctm[4] - ctm[0] * ctm[5]) / det];
    let mut uv = Vec::new();
    for z in zones {
        for (x, y) in [(z[0], z[1]), (z[2], z[1]), (z[0], z[3]), (z[2], z[3])] {
            uv.push(apply(&inv, x, y));
        }
    }
    let b = aabb(&uv);
    let i0 = ((b[0] * w as f32).floor() as i64 - 1).clamp(0, w as i64) as usize;
    let i1 = ((b[2] * w as f32).ceil() as i64 + 1).clamp(0, w as i64) as usize;
    let j0 = (((1.0 - b[3]) * h as f32).floor() as i64 - 1).clamp(0, h as i64) as usize;
    let j1 = (((1.0 - b[1]) * h as f32).ceil() as i64 + 1).clamp(0, h as i64) as usize;
    for j in j0..j1 {
        let v = 1.0 - (j as f32 + 0.5) / h as f32;
        for i in i0..i1 {
            let u = (i as f32 + 0.5) / w as f32;
            let (x, y) = apply(ctm, u, v);
            if zones.iter().any(|z| x >= z[0] && x <= z[2] && y >= z[1] && y <= z[3]) {
                let at = (j * w + i) * n;
                px[at..at + n].fill(0);
            }
        }
    }

    let mut out = st.clone();
    if jpeg {
        let mut buf = Vec::new();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 95);
        let ok = if n == 1 {
            enc.encode(&px, w as u32, h as u32, image::ExtendedColorType::L8).is_ok()
        } else {
            enc.encode(&px, w as u32, h as u32, image::ExtendedColorType::Rgb8).is_ok()
        };
        if !ok {
            return Err("non puiden volver gardar a imaxe".into());
        }
        out.set_content(buf);
        out.allows_compression = false;
    } else {
        out.dict.remove(b"DecodeParms");
        out.dict.remove(b"Filter");
        out.set_content(px);
        out.allows_compression = true;
        let _ = out.compress();
    }
    // A máscara suave (se hai) queda: o negro vai nos píxeles.
    Ok(doc.add_object(Object::Stream(out)))
}

// ------------------------------------------------------------------ páxina

/// Tapa as zonas dunha páxina (`rects`, no espazo da páxina sen xirar). Devolve as anotacións que se quitaron.
/// Se a páxina leva algo que non se pode tratar con certeza, devolve o motivo e non deixa a páxina a medio cambiar.
pub(crate) fn clean_page(doc: &mut Document, page: ObjectId, rects: &[Rect]) -> std::result::Result<Vec<ObjectId>, String> {
    // Contido: todos os fluxos da páxina, xa sen filtros.
    let mut data = Vec::new();
    for id in doc.get_page_contents(page) {
        let st = doc.get_object(id).and_then(Object::as_stream).map_err(|_| "contido ilexible")?;
        let part = match st.filters() {
            Ok(f) if !f.is_empty() => st.decompressed_content().map_err(|_| "contido con filtro non soportado")?,
            _ => st.content.clone(),
        };
        data.extend(part);
        data.push(b'\n');
    }
    let content = Content::decode(&data).map_err(|_| "contido ilexible")?;

    // Recursos: tipografías e obxectos externos desta páxina (a copia propia permite engadir sen tocar outras páxinas).
    let mut res = doc.get_dictionary(page).ok().and_then(|d| d.get(b"Resources").ok()).and_then(|o| dict_at(doc, o)).unwrap_or_default();
    let mut fonts: HashMap<Vec<u8>, std::result::Result<FontInfo, String>> = HashMap::new();
    if let Some(fd) = res.get(b"Font").ok().and_then(|o| dict_at(doc, o)) {
        for (name, o) in fd.iter() {
            fonts.insert(name.clone(), dict_at(doc, o).ok_or_else(|| "tipografía ilexible".to_string()).and_then(|d| load_font(doc, &d)));
        }
    }
    let mut xobj = res.get(b"XObject").ok().and_then(|o| dict_at(doc, o)).unwrap_or_default();

    let mut gs = Gs { ctm: IDENT, tc: 0.0, tw: 0.0, th: 1.0, tl: 0.0, rise: 0.0, font: None, fs: 0.0 };
    let mut stack: Vec<Gs> = Vec::new();
    let (mut tm, mut tlm) = (IDENT, IDENT);
    let mut out: Vec<Operation> = vec![Operation::new("q", vec![])];
    let mut made = 0;

    for op in &content.operations {
        let args = &op.operands;
        match op.operator.as_str() {
            "q" => {
                stack.push(gs.clone());
                out.push(op.clone());
            }
            "Q" => {
                if let Some(g) = stack.pop() {
                    gs = g;
                }
                out.push(op.clone());
            }
            "cm" => {
                if let Some(v) = floats(args).filter(|v| v.len() == 6) {
                    gs.ctm = mul(&[v[0], v[1], v[2], v[3], v[4], v[5]], &gs.ctm);
                }
                out.push(op.clone());
            }
            "BT" => {
                tm = IDENT;
                tlm = IDENT;
                out.push(op.clone());
            }
            "Tf" => {
                if let (Some(Object::Name(n)), Some(s)) = (args.first(), args.get(1).map(num)) {
                    gs.font = Some(n.clone());
                    gs.fs = s;
                }
                out.push(op.clone());
            }
            "Tc" | "Tw" | "TL" | "Ts" | "Tz" => {
                let v = args.first().map(num).unwrap_or(0.0);
                match op.operator.as_str() {
                    "Tc" => gs.tc = v,
                    "Tw" => gs.tw = v,
                    "TL" => gs.tl = v,
                    "Ts" => gs.rise = v,
                    _ => gs.th = v / 100.0,
                }
                out.push(op.clone());
            }
            "Td" | "TD" => {
                if let Some(v) = floats(args).filter(|v| v.len() == 2) {
                    if op.operator == "TD" {
                        gs.tl = -v[1];
                    }
                    tlm = mul(&[1.0, 0.0, 0.0, 1.0, v[0], v[1]], &tlm);
                    tm = tlm;
                }
                out.push(op.clone());
            }
            "Tm" => {
                if let Some(v) = floats(args).filter(|v| v.len() == 6) {
                    tlm = [v[0], v[1], v[2], v[3], v[4], v[5]];
                    tm = tlm;
                }
                out.push(op.clone());
            }
            "T*" => {
                tlm = mul(&[1.0, 0.0, 0.0, 1.0, 0.0, -gs.tl], &tlm);
                tm = tlm;
                out.push(op.clone());
            }
            "Tj" | "TJ" | "'" | "\"" => {
                // Os operadores «'» e «"» mudan de liña (e «"» axusta os espazos) antes de amosar.
                if op.operator == "'" || op.operator == "\"" {
                    if op.operator == "\"" {
                        if let (Some(aw), Some(ac)) = (args.first(), args.get(1)) {
                            gs.tw = num(aw);
                            gs.tc = num(ac);
                        }
                    }
                    tlm = mul(&[1.0, 0.0, 0.0, 1.0, 0.0, -gs.tl], &tlm);
                    tm = tlm;
                }
                let items: Vec<&Object> = match (op.operator.as_str(), args.last()) {
                    ("TJ", Some(Object::Array(a))) => a.iter().collect(),
                    (_, Some(s @ Object::String(..))) => vec![s],
                    _ => return Err("texto ilexible".into()),
                };
                let name = gs.font.clone().ok_or("texto sen tipografía")?;
                let font = match fonts.get(&name) {
                    Some(Ok(f)) => f,
                    Some(Err(e)) => return Err(e.clone()),
                    None => return Err("tipografía descoñecida".into()),
                };
                let mut pieces: Vec<Object> = Vec::new();
                let mut keep: Vec<u8> = Vec::new();
                let mut changed = false;
                let step = if font.two_byte { 2 } else { 1 };
                let sx = gs.fs * gs.th;
                for item in items {
                    match item {
                        Object::String(bytes, _) => {
                            if bytes.len() % step != 0 {
                                return Err("texto con códigos raros".into());
                            }
                            for ch in bytes.chunks(step) {
                                let code = if font.two_byte { u16::from_be_bytes([ch[0], ch[1]]) as u32 } else { ch[0] as u32 };
                                let w = font.width(code);
                                let trm = mul(&[sx, 0.0, 0.0, gs.fs, 0.0, gs.rise], &mul(&tm, &gs.ctm));
                                let b = aabb(&[apply(&trm, 0.0, font.desc), apply(&trm, w, font.desc), apply(&trm, w, font.asc), apply(&trm, 0.0, font.asc)]);
                                let tx = (w * gs.fs + gs.tc + if !font.two_byte && code == 32 { gs.tw } else { 0.0 }) * gs.th;
                                if hits(b, rects) {
                                    if sx.abs() < 1e-6 {
                                        return Err("texto sen tamaño".into());
                                    }
                                    if !keep.is_empty() {
                                        pieces.push(hex(&keep));
                                        keep.clear();
                                    }
                                    // O oco que deixa a letra quitada mantense, para que o resto non se mova.
                                    pieces.push(Object::Real(-tx / sx * 1000.0));
                                    changed = true;
                                } else {
                                    keep.extend_from_slice(ch);
                                }
                                tm = mul(&[1.0, 0.0, 0.0, 1.0, tx, 0.0], &tm);
                            }
                        }
                        n @ (Object::Integer(_) | Object::Real(_)) => {
                            if !keep.is_empty() {
                                pieces.push(hex(&keep));
                                keep.clear();
                            }
                            let v = num(n);
                            pieces.push(n.clone());
                            tm = mul(&[1.0, 0.0, 0.0, 1.0, -v / 1000.0 * sx, 0.0], &tm);
                        }
                        _ => return Err("texto ilexible".into()),
                    }
                }
                if !keep.is_empty() {
                    pieces.push(hex(&keep));
                }
                if !changed {
                    out.push(op.clone());
                } else {
                    if op.operator == "\"" {
                        out.push(Operation::new("Tw", vec![args[0].clone()]));
                        out.push(Operation::new("Tc", vec![args[1].clone()]));
                    }
                    if op.operator == "'" || op.operator == "\"" {
                        out.push(Operation::new("T*", vec![]));
                    }
                    out.push(Operation::new("TJ", vec![Object::Array(pieces)]));
                }
            }
            "Do" => {
                let Some(Object::Name(name)) = args.first() else {
                    out.push(op.clone());
                    continue;
                };
                let target = xobj.get(name).ok().and_then(|o| o.as_reference().ok());
                let Some(tid) = target else {
                    out.push(op.clone());
                    continue;
                };
                let info = doc.get_object(tid).and_then(Object::as_stream).ok().map(|s| {
                    let sub = match s.dict.get(b"Subtype").ok() {
                        Some(Object::Name(n)) => n.clone(),
                        _ => Vec::new(),
                    };
                    let arr = |k: &[u8]| s.dict.get(k).ok().map(|o| deref(doc, o)).and_then(|o| o.as_array().ok()).map(|a| a.iter().map(|x| num(deref(doc, x))).collect::<Vec<f32>>());
                    (sub, arr(b"BBox"), arr(b"Matrix"))
                });
                match info {
                    Some((sub, _, _)) if sub == b"Image" => {
                        let unit = aabb(&[apply(&gs.ctm, 0.0, 0.0), apply(&gs.ctm, 1.0, 0.0), apply(&gs.ctm, 0.0, 1.0), apply(&gs.ctm, 1.0, 1.0)]);
                        if hits(unit, rects) {
                            let new_id = paint_image(doc, tid, &gs.ctm, rects)?;
                            made += 1;
                            let new_name = format!("TrasnoR{made}").into_bytes();
                            xobj.set(new_name.clone(), Object::Reference(new_id));
                            out.push(Operation::new("Do", vec![Object::Name(new_name)]));
                        } else {
                            out.push(op.clone());
                        }
                    }
                    Some((sub, bbox, matrix)) if sub == b"Form" => {
                        let m: M = matrix.filter(|m| m.len() == 6).map(|m| [m[0], m[1], m[2], m[3], m[4], m[5]]).unwrap_or(IDENT);
                        let full = mul(&m, &gs.ctm);
                        let b = match bbox.filter(|b| b.len() == 4) {
                            Some(b) => aabb(&[apply(&full, b[0], b[1]), apply(&full, b[2], b[1]), apply(&full, b[0], b[3]), apply(&full, b[2], b[3])]),
                            None => return Err("formulario sen caixa".into()),
                        };
                        if hits(b, rects) {
                            return Err("contido dentro dun formulario".into());
                        }
                        out.push(op.clone());
                    }
                    _ => out.push(op.clone()),
                }
            }
            "BI" | "ID" | "EI" => {
                let unit = aabb(&[apply(&gs.ctm, 0.0, 0.0), apply(&gs.ctm, 1.0, 0.0), apply(&gs.ctm, 0.0, 1.0), apply(&gs.ctm, 1.0, 1.0)]);
                if hits(unit, rects) {
                    return Err("imaxe dentro do contido".into());
                }
                out.push(op.clone());
            }
            "BDC" | "DP" => {
                // Os textos alternativos que acompañan ao contido poden levar o que se tapa.
                let mut o = op.clone();
                if let Some(Object::Dictionary(d)) = o.operands.get_mut(1) {
                    for k in [&b"ActualText"[..], b"Alt", b"E"] {
                        d.remove(k);
                    }
                }
                out.push(o);
            }
            _ => out.push(op.clone()),
        }
    }
    // Pecha o que o contido deixase aberto e debuxa o negro sen que lle afecte nada do que había.
    for _ in 0..stack.len() {
        out.push(Operation::new("Q", vec![]));
    }
    out.push(Operation::new("Q", vec![]));
    out.push(Operation::new("q", vec![]));
    out.push(Operation::new("g", vec![0.into()]));
    for z in rects {
        out.push(Operation::new("re", vec![z[0].into(), z[1].into(), (z[2] - z[0]).into(), (z[3] - z[1]).into()]));
    }
    out.push(Operation::new("f", vec![]));
    out.push(Operation::new("Q", vec![]));
    let bytes = Content { operations: out }.encode().map_err(|_| "non puiden escribir o contido")?;

    // Anotacións que tocan as zonas: fóra (ligazóns, notas, campos…).
    let mut removed = Vec::new();
    let mut kept = Vec::new();
    for a in annots_of(doc, page) {
        let touches = match &a {
            Object::Reference(id) => doc.get_dictionary(*id).ok().and_then(|d| rect_of(doc, d)).map(|r| hits(r, rects)).unwrap_or(false),
            _ => false,
        };
        match (&a, touches) {
            (Object::Reference(id), true) => removed.push(*id),
            _ => kept.push(a),
        }
    }

    // Todo saíu ben: agora si se cambia a páxina.
    res.set("XObject", Object::Dictionary(xobj));
    let new_content = doc.add_object(Stream::new(Dictionary::new(), bytes));
    let d = doc.get_dictionary_mut(page).map_err(|_| "páxina ilexible")?;
    d.set("Contents", Object::Reference(new_content));
    d.set("Resources", Object::Dictionary(res));
    for k in [&b"Thumb"[..], b"PieceInfo"] {
        d.remove(k);
    }
    if !removed.is_empty() {
        set_annots(doc, page, kept);
    }
    Ok(removed)
}
