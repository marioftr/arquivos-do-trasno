//! Operacións básicas sobre PDF con `lopdf` (unir, dividir, rotar,
//! reordenar páxinas). A sinatura dixital e a conversión doutros formatos
//! a/desde PDF quedan para unha fase posterior (documentadas no ROADMAP).

use crate::error::{EngineError, Result};
use crate::integrity::{assert_non_empty_file, ensure_distinct, AtomicOutput};
use lopdf::{dictionary, Document, Object, ObjectId, Stream};
use std::path::{Path, PathBuf};

fn load(path: &Path) -> Result<Document> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    Document::load(path).map_err(|e| EngineError::Pdf(e.to_string()))
}

/// Segue a cadea `Parent` dunha páxina cara arriba ata atopar `key`
/// (MediaBox, Resources, Rotate e CropBox pódense herdar da árbore de
/// páxinas en vez de estar definidos na propia páxina).
fn resolve_inherited(doc: &Document, start: ObjectId, key: &[u8]) -> Option<lopdf::Object> {
    let mut id = start;
    for _ in 0..64 {
        let dict = doc.get_object(id).ok()?.as_dict().ok()?;
        if let Ok(value) = dict.get(key) {
            return Some(value.clone());
        }
        match dict.get(b"Parent") {
            Ok(lopdf::Object::Reference(parent_id)) => id = *parent_id,
            _ => return None,
        }
    }
    None
}

/// Copia sobre cada páxina os atributos herdados que só estaban definidos
/// nun antepasado na árbore de páxinas, para que sigan sendo correctos
/// despois de mover a páxina a unha árbore de páxinas distinta (unir,
/// extraer...).
fn flatten_inherited_page_attributes(doc: &mut Document) {
    const INHERITABLE: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];
    let page_ids: Vec<ObjectId> = doc.get_pages().into_values().collect();
    for page_id in page_ids {
        for key in INHERITABLE {
            let already_set = doc
                .get_object(page_id)
                .ok()
                .and_then(|o| o.as_dict().ok())
                .map(|d| d.has(key))
                .unwrap_or(false);
            if already_set {
                continue;
            }
            if let Some(value) = resolve_inherited(doc, page_id, key) {
                if let Ok(dict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
                    dict.set(key, value);
                }
            }
        }
    }
}

fn save_atomically(doc: &mut Document, output: &Path, overwrite: bool) -> Result<PathBuf> {
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    let dir = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;

    let temp_path = dir.join(format!(".trasno-{}.pdf.part", uuid::Uuid::new_v4()));
    doc.save(&temp_path)
        .map_err(|e| EngineError::Pdf(e.to_string()))?;

    if let Err(e) = assert_non_empty_file(&temp_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }
    // Verificación: que o PDF resultante se poida volver cargar.
    if Document::load(&temp_path).is_err() {
        let _ = std::fs::remove_file(&temp_path);
        return Err(EngineError::EmptyOrCorruptOutput);
    }

    std::fs::rename(&temp_path, output).map_err(EngineError::Io)?;
    Ok(output.to_path_buf())
}

