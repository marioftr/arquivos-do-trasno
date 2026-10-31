//! Conversión básica de modelos 3D de malla triangular: OBJ, STL (ASCII e
//! binario) e PLY (ASCII). Só se conservan vértices e caras (sen materiais,
//! texturas, normais suavizadas nin escenas) — o obxectivo desta fase é
//! poder pasar dun formato "de intercambio" simple a outro con garantías,
//! non substituír un motor de render.

use crate::error::{EngineError, Result};
use crate::formats::{self, Category, MODEL3D_ENGINE_SUPPORTED};
use crate::integrity::AtomicOutput;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
}

impl Mesh {
    fn push_polygon(&mut self, indices: &[u32]) {
        // Fan-triangulación: calquera polígono convexo (v0..vn) pártese en
        // triángulos (v0,v1,v2), (v0,v2,v3), etc. É o estándar para OBJ.
        if indices.len() < 3 {
            return;
        }
        for i in 1..indices.len() - 1 {
            self.faces.push([indices[0], indices[i], indices[i + 1]]);
        }
    }
}

fn ext_of(path: &Path) -> Result<String> {
    formats::extension_of(path).ok_or_else(|| EngineError::UnknownSourceFormat(path.to_path_buf()))
}

// ---------------------------------------------------------------- OBJ ----

fn read_obj(path: &Path) -> Result<Mesh> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut mesh = Mesh::default();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("v ") {
            let coords: Vec<f32> = rest
                .split_whitespace()
                .take(3)
                .filter_map(|s| s.parse().ok())
                .collect();
            if coords.len() == 3 {
                mesh.vertices.push([coords[0], coords[1], coords[2]]);
            }
        } else if let Some(rest) = line.strip_prefix("f ") {
            let n = mesh.vertices.len() as i64;
            let mut indices = Vec::new();
            for token in rest.split_whitespace() {
                // formato "v", "v/vt", "v/vt/vn" ou "v//vn": só nos interesa o v.
                let v_part = token.split('/').next().unwrap_or(token);
                if let Ok(mut idx) = v_part.parse::<i64>() {
                    if idx < 0 {
                        idx = n + idx + 1; // índices relativos ao final do ficheiro
                    }
                    if idx >= 1 {
                        indices.push((idx - 1) as u32);
                    }
                }
            }
            mesh.push_polygon(&indices);
        }
    }

    if mesh.vertices.is_empty() {
        return Err(EngineError::Model3d(
            "O OBJ non ten ningún vértice.".into(),
        ));
    }
    Ok(mesh)
}

fn write_obj(mesh: &Mesh, out: &mut impl Write) -> Result<()> {
    for v in &mesh.vertices {
        writeln!(out, "v {} {} {}", v[0], v[1], v[2])?;
    }
    for f in &mesh.faces {
        writeln!(out, "f {} {} {}", f[0] + 1, f[1] + 1, f[2] + 1)?;
    }
    Ok(())
}

// ---------------------------------------------------------------- STL ----

