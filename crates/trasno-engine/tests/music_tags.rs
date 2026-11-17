//! Etiquetas de música sobre ficheiros reais de cada formato: garda, le de novo, non toca o orixinal e deixa o son igual.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use trasno_engine::ffmpeg;
use trasno_engine::music_tags::{extract_cover, read_tags, write_tags, CoverAction, Tags};

fn ff(args: &[&str]) {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-y"]).args(args).output().unwrap();
    assert!(o.status.success(), "ffmpeg: {}", String::from_utf8_lossy(&o.stderr));
}

fn sha(p: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(p).unwrap()))
}

fn image(dir: &Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    ff(&["-f", "lavfi", "-i", "testsrc=s=200x200:d=1", "-frames:v", "1", p.to_str().unwrap()]);
    p
}

/// Son de 3 s no formato pedido, con unhas etiquetas de partida (e portada se o formato a leva).
fn song(dir: &Path, ext: &str, cover: Option<&Path>) -> PathBuf {
    let p = dir.join(format!("cancion.{ext}"));
    let mut a: Vec<&str> = vec!["-f", "lavfi", "-i", "sine=f=330:r=44100:d=3"];
    if let Some(c) = cover {
        a.extend(["-i", c.to_str().unwrap(), "-map", "0:a", "-map", "1:v", "-c:v", "mjpeg", "-disposition:v", "attached_pic"]);
    }
    a.extend(["-metadata", "title=Orixinal", "-metadata", "artist=Alguén", p.to_str().unwrap()]);
    ff(&a);
    p
}

fn decodes_clean(p: &Path) -> bool {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-i", p.to_str().unwrap(), "-f", "null", "-"]).output().unwrap();
    o.status.success() && o.stderr.is_empty()
}

fn full() -> Tags {
    Tags {
        title: "A Rula dos Trasnos".into(),
        artist: "Os Cañotas".into(),
        album: "Ñoñerías Vol. 2".into(),
        album_artist: "Varios".into(),
        composer: "M. Pérez".into(),
        genre: "Folk".into(),
        comment: "Gravado en Lugo".into(),
        lyrics: "Primeira liña\nSegunda liña con acentos: áéíóú ñ\n".into(),
        year: Some(2024),
        track: Some(3),
        track_total: Some(12),
        disc: Some(1),
        disc_total: Some(2),
    }
}

const FORMATS: [&str; 8] = ["mp3", "flac", "ogg", "opus", "m4a", "wav", "aiff", "aac"];

#[test]
fn garda_le_e_non_toca_nin_o_orixinal_nin_o_son() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for ext in FORMATS {
        let src = song(dir.path(), ext, None);
        let before = sha(&src);
        let dur0 = read_tags(&src).unwrap().duration;
        let out = dir.path().join(format!("nova.{ext}"));
        let r = write_tags(&src, &out, false, &full(), &CoverAction::Keep).unwrap_or_else(|e| panic!("{ext}: {e}"));
        assert_eq!(sha(&src), before, "{ext}: o orixinal cambiou");
        assert!(decodes_clean(&out), "{ext}: non decodifica limpo");
        let back = read_tags(&out).unwrap();
        assert!((back.duration - dur0).abs() < 0.1, "{ext}: dura {} e antes {dur0}", back.duration);
        // O que o formato non garda ven avisado; o resto ten que estar exacto.
        for (name, ok) in [("title", back.tags.title == full().title), ("artist", back.tags.artist == full().artist), ("album", back.tags.album == full().album)] {
            assert!(ok, "{ext}: {name} non coincide: {:?}", back.tags);
        }
        let want = full();
        let pairs = [
            ("album_artist", back.tags.album_artist == want.album_artist),
            ("composer", back.tags.composer == want.composer),
            ("genre", back.tags.genre == want.genre),
            ("comment", back.tags.comment == want.comment),
            ("lyrics", back.tags.lyrics == want.lyrics.trim()),
            ("year", back.tags.year == want.year),
            ("track", back.tags.track == want.track),
            ("track_total", back.tags.track_total == want.track_total),
            ("disc", back.tags.disc == want.disc),
            ("disc_total", back.tags.disc_total == want.disc_total),
        ];
        for (name, ok) in pairs {
            assert!(ok || r.not_saved.iter().any(|n| n == name), "{ext}: {name} nin quedou nin se avisou: {:?} / {:?}", back.tags, r.not_saved);
        }
        println!("{ext}: avisados {:?}", r.not_saved);
        // Outros programas (ffprobe) len o mesmo título e artista.
        let info = ffmpeg::probe(&out).unwrap();
        let stream_tags = &info["streams"][0]["tags"];
        let tags = if info["format"]["tags"].is_null() { stream_tags } else { &info["format"]["tags"] };
        let get = |k: &str| tags.as_object().and_then(|m| m.iter().find(|(n, _)| n.eq_ignore_ascii_case(k)).map(|(_, v)| v.as_str().unwrap_or("").to_string()));
        assert_eq!(get("title").as_deref(), Some("A Rula dos Trasnos"), "{ext}: ffprobe ve {tags:?}");
        assert_eq!(get("artist").as_deref(), Some("Os Cañotas"), "{ext}");
    }
}

