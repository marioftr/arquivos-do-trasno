// Texto dun PDF con posición (pdf.js) e o que se saca del: Markdown, táboas, títulos para o índice e buscas para tapar.
// Todo en coordenadas do que se ve (orixe arriba á esquerda, en puntos), así as páxinas xiradas non dan problemas.

import * as pdfjs from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { viewUrl } from "./files";

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

export async function openPdf(path: string): Promise<pdfjs.PDFDocumentProxy> {
  const data = new Uint8Array(await (await fetch(await viewUrl(path))).arrayBuffer());
  return pdfjs.getDocument({ data }).promise;
}

export interface Item {
  str: string;
  x: number;
  /** Liña base, desde arriba. */
  y: number;
  w: number;
  size: number;
}
export interface Line {
  y: number;
  x0: number;
  x1: number;
  size: number;
  items: Item[];
  text: string;
  /** Posición X de cada carácter de `text` (cunha entrada máis: o fin). */
  xs: number[];
}
export interface PageText {
  page: number;
  width: number;
  height: number;
  lines: Line[];
}

/** Agrupa os fragmentos de texto en liñas (de arriba abaixo, de esquerda a dereita). */
export function buildLines(items: Item[]): Line[] {
  const sorted = [...items].filter((i) => i.str.trim() !== "").sort((a, b) => a.y - b.y || a.x - b.x);
  const rows: Item[][] = [];
  for (const it of sorted) {
    const row = rows.find((r) => Math.abs(r[0].y - it.y) <= Math.max(1.5, Math.min(r[0].size, it.size) * 0.4));
    if (row) row.push(it);
    else rows.push([it]);
  }
  const lines: Line[] = rows.map((r) => {
    r.sort((a, b) => a.x - b.x);
    let text = "";
    const xs: number[] = [];
    let prevEnd = 0;
    r.forEach((it, k) => {
      if (k > 0) {
        const gap = it.x - prevEnd;
        if (gap > it.size * 0.18 && !text.endsWith(" ") && !it.str.startsWith(" ")) {
          xs.push(prevEnd);
          text += " ";
        }
      }
      const per = it.str.length ? it.w / it.str.length : 0;
      for (let i = 0; i < it.str.length; i++) xs.push(it.x + per * i);
      text += it.str;
      prevEnd = it.x + it.w;
    });
    xs.push(prevEnd);
    const size = r.reduce((a, b) => a + b.size * b.str.length, 0) / Math.max(1, r.reduce((a, b) => a + b.str.length, 0));
    return { y: r.reduce((a, b) => a + b.y, 0) / r.length, x0: r[0].x, x1: prevEnd, size, items: r, text, xs };
  });
  return lines.sort((a, b) => a.y - b.y);
}

export async function readPages(doc: pdfjs.PDFDocumentProxy, pages: number[], onStep?: (done: number) => void): Promise<PageText[]> {
  const out: PageText[] = [];
  let done = 0;
  for (const n of pages) {
    const page = await doc.getPage(n);
    const vp = page.getViewport({ scale: 1 });
    const content = await page.getTextContent();
    const items: Item[] = [];
    for (const it of content.items) {
      if (!("str" in it) || !it.str) continue;
      const m = pdfjs.Util.transform(vp.transform, it.transform);
      // Só texto horizontal tal como se ve.
      if (Math.abs(m[1]) > Math.abs(m[0]) * 0.2) continue;
      const size = Math.hypot(m[2], m[3]);
      if (!(size > 0)) continue;
      items.push({ str: it.str, x: m[4], y: m[5], w: it.width * vp.scale, size });
    }
    out.push({ page: n, width: vp.width, height: vp.height, lines: buildLines(items) });
    page.cleanup();
    onStep?.(++done);
  }
  return out;
}

// ------------------------------------------------------------------ táboas

export interface Table {
  page: number;
  rows: string[][];
  /** Caixa na páxina (puntos). */
  box: { x0: number; y0: number; x1: number; y1: number };
}

interface Cell {
  x0: number;
  x1: number;
  text: string;
}