fn face_normal(mesh: &Mesh, face: [u32; 3]) -> [f32; 3] {
    let a = mesh.vertices[face[0] as usize];
    let b = mesh.vertices[face[1] as usize];
    let c = mesh.vertices[face[2] as usize];
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > f32::EPSILON {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn read_stl(path: &Path) -> Result<Mesh> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.read_to_end(&mut bytes)?;

    // Heurística estándar: un STL binario ten unha cabeceira de 80 bytes +
    // u32 co número de triángulos + 50 bytes por triángulo. Se o tamaño do
    // ficheiro encaixa exactamente con esa fórmula, é binario (aínda que
    // empece polo texto "solid", cousa que confunde a moitos parsers).
    if bytes.len() >= 84 {
        let count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
        if bytes.len() == 84 + count * 50 {
            return read_stl_binary(&bytes, count);
        }
    }
    read_stl_ascii(&bytes)
}

fn read_stl_binary(bytes: &[u8], count: usize) -> Result<Mesh> {
    let mut mesh = Mesh::default();
    let mut offset = 84;
    for _ in 0..count {
        // salta a normal (12 bytes) e le os 3 vértices (36 bytes); os
        // últimos 2 bytes son o "attribute byte count", que ignoramos.
        let mut face = [0u32; 3];
        for f in face.iter_mut() {
            let base = offset + 12;
            *f = mesh.vertices.len() as u32;
            let x = f32::from_le_bytes(bytes[base..base + 4].try_into().unwrap());
            let y = f32::from_le_bytes(bytes[base + 4..base + 8].try_into().unwrap());
            let z = f32::from_le_bytes(bytes[base + 8..base + 12].try_into().unwrap());
            mesh.vertices.push([x, y, z]);
            offset += 12;
        }
        mesh.faces.push(face);
        offset += 12 + 2; // normal xa lida antes do bucle + attribute byte count
    }
    Ok(mesh)
}

fn read_stl_ascii(bytes: &[u8]) -> Result<Mesh> {
    let text = String::from_utf8_lossy(bytes);
    let mut mesh = Mesh::default();
    let mut current: Vec<u32> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("vertex") {
            let coords: Vec<f32> = rest
                .split_whitespace()
                .take(3)
                .filter_map(|s| s.parse().ok())
                .collect();
            if coords.len() == 3 {
                let idx = mesh.vertices.len() as u32;
                mesh.vertices.push([coords[0], coords[1], coords[2]]);
                current.push(idx);
            }
        } else if line.starts_with("endfacet") {
            if current.len() >= 3 {
                mesh.faces.push([current[0], current[1], current[2]]);
            }
            current.clear();
        }
    }

    if mesh.faces.is_empty() {
        return Err(EngineError::Model3d(
            "Non atopei ningún triángulo no STL.".into(),
        ));
    }
    Ok(mesh)
}

fn write_stl_binary(mesh: &Mesh, out: &mut impl Write) -> Result<()> {
    let mut header = [0u8; 80];
    let banner = b"Trasno STL export";
    header[..banner.len()].copy_from_slice(banner);
    out.write_all(&header)?;
    out.write_all(&(mesh.faces.len() as u32).to_le_bytes())?;

    for &face in &mesh.faces {
        let normal = face_normal(mesh, face);
        for c in normal {
            out.write_all(&c.to_le_bytes())?;
        }
        for idx in face {
            let v = mesh.vertices[idx as usize];
            for c in v {
                out.write_all(&c.to_le_bytes())?;
            }
        }
        out.write_all(&0u16.to_le_bytes())?;
    }
    Ok(())
}

// ---------------------------------------------------------------- PLY ----

fn read_ply(path: &Path) -> Result<Mesh> {
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut header_lines = Vec::new();
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Err(EngineError::Model3d("A cabeceira do PLY está incompleta.".into()));
        }
        let trimmed = line.trim().to_string();
        let is_end = trimmed == "end_header";
        header_lines.push(trimmed);
        if is_end {
            break;
        }
    }

    if !header_lines.first().map(|l| l == "ply").unwrap_or(false) {
        return Err(EngineError::Model3d("Isto non é un PLY válido.".into()));
    }
    if header_lines
        .iter()
        .any(|l| l.starts_with("format") && !l.contains("ascii"))
    {
        return Err(EngineError::Model3d(
            "De momento só lin PLY en ASCII, non en binario.".into(),
        ));
    }

    let mut vertex_count = 0usize;
    let mut face_count = 0usize;
    let mut section = "";
    for l in &header_lines {
        if let Some(rest) = l.strip_prefix("element vertex") {
            vertex_count = rest.trim().parse().unwrap_or(0);
            section = "vertex";
        } else if let Some(rest) = l.strip_prefix("element face") {
            face_count = rest.trim().parse().unwrap_or(0);
            section = "face";
        } else if l.starts_with("element") {
            section = "other";
        }
        let _ = section;
    }

    let mut mesh = Mesh::default();
    let mut lines = reader.lines();

    for _ in 0..vertex_count {
        let l = lines
            .next()
            .ok_or_else(|| EngineError::Model3d("Faltan liñas de vértices no PLY.".into()))??;
        let coords: Vec<f32> = l
            .split_whitespace()
            .take(3)
            .filter_map(|s| s.parse().ok())
            .collect();
        if coords.len() != 3 {
            return Err(EngineError::Model3d("Hai unha liña de vértice mal formada no PLY.".into()));
        }
        mesh.vertices.push([coords[0], coords[1], coords[2]]);
    }

    for _ in 0..face_count {
        let l = lines
            .next()
            .ok_or_else(|| EngineError::Model3d("Faltan liñas de caras no PLY.".into()))??;
        let nums: Vec<u32> = l.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        if let Some((&n, indices)) = nums.split_first() {
            if indices.len() as u32 >= n {
                mesh.push_polygon(&indices[..n as usize]);
            }
        }
    }

    Ok(mesh)
}

