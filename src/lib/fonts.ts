// Letras para marcas, números e textos (PDF e imaxes). Hai tres familias estándar e as instaladas no sistema.
// Cada letra elíxese por familia e despois actívase a negriña e a cursiva (non hai unha entrada por cada variante).

import { writable, get } from "svelte/store";
import * as engine from "./engine";

export interface FontSpec {
  /** «Helvetica», «Times», «Courier» ou «sys:<familia>» para unha letra do sistema. */
  family: string;
  bold: boolean;
  italic: boolean;
}

export const STANDARD_FAMILIES: { id: string; label: string; css: string }[] = [
  { id: "Helvetica", label: "Helvetica (sen serifa)", css: 'Helvetica, Arial, sans-serif' },
  { id: "Times", label: "Times (con serifa)", css: '"Times New Roman", Times, serif' },
  { id: "Courier", label: "Courier (monoespazada)", css: '"Courier New", Courier, monospace' },
];

export const sysFonts = writable<engine.SysFont[]>([]);
let started = false;
/** Le as letras instaladas (unha soa vez). */
export function loadFonts() {
  if (started) return;
  started = true;
  engine.listFonts().then((l) => sysFonts.set(l)).catch(() => {});
}

/** Familias do sistema, sen repetir (Arial, Arial Bold e Arial Italic son unha soa: Arial). */
export function systemFamilies(list: engine.SysFont[]): string[] {
  return [...new Set(list.map((f) => f.family))].sort((a, b) => a.localeCompare(b));
}

/** Nome da letra para o motor do PDF: unha das doce estándar ou «file:<ruta>» da variante máis parecida á pedida. */
export function resolveFont(s: FontSpec): string {
  if (s.family.startsWith("sys:")) {
    const fam = s.family.slice(4);
    const cands = get(sysFonts).filter((f) => f.family === fam);
    const score = (f: engine.SysFont) => Math.abs((f.weight >= 600 ? 1 : 0) - (s.bold ? 1 : 0)) * 2 + (f.italic === s.italic ? 0 : 1);
    const best = [...cands].sort((a, b) => score(a) - score(b))[0];
    if (best) return `file:${best.path}`;
    return resolveFont({ ...s, family: "Helvetica" });
  }
  const b = s.bold;
  const i = s.italic;
  if (s.family === "Times") return b && i ? "Times-BoldItalic" : b ? "Times-Bold" : i ? "Times-Italic" : "Times-Roman";
  if (s.family === "Courier") return b && i ? "Courier-BoldOblique" : b ? "Courier-Bold" : i ? "Courier-Oblique" : "Courier";
  return b && i ? "Helvetica-BoldOblique" : b ? "Helvetica-Bold" : i ? "Helvetica-Oblique" : "Helvetica";
}

/** Lista de familias CSS (para a vista previa e para o canvas). */
export function cssFamily(family: string): string {
  if (family.startsWith("sys:")) return `"${family.slice(4)}", sans-serif`;
  return (STANDARD_FAMILIES.find((f) => f.id === family) ?? STANDARD_FAMILIES[0]).css;
}

/** Declaracións CSS da letra, para a vista previa sobre o PDF. */
export function fontCss(s?: FontSpec): string {
  const f = s ?? { family: "Helvetica", bold: false, italic: false };
  return `font-family:${cssFamily(f.family)};font-weight:${f.bold ? 700 : 400};font-style:${f.italic ? "italic" : "normal"};`;
}
