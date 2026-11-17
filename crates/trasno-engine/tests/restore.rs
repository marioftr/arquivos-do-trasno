//! Restaurar vídeo sobre ficheiros reais: que cada axuste faga o que di (medido), que saia válido en cada contedor, e que nada
//! quede a medias.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use trasno_engine::ffmpeg;
use trasno_engine::restore::{has_vidstab, preview_frame, restore_video, RestorePlan};

fn ff(args: &[&str]) {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-y"]).args(args).output().unwrap();
    assert!(o.status.success(), "ffmpeg: {}", String::from_utf8_lossy(&o.stderr));
}

fn sha(p: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(p).unwrap()))
}

/// Media dun dato de `signalstats` ou `idet` sobre todos os fotogramas, aplicando antes `pre` ao filtro.
fn stat(p: &Path, pre: &str, key: &str) -> f64 {
    let vf = format!("{pre}signalstats,metadata=print:file=-");
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-i", p.to_str().unwrap(), "-vf", &vf, "-f", "null", "-"]).output().unwrap();
    let text = String::from_utf8_lossy(&o.stdout);
    let vals: Vec<f64> = text.lines().filter_map(|l| l.strip_prefix(&format!("lavfi.signalstats.{key}="))).filter_map(|v| v.trim().parse().ok()).collect();
    assert!(!vals.is_empty(), "sen datos de {key}: {}", String::from_utf8_lossy(&o.stderr));
    vals.iter().sum::<f64>() / vals.len() as f64
}

/// Cambio medio entre fotogramas consecutivos: canto «treme» ou «ferve» a imaxe.
fn jitter(p: &Path) -> f64 {
    stat(p, "tblend=all_mode=difference,", "YAVG")
}

fn probe_video(p: &Path) -> serde_json::Value {
    ffmpeg::probe(p).unwrap()["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == "video").unwrap().clone()
}

/// Decodifica enteiro. Algunhas mostras de FFmpeg (Opus en WebM) xa traen avisos propios: nese caso só se esixe que non falle.
fn decodes_clean(p: &Path) -> bool {
    let run = |q: &Path| std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-i", q.to_str().unwrap(), "-f", "null", "-"]).output().unwrap();
    let o = run(p);
    let own_noise = p.extension().is_some_and(|e| e == "webm");
    o.status.success() && (o.stderr.is_empty() || own_noise)
}

fn frames(p: &Path) -> f64 {
    let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-v", "error", "-i", p.to_str().unwrap(), "-map", "0:v:0", "-f", "null", "-", "-stats"]).output().unwrap();
    let log = String::from_utf8_lossy(&o.stderr).replace(char::from(13), "\n");
    log.lines().rev().find_map(|l| l.strip_prefix("frame=").and_then(|r| r.split_whitespace().next()).and_then(|n| n.parse::<f64>().ok())).unwrap_or(0.0)
}

/// 4 s de imaxe que treme (barras SMPTE recortadas con desprazamento irregular) e un tono.
fn shaky(dir: &Path, ext: &str) -> PathBuf {
    let p = dir.join(format!("treme.{ext}"));
    let src = "smptebars=s=480x360:r=25:d=4,crop=400:300:x='40+22*sin(t*37)':y='30+16*cos(t*53)'";
    let mut a = vec!["-f", "lavfi", "-i", src, "-f", "lavfi", "-i", "sine=f=440:r=44100:d=4", "-shortest", "-pix_fmt", "yuv420p"];
    match ext {
        "webm" => a.extend(["-c:v", "libvpx-vp9", "-deadline", "realtime", "-cpu-used", "8", "-c:a", "libopus"]),
        "avi" => a.extend(["-c:v", "mpeg4", "-qscale:v", "2", "-c:a", "mp3"]),
        "flv" => a.extend(["-c:v", "flv", "-qscale:v", "2", "-c:a", "mp3"]),
        "wmv" => a.extend(["-c:v", "wmv2", "-b:v", "4M", "-c:a", "wmav2"]),
        "mpg" => a.extend(["-c:v", "mpeg2video", "-qscale:v", "2", "-c:a", "mp2"]),
        _ => a.extend(["-c:v", "libx264", "-crf", "12", "-c:a", "aac"]),
    }
    a.push(p.to_str().unwrap());
    ff(&a);
    p
}

#[test]
fn estabilizar_reduce_de_verdade_o_tremor() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = shaky(dir.path(), "mp4");
    let before = jitter(&src);
    for level in 1..=3u8 {
        let out = dir.path().join(format!("est{level}.mp4"));
        let r = restore_video(&src, &out, false, &RestorePlan { stabilize: level, ..Default::default() }).unwrap();
        assert_eq!(r.stabilizer.as_deref(), Some(if has_vidstab() { "vidstab" } else { "deshake" }));
        let after = jitter(&out);
        println!("estabilizar {level}: {before:.2} → {after:.2}");
        assert!(after < before * 0.75, "nivel {level}: o tremor non baixou abondo ({before:.2} → {after:.2})");
        // O tamaño e a duración non cambian: a imaxe achégase, non se encolle.
        let (v0, v1) = (probe_video(&src), probe_video(&out));
        assert_eq!((v0["width"].clone(), v0["height"].clone()), (v1["width"].clone(), v1["height"].clone()));
        assert!((frames(&out) - frames(&src)).abs() <= 1.0, "mesmos fotogramas");
        assert!(decodes_clean(&out));
    }
}