fn write_ply(mesh: &Mesh, out: &mut impl Write) -> Result<()> {
    writeln!(out, "ply")?;
    writeln!(out, "format ascii 1.0")?;
    writeln!(out, "comment exportado por Trasno")?;
    writeln!(out, "element vertex {}", mesh.vertices.len())?;
    writeln!(out, "property float x")?;
    writeln!(out, "property float y")?;
    writeln!(out, "property float z")?;
    writeln!(out, "element face {}", mesh.faces.len())?;
    writeln!(out, "property list uchar int vertex_indices")?;
    writeln!(out, "end_header")?;
    for v in &mesh.vertices {
        writeln!(out, "{} {} {}", v[0], v[1], v[2])?;
    }
    for f in &mesh.faces {
        writeln!(out, "3 {} {} {}", f[0], f[1], f[2])?;
    }
    Ok(())
}

// ------------------------------------------------------------ público ----

fn read_mesh(path: &Path, ext: &str) -> Result<Mesh> {
    match ext {
        "obj" => read_obj(path),
        "stl" => read_stl(path),
        "ply" => read_ply(path),
        other => Err(EngineError::UnsupportedConversion {
            from: other.to_string(),
            to: "modelo3d".to_string(),
        }),
    }
}

fn write_mesh(mesh: &Mesh, ext: &str, out: &mut impl Write) -> Result<()> {
    match ext {
        "obj" => write_obj(mesh, out),
        "stl" => write_stl_binary(mesh, out),
        "ply" => write_ply(mesh, out),
        other => Err(EngineError::UnsupportedConversion {
            from: "modelo3d".to_string(),
            to: other.to_string(),
        }),
    }
}

