//! Metadatos das fotos (EXIF: cámara, data, ubicación…) e perfil de cor ao converter ou comprimir.
//!
//! Ao recodificar unha imaxe os metadatos non pasan sós ao resultado: por defecto saen sen eles (o que xa protexe a privacidade).
//! Aquí lese o que leva o orixinal e, se o usuario o pide, escríbese no resultado: sen a ubicación (GPS) ou completo.
//! O perfil de cor (ICC) consérvase sempre que sexa un perfil RGB: sen el, as fotos de gama ampla (Display P3, Adobe RGB) cambian de cor.
//! A orientación EXIF aplícase aos píxeles ao abrir (as fotos do móbil gárdanse «deitadas» con unha etiqueta que di como xiralas); no EXIF
//! que se garda a orientación queda a 1, para que a foto non se volva xirar.

use crate::error::Result;
use image::{DynamicImage, ImageDecoder, ImageEncoder};
use std::path::Path;

/// Que facer cos metadatos do orixinal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metadata {
    /// Non levar ningún (só o perfil de cor). É o por defecto.
    #[default]
    Strip,
    /// Levar os da cámara e a data, pero non a ubicación.
    NoLocation,
    /// Levar todo (incluída a ubicación).
    All,
}

/// O que se leva do orixinal ao resultado.
#[derive(Debug, Clone, Default)]
pub struct Carried {
    pub icc: Option<Vec<u8>>,
    pub exif: Option<Vec<u8>>,
}

impl Carried {
    pub fn is_empty(&self) -> bool {
        self.icc.is_none() && self.exif.is_none()
    }

    /// Pono no codificador (os formatos que non os admiten simplemente non os levan).
    pub fn apply<E: ImageEncoder>(&self, enc: &mut E) {
        if let Some(icc) = &self.icc {
            let _ = enc.set_icc_profile(icc.clone());
        }
        if let Some(exif) = &self.exif {
            let _ = enc.set_exif_metadata(exif.clone());
        }
    }
}

/// Abre a imaxe cos píxeles xa na orientación correcta e colle os metadatos que toque levar.
pub fn open_oriented(path: &Path, mode: Metadata) -> Result<(DynamicImage, Carried)> {
    let readable = crate::image_ffmpeg::readable(path)?;
    let mut dec = image::ImageReader::open(&readable)?.with_guessed_format()?.into_decoder()?;
    let orientation = dec.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let icc = dec.icc_profile().ok().flatten();
    let exif = if mode == Metadata::Strip { None } else { dec.exif_metadata().ok().flatten() };
    let mut img = DynamicImage::from_decoder(dec)?;
    img.apply_orientation(orientation);
    let carried = Carried {
        icc: icc.filter(|p| usable_icc(p, &img)),
        exif: exif.and_then(|e| clean_exif(&e, mode)),
    };
    Ok((img, carried))
}

/// Le o perfil e o EXIF que xa leva un ficheiro (sen decodificar os píxeles).
pub fn read_carried(path: &Path) -> Carried {
    let Ok(reader) = image::ImageReader::open(path).and_then(|r| r.with_guessed_format()) else { return Carried::default() };
    let Ok(mut dec) = reader.into_decoder() else { return Carried::default() };
    Carried { icc: dec.icc_profile().ok().flatten(), exif: dec.exif_metadata().ok().flatten() }
}

/// Mete o perfil de cor e o EXIF nun WebP que xa está feito (os que escribe FFmpeg, con perdas, non os levan). Un WebP con metadatos é
/// un contedor RIFF cunha cabeceira «VP8X» que di que chunks hai (perfil antes da imaxe, EXIF despois). Se algo non cadra, devolve o
/// ficheiro tal cal: ao pior, sae sen metadatos, nunca roto.
pub fn add_to_webp(webp: Vec<u8>, carried: &Carried) -> Vec<u8> {
    try_add_to_webp(&webp, carried).unwrap_or(webp)
}

