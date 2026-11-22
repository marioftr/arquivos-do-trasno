// Modelos opcionais (voz para transcribir, imaxe para quitar fondos): descárganse a petición e compróbanse co SHA-256 fixado na app.

import { writable, get } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import * as engine from "./engine";
import type { ModelId } from "./engine";
import { errText, isCancelled } from "./errors";
import { tools } from "./settings";
import { tr } from "./i18n";
import { toast } from "./toasts";

export interface ModelInfo {
  id: ModelId;
  /** Nome curto na interface. */
  label: string;
  /** Nome do modelo orixinal. */
  name: string;
  size: string;
  bytes: number;
  /** Para que serve, en poucas palabras (vai nunha «i»). */
  hint: string;
}

/** Modelos de voz de whisper.cpp, de menos a máis preciso. */
export const WHISPER_MODELS: ModelInfo[] = [
  { id: "whisper-base", label: "Rápida", name: "Whisper base", size: "57 MB", bytes: 59_707_625, hint: "A máis lixeira e rápida; con ruído, acentos ou idiomas pouco comúns, como o galego, comete máis erros." },
  { id: "whisper-small", label: "Equilibrada", name: "Whisper small", size: "181 MB", bytes: 190_085_487, hint: "Boa precisión na maioría das gravacións, a un ritmo medio." },
  { id: "whisper-medium", label: "Precisa", name: "Whisper medium", size: "514 MB", bytes: 539_212_467, hint: "A máis exacta, tamén con acentos e ruído; é a máis lenta e a que máis memoria usa." },
];
export const CUTOUT_MODEL: ModelInfo = { id: "cutout", label: "IS-Net", name: "IS-Net", size: "170 MB", bytes: 178_648_008, hint: "Recoñece o obxecto principal dunha imaxe." };
/** Páxina e licenza do proxecto de cada modelo. */
export const MODEL_PROJECT: Record<ModelId, { link: engine.LinkId; repo: string; license: string }> = {
  "whisper-base": { link: "whisper-openai", repo: "github.com/openai/whisper", license: "MIT" },
  "whisper-small": { link: "whisper-openai", repo: "github.com/openai/whisper", license: "MIT" },
  "whisper-medium": { link: "whisper-openai", repo: "github.com/openai/whisper", license: "MIT" },
  cutout: { link: "dis", repo: "github.com/xuebinqin/DIS", license: "Apache-2.0" },
};
export const MODEL_SOURCE: Record<ModelId, string> = {
  "whisper-base": "huggingface.co/ggerganov/whisper.cpp",
  "whisper-small": "huggingface.co/ggerganov/whisper.cpp",
  "whisper-medium": "huggingface.co/ggerganov/whisper.cpp",
  cutout: "github.com/danielgatis/rembg",
};

export const modelOk = (id: ModelId) => !!get(tools)?.models?.includes(id);

/** Descarga en marcha (ou `null`). */
export const modelBusy = writable<{ id: ModelId; progress: number } | null>(null);
export const modelError = writable<{ id: ModelId; text: string } | null>(null);

let listening = false;
async function ensureListener() {
  if (listening) return;
  listening = true;
  await listen<{ id: ModelId; progress: number }>("trasno://model-download", (e) => modelBusy.update((s) => (s && s.id === e.payload.id ? { ...s, progress: e.payload.progress } : s)));
}

/** Descarga un modelo; devolve se quedou instalado. */
export async function downloadModel(id: ModelId): Promise<boolean> {
  if (get(modelBusy)) return false;
  await ensureListener();
  modelError.set(null);
  modelBusy.set({ id, progress: 0 });
  try {
    tools.set(await engine.installModel(id));
    return true;
  } catch (e) {
    if (isCancelled(e)) toast(tr("Descarga cancelada. Gardo as ferramentas e non pasa nada."));
    else modelError.set({ id, text: errText(e) });
    return false;
  } finally {
    modelBusy.set(null);
  }
}

export async function removeModelNow(id: ModelId) {
  try {
    tools.set(await engine.removeModel(id));
  } catch (e) {
    modelError.set({ id, text: errText(e) });
  }
}

/** Algo que fai falta para unha ferramenta: un programa ou un modelo. */
export type Need = { kind: "tool"; id: engine.ToolId } | { kind: "model"; id: ModelId };

/** O que aínda non está descargado (con `null`, sen saber o estado, non se bloquea nada). */
export function missingOf(needs: Need[], s: engine.ToolsStatus | null): Need[] {
  if (!s) return [];
  return needs.filter((n) => (n.kind === "tool" ? !s[n.id] : !s.models?.includes(n.id)));
}
