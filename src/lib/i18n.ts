// Tradución mínima: o texto en galego é a propia chave. Galego devolve a
// chave tal cal; castelán e inglés buscan en locales/es.json e en.json.
// Se falta unha tradución, móstrase o galego (nunca unha chave rara).
// `scripts/check-i18n.mjs` comproba que non falta nada.

import { writable, derived, get } from "svelte/store";
import es from "./locales/es.json";
import en from "./locales/en.json";

export type Lang = "gl" | "es" | "en";
export const LANGS: { id: Lang; label: string }[] = [
  { id: "gl", label: "Galego" },
  { id: "es", label: "Castellano" },
  { id: "en", label: "English" },
];

const DICTS: Record<Lang, Record<string, string>> = { gl: {}, es, en };
const STORAGE_KEY = "arquivos-do-trasno.lang";

function initial(): Lang {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "gl" || saved === "es" || saved === "en") return saved;
  } catch {
    // sen almacenamento: úsase o idioma do sistema
  }
  const nav = (navigator.language || "gl").slice(0, 2).toLowerCase();
  return nav === "es" ? "es" : nav === "en" ? "en" : "gl";
}

export const lang = writable<Lang>(initial());

lang.subscribe((l) => {
  try {
    localStorage.setItem(STORAGE_KEY, l);
  } catch {
    // ignorar
  }
  if (typeof document !== "undefined") document.documentElement.lang = l;
});

export type Params = Record<string, string | number>;

export function translate(l: Lang, key: string, params?: Params): string {
  let text = DICTS[l][key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) text = text.split(`{${k}}`).join(String(v));
  }
  return text;
}

/** Reactivo: úsase nas plantillas como `$t("texto")`. */
export const t = derived(lang, (l) => (key: string, params?: Params) => translate(l, key, params));

/** Non reactivo: para mensaxes que se constrúen dentro de funcións. */
export function tr(key: string, params?: Params): string {
  return translate(get(lang), key, params);
}
