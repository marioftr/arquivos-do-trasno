//! Comandos de Tauri: a fronteira fina entre a UI (TypeScript/Svelte) e o
//! motor de conversión (`trasno-engine`). Non hai lóxica de conversión
//! aquí, só validación de entrada e tradución de erros a texto para a UI.

use serde::Serialize;
use std::path::PathBuf;
use trasno_engine::{Category, ConvertOptions};

fn to_err(e: trasno_engine::EngineError) -> String {
    e.wire()
}

/// Mensaxe propia da aplicación nunha forma que a interface pode traducir.
fn msg(template: &str, params: serde_json::Value) -> String {
    format!("@@{}", serde_json::json!({ "k": template, "p": params }))
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
pub fn begin_job() {
    trasno_engine::ffmpeg::begin_job();
}

/// Cancela o traballo en marcha. Devolve `true` se había un ffmpeg a correr.
#[tauri::command]
pub fn cancel_job() -> bool {
    trasno_engine::ffmpeg::cancel_current()
}

/// Borra un ficheiro que acaba de crear o propio traballo cancelado. Por seguridade só vale para ficheiros
/// creados desde que arrancou esta aplicación (nunca un orixinal) e que non sexan cartafoles.
#[tauri::command]
pub fn discard_output(path: String) -> bool {
    use std::sync::OnceLock;
    static STARTED: OnceLock<std::time::SystemTime> = OnceLock::new();
    let started = *STARTED.get_or_init(std::time::SystemTime::now);
    let p = PathBuf::from(&path);
    let Ok(meta) = std::fs::metadata(&p) else { return false };
    let created = meta.created().or_else(|_| meta.modified()).unwrap_or(std::time::UNIX_EPOCH);
    // Dá un pouco de marxe: STARTED fíxase na primeira chamada, non ao arrancar.
    let limit = started.checked_sub(std::time::Duration::from_secs(3600)).unwrap_or(started);
    if !meta.is_file() || created < limit {
        return false;
    }
    std::fs::remove_file(&p).is_ok()
}

/// Activa os avisos de progreso do ffmpeg: a interface recibe `trasno://progress` (0 a 1).
#[tauri::command]
pub fn enable_progress(app: tauri::AppHandle) {
    use tauri::Emitter;
    let last = std::sync::Mutex::new(-1.0f64);
    trasno_engine::ffmpeg::set_progress_sink(Some(std::sync::Arc::new(move |f: f64| {
        let mut l = last.lock().unwrap_or_else(|e| e.into_inner());
        // Sen inundar a interface: só cando cambia ao menos un 0,5 %.
        if (f - *l).abs() >= 0.005 || f >= 1.0 {
            *l = f;
            let _ = app.emit("trasno://progress", f);
        }
    })));
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
    // Un traballo do menú de clic dereito (--convert / --compress) non abre os ficheiros na lista.
    if matches!(std::env::args().nth(1).as_deref(), Some("--convert") | Some("--compress")) {
        return Vec::new();
    }
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
        "normalize" => MediaOp::Normalize,
        "reverse" => MediaOp::Reverse,
        "fadein" => MediaOp::FadeIn(value),
        "fadeout" => MediaOp::FadeOut(value),
        "echo" => MediaOp::Echo(value),
        "bass" => MediaOp::Bass(value),
        "treble" => MediaOp::Treble(value),
        "denoise" => MediaOp::Denoise,
        other => return Err(msg("Non coñezo a operación «{op}».", serde_json::json!({ "op": other }))),
    };
    blocking(move || {
        trasno_engine::ffmpeg::media_edit(&PathBuf::from(input), &PathBuf::from(output), overwrite, op, has_video)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(to_err)
    })
    .await
}

