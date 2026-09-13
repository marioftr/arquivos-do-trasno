//! Rexistro de formatos coñecidos polo motor, agrupados por categoría.
//! O frontend traduce estes identificadores a nomes en galego; aquí só se
//! traballa con códigos estables (extensións en minúscula, sen o punto).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Imaxe,
    Audio,
    Video,
    Documento,
    Modelo3d,
    Descoñecido,
}

/// Formatos de imaxe que sabemos decodificar E codificar coa crate `image`
/// (as listas non son simétricas augas arriba: HEIC/AVIF de lectura, por
/// exemplo, requiren códecs adicionais que se engadirán máis adiante).
pub const IMAGE_DECODE: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "tif", "webp", "pnm", "pbm", "pgm", "ppm",
    "tga", "qoi", "avif",
];

pub const IMAGE_ENCODE: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "tif", "webp", "pnm", "pbm", "pgm", "ppm",
    "tga", "qoi",
];

pub const AUDIO_EXTS: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "wma", "aiff",
];

pub const VIDEO_EXTS: &[&str] = &[
    "mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "mpg", "mpeg", "gif",
];

pub const DOCUMENT_EXTS: &[&str] = &["pdf"];

pub const MODEL3D_EXTS: &[&str] = &["obj", "stl", "ply", "gltf", "glb", "fbx"];

/// Subconxunto de `MODEL3D_EXTS` que o motor sabe realmente converter hoxe
/// (malla de triángulos: vértices + caras). glTF/GLB/FBX recoñécense para
/// detectar a categoría do ficheiro pero a conversión queda para máis
/// adiante (formatos binarios/JSON máis complexos, con materiais e escenas).
pub const MODEL3D_ENGINE_SUPPORTED: &[&str] = &["obj", "stl", "ply"];

pub fn extension_of(path: &std::path::Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
}

pub fn category_of_extension(ext: &str) -> Category {
    let ext = ext.to_ascii_lowercase();
    if IMAGE_DECODE.contains(&ext.as_str()) {
        Category::Imaxe
    } else if AUDIO_EXTS.contains(&ext.as_str()) {
        Category::Audio
    } else if VIDEO_EXTS.contains(&ext.as_str()) {
        Category::Video
    } else if DOCUMENT_EXTS.contains(&ext.as_str()) {
        Category::Documento
    } else if MODEL3D_EXTS.contains(&ext.as_str()) {
        Category::Modelo3d
    } else {
        Category::Descoñecido
    }
}

pub fn category_of_path(path: &std::path::Path) -> Category {
    match extension_of(path) {
        Some(ext) => category_of_extension(&ext),
        None => Category::Descoñecido,
    }
}
