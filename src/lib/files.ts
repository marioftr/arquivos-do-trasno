// Estado do espazo de traballo: os ficheiros cargados, cal se está vendo e
// cales están marcados para operacións en lote (unir, comprimir varios...).

import type { FontSpec } from "./fonts";
import { writable, get } from "svelte/store";
import { convertFileSrc } from "@tauri-apps/api/core";
import { call } from "./errors";
import * as engine from "./engine";
import type { Category } from "./engine";
import { extensionOf, fileNameOf } from "./formats";
import { tr } from "./i18n";
import { toast } from "./toasts";

export interface FileEntry {
  path: string;
  name: string;
  ext: string;
  category: Category;
  size?: number;
  pages?: number;
  width?: number;
  height?: number;
  duration?: number;
}

export const files = writable<FileEntry[]>([]);
export const activePath = writable<string | null>(null);
export const checked = writable<Set<string>>(new Set());

/** Edición de imaxe en curso, para previsualizala en directo no visor. */
export interface ImagePreview {
  rotate: number;
  flipH: boolean;
  flipV: boolean;
  grayscale: boolean;
  invert: boolean;
  brightness: number;
  contrast: number;
  blur: number;
}
export const imageEdit = writable<ImagePreview | null>(null);

function patch(path: string, change: Partial<FileEntry>) {
  files.update((list) => list.map((f) => (f.path === path ? { ...f, ...change } : f)));
}

// Lecturas pesadas (ffprobe) de catro en catro: con cen ficheiros non se lanzan cen procesos á vez.
const queue: (() => Promise<void>)[] = [];
let running = 0;
function later(job: () => Promise<void>) {
  queue.push(job);
  pump();
}
function pump() {
  while (running < 4 && queue.length) {
    const job = queue.shift()!;
    running++;
    job().catch(() => {}).finally(() => {
      running--;
      pump();
    });
  }
}

export async function addPaths(input: string[]) {
  let selectedNew = false;
  // Os cartafoles abren as súas ficheiros; as categorías míranse todas á vez e despois entran por orde.
  const paths = [...new Set(await engine.expandPaths(input).catch(() => input))];
  let skipped = 0;
  const kinds = await Promise.all(paths.map((p) => engine.detectCategory(p).then((c) => ({ c }), () => ({ c: null as Category | null }))));
  for (const [idx, path] of paths.entries()) {
    if (get(files).some((f) => f.path === path)) {
      // Xa estaba: avísase e queda á vista.
      toast(tr("«{name}» xa estaba na lista; o trasno non o garda dúas veces.", { name: fileNameOf(path) }));
      if (!selectedNew) activePath.set(path);
      selectedNew = true;
      continue;
    }
    const category = kinds[idx].c;
    if (!category) {
      toast(tr("O trasno non puido ler «{name}».", { name: fileNameOf(path) }), { kind: "warn" });
      continue;
    }
    if (category === "descoñecido") {
      if (paths.length > 3) {
        skipped++;
        continue;
      }
      toast(tr("O trasno non sabe que facer cun ficheiro «.{ext}» («{name}»).", { ext: extensionOf(path) || "?", name: fileNameOf(path) }), { kind: "warn" });
      continue;
    }
    const entry: FileEntry = { path, name: fileNameOf(path), ext: extensionOf(path), category };
    files.update((list) => [...list, entry]);
    // O primeiro ficheiro recén importado queda seleccionado e á vista.
    if (!selectedNew) {
      activePath.set(path);
      selectedNew = true;
    }

    engine.fileSize(path).then((size) => patch(path, { size })).catch(() => {});
    if (category === "documento" && entry.ext === "pdf") {
      engine.pdfPageCount(path).then((pages) => patch(path, { pages })).catch(() => {});
    }
    if (category === "audio" || category === "video") {
      later(() =>
        engine
        .probeMedia(path)
        .then((info) => {
          const v = info.streams?.find((s) => s.codec_type === "video");
          const duration = Number(info.format?.duration);
          patch(path, {
            duration: isFinite(duration) ? duration : undefined,
            width: v?.width,
            height: v?.height,
          });
        })
        .catch(() => {}));
    }
    if (category === "imaxe") {
      engine.imageDimensions(path).then(([width, height]) => patch(path, { width, height })).catch(() => {});
    }
  }
  if (skipped) toast(tr("O trasno deixou {n} ficheiros sen abrir porque non sabe que facer con eles.", { n: skipped }), { kind: "warn" });
}

