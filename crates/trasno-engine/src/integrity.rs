//! Utilidades para garantir que unha conversión nunca deixa un ficheiro a
//! medio escribir nin sobrescribe algo por enriba do orixinal sen querer.
//!
//! Regra xeral do motor: escríbese sempre primeiro nun ficheiro temporal no
//! mesmo directorio de destino, valídase o resultado, e só despois se fai un
//! `rename` (atómico no mesmo sistema de ficheiros) ao nome final. Se algo
//! falla polo medio, o ficheiro temporal desaparece e o destino nunca chega
//! a existir a medias.

use crate::error::{EngineError, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Un ficheiro de saída "en construción". Escribe nun temporal e só se
/// materializa no destino final chamando a `commit`.
pub struct AtomicOutput {
    temp: tempfile::NamedTempFile,
    destination: PathBuf,
}

impl AtomicOutput {
    /// Crea un novo ficheiro temporal no mesmo directorio que `destination`,
    /// para que o `rename` final sexa atómico (mesmo punto de montaxe).
    pub fn new(destination: impl Into<PathBuf>, overwrite: bool) -> Result<Self> {
        let destination = destination.into();
        if destination.exists() && !overwrite {
            return Err(EngineError::DestinationExists(destination));
        }
        let dir = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(dir)?;
        let temp = tempfile::Builder::new()
            .prefix(".trasno-")
            .suffix(".part")
            .tempfile_in(dir)?;
        Ok(Self { temp, destination })
    }

    pub fn path(&self) -> &Path {
        self.temp.path()
    }

    pub fn as_file_mut(&mut self) -> &mut File {
        self.temp.as_file_mut()
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.temp.as_file_mut().write_all(bytes)?;
        Ok(())
    }

    /// Valida o contido escrito (tamaño > 0 como mínimo; o chamador pode
    /// engadir validación específica do formato antes de chamar a isto) e
    /// move o temporal ao destino final.
    pub fn commit(self) -> Result<PathBuf> {
        let metadata = self.temp.as_file().metadata()?;
        if metadata.len() == 0 {
            return Err(EngineError::EmptyOrCorruptOutput);
        }
        self.temp.as_file().sync_all()?;
        self.temp
            .persist(&self.destination)
            .map_err(|e| EngineError::Io(e.error))?;
        Ok(self.destination)
    }
}

/// Rexeita que o destino sexa o propio ficheiro de orixe (por ligazón
/// simbólica, ruta relativa ou maiúsculas incluídas): con "sobrescribir"
/// activado iso destruiría o orixinal.
pub fn ensure_distinct(inputs: &[&Path], output: &Path) -> Result<()> {
    let resolve = |p: &Path| -> PathBuf {
        std::fs::canonicalize(p).unwrap_or_else(|_| {
            // O destino pode non existir aínda: resolvemos o seu directorio.
            let name = p.file_name().map(|n| n.to_owned()).unwrap_or_default();
            let dir = p
                .parent()
                .filter(|d| !d.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            std::fs::canonicalize(dir)
                .map(|d| d.join(&name))
                .unwrap_or_else(|_| p.to_path_buf())
        })
    };
    let out = resolve(output);
    if inputs.iter().any(|i| resolve(i) == out) {
        return Err(EngineError::IntegrityCheck(
            "o ficheiro de destino é o mesmo que o de orixe; escolle outro nome".into(),
        ));
    }
    Ok(())
}

/// Calcula o SHA-256 dun ficheiro, útil para comprobar que unha copia ou
/// conversión "sen perdas" (p.ex. remuxado) non alterou os bytes esperados.
pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Comproba que un ficheiro existe e non está baleiro; é a validación mínima
/// que se aplica sempre antes de dar por boa calquera conversión.
pub fn assert_non_empty_file(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path).map_err(EngineError::Io)?;
    if metadata.len() == 0 {
        return Err(EngineError::EmptyOrCorruptOutput);
    }
    Ok(metadata.len())
}

#[cfg(test)]
mod distinct_tests {
    use super::*;

    #[test]
    fn detecta_mesmo_ficheiro_por_ruta_distinta() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        std::fs::write(&a, b"x").unwrap();
        let alias = dir.path().join(".").join("a.png");
        assert!(ensure_distinct(&[&a], &alias).is_err());
        assert!(ensure_distinct(&[&a], &dir.path().join("b.png")).is_ok());
    }
}
