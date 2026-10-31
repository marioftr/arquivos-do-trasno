// Estado dun traballo longo (converter, comprimir...): ocupado, texto do
// trasno e mensaxe final. Os erros levan unha frase do trasno e despois o erro real.

import { writable, type Writable } from "svelte/store";
import { tr } from "./i18n";
import { errText } from "./errors";

export interface JobMessage {
  ok: boolean;
  text: string;
}

export interface Job {
  busy: Writable<boolean>;
  text: Writable<string>;
  message: Writable<JobMessage | null>;
  run: (working: string, fail: string, job: () => Promise<string | void>) => Promise<void>;
  say: (ok: boolean, text: string) => void;
  clear: () => void;
}

export function createJob(): Job {
  const busy = writable(false);
  const text = writable("");
  const message = writable<JobMessage | null>(null);
  return {
    busy,
    text,
    message,
    async run(working, fail, job) {
      busy.set(true);
      text.set(tr(working));
      message.set(null);
      try {
        const done = await job();
        if (done) message.set({ ok: true, text: done });
      } catch (e) {
        message.set({ ok: false, text: `${tr(fail)}\n${errText(e)}` });
      } finally {
        busy.set(false);
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
