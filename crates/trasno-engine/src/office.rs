//! Documentos de Office (Word, Excel, PowerPoint e OpenDocument): conversión entre os seus formatos e a PDF.
//!
//! O trasno non le estes formatos por si mesmo: usa o que haxa no equipo. En Windows, Microsoft Office (Word, Excel e PowerPoint
//! se están instalados, manexados en segundo plano e sen xanelas) e, onde non, LibreOffice (`soffice --headless`). Escríbese
//! sempre nun cartafol temporal do mesmo directorio de destino e só se move ao nome final cando o resultado se comprobou.

use crate::error::{EngineError, Result};
use crate::formats::extension_of;
use crate::integrity::ensure_distinct;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    Word,
    Excel,
    PowerPoint,
}

pub const WORD_EXTS: &[&str] = &["docx", "doc", "docm", "odt", "rtf"];
pub const EXCEL_EXTS: &[&str] = &["xlsx", "xls", "xlsm", "ods"];
pub const POWERPOINT_EXTS: &[&str] = &["pptx", "ppt", "pptm", "ppsx", "pps", "odp"];

pub fn suite_of(ext: &str) -> Option<Suite> {
    let e = ext.to_ascii_lowercase();
    let e = e.as_str();
    if WORD_EXTS.contains(&e) {
        Some(Suite::Word)
    } else if EXCEL_EXTS.contains(&e) {
        Some(Suite::Excel)
    } else if POWERPOINT_EXTS.contains(&e) {
        Some(Suite::PowerPoint)
    } else {
        None
    }
}

pub fn is_office_ext(ext: &str) -> bool {
    suite_of(ext).is_some()
}

/// Un formato de saída: extensión, código de «gardar como» de Microsoft Office e filtro de LibreOffice.
struct Out {
    ext: &'static str,
    /// Para Microsoft Office: o código de formato; `export` indica que se usa a exportación a PDF en vez de «gardar como».
    ms: Option<(i32, bool)>,
    lo: Option<&'static str>,
}

// Ordenados do máis común ao menos.
const WORD_OUT: &[Out] = &[
    Out { ext: "pdf", ms: Some((17, true)), lo: Some("pdf") },
    Out { ext: "docx", ms: Some((16, false)), lo: Some("docx") },
    Out { ext: "odt", ms: Some((23, false)), lo: Some("odt") },
    Out { ext: "doc", ms: Some((0, false)), lo: Some("doc") },
    Out { ext: "rtf", ms: Some((6, false)), lo: Some("rtf") },
    Out { ext: "txt", ms: Some((2, false)), lo: Some("txt:Text (encoded):UTF8") },
];
const EXCEL_OUT: &[Out] = &[
    Out { ext: "pdf", ms: Some((0, true)), lo: Some("pdf") },
    Out { ext: "xlsx", ms: Some((51, false)), lo: Some("xlsx") },
    Out { ext: "ods", ms: Some((60, false)), lo: Some("ods") },
    Out { ext: "xls", ms: Some((56, false)), lo: Some("xls") },
    Out { ext: "csv", ms: Some((62, false)), lo: Some("csv:Text - txt - csv (StarCalc):44,34,76,1") },
];
const POWERPOINT_OUT: &[Out] = &[
    Out { ext: "pdf", ms: Some((32, false)), lo: Some("pdf") },
    Out { ext: "pptx", ms: Some((24, false)), lo: Some("pptx") },
    Out { ext: "odp", ms: Some((35, false)), lo: Some("odp") },
    Out { ext: "ppt", ms: Some((1, false)), lo: Some("ppt") },
];
/// Un PDF só se converte a documento con Word (reconstrúe o texto e a maquetación).
#[allow(dead_code)]
const PDF_OUT: &[Out] = &[
    Out { ext: "docx", ms: Some((16, false)), lo: None },
    Out { ext: "doc", ms: Some((0, false)), lo: None },
    Out { ext: "odt", ms: Some((23, false)), lo: None },
    Out { ext: "rtf", ms: Some((6, false)), lo: None },
    Out { ext: "txt", ms: Some((2, false)), lo: None },
];