/// Troca os cartafoles por os ficheiros que levan dentro (ata 6 niveis e 3000 ficheiros); os ficheiros pasan tal cal.
#[tauri::command]
pub async fn expand_paths(paths: Vec<String>) -> Vec<String> {
    blocking(move || {
        fn walk(dir: &std::path::Path, depth: u32, out: &mut Vec<String>) {
            let Ok(rd) = std::fs::read_dir(dir) else { return };
            let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            entries.sort();
            for p in entries {
                if out.len() >= 3000 {
                    return;
                }
                let hidden = p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.'));
                if hidden {
                    continue;
                }
                if p.is_dir() {
                    if depth < 6 {
                        walk(&p, depth + 1, out);
                    }
                } else if p.is_file() {
                    out.push(p.to_string_lossy().to_string());
                }
            }
        }
        let mut out = Vec::new();
        for p in paths {
            let path = PathBuf::from(&p);
            if path.is_dir() {
                walk(&path, 0, &mut out);
            } else {
                out.push(p);
            }
        }
        Ok::<_, String>(out)
    })
    .await
    .unwrap_or_default()
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
        return Err(msg("O ficheiro xa non existe.", serde_json::json!({})));
    }
    #[cfg(windows)]
    {
        // Explorador non entende o argumento ao que Rust pon comiñas por fóra (rutas con espazos abrían «Documentos»):
        // pásase en bruto, co formato exacto «/select,"ruta"» e con barras de Windows.
        use std::os::windows::process::CommandExt;
        let win = p.display().to_string().replace('/', "\\");
        std::process::Command::new("explorer")
            .raw_arg(format!("/select,\"{win}\""))
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

/// Como `allow_view`, pero para o cartafol dun modelo 3D: as texturas e os
/// materiais viven ao seu carón. Só lectura; non se amplía a ningún outro sitio.
#[tauri::command]
pub fn allow_view_dir(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use tauri::Manager;
    let p = PathBuf::from(&path);
    let dir = p.parent().ok_or_else(|| msg("A ruta non ten cartafol.", serde_json::json!({})))?;
    app.asset_protocol_scope()
        .allow_directory(dir, true)
        .map_err(|e| e.to_string())?;
    app.asset_protocol_scope()
        .allow_file(&p)
        .map_err(|e| e.to_string())
}

fn preview_dir() -> PathBuf {
    std::env::temp_dir().join("arquivos-do-trasno-previsualizacions")
}

/// Borra as vistas previas de execucións anteriores.
pub fn clean_previews() {
    let _ = std::fs::remove_dir_all(preview_dir());
    // Restos de colas do menú do clic dereito (se unha apertura morreu a medias).
    for name in ["trasno-jobs", "trasno-open"] {
        let dir = std::env::temp_dir().join(name);
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let old = e.metadata().and_then(|m| m.modified()).ok().and_then(|m| m.elapsed().ok()).is_some_and(|a| a.as_secs() > 3600);
                let p = e.path();
                if old && p.extension().and_then(|x| x.to_str()) != Some("lock") {
                    let _ = std::fs::remove_file(p);
                }
            }
        }
    }
}

