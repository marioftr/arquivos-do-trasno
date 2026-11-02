//! Wrapper para ffmpeg/ffprobe. Na fase actual búscanse os binarios no
//! PATH do sistema (ou en `TRASNO_FFMPEG_PATH` / `TRASNO_FFPROBE_PATH`);
//! nunha fase posterior empaquetaranse como "sidecars" de Tauri para non
//! depender de que o usuario os instale á parte.
//!
//! Todo o audio e vídeo (edición, mestura, recorte, montaxe, compresión,
//! cambio de formato, xuntar clips) pasa por aquí: o motor propio non
//! reimplementa códecs, delega en ffmpeg, que é o estándar de facto.

use crate::error::{EngineError, Result};
use crate::integrity::{assert_non_empty_file, ensure_distinct};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Busca un binario de ffmpeg: variable de contorno, despois ao carón do
/// executable da app (`ffmpeg.exe` ou `ffmpeg/ffmpeg.exe`, para levalo
/// portable) e por último o PATH do sistema.
fn find_tool(name: &str, env: &str) -> Result<PathBuf> {
    if let Ok(p) = std::env::var(env) {
        return Ok(PathBuf::from(p));
    }
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())) {
        let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
        for candidate in [dir.join(&file), dir.join("ffmpeg").join(&file), dir.join("bin").join(&file)] {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    // Se non está no sistema, o que descargou a propia app.
    which::which(name)
        .or_else(|_| crate::tools_install::installed(crate::tools_install::Tool::Ffmpeg, &format!("{name}{}", std::env::consts::EXE_SUFFIX)).ok_or(()))
        .map_err(|_| EngineError::FfmpegNotFound)
}

pub fn ffmpeg_path() -> Result<PathBuf> {
    find_tool("ffmpeg", "TRASNO_FFMPEG_PATH")
}

pub fn ffprobe_path() -> Result<PathBuf> {
    find_tool("ffprobe", "TRASNO_FFPROBE_PATH")
}

pub fn is_available() -> bool {
    ffmpeg_path().is_ok()
}

/// En Windows evita que salte unha xanela de consola ao lanzar ffmpeg.
fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    #[cfg(not(windows))]
    let _ = cmd;
}

static CANCELLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static CHILD_PID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Empeza un traballo novo: esquece calquera cancelación anterior.
pub fn begin_job() {
    CANCELLED.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Cancela o traballo: marca a cancelación e mata o ffmpeg que estea a correr (se hai). Devolve se había algo en marcha.
/// Os temporais bórranos cada operación ao ver o erro `Cancelled`; o ficheiro de destino nunca chega a existir.
pub fn cancel_current() -> bool {
    CANCELLED.store(true, std::sync::atomic::Ordering::SeqCst);
    let pid = CHILD_PID.load(std::sync::atomic::Ordering::SeqCst);
    if pid == 0 {
        return false;
    }
    let mut cmd;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd = Command::new("taskkill");
        cmd.args(["/PID", &pid.to_string(), "/T", "/F"]).creation_flags(0x0800_0000);
    }
    #[cfg(not(windows))]
    {
        cmd = Command::new("kill");
        cmd.args(["-9", &pid.to_string()]);
    }
    cmd.output().is_ok()
}

/// Quen recibe o avance (0 a 1) do ffmpeg que estea a traballar.
pub type ProgressSink = std::sync::Arc<dyn Fn(f64) + Send + Sync>;
static SINK: std::sync::Mutex<Option<ProgressSink>> = std::sync::Mutex::new(None);
/// (desde, ancho): para traballos de varias pasadas, cada unha ocupa un tramo da barra.
static PHASE: std::sync::Mutex<(f64, f64)> = std::sync::Mutex::new((0.0, 1.0));

pub fn set_progress_sink(sink: Option<ProgressSink>) {
    *SINK.lock().unwrap_or_else(|e| e.into_inner()) = sink;
}

