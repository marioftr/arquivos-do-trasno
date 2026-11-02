//! Información lexible dun ficheiro (imaxe, PDF, modelo 3D...) para a pestana Información.
//! Cada ficheiro dá un «resumo» (o que lle interesa a calquera) e unha vista «avanzada» (o resto, xa traducido a palabras).
//! As datas van como `@date:<segundos>` para que a interface as escriba co formato do idioma.

use crate::error::{EngineError, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize, Default, Clone)]
pub struct Group {
    pub title: String,
    pub rows: Vec<[String; 2]>,
}

#[derive(Debug, Serialize, Default)]
pub struct Info {
    pub summary: Vec<Group>,
    pub advanced: Vec<Group>,
}

fn row(l: &str, v: impl Into<String>) -> [String; 2] {
    [l.to_string(), v.into()]
}
fn group(title: &str, rows: Vec<[String; 2]>) -> Group {
    Group { title: title.to_string(), rows }
}

/// Tamaño con unidades e, entre parénteses, os bytes exactos.
pub fn human_size(bytes: u64) -> String {
    const U: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {} ({} bytes)", U[i], group_digits(bytes))
    }
}
fn group_digits(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('\u{a0}');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}
/// «4:3», «16:9»... se a proporción é unha das habituais; senón, a fracción simplificada.
fn ratio_text(w: u64, h: u64) -> String {
    if w == 0 || h == 0 {
        return String::new();
    }
    let r = w as f64 / h as f64;
    for (a, b) in [(1u64, 1u64), (4, 3), (3, 2), (16, 9), (21, 9), (5, 4), (16, 10), (3, 4), (2, 3), (9, 16), (4, 5)] {
        if ((a as f64 / b as f64) - r).abs() < 0.015 {
            return format!("{a}:{b}");
        }
    }
    let g = gcd(w, h);
    if w / g > 50 { format!("{r:.2}:1") } else { format!("{}:{}", w / g, h / g) }
}
fn orientation_text(w: u64, h: u64) -> &'static str {
    if w == h { "Cadrada" } else if w > h { "Horizontal" } else { "Vertical" }
}

fn file_group(path: &Path) -> Group {
    let mut rows = Vec::new();
    if let Ok(m) = std::fs::metadata(path) {
        rows.push(row("Tamaño", human_size(m.len())));
        for (label, t) in [("Creado", m.created()), ("Modificado", m.modified())] {
            if let Ok(t) = t {
                if let Ok(d) = t.duration_since(std::time::UNIX_EPOCH) {
                    rows.push(row(label, format!("@date:{}", d.as_secs())));
                }
            }
        }
    }
    if let Some(e) = path.extension().and_then(|e| e.to_str()) {
        rows.insert(0, row("Tipo", e.to_uppercase()));
    }
    group("Ficheiro", rows)
}

// ------------------------------------------------------------------ imaxes

fn color_text(c: image::ColorType) -> (&'static str, &'static str) {
    use image::ColorType::*;
    match c {
        L8 => ("Escala de grises", "8 bits"),
        La8 => ("Escala de grises con transparencia", "8 bits"),
        Rgb8 => ("Cor (RGB)", "8 bits por canle"),
        Rgba8 => ("Cor (RGB) con transparencia", "8 bits por canle"),
        L16 => ("Escala de grises", "16 bits"),
        La16 => ("Escala de grises con transparencia", "16 bits"),
        Rgb16 => ("Cor (RGB)", "16 bits por canle"),
        Rgba16 => ("Cor (RGB) con transparencia", "16 bits por canle"),
        Rgb32F => ("Cor (RGB)", "32 bits en coma flotante"),
        Rgba32F => ("Cor (RGB) con transparencia", "32 bits en coma flotante"),
        _ => ("Outro", ""),
    }
}