/// Une varios PDF, na orde dada, nun só ficheiro.
pub fn merge_pdfs(inputs: &[PathBuf], output: &Path, overwrite: bool) -> Result<PathBuf> {
    if inputs.len() < 2 {
        return Err(EngineError::Pdf(
            "fan falta polo menos dous PDF para unir".into(),
        ));
    }

    let refs: Vec<&Path> = inputs.iter().map(|p| p.as_path()).collect();
    ensure_distinct(&refs, output)?;
    let mut merged = Document::with_version("1.7");
    let mut merged_pages: Vec<ObjectId> = Vec::new();
    let mut next_id = 1u32;

    for input in inputs {
        let mut doc = load(input)?;
        flatten_inherited_page_attributes(&mut doc);
        doc.renumber_objects_with(next_id);
        next_id = doc.max_id + 1;

        let pages = doc.get_pages();
        for (_, page_id) in pages {
            merged_pages.push(page_id);
        }
        merged.objects.extend(doc.objects);
    }

    // `merged` nunca pasou polo seu propio `add_object`/`new_object_id` ata
    // agora (só se lle inseriron obxectos directamente en `.objects`), así
    // que hai que dicirlle onde quedou o reconto de IDs antes de pedirlle
    // novos; se non, `new_object_id` devolvería IDs xa usados por algunha
    // das páxinas copiadas e sobrescribiríaas.
    merged.max_id = next_id - 1;

    let pages_dict_id = merged.new_object_id();
    let kids: Vec<lopdf::Object> = merged_pages
        .iter()
        .map(|id| lopdf::Object::Reference(*id))
        .collect();
    let mut pages_dict = lopdf::Dictionary::new();
    pages_dict.set("Type", lopdf::Object::Name(b"Pages".to_vec()));
    pages_dict.set("Count", lopdf::Object::Integer(kids.len() as i64));
    pages_dict.set("Kids", lopdf::Object::Array(kids));
    merged
        .objects
        .insert(pages_dict_id, lopdf::Object::Dictionary(pages_dict));

    for page_id in &merged_pages {
        if let Ok(page_obj) = merged.get_object_mut(*page_id) {
            if let Ok(dict) = page_obj.as_dict_mut() {
                dict.set("Parent", lopdf::Object::Reference(pages_dict_id));
            }
        }
    }

    let catalog_id = merged.new_object_id();
    let mut catalog = lopdf::Dictionary::new();
    catalog.set("Type", lopdf::Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", lopdf::Object::Reference(pages_dict_id));
    merged
        .objects
        .insert(catalog_id, lopdf::Object::Dictionary(catalog));
    merged.trailer.set("Root", lopdf::Object::Reference(catalog_id));

    merged.max_id = next_id;
    merged.renumber_objects();
    merged.compress();

    save_atomically(&mut merged, output, overwrite)
}

/// Extrae un subconxunto de páxinas (1-indexadas, na orde indicada) a un
/// novo PDF — serve tanto para "dividir" coma para "reordenar".
pub fn extract_pages(
    input: &Path,
    page_numbers: &[u32],
    output: &Path,
    overwrite: bool,
) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let mut doc = load(input)?;
    flatten_inherited_page_attributes(&mut doc);
    if page_numbers.is_empty() {
        return Err(EngineError::Pdf("non se indicou ningunha páxina".into()));
    }

    let all_pages = doc.get_pages();
    let total = all_pages.len() as u32;
    for &n in page_numbers {
        if n == 0 || n > total {
            return Err(EngineError::Pdf(format!(
                "a páxina {n} non existe (o documento ten {total})"
            )));
        }
    }

    let keep: std::collections::HashSet<u32> = page_numbers.iter().copied().collect();
    let to_delete: Vec<u32> = (1..=total).filter(|n| !keep.contains(n)).collect();
    if !to_delete.is_empty() {
        doc.delete_pages(&to_delete);
    }
    doc.prune_objects();
    doc.renumber_objects();
    doc.compress();

    save_atomically(&mut doc, output, overwrite)
}

/// Rota unha páxina (0 significa "todas") un múltiplo de 90 graos.
pub fn rotate_pages(
    input: &Path,
    page_number: Option<u32>,
    degrees: i64,
    output: &Path,
    overwrite: bool,
) -> Result<PathBuf> {
    ensure_distinct(&[input], output)?;
    let mut doc = load(input)?;
    let normalized = degrees.rem_euclid(360);
    if normalized % 90 != 0 {
        return Err(EngineError::Pdf(
            "a rotación ten que ser un múltiplo de 90 graos".into(),
        ));
    }

    let pages = doc.get_pages();
    for (num, page_id) in pages {
        if page_number.is_some() && page_number != Some(num) {
            continue;
        }
        if let Ok(dict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
            let current = dict.get(b"Rotate").and_then(|o| o.as_i64()).unwrap_or(0);
            let new_rotation = (current + normalized).rem_euclid(360);
            dict.set("Rotate", lopdf::Object::Integer(new_rotation));
        }
    }

    save_atomically(&mut doc, output, overwrite)
}

pub fn page_count(input: &Path) -> Result<u32> {
    let doc = load(input)?;
    Ok(doc.get_pages().len() as u32)
}

// Xeometría dunha páxina A4 en puntos (1/72"), coa que se compón un PDF de
// texto plano cando se converte .txt -> .pdf.
const PAGE_WIDTH: f64 = 595.0;
const PAGE_HEIGHT: f64 = 842.0;
const MARGIN: f64 = 50.0;
const FONT_SIZE: f64 = 10.0;
const LINE_HEIGHT: f64 = 14.0;
const CHARS_PER_LINE: usize = 90;

/// Escapa unha liña para un literal PDF `( ... )`, codificando en
/// WinAnsiEncoding (cobre ASCII e as vogais acentuadas/ñ/ü do galego).
/// Calquera outro carácter substitúese por '?' para non xerar bytes que
/// corrompesen o PDF.
fn escape_pdf_line(line: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(line.len() + 4);
    for ch in line.chars() {
        let byte = if (ch as u32) < 256 { ch as u32 as u8 } else { b'?' };
        if matches!(byte, b'(' | b')' | b'\\') {
            out.push(b'\\');
        }
        out.push(byte);
    }
    out
}

