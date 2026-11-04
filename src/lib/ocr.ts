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

// A barra de progreso cobre todas as lecturas dunha imaxe: avanza sempre e chega ao 90 % co traballo feito; o resto é o remate.
let pOff = 0;
let pSpan = 1;

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
        // Nunca retrocede: as lecturas extra non fan saltar a barra de volta ao principio.
        if (m.status === "recognizing text") ocrProgress.update((p) => Math.max(p, pOff + m.progress * pSpan));
      },
    });
    workerLang = lang;
  }
  ocrProgress.set(0);
  const loaded = await loadBitmap(imageUrl);
  // 1) A imaxe tal cal.
  pOff = 0;
  pSpan = 0.4;
  let primary = collect((await worker.recognize(imageUrl)).data, 1);
  // 2) Pasos extra. Ampliada (as letras pequenas melloran moito) se a imaxe non é grande. E se a primeira lectura é pobre
  //    (logotipos, texto solto, letras pequenas sobre fondos cheos como un visor 3D ou unha captura) a imaxe lese tamén co
  //    modo de «texto disperso» de Tesseract (que busca palabras soltas en vez de páxinas) e preparada de varios xeitos:
  //    ampliada, só o claro e só o escuro. Faise unha lectura de cada e xúntase o que cada unha achegue de novo; a que máis
  //    texto fiable dea pasa a ser a principal.
  const weak = primary.score < WEAK;
  const small = !!loaded && loaded.bmp.width * loaded.bmp.height <= 1_600_000;
  const plan: { mode: PrepMode | "raw"; sparse: boolean }[] = weak
    ? [{ mode: "raw", sparse: true }, ...(small ? [{ mode: "up" as const, sparse: true }] : []), { mode: "bright", sparse: true }, { mode: "dark", sparse: true }]
    : small
      ? [{ mode: "up", sparse: false }]
      : [];
  const passes: Pass[] = [primary];
  let step = 0;
  for (const p of plan) {
    const prepared = p.mode === "raw" ? { blob: null as Blob | null, scale: 1 } : loaded ? await prepare(loaded, p.mode) : null;
    if (!prepared) continue;
    pOff = 0.4 + (0.5 * step) / plan.length;
    pSpan = 0.5 / plan.length;
    step++;
    await worker.setParameters({ tessedit_pageseg_mode: p.sparse ? "11" : "3" });
    passes.push(collect((await worker.recognize(prepared.blob ?? imageUrl)).data, prepared.scale));
  }
  if (plan.some((p) => p.sparse)) await worker.setParameters({ tessedit_pageseg_mode: "3" });
  loaded?.bmp.close();
  ocrProgress.set(1);
  // A principal é a de máis texto fiable (a primeira, se empatan); as demais achegan o que a ela lle falta.
  let best = 0;
  passes.forEach((q, i) => {
    if (q.score > passes[best].score) best = i;
  });
  const [main] = passes.splice(best, 1);
  return merge(main, passes);
}

/** Palabras fiables que se esperan dunha lectura boa: por baixo diso proban os pasos extra. */
const WEAK = 12;
const MIN_GAIN = 6;
/** Confianza mínima das palabras que unha lectura extra pode engadir á principal. */
const EXTRA_CONF = 75;

interface Pass {
  /** Liñas tal como as deu Tesseract (en coordenadas da imaxe orixinal) e a confianza de cada palabra. */
  lines: { w: OcrWord; conf: number }[][];
  score: number;
}

/** Colle o resultado de Tesseract (a `scale` veces o tamaño orixinal) e descarta o ruído: palabras de pouca confianza e símbolos soltos. */
function collect(data: any, scale: number): Pass {
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
  const lines = (data.lines ?? [])
    .map((l: any) => {
      const ws = (l.words ?? []).filter((w: any) => (anyKept ? keep(w) : String(w.text ?? "").trim()));
      if (!ws.length) return [];
      for (const w of ws) if (Number(w.confidence ?? 0) >= 60 && /[\p{L}\p{N}]{2,}/u.test(String(w.text))) score++;
      // Todas as palabras dunha liña comparten altura (as letras sen ascendentes non encollen a caixa).
      const y0 = Math.min(...ws.map((w: any) => w.bbox.y0)) / scale;
      const y1 = Math.max(...ws.map((w: any) => w.bbox.y1)) / scale;
      return ws.map((w: any) => ({ w: { text: String(w.text).trim(), x0: w.bbox.x0 / scale, y0, x1: w.bbox.x1 / scale, y1 }, conf: Number(w.confidence ?? 0) }));
    })
    .filter((l: unknown[]) => l.length);
  return { lines, score };
}

