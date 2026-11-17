//! Normalización de volume en dúas pasadas e conservación da calidade do son ao aplicar efectos.

use std::path::{Path, PathBuf};
use trasno_engine::ffmpeg::{self, Loudness, MediaOp};

fn make(dir: &Path, name: &str, source: &str, extra: &[&str]) -> PathBuf {
    let out = dir.join(name);
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", source])
        .args(extra)
        .arg(&out)
        .status()
        .unwrap();
    assert!(st.success(), "non se puido crear {name}");
    out
}

/// Ruído rosa moi baixo (uns −36 LUFS), 6 s, estéreo.
const QUIET: &str = "anoisesrc=d=6:c=pink:a=0.02:r=44100,aformat=channel_layouts=stereo";

fn audio(path: &Path) -> serde_json::Value {
    let info = ffmpeg::probe(path).unwrap();
    info["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == "audio").unwrap().clone()
}

fn rate(path: &Path) -> u32 {
    audio(path)["sample_rate"].as_str().unwrap().parse().unwrap()
}

fn kbps(path: &Path) -> u32 {
    audio(path)["bit_rate"].as_str().unwrap().parse::<u32>().unwrap() / 1000
}

fn lufs(path: &Path) -> f64 {
    ffmpeg::measure_loudness(path, Loudness::default()).unwrap().lufs
}

#[test]
fn normalizar_chega_ao_obxectivo_en_cada_perfil() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = make(dir.path(), "baixo.wav", QUIET, &[]);
    let before = lufs(&src);
    assert!(before < -30.0, "a mostra ten que ser baixa, mide {before}");
    for (name, target) in [("streaming", -14.0), ("podcast", -16.0), ("emision", -23.0), ("personalizado", -19.5)] {
        let out = dir.path().join(format!("{name}.wav"));
        let r = ffmpeg::normalize_loudness(&src, &out, false, Loudness { lufs: target, peak: -1.0, range: 11.0 }, false).unwrap();
        let after = lufs(&r.output);
        assert!((after - target).abs() < 0.6, "{name}: pedíronse {target} LUFS e saíron {after}");
        assert!((r.before.lufs - before).abs() < 0.2, "o informe di como estaba o orixinal");
        assert_eq!(rate(&r.output), 44100, "a frecuencia de mostraxe non cambia");
    }
    // O orixinal non se toca.
    assert!((lufs(&src) - before).abs() < 0.01);
}

/// Volume medio (dB) dun anaco do ficheiro.
fn mean_db(path: &Path, from: &str, len: &str) -> f64 {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-hide_banner", "-nostats", "-ss", from, "-t", len, "-i"])
        .arg(path)
        .args(["-af", "volumedetect", "-f", "null", if cfg!(windows) { "NUL" } else { "/dev/null" }])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&o.stderr).to_string();
    let at = text.find("mean_volume:").expect("volumedetect") + 12;
    text[at..].split_whitespace().next().unwrap().parse().unwrap()
}

#[test]
fn normalizar_en_duas_pasadas_conserva_a_dinamica() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    // Catro segundos baixos e catro altos (26 dB de diferenza). Nunha soa pasada o filtro vai corrixindo segundo le e deixa as
    // dúas partes case iguais (esmaga a dinámica); en dúas, sobe todo o mesmo e o baixo segue sendo baixo.
    let src = make(dir.path(), "sube.wav", "anoisesrc=d=8:c=pink:a=0.3:r=44100:seed=7,volume='if(lt(t,4),0.05,1)':eval=frame", &[]);
    let step = |p: &Path| mean_db(p, "4.5", "3") - mean_db(p, "0", "3.5");
    let before = step(&src);
    assert!(before > 20.0, "a mostra ten un salto grande: {before}");
    let r = ffmpeg::normalize_loudness(&src, &dir.path().join("dous.wav"), false, Loudness { lufs: -16.0, peak: -1.5, range: 20.0 }, false).unwrap();
    assert!(r.linear, "abonda con cambiar o volume por igual");
    assert!((lufs(&r.output) - -16.0).abs() < 0.6, "chega ao obxectivo: {}", lufs(&r.output));
    assert!((step(&r.output) - before).abs() < 1.5, "o salto entre a parte baixa e a alta consérvase: antes {before}, despois {}", step(&r.output));
}