/// Xera unha copia lixeira que o visor si sabe amosar (PNG para imaxes que o
/// navegador non le; MP4 pequeno ou MP3 para audio e vídeo). O orixinal non
/// se toca.
#[tauri::command]
pub async fn make_preview(path: String, video: Option<u32>, audio: Option<u32>) -> Result<String, String> {
    blocking(move || {
        let input = PathBuf::from(&path);
        let category = trasno_engine::formats::category_of_path(&input);
        let dir = preview_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let stem = format!("{:x}", {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            path.hash(&mut h);
            h.finish()
        });
        let tag = track_tag(video, audio);
        let maps = track_maps(video, audio);
        let out = match category {
            Category::Imaxe => {
                let out = dir.join(format!("{stem}.png"));
                if !out.exists() {
                    trasno_engine::convert_file(&input, &out, &ConvertOptions { overwrite: true, ..Default::default() })
                        .map_err(to_err)?;
                }
                out
            }
            Category::Video => {
                // Os contedores que o visor non entende (AVI, MKV...) a miúdo levan códecs que si entende: nese caso
                // só se cambia a caixa (en segundos e sen perder calidade). Se non, transcódase a 720p como moito.
                let codecs = trasno_engine::ffmpeg::probe(&input).ok().map(|info| {
                    let name = |kind: &str, n: u32| {
                        info["streams"].as_array().and_then(|l| l.iter().filter(|s| s["codec_type"] == kind).nth(n as usize)).and_then(|s| s["codec_name"].as_str()).map(|s| s.to_string())
                    };
                    (name("video", video.unwrap_or(0)), name("audio", audio.unwrap_or(0)))
                });
                let copyable = matches!(&codecs, Some((Some(v), a)) if v == "h264" && a.as_deref().map_or(true, |a| matches!(a, "aac" | "mp3")));
                let webm_copyable = matches!(&codecs, Some((Some(v), a)) if matches!(v.as_str(), "vp8" | "vp9" | "av1") && a.as_deref().map_or(true, |a| matches!(a, "opus" | "vorbis")));
                if webm_copyable {
                    let out = dir.join(format!("{stem}{tag}.remux.webm"));
                    if !out.exists() {
                        let mut args: Vec<String> = maps.clone();
                        args.extend(["-c", "copy"].iter().map(|s| s.to_string()));
                        trasno_engine::ffmpeg::convert_media(&input, &out, true, &args).map_err(to_err)?;
                    }
                    out
                } else if copyable {
                    let out = dir.join(format!("{stem}{tag}.remux.mp4"));
                    if !out.exists() {
                        let mut args: Vec<String> = maps.clone();
                        args.extend(["-c", "copy", "-movflags", "+faststart"].iter().map(|s| s.to_string()));
                        trasno_engine::ffmpeg::convert_media(&input, &out, true, &args).map_err(to_err)?;
                    }
                    out
                } else {
                    // O navegador non le este vídeo: o visor pídeo despois por tramos (`make_preview_chunk`).
                    return Ok("chunks".to_string());
                }
            }
            Category::Audio => {
                let out = dir.join(format!("{stem}{tag}.flac"));
                if !out.exists() {
                    let mut args: Vec<String> = vec!["-vn".to_string()];
                    if let Some(a) = audio {
                        args.extend(["-map".to_string(), format!("0:a:{a}?")]);
                    }
                    args.extend(["-c:a", "flac"].iter().map(|s| s.to_string()));
                    trasno_engine::ffmpeg::convert_media(&input, &out, true, &args).map_err(to_err)?;
                }
                out
            }
            _ => return Err(msg("Para este tipo de ficheiro non hai vista previa alternativa.", serde_json::json!({}))),
        };
        Ok(out.to_string_lossy().to_string())
    })
    .await
}

/// Marca no nome do ficheiro temporal segundo as pistas elixidas (nada se se usan as de por defecto).
fn track_tag(video: Option<u32>, audio: Option<u32>) -> String {
    if video.is_none() && audio.is_none() {
        String::new()
    } else {
        format!(".t{}_{}", video.unwrap_or(0), audio.unwrap_or(0))
    }
}

/// `-map` das pistas elixidas (índices dentro do seu tipo); sen elección, ffmpeg escolle as de por defecto.
fn track_maps(video: Option<u32>, audio: Option<u32>) -> Vec<String> {
    if video.is_none() && audio.is_none() {
        Vec::new()
    } else {
        vec!["-map".to_string(), format!("0:v:{}?", video.unwrap_or(0)), "-map".to_string(), format!("0:a:{}?", audio.unwrap_or(0))]
    }
}

