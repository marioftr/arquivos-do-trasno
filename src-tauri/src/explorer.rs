//! Integración co menú de clic dereito do Explorador de Windows («Converter a…»,
//! «Comprimir», «Xirar»… sobre un ficheiro). Escríbese só en `HKEY_CURRENT_USER`
//! (non fai falta ser administrador) e pódese quitar enteira desde Axustes.
//!
//! Cada entrada lanza esta mesma aplicación cun traballo na liña de ordes:
//!   `trasno.exe --convert <formato> "<ficheiro>"`
//!   `trasno.exe --compress <75|50|25> "<ficheiro>"`
//!   `trasno.exe --resize <p75|p50|p25|h1080|h720|h480> "<ficheiro>"`
//!   `trasno.exe --rotate <90|180|270> "<ficheiro>"`
//! A interface mostra unha xanela pequena co progreso (ver `QuickJob.svelte`).
//! «Abrir» (`trasno.exe "<ficheiro>"`) non abre outra aplicación: se xa hai unha aberta,
//! os ficheiros entran nela.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

const ROOT: &str = r"Software\Classes\SystemFileAssociations";
const KEY: &str = "ArquivosDoTrasno";

// ---------------------------------------------------------------- menú

enum Item {
    Cmd { id: String, label: String, args: String },
    Sub { id: String, label: String, items: Vec<Item> },
}

/// Textos do menú, xa traducidos pola interface. Se falta algún úsase a súa chave.
pub struct Labels(HashMap<String, String>);

impl Labels {
    fn get(&self, k: &str) -> String {
        self.0.get(k).cloned().unwrap_or_else(|| k.to_string())
    }
    fn fill(&self, k: &str, vars: &[(&str, &str)]) -> String {
        let mut s = self.get(k);
        for (name, v) in vars {
            s = s.replace(&format!("{{{name}}}"), v);
        }
        s
    }
}

/// Dous formatos son «o mesmo» se un é alias do outro (jpg/jpeg, tif/tiff).
fn same_format(a: &str, b: &str) -> bool {
    fn norm(x: &str) -> &str {
        match x {
            "jpeg" => "jpg",
            "tif" => "tiff",
            "mpeg" => "mpg",
            other => other,
        }
    }
    norm(a) == norm(b)
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Image,
    Audio,
    Video,
    Text,
    Pdf,
}

fn sources() -> Vec<(&'static str, Kind)> {
    let mut v = Vec::new();
    for e in ["png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "tif", "webp", "tga", "qoi"] {
        v.push((e, Kind::Image));
    }
    for e in ["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "wma", "aiff"] {
        v.push((e, Kind::Audio));
    }
    for e in ["mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "mpg", "mpeg"] {
        v.push((e, Kind::Video));
    }
    for e in ["txt", "md", "csv", "json", "xml", "log"] {
        v.push((e, Kind::Text));
    }
    v.push(("pdf", Kind::Pdf));
    v
}

