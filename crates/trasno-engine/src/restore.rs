//! Restaurar vídeo: estabilizar, quitar ruído, afiar, desentrelazar, quitar parpadeo e axustar a cor, todo nunha soa recodificación.
//!
//! O son non se toca (cópiase se o contedor é o mesmo). Estabilizar leva dúas pasadas: a primeira mide como treme a cámara e a
//! segunda compensa o movemento (libvidstab; se esta FFmpeg non a ten, usa o filtro `deshake`, máis simple).

use crate::error::{EngineError, Result};
use crate::ffmpeg::{self, check_success, run_ffmpeg};
use crate::integrity::ensure_distinct;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn bad<T>(m: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(m.into()))
}

/// O que se lle fai ao vídeo. Os valores por defecto non cambian nada. Os niveis van de 0 (non) a 3 (forte).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct RestorePlan {
    pub stabilize: u8,
    pub denoise: u8,
    pub sharpen: u8,
    pub deinterlace: bool,
    pub deflicker: bool,
    /// −0,5 a 0,5 (0 = igual).
    pub brightness: f64,
    /// 0,5 a 2 (1 = igual).
    pub contrast: f64,
    /// 0 a 3 (1 = igual; 0 = branco e negro).
    pub saturation: f64,
}

impl Default for RestorePlan {
    fn default() -> Self {
        RestorePlan { stabilize: 0, denoise: 0, sharpen: 0, deinterlace: false, deflicker: false, brightness: 0.0, contrast: 1.0, saturation: 1.0 }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RestoreReport {
    pub output: PathBuf,
    /// Con que se estabilizou: «vidstab», «deshake» ou nada.
    pub stabilizer: Option<String>,
}

impl RestorePlan {
    fn color(&self) -> bool {
        self.brightness != 0.0 || self.contrast != 1.0 || self.saturation != 1.0
    }
    pub fn is_empty(&self) -> bool {
        self.stabilize == 0 && self.denoise == 0 && self.sharpen == 0 && !self.deinterlace && !self.deflicker && !self.color()
    }
    /// Axustes que se poden ver nun só fotograma (a estabilización precisa o movemento de varios).
    pub fn has_frame_effects(&self) -> bool {
        self.denoise > 0 || self.sharpen > 0 || self.deinterlace || self.deflicker || self.color()
    }
    pub fn validate(&self) -> Result<()> {
        if self.stabilize > 3 || self.denoise > 3 || self.sharpen > 3 {
            return bad("Os niveis van de 0 (non) a 3 (forte).");
        }
        let fin = |v: f64| v.is_finite();
        if !fin(self.brightness) || !(-0.5..=0.5).contains(&self.brightness) {
            return bad("O brillo ten que estar entre −0,5 e 0,5.");
        }
        if !fin(self.contrast) || !(0.5..=2.0).contains(&self.contrast) {
            return bad("O contraste ten que estar entre 0,5 e 2.");
        }
        if !fin(self.saturation) || !(0.0..=3.0).contains(&self.saturation) {
            return bad("A saturación ten que estar entre 0 e 3.");
        }
        Ok(())
    }

    /// Cadea de filtros de imaxe, en orde: desentrelazar, parpadeo, estabilizar, ruído, cor e afiar. `stab` é o filtro de
    /// estabilización xa listo (a súa segunda pasada), se se pide.
    pub fn chain(&self, stab: Option<String>) -> Vec<String> {
        let mut c: Vec<String> = Vec::new();
        if self.deinterlace {
            c.push("yadif=mode=0:parity=-1:deint=0".into());
        }
        if self.deflicker {
            c.push("deflicker=mode=pm:size=5".into());
        }
        if let Some(s) = stab {
            c.push(s);
        }
        match self.denoise {
            1 => c.push("hqdn3d=3:2.5:6:5".into()),
            2 => c.push("hqdn3d=7:6:14:11".into()),
            3 => c.push("hqdn3d=14:12:28:22".into()),
            _ => {}
        }
        if self.color() {
            c.push(format!("eq=brightness={}:contrast={}:saturation={}", self.brightness, self.contrast, self.saturation));
        }
        match self.sharpen {
            1 => c.push("unsharp=5:5:0.5:5:5:0".into()),
            2 => c.push("unsharp=5:5:1.0:5:5:0".into()),
            3 => c.push("unsharp=5:5:1.6:5:5:0".into()),
            _ => {}
        }
        c
    }
}

/// Se esta FFmpeg traer libvidstab (a de Trasno si; unha propia do sistema pode non telo).
pub fn has_vidstab() -> bool {
    static HAS: OnceLock<bool> = OnceLock::new();
    *HAS.get_or_init(|| {
        let Ok(bin) = ffmpeg::ffmpeg_path() else { return false };
        let mut cmd = std::process::Command::new(bin);
        ffmpeg::hide_console(&mut cmd);
        cmd.args(["-hide_banner", "-filters"]).output().map(|o| String::from_utf8_lossy(&o.stdout).contains("vidstabtransform")).unwrap_or(false)
    })
}

/// Escapa unha ruta para un filtro de FFmpeg.
fn filter_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/").replace(':', "\\:").replace('\'', "\\\\\\'")
}

fn shaky_params(level: u8) -> (u32, u32) {
    // (shakiness 1-10, smoothing en fotogramas)
    match level {
        1 => (3, 10),
        2 => (6, 20),
        _ => (9, 35),
    }
}

/// Filtro de estabilización da segunda pasada (ou `deshake`, de pasada única).
fn stabilizer_filter(level: u8, trf: &Path, vidstab: bool) -> String {
    if vidstab {
        let (_, smoothing) = shaky_params(level);
        // optzoom=1: achega o xusto para que non se vexan os bordos negros que deixa o movemento compensado.
        format!("vidstabtransform=input='{}':smoothing={smoothing}:optzoom=1:interpol=bicubic", filter_path(trf))
    } else {
        let r = match level {
            1 => 16,
            2 => 32,
            _ => 48,
        };
        format!("deshake=rx={r}:ry={r}:edge=mirror")
    }
}

fn video_duration(info: &serde_json::Value) -> Option<f64> {
    info["format"]["duration"].as_str().and_then(|d| d.parse::<f64>().ok()).filter(|d| d.is_finite() && *d > 0.0)
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self> {
        let dir = std::env::temp_dir().join(format!("trasno-restaurar-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir)?;
        Ok(Scratch(dir))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Restaura o vídeo e garda o resultado nun ficheiro novo.
pub fn restore_video(input: &Path, output: &Path, overwrite: bool, plan: &RestorePlan) -> Result<RestoreReport> {
    plan.validate()?;
    if plan.is_empty() {
        return bad("Non hai nada que restaurar: todos os axustes están como no orixinal.");
    }
    if !input.is_file() {
        return Err(EngineError::InputNotFound(input.to_path_buf()));
    }
    ensure_distinct(&[input], output)?;
    let info = ffmpeg::probe(input)?;
    let streams = info["streams"].as_array().cloned().unwrap_or_default();
    if !streams.iter().any(|s| s["codec_type"] == "video" && s["disposition"]["attached_pic"] != 1) {
        return bad("Este ficheiro non ten imaxe que restaurar.");
    }
    let out_ext = crate::formats::extension_of(output).unwrap_or_default();
    if out_ext == "gif" {
        return bad("Non restauro GIF animados: convérteo antes a MP4.");
    }
    let has_audio = streams.iter().any(|s| s["codec_type"] == "audio");
    let total = video_duration(&info);
    let scratch = Scratch::new()?;
    let vidstab = has_vidstab();

    // Primeira pasada da estabilización: mide o movemento.
    let mut stab: Option<String> = None;
    let mut stabilizer = None;
    if plan.stabilize > 0 {
        if vidstab {
            let trf = scratch.0.join("movemento.trf");
            let (shakiness, _) = shaky_params(plan.stabilize);
            let sink = if cfg!(windows) { "NUL" } else { "/dev/null" };
            let mut pre = Vec::new();
            // Se tamén hai que desentrelazar, a medida faise sobre a imaxe xa desentrelazada.
            if plan.deinterlace {
                pre.push("yadif=mode=0:parity=-1:deint=0".to_string());
            }
            pre.push(format!("vidstabdetect=shakiness={shakiness}:accuracy=15:result='{}'", filter_path(&trf)));
            let args: Vec<std::ffi::OsString> = ["-i".into(), input.as_os_str().to_owned(), "-map".into(), "0:v:0".into(), "-vf".into(), pre.join(",").into(), "-f".into(), "null".into(), sink.into()].into_iter().collect();
            ffmpeg::set_progress_phase(0.0, 0.3);
            let r = run_ffmpeg(&args).and_then(|o| check_success(&o));
            ffmpeg::set_progress_phase(0.3, 0.7);
            if let Err(e) = r {
                ffmpeg::set_progress_phase(0.0, 1.0);
                return Err(e);
            }
            stab = Some(stabilizer_filter(plan.stabilize, &trf, true));
            stabilizer = Some("vidstab".to_string());
        } else {
            stab = Some(stabilizer_filter(plan.stabilize, Path::new(""), false));
            stabilizer = Some("deshake".to_string());
        }
    }
    let chain = plan.chain(stab);

    let mut args: Vec<String> = vec!["-map".into(), "0:v:0".into(), "-vf".into(), chain.join(",")];
    if has_audio {
        args.extend(["-map".into(), "0:a:0".into()]);
    }
    args.extend(crate::silence::video_encoder_args(output));
    if has_audio {
        if crate::formats::extension_of(input).as_deref() == Some(out_ext.as_str()) {
            args.extend(["-c:a".into(), "copy".into()]);
        } else {
            args.extend(ffmpeg::keep_audio_args(input, output));
        }
    }
    ffmpeg::set_expected_duration(total);
    let res = ffmpeg::convert_media(input, output, overwrite, &args);
    ffmpeg::set_expected_duration(None);
    ffmpeg::set_progress_phase(0.0, 1.0);
    res.map(|o| RestoreReport { output: o, stabilizer })
}

/// Un fotograma co que quedaría o vídeo en `time` (sen estabilizar), como JPG. Procésase un chisco de vídeo anterior para que o
/// filtro de ruído (que mira os fotogramas veciños) traballe como no resultado de verdade.
pub fn preview_frame(input: &Path, output: &Path, time: f64, plan: &RestorePlan) -> Result<PathBuf> {
    plan.validate()?;
    if !time.is_finite() || time < 0.0 {
        return bad("Instante non válido.");
    }
    if !plan.has_frame_effects() {
        return bad("Non hai nada que ver nun fotograma: a estabilización só se ve ao exportar.");
    }
    // Un fotograma menos medio: o último que sae é o do instante pedido, non o seguinte (o visor mostra ese).
    let fps = ffmpeg::probe(input)
        .ok()
        .and_then(|i| i["streams"].as_array().and_then(|l| l.iter().find(|s| s["codec_type"] == "video").cloned()))
        .and_then(|v| v["avg_frame_rate"].as_str().and_then(|r| r.split_once('/').map(|(n, d)| (n.parse::<f64>().ok(), d.parse::<f64>().ok()))).and_then(|(n, d)| Some(n? / d?)))
        .filter(|f| f.is_finite() && *f >= 1.0 && *f <= 240.0)
        .unwrap_or(25.0);
    let lead = time.min(0.8);
    let pre = vec!["-ss".to_string(), format!("{:.3}", time - lead)];
    let mut chain = plan.chain(None);
    chain.push("scale='min(1280,iw)':-2".into());
    let args: Vec<String> = ["-an", "-t", &format!("{:.4}", lead + 0.5 / fps), "-vf", &chain.join(","), "-q:v", "3", "-update", "1"].iter().map(|s| s.to_string()).collect();
    ffmpeg::convert_media_seek(input, output, true, &pre, &args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_cartafol_de_traballo_borrase_ao_acabar() {
        let p = {
            let s = Scratch::new().unwrap();
            assert!(s.0.is_dir());
            s.0.clone()
        };
        assert!(!p.exists());
    }

    #[test]
    fn un_plan_sen_cambios_esta_baleiro() {
        assert!(RestorePlan::default().is_empty());
        assert!(!RestorePlan { sharpen: 1, ..Default::default() }.is_empty());
        assert!(!RestorePlan { saturation: 0.0, ..Default::default() }.is_empty());
    }

    #[test]
    fn valores_fora_de_rango_non_valen() {
        for p in [
            RestorePlan { stabilize: 4, ..Default::default() },
            RestorePlan { contrast: 5.0, ..Default::default() },
            RestorePlan { brightness: f64::NAN, ..Default::default() },
            RestorePlan { saturation: -1.0, ..Default::default() },
        ] {
            assert!(p.validate().is_err(), "{p:?}");
        }
    }

    #[test]
    fn a_cadea_vai_en_orde() {
        let p = RestorePlan { deinterlace: true, deflicker: true, denoise: 2, sharpen: 1, contrast: 1.2, ..Default::default() };
        let c = p.chain(Some("estab".into()));
        assert_eq!(c.len(), 6);
        assert!(c[0].starts_with("yadif") && c[1].starts_with("deflicker") && c[2] == "estab" && c[3].starts_with("hqdn3d") && c[4].starts_with("eq=") && c[5].starts_with("unsharp"), "{c:?}");
    }
}
