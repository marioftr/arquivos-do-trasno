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
/// Procesos ffmpeg en marcha dos traballos do usuario (un lote pode ter varios á vez): a pausa e o cancelar aplícanse a todos.
static CHILDREN: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());
fn children() -> Vec<u32> {
    CHILDREN.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

thread_local! {
    static BACKGROUND: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Executa `f` como traballo de fondo (vistas previas, lectura de formatos...): non lle afectan as cancelacións dos traballos
/// do usuario (nin unha cancelación vella que quedase marcada), non se pode matar con «Cancelar» e non informa do avance.
pub fn background<T>(f: impl FnOnce() -> T) -> T {
    let before = BACKGROUND.with(|b| b.replace(true));
    let out = f();
    BACKGROUND.with(|b| b.set(before));
    out
}

fn is_background() -> bool {
    BACKGROUND.with(|b| b.get())
}

static PAUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Empeza un traballo novo: esquece calquera cancelación anterior (e deixa seguir un ffmpeg que quedase en pausa).
pub fn begin_job() {
    if PAUSED.load(std::sync::atomic::Ordering::SeqCst) {
        set_paused(false);
    }
    CANCELLED.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Pon en pausa (ou retoma) o ffmpeg que estea a correr. Devolve `true` se había un ffmpeg ao que facerllo.
/// En Windows suspende o proceso; noutros sistemas envía SIGSTOP / SIGCONT.
pub fn set_paused(paused: bool) -> bool {
    let pids = children();
    if pids.is_empty() {
        PAUSED.store(false, std::sync::atomic::Ordering::SeqCst);
        return false;
    }
    let ok = pids.into_iter().fold(false, |any, pid| suspend_process(pid, paused) || any);
    if ok {
        PAUSED.store(paused, std::sync::atomic::Ordering::SeqCst);
    }
    ok
}

#[cfg(windows)]
fn suspend_process(pid: u32, suspend: bool) -> bool {
    use std::ffi::c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    #[link(name = "ntdll")]
    extern "system" {
        fn NtSuspendProcess(handle: *mut c_void) -> i32;
        fn NtResumeProcess(handle: *mut c_void) -> i32;
    }
    const PROCESS_SUSPEND_RESUME: u32 = 0x0800;
    // SAFETY: chamadas simples á API de Windows con un identificador que se abre e se pecha aquí mesmo.
    unsafe {
        let h = OpenProcess(PROCESS_SUSPEND_RESUME, 0, pid);
        if h.is_null() {
            return false;
        }
        let status = if suspend { NtSuspendProcess(h) } else { NtResumeProcess(h) };
        CloseHandle(h);
        status >= 0
    }
}

#[cfg(not(windows))]
fn suspend_process(pid: u32, suspend: bool) -> bool {
    Command::new("kill").args([if suspend { "-STOP" } else { "-CONT" }, &pid.to_string()]).output().map(|o| o.status.success()).unwrap_or(false)
}

/// Cancela o traballo: marca a cancelación e mata o ffmpeg que estea a correr (se hai). Devolve se había algo en marcha.
/// Os temporais bórranos cada operación ao ver o erro `Cancelled`; o ficheiro de destino nunca chega a existir.
pub fn cancel_current() -> bool {
    CANCELLED.store(true, std::sync::atomic::Ordering::SeqCst);
    // Un proceso suspendido tamén se mata, pero deixámolo seguir antes para que remate limpo.
    let pids = children();
    if PAUSED.swap(false, std::sync::atomic::Ordering::SeqCst) {
        for pid in &pids {
            suspend_process(*pid, false);
        }
    }
    if pids.is_empty() {
        return false;
    }
    let mut any = false;
    for pid in pids {
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
        any |= cmd.output().is_ok();
    }
    any
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
    let bg = is_background();
    if !bg && CANCELLED.load(std::sync::atomic::Ordering::SeqCst) {
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
    let child_id = child.id();
    if !bg {
        CHILDREN.lock().unwrap_or_else(|e| e.into_inner()).push(child_id);
        // Se se pausou o traballo mentres un lote arranca outro ficheiro, este nace en pausa.
        if PAUSED.load(std::sync::atomic::Ordering::SeqCst) {
            suspend_process(child_id, true);
        }
    }
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
    if !bg {
        let id = child_id;
        CHILDREN.lock().unwrap_or_else(|e| e.into_inner()).retain(|p| *p != id);
    }
    if !bg && CANCELLED.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(EngineError::Cancelled);
    }
    if let (true, Some(sink)) = (status.success(), &sink) {
        sink(offset + span);
    }
    Ok(Output { status, stdout: Vec::new(), stderr: stderr.into_bytes() })
}

/// Resultado dun programa externo lanzado con `run_external`.
pub(crate) struct External {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Executa un programa externo (LibreOffice, Microsoft Office…) como un traballo máis: Pausa e Cancelar chéganlle (a el e aos
/// procesos que declare coa liña `PID <n>` na súa saída) e acaba matado se pasa de `timeout`.
pub(crate) fn run_external(mut cmd: Command, timeout: std::time::Duration) -> Result<External> {
    use std::io::{BufRead, BufReader, Read};
    use std::process::Stdio;
    let bg = is_background();
    if !bg && CANCELLED.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(EngineError::Cancelled);
    }
    hide_console(&mut cmd);
    let mut child = cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(EngineError::Io)?;
    let id = child.id();
    let tracked = std::sync::Arc::new(std::sync::Mutex::new(vec![id]));
    if !bg {
        CHILDREN.lock().unwrap_or_else(|e| e.into_inner()).push(id);
        if PAUSED.load(std::sync::atomic::Ordering::SeqCst) {
            suspend_process(id, true);
        }
    }
    let stdout = child.stdout.take().expect("stdout");
    let mut stderr = child.stderr.take().expect("stderr");
    let t2 = tracked.clone();
    let out_thread = std::thread::spawn(move || {
        let mut text = String::new();
        for line in BufReader::new(stdout).lines().map_while(|l| l.ok()) {
            if let Some(n) = line.strip_prefix("PID ").and_then(|v| v.trim().parse::<u32>().ok()).filter(|n| *n > 0) {
                t2.lock().unwrap_or_else(|e| e.into_inner()).push(n);
                if !bg {
                    CHILDREN.lock().unwrap_or_else(|e| e.into_inner()).push(n);
                    if PAUSED.load(std::sync::atomic::Ordering::SeqCst) {
                        suspend_process(n, true);
                    }
                }
                continue;
            }
            text.push_str(&line);
            text.push('\n');
        }
        text
    });
    let err_thread = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        text
    });
    let started = std::time::Instant::now();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait().map_err(EngineError::Io)? {
            Some(st) => break st,
            None => {
                if started.elapsed() > timeout {
                    timed_out = true;
                    for pid in tracked.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                        suspend_process(pid, false);
                        let mut k;
                        #[cfg(windows)]
                        {
                            use std::os::windows::process::CommandExt;
                            k = Command::new("taskkill");
                            k.args(["/PID", &pid.to_string(), "/T", "/F"]).creation_flags(0x0800_0000);
                        }
                        #[cfg(not(windows))]
                        {
                            k = Command::new("kill");
                            k.args(["-9", &pid.to_string()]);
                        }
                        let _ = k.output();
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    };
    let stdout = out_thread.join().unwrap_or_default();
    let stderr = err_thread.join().unwrap_or_default();
    // Os procesos que declarou o programa (p. ex. o Word que manexou) non deben quedar vivos cando el remata.
    for pid in tracked.lock().unwrap_or_else(|e| e.into_inner()).iter().skip(1).copied().collect::<Vec<_>>() {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).creation_flags(0x0800_0000).output();
        }
        #[cfg(not(windows))]
        {
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
        }
    }
    if !bg {
        let all = tracked.lock().unwrap_or_else(|e| e.into_inner()).clone();
        CHILDREN.lock().unwrap_or_else(|e| e.into_inner()).retain(|p| !all.contains(p));
    }
    if !bg && CANCELLED.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(EngineError::Cancelled);
    }
    if timed_out {
        return Err(EngineError::OfficeFailed("tardou demasiado e o trasno parouno".into()));
    }
    Ok(External { code: status.code(), stdout, stderr })
}

pub(crate) fn run_ffmpeg(args: &[std::ffi::OsString]) -> Result<Output> {
    let bin = ffmpeg_path()?;
    let sink = if is_background() { None } else { SINK.lock().unwrap_or_else(|e| e.into_inner()).clone() };
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
    convert_media_inputs(&[input], output_final, overwrite, pre_args, extra_args)
}

/// Como `convert_media_seek`, con varias entradas (a primeira é a principal; as demais, p. ex. unha capa para superpoñer).
pub fn convert_media_inputs(
    inputs: &[&Path],
    output_final: &Path,
    overwrite: bool,
    pre_args: &[String],
    extra_args: &[String],
) -> Result<PathBuf> {
    for input in inputs {
        if !input.is_file() {
            return Err(EngineError::InputNotFound(input.to_path_buf()));
        }
    }
    ensure_distinct(inputs, output_final)?;
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
    for input in inputs {
        args.push(std::ffi::OsString::from("-i"));
        args.push(input.as_os_str().to_owned());
    }
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
        MediaOp::Normalize => return normalize_loudness(input, output, overwrite, Loudness::default(), has_video).map(|r| r.output),
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
    // Calquera filtro de son obriga a recodificar: que saia coa calidade do orixinal, non coa que FFmpeg pon por defecto.
    if args.iter().any(|a| a == "-af" || a == "-filter:a") {
        args.extend(keep_audio_args(input, output));
    }
    convert_media(input, output, overwrite, &args)
}

/// Opcións para que o son recodificado conserve o que tiña o orixinal: frecuencia de mostraxe, bitrate (formatos con perdas) e
/// profundidade de bits (WAV, AIFF) ou códec sen perdas (ALAC). Sen isto FFmpeg usa os seus valores por defecto: un MP3 de
/// 320 kbps sae a 128, e un WAV de 24 bits a 16.
pub(crate) fn keep_audio_args(input: &Path, output: &Path) -> Vec<String> {
    let mut args = Vec::new();
    let Ok(info) = probe(input) else { return args };
    let Some(a) = info["streams"].as_array().and_then(|l| l.iter().find(|s| s["codec_type"] == "audio")) else { return args };
    let in_ext = crate::formats::extension_of(input).unwrap_or_default();
    let ext = crate::formats::extension_of(output).unwrap_or_default();
    let codec = a["codec_name"].as_str().unwrap_or("");
    // Só se o contedor é o mesmo se sabe que admite o que tiña o orixinal.
    if in_ext != ext {
        return args;
    }
    // Opus sempre traballa a 48 kHz.
    if !matches!(ext.as_str(), "opus" | "webm") && codec != "opus" {
        if let Some(rate) = a["sample_rate"].as_str().and_then(|r| r.parse::<u32>().ok()).filter(|r| (8000..=384_000).contains(r)) {
            args.extend(["-ar".to_string(), rate.to_string()]);
        }
    }
    if codec.starts_with("pcm_") && matches!(ext.as_str(), "wav" | "aiff" | "aif") {
        args.extend(["-c:a".to_string(), codec.to_string()]);
    } else if codec == "alac" && matches!(ext.as_str(), "m4a" | "mp4" | "mov") {
        args.extend(["-c:a".to_string(), "alac".to_string()]);
    } else if let Some(kbps) = a["bit_rate"].as_str().and_then(|b| b.parse::<u64>().ok()).map(|b| ((b + 500) / 1000) as u32).filter(|k| *k >= 8) {
        match codec {
            "vorbis" => args.extend(["-c:a".to_string(), "libvorbis".to_string(), "-q:a".to_string(), vorbis_quality(kbps).to_string()]),
            "mp3" | "aac" | "opus" | "wmav2" | "wmav1" | "ac3" | "eac3" | "mp2" => {
                let top = if codec == "opus" { 510 } else if codec == "mp3" { 320 } else { 640 };
                args.extend(["-b:a".to_string(), format!("{}k", kbps.clamp(16, top))]);
            }
            _ => {}
        }
    }
    args
}

/// libvorbis non acepta calquera bitrate fixo (a 32 kbps estéreo ou a 320 kbps en mono de 8 kHz nin arranca): tradúcese ao
/// nivel de calidade equivalente, que sempre vale.
pub(crate) fn vorbis_quality(kbps: u32) -> &'static str {
    const Q: [(u32, &str); 11] = [(45, "-1"), (64, "0"), (80, "1"), (96, "2"), (112, "3"), (128, "4"), (160, "5"), (192, "6"), (224, "7"), (256, "8"), (320, "9")];
    Q.iter().rev().find(|(k, _)| kbps >= *k).map(|(_, q)| *q).unwrap_or("-1")
}

