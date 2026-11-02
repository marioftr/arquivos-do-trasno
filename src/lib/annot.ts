// Anotacións sobre unha imaxe (debuxo a man, liñas, frechas, figuras e texto), como no Paint ou nos Recortes de Windows.
// As figuras gárdanse como vectores en píxeles da imaxe orixinal; só se «queiman» na imaxe ao gardar a copia.

import { writable, get } from "svelte/store";
import { cssFamily } from "./fonts";

export type Tool = "select" | "pen" | "highlight" | "line" | "arrow" | "rect" | "ellipse" | "text" | "eraser";
export type ShapeType = "pen" | "highlight" | "line" | "arrow" | "rect" | "ellipse" | "text";

export interface Shape {
  id: number;
  type: ShapeType;
  color: string;
  width: number;
  fill?: boolean;
  /** Graos, en sentido horario, arredor do centro da figura. */
  angle?: number;
  /** pen e highlight */
  pts?: [number, number][];
  /** line, arrow, rect, ellipse */
  x1?: number;
  y1?: number;
  x2?: number;
  y2?: number;
  /** text: esquina superior esquerda */
  x?: number;
  y?: number;
  text?: string;
  size?: number;
  family?: string;
  bold?: boolean;
  italic?: boolean;
}

export interface Style {
  color: string;
  width: number;
  fill: boolean;
  family: string;
  size: number;
  bold: boolean;
  italic: boolean;
}

/** O separador de anotar está aberto: o visor deixa de mover a imaxe co rato esquerdo. */
export const annoActive = writable(false);
export const annoTool = writable<Tool>("pen");
export const annoStyle = writable<Style>({ color: "#ff3b30", width: 5, fill: false, family: "Helvetica", size: 40, bold: false, italic: false });
export const shapes = writable<Shape[]>([]);
export const selected = writable<number | null>(null);
/** Imaxe á que pertence o debuxado actual: ao cambiar de imaxe, empeza de cero. */
export const annoPath = writable("");
export const setAnnoPath = (p: string) => annoPath.set(p);
/** O debuxado de cada imaxe lémbrase ao cambiar de ficheiro (mentres a app siga aberta). */
const stash = new Map<string, { shapes: Shape[]; past: Shape[][]; future: Shape[][] }>();
/** Pasa ao debuxado da imaxe `path`: garda o da anterior e recupera o desta (ou empeza baleiro). */
export function activateAnno(path: string) {
  const cur = get(annoPath);
  if (cur === path) return;
  if (cur) stash.set(cur, { shapes: get(shapes), past: [...past], future: [...future] });
  const mine = stash.get(path);
  past.length = 0;
  future.length = 0;
  if (mine) {
    past.push(...mine.past);
    future.push(...mine.future);
    shapes.set(mine.shapes);
  } else shapes.set([]);
  selected.set(null);
  annoPath.set(path);
  sync();
}

// ---- historial: desfacer e refacer só sobre o debuxado polo usuario
const past: Shape[][] = [];
const future: Shape[][] = [];
export const canUndo = writable(false);
export const canRedo = writable(false);
const sync = () => {
  canUndo.set(past.length > 0);
  canRedo.set(future.length > 0);
};
let nextId = 1;
export const newId = () => nextId++;

/** Cambia a lista de figuras gardando o estado anterior para poder desfacer. */
export function commit(next: Shape[]) {
  past.push(get(shapes));
  if (past.length > 200) past.shift();
  future.length = 0;
  shapes.set(next);
  sync();
}
export function undo() {
  const prev = past.pop();
  if (!prev) return;
  future.push(get(shapes));
  shapes.set(prev);
  selected.set(null);
  sync();
}
export function redo() {
  const nxt = future.pop();
  if (!nxt) return;
  past.push(get(shapes));
  shapes.set(nxt);
  selected.set(null);
  sync();
}
/** Borra só o debuxado polo usuario (a imaxe non se toca). Pódese desfacer. */
export function clearAll() {
  if (!get(shapes).length) return;
  commit([]);
  selected.set(null);
}
/** Empeza de cero ao cambiar de imaxe. */
export function resetAnnotations() {
  past.length = 0;
  future.length = 0;
  shapes.set([]);
  selected.set(null);
  sync();
}

// ---- xeometría
const lineH = (s: Shape) => (s.size ?? 32) * 1.2;
export const fontOf = (s: Shape) => `${s.italic ? "italic " : ""}${s.bold ? "bold " : ""}${s.size ?? 32}px ${cssFamily(s.family ?? "Helvetica")}`;

let measureCtx: CanvasRenderingContext2D | null = null;
function textSize(s: Shape): { w: number; h: number } {
  measureCtx ??= document.createElement("canvas").getContext("2d")!;
  measureCtx.font = fontOf(s);
  const lines = (s.text ?? "").split("\n");
  const w = Math.max(...lines.map((l) => measureCtx!.measureText(l || " ").width), 4);
  return { w, h: lines.length * lineH(s) };
}