fn menu_for(ext: &str, kind: Kind, l: &Labels) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    let mut n = 0;
    let mut next = |slug: &str| {
        n += 1;
        format!("{n:02}_{slug}")
    };
    let convert = |targets: &[&str]| -> Vec<(String, String)> {
        targets
            .iter()
            .filter(|t| !same_format(t, ext))
            .map(|t| (l.fill("convert", &[("ext", &t.to_uppercase())]), format!("--convert {t}")))
            .collect()
    };
    let quality = |slug: &str| Item::Sub {
        id: slug.to_string(),
        label: l.get("compress"),
        items: [75, 50, 25]
            .iter()
            .map(|p| Item::Cmd { id: format!("q{p}"), label: l.fill("quality", &[("p", &p.to_string())]), args: format!("--compress {p}") })
            .collect(),
    };
    let rotate = |slug: &str| Item::Sub {
        id: slug.to_string(),
        label: l.get("rotate"),
        items: vec![
            Item::Cmd { id: "r1_left".into(), label: l.get("left"), args: "--rotate 270".into() },
            Item::Cmd { id: "r2_right".into(), label: l.get("right"), args: "--rotate 90".into() },
            Item::Cmd { id: "r3_half".into(), label: l.get("half"), args: "--rotate 180".into() },
        ],
    };
    let mut push_cmds = |items: &mut Vec<Item>, list: Vec<(String, String)>| {
        for (label, args) in list {
            let slug = args.replace("--convert ", "");
            items.push(Item::Cmd { id: next(&slug), label, args });
        }
    };
    match kind {
        Kind::Image => {
            push_cmds(&mut items, convert(&["png", "jpg", "webp", "pdf"]));
            let more = convert(&["gif", "bmp", "tiff", "ico"]);
            if !more.is_empty() {
                let sub: Vec<Item> = more.into_iter().enumerate().map(|(i, (label, args))| Item::Cmd { id: format!("m{i}"), label, args }).collect();
                items.push(Item::Sub { id: next("more"), label: l.get("more"), items: sub });
            }
            items.push(Item::Sub {
                id: next("resize"),
                label: l.get("resize"),
                items: [75, 50, 25]
                    .iter()
                    .map(|p| Item::Cmd { id: format!("s{p}"), label: l.fill("percent", &[("p", &p.to_string())]), args: format!("--resize p{p}") })
                    .collect(),
            });
            items.push(rotate(&next("rotate")));
            items.push(quality(&next("compress")));
        }
        Kind::Video => {
            push_cmds(&mut items, convert(&["mp4", "mkv", "webm", "mp3"]));
            let more = convert(&["mov", "avi", "gif", "wav"]);
            let sub: Vec<Item> = more.into_iter().enumerate().map(|(i, (label, args))| Item::Cmd { id: format!("m{i}"), label, args }).collect();
            items.push(Item::Sub { id: next("more"), label: l.get("more"), items: sub });
            items.push(Item::Sub {
                id: next("scale"),
                label: l.get("scale"),
                items: [1080, 720, 480]
                    .iter()
                    .map(|h| Item::Cmd { id: format!("h{h}"), label: format!("{h}p"), args: format!("--resize h{h}") })
                    .collect(),
            });
            items.push(quality(&next("compress")));
        }
        Kind::Audio => {
            push_cmds(&mut items, convert(&["mp3", "wav", "flac", "ogg"]));
            let more = convert(&["m4a", "aac", "opus"]);
            let sub: Vec<Item> = more.into_iter().enumerate().map(|(i, (label, args))| Item::Cmd { id: format!("m{i}"), label, args }).collect();
            items.push(Item::Sub { id: next("more"), label: l.get("more"), items: sub });
        }
        Kind::Text => push_cmds(&mut items, convert(&["pdf"])),
        Kind::Pdf => {
            push_cmds(&mut items, convert(&["txt"]));
            items.push(rotate(&next("rotate")));
            items.push(quality(&next("compress")));
        }
    }
    items
}

// ---------------------------------------------------------------- .reg

fn reg_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn emit(reg: &mut String, base: &str, exe: &str, items: &[Item]) {
    for it in items {
        match it {
            Item::Cmd { id, label, args } => {
                reg.push_str(&format!(
                    "[{base}\\shell\\{id}]\r\n\"MUIVerb\"=\"{}\"\r\n\r\n[{base}\\shell\\{id}\\command]\r\n@=\"\\\"{exe}\\\" {args} \\\"%1\\\"\"\r\n\r\n",
                    reg_str(label)
                ));
            }
            Item::Sub { id, label, items } => {
                let key = format!("{base}\\shell\\{id}");
                reg.push_str(&format!("[{key}]\r\n\"MUIVerb\"=\"{}\"\r\n\"SubCommands\"=\"\"\r\n\r\n", reg_str(label)));
                emit(reg, &key, exe, items);
            }
        }
    }
}

