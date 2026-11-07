//! Conversión de documentos de Office. Só se proba o que haxa instalado no equipo (Microsoft Office e/ou LibreOffice):
//! cada formato de saída que o motor ofrece ten que dar un ficheiro válido e co texto dentro.

use std::io::Write;
use std::path::{Path, PathBuf};
use trasno_engine::office;

fn zip(path: &Path, parts: &[(&str, &str)]) {
    let f = std::fs::File::create(path).unwrap();
    let mut z = zip::ZipWriter::new(f);
    let o = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, body) in parts {
        z.start_file(*name, o).unwrap();
        z.write_all(body.as_bytes()).unwrap();
    }
    z.finish().unwrap();
}

const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const TXT: &str = "Galicia ñ á é í ó ú Trasno";

fn make_docx(dir: &Path) -> PathBuf {
    let p = dir.join("mostra.docx");
    zip(
        &p,
        &[
            ("[Content_Types].xml", r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#),
            ("_rels/.rels", &format!(r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{REL}/officeDocument" Target="word/document.xml"/></Relationships>"#)),
            ("word/document.xml", &format!(r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>{TXT}</w:t></w:r></w:p></w:body></w:document>"#)),
        ],
    );
    p
}

fn make_xlsx(dir: &Path) -> PathBuf {
    let p = dir.join("mostra.xlsx");
    zip(
        &p,
        &[
            ("[Content_Types].xml", r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#),
            ("_rels/.rels", &format!(r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{REL}/officeDocument" Target="xl/workbook.xml"/></Relationships>"#)),
            ("xl/workbook.xml", &format!(r#"<?xml version="1.0" encoding="UTF-8"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="{REL}"><sheets><sheet name="Folla1" sheetId="1" r:id="rId1"/></sheets></workbook>"#)),
            ("xl/_rels/workbook.xml.rels", &format!(r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{REL}/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#)),
            ("xl/worksheets/sheet1.xml", &format!(r#"<?xml version="1.0" encoding="UTF-8"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>{TXT}</t></is></c><c r="B1"><v>42</v></c></row></sheetData></worksheet>"#)),
        ],
    );
    p
}

#[cfg(windows)]
fn make_pptx(dir: &Path) -> Option<PathBuf> {
    let p = dir.join("mostra.pptx");
    let script = "$a = New-Object -ComObject PowerPoint.Application; $pr = $a.Presentations.Add(0); $s = $pr.Slides.Add(1, 12); $t = $s.Shapes.AddTextbox(1, 50, 50, 600, 80); $t.TextFrame.TextRange.Text = $env:TRASNO_TXT; $pr.SaveAs($env:TRASNO_OUT); $pr.Close(); $a.Quit()";
    let ok = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("TRASNO_TXT", TXT)
        .env("TRASNO_OUT", &p)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    (ok && p.is_file()).then_some(p)
}

#[cfg(not(windows))]
fn make_pptx(_: &Path) -> Option<PathBuf> {
    None
}

/// Convirte a cada formato de saída ofrecido e devolve os fallos (nome → motivo).
fn all_targets(src: &Path, dir: &Path, check_text: bool) -> Vec<String> {
    let ext = src.extension().unwrap().to_str().unwrap().to_string();
    let targets = office::targets_for(&ext);
    println!("{ext}: saídas ofrecidas {targets:?}");
    let mut fails = Vec::new();
    for t in targets {
        let out = dir.join(format!("saida-de-{ext}.{t}"));
        match office::convert(src, &out, true) {
            Err(e) => fails.push(format!("{ext} → {t}: {e}")),
            Ok(p) => {
                let bytes = std::fs::read(&p).unwrap_or_default();
                if bytes.is_empty() {
                    fails.push(format!("{ext} → {t}: baleiro"));
                    continue;
                }
                let ok = match t {
                    "pdf" => bytes.starts_with(b"%PDF"),
                    "docx" | "xlsx" | "pptx" | "odt" | "ods" | "odp" => bytes.starts_with(b"PK"),
                    "doc" | "xls" | "ppt" => bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]),
                    "rtf" => bytes.starts_with(b"{\\rtf"),
                    _ => true,
                };
                if !ok {
                    fails.push(format!("{ext} → {t}: cabeceira inesperada"));
                }
                if check_text && matches!(t, "txt" | "csv" | "html") {
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    let text = if bytes.starts_with(&[0xFF, 0xFE]) { String::from_utf16_lossy(&bytes[2..].chunks(2).map(|c| u16::from_le_bytes([c[0], *c.get(1).unwrap_or(&0)])).collect::<Vec<_>>()) } else { text };
                    // Word escribe o HTML en Windows-1252: o «ñ» vai como o byte 0xF1 (ou como entidade).
                    let ntilde = text.contains('ñ') || text.contains("&ntilde;") || text.contains("&#241;") || bytes.contains(&0xF1);
                    if !text.contains("Galicia") || !ntilde {
                        fails.push(format!("{ext} → {t}: falta o texto ou o «ñ»: {:?}", &text.chars().take(120).collect::<String>()));
                    }
                }
            }
        }
    }
    fails
}

fn skip(msg: &str) {
    println!("{msg}: non se proba");
}

fn report(fails: Vec<String>) {
    assert!(fails.is_empty(), "conversións con problemas:
{}", fails.join("
"));
}

#[test]
fn word_a_todos_os_formatos() {
    let st = office::status();
    println!("estado: {st:?}");
    if !(st.word || st.libreoffice) {
        return skip("sen Word nin LibreOffice");
    }
    let dir = tempfile::tempdir().unwrap();
    let docx = make_docx(dir.path());
    let mut fails = all_targets(&docx, dir.path(), true);
    // Ida e volta: o .odt, o .rtf e o .doc xerados poden volver a docx e a PDF.
    for mid in ["odt", "rtf", "doc"] {
        let m = dir.path().join(format!("saida-de-docx.{mid}"));
        if m.is_file() {
            fails.extend(all_targets(&m, dir.path(), false));
        }
    }
    report(fails);
}

#[test]
fn excel_a_todos_os_formatos() {
    let st = office::status();
    if !(st.excel || st.libreoffice) {
        return skip("sen Excel nin LibreOffice");
    }
    let dir = tempfile::tempdir().unwrap();
    let xlsx = make_xlsx(dir.path());
    report(all_targets(&xlsx, dir.path(), true));
}

#[test]
fn powerpoint_a_todos_os_formatos() {
    let st = office::status();
    if !(st.powerpoint || st.libreoffice) {
        return skip("sen PowerPoint nin LibreOffice");
    }
    let dir = tempfile::tempdir().unwrap();
    match make_pptx(dir.path()) {
        Some(pptx) => report(all_targets(&pptx, dir.path(), false)),
        None => skip("non se puido crear un .pptx de mostra"),
    }
}

#[test]
#[ignore = "PDF → documento con Word está desactivado ata comprobar que funciona sempre"]
fn pdf_a_documento_con_word() {
    let st = office::status();
    if !st.word {
        return skip("sen Word");
    }
    let dir = tempfile::tempdir().unwrap();
    let docx = make_docx(dir.path());
    let pdf = dir.path().join("mostra.pdf");
    office::convert(&docx, &pdf, true).unwrap();
    report(all_targets(&pdf, dir.path(), false));
}