/** A lectura principal máis o que as outras ven e ela non: palabras fiables que non se solapan con ningunha xa lida. */
function merge(primary: Pass, others: Pass[]): OcrResult {
  const have: OcrWord[] = primary.lines.flat().map((x) => x.w);
  const overlaps = (w: OcrWord) =>
    have.some((h) => {
      const ix = Math.min(w.x1, h.x1) - Math.max(w.x0, h.x0);
      const iy = Math.min(w.y1, h.y1) - Math.max(w.y0, h.y0);
      if (ix <= 0 || iy <= 0) return false;
      const inter = ix * iy;
      const small = Math.min((w.x1 - w.x0) * (w.y1 - w.y0), (h.x1 - h.x0) * (h.y1 - h.y0));
      return inter / Math.max(1, small) > 0.3;
    });
  const added: OcrWord[] = [];
  for (const o of others) {
    for (const w of o.lines.flat()) {
      if (w.conf < EXTRA_CONF || !/[\p{L}\p{N}]{2,}/u.test(w.w.text) || overlaps(w.w)) continue;
      have.push(w.w);
      added.push(w.w);
    }
  }
  // As palabras novas agrúpanse en liñas (mesma altura, xuntas) e van despois do resto.
  added.sort((p, q) => (p.y0 + p.y1) / 2 - (q.y0 + q.y1) / 2 || p.x0 - q.x0);
  const extraLines: OcrWord[][] = [];
  for (const w of added) {
    const h = w.y1 - w.y0;
    const last = extraLines[extraLines.length - 1];
    const prev = last?.[last.length - 1];
    if (prev && Math.abs((prev.y0 + prev.y1) / 2 - (w.y0 + w.y1) / 2) < h * 0.5 && w.x0 - prev.x1 < h * 1.2 && w.x0 >= prev.x0) last.push(w);
    else extraLines.push([w]);
  }
  const lines = [...primary.lines.map((l) => l.map((x) => x.w)), ...extraLines];
  return { text: lines.map((l) => l.map((w) => w.text).join(" ")).join("\n"), lines };
}

type PrepMode = "up" | "bright" | "dark";
interface Loaded {
  bmp: ImageBitmap;
}
async function loadBitmap(imageUrl: string): Promise<Loaded | null> {
  try {
    return { bmp: await createImageBitmap(await (await fetch(imageUrl)).blob()) };
  } catch {
    return null;
  }
}

/** Imaxe preparada para ler: ampliada (as letras pequenas melloran moito) e, segundo o modo, só en escala de grises co contraste
 *  estirado («up»), ou en branco e negro quedando só o claro («bright») ou só o escuro («dark»). Devolve tamén a ampliación usada. */
async function prepare(src: Loaded, mode: PrepMode): Promise<{ blob: Blob; scale: number } | null> {
  try {
    const { bmp } = src;
    const side = Math.max(bmp.width, bmp.height);
    // Non se amplían as imaxes grandes: o texto xa ten tamaño de sobra e custaría memoria e tempo.
    const scale = side <= 900 ? 3 : side <= 2200 ? 2 : 1;
    const cv = new OffscreenCanvas(bmp.width * scale, bmp.height * scale);
    const ctx = cv.getContext("2d", { willReadFrequently: true });
    if (!ctx) return null;
    ctx.imageSmoothingQuality = "high";
    ctx.drawImage(bmp, 0, 0, cv.width, cv.height);
    const img = ctx.getImageData(0, 0, cv.width, cv.height);
    const d = img.data;
    if (mode === "up") {
      // Escala de grises e contraste estirado entre o 1 % e o 99 % dos tons.
      const hist = new Uint32Array(256);
      for (let i = 0; i < d.length; i += 4) hist[(d[i] * 0.299 + d[i + 1] * 0.587 + d[i + 2] * 0.114) | 0]++;
      const total = d.length / 4;
      let lo = 0;
      let hi = 255;
      for (let c = 0; lo < 255 && c + hist[lo] < total * 0.01; lo++) c += hist[lo];
      for (let c = 0; hi > 0 && c + hist[hi] < total * 0.01; hi--) c += hist[hi];
      const range = Math.max(1, hi - lo);
      for (let i = 0; i < d.length; i += 4) {
        const y = d[i] * 0.299 + d[i + 1] * 0.587 + d[i + 2] * 0.114;
        const v = Math.max(0, Math.min(255, ((y - lo) * 255) / range));
        d[i] = d[i + 1] = d[i + 2] = v;
        d[i + 3] = 255;
      }
    } else {
      for (let i = 0; i < d.length; i += 4) {
        const y = d[i] * 0.299 + d[i + 1] * 0.587 + d[i + 2] * 0.114;
        const ink = mode === "bright" ? y > 185 : y < 75;
        d[i] = d[i + 1] = d[i + 2] = ink ? 0 : 255;
        d[i + 3] = 255;
      }
    }
    ctx.putImageData(img, 0, 0);
    return { blob: await cv.convertToBlob({ type: "image/png" }), scale };
  } catch {
    return null;
  }
}

