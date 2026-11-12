//! Cortar silencios en audio y vídeo: análise, corte, sincronía de imaxe e son, e casos límite.

use std::path::{Path, PathBuf};
use trasno_engine::ffmpeg;
use trasno_engine::silence::{analyze_silence, cut_silence, SilenceOptions};

fn ff(args: &[&str]) {
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-y"]).args(args).status().unwrap();
    assert!(st.success());
}

/// 10 s con son agás dous silencios dixitais: de 2 a 4 e de 7 a 9.
const TONE: &str = "sine=f=440:r=44100:d=10,volume='if(between(t,2,4)+between(t,7,9),0,1)':eval=frame";

fn audio_file(dir: &Path, name: &str, extra: &[&str]) -> PathBuf {
    let p = dir.join(name);
    let mut a = vec!["-f", "lavfi", "-i", TONE];
    a.extend_from_slice(extra);
    a.push(p.to_str().unwrap());
    ff(&a);
    p
}

fn dur(p: &Path, kind: &str) -> f64 {
    let info = ffmpeg::probe(p).unwrap();
    let s = info["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == kind).unwrap().clone();
    s["duration"].as_str().and_then(|d| d.parse().ok()).unwrap_or_else(|| info["format"]["duration"].as_str().unwrap().parse().unwrap())
}

#[test]
fn analiza_os_silencios_e_o_que_quedaria() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = audio_file(dir.path(), "a.wav", &[]);
    let a = analyze_silence(&src, &SilenceOptions::default()).unwrap();
    assert_eq!(a.cuts.len(), 2, "{:?}", a.cuts);
    assert!((a.cuts[0].0 - 2.1).abs() < 0.05 && (a.cuts[0].1 - 3.9).abs() < 0.05, "{:?}", a.cuts);
    assert!((a.cuts[1].0 - 7.1).abs() < 0.05 && (a.cuts[1].1 - 8.9).abs() < 0.05, "{:?}", a.cuts);
    assert!((a.duration - 10.0).abs() < 0.05 && (a.result_duration - 6.4).abs() < 0.1, "{a:?}");
    // Un silencio mínimo maior que os que hai: non se corta nada.
    assert!(analyze_silence(&src, &SilenceOptions { min_silence: 3.0, ..Default::default() }).unwrap().cuts.is_empty());
    // Sen padding quítase máis.
    let b = analyze_silence(&src, &SilenceOptions { padding: 0.0, ..Default::default() }).unwrap();
    assert!(b.result_duration < a.result_duration - 0.3);
}

#[test]
fn corta_o_audio_e_conserva_formato_e_calidade() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for (name, extra) in [("a.wav", vec![]), ("a.mp3", vec!["-b:a", "256k"]), ("a.flac", vec![])] {
        let src = audio_file(dir.path(), name, &extra);
        let out = dir.path().join(format!("cortado-{name}"));
        let r = cut_silence(&src, &out, false, &SilenceOptions::default()).unwrap();
        assert_eq!(r.cuts, 2);
        let d = dur(&out, "audio");
        assert!((d - 6.4).abs() < 0.1, "{name}: dura {d}");
        // Xa non queda ningún silencio longo.
        let left = analyze_silence(&out, &SilenceOptions { min_silence: 0.5, ..Default::default() }).unwrap();
        assert!(left.cuts.is_empty(), "{name}: {:?}", left.cuts);
        // E a calidade é a do orixinal.
        let info = ffmpeg::probe(&out).unwrap();
        let a = info["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == "audio").unwrap().clone();
        assert_eq!(a["sample_rate"], "44100", "{name}");
        if name.ends_with("mp3") {
            let kbps = a["bit_rate"].as_str().unwrap().parse::<u32>().unwrap() / 1000;
            assert!(kbps >= 240, "{name}: {kbps} kbps");
        }
    }
}

#[test]
fn nun_video_a_imaxe_e_o_son_quedan_sincronizados() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for name in ["v.mp4", "v.mkv"] {
        let src = dir.path().join(name);
        ff(&["-f", "lavfi", "-i", "testsrc2=size=320x180:rate=25:duration=10", "-f", "lavfi", "-i", TONE, "-shortest", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "160k", src.to_str().unwrap()]);
        let out = dir.path().join(format!("c-{name}"));
        let r = cut_silence(&src, &out, false, &SilenceOptions::default()).unwrap();
        assert_eq!(r.cuts, 2);
        let (v, a) = (dur(&out, "video"), dur(&out, "audio"));
        assert!((v - a).abs() < 0.05, "{name}: imaxe {v} s e son {a} s");
        assert!((v - 6.4).abs() < 0.15, "{name}: {v}");
        let info = ffmpeg::probe(&out).unwrap();
        let st = |k: &str| info["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == k).unwrap().clone();
        assert_eq!(st("video")["codec_name"], "h264");
        assert_eq!(st("audio")["codec_name"], "aac");
        assert_eq!((st("video")["width"].as_u64(), st("video")["height"].as_u64()), (Some(320), Some(180)));
        assert_eq!(st("video")["avg_frame_rate"].as_str().unwrap(), "25/1", "a cadencia non cambia");
    }
}

