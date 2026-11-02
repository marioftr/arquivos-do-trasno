// Letras para marcas, números e textos nun PDF: as 12 estándar (non hai que incrustalas) e as instaladas no sistema.
// Un identificador de letra é o nome estándar («Times-Bold») ou «file:<ruta>» para unha do sistema.

import { writable, get } from "svelte/store";
import * as engine from "./engine";

export const STANDARD_FONTS: { id: string; label: string }[] = [
  { id: "Helvetica", label: "Helvetica" },
  { id: "Helvetica-Bold", label: "Helvetica Negriña" },
  { id: "Helvetica-Oblique", label: "Helvetica Cursiva" },
  { id: "Helvetica-BoldOblique", label: "Helvetica Negriña cursiva" },
  { id: "Times-Roman", label: "Times" },
  { id: "Times-Bold", label: "Times Negriña" },
  { id: "Times-Italic", label: "Times Cursiva" },
  { id: "Times-BoldItalic", label: "Times Negriña cursiva" },
  { id: "Courier", label: "Courier" },
  { id: "Courier-Bold", label: "Courier Negriña" },
  { id: "Courier-Oblique", label: "Courier Cursiva" },
  { id: "Courier-BoldOblique", label: "Courier Negriña cursiva" },
];

export const sysFonts = writable<engine.SysFont[]>([]);
let started = false;
/** Le as letras instaladas (unha soa vez). */
export function loadFonts() {
  if (started) return;
  started = true;
  engine.listFonts().then((l) => sysFonts.set(l)).catch(() => {});
}

/** CSS equivalente á letra, para a vista previa sobre o PDF. */
export function fontCss(id?: string): string {
  if (id?.startsWith("file:")) {
    const f = get(sysFonts).find((x) => `file:${x.path}` === id);
    if (f) return `font-family:"${f.family}",sans-serif;font-weight:${f.weight};font-style:${f.italic ? "italic" : "normal"};`;
  }
  const n = id ?? "Helvetica";
  const fam = n.startsWith("Times") ? '"Times New Roman",Times,serif' : n.startsWith("Courier") ? '"Courier New",Courier,monospace' : "Helvetica,Arial,sans-serif";
  return `font-family:${fam};font-weight:${/Bold/.test(n) ? 700 : 400};font-style:${/Italic|Oblique/.test(n) ? "italic" : "normal"};`;
}
