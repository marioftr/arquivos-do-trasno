// Estado do espazo de traballo: os ficheiros cargados, cal se está vendo e
// cales están marcados para operacións en lote (unir, comprimir varios...).

import { writable, get } from "svelte/store";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
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
  for (const path of paths) {
    if (get(files).some((f) => f.path === path)) continue;
    const category = await engine.detectCategory(path);
    const entry: FileEntry = { path, name: fileNameOf(path), ext: extensionOf(path), category };
    files.update((list) => [...list, entry]);
    if (get(activePath) === null) activePath.set(path);

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
  await invoke("allow_view", { path });
  return convertFileSrc(path);
}
