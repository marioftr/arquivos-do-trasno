//! Rexistro de procesos compartido entre procesos: o que fan as xanelas rápidas do clic dereito e a
//! aplicación principal. Cada traballo deixa un ficheiro JSON; a aplicación principal lista os
//! que están en marcha e un pequeno historial, e pode pedir cancelar un deixando un ficheiro `.cancel`
//! que o dono do traballo mira cada pouco.

use serde_json::{json, Value};
use std::path::PathBuf;

const STALE_SECS: u64 = 20; // un traballo «en marcha» sen latexar tanto tempo considérase interrompido
const KEEP_FINISHED: usize = 40;
const KEEP_DAYS: u64 = 14;

fn dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let d = base.join("ArquivosDoTrasno").join("procesos");
    let _ = std::fs::create_dir_all(&d);
    d
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// O identificador vai no nome do ficheiro: só caracteres seguros.
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[tauri::command]
pub fn proc_upsert(rec: Value) -> Result<(), String> {
    let id = rec.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if !valid_id(&id) {
        return Err("Identificador de proceso non válido.".into());
    }
    let mut rec = rec;
    if let Some(o) = rec.as_object_mut() {
        o.insert("updated".into(), json!(now_ms()));
        o.insert("pid".into(), json!(std::process::id()));
    }
    let d = dir();
    let tmp = d.join(format!("{id}.tmp"));
    std::fs::write(&tmp, serde_json::to_vec(&rec).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, d.join(format!("{id}.json"))).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn proc_list() -> Vec<Value> {
    let d = dir();
    let now = now_ms();
    let mut out: Vec<Value> = Vec::new();
    let mut finished: Vec<(u64, PathBuf)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&d) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            let Some(mut v) = std::fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else {
                let _ = std::fs::remove_file(&p);
                continue;
            };
            let updated = v.get("updated").and_then(|x| x.as_u64()).unwrap_or(0);
            let state = v.get("state").and_then(|x| x.as_str()).unwrap_or("").to_string();
            if state == "running" {
                if now.saturating_sub(updated) > STALE_SECS * 1000 {
                    v["state"] = json!("interrupted");
                    v["ended"] = json!(updated);
                }
            } else {
                finished.push((updated, p.clone()));
            }
            out.push(v);
        }
    }
    // Historial acotado: os máis vellos fóra.
    finished.sort_by(|a, b| b.0.cmp(&a.0));
    for (i, (updated, p)) in finished.iter().enumerate() {
        if i >= KEEP_FINISHED || now.saturating_sub(*updated) > KEEP_DAYS * 86_400_000 {
            let _ = std::fs::remove_file(p);
            let _ = std::fs::remove_file(p.with_extension("cancel"));
            if let Some(id) = p.file_stem().and_then(|s| s.to_str()) {
                out.retain(|v| v.get("id").and_then(|x| x.as_str()) != Some(id));
            }
        }
    }
    out.sort_by(|a, b| b.get("started").and_then(|x| x.as_u64()).cmp(&a.get("started").and_then(|x| x.as_u64())));
    out
}

#[tauri::command]
pub fn proc_request_cancel(id: String) -> Result<(), String> {
    if !valid_id(&id) {
        return Err("Identificador de proceso non válido.".into());
    }
    std::fs::write(dir().join(format!("{id}.cancel")), b"1").map_err(|e| e.to_string())
}

#[tauri::command]
pub fn proc_cancel_requested(id: String) -> bool {
    valid_id(&id) && dir().join(format!("{id}.cancel")).exists()
}

#[tauri::command]
pub fn proc_clear_finished() {
    let d = dir();
    if let Ok(rd) = std::fs::read_dir(&d) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            let running = std::fs::read_to_string(&p)
                .ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                .map(|v| {
                    let updated = v.get("updated").and_then(|x| x.as_u64()).unwrap_or(0);
                    v.get("state").and_then(|x| x.as_str()) == Some("running") && now_ms().saturating_sub(updated) <= STALE_SECS * 1000
                })
                .unwrap_or(false);
            if !running {
                let _ = std::fs::remove_file(&p);
                let _ = std::fs::remove_file(p.with_extension("cancel"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_ids_valídanse() {
        assert!(valid_id("abc-123_X"));
        assert!(!valid_id(""));
        assert!(!valid_id("../x"));
        assert!(!valid_id("a b"));
        assert!(!valid_id(&"a".repeat(65)));
    }
}