/** Parte unha liña en celas onde hai ocos grandes entre fragmentos. */
function cellsOf(line: Line): Cell[] {
  const cells: Cell[] = [];
  const gap = Math.max(line.size * 0.9, 6);
  for (const it of line.items) {
    const last = cells[cells.length - 1];
    if (last && it.x - last.x1 < gap) {
      last.text += (it.x - last.x1 > it.size * 0.18 && !last.text.endsWith(" ") ? " " : "") + it.str;
      last.x1 = it.x + it.w;
    } else {
      cells.push({ x0: it.x, x1: it.x + it.w, text: it.str });
    }
  }
  return cells.map((c) => ({ ...c, text: c.text.trim() })).filter((c) => c.text);
}

/** Atopa táboas: liñas seguidas con varias celas, separadas en columnas polos ocos verticais que comparten. */
export function findTables(p: PageText): { tables: Table[]; used: Set<Line> } {
  const used = new Set<Line>();
  const tables: Table[] = [];
  const info = p.lines.map((l) => ({ l, cells: cellsOf(l) }));
  let i = 0;
  while (i < info.length) {
    if (info[i].cells.length < 2) {
      i++;
      continue;
    }
    let j = i + 1;
    while (j < info.length && info[j].cells.length >= 2 && info[j].l.y - info[j - 1].l.y <= Math.max(info[j].l.size, info[j - 1].l.size) * 2.4 && Math.abs(info[j].l.size - info[i].l.size) <= info[i].l.size * 0.25) j++;
    const run = info.slice(i, j);
    i = j;
    if (run.length < 3) continue;
    // Gutters: zonas en X que case ningunha fila cruza.
    const minX = Math.min(...run.map((r) => r.cells[0].x0));
    const maxX = Math.max(...run.map((r) => r.cells[r.cells.length - 1].x1));
    const width = Math.ceil(maxX - minX) + 1;
    const cover = new Array<number>(width).fill(0);
    for (const r of run) for (const c of r.cells) for (let x = Math.floor(c.x0 - minX); x <= Math.min(width - 1, Math.ceil(c.x1 - minX)); x++) cover[x]++;
    const limit = run.length * 0.15;
    const size = run[0].l.size;
    const cuts: number[] = [];
    let start = -1;
    for (let x = 0; x <= width; x++) {
      const empty = x < width && cover[x] <= limit;
      if (empty && start < 0) start = x;
      if (!empty && start >= 0) {
        if (x - start >= size * 0.6 && start > 0 && x < width) cuts.push(minX + (start + x) / 2);
        start = -1;
      }
    }
    const cols = cuts.length + 1;
    if (cols < 2) continue;
    const colOf = (c: Cell) => {
      const mid = (c.x0 + c.x1) / 2;
      let k = 0;
      while (k < cuts.length && mid > cuts[k]) k++;
      return k;
    };
    const rows = run.map((r) => {
      const row = new Array<string>(cols).fill("");
      for (const c of r.cells) {
        const k = colOf(c);
        row[k] = row[k] ? `${row[k]} ${c.text}` : c.text;
      }
      return row;
    });
    // Fiable só se a maioría das filas usa máis dunha columna.
    if (rows.filter((r) => r.filter(Boolean).length >= 2).length < rows.length * 0.6) continue;
    run.forEach((r) => used.add(r.l));
    tables.push({
      page: p.page,
      rows,
      box: { x0: minX, y0: run[0].l.y - run[0].l.size, x1: maxX, y1: run[run.length - 1].l.y + run[run.length - 1].l.size * 0.3 },
    });
  }
  return { tables, used };
}

export function csvOf(rows: string[][], sep: string): string {
  const q = (s: string) => (s.includes(sep) || s.includes('"') || /[\r\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s);
  return rows.map((r) => r.map(q).join(sep)).join("\r\n") + "\r\n";
}

// ---------------------------------------------------------------- Markdown

const BULLET = /^[•·▪◦●○‣⁃\-–—*]\s+/;
const NUMBERED = /^(\d{1,3})[.)]\s+/;

