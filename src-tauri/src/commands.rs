//! Comandos de Tauri: a fronteira fina entre a UI (TypeScript/Svelte) e o
//! motor de conversión (`trasno-engine`). Non hai lóxica de conversión
//! aquí, só validación de entrada e tradución de erros a texto para a UI.

use serde::Serialize;
use std::path::PathBuf;
use trasno_engine::{Category, ConvertOptions};

fn to_err(e: trasno_engine::EngineError) -> String {
    e.to_string()
}

#[derive(Serialize)]
pub struct ConvertResult {
    pub output_path: String,
    pub bytes_written: u64,
    pub category: Category,
}

#[tauri::command]
pub fn detect_category(path: String) -> Category {
    trasno_engine::formats::category_of_path(&PathBuf::from(path))
}

#[tauri::command]
pub fn suggested_targets(path: String) -> Vec<&'static str> {
    trasno_engine::suggested_targets(&PathBuf::from(path))
}

#[tauri::command]
pub fn ffmpeg_available() -> bool {
    trasno_engine::ffmpeg::is_available()
}

#[tauri::command]
pub fn file_size(path: String) -> Result<u64, String> {
    std::fs::metadata(&path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn convert_file(
    input: String,
    output: String,
    overwrite: bool,
    quality: Option<u8>,
    resize: Option<(u32, u32)>,
) -> Result<ConvertResult, String> {
    let options = ConvertOptions {
        overwrite,
        quality,
        resize,
    };
    trasno_engine::convert_file(&PathBuf::from(input), &PathBuf::from(output), &options)
        .map(|r| ConvertResult {
            output_path: r.output_path.to_string_lossy().to_string(),
            bytes_written: r.bytes_written,
            category: r.category,
        })
        .map_err(to_err)
}

#[tauri::command]
pub fn trim_media(
    input: String,
    output: String,
    overwrite: bool,
    start_seconds: f64,
    end_seconds: Option<f64>,
) -> Result<String, String> {
    trasno_engine::ffmpeg::trim_media(
        &PathBuf::from(input),
        &PathBuf::from(output),
        overwrite,
        start_seconds,
        end_seconds,
    )
    .map(|p| p.to_string_lossy().to_string())
    .map_err(to_err)
}

#[tauri::command]
pub fn concat_media(inputs: Vec<String>, output: String, overwrite: bool) -> Result<String, String> {
    let inputs: Vec<PathBuf> = inputs.into_iter().map(PathBuf::from).collect();
    trasno_engine::ffmpeg::concat_media(&inputs, &PathBuf::from(output), overwrite)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(to_err)
}

#[tauri::command]
pub fn pdf_merge(inputs: Vec<String>, output: String, overwrite: bool) -> Result<String, String> {
    let inputs: Vec<PathBuf> = inputs.into_iter().map(PathBuf::from).collect();
    trasno_engine::pdf::merge_pdfs(&inputs, &PathBuf::from(output), overwrite)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(to_err)
}

#[tauri::command]
pub fn pdf_extract_pages(
    input: String,
    pages: Vec<u32>,
    output: String,
    overwrite: bool,
) -> Result<String, String> {
    trasno_engine::pdf::extract_pages(&PathBuf::from(input), &pages, &PathBuf::from(output), overwrite)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(to_err)
}

#[tauri::command]
pub fn pdf_rotate(
    input: String,
    page: Option<u32>,
    degrees: i64,
    output: String,
    overwrite: bool,
) -> Result<String, String> {
    trasno_engine::pdf::rotate_pages(&PathBuf::from(input), page, degrees, &PathBuf::from(output), overwrite)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(to_err)
}

#[tauri::command]
pub fn pdf_page_count(input: String) -> Result<u32, String> {
    trasno_engine::pdf::page_count(&PathBuf::from(input)).map_err(to_err)
}
