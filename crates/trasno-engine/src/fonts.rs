//! Letras do sistema (TrueType) para marcas, números e textos engadidos a un PDF.
//! Lístanse as que hai instaladas e, ao usar unha, incrústase no PDF para que se vexa igual en calquera equipo.

use crate::error::{EngineError, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use ttf_parser::{name_id, Face};

#[derive(Debug, Clone, Serialize)]
pub struct SysFont {
    /// Nome completo («Arial Bold»).
    pub name: String,
    /// Familia («Arial»), para a vista previa.
    pub family: String,
    pub weight: u16,
    pub italic: bool,
    pub path: String,
}

fn name_of(face: &Face, id: u16) -> Option<String> {
    face.names().into_iter().filter(|n| n.name_id == id).find_map(|n| n.to_string()).filter(|s| !s.trim().is_empty())
}

fn font_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(w) = std::env::var_os("WINDIR").or_else(|| std::env::var_os("SystemRoot")) {
        v.push(PathBuf::from(w).join("Fonts"));
    }
    if let Some(l) = std::env::var_os("LOCALAPPDATA") {
        v.push(PathBuf::from(l).join("Microsoft").join("Windows").join("Fonts"));
    }
    v
}

/// Pode incrustarse? Só TrueType con contornos e cuxa licenza non o prohiba.
fn usable(face: &Face) -> bool {
    face.tables().glyf.is_some() && !matches!(face.permissions(), Some(ttf_parser::Permissions::Restricted))
}

/// Letras `.ttf` instaladas, ordenadas por nome. Só se len unha vez por execución.
pub fn list() -> &'static Vec<SysFont> {
    static CACHE: OnceLock<Vec<SysFont>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut out: Vec<SysFont> = Vec::new();
        for dir in font_dirs() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if !p.extension().and_then(|x| x.to_str()).is_some_and(|x| x.eq_ignore_ascii_case("ttf")) {
                    continue;
                }
                let Ok(data) = std::fs::read(&p) else { continue };
                let Ok(face) = Face::parse(&data, 0) else { continue };
                if !usable(&face) {
                    continue;
                }
                let family = name_of(&face, name_id::TYPOGRAPHIC_FAMILY).or_else(|| name_of(&face, name_id::FAMILY));
                let full = name_of(&face, name_id::FULL_NAME);
                let (Some(family), Some(name)) = (family.clone(), full.or(family)) else { continue };
                // Os símbolos (Wingdings, Webdings...) non valen para escribir texto.
                if face.glyph_index('A').is_none() {
                    continue;
                }
                out.push(SysFont { name, family, weight: face.weight().to_number(), italic: face.is_italic(), path: p.to_string_lossy().to_string() });
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        out.dedup_by(|a, b| a.name == b.name);
        out
    })
}

/// Unha letra lista para incrustar: o ficheiro, o nome PostScript e as medidas en milésimas de em.
pub struct Embedded {
    pub ps_name: String,
    pub data: Vec<u8>,
    /// Ancho de cada código WinAnsi 32..=255, en milésimas.
    pub widths: Vec<f32>,
    pub bbox: [f32; 4],
    pub ascent: f32,
    pub descent: f32,
    pub cap_height: f32,
    pub italic_angle: f32,
    pub flags: i64,
}

const CP1252_HIGH: [u32; 32] = [
    0x20AC, 0, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039, 0x0152, 0, 0x017D, 0, 0, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, 0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0, 0x017E, 0x0178,
];

pub fn load(path: &Path) -> Result<Embedded> {
    let data = std::fs::read(path)?;
    let bad = |m: &str| EngineError::IntegrityCheck(m.to_string());
    let (widths, bbox, ascent, descent, cap, italic_angle, flags, ps) = {
        let face = Face::parse(&data, 0).map_err(|_| bad("Non puiden ler esa letra."))?;
        if !usable(&face) {
            return Err(bad("Esa letra non se pode incrustar nun PDF."));
        }
        let upem = face.units_per_em() as f32;
        let k = 1000.0 / upem;
        let widths: Vec<f32> = (32u32..=255)
            .map(|code| {
                let cp = if (0x80..=0x9F).contains(&code) { CP1252_HIGH[(code - 0x80) as usize] } else { code };
                char::from_u32(cp).and_then(|c| face.glyph_index(c)).and_then(|g| face.glyph_hor_advance(g)).map(|a| a as f32 * k).unwrap_or(0.0)
            })
            .collect();
        let b = face.global_bounding_box();
        let bbox = [b.x_min as f32 * k, b.y_min as f32 * k, b.x_max as f32 * k, b.y_max as f32 * k];
        let ps = name_of(&face, name_id::POST_SCRIPT_NAME)
            .or_else(|| name_of(&face, name_id::FULL_NAME))
            .unwrap_or_else(|| "FonteTrasno".to_string())
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect::<String>();
        let mut flags = 32; // non simbólica
        if face.is_italic() {
            flags |= 64;
        }
        if face.is_monospaced() {
            flags |= 1;
        }
        (widths, bbox, face.ascender() as f32 * k, face.descender() as f32 * k, face.capital_height().map(|c| c as f32 * k).unwrap_or(700.0), face.italic_angle(), flags, ps)
    };
    Ok(Embedded { ps_name: if ps.is_empty() { "FonteTrasno".into() } else { ps }, data, widths, bbox, ascent, descent, cap_height: cap, italic_angle, flags })
}

impl Embedded {
    /// Ancho do texto (en puntos) co tamaño dado; as liñas máis longas mandan.
    pub fn width(&self, text: &str, size: f32) -> f32 {
        text.lines()
            .map(|l| l.chars().map(|c| {
                let code = if (c as u32) < 256 { c as u32 } else { b'?' as u32 };
                self.widths.get(code.saturating_sub(32) as usize).copied().unwrap_or(0.0)
            }).sum::<f32>())
            .fold(0.0, f32::max)
            * size
            / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lista_e_incrusta_unha_letra_do_sistema() {
        let list = list();
        if list.is_empty() {
            return; // contornos sen letras (CI)
        }
        let f = list.iter().find(|f| f.family == "Arial").unwrap_or(&list[0]);
        let e = load(Path::new(&f.path)).unwrap();
        assert!(e.widths.len() == 224);
        assert!(e.width("Ola mundo", 12.0) > 20.0);
    }
}