/** Tamaño de letra do corpo do texto: o máis usado (por número de letras). */
function bodySize(pages: PageText[]): number {
  const h = new Map<number, number>();
  for (const p of pages) for (const l of p.lines) h.set(Math.round(l.size * 2) / 2, (h.get(Math.round(l.size * 2) / 2) ?? 0) + l.text.length);
  let best = 11;
  let max = 0;
  for (const [k, v] of h) if (v > max) ((best = k), (max = v));
  return best;
}

/** Títulos: liñas curtas con letra claramente maior ca do corpo. O nivel sae do tamaño (o maior, 1). */
export function headingLevels(pages: PageText[]): { body: number; level: (l: Line) => number } {
  const body = bodySize(pages);
  const sizes = new Set<number>();
  for (const p of pages) for (const l of p.lines) if (l.size >= body * 1.12 && l.text.length <= 120 && !BULLET.test(l.text)) sizes.add(Math.round(l.size * 2) / 2);
  const ranked = [...sizes].sort((a, b) => b - a);
  return {
    body,
    level: (l) => {
      if (l.size < body * 1.12 || l.text.length > 120 || BULLET.test(l.text)) return 0;
      const k = ranked.indexOf(Math.round(l.size * 2) / 2);
      return k < 0 ? 0 : Math.min(6, k + 1);
    },
  };
}

export interface MdOptions {
  tables: boolean;
  pageMarks: boolean;
}

const ITEM = /^(- |\d{1,3}[.)] )/;

export function toMarkdown(pages: PageText[], o: MdOptions): string {
  const { level } = headingLevels(pages);
  const out: string[] = [];
  for (const p of pages) {
    if (o.pageMarks) out.push(`<!-- páxina ${p.page} -->`);
    const { tables, used } = o.tables ? findTables(p) : { tables: [] as Table[], used: new Set<Line>() };
    const blocks: { y: number; md: string }[] = [];
    for (const t of tables) {
      const esc = (s: string) => s.replace(/\|/g, "\\|");
      const head = t.rows[0].map(esc);
      const md = [`| ${head.join(" | ")} |`, `| ${head.map(() => "---").join(" | ")} |`, ...t.rows.slice(1).map((r) => `| ${r.map(esc).join(" | ")} |`)].join("\n");
      blocks.push({ y: t.box.y0, md });
    }
    let para = "";
    let paraY = 0;
    let prev: Line | null = null;
    const flush = () => {
      if (para) blocks.push({ y: paraY, md: para });
      para = "";
    };
    for (const l of p.lines) {
      if (used.has(l)) {
        flush();
        prev = null;
        continue;
      }
      const text = l.text.trim();
      const lv = level(l);
      if (lv) {
        flush();
        blocks.push({ y: l.y, md: `${"#".repeat(lv)} ${text}` });
        prev = null;
        continue;
      }
      const bullet = BULLET.test(text);
      const numbered = NUMBERED.test(text);
      const newPara = !prev || l.y - prev.y > prev.size * 1.7 || bullet || numbered;
      if (newPara) {
        flush();
        paraY = l.y;
        para = bullet ? `- ${text.replace(BULLET, "")}` : text;
      } else if (/[a-záéíóúñü]-$/i.test(para) && /^[a-záéíóúñü]/.test(text)) {
        para = para.slice(0, -1) + text; // palabra partida a final de liña
      } else {
        para += ` ${text}`;
      }
      prev = l;
    }
    flush();
    blocks.sort((a, b) => a.y - b.y);
    out.push(...blocks.map((b) => b.md));
  }
  // Os elementos dunha lista van seguidos; o resto, separado por unha liña en branco.
  let md = "";
  out.forEach((cur, k) => {
    md += k === 0 ? cur : (ITEM.test(out[k - 1]) && ITEM.test(cur) ? "\n" : "\n\n") + cur;
  });
  return md + "\n";
}

// ----------------------------------------------------------------- títulos