#[test]
fn desentrelazar_quita_o_efecto_de_peine() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("entrelazado.mp4");
    // Un cadro que se move rápido, gravado en dous campos.
    ff(&["-f", "lavfi", "-i", "testsrc2=s=320x240:r=50:d=3", "-vf", "interlace=scan=tff", "-c:v", "libx264", "-crf", "14", "-flags", "+ilme+ildct", "-pix_fmt", "yuv420p", src.to_str().unwrap()]);
    let comb = |p: &Path| {
        let o = std::process::Command::new(ffmpeg::ffmpeg_path().unwrap()).args(["-hide_banner", "-i", p.to_str().unwrap(), "-vf", "idet", "-f", "null", "-"]).output().unwrap();
        let t = String::from_utf8_lossy(&o.stderr).to_string();
        let line = t.lines().rev().find(|l| l.contains("Multi frame detection")).unwrap_or("").to_string();
        let n = |k: &str| line.split(k).nth(1).and_then(|r| r.split_whitespace().next()).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        (n("TFF:") + n("BFF:"), n("Progressive:"))
    };
    let (inter0, _) = comb(&src);
    assert!(inter0 > 20.0, "a mostra debía ser entrelazada ({inter0})");
    let out = dir.path().join("progresivo.mp4");
    restore_video(&src, &out, false, &RestorePlan { deinterlace: true, ..Default::default() }).unwrap();
    let (inter1, prog1) = comb(&out);
    println!("entrelazados {inter0} → {inter1} (progresivos {prog1})");
    assert!(inter1 < inter0 * 0.2, "seguen quedando fotogramas entrelazados: {inter1}");
}