#[test]
fn normalizar_e_os_efectos_conservan_a_calidade_do_orixinal() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();

    // MP3 a 320 kbps: antes saía a 128 (ou 64 en mono).
    let mp3 = make(d, "a.mp3", QUIET, &["-b:a", "320k"]);
    let n = ffmpeg::normalize_loudness(&mp3, &d.join("a-n.mp3"), false, Loudness::default(), false).unwrap().output;
    assert!(kbps(&n) >= 300, "o MP3 normalizado quedou a {} kbps", kbps(&n));
    assert_eq!(rate(&n), 44100);
    let f = ffmpeg::media_edit(&mp3, &d.join("a-f.mp3"), false, MediaOp::FadeIn(1.0), false).unwrap();
    assert!(kbps(&f) >= 300, "o MP3 co esvaecemento quedou a {} kbps", kbps(&f));
    let e = ffmpeg::media_edit(&mp3, &d.join("a-e.mp3"), false, MediaOp::Echo(300.0), false).unwrap();
    assert!(kbps(&e) >= 300);

    // WAV de 24 bits a 96 kHz: antes saía a 16 bits (e a 192 kHz ao normalizar).
    let wav = make(d, "b.wav", "anoisesrc=d=4:c=pink:a=0.02:r=96000", &["-c:a", "pcm_s24le"]);
    for out in [
        ffmpeg::normalize_loudness(&wav, &d.join("b-n.wav"), false, Loudness::default(), false).unwrap().output,
        ffmpeg::media_edit(&wav, &d.join("b-v.wav"), false, MediaOp::Volume(2.0), false).unwrap(),
        ffmpeg::media_edit(&wav, &d.join("b-d.wav"), false, MediaOp::Denoise, false).unwrap(),
    ] {
        assert_eq!(audio(&out)["codec_name"], "pcm_s24le", "{}", out.display());
        assert_eq!(rate(&out), 96000, "{}", out.display());
    }

    // FLAC: a frecuencia non pasa a 192 kHz.
    let flac = make(d, "c.flac", QUIET, &[]);
    let n = ffmpeg::normalize_loudness(&flac, &d.join("c-n.flac"), false, Loudness::default(), false).unwrap().output;
    assert_eq!(rate(&n), 44100);
    assert_eq!(audio(&n)["codec_name"], "flac");

    // OGG (Vorbis), Opus e M4A (AAC): seguen saíndo no seu códec e lense ben.
    for (name, extra, codec) in [("d.ogg", vec!["-c:a", "libvorbis", "-q:a", "6"], "vorbis"), ("e.opus", vec!["-b:a", "160k"], "opus"), ("f.m4a", vec!["-b:a", "256k"], "aac")] {
        let src = make(d, name, QUIET, &extra);
        let out = d.join(format!("n-{name}"));
        let n = ffmpeg::normalize_loudness(&src, &out, false, Loudness::default(), false).unwrap().output;
        assert_eq!(audio(&n)["codec_name"], codec, "{name}");
        assert!((lufs(&n) - -16.0).abs() < 1.0, "{name}: {}", lufs(&n));
    }
    let m4a = d.join("n-f.m4a");
    assert!(kbps(&m4a) >= 200, "o AAC de 256 kbps quedou a {}", kbps(&m4a));
}

#[test]
fn normalizar_un_video_copia_a_imaxe_sen_tocala() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("v.mp4");
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=320x180:rate=25:duration=4", "-f", "lavfi", "-i", QUIET])
        .args(["-shortest", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "192k"])
        .arg(&src)
        .status()
        .unwrap();
    assert!(st.success());
    let video = |p: &Path| {
        let info = ffmpeg::probe(p).unwrap();
        let v = info["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == "video").unwrap().clone();
        (v["codec_name"].clone(), v["width"].clone(), v["nb_frames"].clone(), v["bit_rate"].clone())
    };
    let r = ffmpeg::normalize_loudness(&src, &dir.path().join("v-n.mp4"), false, Loudness { lufs: -14.0, peak: -1.0, range: 11.0 }, true).unwrap();
    assert_eq!(video(&r.output), video(&src), "a imaxe cópiase tal cal");
    assert!((lufs(&r.output) - -14.0).abs() < 1.0, "{}", lufs(&r.output));
}