// O último ficheiro quitado, para poder desfacelo.
let lastRemoved: { entry: FileEntry; index: number; wasChecked: boolean } | null = null;

export function undoRemove() {
  const r = lastRemoved;
  if (!r || get(files).some((f) => f.path === r.entry.path)) return;
  lastRemoved = null;
  files.update((l) => {
    const next = [...l];
    next.splice(Math.min(r.index, next.length), 0, r.entry);
    return next;
  });
  if (r.wasChecked) checked.update((s) => new Set(s).add(r.entry.path));
  activePath.set(r.entry.path);
}

export function removeFile(path: string) {
  const index = get(files).findIndex((f) => f.path === path);
  if (index >= 0) {
    lastRemoved = { entry: get(files)[index], index, wasChecked: get(checked).has(path) };
    toast(tr("O trasno quitou «{name}» da lista.", { name: lastRemoved.entry.name }), { action: { label: tr("Desfacer"), run: undoRemove } });
  }
  const list = get(files).filter((f) => f.path !== path);
  files.set(list);
  checked.update((s) => {
    const next = new Set(s);
    next.delete(path);
    return next;
  });
  if (get(activePath) === path) activePath.set(list[0]?.path ?? null);
}

export function clearAll() {
  files.set([]);
  checked.set(new Set());
  activePath.set(null);
}

export function toggleChecked(path: string) {
  checked.update((s) => {
    const next = new Set(s);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    return next;
  });
}

export function setChecked(paths: string[]) {
  checked.set(new Set(paths));
}

/** Move o ficheiro da posición `from` á posición `to` da lista. */
export function moveFileTo(from: number, to: number) {
  files.update((list) => {
    to = Math.max(0, Math.min(list.length - 1, to));
    if (from === to || from < 0 || from >= list.length) return list;
    const next = [...list];
    const [x] = next.splice(from, 1);
    next.splice(to, 0, x);
    return next;
  });
}

/** Marca todos os ficheiros entre dous índices (Maiús + clic). */
export function checkRange(a: number, b: number) {
  const list = get(files);
  const [lo, hi] = a < b ? [a, b] : [b, a];
  checked.update((s) => {
    const next = new Set(s);
    for (let i = lo; i <= hi; i++) if (list[i]) next.add(list[i].path);
    return next;
  });
}

export function moveFile(path: string, delta: -1 | 1) {
  files.update((list) => {
    const i = list.findIndex((f) => f.path === path);
    const j = i + delta;
    if (i < 0 || j < 0 || j >= list.length) return list;
    const next = [...list];
    [next[i], next[j]] = [next[j], next[i]];
    return next;
  });
}

/** URL que o visor pode cargar. Só funciona despois de `allow_view`. */
export async function viewUrl(path: string): Promise<string> {
  await call("allow_view", { path });
  return convertFileSrc(path);
}

// ----------------------------------------------------- vistas previas dos cambios

/** Cambios de PDF que o visor debuxa por riba das páxinas antes de aplicalos. */
export interface PdfOverlay {
  /** Xiro (graos) da páxina `page`, ou de todas se é null. */
  rotate?: { degrees: number; page: number | null };
  /** Páxinas escollidas (as demais atenúanse). */
  select?: number[];
  /** Márxenes a quitar, en puntos do PDF. */
  crop?: { l: number; b: number; r: number; t: number };
  /** Texto de marca de auga (centrado, co seu xiro). */
  text?: { text: string; xPct: number; yPct: number; size: number; color: string; opacity: number; angle: number; centered: boolean; pages: number[] | null; font?: FontSpec };
  numbers?: { position: string; format: string; start: number; size: number; margin: number; color: string; pages?: number[] | null; font?: FontSpec };
  image?: { url: string; xPct: number; yPct: number; widthPct: number; opacity: number; pages: number[] | null };
  /** Textos colocados sobre as páxinas: seleccionables, arrastrables e redimensionables directamente no visor. */
  items?: {
    list: PdfTextBox[];
    selected: number | null;
    editing: number | null;
    onSelect: (id: number | null) => void;
    onPlace: (page: number, xPct: number, yPct: number) => void;
    onMove: (id: number, xPct: number, yPct: number) => void;
    onResize: (id: number, size: number) => void;
    onRotate: (id: number, angle: number) => void;
    onEdit: (id: number, text: string, style?: { bold: boolean; italic: boolean }) => void;
    onEditing: (id: number | null) => void;
  };
}
/** Un texto sobre unha páxina. A posición é a esquina de arriba á esquerda, en % da páxina desde abaixo á esquerda. */
export interface PdfTextBox {
  id: number;
  page: number;
  xPct: number;
  yPct: number;
  text: string;
  size: number;
  color: string;
  family: string;
  bold: boolean;
  italic: boolean;
  /** Graos, en sentido contrario ao reloxo. */
  angle: number;
}
export const pdfOverlay = writable<PdfOverlay | null>(null);
/** Textos engadidos a cada PDF, lembrados ao cambiar de ficheiro (mentres a app siga aberta). */
export const pdfTextStash = new Map<string, PdfTextBox[]>();
let pdfTextId = 0;
export const newPdfTextId = () => ++pdfTextId;


