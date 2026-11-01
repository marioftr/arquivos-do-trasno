// Avisos curtos que desaparecen soos (ficheiro non soportado, xa estaba na lista, desfacer...).

import { writable } from "svelte/store";

export interface Toast {
  id: number;
  text: string;
  kind: "info" | "warn";
  action?: { label: string; run: () => void };
  ms: number;
}

export const toasts = writable<Toast[]>([]);
let next = 1;

export function toast(text: string, opts: { kind?: "info" | "warn"; action?: Toast["action"]; ms?: number } = {}) {
  const id = next++;
  const ms = opts.ms ?? (opts.action ? 8000 : 5000);
  toasts.update((l) => [...l.slice(-3), { id, text, kind: opts.kind ?? "info", action: opts.action, ms }]);
  setTimeout(() => dismiss(id), ms);
}

export const dismiss = (id: number) => toasts.update((l) => l.filter((t) => t.id !== id));