fn outs_for(ext: &str) -> Option<(Option<Suite>, &'static [Out])> {
    // PDF → documento con Word: desactivado ata comprobar que funciona sempre (Word queda esperando ao abrir PDF nalgúns equipos).
    if ext == "pdf" {
        return None;
    }
    match suite_of(ext)? {
        Suite::Word => Some((Some(Suite::Word), WORD_OUT)),
        Suite::Excel => Some((Some(Suite::Excel), EXCEL_OUT)),
        Suite::PowerPoint => Some((Some(Suite::PowerPoint), POWERPOINT_OUT)),
    }
}

// ------------------------------------------------------------ detección

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct OfficeStatus {
    pub word: bool,
    pub excel: bool,
    pub powerpoint: bool,
    pub libreoffice: bool,
}

#[cfg(windows)]
fn com_registered(prog_id: &str) -> bool {
    let mut cmd = Command::new("reg");
    cmd.args(["query", &format!("HKLM\\SOFTWARE\\Classes\\{prog_id}\\CLSID")]);
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000);
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

#[cfg(not(windows))]
fn com_registered(_prog_id: &str) -> bool {
    false
}

pub fn soffice_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("TRASNO_SOFFICE_PATH") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    let mut cands: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    for var in ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"] {
        if let Ok(base) = std::env::var(var) {
            cands.push(PathBuf::from(base).join("LibreOffice").join("program").join("soffice.exe"));
        }
    }
    #[cfg(target_os = "macos")]
    cands.push(PathBuf::from("/Applications/LibreOffice.app/Contents/MacOS/soffice"));
    cands.into_iter().find(|p| p.is_file()).or_else(|| which::which("soffice").ok()).or_else(|| which::which("libreoffice").ok())
}

pub fn status() -> OfficeStatus {
    OfficeStatus {
        word: com_registered("Word.Application"),
        excel: com_registered("Excel.Application"),
        powerpoint: com_registered("PowerPoint.Application"),
        libreoffice: soffice_path().is_some(),
    }
}

fn ms_has(st: &OfficeStatus, suite: Suite) -> bool {
    match suite {
        Suite::Word => st.word,
        Suite::Excel => st.excel,
        Suite::PowerPoint => st.powerpoint,
    }
}

/// Formatos de saída posibles para un ficheiro con esta extensión, do máis común ao menos, segundo o que haxa instalado.
pub fn targets_for(ext: &str) -> Vec<&'static str> {
    let ext = ext.to_ascii_lowercase();
    let Some((suite, outs)) = outs_for(&ext) else { return Vec::new() };
    let st = status();
    outs.iter()
        .filter(|o| o.ext != ext)
        .filter(|o| match suite {
            Some(s) => (ms_has(&st, s) && o.ms.is_some()) || (st.libreoffice && o.lo.is_some()),
            None => st.word && o.ms.is_some(),
        })
        .map(|o| o.ext)
        .collect()
}

// ------------------------------------------------------------ conversión

const TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Convirte `input` ao formato da extensión de `output`.
pub fn convert(input: &Path, output: &Path, overwrite: bool) -> Result<PathBuf> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    ensure_distinct(&[input], output)?;
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    let src_ext = extension_of(input).unwrap_or_default();
    let dst_ext = extension_of(output).unwrap_or_default();
    let unsupported = || EngineError::UnsupportedConversion { from: src_ext.clone(), to: dst_ext.clone() };
    let (suite, outs) = outs_for(&src_ext).ok_or_else(unsupported)?;
    let o = outs.iter().find(|o| o.ext == dst_ext).ok_or_else(unsupported)?;

    let st = status();
    // Prefírese Microsoft Office (maior fidelidade cos seus propios formatos); LibreOffice, se non hai ou non sabe facelo.
    let use_ms = match suite {
        Some(s) => ms_has(&st, s) && o.ms.is_some(),
        None => st.word && o.ms.is_some(),
    };
    let use_lo = !use_ms && st.libreoffice && o.lo.is_some();
    if !use_ms && !use_lo {
        return Err(if suite.is_none() { EngineError::OfficeMissing("Para converter un PDF a documento fai falta Microsoft Word.".into()) } else { EngineError::OfficeMissing(String::new()) });
    }

    let dir = output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let work = tempfile::Builder::new().prefix(".trasno-office-").tempdir_in(dir)?;
    let result = work.path().join(format!("resultado.{dst_ext}"));

    if use_ms {
        run_microsoft(suite.unwrap_or(Suite::Word), input, &result, o, work.path())?;
    } else {
        run_libreoffice(input, &result, &dst_ext, o, work.path())?;
    }

    let ok = std::fs::metadata(&result).map(|m| m.len() > 0).unwrap_or(false);
    if !ok {
        eprintln!("[office] sen resultado: {result:?} existe={}", result.exists());
        return Err(EngineError::EmptyOrCorruptOutput);
    }
    if dst_ext == "pdf" {
        let head = std::fs::read(&result).map(|b| b.starts_with(b"%PDF")).unwrap_or(false);
        if !head {
            eprintln!("[office] o PDF non empeza por %PDF: {:?}", std::fs::read(&result).map(|b| b.into_iter().take(16).collect::<Vec<_>>()));
            return Err(EngineError::EmptyOrCorruptOutput);
        }
    }
    if output.exists() {
        std::fs::remove_file(output)?;
    }
    std::fs::rename(&result, output)?;
    Ok(output.to_path_buf())
}

fn first_lines(s: &str) -> String {
    s.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with("PID ")).take(4).collect::<Vec<_>>().join(" ")
}

const PS_PRELUDE: &str = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$in = $env:TRASNO_IN; $out = $env:TRASNO_OUT; $fmt = [int]$env:TRASNO_FMT
trap { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }
function Procs($name) { @(Get-Process -Name $name -ErrorAction SilentlyContinue | ForEach-Object { $_.Id }) }
# Os procesos novos desde que arrancou a aplicación son os do trasno (e nunca os que o usuario teña abertos): o motor
# anótaos para poder cancelalos e aquí péchanse á forza se non saen soos.
function Mine($name, $before) {
  Start-Sleep -Milliseconds 400
  $m = @(Procs $name | Where-Object { $before -notcontains $_ })
  foreach ($x in $m) { [Console]::Out.WriteLine('PID ' + $x) }
  return ,$m
}
function Reap($mine) {
  foreach ($x in $mine) {
    $p = Get-Process -Id $x -ErrorAction SilentlyContinue
    if ($p) { [void]$p.WaitForExit(8000); if (-not $p.HasExited) { try { $p.Kill() } catch {} } }
  }
}
"#;

const PS_WORD: &str = r#"
$before = Procs 'WINWORD'
$app = New-Object -ComObject Word.Application
$mine = Mine 'WINWORD' $before
try {
  try { $app.Visible = $false; $app.DisplayAlerts = 0 } catch {}
  $doc = $app.Documents.Open($in, $false, $true)
  if ($env:TRASNO_EXPORT -eq '1') { $doc.ExportAsFixedFormat($out, 17) }
  elseif ($fmt -eq 2) { $m = [Type]::Missing; $doc.SaveAs2($out, 2, $m, $m, $m, $m, $m, $m, $m, $m, $m, 65001) }
  else { $doc.SaveAs2($out, $fmt) }
  $doc.Close(0)
} finally { try { $app.Quit() } catch {}; Reap $mine }
"#;

const PS_EXCEL: &str = r#"
$before = Procs 'EXCEL'
$app = New-Object -ComObject Excel.Application
$mine = Mine 'EXCEL' $before
try {
  try { $app.Visible = $false; $app.DisplayAlerts = $false } catch {}
  $wb = $app.Workbooks.Open($in, 0, $true)
  if ($env:TRASNO_EXPORT -eq '1') { $wb.ExportAsFixedFormat(0, $out) } else { $wb.SaveAs($out, $fmt) }
  $wb.Close($false)
} finally { try { $app.Quit() } catch {}; Reap $mine }
"#;