#[test]
fn buscar_baleirar_campos_borra_as_etiquetas() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for ext in ["mp3", "flac", "m4a", "ogg"] {
        let src = song(dir.path(), ext, None);
        let a = dir.path().join(format!("a.{ext}"));
        write_tags(&src, &a, false, &full(), &CoverAction::Keep).unwrap();
        let b = dir.path().join(format!("b.{ext}"));
        write_tags(&a, &b, false, &Tags { title: "Só o título".into(), ..Default::default() }, &CoverAction::Keep).unwrap();
        let t = read_tags(&b).unwrap().tags;
        assert_eq!(t.title, "Só o título", "{ext}");
        assert!(t.artist.is_empty() && t.album.is_empty() && t.lyrics.is_empty() && t.year.is_none() && t.track.is_none() && t.disc_total.is_none(), "{ext}: sobraron campos: {t:?}");
    }
}

#[test]
fn o_total_sen_numero_non_se_garda_e_o_cero_non_vale() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = song(dir.path(), "mp3", None);
    let out = dir.path().join("o.mp3");
    write_tags(&src, &out, false, &Tags { track: Some(0), track_total: Some(10), disc_total: Some(3), ..Default::default() }, &CoverAction::Keep).unwrap();
    let t = read_tags(&out).unwrap().tags;
    assert!(t.track.is_none() && t.track_total.is_none() && t.disc_total.is_none(), "{t:?}");
}

#[test]
fn portada_poñer_cambiar_e_quitar_en_varios_formatos_de_imaxe() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for ext in ["mp3", "flac", "m4a", "ogg", "opus", "wav"] {
        let src = song(dir.path(), ext, None);
        assert!(read_tags(&src).unwrap().cover.is_none(), "{ext}: debía partir sen portada");
        let mut at = src.clone();
        for img in ["c.png", "c.jpg", "c.webp", "c.bmp"] {
            let cover = image(dir.path(), img);
            let out = dir.path().join(format!("p-{img}.{ext}"));
            write_tags(&at, &out, true, &Tags { title: "T".into(), ..Default::default() }, &CoverAction::Replace(cover.to_string_lossy().into())).unwrap_or_else(|e| panic!("{ext} {img}: {e}"));
            assert!(decodes_clean(&out), "{ext} {img}: non decodifica limpo");
            let info = read_tags(&out).unwrap();
            let c = info.cover.unwrap_or_else(|| panic!("{ext} {img}: sen portada"));
            assert!(c.mime == "image/png" || c.mime == "image/jpeg", "{ext} {img}: {}", c.mime);
            // E vese: sácase a un ficheiro que é unha imaxe de verdade.
            let pic = dir.path().join(format!("sacada-{img}.{ext}.img"));
            extract_cover(&out, &pic).unwrap();
            assert!(image::load_from_memory(&std::fs::read(&pic).unwrap()).is_ok(), "{ext} {img}: a portada sacada non é unha imaxe");
            at = out;
        }
        let none = dir.path().join(format!("sen.{ext}"));
        write_tags(&at, &none, true, &Tags { title: "T".into(), ..Default::default() }, &CoverAction::Remove).unwrap();
        assert!(read_tags(&none).unwrap().cover.is_none(), "{ext}: seguía a portada");
        assert!(decodes_clean(&none), "{ext}");
    }
}

