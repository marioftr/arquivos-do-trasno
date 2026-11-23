// Actualizacións desde GitHub Releases (plugin updater de Tauri): comprobar, descargar, instalar e reiniciar.
// Só funciona na app instalada (o .exe solto non ten ficheiros de actualización).

import { writable, get } from "svelte/store";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { toast } from "./toasts";
import { tr } from "./i18n";
import { procs } from "./procs";

export const appVersion = writable("");
getVersion().then((v) => appVersion.set(v)).catch(() => {});

/** Das notas da release (con marcadores <!-- gl -->, <!-- es -->, <!-- en -->), as viñetas do idioma pedido. */
export function notesFor(body: string, lang: string): string[] {
  const parts = body.split(/<!--\s*(gl|es|en)\s*-->/);
  const found: Record<string, string> = {};
  for (let i = 1; i < parts.length; i += 2) found[parts[i]] = parts[i + 1];
  const text = found[lang] ?? found.gl ?? body;
  return text.split("\n").map((l) => l.trim()).filter((l) => l.startsWith("- ")).map((l) => l.slice(2));
}

export interface Found {
  version: string;
  notes: string;
  date?: string;
}
export type UpdState = "idle" | "checking" | "none" | "found" | "downloading" | "ready" | "error";
export const updState = writable<UpdState>("idle");
export const updFound = writable<Found | null>(null);
export const updError = writable("");
export const updProgress = writable(0); // 0..1
/** Mentres haxa unha versión nova sen instalar, o botón Novidades leva un punto. */
export const updateAvailable = writable(false);

/** «Actualizar automaticamente»: acendido por defecto; quen non o queira pode apagalo. */
const AUTO_KEY = "arquivos-do-trasno.buscar-actualizacions";
export const autoCheck = writable(((): boolean => {
  try {
    return localStorage.getItem(AUTO_KEY) !== "0";
  } catch {
    return true;
  }
})());
let first = true;
autoCheck.subscribe((v) => {
  try {
    localStorage.setItem(AUTO_KEY, v ? "1" : "0");
  } catch {
    // ignorar
  }
  // Se se acende co programa aberto, busca xa; ao arrancar xa o fai App.svelte un pouco despois.
  if (!first && v) void autoUpdate();
  first = false;
});

let pending: Update | null = null;

/** Busca unha versión nova e, se hai, descárgaa e instálaa soa (se o usuario non apagou a actualización automática). */
export async function autoUpdate(): Promise<void> {
  if (!get(autoCheck)) return;
  await checkForUpdate();
  if (get(updState) === "found") await applyUpdate();
}

async function checkForUpdate(): Promise<void> {
  if (get(updState) === "checking" || get(updState) === "downloading" || get(updState) === "ready") return;
  updState.set("checking");
  updError.set("");
  try {
    const u = await check();
    if (u) {
      pending = u;
      updFound.set({ version: u.version, notes: u.body ?? "", date: u.date });
      updState.set("found");
      updateAvailable.set(true);
    } else {
      updFound.set(null);
      updState.set("none");
      updateAvailable.set(false);
    }
  } catch (e) {
    // Na copia de desenvolvemento ou sen internet non hai nada que dicir: queda en silencio.
    updState.set("idle");
    updError.set(String((e as any)?.message ?? e));
  }
}

/** Resolve cando non hai ningún proceso en marcha: instalar pecha a app, e un traballo a medias perdería o que leva feito. */
function whenIdle(): Promise<void> {
  return new Promise((resolve) => {
    let off = () => {};
    off = procs.subscribe((l) => {
      if (!l.some((p) => p.state === "running")) {
        queueMicrotask(() => off());
        resolve();
      }
    });
  });
}

/** Descarga a versión nova, agarda a que a app estea libre e instálaa (a app reiníciase soa). */
async function applyUpdate(): Promise<void> {
  if (!pending) return;
  const u = pending;
  updState.set("downloading");
  updProgress.set(0);
  try {
    let total = 0;
    let done = 0;
    await u.download((ev) => {
      if (ev.event === "Started") total = ev.data.contentLength ?? 0;
      else if (ev.event === "Progress") {
        done += ev.data.chunkLength;
        if (total) updProgress.set(Math.min(1, done / total));
      }
    });
    updState.set("ready");
    await whenIdle();
    toast(tr("Instalo a versión {v}. A app reiníciase soa.", { v: u.version }), { ms: 4000 });
    await new Promise((r) => setTimeout(r, 3000));
    await u.install();
    await relaunch();
  } catch (e) {
    updState.set("error");
    updError.set(String((e as any)?.message ?? e));
  }
}