fn build_install_reg(exe: &str, labels: &Labels) -> String {
    let q = reg_str(exe);
    let mut reg = String::from("Windows Registry Editor Version 5.00\r\n\r\n");
    for (ext, kind) in sources() {
        let base = format!("HKEY_CURRENT_USER\\{ROOT}\\.{ext}\\shell\\{KEY}");
        reg.push_str(&format!(
            "[{base}]\r\n\"MUIVerb\"=\"{}\"\r\n\"SubCommands\"=\"\"\r\n\"Icon\"=\"\\\"{q}\\\",0\"\r\n\r\n",
            reg_str(&labels.get("title"))
        ));
        let mut items = vec![Item::Cmd { id: "00_open".into(), label: labels.get("open"), args: String::new() }];
        items.extend(menu_for(ext, kind, labels));
        // «Abrir» non leva argumento extra: `"exe" "%1"`.
        emit(&mut reg, &base, &q, &items);
    }
    reg
}

fn to_reg_file(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFE];
    for u in text.encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    bytes
}

fn run_reg(args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = std::process::Command::new("reg");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd.args(args).output()
}

fn import_reg(text: &str) -> Result<(), String> {
    let path = std::env::temp_dir().join(format!(
        "trasno-{}-{}.reg",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
    ));
    std::fs::write(&path, to_reg_file(text)).map_err(|e| e.to_string())?;
    let out = run_reg(&["import", &path.to_string_lossy()]);
    let _ = std::fs::remove_file(&path);
    let out = out.map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(windows)]
fn exe() -> Result<String, String> {
    std::env::current_exe().map(|p| p.to_string_lossy().to_string()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn explorer_install(labels: HashMap<String, String>) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        let _ = labels;
        Err("Esta función só existe en Windows.".into())
    }
    #[cfg(windows)]
    {
        // Primeiro quítase o que houbese (versións anteriores do menú) para non deixar entradas soltas.
        explorer_uninstall()?;
        import_reg(&build_install_reg(&exe()?, &Labels(labels)))
    }
}

#[tauri::command]
pub fn explorer_uninstall() -> Result<(), String> {
    #[cfg(not(windows))]
    {
        Err("Esta función só existe en Windows.".into())
    }
    #[cfg(windows)]
    {
        let mut reg = String::from("Windows Registry Editor Version 5.00\r\n\r\n");
        for (ext, _) in sources() {
            reg += &format!("[-HKEY_CURRENT_USER\\{ROOT}\\.{ext}\\shell\\{KEY}]\r\n\r\n");
        }
        import_reg(&reg)
    }
}

/// `{ installed, current }`: se hai entradas e se apuntan a esta copia da aplicación.
#[tauri::command]
pub fn explorer_status() -> serde_json::Value {
    #[cfg(not(windows))]
    {
        serde_json::json!({ "available": false, "installed": false, "current": false })
    }
    #[cfg(windows)]
    {
        let key = format!("HKCU\\{ROOT}\\.png\\shell\\{KEY}\\shell\\00_open\\command");
        let out = run_reg(&["query", &key, "/ve"]).ok();
        let text = out.as_ref().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).to_string());
        let installed = text.is_some();
        let current = match (&text, exe()) {
            (Some(t), Ok(e)) => t.to_lowercase().contains(&e.to_lowercase()),
            _ => false,
        };
        serde_json::json!({ "available": true, "installed": installed, "current": current })
    }
}

// ---------------------------------------------------------------- traballos rápidos

/// Traballo pedido desde o menú de clic dereito.
#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq)]
struct Job {
    action: String,
    target: Option<String>,
    paths: Vec<String>,
}

const ACTIONS: [&str; 4] = ["--convert", "--compress", "--resize", "--rotate"];

fn parse_job(args: &[String]) -> Option<Job> {
    let action = args.first()?.as_str();
    if !ACTIONS.contains(&action) || args.len() < 3 {
        return None;
    }
    let arg = args[1].to_lowercase();
    // O argumento (formato, nivel, tamaño ou ángulo) é só letras e números: nada que se poida confundir cunha ruta.
    if arg.is_empty() || !arg.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let paths: Vec<String> = args[2..].iter().filter(|p| PathBuf::from(p).is_file()).cloned().collect();
    (!paths.is_empty()).then(|| Job { action: action.trim_start_matches("--").to_string(), target: Some(arg), paths })
}

