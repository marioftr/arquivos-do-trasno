/** Voz a texto con Whisper no navegador (o núcleo de modelos é @app/lib/mt). */
import { tr } from "@app/lib/i18n";
import { load, type Ctx, type Lang } from "@app/lib/mt";
import type { Cue } from "./subs";

export { translate } from "@app/lib/mt";
export type { Lang };

// ---------------------------------------------------------------- voz a texto

const WHISPER_LANG: Record<Lang, string> = { glg: "galician", spa: "spanish", eng: "english", cat: "catalan" };
const WINDOW = 30; // segundos por tramo (o que Whisper ve de unha vez)

/** Transcribe audio mono de 16 kHz (Float32) en subtítulos. */
export async function transcribe(pcm: Float32Array, lang: Lang, size: "tiny" | "base", ctx: Ctx): Promise<Cue[]> {
  const asr = await load("automatic-speech-recognition", `Xenova/whisper-${size}`, ctx, tr("Descargando o modelo de voz (só a primeira vez)…"));
  const rate = 16000;
  const per = WINDOW * rate;
  const total = Math.max(1, Math.ceil(pcm.length / per));
  const cues: Cue[] = [];
  for (let i = 0; i < total; i++) {
    ctx.progress(i / total, tr("Transcribindo…"));
    const part = pcm.subarray(i * per, (i + 1) * per);
    if (part.length < rate / 2) break;
    const out = await asr(part, { language: WHISPER_LANG[lang], task: "transcribe", return_timestamps: true });
    const base = i * WINDOW;
    for (const c of (out.chunks ?? []) as { text: string; timestamp: [number, number | null] }[]) {
      const text = c.text.trim();
      if (!text) continue;
      const start = base + (c.timestamp[0] ?? 0);
      cues.push({ start, end: base + (c.timestamp[1] ?? Math.min(WINDOW, part.length / rate)), text });
    }
  }
  return cues;
}