/// Obxectivo dunha normalización de volume (EBU R128).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Loudness {
    /// Volume percibido obxectivo, en LUFS (−70 a −5). Streaming: −14; podcast: −16; emisión: −23.
    pub lufs: f64,
    /// Pico máximo permitido, en dBTP (−9 a 0).
    pub peak: f64,
    /// Rango dinámico máximo, en LU (1 a 50). Se o orixinal ten máis, comprímese.
    pub range: f64,
}

impl Default for Loudness {
    fn default() -> Self {
        Loudness { lufs: -16.0, peak: -1.5, range: 11.0 }
    }
}

/// O que se mediu nun ficheiro de son.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct LoudnessMeasure {
    pub lufs: f64,
    pub peak: f64,
    pub range: f64,
    pub thresh: f64,
    pub offset: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LoudnessReport {
    pub output: PathBuf,
    /// Como estaba o orixinal.
    pub before: LoudnessMeasure,
    /// `true` se abondou con subir ou baixar o volume por igual; `false` se houbo que comprimir a dinámica para chegar ao obxectivo.
    pub linear: bool,
}

fn check_loudness(t: &Loudness) -> Result<()> {
    let ok = t.lufs.is_finite() && t.peak.is_finite() && t.range.is_finite() && (-70.0..=-5.0).contains(&t.lufs) && (-9.0..=0.0).contains(&t.peak) && (1.0..=50.0).contains(&t.range);
    if ok {
        Ok(())
    } else {
        Err(EngineError::IntegrityCheck("Os valores de normalización non son válidos: o volume vai de −70 a −5 LUFS, o pico de −9 a 0 dBTP e o rango de 1 a 50 LU.".into()))
    }
}

