// Descarga de FFmpeg e MKVToolNix desde a propia app, e o aviso que sae cando unha acción os necesita.

import { writable, get } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import * as engine from "./engine";
import type { ToolId, LinkId } from "./engine";
import { errText, isCancelled } from "./errors";
import { tools } from "./settings";
import { tr } from "./i18n";
import { toast } from "./toasts";

export interface ToolInfo {
  name: string;
  why: string;
  unlocks: string;
  link: LinkId;
  winget: string;
  size: string;
  source: string;
}

export const TOOL_INFO: Record<ToolId, ToolInfo> = {
  ffmpeg: {
    name: "FFmpeg",
    why: "Lee e escribe case todos os formatos de audio e vídeo.",
    unlocks: "Convertir, comprimir, editar, cortar e unir audio e vídeo, extraer fotogramas e audio, efectos de son, vistas previas de MKV, AVI ou AIFF, e ler ou escribir imaxes AVIF, JPEG 2000 e JPEG XL.",
    link: "ffmpeg",
    winget: "winget install ffmpeg",
    size: "115 MB",
    source: "gyan.dev",
  },
  mkvmerge: {
    name: "MKVToolNix",
    why: "Edita os contedores Matroska (MKV) sen recodificar nada, así que non hai perda de calidade.",
    unlocks: "Quitar ou reordenar pistas de audio e subtítulos nun MKV, cambiarlles o nome e o idioma e marcar cal vai por defecto.",
    link: "mkvtoolnix",
    winget: "winget install MoritzBunkus.MKVToolNix",
    size: "31 MB",
    source: "mkvtoolnix.download",
  },
};

export type InstallPhase = "download" | "verify" | "extract";
export interface InstallState {
  tool: ToolId;
  phase: InstallPhase;
  progress: number;
}

/** Descarga en marcha (ou `null`). */
export const installing = writable<InstallState | null>(null);
export const installError = writable<{ tool: ToolId; text: string } | null>(null);

let listening = false;
async function ensureListener() {
  if (listening) return;
  listening = true;
  await listen<{ tool: ToolId; phase: InstallPhase; progress: number }>("trasno://tool-install", (e) => {
    installing.update((s) => (s ? { ...s, phase: e.payload.phase, progress: e.payload.progress } : s));
  });
}

/** Avance global (0 a 1): a descarga é case todo o tempo. */
export const overall = (s: InstallState) => (s.phase === "download" ? s.progress * 0.8 : s.phase === "verify" ? 0.8 : 0.8 + s.progress * 0.2);
export const phaseText = (p: InstallPhase) => (p === "download" ? tr("Descargando…") : p === "verify" ? tr("Comprobando a descarga…") : tr("Instalando…"));

/** Descarga e instala; devolve se quedou instalado. */
export async function installToolNow(tool: ToolId): Promise<boolean> {
  if (get(installing)) return false;
  await ensureListener();
  installError.set(null);
  installing.set({ tool, phase: "download", progress: 0 });
  try {
    tools.set(await engine.installTool(tool));
    toast(tr("{name} xa está na casa. A traballar!", { name: TOOL_INFO[tool].name }));
    return true;
  } catch (e) {
    if (isCancelled(e)) toast(tr("Descarga cancelada. O trasno garda as ferramentas e non pasa nada."));
    else installError.set({ tool, text: errText(e) });
    return false;
  } finally {
    installing.set(null);
  }
}

export async function removeToolNow(tool: ToolId) {
  try {
    tools.set(await engine.removeTool(tool));
  } catch (e) {
    installError.set({ tool, text: errText(e) });
  }
}

// ---- Aviso «fai falta…» cando unha acción necesita un programa que non está.

export interface ToolNeed {
  tool: ToolId;
  /** Frase que explica que se quería facer e por que falta o programa. */
  reason: string;
  /** Que facer cando xa estea instalado (por exemplo, repetir a acción). */
  retry?: () => void;
}
export const need = writable<ToolNeed | null>(null);

export function requestTool(tool: ToolId, reason?: string, retry?: () => void) {
  installError.set(null);
  need.set({
    tool,
    reason:
      reason ??
      (tool === "ffmpeg"
        ? tr("Para facer isto o trasno precisa FFmpeg, que non está instalado.")
        : tr("Para facer isto o trasno precisa MKVToolNix, que non está instalado.")),
    retry,
  });
}
