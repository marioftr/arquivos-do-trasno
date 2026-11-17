//! Edición de audio e vídeo nunha soa pasada: o usuario axusta todo (corte, velocidade, volume, efectos de son…) e sae un único
//! ficheiro novo. Ir aplicando cada axuste por separado obrigaría a recodificar unha e outra vez e a perder calidade.

use std::path::{Path, PathBuf};

use crate::ffmpeg::{self, Loudness};
use crate::{EngineError, Result};

/// O que se lle fai ao ficheiro. Os valores por defecto non cambian nada.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditPlan {
    /// Inicio do corte, en segundos (0 = desde o principio).
    pub start: f64,
    /// Fin do corte, en segundos (`None` = ata o final).
    pub end: Option<f64>,
    /// Factor de velocidade (0,25 a 4). 1 = igual.
    pub speed: f64,
    /// Multiplicador de volume (0 a 4). 1 = igual.
    pub volume: f64,
    /// Quita a pista de son dun vídeo.
    pub mute: bool,
    /// Reduce o ruído de fondo.
    pub denoise: bool,
    /// Dálle a volta ao son.
    pub reverse: bool,
    /// Esvaece ao principio durante tantos segundos (0 = non).
    pub fade_in: f64,
    /// Esvaece ao final durante tantos segundos (0 = non).
    pub fade_out: f64,
    /// Graves en dB (−12 a 12; 0 = non).
    pub bass: f64,
    /// Agudos en dB (−12 a 12; 0 = non).
    pub treble: f64,
    /// Eco: atraso en milisegundos (0 = sen eco).
    pub echo_ms: f64,
    /// Iguala o volume percibido (substitúe o multiplicador de volume).
    pub normalize: Option<Loudness>,
}

