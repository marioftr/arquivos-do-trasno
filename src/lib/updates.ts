// Actualizacións desde GitHub Releases (plugin updater de Tauri): comprobar, descargar, instalar e reiniciar.
// Só funciona na app instalada (o .exe solto non ten ficheiros de actualización).

import { writable, get } from "svelte/store";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { toast } from "./toasts";
import { tr } from "./i18n";
import { openHub } from "./settings";
import * as engine from "./engine";

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
export type UpdState = "idle" | "checking" | "none" | "found" | "downloading" | "error";
export const updState = writable<UpdState>("idle");
export const updFound = writable<Found | null>(null);
export const updError = writable("");
export const updProgress = writable(0); // 0..1
/** Mentres haxa unha versión nova sen instalar, o botón Novidades leva un punto. */
export const updateAvailable = writable(false);

const AUTO_KEY = "arquivos-do-trasno.buscar-actualizacions";
const EX_KEY = "arquivos-do-trasno.actualizar-menu-explorador";
export const autoCheck = writable(((): boolean => {
  try {
    return localStorage.getItem(AUTO_KEY) !== "0";
  } catch {
    return true;
  }
})());
autoCheck.subscribe((v) => {
  try {
    localStorage.setItem(AUTO_KEY, v ? "1" : "0");
  } catch {
    // ignorar
  }
});

let pending: Update | null = null;

export async function checkForUpdate(silent = false): Promise<void> {
  if (get(updState) === "checking" || get(updState) === "downloading") return;
  updState.set("checking");
  updError.set("");
  try {
    const u = await check();
    if (u) {
      pending = u;
      updFound.set({ version: u.version, notes: u.body ?? "", date: u.date });
      updState.set("found");
      updateAvailable.set(true);
      if (silent) {
        toast(tr("O trasno estreou roupa nova: a versión {v}.", { v: u.version }), { action: { label: tr("Ver novidades"), run: () => openHub("news.updates") }, ms: 12000 });
      }
    } else {
      updFound.set(null);
      updState.set("none");
      updateAvailable.set(false);
    }
  } catch (e) {
    updState.set(silent ? "idle" : "error");
    updError.set(String((e as any)?.message ?? e));
  }
}

/** Descarga e instala. Se `refreshMenu`, ao reiniciar a versión nova actualiza as entradas do menú do Explorador. */
export async function installUpdate(refreshMenu: boolean): Promise<void> {
  if (!pending) return;
  updState.set("downloading");
  updProgress.set(0);
  try {
    let total = 0;
    let done = 0;
    await pending.downloadAndInstall((ev) => {
      if (ev.event === "Started") total = ev.data.contentLength ?? 0;
      else if (ev.event === "Progress") {
        done += ev.data.chunkLength;
        if (total) updProgress.set(Math.min(1, done / total));
      }
    });
    try {
      if (refreshMenu) localStorage.setItem(EX_KEY, "1");
      else localStorage.removeItem(EX_KEY);
    } catch {
      // ignorar
    }
    await relaunch();
  } catch (e) {
    updState.set("error");
    updError.set(String((e as any)?.message ?? e));
  }
}

/** Ao arrancar a versión nova: se se pediu, volve instalar as entradas do menú do Explorador co novo exe. */
export async function applyPendingMenuRefresh(install: () => Promise<void>): Promise<void> {
  let want = false;
  try {
    want = localStorage.getItem(EX_KEY) === "1";
    localStorage.removeItem(EX_KEY);
  } catch {
    // ignorar
  }
  if (!want) return;
  try {
    await install();
    toast(tr("O menú do clic dereito tamén se puxo ao día."));
  } catch {
    toast(tr("O menú do clic dereito resistiuse. Podes reintentalo en Axustes → Xeral."), { kind: "warn" });
  }
}

export const explorerInstalled = async (): Promise<boolean> => {
  try {
    return !!(await engine.explorerStatus()).installed;
  } catch {
    return false;
  }
};