fn is_job_launch() -> bool {
    matches!(std::env::args().nth(1).as_deref(), Some(a) if ACTIONS.contains(&a))
}

// Cando se escollen varios ficheiros, o Explorador lanza un proceso por cada un. Aquí xúntanse
// todos nunha soa xanela: cada proceso deixa o seu traballo nunha cola e só o primeiro (o
// «líder», o que consegue crear o ficheiro de bloqueo) segue adiante; os demais saen sen abrir nada.
static LEADER: AtomicBool = AtomicBool::new(false);
static RESULT: OnceLock<Option<Job>> = OnceLock::new();
#[cfg(windows)]
static MAIN_LOCK: OnceLock<std::fs::File> = OnceLock::new();

fn queue_dir() -> PathBuf {
    std::env::temp_dir().join("trasno-jobs")
}

fn open_dir() -> PathBuf {
    std::env::temp_dir().join("trasno-open")
}

fn nanos() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
}

/// Chámase ao principio da aplicación. Se é un traballo do clic dereito e xa hai outro proceso
/// a encargarse, deixa o seu ficheiro na cola e remata. Se é unha apertura normal e xa hai unha
/// aplicación aberta, pásalle os ficheiros e remata.
pub fn gate() {
    if is_job_launch() {
        gate_job();
    } else {
        gate_main();
    }
}

fn gate_job() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(job) = parse_job(&args) else { return };
    let dir = queue_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    if let Ok(text) = serde_json::to_string(&job) {
        let _ = std::fs::write(dir.join(format!("{}-{}.json", nanos(), std::process::id())), text);
    }
    let lock = dir.join("leader.lock");
    // Un bloqueo vello (un líder que morreu) non pode deixar a todos sen xanela.
    if let Ok(meta) = std::fs::metadata(&lock) {
        if meta.modified().ok().and_then(|m| m.elapsed().ok()).is_some_and(|age| age.as_secs() > 30) {
            let _ = std::fs::remove_file(&lock);
        }
    }
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock) {
        Ok(_) => LEADER.store(true, Ordering::SeqCst),
        Err(_) => std::process::exit(0),
    }
}

fn gate_main() {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = open_dir();
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        // Bloqueo exclusivo mentres viva a aplicación: Windows solta o ficheiro soo se o proceso morre.
        match std::fs::OpenOptions::new().write(true).create(true).truncate(false).share_mode(0).open(dir.join("main.lock")) {
            Ok(f) => {
                let _ = MAIN_LOCK.set(f);
            }
            Err(e) if e.raw_os_error() == Some(32) => {
                // ERROR_SHARING_VIOLATION: xa hai unha aplicación aberta. Pásanselle os ficheiros e remata.
                let paths: Vec<String> = std::env::args().skip(1).filter(|p| PathBuf::from(p).is_file()).collect();
                if let Ok(text) = serde_json::to_string(&paths) {
                    let _ = std::fs::write(dir.join(format!("{}-{}.json", nanos(), std::process::id())), text);
                }
                std::process::exit(0);
            }
            Err(_) => {}
        }
    }
}

/// Fío da aplicación principal: recolle os ficheiros que mandan as outras aperturas e avisa á interface.
pub fn watch_open_requests(app: tauri::AppHandle) {
    use tauri::{Emitter, Manager};
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let Ok(rd) = std::fs::read_dir(open_dir()) else { continue };
        let mut paths: Vec<String> = Vec::new();
        let mut any = false;
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            any = true;
            if let Some(list) = std::fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str::<Vec<String>>(&t).ok()) {
                paths.extend(list.into_iter().filter(|x| PathBuf::from(x).is_file()));
            }
            let _ = std::fs::remove_file(&p);
        }
        if any {
            let _ = app.emit("trasno://open", paths);
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    });
}

