// Procesos (traballos) en marcha e historial. O rexistro vive en disco para que a aplicación principal
// vexa tamén os traballos das xanelas rápidas do clic dereito, que son outros procesos.

import { writable } from "svelte/store";
import { call } from "./errors";

export type ProcState = "running" | "done" | "error" | "cancelled" | "interrupted";

export interface Proc {
  id: string;
  title: string;
  subtitle?: string;
  /** «quick»: xanela rápida do clic dereito; «app»: ferramentas da aplicación principal. */
  source: "quick" | "app";
  state: ProcState;
  /** De 0 a 1. */
  progress: number;
  started: number;
  ended?: number;
  outputs?: string[];
  detail?: string;
  updated?: number;
}

export const procs = writable<Proc[]>([]);

export const refreshProcs = () =>
  call<Proc[]>("proc_list")
    .then((l) => procs.set(l))
    .catch(() => {});
export const requestCancel = (id: string) => call<void>("proc_request_cancel", { id });
export const cancelRequested = (id: string) => call<boolean>("proc_cancel_requested", { id });
export const clearFinished = () => call<void>("proc_clear_finished").then(refreshProcs);

const newId = () => `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

export interface Tracker {
  id: string;
  update: (patch: Partial<Proc>) => void;
  finish: (state: Exclude<ProcState, "running">, patch?: Partial<Proc>) => void;
}

/** Empeza a rexistrar un traballo: garda o seu estado en disco e latexa mentres siga en marcha. */
export function track(init: { title: string; subtitle?: string; source: "quick" | "app" }): Tracker {
  const rec: Proc = { id: newId(), state: "running", progress: 0, started: Date.now(), outputs: [], ...init };
  let last = 0;
  let pending: ReturnType<typeof setTimeout> | null = null;
  const save = () => {
    last = Date.now();
    call("proc_upsert", { rec }).catch(() => {});
  };
  const heartbeat = setInterval(save, 5000);
  save();
  return {
    id: rec.id,
    update(patch) {
      Object.assign(rec, patch);
      // Sen inundar o disco: como máximo unha escritura cada 300 ms.
      if (Date.now() - last > 300) save();
      else if (!pending) pending = setTimeout(() => ((pending = null), save()), 300);
    },
    finish(state, patch) {
      clearInterval(heartbeat);
      if (pending) clearTimeout(pending);
      Object.assign(rec, patch, { state, ended: Date.now(), progress: state === "done" ? 1 : rec.progress });
      save();
    },
  };
}