#[test]
fn cor_ruido_e_afiar_fan_o_que_din() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("cor.mp4");
    ff(&["-f", "lavfi", "-i", "testsrc2=s=320x240:r=25:d=2", "-c:v", "libx264", "-crf", "12", "-pix_fmt", "yuv420p", src.to_str().unwrap()]);
    let run = |tag: &str, plan: RestorePlan| {
        let out = dir.path().join(format!("{tag}.mp4"));
        restore_video(&src, &out, false, &plan).unwrap();
        assert!(decodes_clean(&out), "{tag}");
        out
    };
    // Brillo: máis claro.
    let bright = run("brillo", RestorePlan { brightness: 0.2, ..Default::default() });
    assert!(stat(&bright, "", "YAVG") > stat(&src, "", "YAVG") + 15.0, "o brillo non subiu");
    // Saturación 0: branco e negro (a cor medida cae a case nada).
    let gray = run("gris", RestorePlan { saturation: 0.0, ..Default::default() });
    assert!(stat(&gray, "", "SATAVG") < 2.0 && stat(&src, "", "SATAVG") > 20.0, "non quedou en branco e negro");
    // Contraste: máis separación entre os tons.
    let hard = run("contraste", RestorePlan { contrast: 1.6, ..Default::default() });
    assert!(stat(&hard, "", "YHIGH") > stat(&src, "", "YHIGH") || stat(&hard, "", "YLOW") < stat(&src, "", "YLOW"), "o contraste non cambiou");
    // Ruído: unha imaxe chea de gran ferve menos entre fotogramas despois de limpala.
    let noisy = dir.path().join("ruido.mp4");
    ff(&["-f", "lavfi", "-i", "smptebars=s=320x240:r=25:d=2,noise=alls=28:allf=t", "-c:v", "libx264", "-crf", "10", "-pix_fmt", "yuv420p", noisy.to_str().unwrap()]);
    let before = jitter(&noisy);
    let mut last = before;
    for (level, limit) in [(1u8, 0.97), (2, 0.85), (3, 0.55)] {
        let out = dir.path().join(format!("limpo{level}.mp4"));
        restore_video(&noisy, &out, false, &RestorePlan { denoise: level, ..Default::default() }).unwrap();
        let after = jitter(&out);
        println!("ruído {level}: {before:.2} → {after:.2}");
        assert!(after < before * limit && after < last, "nivel {level}: o ruído non baixou abondo ({before:.2} → {after:.2})");
        last = after;
    }
    // Afiar: máis detalle fino (a diferenza entre un fotograma e a súa versión borrosa sobe).
    let soft = dir.path().join("brando.mp4");
    ff(&["-f", "lavfi", "-i", "testsrc2=s=320x240:r=25:d=2,gblur=sigma=1.5", "-c:v", "libx264", "-crf", "10", "-pix_fmt", "yuv420p", soft.to_str().unwrap()]);
    let sharp = dir.path().join("nitido.mp4");
    restore_video(&soft, &sharp, false, &RestorePlan { sharpen: 3, ..Default::default() }).unwrap();
    let detail = |p: &Path| stat(p, "split[a][b];[b]gblur=sigma=2[bl];[a][bl]blend=all_mode=difference,", "YAVG");
    assert!(detail(&sharp) > detail(&soft) * 1.3, "non se afiou: {} → {}", detail(&soft), detail(&sharp));
}

