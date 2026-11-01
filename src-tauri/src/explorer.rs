//! Integración co menú de clic dereito do Explorador de Windows («Converter a…»
//! sobre un ficheiro). Escríbese só en `HKEY_CURRENT_USER` (non fai falta ser
//! administrador) e pódese quitar enteira desde Axustes.
//!
//! Cada entrada lanza esta mesma aplicación cun traballo na liña de ordes:
//!   `trasno.exe --convert <formato> "<ficheiro>"`
//!   `trasno.exe --compress "<ficheiro>"`
//! A interface mostra unha xanela pequena co progreso (ver `QuickJob.svelte`).

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Labels {
    /// Título do submenú («Arquivos do Trasno»).
    pub title: String,
    /// «Abrir en Arquivos do Trasno».
    pub open: String,
    /// «Converter a {ext}»
    pub convert: String,
    /// «Comprimir»
    pub compress: String,
}

const ROOT: &str = r"Software\Classes\SystemFileAssociations";
const KEY: &str = "ArquivosDoTrasno";

/// Formatos de orixe e a que se poden converter desde o menú (lista curta, os máis usados).
fn menu() -> Vec<(&'static str, Vec<&'static str>, bool)> {
    let img_src = ["png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "tif", "webp", "tga", "qoi"];
    let img_dst = ["png", "jpg", "webp", "gif", "bmp", "tiff", "ico", "pdf"];
    let aud_src = ["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "wma", "aiff"];
    let aud_dst = ["mp3", "wav", "flac", "ogg", "m4a", "aac", "opus"];
    let vid_src = ["mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "mpg", "mpeg"];
    let vid_dst = ["mp4", "mkv", "webm", "mov", "avi", "gif", "mp3", "wav"];
    let txt_src = ["txt", "md", "csv", "json", "xml", "log"];
    let mut out: Vec<(&str, Vec<&str>, bool)> = Vec::new();
    for e in img_src {
        out.push((e, img_dst.to_vec(), true));
    }
    for e in aud_src {
        out.push((e, aud_dst.to_vec(), false));
    }
    for e in vid_src {
        out.push((e, vid_dst.to_vec(), true));
    }
    for e in txt_src {
        out.push((e, vec!["pdf"], false));
    }
    out.push(("pdf", vec!["txt"], true));
    out
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

fn reg_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
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
    let path = std::env::temp_dir().join(format!("trasno-{}-{}.reg", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
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

fn build_install_reg(exe: &str, labels: &Labels) -> String {
    let q = reg_str(exe);
    let mut reg = String::from("Windows Registry Editor Version 5.00\r\n\r\n");
    for (ext, targets, can_compress) in menu() {
        let base = format!("HKEY_CURRENT_USER\\{ROOT}\\.{ext}\\shell\\{KEY}");
        reg += &format!("[{base}]\r\n\"MUIVerb\"=\"{}\"\r\n\"SubCommands\"=\"\"\r\n\"Icon\"=\"\\\"{q}\\\",0\"\r\n\r\n", reg_str(&labels.title));
        reg += &format!("[{base}\\shell\\a_open]\r\n\"MUIVerb\"=\"{}\"\r\n\r\n[{base}\\shell\\a_open\\command]\r\n@=\"\\\"{q}\\\" \\\"%1\\\"\"\r\n\r\n", reg_str(&labels.open));
        for (i, t) in targets.iter().filter(|t| !same_format(t, ext)).enumerate() {
            let label = labels.convert.replace("{ext}", &t.to_uppercase());
            reg += &format!(
                "[{base}\\shell\\b_{i:02}_{t}]\r\n\"MUIVerb\"=\"{}\"\r\n\r\n[{base}\\shell\\b_{i:02}_{t}\\command]\r\n@=\"\\\"{q}\\\" --convert {t} \\\"%1\\\"\"\r\n\r\n",
                reg_str(&label)
            );
        }
        if can_compress {
            reg += &format!(
                "[{base}\\shell\\c_compress]\r\n\"MUIVerb\"=\"{}\"\r\n\r\n[{base}\\shell\\c_compress\\command]\r\n@=\"\\\"{q}\\\" --compress \\\"%1\\\"\"\r\n\r\n",
                reg_str(&labels.compress)
            );
        }
    }
    reg
}

#[cfg(windows)]
fn exe() -> Result<String, String> {
    std::env::current_exe().map(|p| p.to_string_lossy().to_string()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn explorer_install(labels: Labels) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        let _ = labels;
        Err("Esta función só existe en Windows.".into())
    }
    #[cfg(windows)]
    {
        let reg = build_install_reg(&exe()?, &labels);
        import_reg(&reg)
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
        for (ext, _, _) in menu() {
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
        let key = format!("HKCU\\{ROOT}\\.png\\shell\\{KEY}\\shell\\a_open\\command");
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

/// Traballo pedido desde o menú de clic dereito, se a aplicación se lanzou así.
#[tauri::command]
pub fn startup_job() -> Option<serde_json::Value> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let valid = |p: &String| PathBuf::from(p).is_file();
    match args.first().map(String::as_str) {
        Some("--convert") if args.len() >= 3 => {
            let target = args[1].to_lowercase();
            // O formato de destino é só letras e números: nada que se poida confundir cunha ruta.
            if target.is_empty() || !target.chars().all(|c| c.is_ascii_alphanumeric()) {
                return None;
            }
            let paths: Vec<&String> = args[2..].iter().filter(|p| valid(p)).collect();
            (!paths.is_empty()).then(|| serde_json::json!({ "action": "convert", "target": target, "paths": paths }))
        }
        Some("--compress") if args.len() >= 2 => {
            let paths: Vec<&String> = args[1..].iter().filter(|p| valid(p)).collect();
            (!paths.is_empty()).then(|| serde_json::json!({ "action": "compress", "paths": paths }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_ficheiro_reg_ten_as_entradas_esperadas() {
        let labels = Labels { title: "Arquivos do Trasno".into(), open: "Abrir".into(), convert: "Converter a {ext}".into(), compress: "Comprimir".into() };
        let reg = build_install_reg(r"C:\Programas\Trasno\trasno.exe", &labels);
        assert!(reg.starts_with("Windows Registry Editor Version 5.00"));
        assert!(reg.contains(r"[HKEY_CURRENT_USER\Software\Classes\SystemFileAssociations\.png\shell\ArquivosDoTrasno]"));
        assert!(reg.contains("\"MUIVerb\"=\"Converter a WEBP\""));
        assert!(reg.contains(r#"@="\"C:\\Programas\\Trasno\\trasno.exe\" --convert webp \"%1\"""#));
        assert!(reg.contains("--compress"));
        // O mesmo formato non se ofrece como destino de si mesmo.
        let png = reg.split("[HKEY_CURRENT_USER").filter(|b| b.contains(r".png\shell")).collect::<String>();
        assert!(!png.contains("--convert png "));
        assert!(!reg.contains("[-HKEY"));
    }

    #[test]
    fn formatos_alias_son_o_mesmo() {
        assert!(same_format("jpeg", "jpg"));
        assert!(same_format("tif", "tiff"));
        assert!(!same_format("png", "jpg"));
    }
}