const PS_POWERPOINT: &str = r#"
$before = Procs 'POWERPNT'
$app = New-Object -ComObject PowerPoint.Application
$mine = Mine 'POWERPNT' $before
try {
  $p = $app.Presentations.Open($in, -1, 0, 0)
  $p.SaveAs($out, $fmt)
  $p.Close()
} finally { try { $app.Quit() } catch {}; Reap $mine }
"#;

#[cfg(windows)]
fn run_microsoft(suite: Suite, input: &Path, result: &Path, o: &Out, work: &Path) -> Result<()> {
    let (code, export) = o.ms.unwrap_or((0, false));
    let body = match suite {
        Suite::Word => PS_WORD,
        Suite::Excel => PS_EXCEL,
        Suite::PowerPoint => PS_POWERPOINT,
    };
    let script = work.join("convertir.ps1");
    // Con BOM: Windows PowerShell 5 le así o UTF-8.
    let mut text = String::from('\u{feff}');
    text.push_str(PS_PRELUDE);
    text.push_str(body);
    std::fs::write(&script, text)?;
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&script)
        .env("TRASNO_IN", input)
        .env("TRASNO_OUT", result)
        .env("TRASNO_FMT", code.to_string())
        .env("TRASNO_EXPORT", if export { "1" } else { "0" });
    let r = crate::ffmpeg::run_external(cmd, TIMEOUT)?;
    if r.code != Some(0) {
        let why = first_lines(&r.stderr);
        let why = if why.is_empty() { first_lines(&r.stdout) } else { why };
        return Err(EngineError::OfficeFailed(if why.is_empty() { "o programa de Office non deu ningunha explicación".into() } else { why }));
    }
    Ok(())
}

#[cfg(not(windows))]
fn run_microsoft(_: Suite, _: &Path, _: &Path, _: &Out, _: &Path) -> Result<()> {
    Err(EngineError::OfficeMissing(String::new()))
}

fn file_url(p: &Path) -> String {
    let mut s = p.to_string_lossy().replace('\\', "/");
    if !s.starts_with('/') {
        s.insert(0, '/');
    }
    let mut out = String::from("file://");
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"/-_.~:".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn run_libreoffice(input: &Path, result: &Path, dst_ext: &str, o: &Out, work: &Path) -> Result<()> {
    let soffice = soffice_path().ok_or_else(|| EngineError::OfficeMissing(String::new()))?;
    let outdir = work.join("lo-saida");
    std::fs::create_dir_all(&outdir)?;
    let profile = work.join("lo-perfil");
    // O ficheiro con nome ASCII sinxelo evita problemas de codificación na liña de ordes.
    let src = work.join(format!("orixe.{}", extension_of(input).unwrap_or_default()));
    std::fs::copy(input, &src)?;
    let mut cmd = Command::new(soffice);
    cmd.arg(format!("-env:UserInstallation={}", file_url(&profile)))
        .args(["--headless", "--norestore", "--nologo", "--nodefault", "--convert-to"])
        .arg(o.lo.unwrap_or(dst_ext))
        .arg("--outdir")
        .arg(&outdir)
        .arg(&src);
    let r = crate::ffmpeg::run_external(cmd, TIMEOUT)?;
    let produced = outdir.join(format!("orixe.{dst_ext}"));
    if r.code != Some(0) || !produced.is_file() {
        let why = first_lines(&r.stderr);
        let why = if why.is_empty() { first_lines(&r.stdout) } else { why };
        return Err(EngineError::OfficeFailed(if why.is_empty() { "LibreOffice non deu ningunha explicación".into() } else { why }));
    }
    std::fs::rename(&produced, result)?;
    Ok(())
}
