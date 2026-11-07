// Recoñecemento de texto en imaxes con Tesseract (en WebAssembly, dentro da app). Os paquetes de idioma descárganse a petición.

import { writable, get } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { call } from "./errors";
import { viewUrl } from "./files";
import { tr } from "./i18n";
import { detectTextRegions, seedRegions, planRegions, isolatedGlyphs, segmentGlyphs, type Box, type Region } from "./ocrRegions";

export const OCR_LANGS: { id: string; label: string; code: string }[] = [
  { id: "glg", label: "Galego", code: "gl" },
  { id: "spa", label: "Español", code: "es" },
  { id: "eng", label: "English", code: "en" },
  { id: "por", label: "Português", code: "pt" },
  { id: "cat", label: "Català", code: "ca" },
  { id: "fra", label: "Français", code: "fr" },
  { id: "deu", label: "Deutsch", code: "de" },
  { id: "ita", label: "Italiano", code: "it" },
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
  // 1) A imaxe tal cal, pero decodificada polo navegador e pasada a PNG: Tesseract non le AVIF, HEIC nin WebP e, ademais, ignora a orientación
  //    EXIF, polo que unha foto xirada daría caixas fóra do sitio. O navegador amósaa ben, así que así coincide co que se ve.
  pOff = 0;
  pSpan = 0.3;
  const plain = loaded ? await prepare(loaded, "plain") : null;
  let primary = collect((await worker.recognize(plain?.blob ?? imageUrl)).data, 1);
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
    const prepared = p.mode === "raw" ? { blob: plain?.blob ?? null, scale: 1 } : loaded ? await prepare(loaded, p.mode) : null;
    if (!prepared) continue;
    pOff = 0.3 + (0.3 * step) / plan.length;
    pSpan = 0.3 / plan.length;
    step++;
    await worker.setParameters({ tessedit_pageseg_mode: p.sparse ? "11" : "3" });
    passes.push(collect((await worker.recognize(prepared.blob ?? imageUrl)).data, prepared.scale, 1));
  }
  if (plan.some((p) => p.sparse)) await worker.setParameters({ tessedit_pageseg_mode: "3" });
  // A principal é a de máis texto fiable (a primeira, se empatan); as demais achegan o que a ela lle falta.
  let best = 0;
  passes.forEach((q, i) => {
    if (q.score > passes[best].score) best = i;
  });
  const [main] = passes.splice(best, 1);
  let result = merge(main, passes);
  // 3) Zoom nas zonas con texto. As letras pequenas sobre fondos cheos (capturas, visores, etiquetas) non as le ben unha lectura da
  //    imaxe enteira: búscase onde hai texto (as palabras xa lidas e, ademais, as zonas con moitos bordos pequenos), recórtase cada
  //    zona, amplíase ata que as letras teñan un bo tamaño e léese de novo só ela. O que sae mellor ou que faltaba engádese.
  //    Un texto en páxina (moitas palabras xa lidas) non o necesita.
  if (loaded && result.score < PAGE_WORDS) {
    try {
      result = await refine(loaded, result);
    } catch {
      // se o zoom falla, vale o que había
    }
  }
  // 4) Signos soltos (número de páxina, eixos dun visor 3D...): non forman palabras, así que as lecturas anteriores os deixan.
  if (loaded) {
    try {
      await looseGlyphs(loaded.bmp, result);
    } catch {
      // sen signos soltos: vale o que había
    }
  }
  await worker.setParameters({ tessedit_pageseg_mode: "3" });
  loaded?.bmp.close();
  ocrProgress.set(1);
  const lines = result.lines.map((l) => l.map((x) => x.w));
  return { text: lines.map((l) => l.map((w) => w.text).join(" ")).join("\n"), lines };
}

/** Con tantas palabras fiables xa é un texto en páxina: as zonas ampliadas non achegarían nada. */
const PAGE_WORDS = 100;
const MAX_REGIONS = 8;

