// Recoñecemento de texto en imaxes con Tesseract (en WebAssembly, dentro da app). Os paquetes de idioma descárganse a petición.

import { writable, get } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { call } from "./errors";
import { viewUrl } from "./files";

export const OCR_LANGS: { id: string; label: string }[] = [
  { id: "glg", label: "Galego" },
  { id: "spa", label: "Español" },
  { id: "eng", label: "English" },
  { id: "por", label: "Português" },
  { id: "cat", label: "Català" },
  { id: "fra", label: "Français" },
  { id: "deu", label: "Deutsch" },
  { id: "ita", label: "Italiano" },
];
export const OCR_SIZE = "2–15 MB";

const KEY = "arquivos-do-trasno.ocr-lingua";
/** Idioma que se lembra; por defecto, o da app. */
export function savedLang(appLang: string): string {
  try {
    const v = localStorage.getItem(KEY);
    if (v && OCR_LANGS.some((l) => l.id === v)) return v;
  } catch {
    // ignorar
  }
  return appLang === "es" ? "spa" : appLang === "en" ? "eng" : "glg";
}
export function rememberLang(l: string) {
  try {
    localStorage.setItem(KEY, l);
  } catch {
    // ignorar
  }
}

export const ocrInstalled = writable<string[]>([]);
export const ocrDownload = writable<{ lang: string; progress: number } | null>(null);
export const ocrProgress = writable(0);
/** Mentres se le ou se selecciona o texto da imaxe, as ferramentas do panel de ao lado quedan desactivadas. */
export const ocrActive = writable(false);
let dir = "";

export async function refreshOcr() {
  const r = await call<{ installed: string[]; dir: string | null }>("ocr_languages");
  ocrInstalled.set(r.installed);
  dir = r.dir ?? "";
}

let listening = false;
export async function downloadLang(lang: string): Promise<void> {
  if (!listening) {
    listening = true;
    await listen<{ lang: string; progress: number }>("trasno://ocr-download", (e) => ocrDownload.update((d) => (d ? { ...d, progress: e.payload.progress } : d)));
  }
  ocrDownload.set({ lang, progress: 0 });
  try {
    const r = await call<{ installed: string[] }>("install_ocr_language", { lang });
    ocrInstalled.set(r.installed);
  } finally {
    ocrDownload.set(null);
  }
}

export interface OcrWord {
  text: string;
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}
export interface OcrResult {
  text: string;
  lines: OcrWord[][];
}

let worker: any = null;
let workerLang = "";
const abs = (p: string) => new URL(p, window.location.origin).href;

/** Carpeta (como URL do visor de ficheiros) que ve o motor de OCR para ler os paquetes de idioma. */
async function langPathUrl(lang: string): Promise<string> {
  const file = `${dir}\\${lang}.traineddata`;
  await viewUrl(file); // permite ler o ficheiro
  const origin = convertFileSrc("x").replace(/x$/, "");
  return origin + dir.split(/[\\/]/).map(encodeURIComponent).join("/");
}

let queue: Promise<unknown> = Promise.resolve();
// Lecturas en curso e feitas, por imaxe e idioma: ao cambiar de ficheiro e volver non se perde nin se repite nada.
const inflight = new Map<string, Promise<OcrResult>>();
const finished = new Map<string, OcrResult>();
const keyOf = (path: string, lang: string) => `${path}|${lang}`;
/** Resultado xa lido (ou `"busy"` se aínda se está lendo) desa imaxe, se o hai. */
export function ocrPeek(path: string, lang: string): OcrResult | "busy" | null {
  const k = keyOf(path, lang);
  return finished.get(k) ?? (inflight.has(k) ? "busy" : null);
}
/** As lecturas fanse unha a unha: o motor é un só e cambiar de idioma non pode cortar a lectura en curso. */
export function recognize(path: string, imageUrl: string, lang: string): Promise<OcrResult> {
  const k = keyOf(path, lang);
  const done = finished.get(k);
  if (done) return Promise.resolve(done);
  const going = inflight.get(k);
  if (going) return going;
  const run = queue.then(() => recognizeNow(imageUrl, lang));
  queue = run.catch(() => {});
  inflight.set(k, run);
  run
    .then((r) => {
      finished.set(k, r);
      if (finished.size > 30) finished.delete(finished.keys().next().value as string);
    })
    .catch(() => {})
    .finally(() => inflight.delete(k));
  return run;
}