fn try_add_to_webp(webp: &[u8], carried: &Carried) -> Option<Vec<u8>> {
    if carried.is_empty() || webp.len() < 20 || &webp[0..4] != b"RIFF" || &webp[8..12] != b"WEBP" {
        return None;
    }
    let le32 = |b: &[u8]| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize;
    let mut chunks: Vec<(&[u8], &[u8])> = Vec::new();
    let mut pos = 12;
    while pos + 8 <= webp.len() {
        let size = le32(&webp[pos + 4..]);
        let data = webp.get(pos + 8..pos + 8 + size)?;
        chunks.push((&webp[pos..pos + 4], data));
        pos += 8 + size + (size & 1);
    }
    // Animados (ANIM) ou con metadatos propios: non se tocan.
    if chunks.iter().any(|(id, _)| matches!(*id, b"ANIM" | b"ANMF" | b"ICCP" | b"EXIF")) || chunks.is_empty() {
        return None;
    }
    let dec = image::ImageReader::new(std::io::Cursor::new(webp)).with_guessed_format().ok()?.into_decoder().ok()?;
    let (w, h) = dec.dimensions();
    let mut flags = 0u8;
    if chunks.iter().any(|(id, _)| *id == b"ALPH") || dec.color_type().has_alpha() {
        flags |= 0x10;
    }
    if carried.icc.is_some() {
        flags |= 0x20;
    }
    if carried.exif.is_some() {
        flags |= 0x08;
    }
    let mut vp8x = [0u8; 10];
    vp8x[0] = flags;
    vp8x[4..7].copy_from_slice(&(w - 1).to_le_bytes()[..3]);
    vp8x[7..10].copy_from_slice(&(h - 1).to_le_bytes()[..3]);

    let mut body: Vec<u8> = b"WEBP".to_vec();
    let mut put = |id: &[u8], data: &[u8]| {
        body.extend_from_slice(id);
        body.extend_from_slice(&(data.len() as u32).to_le_bytes());
        body.extend_from_slice(data);
        if data.len() & 1 == 1 {
            body.push(0);
        }
    };
    put(b"VP8X", &vp8x);
    if let Some(icc) = &carried.icc {
        put(b"ICCP", icc);
    }
    for (id, data) in chunks.iter().filter(|(id, _)| *id != b"VP8X") {
        put(id, data);
    }
    if let Some(exif) = &carried.exif {
        put(b"EXIF", exif);
    }
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    Some(out)
}

/// Como `add_to_webp`, sobre un ficheiro: escribe nun temporal ao lado e substitúeo, para que nunca quede a medias.
pub fn add_to_webp_file(path: &Path, carried: &Carried) -> Result<()> {
    if carried.is_empty() {
        return Ok(());
    }
    let bytes = std::fs::read(path)?;
    let patched = add_to_webp(bytes.clone(), carried);
    if patched == bytes {
        return Ok(());
    }
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::Builder::new().prefix(".trasno-").suffix(".part.webp").tempfile_in(dir)?;
    std::io::Write::write_all(&mut tmp, &patched)?;
    // Comproba que o resultado le ben antes de substituír nada.
    if image::open(tmp.path()).is_err() {
        return Ok(());
    }
    crate::integrity::persist_retry(tmp, path).map_err(crate::error::EngineError::Io)?;
    Ok(())
}

/// Un perfil só vale se é RGB e a imaxe ten cor: un perfil de CMYK ou de grises sobre píxeles RGB deixaría a foto mal.
fn usable_icc(profile: &[u8], img: &DynamicImage) -> bool {
    profile.len() >= 128 && &profile[16..20] == b"RGB " && img.color().channel_count() >= 3
}

