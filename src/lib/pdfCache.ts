// PDF compartido entre o visor e as grellas de miniaturas: un documento aberto e unhas miniaturas por ficheiro,
// que se gardan mentres a aplicación siga aberta. Cambiar de pestana ou de ficheiro non obriga a cargar de novo.

import * as pdfjs from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { writable } from "svelte/store";
import { viewUrl } from "./files";

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

const MAX_DOCS = 12;
const docs = new Map<string, Promise<pdfjs.PDFDocumentProxy>>();

/** Abre o PDF (ou devolve o que xa estaba aberto). */
export function getPdf(path: string): Promise<pdfjs.PDFDocumentProxy> {
  const hit = docs.get(path);
  if (hit) {
    // Úsase agora: pasa ao final da cola de expulsión.
    docs.delete(path);
    docs.set(path, hit);
    return hit;
  }
  const p = (async () => {
    const res = await fetch(await viewUrl(path));
    return pdfjs.getDocument({ data: new Uint8Array(await res.arrayBuffer()) }).promise;
  })();
  docs.set(path, p);
  p.catch(() => docs.delete(path));
  while (docs.size > MAX_DOCS) {
    const [oldPath, old] = docs.entries().next().value as [string, Promise<pdfjs.PDFDocumentProxy>];
    docs.delete(oldPath); // as miniaturas quedan: pesan pouco
    old.then((d) => d.destroy()).catch(() => {});
  }
  return p;
}

/** Onde estaba cada PDF (páxina e zoom), para volver exactamente ao mesmo sitio. */
export const viewState = new Map<string, { page: number; scale: number; fit: "none" | "width" | "page" }>();

/** Esquece un ficheiro (por exemplo, ao quitalo da lista). */
export function forgetPdf(path: string) {
  const d = docs.get(path);
  docs.delete(path);
  thumbs.delete(path);
  d?.then((x) => x.destroy()).catch(() => {});
}

// ------------------------------------------------------------ miniaturas

const thumbs = new Map<string, Map<number, string>>();
/** Sobe cada vez que chega unha miniatura nova, para que as grellas se repinten. */
export const thumbVersion = writable(0);

export const cachedThumbs = (path: string): Map<number, string> => {
  let m = thumbs.get(path);
  if (!m) thumbs.set(path, (m = new Map()));
  return m;
};

interface Job {
  path: string;
  n: number;
}
const high: Job[] = []; // as que se están vendo: a última pedida vai primeiro
let low: Job[] = []; // o resto, en segundo plano
let working = false;
const SCALE = 0.35;

function take(): Job | undefined {
  while (high.length) {
    const j = high.pop()!;
    if (!cachedThumbs(j.path).has(j.n)) return j;
  }
  while (low.length) {
    const j = low.shift()!;
    if (!cachedThumbs(j.path).has(j.n)) return j;
  }
  return undefined;
}

async function work() {
  if (working) return;
  working = true;
  try {
    for (let j = take(); j; j = take()) {
      try {
        const doc = await getPdf(j.path);
        const page = await doc.getPage(j.n);
        const vp = page.getViewport({ scale: SCALE });
        const canvas = document.createElement("canvas");
        canvas.width = Math.ceil(vp.width);
        canvas.height = Math.ceil(vp.height);
        const ctx = canvas.getContext("2d")!;
        ctx.fillStyle = "#fff";
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        await page.render({ canvasContext: ctx, viewport: vp }).promise;
        const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, "image/jpeg", 0.7));
        if (blob) {
          cachedThumbs(j.path).set(j.n, URL.createObjectURL(blob));
          thumbVersion.update((v) => v + 1);
        }
        page.cleanup();
      } catch {
        // unha páxina que non se debuxa queda en branco
      }
      // Respira entre páxinas: a interface non se queda parada aínda que haxa 300.
      await new Promise((r) => setTimeout(r, high.length ? 0 : 24));
    }
  } finally {
    working = false;
  }
}

/** Pide unha miniatura con prioridade (a que o usuario ten diante). */
export function wantThumb(path: string, n: number) {
  if (cachedThumbs(path).has(n)) return;
  const i = high.findIndex((j) => j.path === path && j.n === n);
  if (i >= 0) high.splice(i, 1);
  high.push({ path, n });
  work();
}

/** Xa non se ve: sae da cola de prioridade (pasa ao fondo). */
export function dropThumb(path: string, n: number) {
  const i = high.findIndex((j) => j.path === path && j.n === n);
  if (i >= 0) {
    high.splice(i, 1);
    low.push({ path, n });
  }
}

/** Debuxa o resto das páxinas en segundo plano, por orde, cando non hai nada máis urxente. */
export function prefetchThumbs(path: string, total: number) {
  low = low.filter((j) => j.path !== path);
  for (let n = 1; n <= total; n++) if (!cachedThumbs(path).has(n)) low.push({ path, n });
  work();
}