async function recognizeNow(imageUrl: string, lang: string): Promise<OcrResult> {
  if (!dir) await refreshOcr();
  if (!get(ocrInstalled).includes(lang)) throw new Error("Falta o paquete de idioma.");
  const { createWorker } = await import("tesseract.js");
  if (!worker || workerLang !== lang) {
    await worker?.terminate();
    ocrProgress.set(0);
    worker = await createWorker(lang, 1, {
      workerPath: abs("/ocr/worker.min.js"),
      corePath: abs("/ocr/tesseract-core-simd-lstm.wasm.js"),
      langPath: await langPathUrl(lang),
      gzip: false,
      cacheMethod: "none",
      workerBlobURL: false,
      logger: (m: { status: string; progress: number }) => {
        if (m.status === "recognizing text") ocrProgress.set(m.progress);
      },
    });
    workerLang = lang;
  }
  // Primeira lectura coa imaxe tal cal; se fica pobre (texto pequeno sobre fondos cheos, como un visor 3D ou unha captura),
  // préparase a imaxe de dous xeitos máis (só o claro e só o escuro, ampliada) e queda a lectura que máis texto fiable dea.
  let best = shape((await worker.recognize(imageUrl)).data, 1);
  if (best.score < WEAK) {
    for (const mode of ["bright", "dark"] as const) {
      const prepared = await prepare(imageUrl, mode);
      if (!prepared) break;
      const r = shape((await worker.recognize(prepared.blob)).data, prepared.scale);
      if (r.score > Math.max(MIN_GAIN, best.score * 1.3)) best = r;
    }
  }
  return best.result;
}

/** Palabras fiables que se esperan dunha lectura boa: por baixo diso proban os pasos extra. */
const WEAK = 12;
const MIN_GAIN = 6;

/** Colle o resultado de Tesseract (a `scale` veces o tamaño orixinal), descarta o ruído e devolve as caixas en coordenadas da imaxe. */
function shape(data: any, scale: number): { result: OcrResult; score: number } {
  // Descártanse as palabras que o motor le con pouca confianza (ruído de fotos, texturas...) e os símbolos soltos.
  const keep = (w: any) => {
    const t = String(w.text ?? "").trim();
    if (!t) return false;
    const c = Number(w.confidence ?? 100);
    if (c < 35) return false;
    if (t.length === 1 && !/[\p{L}\p{N}]/u.test(t) && c < 85) return false;
    return true;
  };
  // Se o filtro deixase todo en branco, mellor amosar o que se leu que nada.
  const anyKept = (data.lines ?? []).some((l: any) => (l.words ?? []).some(keep));
  let score = 0;
  const lines: OcrWord[][] = (data.lines ?? [])
    .map((l: any) => {
      const ws = (l.words ?? []).filter((w: any) => (anyKept ? keep(w) : String(w.text ?? "").trim()));
      if (!ws.length) return [];
      for (const w of ws) if (Number(w.confidence ?? 0) >= 60 && /[\p{L}\p{N}]{2,}/u.test(String(w.text))) score++;
      // Todas as palabras dunha liña comparten altura (as letras sen ascendentes non encollen a caixa).
      const y0 = Math.min(...ws.map((w: any) => w.bbox.y0)) / scale;
      const y1 = Math.max(...ws.map((w: any) => w.bbox.y1)) / scale;
      return ws.map((w: any) => ({ text: String(w.text).trim(), x0: w.bbox.x0 / scale, y0, x1: w.bbox.x1 / scale, y1 }));
    })
    .filter((l: OcrWord[]) => l.length);
  return { result: { text: lines.map((l) => l.map((w) => w.text).join(" ")).join("\n"), lines }, score };
}

/** Imaxe preparada para ler: ampliada (as letras pequenas melloran moito) e en branco e negro, quedando só o claro («bright», texto claro
 *  sobre fondo medio) ou só o escuro («dark»). Devolve tamén a ampliación usada para volver ás coordenadas orixinais. */
async function prepare(imageUrl: string, mode: "bright" | "dark"): Promise<{ blob: Blob; scale: number } | null> {
  try {
    const bmp = await createImageBitmap(await (await fetch(imageUrl)).blob());
    const side = Math.max(bmp.width, bmp.height);
    // Non se amplían as imaxes grandes: o texto xa ten tamaño de sobra e custaría memoria e tempo.
    const scale = side <= 900 ? 3 : side <= 1800 ? 2 : 1;
    const cv = new OffscreenCanvas(bmp.width * scale, bmp.height * scale);
    const ctx = cv.getContext("2d", { willReadFrequently: true });
    if (!ctx) return null;
    ctx.imageSmoothingQuality = "high";
    ctx.drawImage(bmp, 0, 0, cv.width, cv.height);
    bmp.close();
    const img = ctx.getImageData(0, 0, cv.width, cv.height);
    const d = img.data;
    for (let i = 0; i < d.length; i += 4) {
      const y = d[i] * 0.299 + d[i + 1] * 0.587 + d[i + 2] * 0.114;
      const ink = mode === "bright" ? y > 185 : y < 75;
      const v = ink ? 0 : 255;
      d[i] = d[i + 1] = d[i + 2] = v;
      d[i + 3] = 255;
    }
    ctx.putImageData(img, 0, 0);
    return { blob: await cv.convertToBlob({ type: "image/png" }), scale };
  } catch {
    return null;
  }
}

/** Esquece os textos lidos dunha imaxe (para reimportala desde cero). */
export function forgetOcr(path: string) {
  for (const k of [...finished.keys()]) if (k.startsWith(path + "|")) finished.delete(k);
  for (const k of [...inflight.keys()]) if (k.startsWith(path + "|")) inflight.delete(k);
}
