// Preferencias da interface (tema) e estado dos programas externos (extensións).

import { writable, get } from "svelte/store";
import { getCurrentWindow } from "@tauri-apps/api/window";
import * as engine from "./engine";

export type Theme = "system" | "light" | "dark";
const THEME_KEY = "arquivos-do-trasno.theme";
const WELCOME_KEY = "arquivos-do-trasno.extensions-seen";

function savedTheme(): Theme {
  try {
    const v = localStorage.getItem(THEME_KEY);
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {
    // sen almacenamento: segue o sistema
  }
  return "system";
}

export const theme = writable<Theme>(savedTheme());
/** Tema que se está vendo de verdade (o «sistema» xa resolto). */
export const resolvedTheme = writable<"light" | "dark">("dark");

const media = typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: light)") : null;

function apply(t: Theme) {
  const light = t === "light" || (t === "system" && !!media?.matches);
  document.documentElement.dataset.theme = light ? "light" : "dark";
  resolvedTheme.set(light ? "light" : "dark");
  // A barra de título nativa tamén segue o tema.
  getCurrentWindow().setTheme(t === "system" ? null : light ? "light" : "dark").catch(() => {});
}

theme.subscribe((t) => {
  try {
    localStorage.setItem(THEME_KEY, t);
  } catch {
    // ignorar
  }
  if (typeof document !== "undefined") apply(t);
});
media?.addEventListener("change", () => apply(get(theme)));

/** Que xanela modal está aberta. */
export const dialog = writable<"tools" | "credits" | null>(null);

/** Programas externos que a app aproveita. `null` ata saber o resultado. */
export const tools = writable<engine.ToolsStatus | null>(null);

export async function refreshTools(): Promise<engine.ToolsStatus> {
  const s = await engine.toolsStatus();
  tools.set(s);
  return s;
}

/** Na primeira execución, se falta algo, ábrese a xanela de extensións unha vez. */
export async function welcomeIfNeeded() {
  const s = await refreshTools();
  let seen = false;
  try {
    seen = localStorage.getItem(WELCOME_KEY) === "1";
  } catch {
    // ignorar
  }
  if (!seen && !(s.ffmpeg && s.mkvmerge)) dialog.set("tools");
  try {
    localStorage.setItem(WELCOME_KEY, "1");
  } catch {
    // ignorar
  }
}
