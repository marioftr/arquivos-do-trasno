/** Que modelos de IA xa están descargados neste navegador (para non dar a entender que se van volver descargar). */

export interface ModelInfo {
  /** Tamaño aproximado da descarga. */
  mb: number;
  /** Anacos que ten que levar a ligazón dun ficheiro gardado para ser deste modelo. */
  match: string[];
}

export const MODELS: Record<string, ModelInfo> = {
  "cut-fast": { mb: 5, match: ["U-2-Netp"] },
  "cut-best": { mb: 46, match: ["isnet-general-use-onnx"] },
  "asr-tiny": { mb: 40, match: ["whisper-tiny"] },
  "asr-base": { mb: 80, match: ["whisper-base"] },
  "mt-to-en": { mb: 120, match: ["opus-mt-ROMANCE-en"] },
  "mt-from-en": { mb: 120, match: ["opus-mt-en-ROMANCE"] },
};

/** Os modelos que están gardados na caché do navegador (a das ferramentas e a de transformers.js). Se non hai caché, ningún. */
export async function downloadedModels(): Promise<Set<string>> {
  const urls: string[] = [];
  try {
    for (const name of await caches.keys()) {
      if (name.startsWith("trasno-web")) continue; // a do propio programa, non de modelos
      const cache = await caches.open(name);
      for (const r of await cache.keys()) urls.push(r.url);
    }
  } catch {
    return new Set();
  }
  const have = new Set<string>();
  for (const [id, m] of Object.entries(MODELS)) if (urls.some((u) => /\.onnx(\?|$)/.test(u) && m.match.every((x) => u.includes(x)))) have.add(id);
  return have;
}

/** Descarga que falta para estes modelos (en MB). */
export const missingMb = (ids: string[], have: Set<string>) => ids.filter((i) => !have.has(i)).reduce((a, i) => a + (MODELS[i]?.mb ?? 0), 0);
