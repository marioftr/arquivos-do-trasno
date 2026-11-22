/** Utilidades de ficheiros da versión web (sen nada de Tauri: todo corre no navegador). */

import { get } from "svelte/store";
import { lang } from "@app/lib/i18n";

export { UserError } from "@app/lib/mt";
export type { Ctx } from "@app/lib/mt";

export interface Output {
  name: string;
  blob: Blob;
  /** Detalle curto: tamaño orixinal, aviso… */
  note?: string;
  /** Texto do resultado (OCR, transcrición): amósase e pódese copiar. */
  text?: string;
}


export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB"];
  let v = n / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  const num = v >= 100 ? String(Math.round(v)) : v.toFixed(1).replace(/\.0$/, "");
  return `${get(lang) === "en" ? num : num.replace(".", ",")} ${units[i]}`;
}

export const stemOf = (name: string) => name.replace(/\.[^./\\]+$/, "") || name;
export const extOf = (name: string) => (/\.([^./\\]+)$/.exec(name)?.[1] ?? "").toLowerCase();

/** Nome que non choca cos xa usados: «foto.jpg», «foto (2).jpg»… */
export function uniqueName(name: string, taken: Set<string>): string {
  let out = name;
  let n = 2;
  while (taken.has(out.toLowerCase())) out = `${stemOf(name)} (${n++}).${extOf(name)}`;
  taken.add(out.toLowerCase());
  return out;
}

/** Cumpre un valor de `accept` de <input type=file> («image/*», «application/pdf», «.jpg»)? */
export function accepts(accept: string, file: File): boolean {
  const rules = accept.split(",").map((s) => s.trim().toLowerCase()).filter(Boolean);
  if (!rules.length) return true;
  const type = file.type.toLowerCase();
  const ext = `.${extOf(file.name)}`;
  return rules.some((r) => (r.endsWith("/*") ? type.startsWith(r.slice(0, -1)) : r.startsWith(".") ? ext === r : type === r));
}

/** Descarga un resultado (en móbil, o navegador pregunta onde gardalo). */
export function download(o: Output) {
  const url = URL.createObjectURL(o.blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = o.name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 30_000);
}

const asFiles = (list: Output[]) => list.map((o) => new File([o.blob], o.name, { type: o.blob.type }));

/** O aparello pode compartir estes ficheiros? Só se usa a folla de compartir do móbil (Fotos, Ficheiros, aplicacións…): no escritorio, descargar é o normal. */
export function canShare(list: Output[]): boolean {
  try {
    return matchMedia("(pointer: coarse)").matches && !!navigator.canShare && !!list.length && navigator.canShare({ files: asFiles(list) });
  } catch {
    return false;
  }
}

/** Abre a folla de compartir. Devolve false se a persoa a pecha sen elixir nada. */
export async function share(list: Output[]): Promise<boolean> {
  try {
    await navigator.share({ files: asFiles(list) });
    return true;
  } catch (e) {
    if ((e as DOMException)?.name === "AbortError") return false;
    throw e;
  }
}

/** Garda un resultado: no móbil pola folla de compartir (un PWA de iOS non pode descargar ficheiros), no escritorio descargándoo. */
export async function save(o: Output): Promise<void> {
  if (canShare([o])) await share([o]);
  else download(o);
}