pub fn set_progress_phase(offset: f64, span: f64) {
    *PHASE.lock().unwrap_or_else(|e| e.into_inner()) = (offset, span);
}

/// «00:01:30.50» → segundos.
fn parse_clock(s: &str) -> Option<f64> {
    let mut parts = s.trim().split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let sec: f64 = parts.next()?.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + sec)
}

/// Executa ffmpeg lendo o seu avance (`-progress`) para informar de canto leva feito.
fn run_ffmpeg_with_progress(bin: PathBuf, args: &[std::ffi::OsString], sink: Option<ProgressSink>) -> Result<Output> {
    if CANCELLED.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(EngineError::Cancelled);
    }
    use std::io::{BufRead, BufReader, Read};
    use std::process::Stdio;
    let mut cmd = Command::new(bin);
    hide_console(&mut cmd);
    let mut child = cmd
        .args(["-hide_banner", "-nostdin", "-y", "-nostats", "-progress", "pipe:1"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(EngineError::Io)?;
    CHILD_PID.store(child.id(), std::sync::atomic::Ordering::SeqCst);
    let duration = std::sync::Arc::new(std::sync::Mutex::new(0.0f64));
    let mut stderr = child.stderr.take().expect("stderr");
    let dur2 = duration.clone();
    let err_thread = std::thread::spawn(move || {
        let mut text = String::new();
        let mut buf = [0u8; 4096];
        while let Ok(n) = stderr.read(&mut buf) {
            if n == 0 {
                break;
            }
            text.push_str(&String::from_utf8_lossy(&buf[..n]));
            if *dur2.lock().unwrap_or_else(|e| e.into_inner()) == 0.0 {
                if let Some(i) = text.find("Duration: ") {
                    let rest = &text[i + 10..];
                    if let Some(end) = rest.find(',') {
                        if let Some(d) = parse_clock(&rest[..end]) {
                            *dur2.lock().unwrap_or_else(|e| e.into_inner()) = d;
                        }
                    }
                }
            }
        }
        text
    });
    let (offset, span) = *PHASE.lock().unwrap_or_else(|e| e.into_inner());
    let stdout = child.stdout.take().expect("stdout");
    for line in BufReader::new(stdout).lines().map_while(|l| l.ok()) {
        let Some(sink) = &sink else { continue };
        if let Some(v) = line.strip_prefix("out_time_us=") {
            let d = *duration.lock().unwrap_or_else(|e| e.into_inner());
            if let (Ok(us), true) = (v.trim().parse::<f64>(), d > 0.0) {
                sink(offset + span * (us / 1e6 / d).clamp(0.0, 1.0));
            }
        }
    }
    let status = child.wait().map_err(EngineError::Io)?;
    let stderr = err_thread.join().unwrap_or_default();
    CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);
    if CANCELLED.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(EngineError::Cancelled);
    }
    if let (true, Some(sink)) = (status.success(), &sink) {
        sink(offset + span);
    }
    Ok(Output { status, stdout: Vec::new(), stderr: stderr.into_bytes() })
}

pub(crate) fn run_ffmpeg(args: &[std::ffi::OsString]) -> Result<Output> {
    let bin = ffmpeg_path()?;
    let sink = SINK.lock().unwrap_or_else(|e| e.into_inner()).clone();
    run_ffmpeg_with_progress(bin, args, sink)
}