/** Recorte de imaxe sobre o visor, en píxeles da imaxe orixinal. */
export interface ImageCrop {
  x: number;
  y: number;
  w: number;
  h: number;
}
export const imageCrop = writable<ImageCrop | null>(null);
/** Mentres está activo, arrastrar sobre a imaxe debuxa un recorte novo (en vez de mover a vista). */
export const imageCropEnabled = writable(false);
/** URL que está a amosar o visor de imaxe (a orixinal ou unha vista previa PNG), para poder debuxar sobre ela. */
export const imageUrl = writable("");
/** Relación de aspecto elixida para un vídeo («16/9»), que o visor amosa antes de converter; baleiro = a orixinal. */
export const videoDar = writable("");

/** Vista previa de audio e vídeo: velocidade, volume e salto a un instante. */
export interface MediaPreview {
  speed: number;
  volume: number;
}
export const mediaPreview = writable<MediaPreview | null>(null);
export const mediaSeek = writable<{ t: number; stamp: number } | null>(null);
/** Instante (s) en que está o reprodutor do visor, para usalo nas ferramentas. */
export const mediaTime = writable<number>(0);
/** Intervalo que queda ao cortar (s); o visor móstrao sobre a liña de tempo e permite arrastralo. */
export const trimRange = writable<{ start: number; end: number } | null>(null);

/** Proporción fixa do recorte (ancho / alto) ou null para libre. */
export const imageCropAspect = writable<number | null>(null);

// ----------------------------------------------------- lembrar a sesión

const SESSION_KEY = "arquivos-do-trasno.session";
let restored = false;

/** Garda a lista, o ficheiro activo e os marcados cada vez que cambian (só despois de restaurar, para non pisar o gardado). */
function startSaving() {
  const save = () => {
    if (!restored) return;
    try {
      localStorage.setItem(
        SESSION_KEY,
        JSON.stringify({ paths: get(files).map((f) => f.path), active: get(activePath), checked: [...get(checked)] }),
      );
    } catch {
      // sen almacenamento: simplemente non se lembra
    }
  };
  files.subscribe(save);
  activePath.subscribe(save);
  checked.subscribe(save);
}
startSaving();

/** Volve poñer os ficheiros da última vez que seguen existindo. Chámase unha vez ao abrir a aplicación. */
export async function restoreSession() {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (raw) {
      const s = JSON.parse(raw) as { paths?: string[]; active?: string | null; checked?: string[] };
      const alive: string[] = [];
      for (const p of s.paths ?? []) {
        if (await engine.fileSize(p).then(() => true).catch(() => false)) alive.push(p);
      }
      const missing = (s.paths?.length ?? 0) - alive.length;
      await addPaths(alive);
      if (s.active && alive.includes(s.active)) activePath.set(s.active);
      checked.set(new Set((s.checked ?? []).filter((p) => alive.includes(p))));
      if (missing > 0) toast(tr("{n} ficheiros da última vez xa non están no seu sitio: o trasno non os atopa.", { n: missing }), { kind: "warn" });
    }
  } catch {
    // sesión rota: empézase en branco
  }
  restored = true;
  files.update((l) => l); // fixa o estado actual
}