/** Imaxe en grises (ata `max` px de lado) para buscar zonas de texto; devolve tamén canto se reduciu. */
function grayForDetection(bmp: ImageBitmap, max = 1600): { lum: Uint8Array; w: number; h: number; f: number } | null {
  try {
    const f = Math.max(1, Math.max(bmp.width, bmp.height) / max);
    const w = Math.max(1, Math.round(bmp.width / f));
    const h = Math.max(1, Math.round(bmp.height / f));
    const cv = new OffscreenCanvas(w, h);
    const ctx = cv.getContext("2d", { willReadFrequently: true });
    if (!ctx) return null;
    ctx.drawImage(bmp, 0, 0, w, h);
    const d = ctx.getImageData(0, 0, w, h).data;
    const lum = new Uint8Array(w * h);
    for (let i = 0, j = 0; i < d.length; i += 4, j++) lum[j] = (d[i] * 77 + d[i + 1] * 150 + d[i + 2] * 29) >> 8;
    return { lum, w, h, f };
  } catch {
    return null;
  }
}

/** Zona recortada da imaxe, ampliada `scale` veces e en grises co contraste estirado (as letras claras sobre fondo escuro léense igual). */
async function prepareRegion(bmp: ImageBitmap, box: Box, scale: number): Promise<Blob | null> {
  try {
    const w = Math.max(1, Math.round((box.x1 - box.x0) * scale));
    const h = Math.max(1, Math.round((box.y1 - box.y0) * scale));
    const cv = new OffscreenCanvas(w, h);
    const ctx = cv.getContext("2d", { willReadFrequently: true });
    if (!ctx) return null;
    ctx.imageSmoothingQuality = "high";
    ctx.drawImage(bmp, box.x0, box.y0, box.x1 - box.x0, box.y1 - box.y0, 0, 0, w, h);
    const img = ctx.getImageData(0, 0, w, h);
    const d = img.data;
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
      d[i] = d[i + 1] = d[i + 2] = Math.max(0, Math.min(255, ((y - lo) * 255) / range));
      d[i + 3] = 255;
    }
    ctx.putImageData(img, 0, 0);
    return await cv.convertToBlob({ type: "image/png" });
  } catch {
    return null;
  }
}

async function refine(loaded: Loaded, base: Pass): Promise<Pass> {
  const { bmp } = loaded;
  const W = bmp.width;
  const H = bmp.height;
  const seeds: Box[] = base.lines
    .flat()
    .filter((x) => x.conf >= 60 && alnumCount(x.w.text) >= 2)
    .map((x) => x.w);
  const g = grayForDetection(bmp);
  const found = g ? detectTextRegions(g.lum, g.w, g.h).map((r) => ({ ...r, box: { x0: r.box.x0 * g.f, y0: r.box.y0 * g.f, x1: r.box.x1 * g.f, y1: r.box.y1 * g.f } })) : [];
  const regions = planRegions([...seedRegions(seeds, W, H), ...found], W, H, MAX_REGIONS);
  if (!regions.length) return base;
  await worker.setParameters({ tessedit_pageseg_mode: "11" });
  const extra: Pass[] = [];
  let n = 0;
  for (const r of regions) {
    pOff = 0.6 + (0.35 * n++) / regions.length;
    pSpan = 0.35 / regions.length;
    // Marxe para non cortar letras e ampliación: que as letras queden arredor de 30 px de alto (sen pasar de ~3000 px de lado).
    const m = Math.max(6, (r.lineH ?? 10) * 0.8);
    const box: Box = { x0: Math.max(0, Math.floor(r.box.x0 - m)), y0: Math.max(0, Math.floor(r.box.y0 - m)), x1: Math.min(W, Math.ceil(r.box.x1 + m)), y1: Math.min(H, Math.ceil(r.box.y1 + m)) };
    const bw = box.x1 - box.x0;
    const bh = box.y1 - box.y0;
    let scale = Math.max(2, Math.min(5, 30 / (r.lineH ?? 10)));
    scale = Math.min(scale, 3200 / Math.max(bw, bh), Math.sqrt(8_000_000 / (bw * bh)));
    if (scale < 1.5) continue;
    const blob = await prepareRegion(bmp, box, scale);
    if (!blob) continue;
    const pass = collect((await worker.recognize(blob)).data, scale, r.seeded ? -1 : 1, box.x0, box.y0);
    // Nunha zona onde xa se viu texto, abonda con menos confianza; nunha achada só polos bordos, esíxese máis.
    // Unha zona achada só polos bordos (sen texto visto antes) ten que dar polo menos dúas palabras fiables: unha soa é quizais ruído.
    if (!r.seeded && pass.score < 2) continue;
    pass.minConf = r.seeded ? 62 : EXTRA_CONF + 5;
    extra.push(pass);
  }
  const merged = merge(base, extra);
  try {
    await glyphPass(bmp, regions, merged);
  } catch {
    // sen cifras soltas: vale o que había
  }
  return merged;
}