#[test]
fn os_efectos_de_son_nun_video_non_tocan_a_imaxe_nin_cambian_o_codec_do_son() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    // MP4 con AAC e MKV con AAC (o MKV escollería outro códec se non se fixa).
    for (name, acodec) in [("v.mp4", "aac"), ("v.mkv", "aac"), ("w.webm", "libopus")] {
        let src = d.join(name);
        let vcodec = if name.ends_with("webm") { "libvpx-vp9" } else { "libx264" };
        let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=320x180:rate=25:duration=3", "-f", "lavfi", "-i", QUIET])
            .args(["-shortest", "-c:v", vcodec, "-pix_fmt", "yuv420p", "-c:a", acodec, "-b:a", "160k"])
            .arg(&src)
            .status()
            .unwrap();
        assert!(st.success(), "{name}");
        let codecs = |p: &Path| {
            let info = ffmpeg::probe(p).unwrap();
            let s = info["streams"].as_array().unwrap();
            let c = |kind: &str| s.iter().find(|x| x["codec_type"] == kind).map(|x| (x["codec_name"].clone(), x["width"].clone(), x["nb_frames"].clone()));
            (c("video"), c("audio").map(|a| a.0))
        };
        let want = codecs(&src);
        let outs = [
            ffmpeg::media_edit(&src, &d.join(format!("fi-{name}")), false, MediaOp::FadeIn(1.0), true).unwrap(),
            ffmpeg::media_edit(&src, &d.join(format!("ec-{name}")), false, MediaOp::Echo(300.0), true).unwrap(),
            ffmpeg::media_edit(&src, &d.join(format!("dn-{name}")), false, MediaOp::Denoise, true).unwrap(),
            ffmpeg::normalize_loudness(&src, &d.join(format!("nm-{name}")), false, Loudness::default(), true).unwrap().output,
        ];
        for out in outs {
            assert_eq!(codecs(&out), want, "{name}: {}", out.display());
        }
    }
}

#[test]
fn normalizar_rexeita_o_silencio_os_valores_malos_e_non_pisa_nada() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let silent = make(d, "s.wav", "anullsrc=r=44100:cl=stereo:d=3", &[]);
    assert!(ffmpeg::normalize_loudness(&silent, &d.join("s-n.wav"), false, Loudness::default(), false).is_err());
    assert!(!d.join("s-n.wav").exists(), "sen resultado non queda ficheiro");

    let src = make(d, "a.wav", QUIET, &[]);
    for bad in [Loudness { lufs: 0.0, peak: -1.0, range: 11.0 }, Loudness { lufs: -16.0, peak: 3.0, range: 11.0 }, Loudness { lufs: -16.0, peak: -1.0, range: 0.0 }, Loudness { lufs: f64::NAN, peak: -1.0, range: 11.0 }] {
        assert!(ffmpeg::normalize_loudness(&src, &d.join("bad.wav"), false, bad, false).is_err());
        assert!(!d.join("bad.wav").exists());
    }
    // Non pisa un ficheiro que xa existe nin o propio orixinal.
    std::fs::write(d.join("xa.wav"), b"x").unwrap();
    assert!(ffmpeg::normalize_loudness(&src, &d.join("xa.wav"), false, Loudness::default(), false).is_err());
    assert_eq!(std::fs::read(d.join("xa.wav")).unwrap(), b"x");
    assert!(ffmpeg::normalize_loudness(&src, &src, true, Loudness::default(), false).is_err());
    // Nin deixa restos temporais.
    let leftovers: Vec<_> = std::fs::read_dir(d).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with(".trasno-")).collect();
    assert!(leftovers.is_empty());
}