/// Subtítulos dun ficheiro de vídeo en WebVTT (só se son de texto), para amosalos por riba do vídeo.
#[tauri::command]
pub async fn make_subtitles(path: String, index: u32) -> Result<String, String> {
    blocking(move || {
        let input = PathBuf::from(&path);
        let dir = preview_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let stem = format!("{:x}", {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            path.hash(&mut h);
            h.finish()
        });
        let out = dir.join(format!("{stem}.sub{index}.vtt"));
        if !out.exists() {
            let args: Vec<String> = ["-map".to_string(), format!("0:s:{index}"), "-c:s".to_string(), "webvtt".to_string()].to_vec();
            trasno_engine::ffmpeg::convert_media(&input, &out, true, &args).map_err(to_err)?;
        }
        Ok(out.to_string_lossy().to_string())
    })
    .await
}

/// Un tramo de vídeo (`start`..`start+len` segundos) recodificado só para velo: mesma resolución, calidade moi alta.
/// O visor pide o tramo que precisa (ao saltar) e o seguinte (ao reproducir), así que se pode ir a calquera punto sen esperar.
#[tauri::command]
pub async fn make_preview_chunk(path: String, start: f64, len: f64, video: Option<u32>, audio: Option<u32>) -> Result<String, String> {
    blocking(move || {
        if !(start >= 0.0 && start.is_finite() && len > 0.0 && len <= 300.0) {
            return Err(msg("Tramo non válido.", serde_json::json!({})));
        }
        let input = PathBuf::from(&path);
        let dir = preview_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let stem = format!("{:x}", {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            path.hash(&mut h);
            h.finish()
        });
        let out = dir.join(format!("{stem}{}.s{}.mp4", track_tag(video, audio), (start * 1000.0) as u64));
        if !out.exists() {
            let pre = vec!["-ss".to_string(), format!("{start:.3}")];
            let mut args: Vec<String> = track_maps(video, audio);
            args.extend(
                ["-t", &format!("{len:.3}"), "-vf", "yadif=0:-1:1", "-c:v", "libx264", "-preset", "ultrafast", "-crf", "17", "-c:a", "aac", "-b:a", "256k", "-pix_fmt", "yuv420p", "-movflags", "+faststart"]
                    .iter()
                    .map(|s| s.to_string()),
            );
            trasno_engine::ffmpeg::convert_media_seek(&input, &out, true, &pre, &args).map_err(to_err)?;
        }
        Ok(out.to_string_lossy().to_string())
    })
    .await
}

/// Garda os bytes que chegan no corpo (a interface xa renderizou a imaxe) no ficheiro da cabeceira `x-path`.
/// Escríbese nun temporal ao carón e renómease, así nunca queda un ficheiro a medias.
#[tauri::command]
pub async fn save_bytes(request: tauri::ipc::Request<'_>) -> Result<String, String> {
    let tauri::ipc::InvokeBody::Raw(data) = request.body() else {
        return Err(msg("Non chegaron datos para gardar.", serde_json::json!({})));
    };
    let data = data.clone();
    let path = request
        .headers()
        .get("x-path")
        .and_then(|v| v.to_str().ok())
        .map(percent_decode)
        .ok_or_else(|| msg("Falta a ruta de destino.", serde_json::json!({})))?;
    blocking(move || {
        let out = PathBuf::from(&path);
        if data.is_empty() {
            return Err(msg("Non hai nada que gardar.", serde_json::json!({})));
        }
        let dir = out.parent().ok_or_else(|| msg("A ruta non ten cartafol.", serde_json::json!({})))?;
        let tmp = dir.join(format!(".{}.part", out.file_name().and_then(|n| n.to_str()).unwrap_or("saida")));
        std::fs::write(&tmp, &data).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &out).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            e.to_string()
        })?;
        Ok(path)
    })
    .await
}