/** Pon unha palabra onde lle toca: na liña da mesma fila (por orde de x) ou, se non hai, entre as liñas, segundo a altura. */
function placeWord(pass: Pass, item: Pass["lines"][number][number]) {
  const cy = (item.w.y0 + item.w.y1) / 2;
  const hh = item.w.y1 - item.w.y0;
  for (const line of pass.lines) {
    const ly = line.reduce((a, q) => a + (q.w.y0 + q.w.y1) / 2, 0) / line.length;
    const lh = line.reduce((a, q) => a + (q.w.y1 - q.w.y0), 0) / line.length;
    if (Math.abs(ly - cy) < Math.min(hh, lh) * 0.6) {
      const at = line.findIndex((q) => q.w.x0 > item.w.x0);
      line.splice(at < 0 ? line.length : at, 0, item);
      return;
    }
  }
  const at = pass.lines.findIndex((line) => line.reduce((a, q) => a + (q.w.y0 + q.w.y1) / 2, 0) / line.length > cy);
  pass.lines.splice(at < 0 ? pass.lines.length : at, 0, [item]);
}

/** Letras e cifras soltas: ver `isolatedGlyphs`. Cada candidato recórtase, ámplíase, pásase a tinta negra sobre branco (a cor da tinta
 *  sácase da diferenza co fondo, así vale unha letra verde sobre gris ou branca sobre negro) e léese como un só carácter. Só se aceptan
 *  letras e cifras lidas con moita confianza; as máis confundibles cun trazo («l», «I», «1», «|», «o», «0») piden aínda máis. */
