mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_category,
            commands::suggested_targets,
            commands::ffmpeg_available,
            commands::file_size,
            commands::image_dimensions,
            commands::convert_file,
            commands::trim_media,
            commands::concat_media,
            commands::pdf_merge,
            commands::pdf_extract_pages,
            commands::pdf_rotate,
            commands::pdf_page_count,
        ])
        .run(tauri::generate_context!())
        .expect("erro executando a aplicación Trasno");
}