/// Un fotograma dun vídeo (en calquera formato que lea ffmpeg) como JPG pequeno, para a vista previa de «Extraer fotogramas».
#[tauri::command]
pub async fn make_frame_preview(path: String, time: f64) -> Result<String, String> {
    blocking(move || {
        if !(time >= 0.0 && time.is_finite()) {
            return Err(msg("Instante non válido.", serde_json::json!({})));
        }
        let input = PathBuf::from(&path);
        let dir = preview_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let stem = format!("{:x}", {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            path.hash(&mut h);
            h.finish()
        });
        let out = dir.join(format!("{stem}.f{}.jpg", (time * 1000.0) as u64));
        if !out.exists() {
            let pre = vec!["-ss".to_string(), format!("{time:.3}")];
            let args: Vec<String> = ["-an", "-frames:v", "1", "-vf", "yadif=0:-1:0,scale=trunc(iw*sar/2)*2:ih,setsar=1,scale=640:-2", "-q:v", "3"].iter().map(|s| s.to_string()).collect();
            trasno_engine::ffmpeg::convert_media_seek(&input, &out, true, &pre, &args).map_err(to_err)?;
        }
        Ok(out.to_string_lossy().to_string())
    })
    .await
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

/// Garda bytes xerados na interface (p. ex. un modelo 3D exportado) de forma
/// atómica: temporal, validación e `rename`. Nunca pisa o ficheiro de orixe.
#[tauri::command]
pub fn write_bytes(request: tauri::ipc::Request<'_>) -> Result<(), String> {
    let header = |name: &str| -> Result<String, String> {
        request
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(percent_decode)
            .ok_or_else(|| format!("falta a cabeceira {name}"))
    };
    let path = PathBuf::from(header("x-path")?);
    let source = PathBuf::from(header("x-source")?);
    let overwrite = header("x-overwrite").map(|v| v == "1").unwrap_or(false);
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err(msg("A petición chegou sen datos.", serde_json::json!({})));
    };
    trasno_engine::integrity::ensure_distinct(&[source.as_path()], &path).map_err(to_err)?;
    let mut out = trasno_engine::integrity::AtomicOutput::new(&path, overwrite).map_err(to_err)?;
    out.write_all(bytes).map_err(to_err)?;
    out.commit().map_err(to_err)?;
    Ok(())
}

#[tauri::command]
pub async fn compress_image(
    input: String,
    output: String,
    overwrite: bool,
    quality: Option<u8>,
    target_bytes: Option<u64>,
    max_side: Option<u32>,
) -> Result<trasno_engine::compress::ImageCompressReport, String> {
    blocking(move || {
        let o = trasno_engine::compress::ImageCompress { quality, target_bytes, max_side };
        trasno_engine::compress::compress_image(&PathBuf::from(input), &PathBuf::from(output), overwrite, &o)
            .map_err(to_err)
    })
    .await
}

#[tauri::command]
pub async fn estimate_image(input: String, quality: u8, max_side: Option<u32>) -> Result<u64, String> {
    blocking(move || {
        trasno_engine::compress::estimate_image_bytes(&PathBuf::from(input), quality, max_side).map_err(to_err)
    })
    .await
}

#[tauri::command]
pub async fn compress_video(
    input: String,
    output: String,
    overwrite: bool,
    crf: u8,
    target_bytes: Option<u64>,
    max_height: Option<u32>,
    audio_kbps: u32,
    video_kbps: Option<u32>,
    deinterlace: Option<u8>,
    compat: Option<bool>,
    dar: Option<String>,
) -> Result<String, String> {
    blocking(move || {
        let o = trasno_engine::compress::VideoCompress {
            crf,
            target_bytes,
            max_height,
            audio_kbps,
            video_kbps,
            deinterlace: deinterlace.unwrap_or(0),
            compat: compat.unwrap_or(false),
            dar,
        };
        trasno_engine::compress::compress_video(&PathBuf::from(input), &PathBuf::from(output), overwrite, &o)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(to_err)
    })
    .await
}