async function looseGlyphs(bmp: ImageBitmap, pass: Pass): Promise<void> {
  const heights = pass.lines.flat().filter((x) => x.conf >= 60).map((x) => x.w.y1 - x.w.y0).sort((a, b) => a - b);
  if (!heights.length) return;
  const lineH = heights[heights.length >> 1];
  const cv = new OffscreenCanvas(bmp.width, bmp.height);
  const ctx = cv.getContext("2d", { willReadFrequently: true });
  if (!ctx) return;
  ctx.drawImage(bmp, 0, 0);
  const rgba = ctx.getImageData(0, 0, bmp.width, bmp.height).data;
  const words = pass.lines.flat();
  // Cada mancha de bordos pártese en signos (as letras e as raias dun debuxo van por separado) e descártase o que xa é parte dunha palabra lida.
  const cands = isolatedGlyphs(rgba, bmp.width, bmp.height, lineH, 30)
    .flatMap((c) => segmentGlyphs(rgba, bmp.width, bmp.height, c.box, lineH).map((box) => ({ ...c, box })))
    .filter((c) => c.box.y1 - c.box.y0 <= lineH * 3)
    .filter((c) => !words.some((q) => Math.min(q.w.x1, c.box.x1) - Math.max(q.w.x0, c.box.x0) > 0 && Math.min(q.w.y1, c.box.y1) - Math.max(q.w.y0, c.box.y0) > 0));
  if (!cands.length) return;
  await worker.setParameters({ tessedit_pageseg_mode: "10" });
  const AMBIG = /^[lI1|iLoO0]$/;
  /** Le o signo da caixa `b` con letras duns `target` px de alto (Tesseract le mellor un signo de ~25 px ca un enorme: ampliado de máis
   *  «inventa» letras de máis). Devolve o carácter e a confianza, ou null. */
  const readAt = async (b: Box, target: number): Promise<{ t: string; conf: number } | null> => {
    const bh = b.y1 - b.y0;
    const pad = Math.max(3, Math.round(bh * 0.4));
    const sx0 = Math.max(0, b.x0 - pad);
    const sy0 = Math.max(0, b.y0 - pad);
    const sw = Math.min(bmp.width, b.x1 + pad) - sx0;
    const sh = Math.min(bmp.height, b.y1 + pad) - sy0;
    const sc = Math.max(2.5, Math.min(10, target / bh));
    const cw = Math.max(1, Math.round(sw * sc));
    const ch = Math.max(1, Math.round(sh * sc));
    const m = 20;
    const c2 = new OffscreenCanvas(cw + m * 2, ch + m * 2);
    const g2 = c2.getContext("2d", { willReadFrequently: true });
    if (!g2) return null;
    g2.fillStyle = "#fff";
    g2.fillRect(0, 0, c2.width, c2.height);
    g2.imageSmoothingQuality = "high";
    g2.drawImage(bmp, sx0, sy0, sw, sh, m, m, cw, ch);
    const im = g2.getImageData(m, m, cw, ch);
    const d = im.data;
    // Cor do fondo: a mediana da beira do recorte; tinta = o que máis se afasta dela.
    const edge: number[][] = [[], [], []];
    const take = (x: number, y: number) => {
      const i = (y * cw + x) * 4;
      for (let k = 0; k < 3; k++) edge[k].push(d[i + k]);
    };
    for (let x = 0; x < cw; x++) (take(x, 0), take(x, ch - 1));
    for (let y = 0; y < ch; y++) (take(0, y), take(cw - 1, y));
    const bg = edge.map((a) => a.sort((p, q) => p - q)[a.length >> 1]);
    const dist = new Uint8Array(cw * ch);
    const dh = new Uint32Array(256);
    for (let i = 0, j = 0; i < d.length; i += 4, j++) dh[(dist[j] = Math.min(255, Math.max(Math.abs(d[i] - bg[0]), Math.abs(d[i + 1] - bg[1]), Math.abs(d[i + 2] - bg[2]))))]++;
    const thr = otsu(dh, cw * ch);
    if (thr < 12) return null; // sen contraste: non hai signo
    for (let i = 0, j = 0; i < d.length; i += 4, j++) {
      d[i] = d[i + 1] = d[i + 2] = dist[j] > thr ? 0 : 255;
      d[i + 3] = 255;
    }
    g2.putImageData(im, m, m);
    const res = (await worker.recognize(await c2.convertToBlob({ type: "image/png" }))).data;
    const word = (res.words ?? [])[0] ?? { text: res.text, confidence: res.confidence };
    const t = String(word.text ?? "").replace(/[^\p{L}\p{N}]/gu, "");
    return t.length === 1 ? { t, conf: Number(word.confidence ?? 0) } : null;
  };
  for (const c of cands) {
    const b = c.box;
    const bh = b.y1 - b.y0;
    const bw = b.x1 - b.x0;
    // Dúas ampliacións; vale a lectura máis segura.
    let best: { t: string; conf: number } | null = null;
    for (const target of [24, 34]) {
      const r = await readAt(b, target);
      if (r && (!best || r.conf > best.conf)) best = r;
    }
    if (!best) continue;
    // Os signos moi estreitos (unha raia alta) confúndense con letras: piden a mesma confianza ca os ambiguos.
    if (best.conf < (AMBIG.test(best.t) || bw < bh * 0.35 ? 92 : 70)) continue;
    // Un trazo recto illado confúndese con «l», «I» ou «1»: non se aceptan se o signo é unha liña moi fina.
    if (AMBIG.test(best.t) && bw < bh * 0.12) continue;
    placeWord(pass, { w: { text: best.t, x0: b.x0, y0: b.y0, x1: b.x1, y1: b.y1 }, conf: best.conf, good: true });
  }
  pass.score = pass.lines.flat().filter((x) => x.good).length;
}