fn gather() -> Option<Job> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let own = parse_job(&args)?;
    if !LEADER.load(Ordering::SeqCst) {
        return Some(own);
    }
    // Dá tempo a que cheguen os outros procesos da mesma selección.
    std::thread::sleep(std::time::Duration::from_millis(1200));
    let dir = queue_dir();
    let _ = std::fs::remove_file(dir.join("leader.lock"));
    let mut all = own.clone();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            if let Some(j) = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str::<Job>(&t).ok()) {
                if j.action == own.action && j.target == own.target {
                    for p in j.paths {
                        if PathBuf::from(&p).is_file() && !all.paths.contains(&p) {
                            all.paths.push(p);
                        }
                    }
                }
            }
            let _ = std::fs::remove_file(&path);
        }
    }
    all.paths.sort();
    Some(all)
}

#[tauri::command]
pub async fn startup_job() -> Option<serde_json::Value> {
    tauri::async_runtime::spawn_blocking(|| RESULT.get_or_init(gather).clone())
        .await
        .ok()
        .flatten()
        .and_then(|j| serde_json::to_value(j).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Labels {
        Labels(
            [("title", "Arquivos do Trasno"), ("open", "Abrir"), ("convert", "Converter a {ext}"), ("compress", "Comprimir"), ("quality", "Calidade {p} %"), ("rotate", "Xirar"), ("left", "Á esquerda")]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }

    #[test]
    fn o_ficheiro_reg_ten_as_entradas_esperadas() {
        let reg = build_install_reg(r"C:\Programas\Trasno\trasno.exe", &labels());
        assert!(reg.starts_with("Windows Registry Editor Version 5.00"));
        assert!(reg.contains(r"[HKEY_CURRENT_USER\Software\Classes\SystemFileAssociations\.png\shell\ArquivosDoTrasno]"));
        assert!(reg.contains("\"MUIVerb\"=\"Converter a WEBP\""));
        assert!(reg.contains(r#"@="\"C:\\Programas\\Trasno\\trasno.exe\" --convert webp \"%1\"""#));
        // «Abrir» é só `"exe" "%1"`, sen argumentos de traballo.
        assert!(reg.contains(r#"@="\"C:\\Programas\\Trasno\\trasno.exe\"  \"%1\"""#) || reg.contains(r#"@="\"C:\\Programas\\Trasno\\trasno.exe\" \"%1\"""#));
        // Submenús aniñados con nivel de calidade en porcentaxe.
        assert!(reg.contains("\"MUIVerb\"=\"Calidade 75 %\""));
        assert!(reg.contains("--compress 50 "));
        assert!(reg.contains("--rotate 270 "));
        assert!(reg.contains("--resize h720 "));
        // O mesmo formato non se ofrece como destino de si mesmo.
        let png: String = reg.split("[HKEY_CURRENT_USER").filter(|b| b.starts_with(r"\Software\Classes\SystemFileAssociations\.png\")).collect();
        assert!(!png.contains("--convert png "));
        assert!(!reg.contains("[-HKEY"));
    }

    #[test]
    fn formatos_alias_son_o_mesmo() {
        assert!(same_format("jpeg", "jpg"));
        assert!(same_format("tif", "tiff"));
        assert!(!same_format("png", "jpg"));
    }

    #[test]
    fn os_argumentos_do_traballo_valídanse() {
        let me = std::env::current_exe().unwrap().to_string_lossy().to_string();
        let ok = parse_job(&["--compress".into(), "75".into(), me.clone()]).unwrap();
        assert_eq!(ok.action, "compress");
        assert_eq!(ok.target.as_deref(), Some("75"));
        assert!(parse_job(&["--convert".into(), "../x".into(), me.clone()]).is_none());
        assert!(parse_job(&["--convert".into(), "png".into(), "non-existe.png".into()]).is_none());
        assert!(parse_job(&["--outra".into(), "png".into(), me]).is_none());
    }
}