#[tauri::command]
pub async fn estimate_video(
    input: String,
    crf: u8,
    max_height: Option<u32>,
    audio_kbps: u32,
    deinterlace: Option<u8>,
    compat: Option<bool>,
    dar: Option<String>,
) -> Result<u64, String> {
    blocking(move || {
        let o = trasno_engine::compress::VideoCompress {
            crf,
            max_height,
            audio_kbps,
            deinterlace: deinterlace.unwrap_or(0),
            compat: compat.unwrap_or(false),
            dar,
            ..Default::default()
        };
        trasno_engine::compress::estimate_video_bytes(&PathBuf::from(input), &o).map_err(to_err)
    })
    .await
}

// ------------------------------------------------------------ ferramentas PDF

fn paths(input: String, output: String) -> (PathBuf, PathBuf) {
    (PathBuf::from(input), PathBuf::from(output))
}

fn done(r: trasno_engine::Result<PathBuf>) -> Result<String, String> {
    r.map(|p| p.to_string_lossy().to_string()).map_err(to_err)
}

#[tauri::command]
pub async fn pdf_watermark_text(
    input: String, output: String, overwrite: bool,
    text: String, size: f32, opacity: f32, angle: f32, color: String, font: Option<String>, pages: Option<Vec<u32>>,
) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        let w = trasno_engine::pdf_tools::WatermarkText { text, size, opacity, angle, color, font };
        done(trasno_engine::pdf_tools::watermark_text(&i, &o, overwrite, &w, &pages))
    })
    .await
}

#[tauri::command]
pub async fn pdf_page_numbers(
    input: String, output: String, overwrite: bool,
    position: String, format: String, start: i64, size: f32, margin: f32, color: String, font: Option<String>, pages: Option<Vec<u32>>,
) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        let n = trasno_engine::pdf_tools::PageNumbers { position, format, start, size, margin, color, font };
        done(trasno_engine::pdf_tools::page_numbers(&i, &o, overwrite, &n, &pages))
    })
    .await
}

#[tauri::command]
pub async fn pdf_add_text(
    input: String, output: String, overwrite: bool,
    text: String, x_pct: f32, y_pct: f32, size: f32, color: String, font: Option<String>, pages: Option<Vec<u32>>,
) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        let t = trasno_engine::pdf_tools::AddText { text, x_pct, y_pct, size, color, font };
        done(trasno_engine::pdf_tools::add_text(&i, &o, overwrite, &t, &pages))
    })
    .await
}

#[tauri::command]
pub async fn pdf_stamp_image(
    input: String, output: String, overwrite: bool,
    image: String, x_pct: f32, y_pct: f32, width_pct: f32, opacity: f32, pages: Option<Vec<u32>>,
) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        let st = trasno_engine::pdf_tools::StampImage { image: PathBuf::from(image), x_pct, y_pct, width_pct, opacity };
        done(trasno_engine::pdf_tools::stamp_image(&i, &o, overwrite, &st, &pages))
    })
    .await
}

#[tauri::command]
pub async fn pdf_crop(
    input: String, output: String, overwrite: bool,
    left: f32, bottom: f32, right: f32, top: f32, pages: Option<Vec<u32>>,
) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        done(trasno_engine::pdf_tools::crop_pages(&i, &o, overwrite, left, bottom, right, top, &pages))
    })
    .await
}

#[tauri::command]
pub async fn pdf_compress(input: String, output: String, overwrite: bool, quality: u8, scale_percent: u32) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        done(trasno_engine::pdf_tools::compress_pdf(&i, &o, overwrite, &trasno_engine::pdf_tools::PdfCompress { quality, scale_percent }))
    })
    .await
}

#[tauri::command]
pub async fn pdf_protect(
    input: String, output: String, overwrite: bool,
    user_password: String, owner_password: String, allow_print: bool, allow_copy: bool,
) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        done(trasno_engine::pdf_tools::protect_pdf(&i, &o, overwrite, &user_password, &owner_password, allow_print, allow_copy))
    })
    .await
}

