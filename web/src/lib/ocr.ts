/** Recoñecer texto en imaxes con Tesseract (WebAssembly). O motor e os idiomas serve os a propia web: nada sae do aparello. */
import { render } from "./image";
import type { Ctx } from "./files";

const abs = (p: string) => new URL(p, location.origin).href;

interface Worker {
  recognize(image: Blob): Promise<{ data: { text: string; confidence: number } }>;
  terminate(): Promise<unknown>;
}
let worker: Worker | null = null;
let workerLang = "";
let current: Ctx | null = null;

async function open(lang: string, ctx: Ctx): Promise<Worker> {
  if (worker && workerLang === lang) return worker;
  await worker?.terminate();
  worker = null;
  ctx.progress(null, "…");
  const { createWorker } = await import("tesseract.js");
  // O núcleo con SIMD vai máis rápido; os navegadores vellos usan o outro.
  const simd = WebAssembly.validate(new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 123, 3, 2, 1, 0, 10, 10, 1, 8, 0, 65, 0, 253, 15, 253, 98, 11]));
  worker = (await createWorker(lang, 1, {
    workerPath: abs("/ocr/worker.min.js"),
    corePath: abs(simd ? "/ocr/tesseract-core-simd-lstm.wasm.js" : "/ocr/tesseract-core-lstm.wasm.js"),
    langPath: abs("/ocr"),
    logger: (m: { status: string; progress: number }) => m.status === "recognizing text" && current?.progress(m.progress),
  })) as unknown as Worker;
  workerLang = lang;
  return worker;
}

export interface Read {
  text: string;
  confidence: number;
}

/** Lé o texto dunha imaxe. `lang` é o código de Tesseract (glg, spa, eng, cat). */
export async function readText(file: File, lang: string, ctx: Ctx, label: string): Promise<Read> {
  current = ctx;
  ctx.progress(null, label);
  // Pasar pola canvas aplica a orientación da foto e limita o tamaño (a memoria dun móbil é xusta).
  const png = await render(file, { mime: "image/png", maxSide: 3200 });
  const w = await open(lang, ctx);
  ctx.progress(0, label);
  const r = await w.recognize(png.blob);
  return { text: r.data.text.trim(), confidence: Math.round(r.data.confidence) };
}