/// Envolve unha liña de texto en varias, ningunha máis longa que
/// `max_chars`, rompendo por palabras cando é posible (e por caracteres
/// cando unha palabra soa xa supera o límite).
fn wrap_line(line: &str, max_chars: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }
    let max_chars = max_chars.max(1);
    let mut out = Vec::new();
    let mut current = String::new();
    for word in line.split(' ') {
        let mut remaining = word;
        loop {
            let sep_len = usize::from(!current.is_empty());
            let fits = current.chars().count() + sep_len + remaining.chars().count() <= max_chars;
            if fits {
                if sep_len == 1 {
                    current.push(' ');
                }
                current.push_str(remaining);
                break;
            }
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
                continue;
            }
            // `current` está baleira e `remaining` por si só xa supera o
            // límite: hai que partila por caracteres.
            let head: String = remaining.chars().take(max_chars).collect();
            let head_chars = head.chars().count();
            let byte_offset = remaining
                .char_indices()
                .nth(head_chars)
                .map(|(i, _)| i)
                .unwrap_or(remaining.len());
            out.push(head);
            remaining = &remaining[byte_offset..];
            if remaining.is_empty() {
                break;
            }
        }
    }
    out.push(current);
    out
}

/// Convirte un ficheiro de texto plano (UTF-8) nun PDF sinxelo (unha fonte
/// estándar Courier, sen incrustar, paxinado automaticamente). Pensado para
/// notas ou texto simple, non para reproducir formato de documentos ricos
/// (Word/ODT quedan para unha fase posterior con motor de renderizado).
pub fn text_to_pdf(input: &Path, output: &Path, overwrite: bool) -> Result<PathBuf> {
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    let text = std::fs::read_to_string(input)
        .map_err(|e| EngineError::Pdf(format!("non se puido ler o ficheiro de texto: {e}")))?;

    let mut wrapped_lines: Vec<String> = Vec::new();
    for raw_line in text.lines() {
        wrapped_lines.extend(wrap_line(raw_line, CHARS_PER_LINE));
    }
    if wrapped_lines.is_empty() {
        wrapped_lines.push(String::new());
    }

    let lines_per_page = (((PAGE_HEIGHT - 2.0 * MARGIN) / LINE_HEIGHT) as usize).max(1);
    let mut doc = Document::with_version("1.5");

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
        "Encoding" => "WinAnsiEncoding",
    });

    let mut page_ids: Vec<ObjectId> = Vec::new();
    for chunk in wrapped_lines.chunks(lines_per_page) {
        let mut content = Vec::new();
        content.extend_from_slice(
            format!(
                "BT /F1 {FONT_SIZE} Tf {LINE_HEIGHT} TL {MARGIN} {} Td\n",
                PAGE_HEIGHT - MARGIN
            )
            .as_bytes(),
        );
        for (i, line) in chunk.iter().enumerate() {
            if i > 0 {
                content.extend_from_slice(b"T*\n");
            }
            content.push(b'(');
            content.extend(escape_pdf_line(line));
            content.extend_from_slice(b") Tj\n");
        }
        content.extend_from_slice(b"ET");

        let content_id = doc.add_object(Stream::new(dictionary! {}, content));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), PAGE_WIDTH.into(), PAGE_HEIGHT.into()],
            "Contents" => content_id,
            "Resources" => dictionary! {
                "Font" => dictionary! { "F1" => font_id },
            },
        });
        page_ids.push(page_id);
    }

    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.iter().map(|id| Object::Reference(*id)).collect::<Vec<_>>(),
        "Count" => page_ids.len() as i64,
    });
    for page_id in &page_ids {
        if let Ok(dict) = doc.get_object_mut(*page_id).and_then(|o| o.as_dict_mut()) {
            dict.set("Parent", Object::Reference(pages_id));
        }
    }
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    save_atomically(&mut doc, output, overwrite)
}

