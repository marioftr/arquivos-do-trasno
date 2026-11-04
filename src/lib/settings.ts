// Preferencias da interface (tema) e estado dos programas externos (extensións).

import { writable, get, derived } from "svelte/store";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebview } from "@tauri-apps/api/webview";
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

// ---- Accesibilidade: animacións e tamaño da interface -------------------------------------------------
const osReduces = () => typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
/** «system»: segue o axuste do sistema operativo; «reduce» e «full»: elección da persoa na app. */
export const motionPref = persisted<"system" | "reduce" | "full">("arquivos-do-trasno.motion", ["system", "reduce", "full"], "system");
/** Se hai que reducir as animacións (por axuste da app ou, se non se tocou, do sistema). */
export const motionReduced = derived(motionPref, (p) => p === "reduce" || (p === "system" && osReduces()));
motionReduced.subscribe((r) => {
  if (typeof document !== "undefined") document.documentElement.dataset.motion = r ? "reduce" : "full";
});

/** Tamaño da interface (texto e controis por igual, como o zoom do navegador). */
export const UI_SCALES = ["100", "115", "130", "150"] as const;
export type UiScale = (typeof UI_SCALES)[number];
export const uiScale = persisted<UiScale>("arquivos-do-trasno.ui-scale", [...UI_SCALES], "100");
/** Aplica o zoom da interface (a xanela non se toca: o deseño adáptase ao ancho que quede). */
export async function applyUiScale(v: UiScale) {
  const z = Number(v) / 100;
  try {
    await getCurrentWebview().setZoom(z);
  } catch {
    // fóra de Tauri (probas no navegador): sen zoom
  }
}
uiScale.subscribe((v) => void applyUiScale(v));

/** Onde se gardan os resultados: preguntar cada vez, xunto ao orixinal ou nun cartafol fixo. */
export type OutputMode = "ask" | "beside" | "folder";
function persisted<T extends string>(key: string, allowed: T[], fallback: T) {
  let v = fallback;
  try {
    const s = localStorage.getItem(key);
    if (s && (allowed as string[]).includes(s)) v = s as T;
  } catch {
    // ignorar
  }
  const store = writable<T>(v);
  store.subscribe((x) => {
    try {
      localStorage.setItem(key, x);
    } catch {
      // ignorar
    }
  });
  return store;
}
export const outputMode = persisted<OutputMode>("arquivos-do-trasno.output-mode", ["ask", "beside", "folder"], "ask");
function persistedText(key: string) {
  let v = "";
  try {
    v = localStorage.getItem(key) ?? "";
  } catch {
    // ignorar
  }
  const store = writable<string>(v);
  store.subscribe((x) => {
    try {
      localStorage.setItem(key, x);
    } catch {
      // ignorar
    }
  });
  return store;
}
export const outputFolder = persistedText("arquivos-do-trasno.output-folder");

/** Que xanela modal está aberta: o centro (procesos, axustes, novidades, créditos) ou o buscador de accións. */
export const dialog = writable<"hub" | "palette" | null>(null);
/** Páxina aberta no centro, como «grupo.subcategoría». */
export const hubPage = writable("settings.general");
export function openHub(page: string) {
  hubPage.set(page);
  dialog.set("hub");
}

/** Ferramentas do ficheiro actual (para o buscador Ctrl+K) e petición de saltar a unha delas. */
export interface ToolAction {
  id: string;
  label: string;
  tab: string;
  mode?: string;
}
export const toolActions = writable<ToolAction[]>([]);
export const jump = writable<{ tab: string; mode?: string } | null>(null);
export type SettingsTab = "general" | "shortcuts" | "profiles" | "tools" | "credits";

/** Perfil de saída de vídeo gardado polo usuario (pestana Comprimir). */
export interface ProfileValues {
  mode: "quality" | "bitrate";
  crf: number;
  videoKbps: number;
  maxHeight: number;
  audioKbps: number;
  deinterlace: number;
  compat: boolean;
  dar: string;
}
export interface SavedProfile {
  id: string;
  name: string;
  v: ProfileValues;
}
const PROFILES_KEY = "arquivos-do-trasno.perfis";
function loadProfiles(): SavedProfile[] {
  try {
    const l = JSON.parse(localStorage.getItem(PROFILES_KEY) ?? "[]");
    return Array.isArray(l) ? l.filter((p) => p && typeof p.id === "string" && typeof p.name === "string" && p.v) : [];
  } catch {
    return [];
  }
}
export const savedProfiles = writable<SavedProfile[]>(loadProfiles());
savedProfiles.subscribe((l) => {
  try {
    localStorage.setItem(PROFILES_KEY, JSON.stringify(l));
  } catch {
    // sen almacenamento: vale mentres a app está aberta
  }
});
/** Resumo curto dos axustes dun perfil, para amosalo en listas. */
export function summaryOf(v: ProfileValues): string {
  const bits = [v.mode === "bitrate" ? `${v.videoKbps} kbps` : `CRF ${v.crf}`, v.maxHeight ? `${v.maxHeight}p` : "", `${v.audioKbps} kbps audio`, v.deinterlace === 2 ? "yadif×2" : v.deinterlace === 1 ? "yadif" : "", v.dar ? v.dar.replace("/", ":") : ""];
  return bits.filter(Boolean).join(" · ");
}
export function openSettings(tab: SettingsTab = "general") {
  openHub(tab === "credits" ? "about.credits" : `settings.${tab}`);
}

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
  if (!seen && !(s.ffmpeg && s.mkvmerge)) openSettings("tools");
  try {
    localStorage.setItem(WELCOME_KEY, "1");
  } catch {
    // ignorar
  }
}