/** Limiar de Otsu dun histograma de 256 niveis. */
function otsu(hist: Uint32Array, total: number): number {
  let sum = 0;
  for (let i = 0; i < 256; i++) sum += i * hist[i];
  let wB = 0;
  let sB = 0;
  let best = 0;
  let thr = 128;
  for (let t = 0; t < 256; t++) {
    wB += hist[t];
    if (!wB) continue;
    const wF = total - wB;
    if (!wF) break;
    sB += t * hist[t];
    const v = wB * wF * (sB / wB - (sum - sB) / wF) ** 2;
    if (v > best) (best = v), (thr = t);
  }
  return thr;
}

/** Cifras soltas dunha columna (o «0» dunha táboa de datos): Tesseract non le un carácter só, así que se buscan á man os signos
 *  pequenos que quedan á dereita dunha cifra xa lida, na mesma fila e co tamaño das letras de arredor, e lese cada un recortado e
 *  moi ampliado. Só se aceptan se o lido pode ser un cero («0», «o», «O») e hai unha cifra ao seu lado: o contexto fai de garantía. */
async function glyphPass(bmp: ImageBitmap, regions: Region[], pass: Pass): Promise<void> {
  let budget = 40;
  await worker.setParameters({ tessedit_pageseg_mode: "10" });
  const numeric = (t: string) => /^\d[\d.,]*$/.test(t);
  for (const r of regions) {
    if (!r.seeded || !r.lineH || budget <= 0) continue;
    const x0 = Math.max(0, Math.floor(r.box.x0));
    const y0 = Math.max(0, Math.floor(r.box.y0));
    // Máis ancha cara á dereita: as columnas de cifras seguen moito despois da etiqueta.
    const w = Math.min(bmp.width, Math.ceil(r.box.x1 + r.lineH * 30)) - x0;
    const h = Math.min(bmp.height, Math.ceil(r.box.y1)) - y0;
    if (w < 8 || h < 8 || w * h > 4_000_000) continue;
    const cv = new OffscreenCanvas(w, h);
    const ctx = cv.getContext("2d", { willReadFrequently: true });
    if (!ctx) continue;
    ctx.drawImage(bmp, x0, y0, w, h, 0, 0, w, h);
    const d = ctx.getImageData(0, 0, w, h).data;
    const lum = new Uint8Array(w * h);
    const hist = new Uint32Array(256);
    for (let i = 0, j = 0; i < d.length; i += 4, j++) hist[(lum[j] = (d[i] * 77 + d[i + 1] * 150 + d[i + 2] * 29) >> 8)]++;
    const thr = otsu(hist, w * h);
    let below = 0;
    for (let i = 0; i <= thr; i++) below += hist[i];
    // A tinta é o que menos hai na zona (as letras ocupan menos ca o fondo).
    const bright = below > (w * h) / 2;
    const ink = (i: number) => (bright ? lum[i] > thr : lum[i] <= thr);
    // Compoñentes conexas da tinta.
    const seen = new Uint8Array(w * h);
    const blobs: Box[] = [];
    const st: number[] = [];
    for (let s0 = 0; s0 < w * h; s0++) {
      if (seen[s0] || !ink(s0)) continue;
      let bx0 = w, by0 = h, bx1 = -1, by1 = -1, px = 0;
      st.push(s0);
      seen[s0] = 1;
      while (st.length) {
        const i = st.pop()!;
        const x = i % w;
        const y = (i / w) | 0;
        px++;
        if (x < bx0) bx0 = x;
        if (x > bx1) bx1 = x;
        if (y < by0) by0 = y;
        if (y > by1) by1 = y;
        for (let dy = -1; dy <= 1; dy++) {
          for (let dx = -1; dx <= 1; dx++) {
            const xx = x + dx;
            const yy = y + dy;
            if (xx < 0 || yy < 0 || xx >= w || yy >= h) continue;
            const k = yy * w + xx;
            if (!seen[k] && ink(k)) (seen[k] = 1), st.push(k);
          }
        }
      }
      const bw = bx1 - bx0 + 1;
      const bh = by1 - by0 + 1;
      const lh = r.lineH;
      if (bh < lh * 0.55 || bh > lh * 1.6 || bw < 2 || bw > bh * 1.3 || px < bw * bh * 0.25) continue;
      blobs.push({ x0: x0 + bx0, y0: y0 + by0, x1: x0 + bx1 + 1, y1: y0 + by1 + 1 });
    }
    blobs.sort((a, b) => a.x0 - b.x0 || a.y0 - b.y0);
    for (const b of blobs) {
      if (budget <= 0) break;
      const bh = b.y1 - b.y0;
      const words = pass.lines.flat();
      // Que non estea xa lido, e que á súa esquerda, na mesma fila, haxa unha cifra.
      if (words.some((q) => Math.min(q.w.x1, b.x1) - Math.max(q.w.x0, b.x0) > 0 && Math.min(q.w.y1, b.y1) - Math.max(q.w.y0, b.y0) > 0)) continue;
      let left: { line: Pass["lines"][number]; x1: number } | null = null;
      for (const line of pass.lines) {
        for (const q of line) {
          if (!numeric(q.w.text)) continue;
          const oy = Math.min(q.w.y1, b.y1) - Math.max(q.w.y0, b.y0);
          if (oy < bh * 0.6 || q.w.x1 > b.x0 + 1 || b.x0 - q.w.x1 > r.lineH! * 16) continue;
          if (!left || q.w.x1 > left.x1) left = { line, x1: q.w.x1 };
        }
      }
      if (!left) continue;
      budget--;
      // Recorte moi ampliado, en negro sobre branco e con marxe branca.
      const sc = Math.max(4, Math.min(10, 40 / bh));
      const m = 20;
      const pad = 3;
      const cw = Math.round((b.x1 - b.x0 + pad * 2) * sc);
      const ch = Math.round((bh + pad * 2) * sc);
      const c2 = new OffscreenCanvas(cw + m * 2, ch + m * 2);
      const g2 = c2.getContext("2d", { willReadFrequently: true });
      if (!g2) continue;
      g2.fillStyle = "#fff";
      g2.fillRect(0, 0, c2.width, c2.height);
      g2.imageSmoothingQuality = "high";
      g2.drawImage(bmp, b.x0 - pad, b.y0 - pad, b.x1 - b.x0 + pad * 2, bh + pad * 2, m, m, cw, ch);
      const im = g2.getImageData(m, m, cw, ch);
      for (let i = 0; i < im.data.length; i += 4) {
        const y = (im.data[i] * 77 + im.data[i + 1] * 150 + im.data[i + 2] * 29) >> 8;
        const on = bright ? y > thr : y <= thr;
        im.data[i] = im.data[i + 1] = im.data[i + 2] = on ? 0 : 255;
        im.data[i + 3] = 255;
      }
      g2.putImageData(im, m, m);
      const res = (await worker.recognize(await c2.convertToBlob({ type: "image/png" }))).data;
      const word = (res.words ?? [])[0] ?? { text: res.text, confidence: res.confidence };
      const t = String(word.text ?? "").replace(/[^A-Za-z0-9]/g, "");
      if (!/^[oO0]$/.test(t) || Number(word.confidence ?? 0) < 40) continue;
      left.line.push({ w: { text: "0", x0: b.x0, y0: b.y0, x1: b.x1, y1: b.y1 }, conf: 70, good: true });
      left.x1 = b.x1;
    }
  }
  pass.score = pass.lines.flat().filter((x) => x.good).length;
}

