// Descarga dos programas externos (FFmpeg, MKVToolNix, whisper.cpp e yt-dlp) desde a propia app, e o aviso que sae cando unha acción os necesita.

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
  /** Para que se usa, en tres palabras (na lista de «falta instalar»). */
  role: string;
  link: LinkId;
  winget: string;
  size: string;
  /** Tamaño da descarga, en bytes. */
  bytes: number;
  source: string;
  /** Páxina do proxecto, a licenza e o enderezo que se amosa. */
  project: LinkId;
  repo: string;
  license: string;
}
const MB = 1024 * 1024;

export const TOOL_INFO: Record<ToolId, ToolInfo> = {
  ffmpeg: {
    name: "FFmpeg",
    role: "audio e vídeo",
    bytes: 115 * MB,
    project: "ffmpeg-project",
    repo: "github.com/FFmpeg/FFmpeg",
    license: "LGPL / GPL",
    why: "Lee e escribe case todos os formatos de audio e vídeo.",
    unlocks: "Convertir, comprimir, editar, cortar e unir audio e vídeo, extraer fotogramas e audio, efectos de son, vistas previas de MKV, AVI ou AIFF, e ler ou escribir imaxes AVIF, JPEG 2000 e JPEG XL.",
    link: "ffmpeg",
    winget: "winget install ffmpeg",
    size: "115 MB",
    source: "gyan.dev",
  },
  mkvmerge: {
    name: "MKVToolNix",
    role: "pistas de MKV",
    bytes: 31 * MB,
    project: "mkvtoolnix-project",
    repo: "gitlab.com/mbunkus/mkvtoolnix",
    license: "GPL-2.0",
    why: "Edita os contedores Matroska (MKV) sen recodificar nada, así que non hai perda de calidade.",
    unlocks: "Quitar ou reordenar pistas de audio e subtítulos nun MKV, cambiarlles o nome e o idioma e marcar cal vai por defecto.",
    link: "mkvtoolnix",
    winget: "winget install MoritzBunkus.MKVToolNix",
    size: "31 MB",
    source: "mkvtoolnix.download",
  },
  whisper: {
    name: "whisper.cpp",
    role: "programa de transcrición",
    bytes: 9 * MB,
    project: "whispercpp",
    repo: "github.com/ggml-org/whisper.cpp",
    license: "MIT",
    why: "Pasa a voz a texto. Traballa co modelo de recoñecemento de voz que se elixa, que se descarga á parte.",
    unlocks: "Transcribir audio e vídeo a subtítulos e texto.",
    link: "whispercpp",
    winget: "",
    size: "9 MB",
    source: "github.com/ggml-org/whisper.cpp",
  },
  ytdlp: {
    name: "yt-dlp",
    role: "descarga vídeos",
    bytes: 24 * MB,
    project: "ytdlp",
    repo: "github.com/yt-dlp/yt-dlp",
    license: "Unlicense",
    why: "Descarga vídeo e audio dunha ligazón. Convén telo ao día, porque as webs cambian.",
    unlocks: "Descargar vídeo ou audio por ligazón, para velo e traballar con el despois.",
    link: "ytdlp",
    winget: "winget install yt-dlp.yt-dlp",
    size: "24 MB",
    source: "github.com/yt-dlp/yt-dlp",
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
    if (isCancelled(e)) toast(tr("Descarga cancelada. Gardo as ferramentas e non pasa nada."));
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
        ? tr("Para facer isto necesito FFmpeg, que non está instalado.")
        : tool === "mkvmerge"
          ? tr("Para facer isto necesito MKVToolNix, que non está instalado.")
          : tool === "whisper"
            ? tr("Para transcribir necesito whisper.cpp, que non está instalado.")
            : tr("Para descargar necesito yt-dlp, que non está instalado.")),
    retry,
  });
}
