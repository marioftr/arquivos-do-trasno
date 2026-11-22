//! Imaxes do portapapeis: o sistema dá os píxeles en bruto (RGBA) e aquí gárdanse como PNG nun ficheiro novo.

use crate::error::{EngineError, Result};
use std::path::{Path, PathBuf};

/// Un nome libre en `dir`: «portapapeis.png», «portapapeis (2).png»…
pub fn free_name(dir: &Path, stem: &str) -> PathBuf {
    let mut n = 1;
    loop {
        let name = if n == 1 { format!("{stem}.png") } else { format!("{stem} ({n}).png") };
        let p = dir.join(name);
        if !p.exists() {
            return p;
        }
        n += 1;
    }
}

/// Garda píxeles RGBA (8 bits, sen filas de enchido) como PNG en `dir` e devolve a ruta.
pub fn save_rgba_png(width: usize, height: usize, rgba: &[u8], dir: &Path) -> Result<PathBuf> {
    let img = image::RgbaImage::from_raw(width as u32, height as u32, rgba.to_vec())
        .ok_or_else(|| EngineError::IntegrityCheck("A imaxe do portapapeis non ten o tamaño que di.".into()))?;
    std::fs::create_dir_all(dir)?;
    let path = free_name(dir, "portapapeis");
    img.save(&path).map_err(|e| EngineError::IntegrityCheck(format!("Non puiden gardar a imaxe do portapapeis: {e}")))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn garda_o_png_e_non_pisa_o_anterior() {
        let dir = tempfile::tempdir().unwrap();
        let px: Vec<u8> = (0..2 * 2).flat_map(|i| [i as u8 * 40, 10, 200, 255]).collect();
        let a = save_rgba_png(2, 2, &px, dir.path()).unwrap();
        let b = save_rgba_png(2, 2, &px, dir.path()).unwrap();
        assert_eq!(a.file_name().unwrap(), "portapapeis.png");
        assert_eq!(b.file_name().unwrap(), "portapapeis (2).png");
        let back = image::open(&a).unwrap().to_rgba8();
        assert_eq!((back.width(), back.height()), (2, 2));
        assert_eq!(back.as_raw(), &px);
    }

    #[test]
    fn rexeita_un_tamano_que_non_cadra() {
        let dir = tempfile::tempdir().unwrap();
        assert!(save_rgba_png(3, 3, &[0; 8], dir.path()).is_err());
    }
}