#[test]
fn silencio_no_principio_e_no_final_e_casos_limite() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    // 1 s de silencio, 3 s de son, 1 s de silencio.
    let src = d.join("puntas.wav");
    ff(&["-f", "lavfi", "-i", "sine=f=330:r=44100:d=5,volume='if(between(t,1,4),1,0)':eval=frame", src.to_str().unwrap()]);
    let r = cut_silence(&src, &d.join("o.wav"), false, &SilenceOptions { padding: 0.0, ..Default::default() }).unwrap();
    assert!((r.after - 3.0).abs() < 0.1, "{r:?}");
    assert!((dur(&d.join("o.wav"), "audio") - 3.0).abs() < 0.1);

    // Sen silencios: erro claro e nada escrito.
    let tone = d.join("tono.wav");
    ff(&["-f", "lavfi", "-i", "sine=f=440:r=44100:d=3", tone.to_str().unwrap()]);
    assert!(cut_silence(&tone, &d.join("nada.wav"), false, &SilenceOptions::default()).is_err());
    assert!(!d.join("nada.wav").exists());

    // Todo silencio: erro.
    let mute = d.join("mudo.wav");
    ff(&["-f", "lavfi", "-i", "anullsrc=r=44100:cl=mono:d=3", mute.to_str().unwrap()]);
    assert!(cut_silence(&mute, &d.join("nada2.wav"), false, &SilenceOptions::default()).is_err());
    assert!(!d.join("nada2.wav").exists());

    // Un vídeo sen son non ten silencios que cortar.
    let silent_video = d.join("sen-son.mp4");
    ff(&["-f", "lavfi", "-i", "testsrc2=size=160x90:rate=25:duration=2", "-c:v", "libx264", "-pix_fmt", "yuv420p", silent_video.to_str().unwrap()]);
    assert!(cut_silence(&silent_video, &d.join("nada3.mp4"), false, &SilenceOptions::default()).is_err());

    // Non pisa un ficheiro que xa existe nin o orixinal, e deixa o orixinal como estaba.
    let src2 = audio_file(d, "b.wav", &[]);
    let before = std::fs::read(&src2).unwrap();
    std::fs::write(d.join("xa.wav"), b"x").unwrap();
    assert!(cut_silence(&src2, &d.join("xa.wav"), false, &SilenceOptions::default()).is_err());
    assert_eq!(std::fs::read(d.join("xa.wav")).unwrap(), b"x");
    assert!(cut_silence(&src2, &src2, true, &SilenceOptions::default()).is_err());
    assert_eq!(std::fs::read(&src2).unwrap(), before);
    // Valores fóra de rango.
    assert!(cut_silence(&src2, &d.join("m.wav"), false, &SilenceOptions { threshold_db: 5.0, ..Default::default() }).is_err());
    // Nin restos temporais.
    let left: Vec<_> = std::fs::read_dir(d).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with(".trasno-")).collect();
    assert!(left.is_empty(), "{left:?}");
}

#[test]
fn con_moitos_cortes_tamen_funciona_sen_perder_a_sincronia() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    // 60 s: un silencio de 0,6 s cada 2 s (30 cortes), en vídeo.
    let src = d.join("muchos.mp4");
    ff(&["-f", "lavfi", "-i", "testsrc2=size=160x90:rate=30:duration=60", "-f", "lavfi", "-i", "sine=f=500:r=48000:d=60,volume='if(lt(mod(t,2),0.6),0,1)':eval=frame", "-shortest", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", src.to_str().unwrap()]);
    let r = cut_silence(&src, &d.join("c.mp4"), false, &SilenceOptions { padding: 0.05, ..Default::default() }).unwrap();
    assert!(r.cuts >= 28, "{}", r.cuts);
    let (v, a) = (dur(&d.join("c.mp4"), "video"), dur(&d.join("c.mp4"), "audio"));
    assert!((v - a).abs() < 0.05, "tras {} cortes a imaxe dura {v} e o son {a}", r.cuts);
    assert!((v - r.after).abs() < 0.2, "agardábase {} e saíron {v}", r.after);
}

#[test]
fn funciona_se_os_tempos_do_ficheiro_non_empezan_en_cero() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("desprazado.mkv");
    ff(&["-f", "lavfi", "-i", "testsrc2=size=320x180:rate=25:duration=10", "-f", "lavfi", "-i", TONE, "-shortest", "-output_ts_offset", "3", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", src.to_str().unwrap()]);
    let start: f64 = ffmpeg::probe(&src).unwrap()["format"]["start_time"].as_str().unwrap().parse().unwrap();
    assert!(start > 2.5, "a mostra empeza en {start}");
    let out = dir.path().join("o.mkv");
    let r = cut_silence(&src, &out, false, &SilenceOptions::default()).unwrap();
    assert_eq!(r.cuts, 2);
    let (v, a) = (dur(&out, "video"), dur(&out, "audio"));
    assert!((v - 6.5).abs() < 0.2 && (v - a).abs() < 0.06, "imaxe {v} e son {a}");
    // Os cortes caeron nos silencios de verdade: no resultado xa non queda ningún.
    let left = analyze_silence(&out, &SilenceOptions::default()).unwrap();
    assert!(left.cuts.is_empty(), "{:?}", left.cuts);
    // E o análise do orixinal dá os mesmos tempos ca se empezase en cero.
    let a = analyze_silence(&src, &SilenceOptions::default()).unwrap();
    assert!((a.cuts[0].0 - 2.1).abs() < 0.1, "{:?}", a.cuts);
}
