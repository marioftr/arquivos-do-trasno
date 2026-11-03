// Estado dun traballo longo (converter, comprimir...): ocupado, texto do
// trasno, progreso e mensaxe final. Os erros levan unha frase do trasno e despois o erro real.
// Cada traballo queda rexistrado na lista de procesos.

import { writable, get, type Writable } from "svelte/store";
import { tr } from "./i18n";
import { errText, isCancelled, missingToolOf, outputHook } from "./errors";
import { requestTool } from "./toolinstall";
import { beginJob } from "./engine";
import { track, refreshProcs, type Tracker } from "./procs";
import { files, activePath } from "./files";
import { noteAction, liveProgress, trasnoSay } from "./agent";

export interface JobMessage {
  ok: boolean;
  text: string;
  /** Aviso sen valoración (por exemplo, cancelado): non se pinta nin de verde nin de vermello. */
  neutral?: boolean;
}

export interface Job {
  busy: Writable<boolean>;
  text: Writable<string>;
  /** Avance real (0 a 1) cando o motor o dá; `null` se non se sabe. */
  progress: Writable<number | null>;
  message: Writable<JobMessage | null>;
  run: (working: string, fail: string, job: () => Promise<string | void>) => Promise<void>;
  say: (ok: boolean, text: string) => void;
  clear: () => void;
}

let active: { tracker: Tracker; progress: Writable<number | null> } | null = null;

/** Chámase cando chega un aviso de progreso do motor: vai ao traballo da app que estea en marcha. */
export function reportProgress(f: number) {
  if (!active) return;
  active.progress.set(f);
  liveProgress.set(f);
  active.tracker.update({ progress: f });
}

export function createJob(): Job {
  const busy = writable(false);
  const text = writable("");
  const progress = writable<number | null>(null);
  const message = writable<JobMessage | null>(null);
  return {
    busy,
    text,
    progress,
    message,
    async run(working, fail, job) {
      busy.set(true);
      text.set(tr(working));
      progress.set(null);
      message.set(null);
      await beginJob().catch(() => {});
      const current = get(files).find((f) => f.path === get(activePath));
      const tracker = track({ title: current?.name ?? tr("Traballo"), subtitle: tr(working), source: "app" });
      active = { tracker, progress };
      setTimeout(refreshProcs, 250);
      noteAction("O trasno comezou o traballo de «{name}»", { name: current?.name ?? "" });
      const outs: string[] = [];
      outputHook.fn = (p) => {
        if (!outs.includes(p)) outs.push(p);
        tracker.update({ outputs: [...outs] });
      };
      try {
        const done = await job();
        if (done) message.set({ ok: true, text: done });
        noteAction("O trasno rematou o traballo de «{name}»", { name: current?.name ?? "" });
        tracker.finish("done", { detail: done ? String(done).slice(0, 200) : undefined });
      } catch (e) {
        // Cancelar non é un fallo: sen frase de «tropezou».
        if (isCancelled(e)) {
          trasnoSay(tr("Traballo cancelado. Deixei todo ordenado, sen ficheiros a medias."), "info", 8000);
          noteAction("O trasno cancelou o traballo de «{name}»", { name: current?.name ?? "" });
          tracker.finish("cancelled");
        } else {
          const missing = missingToolOf(e);
          if (missing) requestTool(missing, undefined, () => void this.run(working, fail, job));
          message.set({ ok: false, text: `${tr(fail)}\n${errText(e)}` });
          noteAction("O trasno tropezou co traballo de «{name}»", { name: current?.name ?? "" });
          tracker.finish("error", { detail: errText(e).slice(0, 200) });
        }
      } finally {
        active = null;
        outputHook.fn = null;
        progress.set(null);
        liveProgress.set(null);
        busy.set(false);
        setTimeout(refreshProcs, 250);
      }
    },
    say(ok, t) {
      message.set({ ok, text: t });
    },
    clear() {
      message.set(null);
    },
  };
}