/** Palabras fiables que se esperan dunha lectura boa: por baixo diso proban os pasos extra. */
const WEAK = 12;
const MIN_GAIN = 6;
/** Confianza mínima das palabras que unha lectura extra pode engadir á principal. */
const EXTRA_CONF = 80;

interface Pass {
  /** Liñas tal como as deu Tesseract (en coordenadas da imaxe orixinal), a confianza de cada palabra e se é «fiable». */
  lines: { w: OcrWord; conf: number; good: boolean }[][];
  /** Número de palabras fiables. Con 0 non hai texto: o que Tesseract «le» nunha foto ou nunha textura é ruído. */
  score: number;
  /** Confianza mínima para que unha palabra desta lectura entre na principal (por defecto, `EXTRA_CONF`). */
  minConf?: number;
}

const alnumCount = (t: string) => (t.match(/[\p{L}\p{N}]/gu) ?? []).length;
/** Palabra fiable: ten letras ou cifras de verdade (non símbolos) e a confianza esixida segundo o seu tamaño. Nas lecturas extra
 *  (imaxes preparadas) esíxese máis, porque deixan moito máis ruído. */
function isGood(text: string, conf: number, level: Level): boolean {
  const t = text.trim();
  const a = alnumCount(t);
  if (a < 2 || a / t.length < 0.6) return false;
  // `level`: 1 esixente (lecturas preparadas), 0 normal, -1 relaxado (zonas onde xa se viu texto seguro: o que hai ao carón é moi
  // probablemente texto tamén, e non se quere perder unha palabra por ter unha letra dubidosa).
  if (a >= 4) return conf >= (level > 0 ? 85 : level < 0 ? 62 : 75);
  if (a === 3) return conf >= (level > 0 ? 88 : level < 0 ? 70 : 78);
  return conf >= (level < 0 ? 85 : 90);
}
type Level = -1 | 0 | 1;