export interface Box {
  l: number;
  t: number;
  r: number;
  b: number;
}
export function boundsOf(s: Shape): Box {
  const pad = s.width / 2;
  if (s.type === "pen" || s.type === "highlight") {
    const p = s.pts ?? [];
    const xs = p.map((q) => q[0]);
    const ys = p.map((q) => q[1]);
    const w = s.type === "highlight" ? s.width * 1.5 : pad;
    return { l: Math.min(...xs) - w, t: Math.min(...ys) - w, r: Math.max(...xs) + w, b: Math.max(...ys) + w };
  }
  if (s.type === "text") {
    const { w, h } = textSize(s);
    return { l: s.x ?? 0, t: s.y ?? 0, r: (s.x ?? 0) + w, b: (s.y ?? 0) + h };
  }
  const { x1 = 0, y1 = 0, x2 = 0, y2 = 0 } = s;
  const extra = s.type === "arrow" ? Math.max(12, s.width * 4) : pad;
  return { l: Math.min(x1, x2) - extra, t: Math.min(y1, y2) - extra, r: Math.max(x1, x2) + extra, b: Math.max(y1, y2) + extra };
}

function distSeg(px: number, py: number, ax: number, ay: number, bx: number, by: number) {
  const dx = bx - ax;
  const dy = by - ay;
  const len2 = dx * dx + dy * dy;
  const t = len2 ? Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / len2)) : 0;
  return Math.hypot(px - (ax + t * dx), py - (ay + t * dy));
}

export function centerOf(s: Shape): [number, number] {
  const b = boundsOf(s);
  return [(b.l + b.r) / 2, (b.t + b.b) / 2];
}

/** Se o punto (en píxeles da imaxe) toca a figura; `tol` é a tolerancia (máis ampla ao usar a goma). */
export function hit(s: Shape, x: number, y: number, tol: number): boolean {
  if (s.angle) {
    // Desfaise o xiro no punto e compróbase coa figura sen xirar.
    const [cx, cy] = centerOf(s);
    const a = (-s.angle * Math.PI) / 180;
    const dx = x - cx;
    const dy = y - cy;
    return hitRaw(s, cx + dx * Math.cos(a) - dy * Math.sin(a), cy + dx * Math.sin(a) + dy * Math.cos(a), tol);
  }
  return hitRaw(s, x, y, tol);
}

function hitRaw(s: Shape, x: number, y: number, tol: number): boolean {
  const reach = tol + s.width / 2;
  if (s.type === "pen" || s.type === "highlight") {
    const p = s.pts ?? [];
    if (p.length === 1) return Math.hypot(x - p[0][0], y - p[0][1]) <= reach;
    for (let i = 1; i < p.length; i++) if (distSeg(x, y, p[i - 1][0], p[i - 1][1], p[i][0], p[i][1]) <= reach) return true;
    return false;
  }
  if (s.type === "line" || s.type === "arrow") return distSeg(x, y, s.x1 ?? 0, s.y1 ?? 0, s.x2 ?? 0, s.y2 ?? 0) <= reach;
  if (s.type === "text") {
    const b = boundsOf(s);
    return x >= b.l - tol && x <= b.r + tol && y >= b.t - tol && y <= b.b + tol;
  }
  const l = Math.min(s.x1 ?? 0, s.x2 ?? 0);
  const r = Math.max(s.x1 ?? 0, s.x2 ?? 0);
  const t = Math.min(s.y1 ?? 0, s.y2 ?? 0);
  const b = Math.max(s.y1 ?? 0, s.y2 ?? 0);
  if (s.type === "rect") {
    const inside = x >= l - reach && x <= r + reach && y >= t - reach && y <= b + reach;
    if (!inside) return false;
    if (s.fill) return true;
    return Math.min(Math.abs(x - l), Math.abs(x - r), Math.abs(y - t), Math.abs(y - b)) <= reach;
  }
  // elipse
  const cx = (l + r) / 2;
  const cy = (t + b) / 2;
  const rx = Math.max(1, (r - l) / 2);
  const ry = Math.max(1, (b - t) / 2);
  const k = Math.hypot((x - cx) / rx, (y - cy) / ry);
  if (s.fill) return k <= 1 + reach / Math.min(rx, ry);
  return Math.abs(k - 1) <= reach / Math.min(rx, ry);
}

/** Escala a figura por `k` tomando como ancoraxe a súa esquina de arriba á esquerda (o texto cambia de tamaño). */
export function scaled(s: Shape, k: number): Shape {
  const b = boundsOf(s);
  const ox = b.l;
  const oy = b.t;
  const f = (v: number, o: number) => o + (v - o) * k;
  const c: Shape = { ...s };
  if (s.type === "text") {
    c.size = Math.max(4, (s.size ?? 32) * k);
    return c;
  }
  c.width = Math.max(1, s.width * k);
  if (s.pts) c.pts = s.pts.map(([x, y]) => [f(x, ox), f(y, oy)]);
  for (const key of ["x1", "x2"] as const) if (s[key] !== undefined) c[key] = f(s[key] as number, ox);
  for (const key of ["y1", "y2"] as const) if (s[key] !== undefined) c[key] = f(s[key] as number, oy);
  return c;
}