/// Primeira pasada: mide o volume do ficheiro enteiro sen escribir nada.
pub fn measure_loudness(input: &Path, target: Loudness) -> Result<LoudnessMeasure> {
    check_loudness(&target)?;
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    let sink = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let args: Vec<std::ffi::OsString> = vec![
        "-i".into(),
        input.as_os_str().to_owned(),
        "-map".into(),
        "0:a:0".into(),
        "-af".into(),
        format!("loudnorm=I={}:TP={}:LRA={}:print_format=json", target.lufs, target.peak, target.range).into(),
        "-f".into(),
        "null".into(),
        sink.into(),
    ];
    let out = run_ffmpeg(&args)?;
    check_success(&out)?;
    let text = String::from_utf8_lossy(&out.stderr);
    // O filtro escribe un bloque JSON ao final da súa saída.
    let block = text.rfind('{').zip(text.rfind('}')).filter(|(a, b)| a < b).map(|(a, b)| &text[a..=b]);
    let json: serde_json::Value = block.and_then(|b| serde_json::from_str(b).ok()).ok_or_else(|| EngineError::IntegrityCheck("Non se puido medir o volume do ficheiro.".into()))?;
    let num = |k: &str| json[k].as_str().and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| v.is_finite());
    match (num("input_i"), num("input_tp"), num("input_lra"), num("input_thresh"), num("target_offset")) {
        (Some(lufs), Some(peak), Some(range), Some(thresh), Some(offset)) => Ok(LoudnessMeasure { lufs, peak, range, thresh, offset }),
        // Un ficheiro en silencio mide «-inf»: non hai nada que igualar.
        _ => Err(EngineError::IntegrityCheck("O ficheiro está en silencio (ou case): non hai volume que igualar.".into())),
    }
}

