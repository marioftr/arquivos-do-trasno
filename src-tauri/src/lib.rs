mod commands;
mod explorer;
mod procs;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    explorer::gate();
    // Un traballo rápido do clic dereito non borra as vistas previas da aplicación que xa estea aberta.
    if !explorer::is_job_launch() {
        commands::clean_previews();
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .on_window_event(|_window, event| {
            // Se se pecha a xanela con ffmpeg a medias, non pode quedar un ffmpeg orfo escribindo un ficheiro roto:
            // mátase e dáselle un instante ao traballo para borrar o seu temporal.
            if let tauri::WindowEvent::Destroyed = event {
                if trasno_engine::ffmpeg::cancel_current() {
                    std::thread::sleep(std::time::Duration::from_millis(700));
                }
            }
        })
        .setup(|app| {
            explorer::watch_open_requests(app.handle().clone());
            // A xanela nace agochada para non verse a medio montar; a interface móstraa cando está lista.
            // Se algo fallase e nunca o fixese, aos 6 segundos amósase igual.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                use tauri::Manager;
                std::thread::sleep(std::time::Duration::from_secs(6));
                if let Some(w) = handle.get_webview_window("main") {
                    if !w.is_visible().unwrap_or(true) {
                        let _ = w.show();
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_category,
            commands::forget_previews,
            commands::ffmpeg_available,
            commands::tools_status,
            commands::install_tool,
            commands::cancel_tool_install,
            commands::remove_tool,
            commands::install_model,
            commands::remove_model,
            commands::transcribe_media,
            commands::remove_background,
            commands::cutout_analyze,
            commands::cutout_preview,
            commands::cutout_forget,
            commands::restore_image,
            commands::restore_image_preview,
            commands::clipboard_text,
            commands::clipboard_image,
            commands::ytdlp_info,
            commands::ytdlp_thumbnail,
            commands::ytdlp_download,
            commands::torrent_info,
            commands::torrent_download,
            commands::enable_progress,
            commands::begin_job,
            commands::cancel_job,
            commands::job_cancelled,
            commands::analyze_silence,
            commands::subtitle_info,
            commands::retime_subtitle,
            commands::subtitle_texts,
            commands::subtitle_replace_texts,
            commands::subtitle_tracks,
            commands::subtitle_embed,
            commands::restore_apply,
            commands::restore_frame,
            commands::tags_read,
            commands::tags_cover,
            commands::tags_write,
            commands::subtitle_burn,
            commands::subtitle_extract,
            commands::subtitle_two_points,
            commands::subtitle_fps_change,
            commands::cut_silence,
            commands::pdf_add_text_layer,
            commands::pdf_sanitize,
            commands::pdf_set_outline,
            commands::pdf_redact,
            commands::pdf_redact_plan,
            commands::pdf_form_fields,
            commands::make_icon_pack,
            commands::pdf_list_attachments,
            commands::pdf_edit_attachments,
            commands::pdf_fill_form,
            commands::pause_job,
            commands::discard_output,
            procs::proc_upsert,
            procs::proc_list,
            procs::proc_request_cancel,
            procs::proc_cancel_requested,
            procs::proc_clear_finished,
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
            commands::image_has_transparency,
            commands::images_to_pdf,
            commands::media_edit,
            commands::edit_apply,
            commands::edit_audio_preview,
            commands::normalize_audio,
            commands::probe_media,
            commands::sha256,
            commands::reveal_in_folder,
            commands::allow_view_dir,
            commands::make_preview,
            commands::make_preview_chunk,
            commands::make_frame_preview,
            commands::save_bytes,
            commands::save_image_bytes,
            commands::image_is_animated,
            commands::annotate_animated,
            commands::office_status,
            commands::office_targets,
            commands::convert_office,
            commands::file_info,
            commands::pdf_add_texts,
            commands::list_fonts,
            commands::capture_window,
            commands::ocr_languages,
            commands::install_ocr_language,
            commands::make_subtitles,
            commands::write_bytes,
            commands::compress_image,
            commands::resize_image,
            commands::resize_preview,
            commands::stitch_images,
            commands::slice_image,
            commands::estimate_image,
            commands::compress_video,
            commands::open_url,
            commands::expand_paths,
            explorer::open_in_app,
            commands::estimate_video,
            commands::pdf_watermark_text,
            commands::pdf_page_numbers,
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
