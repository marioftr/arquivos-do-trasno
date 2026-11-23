/** Ferramentas de imaxe con canvas: recortar, dividir, xuntar e iconas. */
import { open, type Mime } from "./image";

const MAX_AREA = 16_000_000;
export interface Piece {
  blob: Blob;
  width: number;
  height: number;
}

function paint(w: number, h: number, mime: Mime) {
  const c = document.createElement("canvas");
  c.width = w;
  c.height = h;
  const g = c.getContext("2d");
  if (!g) throw new Error("canvas");
  if (mime === "image/jpeg") {
    g.fillStyle = "#fff";
    g.fillRect(0, 0, w, h);
  }
  g.imageSmoothingQuality = "high";
  return { c, g };
}
const encode = (c: HTMLCanvasElement, mime: Mime, quality = 0.92) => new Promise<Blob>((res, rej) => c.toBlob((b) => (b ? res(b) : rej(new Error("codificar"))), mime, mime === "image/png" ? undefined : quality));
const fit = (w: number, h: number) => Math.min(1, Math.sqrt(MAX_AREA / (w * h)));

/** Recorta á proporción `ratio` (ancho/alto). `align`: 0 deixa a parte de arriba ou esquerda, .5 o centro, 1 a de abaixo ou dereita. */
export async function crop(file: File, ratio: number, align: number, mime: Mime): Promise<Piece> {
  const src = await open(file);
  try {
    let cw = src.width;
    let ch = src.height;
    if (cw / ch > ratio) cw = Math.max(1, Math.round(ch * ratio));
    else ch = Math.max(1, Math.round(cw / ratio));
    const k = fit(cw, ch);
    const w = Math.max(1, Math.round(cw * k));
    const h = Math.max(1, Math.round(ch * k));
    const { c, g } = paint(w, h, mime);
    g.drawImage(src.img, Math.round((src.width - cw) * align), Math.round((src.height - ch) * align), cw, ch, 0, 0, w, h);
    const blob = await encode(c, mime);
    c.width = c.height = 0;
    return { blob, width: w, height: h };
  } finally {
    src.close();
  }
}

/** Corta en `cols` × `rows` partes iguais, en orde de lectura. */
export async function split(file: File, cols: number, rows: number, mime: Mime): Promise<Piece[]> {
  const src = await open(file);
  try {
    const pw = Math.floor(src.width / cols);
    const ph = Math.floor(src.height / rows);
    const out: Piece[] = [];
    for (let r = 0; r < rows; r++) {
      for (let q = 0; q < cols; q++) {
        const { c, g } = paint(pw, ph, mime);
        g.drawImage(src.img, q * pw, r * ph, pw, ph, 0, 0, pw, ph);
        out.push({ blob: await encode(c, mime), width: pw, height: ph });
        c.width = c.height = 0;
      }
    }
    return out;
  } finally {
    src.close();
  }
}

/** Pon as imaxes unha tras outra (en vertical, co ancho da primeira; en horizontal, co alto da primeira). */
export async function stitch(files: File[], vertical: boolean, mime: Mime, progress: (f: number) => void): Promise<Piece> {
  const dims: [number, number][] = [];
  for (const f of files) {
    const s = await open(f);
    dims.push([s.width, s.height]);
    s.close();
  }
  const base = vertical ? dims[0][0] : dims[0][1];
  const sizes = dims.map(([w, h]) => (vertical ? [base, Math.round((h * base) / w)] : [Math.round((w * base) / h), base]) as [number, number]);
  const total = sizes.reduce((a, [w, h]) => a + (vertical ? h : w), 0);
  const k = fit(vertical ? base : total, vertical ? total : base);
  const W = Math.max(1, Math.round((vertical ? base : total) * k));
  const H = Math.max(1, Math.round((vertical ? total : base) * k));
  const { c, g } = paint(W, H, mime);
  let pos = 0;
  for (const [i, f] of files.entries()) {
    progress(i / files.length);
    const s = await open(f);
    const w = Math.round(sizes[i][0] * k);
    const h = Math.round(sizes[i][1] * k);
    g.drawImage(s.img, vertical ? 0 : pos, vertical ? pos : 0, w, h);
    pos += vertical ? h : w;
    s.close();
  }
  const blob = await encode(c, mime);
  c.width = c.height = 0;
  return { blob, width: W, height: H };
}

/** Iconas cadradas (recorte polo centro) de cada tamaño, de maior a menor, cada unha reducida da anterior para que quede nítida. */
export async function squares(file: File, sizes: number[]): Promise<Map<number, Blob>> {
  const src = await open(file);
  const out = new Map<number, Blob>();
  try {
    const order = [...new Set(sizes)].sort((a, b) => b - a);
    const side = Math.min(src.width, src.height);
    let prev: HTMLCanvasElement | null = null;
    for (const s of order) {
      const { c, g } = paint(s, s, "image/png");
      if (prev) g.drawImage(prev, 0, 0, prev.width, prev.height, 0, 0, s, s);
      else g.drawImage(src.img, (src.width - side) / 2, (src.height - side) / 2, side, side, 0, 0, s, s);
      out.set(s, await encode(c, "image/png"));
      if (prev) prev.width = prev.height = 0;
      prev = c;
    }
    if (prev) prev.width = prev.height = 0;
    return out;
  } finally {
    src.close();
  }
}

/** Un ICO con PNG dentro (vale desde Windows Vista). */
export async function ico(pngs: Map<number, Blob>): Promise<Blob> {
  const list = [...pngs.entries()].sort((a, b) => a[0] - b[0]);
  const data = await Promise.all(list.map(async ([, b]) => new Uint8Array(await b.arrayBuffer())));
  const head = new DataView(new ArrayBuffer(6 + 16 * list.length));
  head.setUint16(2, 1, true);
  head.setUint16(4, list.length, true);
  let offset = head.byteLength;
  list.forEach(([size], i) => {
    const e = 6 + i * 16;
    head.setUint8(e, size >= 256 ? 0 : size);
    head.setUint8(e + 1, size >= 256 ? 0 : size);
    head.setUint16(e + 4, 1, true);
    head.setUint16(e + 6, 32, true);
    head.setUint32(e + 8, data[i].length, true);
    head.setUint32(e + 12, offset, true);
    offset += data[i].length;
  });
  return new Blob([head.buffer, ...data] as BlobPart[], { type: "image/x-icon" });
}