pub fn describe_image(path: &Path) -> Result<Info> {
    use image::ImageDecoder;
    let reader = image::ImageReader::open(path)?.with_guessed_format()?;
    let format = reader.format();
    let mut dec = reader.into_decoder()?;
    let (w, h) = dec.dimensions();
    let (mode, depth) = color_text(dec.color_type());
    let has_alpha = dec.color_type().has_alpha();
    let exif_raw = dec.exif_metadata().ok().flatten();
    let mut summary = Vec::new();
    let mut size_rows = vec![
        row("Dimensións", format!("{w} × {h} px")),
        row("Megapíxeles", format!("{:.1} MP", (w as f64 * h as f64) / 1_000_000.0)),
        row("Forma", format!("{} · {}", orientation_text(w as u64, h as u64), ratio_text(w as u64, h as u64))),
    ];
    size_rows.push(row("Cor", if depth.is_empty() { mode.to_string() } else { format!("{mode} · {depth}") }));
    size_rows.push(row("Transparencia", if has_alpha { "Si" } else { "Non" }));
    if let Some(f) = format {
        size_rows.insert(0, row("Formato", format!("{f:?}").to_uppercase()));
    }
    summary.push(group("Imaxe", size_rows));

    let mut advanced = Vec::new();
    // EXIF (cámara, data da toma, exposición...) se o ficheiro o ten.
    let exif = exif_raw
        .as_deref()
        .and_then(|raw| exif::Reader::new().read_raw(raw.to_vec()).ok())
        .or_else(|| std::fs::File::open(path).ok().and_then(|f| exif::Reader::new().read_from_container(&mut std::io::BufReader::new(f)).ok()));
    if let Some(ex) = exif {
        let get = |tag: exif::Tag| ex.get_field(tag, exif::In::PRIMARY).map(|f| f.display_value().with_unit(&ex).to_string().trim_matches('"').to_string());
        let mut cam = Vec::new();
        let make = get(exif::Tag::Make);
        let model = get(exif::Tag::Model);
        match (make, model) {
            (Some(a), Some(b)) => cam.push(row("Cámara", if b.starts_with(&a) { b } else { format!("{a} {b}") })),
            (None, Some(b)) | (Some(b), None) => cam.push(row("Cámara", b)),
            _ => {}
        }
        for (label, tag) in [
            ("Data da toma", exif::Tag::DateTimeOriginal),
            ("Exposición", exif::Tag::ExposureTime),
            ("Apertura", exif::Tag::FNumber),
            ("ISO", exif::Tag::PhotographicSensitivity),
            ("Distancia focal", exif::Tag::FocalLength),
            ("Obxectivo", exif::Tag::LensModel),
            ("Programa", exif::Tag::Software),
        ] {
            if let Some(v) = get(tag) {
                cam.push(row(label, v));
            }
        }
        if ex.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY).is_some() {
            cam.push(row("Localización GPS", "Si (a foto garda onde se tomou)"));
        }
        if !cam.is_empty() {
            summary.push(group("Cámara", cam));
        }
        let all: Vec<[String; 2]> = ex.fields().map(|f| [format!("{}", f.tag), f.display_value().with_unit(&ex).to_string().trim_matches('"').to_string()]).collect();
        if !all.is_empty() {
            advanced.push(group("Datos EXIF", all));
        }
    }
    summary.push(file_group(path));
    advanced.insert(0, group("Imaxe", vec![row("Ancho", format!("{w} px")), row("Alto", format!("{h} px")), row("Píxeles en total", group_digits(w as u64 * h as u64))]));
    Ok(Info { summary, advanced })
}

// --------------------------------------------------------------------- PDF

fn paper_name(w_mm: f64, h_mm: f64) -> Option<&'static str> {
    let (a, b) = if w_mm < h_mm { (w_mm, h_mm) } else { (h_mm, w_mm) };
    for (n, pw, ph) in [("A3", 297.0, 420.0), ("A4", 210.0, 297.0), ("A5", 148.0, 210.0), ("Carta (Letter)", 215.9, 279.4), ("Legal", 215.9, 355.6)] {
        if (a - pw).abs() < 3.0 && (b - ph).abs() < 3.0 {
            return Some(n);
        }
    }
    None
}