#[test]
fn manter_conserva_a_portada_que_xa_tiña_e_o_que_non_se_edita() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let cover = image(dir.path(), "c.jpg");
    for ext in ["mp3", "m4a", "flac"] {
        let src = song(dir.path(), ext, Some(&cover));
        assert!(read_tags(&src).unwrap().cover.is_some(), "{ext}: a mostra debía levar portada");
        let out = dir.path().join(format!("k.{ext}"));
        write_tags(&src, &out, false, &Tags { title: "Novo".into(), ..Default::default() }, &CoverAction::Keep).unwrap();
        assert!(read_tags(&out).unwrap().cover.is_some(), "{ext}: perdeuse a portada");
        assert!(decodes_clean(&out));
    }
    // Unha etiqueta propia (que a ferramenta non edita) sobrevive.
    let src = dir.path().join("p.mp3");
    ff(&["-f", "lavfi", "-i", "sine=f=330:r=44100:d=2", "-metadata", "title=X", "-metadata", "MINHA_ETIQUETA=conservar", src.to_str().unwrap()]);
    let out = dir.path().join("p2.mp3");
    write_tags(&src, &out, false, &Tags { title: "Y".into(), ..Default::default() }, &CoverAction::Keep).unwrap();
    let info = ffmpeg::probe(&out).unwrap();
    assert_eq!(info["format"]["tags"]["MINHA_ETIQUETA"], "conservar", "{:?}", info["format"]["tags"]);
}

#[test]
fn erros_claros_e_nada_a_medias() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = song(dir.path(), "mp3", None);
    let t = Tags { title: "T".into(), ..Default::default() };
    // Non machaca un ficheiro que xa existe, nin se escribe sobre o propio orixinal.
    let exists = dir.path().join("xa.mp3");
    std::fs::write(&exists, b"x").unwrap();
    assert!(write_tags(&src, &exists, false, &t, &CoverAction::Keep).is_err());
    assert_eq!(std::fs::read(&exists).unwrap(), b"x");
    assert!(write_tags(&src, &src, true, &t, &CoverAction::Keep).is_err());
    // Portadas que non valen: non saen ficheiros a medias.
    let out = dir.path().join("mala.mp3");
    let not_img = dir.path().join("texto.png");
    std::fs::write(&not_img, b"isto non e unha imaxe").unwrap();
    assert!(write_tags(&src, &out, false, &t, &CoverAction::Replace(not_img.to_string_lossy().into())).is_err());
    assert!(write_tags(&src, &out, false, &t, &CoverAction::Replace(dir.path().join("non-existe.png").to_string_lossy().into())).is_err());
    assert!(!out.exists());
    // Valores fóra de rango.
    assert!(write_tags(&src, &out, false, &Tags { year: Some(0), ..Default::default() }, &CoverAction::Keep).is_err());
    assert!(write_tags(&src, &out, false, &Tags { comment: "x".repeat(5000), ..Default::default() }, &CoverAction::Keep).is_err());
    // Un ficheiro que non é música.
    let txt = dir.path().join("a.mp3");
    std::fs::write(&txt, b"ola mundo").unwrap();
    assert!(write_tags(&txt, &out, false, &t, &CoverAction::Keep).is_err());
    // WMA (sen soporte): mensaxe clara, sen restos.
    let wma = dir.path().join("a.wma");
    ff(&["-f", "lavfi", "-i", "sine=f=330:d=1", wma.to_str().unwrap()]);
    let e = write_tags(&wma, &dir.path().join("o.wma"), false, &t, &CoverAction::Keep).unwrap_err().to_string();
    println!("wma: {e}");
    assert!(!dir.path().join("o.wma").exists());
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().contains(".trasno-")).collect();
    assert!(leftovers.is_empty(), "sobraron ficheiros temporais");
}