pub(crate) fn check_success(output: &Output) -> Result<()> {
    if !output.status.success() {
        return Err(EngineError::FfmpegFailed {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(())
}

/// Conversión xenérica de audio/vídeo dun formato a outro. `extra_args`
/// permite pasar opcións específicas (bitrate, códec, filtros de recorte...)
/// sen ter que abrir unha función nova por cada caso de uso.
pub fn convert_media(
    input: &Path,
    output_final: &Path,
    overwrite: bool,
    extra_args: &[String],
) -> Result<PathBuf> {
    convert_media_seek(input, output_final, overwrite, &[], extra_args)
}

/// Como `convert_media`, con opcións que van ANTES da entrada (p. ex. `-ss`, que así salta sen decodificar o anterior).
pub fn convert_media_seek(
    input: &Path,
    output_final: &Path,
    overwrite: bool,
    pre_args: &[String],
    extra_args: &[String],
) -> Result<PathBuf> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    ensure_distinct(&[input], output_final)?;
    if output_final.extension().is_none() {
        return Err(EngineError::IntegrityCheck(
            "O nome do ficheiro de saída non ten extensión (por exemplo «.mp4»): ffmpeg necesita saber que formato crear.".into(),
        ));
    }
    if output_final.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output_final.to_path_buf()));
    }

    let dir = output_final
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;

    // ffmpeg necesita escribir directamente nun ficheiro real (non nun
    // NamedTempFile xa aberto), así que xeramos un nome temporal propio,
    // deixamos que ffmpeg o cree, e só ao final facemos o rename atómico.
    let ext = output_final
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("out");
    let temp_path = dir.join(format!(".trasno-{}.part.{}", uuid::Uuid::new_v4(), ext));

    let mut args: Vec<std::ffi::OsString> = pre_args.iter().map(std::ffi::OsString::from).collect();
    args.push(std::ffi::OsString::from("-i"));
    args.push(input.as_os_str().to_owned());
    args.extend(extra_args.iter().map(std::ffi::OsString::from));
    args.push(temp_path.as_os_str().to_owned());

    let result = run_ffmpeg(&args);

    let output = match result {
        Ok(o) => o,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }
    };

    if let Err(e) = check_success(&output) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    if let Err(e) = assert_non_empty_file(&temp_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    // Verificación de coherencia: que ffprobe sexa quen de ler o ficheiro
    // resultante antes de darlle o visto e prace e movelo ao destino final.
    if let Ok(probe_bin) = ffprobe_path() {
        let mut probe_cmd = Command::new(probe_bin);
        hide_console(&mut probe_cmd);
        let probe = probe_cmd
            .args(["-v", "error", "-show_entries", "format=duration"])
            .arg(&temp_path)
            .output();
        if let Ok(probe) = probe {
            if !probe.status.success() {
                let _ = std::fs::remove_file(&temp_path);
                return Err(EngineError::IntegrityCheck(
                    "O ficheiro convertido non se pode ler; descarteino para non deixarche un ficheiro roto.".into(),
                ));
            }
        }
    }

    std::fs::rename(&temp_path, output_final).map_err(EngineError::Io)?;
    Ok(output_final.to_path_buf())
}

/// Recorta un intervalo [inicio, fin) dun ficheiro de audio ou vídeo sen
/// recodificar cando é posible (copia de stream), o que é case instantáneo
/// e non perde calidade.
pub fn trim_media(
    input: &Path,
    output_final: &Path,
    overwrite: bool,
    start_seconds: f64,
    end_seconds: Option<f64>,
) -> Result<PathBuf> {
    if !start_seconds.is_finite() || start_seconds < 0.0 {
        return Err(EngineError::IntegrityCheck("O inicio do corte non é válido.".into()));
    }
    if let Some(end) = end_seconds {
        if !end.is_finite() || end <= start_seconds {
            return Err(EngineError::IntegrityCheck(
                "O fin do corte ten que vir despois do inicio.".into(),
            ));
        }
    }
    let mut args = vec!["-ss".to_string(), start_seconds.to_string()];
    if let Some(end) = end_seconds {
        args.push("-to".to_string());
        args.push(end.to_string());
    }
    args.push("-c".to_string());
    args.push("copy".to_string());
    convert_media(input, output_final, overwrite, &args)
}

