//! Edición conxunta: todos os axustes nunha soa exportación, sobre ficheiros reais.

use std::path::{Path, PathBuf};
use trasno_engine::edit_plan::{apply_plan, preview_audio, EditPlan};
use trasno_engine::ffmpeg::{self, Loudness};

fn ff(args: &[&str]) {
    let st = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-y"]).args(args).status().unwrap();
    assert!(st.success());
}

/// Vídeo de 10 s (25 fps) con tono de 440 Hz.
fn video(dir: &Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    ff(&["-f", "lavfi", "-i", "testsrc=s=320x240:r=25:d=10", "-f", "lavfi", "-i", "sine=f=440:r=44100:d=10", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "192k", p.to_str().unwrap()]);
    p
}

fn stream(p: &Path, kind: &str) -> serde_json::Value {
    ffmpeg::probe(p).unwrap()["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == kind).unwrap().clone()
}
fn duration(p: &Path) -> f64 {
    ffmpeg::probe(p).unwrap()["format"]["duration"].as_str().unwrap().parse().unwrap()
}

#[test]
fn todos_os_axustes_saen_nun_so_ficheiro_coa_duracion_e_a_calidade_agardadas() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = video(dir.path(), "a.mp4");
    let out = dir.path().join("b.mp4");
    // Cortar de 2 a 8 s (6 s), ao dobre de velocidade (3 s), a medio volume, con graves e esvaecer final.
    let plan = EditPlan { start: 2.0, end: Some(8.0), speed: 2.0, volume: 0.5, bass: 4.0, fade_out: 0.5, ..Default::default() };
    let r = apply_plan(&src, &out, false, &plan).unwrap();
    assert!(!r.video_copied);
    assert!((duration(&out) - 3.0).abs() < 0.15, "dura {}", duration(&out));
    let (v, a) = (stream(&out, "video"), stream(&out, "audio"));
    assert_eq!(v["codec_name"], "h264");
    assert_eq!(a["sample_rate"], "44100", "o son conserva a frecuencia");
    // A imaxe tamén chega aos 3 s (75 fotogramas): son e imaxe saen sincronizados.
    let frames: f64 = v["nb_frames"].as_str().unwrap().parse().unwrap();
    assert!((frames - 75.0).abs() <= 3.0, "{frames} fotogramas");
    // O orixinal non se tocou.
    assert!((duration(&src) - 10.0).abs() < 0.1);
}

#[test]
fn se_so_se_toca_o_son_a_imaxe_copiase_sen_recodificar() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = video(dir.path(), "a.mp4");
    let out = dir.path().join("b.mp4");
    let r = apply_plan(&src, &out, false, &EditPlan { volume: 2.0, echo_ms: 300.0, ..Default::default() }).unwrap();
    assert!(r.video_copied);
    let (v0, v1) = (stream(&src, "video"), stream(&out, "video"));
    assert_eq!(v0["nb_frames"], v1["nb_frames"], "mesmos fotogramas");
    assert_eq!(v0["codec_name"], v1["codec_name"]);
    // O eco deixa unha pequena cola de son despois do final (300 ms de atraso).
    assert!((duration(&out) - 10.0).abs() < 0.5, "dura {}", duration(&out));
    // Cortar desde o principio é unha copia; cortar desde outro instante recodifica a imaxe para que o corte sexa exacto
    // (copiar caería no fotograma clave máis próximo e a imaxe quedaría desaliñada do son ou desaparecería).
    let head = dir.path().join("h.mp4");
    assert!(apply_plan(&src, &head, false, &EditPlan { end: Some(4.0), ..Default::default() }).unwrap().video_copied);
    let cut = dir.path().join("c.mp4");
    let r = apply_plan(&src, &cut, false, &EditPlan { start: 1.0, end: Some(4.0), ..Default::default() }).unwrap();
    assert!(!r.video_copied);
    assert!((duration(&cut) - 3.0).abs() < 0.15, "dura {}", duration(&cut));
    let frames: f64 = stream(&cut, "video")["nb_frames"].as_str().unwrap().parse().unwrap();
    assert!((frames - 75.0).abs() <= 2.0, "{frames} fotogramas");
}