/** Colle o resultado de Tesseract (a `scale` veces o tamaño orixinal) e descarta o ruído. Se non hai nin unha palabra fiable,
 *  a lectura queda baleira (en vez de amosar lixo); se si, consérvanse tamén as palabras de menos confianza que a acompañan. */
function collect(data: any, scale: number, level: Level = 0, ox = 0, oy = 0): Pass {
  const keep = (w: any) => {
    const t = String(w.text ?? "").trim();
    if (!t) return false;
    const c = Number(w.confidence ?? 100);
    if (c < 35) return false;
    if (t.length === 1 && !/[\p{L}\p{N}]/u.test(t) && c < 85) return false;
    return true;
  };
  let score = 0;
  const lines = (data.lines ?? [])
    .map((l: any) => {
      const ws = (l.words ?? []).filter(keep);
      if (!ws.length) return [];
      // Todas as palabras dunha liña comparten altura (as letras sen ascendentes non encollen a caixa).
      const y0 = Math.min(...ws.map((w: any) => w.bbox.y0)) / scale + oy;
      const y1 = Math.max(...ws.map((w: any) => w.bbox.y1)) / scale + oy;
      return ws.map((w: any) => {
        const text = String(w.text).trim();
        const conf = Number(w.confidence ?? 0);
        const good = isGood(text, conf, level);
        if (good) score++;
        return { w: { text, x0: w.bbox.x0 / scale + ox, y0, x1: w.bbox.x1 / scale + ox, y1 }, conf, good };
      });
    })
    .filter((l: unknown[]) => l.length);
  return score ? { lines, score } : { lines: [], score: 0 };
}

/** A lectura principal máis o que as outras ven e ela non: palabras fiables que non se solapan con ningunha xa lida. Se unha palabra
 *  nova cae sobre unha xa lida e a ve con moita máis confianza (a mesma palabra mellor lida), substitúea. */
function merge(primary: Pass, others: Pass[]): Pass {
  const lines = primary.lines.map((l) => l.map((x) => ({ ...x })));
  const added: Pass["lines"][number] = [];
  const overlaps = (w: OcrWord, h: OcrWord) => {
    const ix = Math.min(w.x1, h.x1) - Math.max(w.x0, h.x0);
    const iy = Math.min(w.y1, h.y1) - Math.max(w.y0, h.y0);
    if (ix <= 0 || iy <= 0) return false;
    const inter = ix * iy;
    const small = Math.min((w.x1 - w.x0) * (w.y1 - w.y0), (h.x1 - h.x0) * (h.y1 - h.y0));
    return inter / Math.max(1, small) > 0.3;
  };
  for (const o of others) {
    const min = o.minConf ?? EXTRA_CONF;
    for (const x of o.lines.flat()) {
      // Só entra o que a lectura extra ve con seguridade; unha letra solta, só se vai acompañada doutras palabras (o «A» dun logotipo).
      const single = alnumCount(x.w.text) === 1 && x.conf >= 92 && o.score >= 2;
      if (!(x.good && x.conf >= min) && !single) continue;
      const hits = [...lines.flat(), ...added].filter((h) => overlaps(x.w, h.w));
      if (!hits.length) {
        added.push({ ...x });
      } else if (hits.length === 1 && x.good && x.conf > hits[0].conf + 12) {
        Object.assign(hits[0], x);
      }
    }
  }
  // As palabras novas agrúpanse en liñas (mesma altura, xuntas) e van despois do resto.
  added.sort((p, q) => (p.w.y0 + p.w.y1) / 2 - (q.w.y0 + q.w.y1) / 2 || p.w.x0 - q.w.x0);
  const extraLines: Pass["lines"] = [];
  for (const x of added) {
    const w = x.w;
    const h = w.y1 - w.y0;
    const last = extraLines[extraLines.length - 1];
    const prev = last?.[last.length - 1]?.w;
    if (prev && Math.abs((prev.y0 + prev.y1) / 2 - (w.y0 + w.y1) / 2) < h * 0.5 && w.x0 - prev.x1 < h * 1.2 && w.x0 >= prev.x0) last.push(x);
    else extraLines.push([x]);
  }
  const all = [...lines, ...extraLines];
  return { lines: all, score: all.flat().filter((x) => x.good).length };
}

