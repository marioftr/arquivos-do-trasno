// O trasno como axente: lembra o que fixo a persoa (para contalo na barra inferior) e ten un
// repertorio de consellos que solta de cando en vez. Os consellos pódense apagar en Axustes.

import { writable } from "svelte/store";
import { tr } from "./i18n";

function persistedBool(key: string, fallback: boolean) {
  let v = fallback;
  try {
    const s = localStorage.getItem(key);
    if (s === "1" || s === "0") v = s === "1";
  } catch {
    // ignorar
  }
  const store = writable(v);
  store.subscribe((x) => {
    try {
      localStorage.setItem(key, x ? "1" : "0");
    } catch {
      // ignorar
    }
  });
  return store;
}

export const tipsOn = persistedBool("arquivos-do-trasno.tips", true);

export interface Action {
  text: string;
  at: number;
}
/** Últimas accións, a máis recente primeiro (en primeira persoa: fainas o trasno). */
export const actions = writable<Action[]>([]);
export function noteAction(gl: string, vars?: Record<string, string | number>) {
  actions.update((l) => [{ text: tr(gl, vars), at: Date.now() }, ...l].slice(0, 8));
}

/** Avance en directo do traballo da app en marcha (a lista de procesos só se refresca cada poucos segundos). */
export const liveProgress = writable<number | null>(null);

export const TIPS = [
  "Ctrl+K abre o buscador de accións. Tamén podes premerme a min.",
  "Arrastra ficheiros á xanela e engádoos sen botóns.",
  "Con Seleccionar podes mover, escalar e xirar calquera anotación.",
  "Ctrl+Z desfai o último cambio dos axustes e Restablecer volve ao por defecto.",
  "Cada ficheiro lembra os seus axustes mentres a app siga aberta.",
  "Co botón central do rato móvese a imaxe aínda que esteas anotando.",
  "Na lista podes arrastrar os ficheiros para cambiarlles a orde.",
  "Se marcas varios ficheiros do mesmo tipo, a ferramenta aplícase a todos á vez.",
  "Quitar un ficheiro da lista non o borra do disco, e ata podes desfacelo.",
  "Preme o recadro de progreso para ver todos os procesos.",
  "O texto das imaxes pódese seleccionar e copiar coa ferramenta de texto.",
];