pub fn convert_model3d(input: &Path, output: &Path, overwrite: bool) -> Result<PathBuf> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    let src_ext = ext_of(input)?;
    if formats::category_of_extension(&src_ext) != Category::Modelo3d {
        return Err(EngineError::UnsupportedConversion {
            from: src_ext,
            to: "modelo3d".to_string(),
        });
    }
    let dst_ext = ext_of(output)?;
    if !MODEL3D_ENGINE_SUPPORTED.contains(&src_ext.as_str())
        || !MODEL3D_ENGINE_SUPPORTED.contains(&dst_ext.as_str())
    {
        return Err(EngineError::Model3dUnsupported {
            formats: MODEL3D_ENGINE_SUPPORTED.join("/"),
            from: src_ext,
            to: dst_ext,
        });
    }

    let mesh = read_mesh(input, &src_ext)?;
    if mesh.vertices.is_empty() || mesh.faces.is_empty() {
        return Err(EngineError::EmptyOrCorruptOutput);
    }

    let mut out = AtomicOutput::new(output, overwrite)?;
    write_mesh(&mesh, &dst_ext, out.as_file_mut())?;
    let final_path = out.commit()?;

    // Verificación: o ficheiro escrito ten que volver ler coa mesma
    // cantidade de caras (o número de vértices pode medrar en STL, que
    // non comparte vértices entre triángulos — iso é normal e non é erro).
    match read_mesh(&final_path, &dst_ext) {
        Ok(check) if check.faces.len() == mesh.faces.len() && !check.vertices.is_empty() => {
            Ok(final_path)
        }
        _ => {
            let _ = std::fs::remove_file(&final_path);
            Err(EngineError::EmptyOrCorruptOutput)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cubo() -> Mesh {
        // Un cubo unitario simple: 8 vértices, 12 triángulos (2 por cara).
        let v = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let faces: Vec<[u32; 3]> = vec![
            [0, 1, 2], [0, 2, 3], // abaixo
            [4, 6, 5], [4, 7, 6], // arriba
            [0, 4, 5], [0, 5, 1], // lados...
            [1, 5, 6], [1, 6, 2],
            [2, 6, 7], [2, 7, 3],
            [3, 7, 4], [3, 4, 0],
        ];
        Mesh { vertices: v.to_vec(), faces }
    }

    #[test]
    fn obj_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cubo.obj");
        let mesh = cubo();
        let mut f = std::fs::File::create(&path).unwrap();
        write_obj(&mesh, &mut f).unwrap();
        drop(f);

        let loaded = read_obj(&path).unwrap();
        assert_eq!(loaded.vertices.len(), 8);
        assert_eq!(loaded.faces.len(), 12);
    }

    #[test]
    fn stl_binario_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cubo.stl");
        let mesh = cubo();
        let mut f = std::fs::File::create(&path).unwrap();
        write_stl_binary(&mesh, &mut f).unwrap();
        drop(f);

        let loaded = read_stl(&path).unwrap();
        assert_eq!(loaded.faces.len(), 12);
        // STL non comparte vértices entre triángulos: 12 caras * 3 = 36.
        assert_eq!(loaded.vertices.len(), 36);
    }

    #[test]
    fn ply_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cubo.ply");
        let mesh = cubo();
        let mut f = std::fs::File::create(&path).unwrap();
        write_ply(&mesh, &mut f).unwrap();
        drop(f);

        let loaded = read_ply(&path).unwrap();
        assert_eq!(loaded.vertices.len(), 8);
        assert_eq!(loaded.faces.len(), 12);
    }

    #[test]
    fn convert_obj_a_stl_e_a_ply() {
        let dir = tempfile::tempdir().unwrap();
        let obj_path = dir.path().join("cubo.obj");
        let mut f = std::fs::File::create(&obj_path).unwrap();
        write_obj(&cubo(), &mut f).unwrap();
        drop(f);

        let stl_path = dir.path().join("cubo.stl");
        convert_model3d(&obj_path, &stl_path, false).unwrap();
        assert!(stl_path.exists());

        let ply_path = dir.path().join("cubo.ply");
        convert_model3d(&stl_path, &ply_path, false).unwrap();
        let final_mesh = read_ply(&ply_path).unwrap();
        // Ao pasar por STL os vértices deixan de compartirse (36), pero o
        // número de caras (triángulos) mantense sempre en 12.
        assert_eq!(final_mesh.faces.len(), 12);
    }

    #[test]
    fn formato_non_soportado_da_erro_claro() {
        let dir = tempfile::tempdir().unwrap();
        let obj_path = dir.path().join("cubo.obj");
        let mut f = std::fs::File::create(&obj_path).unwrap();
        write_obj(&cubo(), &mut f).unwrap();
        drop(f);

        let gltf_path = dir.path().join("cubo.gltf");
        let err = convert_model3d(&obj_path, &gltf_path, false).unwrap_err();
        assert!(matches!(err, EngineError::Model3dUnsupported { .. }));
    }
}
