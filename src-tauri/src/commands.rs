//! Comandos de Tauri: a fronteira fina entre a UI (TypeScript/Svelte) e o
//! motor de conversión (`trasno-engine`). Non hai lóxica de conversión
//! aquí, só validación de entrada e tradución de erros a texto para a UI.

use serde::Serialize;
use std::path::PathBuf;
use trasno_engine::{Category, ConvertOptions};

fn to_err(e: trasno_engine::EngineError) -> String {
    e.to_string()
}

/// Executa traballo pesado fóra do fío da interface, para que a xanela non
/// se conxele mentres o trasno converte.
async fn blocking<T, F>(job: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|e| e.to_string())?
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

/// Permite ao visor da UI ler *este* ficheiro (e só este) a través do
/// protocolo `asset:`. O ámbito do protocolo empeza baleiro: cada ficheiro
/// que o usuario abre engádese aquí, nunca un cartafol ou unha unidade enteira.
#[tauri::command]
pub fn allow_view(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use tauri::Manager;
    app.asset_protocol_scope()
        .allow_file(&path)
        .map_err(|e| e.to_string())
}

/// Ficheiros pasados na liña de ordes (p. ex. "Abrir con" dende o explorador).
#[tauri::command]
pub fn startup_files() -> Vec<String> {
    std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-') && PathBuf::from(a).is_file())
        .collect()
}

#[tauri::command]
pub fn image_dimensions(path: String) -> Result<(u32, u32), String> {
    trasno_engine::image_convert::image_dimensions(&PathBuf::from(path)).map_err(to_err)
}

#[tauri::command]
pub async fn convert_file(
    input: String,
    output: String,
    overwrite: bool,
    quality: Option<u8>,
    resize: Option<(u32, u32)>,
    bitrate_kbps: Option<u32>,
) -> Result<ConvertResult, String> {
    let options = ConvertOptions {
        overwrite,
        quality,
        resize,
        bitrate_kbps,
    };
    blocking(move || {
        trasno_engine::convert_file(&PathBuf::from(input), &PathBuf::from(output), &options)
            .map(|r| ConvertResult {
                output_path: r.output_path.to_string_lossy().to_string(),
                bytes_written: r.bytes_written,
                category: r.category,
            })
            .map_err(to_err)
    })
    .await
}

#[tauri::command]
pub async fn trim_media(
    input: String,
    output: String,
    overwrite: bool,
    start_seconds: f64,
    end_seconds: Option<f64>,
) -> Result<String, String> {
    blocking(move || {
        trasno_engine::ffmpeg::trim_media(
            &PathBuf::from(input),
            &PathBuf::from(output),
            overwrite,
            start_seconds,
            end_seconds,
        )
        .map(|p| p.to_string_lossy().to_string())
        .map_err(to_err)
    })
    .await
}

#[tauri::command]
pub async fn concat_media(inputs: Vec<String>, output: String, overwrite: bool) -> Result<String, String> {
    let inputs: Vec<PathBuf> = inputs.into_iter().map(PathBuf::from).collect();
    blocking(move || {
        trasno_engine::ffmpeg::concat_media(&inputs, &PathBuf::from(output), overwrite)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(to_err)
    })
    .await
}

#[tauri::command]
pub async fn pdf_merge(inputs: Vec<String>, output: String, overwrite: bool) -> Result<String, String> {
    let inputs: Vec<PathBuf> = inputs.into_iter().map(PathBuf::from).collect();
    blocking(move || {
        trasno_engine::pdf::merge_pdfs(&inputs, &PathBuf::from(output), overwrite)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(to_err)
    })
    .await
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

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ImageEditsDto {
    pub rotate: u16,
    pub flip_h: bool,
    pub flip_v: bool,
    pub grayscale: bool,
    pub invert: bool,
    pub brightness: i32,
    pub contrast: f32,
    pub blur: f32,
    pub crop: Option<(u32, u32, u32, u32)>,
    pub quality: Option<u8>,
    pub resize: Option<(u32, u32)>,
}

#[tauri::command]
pub async fn edit_image(
    input: String,
    output: String,
    overwrite: bool,
    edits: ImageEditsDto,
) -> Result<String, String> {
    blocking(move || {
        let e = trasno_engine::image_convert::ImageEdits {
            rotate: edits.rotate,
            flip_h: edits.flip_h,
            flip_v: edits.flip_v,
            grayscale: edits.grayscale,
            invert: edits.invert,
            brightness: edits.brightness,
            contrast: edits.contrast,
            blur: edits.blur,
            crop: edits.crop,
            quality: edits.quality,
            resize: edits.resize,
        };
        trasno_engine::image_convert::edit_image(&PathBuf::from(input), &PathBuf::from(output), overwrite, &e)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(to_err)
    })
    .await
}

#[tauri::command]
pub async fn images_to_pdf(inputs: Vec<String>, output: String, overwrite: bool) -> Result<String, String> {
    let inputs: Vec<PathBuf> = inputs.into_iter().map(PathBuf::from).collect();
    blocking(move || {
        trasno_engine::pdf::images_to_pdf(&inputs, &PathBuf::from(output), overwrite)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(to_err)
    })
    .await
}

/// `op`: "frame", "speed", "mute", "volume" ou "scale".
#[tauri::command]
pub async fn media_edit(
    input: String,
    output: String,
    overwrite: bool,
    op: String,
    value: f64,
    has_video: bool,
) -> Result<String, String> {
    use trasno_engine::ffmpeg::MediaOp;
    let op = match op.as_str() {
        "frame" => MediaOp::Frame(value),
        "speed" => MediaOp::Speed(value),
        "mute" => MediaOp::Mute,
        "volume" => MediaOp::Volume(value),
        "scale" => MediaOp::ScaleHeight(value.max(0.0) as u32),
        other => return Err(format!("operación descoñecida: {other}")),
    };
    blocking(move || {
        trasno_engine::ffmpeg::media_edit(&PathBuf::from(input), &PathBuf::from(output), overwrite, op, has_video)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(to_err)
    })
    .await
}

#[tauri::command]
pub async fn probe_media(path: String) -> Result<serde_json::Value, String> {
    blocking(move || trasno_engine::ffmpeg::probe(&PathBuf::from(path)).map_err(to_err)).await
}

#[tauri::command]
pub async fn sha256(path: String) -> Result<String, String> {
    blocking(move || trasno_engine::integrity::sha256_file(&PathBuf::from(path)).map_err(to_err)).await
}

/// Mostra o ficheiro no explorador do sistema.
#[tauri::command]
pub fn reveal_in_folder(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err("o ficheiro xa non existe".into());
    }
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", p.display()))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg("-R").arg(&p).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = p.parent().unwrap_or(&p);
        std::process::Command::new("xdg-open").arg(dir).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}