/// Extrae o texto de tódalas páxinas dun PDF a un ficheiro `.txt` (UTF-8).
/// Non intenta recompor a maquetación orixinal, só o contido textual.
pub fn pdf_to_text(input: &Path, output: &Path, overwrite: bool) -> Result<PathBuf> {
    let doc = load(input)?;
    let page_numbers: Vec<u32> = doc.get_pages().into_keys().collect();
    let text = doc
        .extract_text(&page_numbers)
        .map_err(|e| EngineError::Pdf(format!("non se puido extraer o texto: {e}")))?;

    let mut out = AtomicOutput::new(output, overwrite)?;
    out.write_all(text.as_bytes())?;
    out.commit()?;
    Ok(output.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Object};

    /// Crea un PDF de proba con `n` páxinas. O MediaBox só se define na
    /// árbore de páxinas (Pages), non en cada páxina, a propósito: así se
    /// comproba que `flatten_inherited_page_attributes` fai o seu traballo.
    fn fixture_pdf(path: &Path, n: u32) {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();

        let mut kids = Vec::new();
        for _ in 0..n {
            let page_id = doc.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
            });
            kids.push(Object::Reference(page_id));
        }

        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => n as i64,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        doc.save(path).unwrap();
    }

    #[test]
    fn conta_paxinas_correctamente() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.pdf");
        fixture_pdf(&a, 3);
        assert_eq!(page_count(&a).unwrap(), 3);
    }

    #[test]
    fn unir_pdfs_suma_paxinas_e_herda_mediabox() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.pdf");
        let b = dir.path().join("b.pdf");
        fixture_pdf(&a, 2);
        fixture_pdf(&b, 3);

        let output = dir.path().join("unido.pdf");
        merge_pdfs(&[a, b], &output, false).unwrap();

        assert_eq!(page_count(&output).unwrap(), 5);

        let doc = Document::load(&output).unwrap();
        for (_, page_id) in doc.get_pages() {
            let dict = doc.get_object(page_id).unwrap().as_dict().unwrap();
            assert!(
                dict.has(b"MediaBox"),
                "a páxina tería que ter herdado o MediaBox antes de unir"
            );
        }
    }

    #[test]
    fn extraer_paxinas_selecciona_e_reordena() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.pdf");
        fixture_pdf(&input, 4);

        let output = dir.path().join("extracto.pdf");
        extract_pages(&input, &[3, 1], &output, false).unwrap();
        assert_eq!(page_count(&output).unwrap(), 2);
    }

    #[test]
    fn extraer_paxina_inexistente_da_erro() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.pdf");
        fixture_pdf(&input, 2);

        let output = dir.path().join("extracto.pdf");
        let err = extract_pages(&input, &[5], &output, false).unwrap_err();
        assert!(matches!(err, EngineError::Pdf(_)));
    }

    #[test]
    fn rotar_todas_as_paxinas() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.pdf");
        fixture_pdf(&input, 2);

        let output = dir.path().join("rotado.pdf");
        rotate_pages(&input, None, 90, &output, false).unwrap();

        let doc = Document::load(&output).unwrap();
        for (_, page_id) in doc.get_pages() {
            let dict = doc.get_object(page_id).unwrap().as_dict().unwrap();
            assert_eq!(dict.get(b"Rotate").unwrap().as_i64().unwrap(), 90);
        }
    }

    #[test]
    fn rotar_180_dous_veces_volve_a_0() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("orixe.pdf");
        fixture_pdf(&input, 1);

        let paso1 = dir.path().join("paso1.pdf");
        rotate_pages(&input, None, 180, &paso1, false).unwrap();
        let paso2 = dir.path().join("paso2.pdf");
        rotate_pages(&paso1, None, 180, &paso2, false).unwrap();

        let doc = Document::load(&paso2).unwrap();
        for (_, page_id) in doc.get_pages() {
            let dict = doc.get_object(page_id).unwrap().as_dict().unwrap();
            assert_eq!(dict.get(b"Rotate").unwrap().as_i64().unwrap(), 0);
        }
    }

    #[test]
    fn wrap_line_respecta_o_limite_e_non_perde_texto() {
        let orixinal = "unha liña bastante longa que hai que envolver en varias partes máis curtas";
        let partes = wrap_line(orixinal, 20);
        assert!(partes.iter().all(|p| p.chars().count() <= 20));
        assert_eq!(partes.join(" "), orixinal);
    }

    #[test]
    fn wrap_line_parte_palabras_moi_longas() {
        let partes = wrap_line("supercalifragilisticoexpialidoso", 10);
        assert!(partes.iter().all(|p| p.chars().count() <= 10));
        assert_eq!(partes.concat(), "supercalifragilisticoexpialidoso");
    }

    #[test]
    fn texto_a_pdf_e_volta_conserva_o_contido() {
        let dir = tempfile::tempdir().unwrap();
        let txt = dir.path().join("nota.txt");
        std::fs::write(&txt, "Ola trasno!\nSegunda liña con ñ e á.\n").unwrap();

        let pdf = dir.path().join("nota.pdf");
        text_to_pdf(&txt, &pdf, false).unwrap();
        assert_eq!(page_count(&pdf).unwrap(), 1);

        let extraido = dir.path().join("volta.txt");
        pdf_to_text(&pdf, &extraido, false).unwrap();
        let contido = std::fs::read_to_string(&extraido).unwrap();
        assert!(contido.contains("Ola trasno!"));
        assert!(contido.contains("Segunda liña con"));
    }

    #[test]
    fn texto_a_pdf_paxina_cando_hai_moitas_liñas() {
        let dir = tempfile::tempdir().unwrap();
        let txt = dir.path().join("longo.txt");
        let contido = "liña\n".repeat(200);
        std::fs::write(&txt, contido).unwrap();

        let pdf = dir.path().join("longo.pdf");
        text_to_pdf(&txt, &pdf, false).unwrap();
        assert!(page_count(&pdf).unwrap() > 1, "200 liñas teñen que ocupar máis dunha páxina");
    }
}