/// Xunta varios ficheiros de vídeo/audio do MESMO formato/códec nun só,
/// usando o demuxer "concat" de ffmpeg (sen recodificar).
pub fn concat_media(inputs: &[PathBuf], output_final: &Path, overwrite: bool) -> Result<PathBuf> {
    if inputs.is_empty() {
        return Err(EngineError::IntegrityCheck(
            "Falta indicar que ficheiros hai que xuntar.".into(),
        ));
    }
    for i in inputs {
        if !i.is_file() {
            return Err(EngineError::InputNotFound(i.clone()));
        }
        // O ficheiro de lista de concat é liña a liña: un salto de liña no
        // nome permitiría inxectar directivas alleas.
        if i.to_string_lossy().contains(['\n', '\r']) {
            return Err(EngineError::IntegrityCheck(
                "Un dos nomes de ficheiro ten un salto de liña e non é seguro usalo.".into(),
            ));
        }
    }
    let refs: Vec<&Path> = inputs.iter().map(|p| p.as_path()).collect();
    ensure_distinct(&refs, output_final)?;
    if output_final.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output_final.to_path_buf()));
    }

    let dir = output_final
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;

    let list_path = dir.join(format!(".trasno-concat-{}.txt", uuid::Uuid::new_v4()));
    let list_contents = inputs
        .iter()
        .map(|p| {
            // Rutas absolutas: as relativas resolveríanse contra o directorio
            // da lista, non contra o do usuario.
            let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone());
            format!("file '{}'", abs.to_string_lossy().replace('\'', "'\\''"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&list_path, list_contents)?;

    let ext = output_final
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("out");
    let temp_path = dir.join(format!(".trasno-{}.part.{}", uuid::Uuid::new_v4(), ext));

    let args: Vec<std::ffi::OsString> = vec![
        "-f".into(),
        "concat".into(),
        "-safe".into(),
        "0".into(),
        "-i".into(),
        list_path.as_os_str().to_owned(),
        "-c".into(),
        "copy".into(),
        temp_path.as_os_str().to_owned(),
    ];

    let run_result = run_ffmpeg(&args);
    let _ = std::fs::remove_file(&list_path);

    let output = match run_result {
        Ok(o) => o,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }
    };
    if let Err(e) = check_success(&output) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }
    if let Err(e) = assert_non_empty_file(&temp_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    std::fs::rename(&temp_path, output_final).map_err(EngineError::Io)?;
    Ok(output_final.to_path_buf())
}

/// Operacións sinxelas de edición sobre audio/vídeo, todas cara un ficheiro
/// novo (nunca se toca o orixinal).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaOp {
    /// Garda un só fotograma (imaxe) no segundo indicado.
    Frame(f64),
    /// Factor de velocidade (0.25 a 4.0); 2.0 é o dobre de rápido.
    Speed(f64),
    /// Quita a pista de audio dun vídeo sen recodificar a imaxe.
    Mute,
    /// Multiplicador de volume (0.0 a 4.0).
    Volume(f64),
    /// Cambia a altura do vídeo (en píxeles, par) mantendo a proporción.
    ScaleHeight(u32),
    /// Efectos de audio (só para ficheiros de son).
    /// Iguala o volume percibido (EBU R128).
    Normalize,
    /// Dálle a volta ao son.
    Reverse,
    /// Aparece aos poucos durante N segundos.
    FadeIn(f64),
    /// Esvaece durante os últimos N segundos.
    FadeOut(f64),
    /// Eco; o valor é o atraso en milisegundos.
    Echo(f64),
    /// Graves en dB (-12 a 12).
    Bass(f64),
    /// Agudos en dB (-12 a 12).
    Treble(f64),
    /// Reduce o ruído de fondo.
    Denoise,
}

