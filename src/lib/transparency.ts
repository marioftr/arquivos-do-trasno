// Transparencia das imaxes: ao pasar a un formato que non a admite (JPG, BMP, PNM) hai que escoller un fondo.

import { writable } from "svelte/store";
import { call } from "./errors";

/** Formatos sen transparencia: o motor compón a imaxe sobre un fondo ao gardar nelas. */
export const NO_ALPHA = ["jpg", "jpeg", "bmp", "pnm", "pbm", "pgm", "ppm"];

export type Bg = "white" | "black" | "gray";
export const BACKGROUNDS: Record<Bg, [number, number, number]> = { white: [255, 255, 255], black: [0, 0, 0], gray: [128, 128, 128] };

const cache = new Map<string, Promise<boolean>>();
/** A imaxe ten algún píxel transparente? (Lembra a resposta mentres non se reimporte.) */
export function hasTransparency(path: string): Promise<boolean> {
  let p = cache.get(path);
  if (!p) {
    p = call<boolean>("image_has_transparency", { path }).catch(() => false);
    cache.set(path, p);
  }
  return p;
}
/** Cor CSS do fondo que escolleu a persoa (mentres hai que escoller un): o visor amósao detrás da imaxe, como quedará ao exportar. */
export const backdrop = writable<string | null>(null);
export const cssOf = (bg: Bg) => `rgb(${BACKGROUNDS[bg].join(",")})`;

export const forgetTransparency = (path: string) => void cache.delete(path);

/** Algunha destas imaxes ten transparencia? */
export async function anyTransparent(paths: string[]): Promise<boolean> {
  return (await Promise.all(paths.map(hasTransparency))).some(Boolean);
}