export function moved(s: Shape, dx: number, dy: number): Shape {
  const c: Shape = { ...s };
  if (s.pts) c.pts = s.pts.map(([x, y]) => [x + dx, y + dy]);
  for (const k of ["x1", "x2", "x"] as const) if (s[k] !== undefined) c[k] = (s[k] as number) + dx;
  for (const k of ["y1", "y2", "y"] as const) if (s[k] !== undefined) c[k] = (s[k] as number) + dy;
  return c;
}

/** Cabeza da frecha: dous trazos a ±25° do extremo. */
export function arrowHead(s: Shape): [number, number][] {
  const { x1 = 0, y1 = 0, x2 = 0, y2 = 0 } = s;
  const a = Math.atan2(y2 - y1, x2 - x1);
  const len = Math.max(12, s.width * 4);
  const f = (d: number): [number, number] => [x2 - len * Math.cos(a + d), y2 - len * Math.sin(a + d)];
  return [f(Math.PI / 7), f(-Math.PI / 7)];
}

// ---- debuxo en canvas (o mesmo aspecto que o SVG do visor)
export function drawShape(ctx: CanvasRenderingContext2D, s: Shape) {
  ctx.save();
  if (s.angle) {
    const [cx, cy] = centerOf(s);
    ctx.translate(cx, cy);
    ctx.rotate((s.angle * Math.PI) / 180);
    ctx.translate(-cx, -cy);
  }
  ctx.strokeStyle = s.color;
  ctx.fillStyle = s.color;
  ctx.lineWidth = s.width;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  if (s.type === "pen" || s.type === "highlight") {
    const p = s.pts ?? [];
    if (s.type === "highlight") {
      ctx.globalAlpha = 0.38;
      ctx.lineWidth = s.width * 3;
      ctx.lineCap = "butt";
    }
    ctx.beginPath();
    if (p.length === 1) {
      ctx.arc(p[0][0], p[0][1], ctx.lineWidth / 2, 0, Math.PI * 2);
      ctx.fill();
    } else {
      p.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
      ctx.stroke();
    }
  } else if (s.type === "line" || s.type === "arrow") {
    ctx.beginPath();
    ctx.moveTo(s.x1 ?? 0, s.y1 ?? 0);
    ctx.lineTo(s.x2 ?? 0, s.y2 ?? 0);
    ctx.stroke();
    if (s.type === "arrow") {
      const [a, b] = arrowHead(s);
      ctx.beginPath();
      ctx.moveTo(a[0], a[1]);
      ctx.lineTo(s.x2 ?? 0, s.y2 ?? 0);
      ctx.lineTo(b[0], b[1]);
      ctx.stroke();
    }
  } else if (s.type === "rect") {
    const l = Math.min(s.x1 ?? 0, s.x2 ?? 0);
    const t = Math.min(s.y1 ?? 0, s.y2 ?? 0);
    const w = Math.abs((s.x2 ?? 0) - (s.x1 ?? 0));
    const h = Math.abs((s.y2 ?? 0) - (s.y1 ?? 0));
    if (s.fill) ctx.fillRect(l, t, w, h);
    else ctx.strokeRect(l, t, w, h);
  } else if (s.type === "ellipse") {
    const cx = ((s.x1 ?? 0) + (s.x2 ?? 0)) / 2;
    const cy = ((s.y1 ?? 0) + (s.y2 ?? 0)) / 2;
    ctx.beginPath();
    ctx.ellipse(cx, cy, Math.max(1, Math.abs((s.x2 ?? 0) - (s.x1 ?? 0)) / 2), Math.max(1, Math.abs((s.y2 ?? 0) - (s.y1 ?? 0)) / 2), 0, 0, Math.PI * 2);
    if (s.fill) ctx.fill();
    else ctx.stroke();
  } else if (s.type === "text") {
    ctx.font = fontOf(s);
    ctx.textBaseline = "top";
    (s.text ?? "").split("\n").forEach((line, i) => ctx.fillText(line, s.x ?? 0, (s.y ?? 0) + i * lineH(s)));
  }
  ctx.restore();
}

/** A imaxe co debuxado queimado enriba, no seu tamaño orixinal. */
export async function renderAnnotated(src: string, list: Shape[], mime: string, quality: number): Promise<Blob> {
  // Descárgase como datos (non como <img> de outra orixe) para que o canvas poida exportarse.
  const img = await createImageBitmap(await (await fetch(src)).blob());
  const c = document.createElement("canvas");
  c.width = img.width;
  c.height = img.height;
  const ctx = c.getContext("2d")!;
  if (mime === "image/jpeg") {
    ctx.fillStyle = "#fff";
    ctx.fillRect(0, 0, c.width, c.height);
  }
  ctx.drawImage(img, 0, 0);
  for (const s of list) drawShape(ctx, s);
  return await new Promise((res, rej) => c.toBlob((b) => (b ? res(b) : rej(new Error("toBlob"))), mime, quality));
}
export { lineH };