/// Iguala o volume percibido en dúas pasadas (EBU R128): a primeira mide o ficheiro enteiro e a segunda aplica a corrección
/// exacta, a mesma para todo o ficheiro. Nunha soa pasada o filtro vai corrixindo segundo le e esmaga a dinámica (o baixo e o alto
/// quedan case iguais).
/// Se o ficheiro leva vídeo, a imaxe cópiase sen tocar.
pub fn normalize_loudness(input: &Path, output: &Path, overwrite: bool, target: Loudness, has_video: bool) -> Result<LoudnessReport> {
    check_loudness(&target)?;
    ensure_distinct(&[input], output)?;
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    set_progress_phase(0.0, 0.4);
    let measured = measure_loudness(input, target);
    set_progress_phase(0.4, 0.6);
    let result = measured.and_then(|m| {
        let filter = format!(
            "loudnorm=I={}:TP={}:LRA={}:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true",
            target.lufs, target.peak, target.range, m.lufs, m.peak, m.range, m.thresh, m.offset
        );
        let mut args: Vec<String> = vec!["-af".into(), filter];
        if has_video {
            args.extend(["-c:v".into(), "copy".into()]);
        }
        args.extend(keep_audio_args(input, output));
        // O filtro traballa por dentro a 192 kHz: sen fixar a frecuencia, un contedor que a admita sairía a 192 kHz.
        if !args.iter().any(|a| a == "-ar") {
            args.extend(["-ar".into(), "48000".into()]);
        }
        let out = convert_media(input, output, overwrite, &args)?;
        // Abonda con cambiar o volume por igual se o rango xa cabe e o pico non pasa do tope ao aplicar a ganancia.
        let linear = m.range <= target.range && m.peak + (target.lufs - m.lufs) <= target.peak;
        Ok(LoudnessReport { output: out, before: m, linear })
    });
    set_progress_phase(0.0, 1.0);
    result
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

/// Como `probe`, pero tamén cos grupos de fluxos (teselas dunha cuadrícula HEIF/AVIF).
pub fn probe_with_groups(path: &Path) -> Result<serde_json::Value> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    let mut cmd = Command::new(ffprobe_path()?);
    hide_console(&mut cmd);
    let out = cmd
        .args(["-v", "error", "-print_format", "json", "-show_streams", "-show_stream_groups"])
        .arg(path)
        .output()
        .map_err(EngineError::Io)?;
    check_success(&out)?;
    serde_json::from_slice(&out.stdout).map_err(|e| EngineError::IntegrityCheck(e.to_string()))
}
