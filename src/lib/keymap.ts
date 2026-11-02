// Atallos de teclado configurables: cada acción ten ata dous atallos (os dous valen á vez).
// Os cambios gárdanse e pódense devolver aos valores por defecto, un a un ou todos.

import { writable, get } from "svelte/store";

export type Slots = [string | null, string | null];
export type Group = "general" | "media" | "pdf" | "image";

export interface ActionDef {
  id: string;
  label: string;
  group: Group;
  keys: Slots;
}

export const GROUPS: { id: Group; title: string }[] = [
  { id: "general", title: "Xeral" },
  { id: "media", title: "Audio e vídeo" },
  { id: "pdf", title: "PDF e texto" },
  { id: "image", title: "Imaxe" },
];

export const ACTIONS: ActionDef[] = [
  { id: "g.open", group: "general", label: "Abrir ficheiros", keys: ["Ctrl+O", null] },
  { id: "g.palette", group: "general", label: "Buscar unha acción ou ferramenta", keys: ["Ctrl+K", null] },
  { id: "g.remove", group: "general", label: "Pechar (quitar da lista) o ficheiro que se está vendo", keys: ["Ctrl+W", null] },
  { id: "g.undo", group: "general", label: "Desfacer os axustes da ferramenta", keys: ["Ctrl+Z", null] },
  { id: "g.redo", group: "general", label: "Refacer os axustes da ferramenta", keys: ["Ctrl+Y", "Ctrl+Shift+Z"] },

  { id: "m.play", group: "media", label: "Reproducir e pausar", keys: ["Space", "K"] },
  { id: "m.back5", group: "media", label: "Retroceder 5 segundos", keys: ["ArrowLeft", null] },
  { id: "m.fwd5", group: "media", label: "Avanzar 5 segundos", keys: ["ArrowRight", null] },
  { id: "m.back1", group: "media", label: "Retroceder 1 segundo", keys: ["Shift+ArrowLeft", null] },
  { id: "m.fwd1", group: "media", label: "Avanzar 1 segundo", keys: ["Shift+ArrowRight", null] },
  { id: "m.back10", group: "media", label: "Retroceder 10 segundos", keys: ["J", null] },
  { id: "m.fwd10", group: "media", label: "Avanzar 10 segundos", keys: ["L", null] },
  { id: "m.volUp", group: "media", label: "Subir o volume", keys: ["ArrowUp", null] },
  { id: "m.volDown", group: "media", label: "Baixar o volume", keys: ["ArrowDown", null] },
  { id: "m.mute", group: "media", label: "Silenciar", keys: ["M", null] },
  { id: "m.fullscreen", group: "media", label: "Pantalla completa (vídeo)", keys: ["F", null] },
  { id: "m.start", group: "media", label: "Ir ao principio", keys: ["Home", null] },
  { id: "m.end", group: "media", label: "Ir ao final", keys: ["End", null] },
  { id: "m.framePrev", group: "media", label: "Un fotograma cara atrás (vídeo)", keys: [",", null] },
  { id: "m.frameNext", group: "media", label: "Un fotograma cara adiante (vídeo)", keys: [".", null] },

  { id: "p.find", group: "pdf", label: "Buscar no PDF ou no texto", keys: ["Ctrl+F", null] },
  { id: "p.next", group: "pdf", label: "Páxina seguinte", keys: ["PageDown", "ArrowRight"] },
  { id: "p.prev", group: "pdf", label: "Páxina anterior", keys: ["PageUp", "ArrowLeft"] },
  { id: "p.first", group: "pdf", label: "Primeira páxina", keys: ["Home", null] },
  { id: "p.last", group: "pdf", label: "Última páxina", keys: ["End", null] },
  { id: "p.zoomIn", group: "pdf", label: "Ampliar", keys: ["+", "="] },
  { id: "p.zoomOut", group: "pdf", label: "Reducir", keys: ["-", null] },
  { id: "p.fitWidth", group: "pdf", label: "Axustar ao ancho", keys: ["0", null] },
  { id: "p.fitPage", group: "pdf", label: "Axustar á páxina enteira", keys: ["9", null] },

  { id: "i.zoomIn", group: "image", label: "Ampliar", keys: ["+", "="] },
  { id: "i.zoomOut", group: "image", label: "Reducir", keys: ["-", null] },
  { id: "i.fit", group: "image", label: "Axustar á xanela", keys: ["0", null] },
  { id: "i.actual", group: "image", label: "Tamaño real (100 %)", keys: ["1", null] },
];