pub fn media_edit(input: &Path, output: &Path, overwrite: bool, op: MediaOp, has_video: bool) -> Result<PathBuf> {
    let bad = |m: &str| Err(EngineError::IntegrityCheck(m.into()));
    let mut args: Vec<String> = Vec::new();
    match op {
        MediaOp::Frame(at) => {
            if !at.is_finite() || at < 0.0 {
                return bad("O instante do fotograma non é válido.");
            }
            args.extend(["-ss".into(), at.to_string(), "-frames:v".into(), "1".into()]);
        }
        MediaOp::Speed(f) => {
            if !f.is_finite() || !(0.25..=4.0).contains(&f) {
                return bad("A velocidade ten que estar entre 0,25 e 4.");
            }
            if has_video {
                args.extend(["-filter:v".into(), format!("setpts=PTS/{f}")]);
            }
            // atempo só admite 0,5 a 2: encadéase para chegar ao resto.
            let mut chain = Vec::new();
            let mut rest = f;
            while rest > 2.0 {
                chain.push("atempo=2.0".to_string());
                rest /= 2.0;
            }
            while rest < 0.5 {
                chain.push("atempo=0.5".to_string());
                rest /= 0.5;
            }
            chain.push(format!("atempo={rest}"));
            args.extend(["-filter:a".into(), chain.join(",")]);
        }
        MediaOp::Mute => args.extend(["-an".into(), "-c:v".into(), "copy".into()]),
        MediaOp::Volume(v) => {
            if !v.is_finite() || !(0.0..=4.0).contains(&v) {
                return bad("O volume ten que estar entre 0 e 4.");
            }
            args.extend(["-af".into(), format!("volume={v}")]);
            if has_video {
                args.extend(["-c:v".into(), "copy".into()]);
            }
        }
        MediaOp::ScaleHeight(h) => {
            if !(16..=8640).contains(&h) {
                return bad("A altura non é válida.");
            }
            args.extend(["-vf".into(), format!("scale=-2:{}", h - h % 2)]);
        }
        MediaOp::Normalize => args.extend(["-af".into(), "loudnorm=I=-16:TP=-1.5:LRA=11".into()]),
        MediaOp::Reverse => args.extend(["-af".into(), "areverse".into()]),
        MediaOp::FadeIn(d) => {
            if !d.is_finite() || !(0.05..=600.0).contains(&d) {
                return bad("A duración do efecto non é válida.");
            }
            args.extend(["-af".into(), format!("afade=t=in:st=0:d={d}")]);
        }
        MediaOp::FadeOut(d) => {
            if !d.is_finite() || !(0.05..=600.0).contains(&d) {
                return bad("A duración do efecto non é válida.");
            }
            let total = probe(input)?["format"]["duration"].as_str().and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0);
            args.extend(["-af".into(), format!("afade=t=out:st={}:d={d}", (total - d).max(0.0))]);
        }
        MediaOp::Echo(ms) => {
            if !ms.is_finite() || !(20.0..=2000.0).contains(&ms) {
                return bad("O atraso do eco non é válido.");
            }
            args.extend(["-af".into(), format!("aecho=0.8:0.6:{ms}:0.4")]);
        }
        MediaOp::Bass(db) | MediaOp::Treble(db) => {
            if !db.is_finite() || !(-12.0..=12.0).contains(&db) {
                return bad("Os decibelios teñen que estar entre -12 e 12.");
            }
            let f = if matches!(op, MediaOp::Bass(_)) { "bass" } else { "treble" };
            args.extend(["-af".into(), format!("{f}=g={db}")]);
        }
        MediaOp::Denoise => args.extend(["-af".into(), "afftdn=nr=12".into()]),
    }
    convert_media(input, output, overwrite, &args)
}

/// Información de ffprobe (formato e pistas) como JSON.
pub fn probe(path: &Path) -> Result<serde_json::Value> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    let mut cmd = Command::new(ffprobe_path()?);
    hide_console(&mut cmd);
    let out = cmd
        .args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams", "-show_chapters"])
        .arg(path)
        .output()
        .map_err(EngineError::Io)?;
    check_success(&out)?;
    serde_json::from_slice(&out.stdout).map_err(|e| EngineError::IntegrityCheck(e.to_string()))
}