#[test]
fn igualar_o_volume_mide_o_que_queda_despois_do_corte() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("a.wav");
    // 6 s: os 3 primeiros fortes e os 3 últimos moi baixos. Cortando só o tramo baixo, o volume final ten que ser o pedido.
    ff(&["-f", "lavfi", "-i", "sine=f=440:r=44100:d=6,volume='if(lt(t,3),1,0.05)':eval=frame", src.to_str().unwrap()]);
    let out = dir.path().join("b.wav");
    let target = Loudness { lufs: -16.0, peak: -1.5, range: 11.0 };
    apply_plan(&src, &out, false, &EditPlan { start: 3.0, normalize: Some(target), ..Default::default() }).unwrap();
    assert!((duration(&out) - 3.0).abs() < 0.1);
    let m = ffmpeg::measure_loudness(&out, target).unwrap();
    assert!((m.lufs - -16.0).abs() < 1.0, "volume final {} LUFS", m.lufs);
}

#[test]
fn os_erros_son_claros_e_non_deixan_ficheiros_a_medias() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = video(dir.path(), "a.mp4");
    let out = dir.path().join("x.mp4");
    assert!(apply_plan(&src, &out, false, &EditPlan::default()).is_err(), "sen cambios");
    assert!(apply_plan(&src, &out, false, &EditPlan { speed: 9.0, ..Default::default() }).is_err(), "velocidade imposible");
    assert!(apply_plan(&src, &out, false, &EditPlan { start: 99.0, ..Default::default() }).is_err(), "inicio despois do final");
    let wav = dir.path().join("a.wav");
    ff(&["-f", "lavfi", "-i", "sine=d=2", wav.to_str().unwrap()]);
    assert!(apply_plan(&wav, &dir.path().join("m.wav"), false, &EditPlan { mute: true, ..Default::default() }).is_err(), "mudo nun audio");
    assert!(!out.exists(), "non queda ningún ficheiro a medias");
}

/// Nivel (RMS) do son decodificado, por tramos dun segundo.
fn rms_per_second(p: &Path) -> Vec<f64> {
    let out = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-i", p.to_str().unwrap(), "-ac", "1", "-ar", "8000", "-f", "f32le", "-"]).output().unwrap();
    let samples: Vec<f32> = out.stdout.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
    samples.chunks(8000).map(|c| (c.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / c.len() as f64).sqrt()).collect()
}

#[test]
fn a_vista_previa_do_son_cae_no_seu_sitio_da_liña_de_tempo_e_da_a_volta_ao_tramo() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    // 10 s de tono cuxo volume sobe de 0 a 1 ao longo do ficheiro.
    let src = dir.path().join("rampa.wav");
    ff(&["-f", "lavfi", "-i", "sine=f=440:r=44100:d=10,volume='t/10':eval=frame", src.to_str().unwrap()]);
    let out = dir.path().join("p.m4a");
    preview_audio(&src, &out, 2.0, Some(8.0), false, true).unwrap();
    assert!((duration(&out) - 10.0).abs() < 0.3, "dura o mesmo ca o orixinal: {}", duration(&out));
    let r = rms_per_second(&out);
    assert!(r[0] < 0.01 && r[1] < 0.01 && r[9] < 0.01, "fóra do tramo hai silencio: {r:?}");
    // Dentro do tramo o volume baixa (antes subía): o comezo do tramo soa máis ca o final.
    assert!(r[2] > r[7] * 2.0, "o tramo está ao revés: {r:?}");
    // Sen darlle a volta (só ruído), o tramo segue subindo.
    let q = dir.path().join("q.m4a");
    preview_audio(&src, &q, 2.0, Some(8.0), true, false).unwrap();
    let r2 = rms_per_second(&q);
    assert!(r2[0] < 0.01 && r2[2] < r2[7], "co ruído quitado o tramo non cambia de sentido: {r2:?}");
    // Erros: nada que facer, e dar a volta a máis de 20 minutos.
    assert!(preview_audio(&src, &dir.path().join("x.m4a"), 0.0, None, false, false).is_err());
    let largo = dir.path().join("largo.wav");
    ff(&["-f", "lavfi", "-i", "anullsrc=r=8000:cl=mono", "-t", "1300", largo.to_str().unwrap()]);
    assert!(preview_audio(&largo, &dir.path().join("y.m4a"), 0.0, None, false, true).is_err());
}