const KEY = "arquivos-do-trasno.atallos";

function load(): Record<string, Slots> {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? "{}") as Record<string, Slots>;
    const out: Record<string, Slots> = {};
    for (const [id, v] of Object.entries(raw)) {
      if (ACTIONS.some((a) => a.id === id) && Array.isArray(v) && v.length === 2) out[id] = [v[0] ?? null, v[1] ?? null];
    }
    return out;
  } catch {
    return {};
  }
}

/** Só se gardan as accións que o usuario mudou; o resto segue os valores por defecto. */
export const custom = writable<Record<string, Slots>>(load());
custom.subscribe((v) => {
  try {
    localStorage.setItem(KEY, JSON.stringify(v));
  } catch {
    // sen almacenamento: vale só mentres a app está aberta
  }
});

const def = (id: string) => ACTIONS.find((a) => a.id === id)!;
export const slotsOf = (id: string, map = get(custom)): Slots => map[id] ?? def(id).keys;
export const isChanged = (id: string, map = get(custom)) => !!map[id];

/** Atallo en texto estable («Ctrl+Shift+Z», «ArrowLeft», «K»); null se só se premeu un modificador. */
export function comboOf(e: KeyboardEvent): string | null {
  if (["Control", "Shift", "Alt", "Meta", "AltGraph", "CapsLock"].includes(e.key)) return null;
  let key = e.key === " " ? "Space" : e.key;
  if (key.length === 1) key = key.toUpperCase();
  const ctrl = e.ctrlKey || e.metaKey;
  // Maiús só conta nas teclas con nome (frechas…) ou xunto a Ctrl/Alt: «+» ou «L» maiúscula xa levan o seu Maiús.
  const shift = e.shiftKey && (key.length > 1 || ctrl || e.altKey);
  return [ctrl ? "Ctrl" : "", e.altKey ? "Alt" : "", shift ? "Shift" : "", key].filter(Boolean).join("+");
}

/** Acción do grupo (ou xeral) á que corresponde a tecla premida. */
export function actionFor(e: KeyboardEvent, group: Group): string | null {
  const combo = comboOf(e);
  if (!combo) return null;
  const map = get(custom);
  for (const a of ACTIONS) {
    if (a.group !== group) continue;
    if (slotsOf(a.id, map).includes(combo)) return a.id;
  }
  return null;
}

/** Pon un atallo nunha posición. Se xa o usaba outra acción do mesmo ámbito, quítallo e devólvese o seu id. */
export function setSlot(id: string, slot: 0 | 1, combo: string | null): string | null {
  let stolen: string | null = null;
  custom.update((map) => {
    const next = { ...map };
    const g = def(id).group;
    if (combo) {
      for (const a of ACTIONS) {
        if (a.id === id) continue;
        const clash = a.group === g || a.group === "general" || g === "general";
        if (!clash) continue;
        const s = slotsOf(a.id, next);
        if (s.includes(combo)) {
          next[a.id] = [s[0] === combo ? null : s[0], s[1] === combo ? null : s[1]];
          stolen = a.id;
        }
      }
    }
    const mine = [...slotsOf(id, next)] as Slots;
    mine[slot] = combo;
    // Se os dous quedan como os de fábrica, deixa de contar como mudado.
    next[id] = mine;
    if (JSON.stringify(mine) === JSON.stringify(def(id).keys)) delete next[id];
    return next;
  });
  return stolen;
}

export function resetAction(id: string) {
  custom.update((m) => {
    const n = { ...m };
    delete n[id];
    return n;
  });
}
export const resetAll = () => custom.set({});

const NAMES: Record<string, string> = {
  ArrowLeft: "←", ArrowRight: "→", ArrowUp: "↑", ArrowDown: "↓",
  Space: "Espazo", Delete: "Supr", Escape: "Esc", PageUp: "PáxRe", PageDown: "PáxAv", Home: "Inicio", End: "Fin", Enter: "Intro", Backspace: "Retroceso", Tab: "Tab",
};
/** Texto para amosar: «Ctrl + Maiús + Z», «←», «Espazo». */
export function pretty(combo: string | null): string {
  if (!combo) return "";
  const key = combo.endsWith("+") ? "+" : combo.split("+").pop()!;
  const mods = combo.slice(0, combo.length - key.length).split("+").filter(Boolean).map((m) => (m === "Shift" ? "Maiús" : m));
  return [...mods, NAMES[key] ?? key].join(" + ");
}