/// Reescribe o EXIF para levalo ao resultado: sen ubicación (se se pide), sen miniatura (mostraría a foto sen recortar e sen xirar),
/// sen as notas do fabricante (os seus enderezos internos rompen ao mover o bloque, e poden levar números de serie), sen os
/// números de serie nin o dono da cámara (salvo con «todo»), coa
/// orientación a 1 e sen as medidas do orixinal (xa non valen se a foto cambiou de tamaño).
fn clean_exif(raw: &[u8], mode: Metadata) -> Option<Vec<u8>> {
    use exif::{Context, Field, In, Tag, Value};
    let parsed = exif::Reader::new().read_raw(raw.to_vec()).ok()?;
    let mut keep: Vec<Field> = Vec::new();
    for f in parsed.fields() {
        if f.ifd_num != In::PRIMARY {
            continue;
        }
        let Tag(ctx, _) = f.tag;
        let drop = matches!(f.value, Value::Unknown(..))
            || (ctx == Context::Gps && mode != Metadata::All)
            || matches!(
                f.tag,
                Tag::MakerNote | Tag::PixelXDimension | Tag::PixelYDimension | Tag::ImageWidth | Tag::ImageLength | Tag::XResolution | Tag::YResolution | Tag::ResolutionUnit
            );
        // Sen «todo»: tampouco o que identifica a cámara ou a quen a leva (números de serie, dono, autor).
        let identifies = mode != Metadata::All
            && matches!(f.tag, Tag::BodySerialNumber | Tag::LensSerialNumber | Tag::CameraOwnerName | Tag::ImageUniqueID | Tag::Artist);
        if drop || identifies {
            continue;
        }
        if f.tag == Tag::Orientation {
            keep.push(Field { tag: Tag::Orientation, ifd_num: In::PRIMARY, value: Value::Short(vec![1]) });
        } else {
            keep.push(f.clone());
        }
    }
    if keep.is_empty() {
        return None;
    }
    let mut w = exif::experimental::Writer::new();
    for f in &keep {
        w.push_field(f);
    }
    let mut buf = std::io::Cursor::new(Vec::new());
    w.write(&mut buf, parsed.little_endian()).ok()?;
    Some(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EXIF de proba con cámara, data, ubicación, orientación e unha nota do fabricante.
    pub fn sample_exif(orientation: u16) -> Vec<u8> {
        use exif::{Field, In, Tag, Value};
        let fields = vec![
            Field { tag: Tag::Make, ifd_num: In::PRIMARY, value: Value::Ascii(vec![b"Trasno".to_vec()]) },
            Field { tag: Tag::Model, ifd_num: In::PRIMARY, value: Value::Ascii(vec![b"Proba 1".to_vec()]) },
            Field { tag: Tag::Orientation, ifd_num: In::PRIMARY, value: Value::Short(vec![orientation]) },
            Field { tag: Tag::DateTimeOriginal, ifd_num: In::PRIMARY, value: Value::Ascii(vec![b"2024:05:06 07:08:09".to_vec()]) },
            Field { tag: Tag::GPSLatitudeRef, ifd_num: In::PRIMARY, value: Value::Ascii(vec![b"N".to_vec()]) },
            Field {
                tag: Tag::GPSLatitude,
                ifd_num: In::PRIMARY,
                value: Value::Rational(vec![exif::Rational { num: 42, denom: 1 }, exif::Rational { num: 52, denom: 1 }, exif::Rational { num: 0, denom: 1 }]),
            },
            Field { tag: Tag::PixelXDimension, ifd_num: In::PRIMARY, value: Value::Long(vec![999]) },
            Field { tag: Tag::BodySerialNumber, ifd_num: In::PRIMARY, value: Value::Ascii(vec![b"SERIE-1".to_vec()]) },
        ];
        let mut w = exif::experimental::Writer::new();
        for f in &fields {
            w.push_field(f);
        }
        let mut buf = std::io::Cursor::new(Vec::new());
        w.write(&mut buf, true).unwrap();
        buf.into_inner()
    }

    fn tags(raw: &[u8]) -> Vec<String> {
        let e = exif::Reader::new().read_raw(raw.to_vec()).unwrap();
        e.fields().map(|f| f.tag.to_string()).collect()
    }

    #[test]
    fn o_exif_limpase_segundo_o_modo() {
        let raw = sample_exif(6);
        let none_gps = clean_exif(&raw, Metadata::NoLocation).unwrap();
        let t = tags(&none_gps);
        assert!(t.iter().any(|x| x == "Make") && t.iter().any(|x| x == "DateTimeOriginal"), "{t:?}");
        assert!(!t.iter().any(|x| x.starts_with("GPS")), "sen ubicación: {t:?}");
        assert!(!t.iter().any(|x| x == "PixelXDimension"), "as medidas do orixinal non se levan: {t:?}");
        assert!(!t.iter().any(|x| x == "BodySerialNumber"), "o número de serie identifica a cámara: {t:?}");
        let e = exif::Reader::new().read_raw(none_gps).unwrap();
        let o = e.get_field(exif::Tag::Orientation, exif::In::PRIMARY).unwrap();
        assert_eq!(o.value.get_uint(0), Some(1), "a orientación queda a 1 porque os píxeles xa están xirados");

        let all = clean_exif(&raw, Metadata::All).unwrap();
        assert!(tags(&all).iter().any(|x| x.starts_with("GPS")), "con «todo» vai a ubicación");
        assert!(tags(&all).iter().any(|x| x == "BodySerialNumber"), "con «todo» vai todo");
    }

    #[test]
    fn un_exif_roto_ou_baleiro_non_se_leva() {
        assert!(clean_exif(b"isto non e exif", Metadata::All).is_none());
        assert!(clean_exif(&[], Metadata::NoLocation).is_none());
    }
}
