/** Que modelos de IA xa están descargados neste navegador (para non dar a entender que se van volver descargar). */

export interface ModelInfo {
  /** Anacos que ten que levar a ligazón dun ficheiro gardado para ser deste modelo. */
  match: string[];
}

export const MODELS: Record<string, ModelInfo> = {
  "cut-fast": { match: ["U-2-Netp"] },
  "cut-best": { match: ["isnet-general-use-onnx"] },
  "asr-tiny": { match: ["whisper-tiny"] },
  "asr-base": { match: ["whisper-base"] },
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