// ---- idioma do texto lido: palabras moi comúns de cada idioma (as propias valen máis)
const STOP: Record<string, { own: string[]; common: string[] }> = {
  glg: { own: ["non", "unha", "unhas", "dun", "dunha", "tamén", "cando", "coa", "cos", "nin", "vostede", "moi", "ás", "aos", "máis", "hai", "está", "están"], common: ["que", "da", "do", "das", "dos", "ao", "é", "para", "por", "con", "sen", "pero", "como", "os", "as", "un", "uns", "se", "no", "na", "nos", "nas", "foi"] },
  spa: { own: ["el", "los", "las", "también", "más", "está", "están", "hay", "muy", "sin", "del", "unos", "unas", "pero", "una", "cuando", "porque"], common: ["que", "de", "la", "en", "un", "y", "a", "con", "para", "por", "es", "se", "su", "sus", "al", "lo", "como", "fue"] },
  eng: { own: ["the", "and", "of", "is", "that", "this", "with", "was", "are", "you", "have", "from", "not", "they", "will", "would", "there", "their"], common: ["to", "in", "it", "for", "as", "on", "be", "by", "an", "or", "at", "we", "his", "her"] },
  por: { own: ["não", "uma", "você", "também", "são", "mais", "muito", "tem", "com", "mas", "dos", "das", "pelo", "pela", "está", "foi"], common: ["que", "de", "o", "os", "as", "um", "em", "para", "por", "se", "ao", "do", "da", "no", "na", "e", "é"] },
  cat: { own: ["els", "les", "amb", "això", "però", "també", "és", "són", "molt", "hi", "dels", "una", "perquè", "quan", "aquest", "aquesta"], common: ["que", "de", "la", "el", "i", "en", "un", "per", "al", "als", "del", "no", "es", "se", "com"] },
  fra: { own: ["le", "les", "des", "est", "une", "pour", "dans", "qui", "pas", "avec", "sur", "aux", "du", "ne", "vous", "nous", "sont", "mais", "cette", "être"], common: ["la", "de", "et", "en", "un", "que", "au", "ce", "se", "il", "elle", "ou", "plus"] },
  deu: { own: ["der", "die", "das", "und", "ist", "nicht", "ein", "eine", "mit", "für", "auf", "den", "dem", "zu", "von", "auch", "sich", "im", "ich", "sie", "wir"], common: ["es", "in", "an", "als", "wie", "bei", "nach", "aus", "oder", "wird"] },
  ita: { own: ["il", "che", "non", "per", "una", "sono", "con", "della", "del", "gli", "più", "anche", "questo", "come", "nel", "nella", "dei", "delle", "è"], common: ["di", "e", "la", "le", "un", "in", "si", "da", "lo", "ma", "ha", "al"] },
};
/** Idioma máis probable do texto (id de Tesseract), ou `null` se non hai texto abondo ou non se distingue con claridade. */
export function detectLang(text: string): string | null {
  const words = text.toLowerCase().match(/\p{L}+/gu) ?? [];
  if (words.length < 15) return null;
  const counts = new Map<string, number>();
  for (const w of words) counts.set(w, (counts.get(w) ?? 0) + 1);
  const scores = Object.entries(STOP).map(([id, t]) => {
    let sc = 0;
    for (const w of t.own) sc += (counts.get(w) ?? 0) * 3;
    for (const w of t.common) sc += counts.get(w) ?? 0;
    return [id, sc] as const;
  });
  scores.sort((x, y) => y[1] - x[1]);
  const [top, second] = scores;
  return top[1] >= 9 && top[1] >= second[1] * 1.4 ? top[0] : null;
}

/** Esquece os textos lidos dunha imaxe (para reimportala desde cero). */
export function forgetOcr(path: string) {
  for (const k of [...finished.keys()]) if (k.startsWith(path + "|")) finished.delete(k);
  for (const k of [...inflight.keys()]) if (k.startsWith(path + "|")) inflight.delete(k);
}