fn pdf_text(o: &lopdf::Object) -> Option<String> {
    match o {
        lopdf::Object::String(b, _) => {
            if b.starts_with(&[0xFE, 0xFF]) {
                let u: Vec<u16> = b[2..].chunks(2).filter(|c| c.len() == 2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
                Some(String::from_utf16_lossy(&u))
            } else {
                Some(b.iter().map(|c| *c as char).collect())
            }
        }
        lopdf::Object::Name(n) => Some(String::from_utf8_lossy(n).to_string()),
        _ => None,
    }
}

/// `D:20240131120000+01'00'` → segundos desde 1970 (aprox., sen zona horaria).
fn pdf_date(s: &str) -> Option<String> {
    let d = s.trim_start_matches("D:");
    let n = |a: usize, b: usize| d.get(a..b).and_then(|x| x.parse::<i64>().ok());
    let (y, m, day) = (n(0, 4)?, n(4, 6).unwrap_or(1), n(6, 8).unwrap_or(1));
    let (hh, mm, ss) = (n(8, 10).unwrap_or(0), n(10, 12).unwrap_or(0), n(12, 14).unwrap_or(0));
    // días desde 1970 (algoritmo de Howard Hinnant)
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(format!("@date:{}", days * 86400 + hh * 3600 + mm * 60 + ss))
}

pub fn describe_pdf(path: &Path) -> Result<Info> {
    let mut summary = Vec::new();
    let mut advanced = Vec::new();
    match lopdf::Document::load(path) {
        Ok(doc) => {
            let pages = doc.get_pages();
            let mut page_rows = vec![row("Páxinas", pages.len().to_string()), row("Versión do PDF", doc.version.clone())];
            // tamaño da primeira páxina
            if let Some((_, id)) = pages.iter().next() {
                if let Ok(d) = doc.get_dictionary(*id) {
                    if let Ok(mb) = d.get(b"MediaBox").and_then(|o| o.as_array()) {
                        let v: Vec<f64> = mb.iter().filter_map(|o| o.as_float().map(|f| f as f64).ok().or_else(|| o.as_i64().ok().map(|i| i as f64))).collect();
                        if v.len() == 4 {
                            let (w, h) = ((v[2] - v[0]).abs() * 25.4 / 72.0, (v[3] - v[1]).abs() * 25.4 / 72.0);
                            let name = paper_name(w, h).map(|n| format!(" · {n}")).unwrap_or_default();
                            page_rows.push(row("Tamaño da páxina", format!("{w:.0} × {h:.0} mm{name}")));
                        }
                    }
                }
            }
            page_rows.push(row("Protexido con contrasinal", if doc.is_encrypted() { "Si" } else { "Non" }));
            summary.push(group("Documento", page_rows));

            let mut meta = Vec::new();
            let mut raw = Vec::new();
            if let Ok(info) = doc.trailer.get(b"Info").and_then(|o| doc.dereference(o)).and_then(|(_, o)| o.as_dict().cloned()) {
                for (k, label) in [("Title", "Título"), ("Author", "Autoría"), ("Subject", "Asunto"), ("Keywords", "Palabras clave"), ("Creator", "Creado con"), ("Producer", "Convertido con"), ("CreationDate", "Data de creación"), ("ModDate", "Última modificación")] {
                    if let Some(v) = info.get(k.as_bytes()).ok().and_then(|o| doc.dereference(o).ok().map(|x| x.1)).and_then(pdf_text) {
                        let v = if k.ends_with("Date") { pdf_date(&v).unwrap_or(v) } else { v };
                        if !v.trim().is_empty() {
                            meta.push(row(label, v));
                        }
                    }
                }
                for (k, v) in info.iter() {
                    if let Some(t) = pdf_text(v) {
                        raw.push([String::from_utf8_lossy(k).to_string(), t]);
                    }
                }
            }
            if !meta.is_empty() {
                summary.push(group("Datos do documento", meta));
            }
            let images = doc.objects.values().filter(|o| matches!(o, lopdf::Object::Stream(s) if s.dict.get(b"Subtype").ok().and_then(|x| x.as_name().ok()) == Some(b"Image"))).count();
            let has = |k: &[u8]| doc.catalog().ok().is_some_and(|c| c.has(k));
            advanced.push(group(
                "Estrutura",
                vec![
                    row("Imaxes dentro do PDF", images.to_string()),
                    row("Formularios", if has(b"AcroForm") { "Si" } else { "Non" }),
                    row("Marcadores (índice)", if has(b"Outlines") { "Si" } else { "Non" }),
                    row("Obxectos internos", doc.objects.len().to_string()),
                ],
            ));
            if !raw.is_empty() {
                advanced.push(group("Metadatos completos", raw));
            }
        }
        Err(e) => {
            // Moitas veces: PDF con contrasinal.
            summary.push(group("Documento", vec![row("Estado", format!("Non se puido ler por dentro ({e}); pode estar protexido"))]));
        }
    }
    summary.push(file_group(path));
    Ok(Info { summary, advanced })
}

// ---------------------------------------------------------------------- 3D

pub fn describe_model(path: &Path) -> Result<Info> {
    let mesh = crate::model3d::read_mesh_public(path)?;
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for v in &mesh.vertices {
        for i in 0..3 {
            lo[i] = lo[i].min(v[i]);
            hi[i] = hi[i].max(v[i]);
        }
    }
    if mesh.vertices.is_empty() {
        lo = [0.0; 3];
        hi = [0.0; 3];
    }
    let d = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    let fmt = |x: f32| format!("{x:.3}");
    let summary = vec![
        group(
            "Modelo",
            vec![
                row("Vértices", group_digits(mesh.vertices.len() as u64)),
                row("Triángulos", group_digits(mesh.faces.len() as u64)),
                row("Tamaño (ancho × alto × fondo)", format!("{} × {} × {}", fmt(d[0]), fmt(d[1]), fmt(d[2]))),
            ],
        ),
        file_group(path),
    ];
    let advanced = vec![group(
        "Caixa que o contén",
        vec![
            row("Mínimo (x, y, z)", format!("{}, {}, {}", fmt(lo[0]), fmt(lo[1]), fmt(lo[2]))),
            row("Máximo (x, y, z)", format!("{}, {}, {}", fmt(hi[0]), fmt(hi[1]), fmt(hi[2]))),
            row("Centro", format!("{}, {}, {}", fmt((lo[0] + hi[0]) / 2.0), fmt((lo[1] + hi[1]) / 2.0), fmt((lo[2] + hi[2]) / 2.0))),
        ],
    )];
    Ok(Info { summary, advanced })
}

/// Información segundo o tipo de ficheiro; para audio e vídeo ocúpase a interface (ffprobe).
pub fn describe(path: &Path) -> Result<Info> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => describe_pdf(path),
        "obj" | "stl" | "ply" => describe_model(path),
        _ => match describe_image(path) {
            Ok(i) => Ok(i),
            Err(_) => Ok(Info { summary: vec![file_group(path)], advanced: vec![] }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(i: &Info, label: &str, value: &str) -> bool {
        i.summary.iter().flat_map(|g| &g.rows).any(|r| r[0] == label && r[1].contains(value))
    }

    #[test]
    fn describe_dun_png_e_dun_pdf() {
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("a.png");
        image::RgbaImage::from_pixel(160, 90, image::Rgba([10, 20, 30, 255])).save(&png).unwrap();
        let i = describe(&png).unwrap();
        assert!(has(&i, "Dimensións", "160 × 90"));
        assert!(has(&i, "Forma", "16:9"));

        let txt = dir.path().join("a.txt");
        std::fs::write(&txt, "ola").unwrap();
        let pdf = dir.path().join("a.pdf");
        crate::pdf::text_to_pdf(&txt, &pdf, false).unwrap();
        let p = describe(&pdf).unwrap();
        assert!(has(&p, "Páxinas", "1"));
        assert!(has(&p, "Tamaño da páxina", "A4"));
    }

    #[test]
    fn proporcions_habituais() {
        assert_eq!(ratio_text(1920, 1080), "16:9");
        assert_eq!(ratio_text(720, 576), "5:4");
        assert_eq!(ratio_text(1000, 1000), "1:1");
    }
}
