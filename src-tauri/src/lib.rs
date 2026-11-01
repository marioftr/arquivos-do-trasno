mod commands;
mod explorer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    explorer::gate();
    commands::clean_previews();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            explorer::watch_open_requests(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_category,
            commands::suggested_targets,
            commands::ffmpeg_available,
            commands::tools_status,
            commands::mkv_tracks,
            commands::mkv_mux,
            commands::open_link,
            commands::unique_path,
            explorer::explorer_install,
            explorer::explorer_uninstall,
            explorer::explorer_status,
            explorer::startup_job,
            commands::file_size,
            commands::allow_view,
            commands::startup_files,
            commands::edit_image,
            commands::images_to_pdf,
            commands::media_edit,
            commands::probe_media,
            commands::sha256,
            commands::reveal_in_folder,
            commands::allow_view_dir,
            commands::make_preview,
            commands::write_bytes,
            commands::compress_image,
            commands::estimate_image,
            commands::compress_video,
            commands::estimate_video,
            commands::pdf_watermark_text,
            commands::pdf_page_numbers,
            commands::pdf_add_text,
            commands::pdf_stamp_image,
            commands::pdf_crop,
            commands::pdf_compress,
            commands::pdf_protect,
            commands::pdf_unprotect,
            commands::pdf_is_encrypted,
            commands::pdf_arrange,
            commands::pdf_repair,
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
        .expect("erro executando a aplicación Arquivos do Trasno");
}
