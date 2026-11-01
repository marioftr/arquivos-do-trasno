// Estado do espazo de traballo: os ficheiros cargados, cal se está vendo e
// cales están marcados para operacións en lote (unir, comprimir varios...).

import { writable, get } from "svelte/store";
import { convertFileSrc } from "@tauri-apps/api/core";
import { call } from "./errors";
import * as engine from "./engine";
import type { Category } from "./engine";
import { extensionOf, fileNameOf } from "./formats";

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

export async function addPaths(paths: string[]) {
  let selectedNew = false;
  for (const path of paths) {
    if (get(files).some((f) => f.path === path)) continue;
    const category = await engine.detectCategory(path);
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
        .catch(() => {});
    }
    if (category === "imaxe") {
      engine.imageDimensions(path).then(([width, height]) => patch(path, { width, height })).catch(() => {});
    }
  }
}

export function removeFile(path: string) {
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
  /** Texto de marca de auga ou texto engadido (posición en % da páxina, desde abaixo á esquerda). */
  text?: { text: string; xPct: number; yPct: number; size: number; color: string; opacity: number; angle: number; centered: boolean; pages: number[] | null };
  numbers?: { position: string; format: string; start: number; size: number; margin: number; color: string };
  image?: { url: string; xPct: number; yPct: number; widthPct: number; opacity: number; pages: number[] | null };
}
export const pdfOverlay = writable<PdfOverlay | null>(null);

/** Recorte de imaxe sobre o visor, en píxeles da imaxe orixinal. */
export interface ImageCrop {
  x: number;
  y: number;
  w: number;
  h: number;
}
export const imageCrop = writable<ImageCrop | null>(null);

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