/** Propón un índice: os títulos que atopa (letra maior ca a do corpo), con nivel segundo o tamaño. */
export function detectHeadings(pages: PageText[]): { title: string; page: number; level: number }[] {
  const { level } = headingLevels(pages);
  const out: { title: string; page: number; level: number }[] = [];
  for (const p of pages) {
    const { used } = findTables(p);
    let prev: { page: number; level: number; y: number; size: number } | null = null;
    for (const l of p.lines) {
      const lv = level(l);
      if (!lv || used.has(l) || l.text.trim().length < 2 || /^\d+$/.test(l.text.trim())) {
        prev = null;
        continue;
      }
      // Un título de varias liñas xúntase nun só.
      const last = out[out.length - 1];
      if (prev && last && prev.page === p.page && prev.level === lv && l.y - prev.y <= prev.size * 1.5) last.title += ` ${l.text.trim()}`;
      else out.push({ title: l.text.trim(), page: p.page, level: lv - 1 });
      prev = { page: p.page, level: lv, y: l.y, size: l.size };
    }
  }
  return out.slice(0, 300);
}

/** O índice que xa leva o PDF (os marcadores), aplanado con niveis. */
export async function readOutline(doc: pdfjs.PDFDocumentProxy): Promise<{ title: string; page: number; level: number }[]> {
  const out: { title: string; page: number; level: number }[] = [];
  const walk = async (items: any[], level: number) => {
    for (const it of items) {
      let page = 1;
      try {
        let dest = it.dest;
        if (typeof dest === "string") dest = await doc.getDestination(dest);
        if (Array.isArray(dest)) {
          const ref = dest[0];
          page = typeof ref === "object" && ref !== null ? (await doc.getPageIndex(ref)) + 1 : Number(ref) + 1;
        }
      } catch {
        // un destino roto: queda na páxina 1
      }
      out.push({ title: String(it.title ?? "").trim() || "—", page: Math.max(1, page), level });
      if (it.items?.length) await walk(it.items, level + 1);
    }
  };
  const ol = await doc.getOutline();
  if (ol) await walk(ol, 0);
  return out;
}

// ------------------------------------------------------------------ buscas

export type FindKind = "text" | "email" | "phone" | "iban" | "dni";

const PATTERNS: Record<Exclude<FindKind, "text">, RegExp> = {
  email: /[\w.+-]+@[\w-]+(?:\.[\w-]+)+/g,
  phone: /(?:\+\d{1,3}[\s.-]?)?\(?\d{2,4}\)?[\s.-]?\d{2,3}[\s.-]?\d{2,3}(?:[\s.-]?\d{2,3})?/g,
  iban: /\b[A-Z]{2}\d{2}(?: ?[A-Z0-9]{4}){3,7}(?: ?[A-Z0-9]{1,4})?\b/g,
  dni: /\b(?:\d{8}|[XYZ]\d{7})[- ]?[A-Z]\b/gi,
};

export interface Found {
  page: number;
  /** Caixa en % da páxina, desde arriba á esquerda. */
  x: number;
  y: number;
  w: number;
  h: number;
  text: string;
}

export function findInPages(pages: PageText[], kind: FindKind, query: string): Found[] {
  let re: RegExp;
  if (kind === "text") {
    const q = query.trim();
    if (!q) return [];
    re = new RegExp(q.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "gi");
  } else {
    re = new RegExp(PATTERNS[kind].source, PATTERNS[kind].flags);
  }
  const out: Found[] = [];
  for (const p of pages) {
    for (const l of p.lines) {
      re.lastIndex = 0;
      for (const m of l.text.matchAll(re)) {
        if (!m[0]) continue;
        if (kind === "phone") {
          const digits = m[0].replace(/\D/g, "").length;
          if (digits < 9 || digits > 15) continue;
        }
        const a = m.index ?? 0;
        const b = a + m[0].length;
        const x0 = l.xs[a] ?? l.x0;
        const x1 = l.xs[b] ?? l.x1;
        const top = l.y - l.size * 0.95;
        const h = l.size * 1.25;
        out.push({ page: p.page, x: (Math.max(0, x0 - 1) / p.width) * 100, y: (Math.max(0, top) / p.height) * 100, w: ((x1 - x0 + 2) / p.width) * 100, h: (h / p.height) * 100, text: m[0] });
      }
    }
  }
  return out;
}