#[tauri::command]
pub async fn pdf_unprotect(input: String, output: String, overwrite: bool, password: String) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        done(trasno_engine::pdf_tools::unprotect_pdf(&i, &o, overwrite, &password))
    })
    .await
}

#[tauri::command]
pub async fn pdf_is_encrypted(input: String) -> Result<bool, String> {
    blocking(move || trasno_engine::pdf_tools::is_encrypted(&PathBuf::from(input)).map_err(to_err)).await
}

#[tauri::command]
pub async fn pdf_arrange(input: String, output: String, overwrite: bool, order: Vec<(u32, i64)>) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        done(trasno_engine::pdf_tools::arrange_pages(&i, &o, overwrite, &order))
    })
    .await
}

#[tauri::command]
pub async fn pdf_repair(input: String, output: String, overwrite: bool) -> Result<String, String> {
    blocking(move || {
        let (i, o) = paths(input, output);
        done(trasno_engine::pdf_tools::repair_pdf(&i, &o, overwrite))
    })
    .await
}

/// Un programa instalado mentres a app está aberta non aparece no PATH que ela herdou ao arrancar:
/// volve ler o PATH do sistema e o do usuario desde o rexistro de Windows.
#[cfg(windows)]
fn refresh_path() {
    fn read(key: &str) -> String {
        let mut cmd = std::process::Command::new("reg");
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        let Ok(out) = cmd.args(["query", key, "/v", "Path"]).output() else { return String::new() };
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        text.lines()
            .find(|l| l.trim_start().starts_with("Path"))
            .and_then(|l| l.split("REG_EXPAND_SZ").nth(1).or_else(|| l.split("REG_SZ").nth(1)))
            .map(|v| v.trim().to_string())
            .unwrap_or_default()
    }
    fn expand(v: &str) -> String {
        let mut out = String::new();
        let mut rest = v;
        while let Some(i) = rest.find('%') {
            out.push_str(&rest[..i]);
            let tail = &rest[i + 1..];
            match tail.find('%') {
                Some(j) => {
                    out.push_str(&std::env::var(&tail[..j]).unwrap_or_default());
                    rest = &tail[j + 1..];
                }
                None => {
                    out.push('%');
                    rest = tail;
                }
            }
        }
        out.push_str(rest);
        out
    }
    let machine = read(r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment");
    let user = read(r"HKCU\Environment");
    let current = std::env::var("PATH").unwrap_or_default();
    let joined = [expand(&machine), expand(&user), current].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(";");
    std::env::set_var("PATH", joined);
}
#[cfg(not(windows))]
fn refresh_path() {}

/// Programas externos que a app aproveita se están instalados.
#[tauri::command]
pub async fn tools_status() -> serde_json::Value {
    blocking(|| {
        refresh_path();
        Ok::<_, String>(tools_now())
    })
    .await
    .unwrap_or_default()
}

/// Pide cancelar a descarga de programas que está en marcha.
static TOOL_CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[tauri::command]
pub fn cancel_tool_install() {
    TOOL_CANCEL.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Descarga e instala FFmpeg ou MKVToolNix desde a súa web oficial. A interface recibe
/// `trasno://tool-install` con `{ tool, phase, progress }` (phase: download | verify | extract).
#[tauri::command]
pub async fn install_tool(app: tauri::AppHandle, tool: String) -> Result<serde_json::Value, String> {
    use tauri::Emitter;
    use trasno_engine::tools_install::{self, Phase, Tool};
    let t = Tool::parse(&tool).ok_or_else(|| "Programa descoñecido.".to_string())?;
    TOOL_CANCEL.store(false, std::sync::atomic::Ordering::SeqCst);
    blocking(move || {
        let name = tool.clone();
        let sink = |p: Phase, f: f64| {
            let phase = match p {
                Phase::Download => "download",
                Phase::Verify => "verify",
                Phase::Extract => "extract",
            };
            let _ = app.emit("trasno://tool-install", serde_json::json!({ "tool": name, "phase": phase, "progress": f }));
        };
        tools_install::install(t, &sink, &|| TOOL_CANCEL.load(std::sync::atomic::Ordering::SeqCst)).map_err(to_err)?;
        Ok(tools_now())
    })
    .await
}

/// Quita un programa que descargou a propia app (non toca os instalados polo usuario).
#[tauri::command]
pub async fn remove_tool(tool: String) -> Result<serde_json::Value, String> {
    let t = trasno_engine::tools_install::Tool::parse(&tool).ok_or_else(|| "Programa descoñecido.".to_string())?;
    blocking(move || {
        trasno_engine::tools_install::remove(t).map_err(to_err)?;
        Ok(tools_now())
    })
    .await
}

fn tools_now() -> serde_json::Value {
    use trasno_engine::tools_install::{installed, Tool};
    let own = |t: Tool, f: &str| installed(t, f).is_some();
    serde_json::json!({
        "ffmpeg_own": own(Tool::Ffmpeg, "ffmpeg.exe"),
        "mkvmerge_own": own(Tool::MkvToolNix, "mkvmerge.exe"),
        "ffmpeg": trasno_engine::ffmpeg::ffmpeg_path().is_ok(),
        "ffprobe": trasno_engine::ffmpeg::ffprobe_path().is_ok(),
        "mkvmerge": trasno_engine::mkv::is_available(),
    })
}

#[tauri::command]
pub async fn mkv_tracks(path: String) -> Result<serde_json::Value, String> {
    blocking(move || trasno_engine::mkv::tracks(&PathBuf::from(path)).map_err(to_err)).await
}

#[tauri::command]
pub async fn mkv_mux(
    inputs: Vec<trasno_engine::mkv::MuxInput>,
    order: Vec<trasno_engine::mkv::TrackRef>,
    output: String,
    overwrite: bool,
) -> Result<String, String> {
    blocking(move || {
        trasno_engine::mkv::mux(&inputs, &order, &PathBuf::from(output), overwrite)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(to_err)
    })
    .await
}

/// Se o camiño xa existe, devolve outro libre co mesmo nome e un « (2)», « (3)»...
#[tauri::command]
pub fn unique_path(path: String) -> String {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return path;
    }
    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let dir = p.parent().map(|d| d.to_path_buf()).unwrap_or_default();
    for n in 2..10_000 {
        let cand = dir.join(format!("{stem} ({n}){ext}"));
        if !cand.exists() {
            return cand.to_string_lossy().to_string();
        }
    }
    path
}

/// Abre unha páxina web fixa (nunca un enderezo que veña da interface).
/// Abre unha ligazón http(s) no navegador. Só ese tipo de enderezos: nada de ficheiros nin programas.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    let ok = (url.starts_with("https://") || url.starts_with("http://")) && url.len() < 2000 && !url.chars().any(|c| c.is_whitespace() || c == '"' || c == '<' || c == '>' || c == '^' || c == '&' || c == '|');
    if !ok {
        return Err(msg("Esa ligazón non é segura para abrila.", serde_json::json!({})));
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", &url]).creation_flags(0x0800_0000).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open").arg(&url).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_link(id: String) -> Result<(), String> {
    let url = match id.as_str() {
        "ffmpeg" => "https://ffmpeg.org/download.html",
        "ffmpeg-license" => "https://ffmpeg.org/legal.html",
        "mkvtoolnix" => "https://mkvtoolnix.download/downloads.html",
        "mkvtoolnix-license" => "https://mkvtoolnix.download/doc/COPYING",
        "iso639-2" => "https://www.loc.gov/standards/iso639-2/php/code_list.php",
        "mkvmerge-doc" => "https://mkvtoolnix.download/doc/mkvmerge.html",
        _ => return Err(msg("Non coñezo esa ligazón.", serde_json::json!({}))),
    };
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .creation_flags(0x0800_0000)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open").arg(url).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}