type PrepMode = "plain" | "up" | "bright" | "dark";
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
    const scale = mode === "plain" ? 1 : side <= 900 ? 3 : side <= 2200 ? 2 : 1;
    const cv = new OffscreenCanvas(bmp.width * scale, bmp.height * scale);
    const ctx = cv.getContext("2d", { willReadFrequently: true });
    if (!ctx) return null;
    ctx.imageSmoothingQuality = "high";
    ctx.drawImage(bmp, 0, 0, cv.width, cv.height);
    if (mode === "plain") return { blob: await cv.convertToBlob({ type: "image/png" }), scale: 1 };
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

// ---- o que di o trasno despois de ler o texto dunha imaxe
const OCR_NONE = [
  "O trasno mirou ben «{name}» e non lle viu nin unha letra. Se hai texto, proba con outro idioma ou cunha imaxe máis nítida.",
  "Nada de nada: o trasno non atopou texto en «{name}». Ou non hai letras, ou están moi escondidas.",
  "O trasno apertou os ollos, espreitou polas esquinas e non atopou texto en «{name}».",
];
const OCR_FEW = [
  "O trasno atopou só un cachiño de texto en «{name}»: pouca cousa, pero xa o podes copiar.",
  "Un pouquiño de texto leu o trasno en «{name}». Se esperabas máis, proba con outro idioma ou cunha imaxe máis nítida.",
  "Só unhas palabras en «{name}», pero o trasno non deixou escapar ningunha.",
];
const OCR_SOME = [
  "O trasno atopou texto en «{name}» ({n} palabras) e xa o podes seleccionar e copiar.",
  "Ben! O trasno leu o texto de «{name}»: selecciónao co rato ou cópiao todo dun golpe.",
  "O trasno sacoulle as letras a «{name}» ({n} palabras): xa son túas para copiar.",
];
const OCR_LOT = [
  "Boh, canto texto! O trasno leu unha parrafada enorme en «{name}» ({n} palabras). Revísao antes de fiarte de cada letra.",
  "O trasno ven de ler un montón de texto en «{name}»: {n} palabras. Cópiao todo dun golpe co botón.",
  "Que parrafada! O trasno quedou sen folgos lendo «{name}» ({n} palabras), pero aí tes o texto.",
];
let lastOcrLine = "";
/** Que di o trasno (e que queda no historial) segundo canto texto atopou: nada, un pouco, bastante ou moito. */
export function ocrVerdict(res: { text: string; lines: unknown[] }, name: string): { say: string; note: string; found: boolean } {
  const n = res.lines.length ? res.text.split(/\s+/).filter(Boolean).length : 0;
  const list = !n ? OCR_NONE : n <= 5 ? OCR_FEW : n <= 80 ? OCR_SOME : OCR_LOT;
  let pick = list[Math.floor(Math.random() * list.length)];
  if (pick === lastOcrLine) pick = list[(list.indexOf(pick) + 1) % list.length];
  lastOcrLine = pick;
  return {
    say: tr(pick, { name, n }),
    note: n ? tr("O trasno leu o texto de «{name}»: {n} palabras", { name, n }) : tr("O trasno non atopou texto en «{name}»", { name }),
    found: n > 0,
  };
}