#[test]
fn saen_validos_en_todos_os_contedores_co_son_igual_e_a_orixe_intacta() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let plan = RestorePlan { stabilize: 2, denoise: 2, sharpen: 1, contrast: 1.1, deflicker: true, ..Default::default() };
    let mut bad = Vec::new();
    for ext in ["mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "mpg"] {
        let src = shaky(dir.path(), ext);
        let before = sha(&src);
        let out = dir.path().join(format!("rest-{ext}.{ext}"));
        let res = restore_video(&src, &out, false, &plan);
        let verdict = (|| -> Result<(), String> {
            res.map_err(|e| e.to_string())?;
            if sha(&src) != before {
                return Err("o orixinal cambiou".into());
            }
            if !decodes_clean(&out) {
                return Err("non decodifica limpo".into());
            }
            let (v0, v1) = (probe_video(&src), probe_video(&out));
            if v0["width"] != v1["width"] || v0["height"] != v1["height"] {
                return Err(format!("o tamaño cambiou: {}x{} → {}x{}", v0["width"], v0["height"], v1["width"], v1["height"]));
            }
            if (frames(&out) - frames(&src)).abs() > 2.0 {
                return Err(format!("fotogramas {} → {}", frames(&src), frames(&out)));
            }
            let streams = ffmpeg::probe(&out).unwrap();
            let audio = streams["streams"].as_array().unwrap().iter().any(|s| s["codec_type"] == "audio");
            if !audio {
                return Err("perdeu o son".into());
            }
            let a0 = ffmpeg::probe(&src).unwrap()["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == "audio").unwrap()["codec_name"].clone();
            let a1 = streams["streams"].as_array().unwrap().iter().find(|s| s["codec_type"] == "audio").unwrap()["codec_name"].clone();
            if a0 != a1 {
                return Err(format!("o son cambiou de códec: {a0} → {a1}"));
            }
            let d0 = ffmpeg::probe(&src).unwrap()["format"]["duration"].as_str().unwrap().parse::<f64>().unwrap();
            let d1 = streams["format"]["duration"].as_str().unwrap().parse::<f64>().unwrap();
            if (d0 - d1).abs() > 0.3 {
                return Err(format!("dura {d1:.2} s e antes {d0:.2} s"));
            }
            Ok(())
        })();
        println!("  {} {ext}{}", if verdict.is_ok() { "✓" } else { "✗" }, verdict.as_ref().err().map(|e| format!(" → {e}")).unwrap_or_default());
        if let Err(e) = verdict {
            bad.push(format!("{ext}: {e}"));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn fotograma_de_vista_previa_e_erros_claros_sen_restos() {
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = shaky(dir.path(), "mp4");
    let jpg = dir.path().join("f.jpg");
    preview_frame(&src, &jpg, 2.0, &RestorePlan { denoise: 2, contrast: 1.2, ..Default::default() }).unwrap();
    assert!(image::load_from_memory(&std::fs::read(&jpg).unwrap()).is_ok(), "o fotograma non é unha imaxe");
    // Só estabilizar non se ve nun fotograma.
    assert!(preview_frame(&src, &jpg, 1.0, &RestorePlan { stabilize: 2, ..Default::default() }).is_err());
    // Un instante cero tamén vale.
    preview_frame(&src, &jpg, 0.0, &RestorePlan { sharpen: 1, ..Default::default() }).unwrap();

    let out = dir.path().join("o.mp4");
    assert!(restore_video(&src, &out, false, &RestorePlan::default()).is_err(), "plan baleiro");
    assert!(restore_video(&src, &out, false, &RestorePlan { contrast: 9.0, ..Default::default() }).is_err(), "fóra de rango");
    assert!(restore_video(&dir.path().join("non-existe.mp4"), &out, false, &RestorePlan { sharpen: 1, ..Default::default() }).is_err());
    assert!(restore_video(&src, &src, true, &RestorePlan { sharpen: 1, ..Default::default() }).is_err(), "non pisa o orixinal");
    // Un audio non ten imaxe; un GIF non se restaura.
    let wav = dir.path().join("a.wav");
    ff(&["-f", "lavfi", "-i", "sine=f=440:d=1", wav.to_str().unwrap()]);
    assert!(restore_video(&wav, &dir.path().join("a.mp4"), false, &RestorePlan { sharpen: 1, ..Default::default() }).is_err());
    assert!(restore_video(&src, &dir.path().join("o.gif"), false, &RestorePlan { sharpen: 1, ..Default::default() }).is_err());
    assert!(!out.exists() && !dir.path().join("a.mp4").exists() && !dir.path().join("o.gif").exists());
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().contains(".trasno-")).collect();
    assert!(leftovers.is_empty(), "sobraron ficheiros temporais");
}

#[test]
fn se_non_hai_vidstab_o_filtro_alternativo_tamen_funciona() {
    // O camiño de reserva (`deshake`) non se usa con esta FFmpeg, así que se proba a cadea á parte.
    if !ffmpeg::is_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = shaky(dir.path(), "mp4");
    let out = dir.path().join("d.mp4");
    for r in [16, 32, 48] {
        ff(&["-i", src.to_str().unwrap(), "-vf", &format!("deshake=rx={r}:ry={r}:edge=mirror"), "-c:v", "libx264", "-pix_fmt", "yuv420p", out.to_str().unwrap()]);
        assert!(decodes_clean(&out));
    }
}
