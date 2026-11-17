//! Etiquetas de música (título, artista, álbum, portada…) de MP3, FLAC, OGG, Opus, M4A/MP4, WAV, AIFF e AAC.
//!
//! Sempre se traballa sobre unha copia: o ficheiro orixinal non se toca nin un byte. Só se reescriben as etiquetas; o son pasa tal cal
//! (non se recodifica), e antes de deixar o resultado compróbase que se le de novo, que as etiquetas son as pedidas e que dura igual.

use crate::error::{EngineError, Result};
use crate::integrity::ensure_distinct;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::{AudioFile, TaggedFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn bad<T>(m: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(m.into()))
}

/// Campos que se editan. O texto baleiro significa «sen valor» (bórrase da etiqueta).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tags {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub composer: String,
    pub genre: String,
    pub comment: String,
    pub lyrics: String,
    pub year: Option<u32>,
    pub track: Option<u32>,
    pub track_total: Option<u32>,
    pub disc: Option<u32>,
    pub disc_total: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverInfo {
    pub mime: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TagInfo {
    pub tags: Tags,
    pub cover: Option<CoverInfo>,
    /// Tipo de ficheiro e de etiqueta, p. ex. «MP3 · ID3v2».
    pub format: String,
    pub duration: f64,
}

/// Que facer coa portada.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "kind", content = "path", rename_all = "lowercase")]
pub enum CoverAction {
    #[default]
    Keep,
    Remove,
    Replace(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct TagReport {
    pub output: PathBuf,
    /// Campos que o formato non pode gardar (p. ex. o compositor nun WAV): non quedaron na etiqueta.
    pub not_saved: Vec<String>,
}

fn open(path: &Path) -> Result<TaggedFile> {
    if !path.is_file() {
        return Err(EngineError::InputNotFound(path.to_path_buf()));
    }
    let fail = || EngineError::IntegrityCheck("Non sei ler as etiquetas deste ficheiro. Funciono con MP3, FLAC, OGG, Opus, M4A, WAV, AIFF e AAC: se é outro formato, convérteo antes.".into());
    Probe::open(path).map_err(|_| fail())?.options(ParseOptions::new()).guess_file_type().map_err(|_| fail())?.read().map_err(|_| fail())
}

fn tag_of(t: &TaggedFile) -> Option<&Tag> {
    t.primary_tag().or_else(|| t.first_tag())
}

fn text(tag: &Tag, key: ItemKey) -> String {
    tag.get_string(key).unwrap_or_default().to_string()
}

fn tags_of(tag: &Tag) -> Tags {
    Tags {
        title: tag.title().map(|s| s.to_string()).unwrap_or_default(),
        artist: tag.artist().map(|s| s.to_string()).unwrap_or_default(),
        album: tag.album().map(|s| s.to_string()).unwrap_or_default(),
        album_artist: text(tag, ItemKey::AlbumArtist),
        composer: text(tag, ItemKey::Composer),
        genre: tag.genre().map(|s| s.to_string()).unwrap_or_default(),
        comment: tag.comment().map(|s| s.to_string()).unwrap_or_default(),
        lyrics: Some(text(tag, ItemKey::UnsyncLyrics)).filter(|l| !l.is_empty()).unwrap_or_else(|| text(tag, ItemKey::Lyrics)),
        year: tag.date().map(|d| d.year as u32),
        track: tag.track(),
        track_total: tag.track_total(),
        disc: tag.disk(),
        disc_total: tag.disk_total(),
    }
}

fn cover_of(tag: &Tag) -> Option<&Picture> {
    tag.get_picture_type(PictureType::CoverFront).or_else(|| tag.pictures().first())
}

fn mime_text(p: &Picture) -> String {
    p.mime_type().map(|m| m.to_string()).unwrap_or_else(|| "image/jpeg".into())
}

/// Le as etiquetas e di se hai portada.
pub fn read_tags(path: &Path) -> Result<TagInfo> {
    let t = open(path)?;
    let tag = tag_of(&t);
    Ok(TagInfo {
        tags: tag.map(tags_of).unwrap_or_default(),
        cover: tag.and_then(cover_of).map(|p| CoverInfo { mime: mime_text(p), bytes: p.data().len() }),
        format: format!("{:?}{}", t.file_type(), tag.map(|g| format!(" · {:?}", g.tag_type())).unwrap_or_default()),
        duration: t.properties().duration().as_secs_f64(),
    })
}

/// Garda a portada nun ficheiro de imaxe (para vela ou gardala).
pub fn extract_cover(path: &Path, output: &Path) -> Result<PathBuf> {
    let t = open(path)?;
    let Some(p) = tag_of(&t).and_then(cover_of) else { return bad("Este ficheiro non ten portada.") };
    if let Some(dir) = output.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(output, p.data())?;
    Ok(output.to_path_buf())
}

/// Máximo de tamaño dunha portada nova: unha imaxe maior só enche o ficheiro de música.
const MAX_COVER_BYTES: u64 = 20 * 1024 * 1024;

/// Carga unha imaxe como portada. Admítense JPEG e PNG tal cal; calquera outra (WebP, BMP, TIFF…) pásase a PNG para que todos os
/// reprodutores a lean.
fn load_cover(path: &Path) -> Result<Picture> {
    let meta = std::fs::metadata(path).map_err(|_| EngineError::InputNotFound(path.to_path_buf()))?;
    if meta.len() > MAX_COVER_BYTES {
        return bad("Esa imaxe pesa demasiado para ser unha portada (máis de 20 MB). Reduce o seu tamaño antes.");
    }
    let bytes = std::fs::read(path)?;
    let (data, mime) = match image::guess_format(&bytes) {
        Ok(image::ImageFormat::Jpeg) => (bytes, MimeType::Jpeg),
        Ok(image::ImageFormat::Png) => (bytes, MimeType::Png),
        Ok(_) => {
            let img = image::load_from_memory(&bytes).map_err(|_| EngineError::IntegrityCheck("Non puiden ler esa imaxe como portada.".into()))?;
            let mut out = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png).map_err(|_| EngineError::IntegrityCheck("Non puiden preparar esa imaxe como portada.".into()))?;
            (out, MimeType::Png)
        }
        Err(_) => return bad("Ese ficheiro non é unha imaxe que eu poida usar como portada."),
    };
    Ok(Picture::unchecked(data).pic_type(PictureType::CoverFront).mime_type(mime).build())
}

fn set_text(tag: &mut Tag, key: ItemKey, v: &str) {
    tag.remove_key(key);
    let v = v.trim();
    if !v.is_empty() {
        tag.insert_text(key, v.to_string());
    }
}

/// Número de pista/disco: as etiquetas non admiten un total sen o número, e 0 non é un valor.
fn clean_number(n: Option<u32>) -> Option<u32> {
    n.filter(|v| (1..=9999).contains(v))
}

fn validate(t: &Tags) -> Result<()> {
    if let Some(y) = t.year {
        if !(1..=9999).contains(&y) {
            return bad("O ano non é válido (de 1 a 9999).");
        }
    }
    if [&t.title, &t.artist, &t.album, &t.album_artist, &t.composer, &t.genre, &t.comment].iter().any(|v| v.chars().count() > 2000) {
        return bad("Algún campo é demasiado longo (máis de 2000 letras).");
    }
    if t.lyrics.chars().count() > 200_000 {
        return bad("A letra é demasiado longa (máis de 200 000 letras).");
    }
    Ok(())
}

fn apply(tag: &mut Tag, t: &Tags, cover: &CoverAction) -> Result<()> {
    set_text(tag, ItemKey::TrackTitle, &t.title);
    set_text(tag, ItemKey::TrackArtist, &t.artist);
    set_text(tag, ItemKey::AlbumTitle, &t.album);
    set_text(tag, ItemKey::AlbumArtist, &t.album_artist);
    set_text(tag, ItemKey::Composer, &t.composer);
    set_text(tag, ItemKey::Genre, &t.genre);
    set_text(tag, ItemKey::Comment, &t.comment);
    // A letra consérvase tal cal (con saltos de liña), só se quitan os espazos dos extremos.
    tag.remove_key(ItemKey::Lyrics); // M4A e Vorbis tamén o len coma «Lyrics»: queda só unha
    set_text(tag, ItemKey::UnsyncLyrics, &t.lyrics);
    match t.year {
        Some(y) => tag.set_date(lofty::tag::items::Timestamp { year: y as u16, ..Default::default() }),
        None => tag.remove_date(),
    }
    match clean_number(t.track) {
        Some(n) => tag.set_track(n),
        None => tag.remove_track(),
    }
    match clean_number(t.track).and(clean_number(t.track_total)) {
        Some(n) => tag.set_track_total(n),
        None => tag.remove_track_total(),
    }
    match clean_number(t.disc) {
        Some(n) => tag.set_disk(n),
        None => tag.remove_disk(),
    }
    match clean_number(t.disc).and(clean_number(t.disc_total)) {
        Some(n) => tag.set_disk_total(n),
        None => tag.remove_disk_total(),
    }
    match cover {
        CoverAction::Keep => {}
        CoverAction::Remove => {
            while tag.picture_count() > 0 {
                tag.remove_picture(0);
            }
        }
        CoverAction::Replace(path) => {
            let pic = load_cover(Path::new(path))?;
            while tag.picture_count() > 0 {
                tag.remove_picture(0);
            }
            tag.push_picture(pic);
        }
    }
    Ok(())
}

/// Garda unha copia de `input` en `output` coas etiquetas novas. O son non se toca.
pub fn write_tags(input: &Path, output: &Path, overwrite: bool, new: &Tags, cover: &CoverAction) -> Result<TagReport> {
    validate(new)?;
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    ensure_distinct(&[input], output)?;
    if output.exists() && !overwrite {
        return Err(EngineError::DestinationExists(output.to_path_buf()));
    }
    // Antes de copiar nada, comprobar que sei editar este ficheiro (e que a portada nova vale).
    let original = open(input)?;
    let before = original.properties().duration();
    if let CoverAction::Replace(p) = cover {
        load_cover(Path::new(p))?;
    }

    let dir = output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let ext = output.extension().and_then(|e| e.to_str()).unwrap_or("out");
    let part = dir.join(format!(".trasno-{}.part.{ext}", uuid::Uuid::new_v4()));
    let mut lost: Vec<String> = Vec::new();
    let result = (|| -> Result<()> {
        std::fs::copy(input, &part)?;
        let mut file = open(&part)?;
        let tag_type = file.primary_tag_type();
        if file.primary_tag().is_none() {
            file.insert_tag(Tag::new(tag_type));
        }
        let tag = file.primary_tag_mut().expect("acabo de crear a etiqueta");
        apply(tag, new, cover)?;
        tag.save_to_path(&part, WriteOptions::default()).map_err(|_| EngineError::IntegrityCheck("Non puiden gardar as etiquetas neste ficheiro.".into()))?;

        // Comprobación: lese de novo, ten as etiquetas pedidas e dura o mesmo.
        let back = open(&part)?;
        let got = tag_of(&back).map(tags_of).unwrap_or_default();
        let want = expected(new);
        match compare(&want, &got) {
            Ok(l) => lost = l,
            Err(()) => return bad("Gardei as etiquetas pero ao lelas de novo non coinciden co que pedías; descarteino para non deixarche un ficheiro dubidoso."),
        }
        if (back.properties().duration().as_secs_f64() - before.as_secs_f64()).abs() > 0.1 {
            return bad("O son cambiou ao gardar as etiquetas; descarteino para non deixarche un ficheiro roto.");
        }
        match cover {
            CoverAction::Remove if tag_of(&back).and_then(cover_of).is_some() => return bad("Non puiden quitar a portada; descarteino para non deixarche un ficheiro dubidoso."),
            CoverAction::Replace(_) if tag_of(&back).and_then(cover_of).is_none() => return bad("Non puiden meter a portada; descarteino para non deixarche un ficheiro dubidoso."),
            _ => {}
        }
        Ok(())
    })();
    if let Err(e) = result {
        let _ = std::fs::remove_file(&part);
        return Err(e);
    }
    std::fs::rename(&part, output).map_err(|e| {
        let _ = std::fs::remove_file(&part);
        EngineError::Io(e)
    })?;
    Ok(TagReport { output: output.to_path_buf(), not_saved: lost })
}

/// O que se agarda ler despois de gardar `t` (campos limpos como os deixa `apply`).
fn expected(t: &Tags) -> Tags {
    let trim = |s: &str| s.trim().to_string();
    Tags {
        title: trim(&t.title),
        artist: trim(&t.artist),
        album: trim(&t.album),
        album_artist: trim(&t.album_artist),
        composer: trim(&t.composer),
        genre: trim(&t.genre),
        comment: trim(&t.comment),
        lyrics: trim(&t.lyrics),
        year: t.year,
        track: clean_number(t.track),
        track_total: clean_number(t.track).and(clean_number(t.track_total)),
        disc: clean_number(t.disc),
        disc_total: clean_number(t.disc).and(clean_number(t.disc_total)),
    }
}

/// Compara o pedido co que se le: Ok(campos que o formato non gardou) ou Err se algún quedou con outro valor.
fn compare(want: &Tags, got: &Tags) -> std::result::Result<Vec<String>, ()> {
    let mut lost = Vec::new();
    let mut text = |name: &str, w: &str, g: &str| -> bool {
        if w == g {
            true
        } else if g.is_empty() {
            lost.push(name.to_string());
            true
        } else {
            false
        }
    };
    let ok = text("title", &want.title, &got.title)
        && text("artist", &want.artist, &got.artist)
        && text("album", &want.album, &got.album)
        && text("album_artist", &want.album_artist, &got.album_artist)
        && text("composer", &want.composer, &got.composer)
        && text("genre", &want.genre, &got.genre)
        && text("comment", &want.comment, &got.comment)
        && text("lyrics", &want.lyrics, &got.lyrics);
    if !ok {
        return Err(());
    }
    for (name, w, g) in [("year", want.year, got.year), ("track", want.track, got.track), ("track_total", want.track_total, got.track_total), ("disc", want.disc, got.disc), ("disc_total", want.disc_total, got.disc_total)] {
        if w == g {
            continue;
        }
        if g.is_none() {
            lost.push(name.to_string());
        } else {
            return Err(());
        }
    }
    Ok(lost)
}
