//! Subtítulos como ficheiros da app: categoría, conversión polo conversor xeral, UTF-8 e resincronización de ficheiros reais.

use trasno_engine::subtitles::{retime_subtitle, subtitle_info, Retime};
use trasno_engine::{convert_file, formats, suggested_targets, Category, ConvertOptions};

const SRT: &str = "1\n00:00:01,000 --> 00:00:03,500\nOlá, mundo!\n\n2\n00:00:05,250 --> 00:00:07,000\nNiño, ñoño\n\n3\n01:00:00,000 --> 01:00:02,000\nÚltima\n";

#[test]
fn os_subtitulos_son_documentos_e_ofrecen_srt_vtt_e_ass() {
    for e in ["srt", "vtt", "ass", "ssa", "SRT"] {
        assert_eq!(formats::category_of_extension(e), Category::Documento, "{e}");
    }
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("a.srt");
    std::fs::write(&p, SRT).unwrap();
    assert_eq!(suggested_targets(&p), vec!["srt", "vtt", "ass"]);
}

#[test]
fn o_conversor_xeral_pasa_entre_formatos_e_garda_en_utf8() {
    let dir = tempfile::tempdir().unwrap();
    // Uns subtítulos antigos en Windows-1252 («Niño» co ñ = 0xF1).
    let latin: Vec<u8> = SRT.chars().map(|c| match c { 'ñ' => 0xF1, 'á' => 0xE1, 'Ú' => 0xDA, 'O' => b'O', 'l' => b'l', c if (c as u32) < 128 => c as u8, _ => b'?' }).collect();
    let src = dir.path().join("vella.srt");
    std::fs::write(&src, &latin).unwrap();
    assert!(subtitle_info(&src).unwrap().encoding == "Windows-1252");

    for to in ["vtt", "ass", "srt"] {
        let out = dir.path().join(format!("nova.{to}"));
        let r = convert_file(&src, &out, &ConvertOptions::default()).unwrap();
        assert!(r.bytes_written > 0);
        let text = std::fs::read_to_string(&out).expect("sae en UTF-8 válido");
        assert!(text.contains("Niño, ñoño") && text.contains("Olá, mundo!") && text.contains("Última"), "{to}: {text}");
        assert!(!text.starts_with('\u{feff}'), "sen BOM");
        assert_eq!(subtitle_info(&out).unwrap().cues, 3, "{to}");
        assert_eq!(subtitle_info(&out).unwrap().encoding, "UTF-8");
    }
    // O orixinal non se tocou.
    assert_eq!(std::fs::read(&src).unwrap(), latin);
    // A VTT non se converte a algo que non son subtítulos.
    assert!(convert_file(&src, &dir.path().join("x.docx"), &ConvertOptions::default()).is_err());
}

#[test]
fn resincronizar_un_ficheiro_conserva_o_formato_e_o_texto() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("a.srt");
    std::fs::write(&src, SRT).unwrap();
    let out = dir.path().join("b.srt");
    let r = retime_subtitle(&src, &out, false, &Retime { shift_ms: 1500, factor: 1.0 }).unwrap();
    assert_eq!((r.cues, r.dropped), (3, 0));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("00:00:02,500 --> 00:00:05,000\nOlá, mundo!"), "{text}");
    assert!(text.contains("01:00:01,500 --> 01:00:03,500\nÚltima"), "{text}");
    // E en ASS con estilos propios non se perden.
    let ass = dir.path().join("e.ass");
    std::fs::write(&ass, "[Script Info]\nTitle: x\n\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Rotulo,Impact\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:02.00,Rotulo,,0,0,0,,Ola\n").unwrap();
    let out2 = dir.path().join("e2.ass");
    retime_subtitle(&ass, &out2, false, &Retime { shift_ms: 1000, factor: 1.0 }).unwrap();
    let t2 = std::fs::read_to_string(&out2).unwrap();
    assert!(t2.contains("Style: Rotulo,Impact") && t2.contains("Dialogue: 0,0:00:02.00,0:00:03.00,Rotulo,,0,0,0,,Ola"), "{t2}");
}