impl Default for EditPlan {
    fn default() -> Self {
        EditPlan { start: 0.0, end: None, speed: 1.0, volume: 1.0, mute: false, denoise: false, reverse: false, fade_in: 0.0, fade_out: 0.0, bass: 0.0, treble: 0.0, echo_ms: 0.0, normalize: None }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EditReport {
    pub output: PathBuf,
    /// Duración que se agarda do resultado, en segundos (se se coñece a do orixinal).
    pub duration: Option<f64>,
    /// Se non houbo que recodificar a imaxe (só se cortou ou se tocou o son).
    pub video_copied: bool,
}

fn bad<T>(m: &str) -> Result<T> {
    Err(EngineError::IntegrityCheck(m.into()))
}

impl EditPlan {
    fn trimmed(&self) -> bool {
        self.start > 0.0 || self.end.is_some()
    }
    /// Axustes que cambian o son (sen contar o corte nin a velocidade).
    fn audio_effects(&self) -> bool {
        self.volume != 1.0 || self.denoise || self.reverse || self.fade_in > 0.0 || self.fade_out > 0.0 || self.bass != 0.0 || self.treble != 0.0 || self.echo_ms > 0.0 || self.normalize.is_some()
    }
    pub fn is_empty(&self) -> bool {
        !self.trimmed() && self.speed == 1.0 && !self.mute && !self.audio_effects()
    }
    /// Só un corte: cópiase sen recodificar.
    fn trim_only(&self) -> bool {
        self.trimmed() && self.speed == 1.0 && !self.mute && !self.audio_effects()
    }

    pub fn validate(&self) -> Result<()> {
        let fin = |v: f64| v.is_finite();
        if !fin(self.start) || self.start < 0.0 {
            return bad("O inicio do corte non é válido.");
        }
        if let Some(e) = self.end {
            if !fin(e) || e <= self.start {
                return bad("O fin do corte ten que vir despois do inicio.");
            }
        }
        if !fin(self.speed) || !(0.25..=4.0).contains(&self.speed) {
            return bad("A velocidade ten que estar entre 0,25 e 4.");
        }
        if !fin(self.volume) || !(0.0..=4.0).contains(&self.volume) {
            return bad("O volume ten que estar entre 0 e 4.");
        }
        for f in [self.fade_in, self.fade_out] {
            if !fin(f) || !(0.0..=600.0).contains(&f) || (f > 0.0 && f < 0.05) {
                return bad("A duración do efecto non é válida.");
            }
        }
        if !fin(self.bass) || !fin(self.treble) || !(-12.0..=12.0).contains(&self.bass) || !(-12.0..=12.0).contains(&self.treble) {
            return bad("Os decibelios teñen que estar entre -12 e 12.");
        }
        if !fin(self.echo_ms) || (self.echo_ms != 0.0 && !(20.0..=2000.0).contains(&self.echo_ms)) {
            return bad("O atraso do eco non é válido.");
        }
        Ok(())
    }

    /// Cadea de filtros de son, sen a igualación de volume. `out_dur` é a duración do resultado (para o esvaecer final).
    fn audio_chain(&self, out_dur: Option<f64>) -> Result<Vec<String>> {
        let mut c: Vec<String> = Vec::new();
        if self.denoise {
            c.push("afftdn=nr=12".into());
        }
        if self.reverse {
            c.push("areverse".into());
        }
        if self.bass != 0.0 {
            c.push(format!("bass=g={}", self.bass));
        }
        if self.treble != 0.0 {
            c.push(format!("treble=g={}", self.treble));
        }
        if self.echo_ms > 0.0 {
            c.push(format!("aecho=0.8:0.6:{}:0.4", self.echo_ms));
        }
        if self.speed != 1.0 {
            c.push(ffmpeg::atempo_chain(self.speed));
        }
        if self.normalize.is_none() && self.volume != 1.0 {
            c.push(format!("volume={}", self.volume));
        }
        if self.fade_in > 0.0 {
            c.push(format!("afade=t=in:st=0:d={}", self.fade_in));
        }
        if self.fade_out > 0.0 {
            let Some(d) = out_dur else { return bad("Non se sabe canto dura o ficheiro: non se pode esvaecer o final.") };
            c.push(format!("afade=t=out:st={}:d={}", (d - self.fade_out).max(0.0), self.fade_out));
        }
        Ok(c)
    }
}

fn media_duration(info: &serde_json::Value) -> Option<f64> {
    info["format"]["duration"].as_str().and_then(|x| x.parse::<f64>().ok()).filter(|d| d.is_finite() && *d > 0.0)
}

/// Aplica todo o plan nun ficheiro novo.
pub fn apply_plan(input: &Path, output: &Path, overwrite: bool, plan: &EditPlan) -> Result<EditReport> {
    run(input, output, overwrite, plan, false)
}

/// Un anaco curto (ata `seconds`) co plan aplicado, para escoitar como queda antes de exportar. Non se iguala o volume (esixe medir
/// o ficheiro enteiro) e os esvaeceres só se aplican se o anaco chega ao principio ou ao final do corte. A saída é MP4 (vídeo) ou WAV (son).
pub fn preview_plan(input: &Path, output: &Path, plan: &EditPlan, at: f64, seconds: f64) -> Result<EditReport> {
    plan.validate()?;
    let info = ffmpeg::probe(input)?;
    let total = media_duration(&info);
    let end = plan.end.or(total);
    let start = if at.is_finite() { at.max(plan.start) } else { plan.start };
    let start = match end {
        // Se o visor está despois do final do corte, enséñase o final.
        Some(e) if start >= e - 0.2 => (e - seconds.min(e - plan.start)).max(plan.start),
        _ => start,
    };
    let clip_end = end.map(|e| (start + seconds).min(e)).unwrap_or(start + seconds);
    let mut p = plan.clone();
    p.start = start;
    p.end = Some(clip_end);
    p.normalize = None;
    if start > plan.start {
        p.fade_in = 0.0;
    }
    if end.map(|e| clip_end < e - 1e-6).unwrap_or(true) {
        p.fade_out = 0.0;
    }
    if p.is_empty() {
        // Sen axustes que oír: o mesmo tramo tal cal.
        p.start = start;
    }
    run(input, output, true, &p, true)
}

fn run(input: &Path, output: &Path, overwrite: bool, plan: &EditPlan, preview: bool) -> Result<EditReport> {
    plan.validate()?;
    if plan.is_empty() {
        return bad("Non hai nada que aplicar: todos os axustes están como no orixinal.");
    }
    let info = ffmpeg::probe(input)?;
    let streams = info["streams"].as_array().cloned().unwrap_or_default();
    let has_video = streams.iter().any(|s| s["codec_type"] == "video" && s["disposition"]["attached_pic"] != 1);
    let has_audio = streams.iter().any(|s| s["codec_type"] == "audio");
    let total = media_duration(&info);
    if let (Some(t), true) = (total, plan.start > 0.0) {
        if plan.start >= t {
            return bad("O inicio do corte está despois do final do ficheiro.");
        }
    }
    let wants_audio = plan.audio_effects() || plan.speed != 1.0;
    if plan.mute && !has_video {
        return bad("Un ficheiro de son non se pode deixar mudo: non quedaría nada.");
    }
    if plan.audio_effects() && !has_audio && !plan.mute {
        return bad("Este ficheiro non ten son para aplicarlle ese axuste.");
    }
    if plan.trim_only() && !preview {
        return ffmpeg::trim_media(input, output, overwrite, plan.start, plan.end).map(|o| EditReport { output: o, duration: total.map(|t| plan.end.unwrap_or(t).min(t) - plan.start), video_copied: true });
    }

    // Duración do tramo que se le e do resultado (a velocidade encolle ou estira).
    let seg_end = plan.end.map(|e| total.map(|t| e.min(t)).unwrap_or(e)).or(total);
    let seg_dur = seg_end.map(|e| e - plan.start).filter(|d| *d > 0.0);
    let out_dur = seg_dur.map(|d| d / plan.speed);

    // O tramo léese con `-ss`/`-t` antes da entrada: así o corte é exacto e os filtros só ven o que queda.
    let mut pre: Vec<String> = Vec::new();
    if plan.start > 0.0 {
        pre.extend(["-ss".into(), plan.start.to_string()]);
    }
    if let Some(d) = plan.end.map(|e| e - plan.start) {
        pre.extend(["-t".into(), d.to_string()]);
    }

    let chain = if has_audio && !plan.mute { plan.audio_chain(out_dur)? } else { Vec::new() };
    let prefix = if chain.is_empty() { String::new() } else { format!("{},", chain.join(",")) };

    let mut args: Vec<String> = Vec::new();
    // A imaxe só se cópia se non cambia a velocidade nin se corta (un corte con copia saltaría ao fotograma clave máis próximo e a
    // imaxe quedaría desaliñada co son que si se corta exacto).
    let video_copy = has_video && plan.speed == 1.0 && !plan.trimmed();
    if has_video {
        if plan.speed != 1.0 {
            args.extend(["-filter:v".into(), format!("setpts=PTS/{}", plan.speed)]);
        }
        if video_copy {
            args.extend(["-c:v".into(), "copy".into()]);
        } else if preview {
            args.extend(["-c:v".into(), "libx264".into(), "-preset".into(), "veryfast".into(), "-crf".into(), "26".into(), "-pix_fmt".into(), "yuv420p".into()]);
            // A imaxe de preview non precisa ser grande.
            if plan.speed == 1.0 {
                args.extend(["-vf".into(), "scale=-2:'min(480,ih)'".into()]);
            }
        } else {
            args.extend(crate::silence::video_encoder_args(output));
        }
    }

    if plan.mute {
        args.push("-an".into());
    } else if has_audio && (wants_audio || !video_copy) {
        let mut af = chain.join(",");
        if let Some(target) = plan.normalize {
            // Dúas pasadas sobre o que de verdade vai quedar (despois do corte e dos demais efectos).
            ffmpeg::set_progress_phase(0.0, 0.3);
            let m = ffmpeg::measure_loudness_with(input, target, &pre, &prefix);
            ffmpeg::set_progress_phase(0.3, 0.7);
            let m = m?;
            let ln = format!(
                "loudnorm=I={}:TP={}:LRA={}:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true",
                target.lufs, target.peak, target.range, m.lufs, m.peak, m.range, m.thresh, m.offset
            );
            af = if af.is_empty() { ln } else { format!("{af},{ln}") };
        }
        if !af.is_empty() {
            args.extend(["-af".into(), af]);
        }
        if preview {
            if has_video {
                args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "160k".into()]);
            }
        } else {
            args.extend(ffmpeg::keep_audio_args(input, output));
            if plan.normalize.is_some() && !args.iter().any(|a| a == "-ar") {
                // O filtro traballa por dentro a 192 kHz: sen fixar a frecuencia, o contedor podería quedarse con ela.
                args.extend(["-ar".into(), "48000".into()]);
            }
        }
    }
    ffmpeg::set_expected_duration(out_dur);
    let res = ffmpeg::convert_media_seek(input, output, overwrite, &pre, &args);
    ffmpeg::set_expected_duration(None);
    ffmpeg::set_progress_phase(0.0, 1.0);
    res.map(|o| EditReport { output: o, duration: out_dur, video_copied: video_copy })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_plan_sen_cambios_esta_baleiro() {
        assert!(EditPlan::default().is_empty());
        assert!(!EditPlan { speed: 2.0, ..Default::default() }.is_empty());
        assert!(!EditPlan { start: 1.0, ..Default::default() }.is_empty());
    }

    #[test]
    fn os_valores_fora_de_rango_non_valen() {
        for p in [
            EditPlan { speed: 5.0, ..Default::default() },
            EditPlan { volume: -1.0, ..Default::default() },
            EditPlan { start: 5.0, end: Some(3.0), ..Default::default() },
            EditPlan { bass: 20.0, ..Default::default() },
            EditPlan { echo_ms: 5.0, ..Default::default() },
            EditPlan { fade_in: 0.01, ..Default::default() },
        ] {
            assert!(p.validate().is_err(), "{p:?}");
        }
    }

    #[test]
    fn a_cadea_de_son_vai_en_orde_e_o_esvaecer_final_conta_a_velocidade() {
        let p = EditPlan { speed: 2.0, volume: 0.5, fade_out: 1.0, bass: 3.0, ..Default::default() };
        let c = p.audio_chain(Some(5.0)).unwrap();
        assert_eq!(c, vec!["bass=g=3", "atempo=2", "volume=0.5", "afade=t=out:st=4:d=1"]);
        // Igualar o volume substitúe o multiplicador.
        let n = EditPlan { volume: 2.0, normalize: Some(Loudness::default()), ..Default::default() };
        assert!(n.audio_chain(None).unwrap().is_empty());
    }
}
