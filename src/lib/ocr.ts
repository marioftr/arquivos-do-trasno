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
/** As lecturas fanse unha a unha: o motor é un só e cambiar de idioma non pode cortar a lectura en curso. */
export function recognize(imageUrl: string, lang: string): Promise<OcrResult> {
  const run = queue.then(() => recognizeNow(imageUrl, lang));
  queue = run.catch(() => {});
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
  const { data } = await worker.recognize(imageUrl);
  const lines: OcrWord[][] = (data.lines ?? []).map((l: any) => (l.words ?? []).filter((w: any) => w.text?.trim()).map((w: any) => ({ text: w.text, x0: w.bbox.x0, y0: w.bbox.y0, x1: w.bbox.x1, y1: w.bbox.y1 }))).filter((l: OcrWord[]) => l.length);
  return { text: (data.text ?? "").trim(), lines };
}
