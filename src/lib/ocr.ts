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
  const { data } = await worker.recognize(imageUrl);
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
  const lines: OcrWord[][] = (data.lines ?? [])
    .map((l: any) => {
      const ws = (l.words ?? []).filter((w: any) => (anyKept ? keep(w) : String(w.text ?? "").trim()));
      if (!ws.length) return [];
      // Todas as palabras dunha liña comparten altura (as letras sen ascendentes non encollen a caixa).
      const y0 = Math.min(...ws.map((w: any) => w.bbox.y0));
      const y1 = Math.max(...ws.map((w: any) => w.bbox.y1));
      return ws.map((w: any) => ({ text: String(w.text).trim(), x0: w.bbox.x0, y0, x1: w.bbox.x1, y1 }));
    })
    .filter((l: OcrWord[]) => l.length);
  return { text: lines.map((l) => l.map((w) => w.text).join(" ")).join("\n"), lines };
}

/** Esquece os textos lidos dunha imaxe (para reimportala desde cero). */
export function forgetOcr(path: string) {
  for (const k of [...finished.keys()]) if (k.startsWith(path + "|")) finished.delete(k);
  for (const k of [...inflight.keys()]) if (k.startsWith(path + "|")) inflight.delete(k);
}
