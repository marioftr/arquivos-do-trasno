// Avisos curtos (ficheiro non soportado, xa estaba na lista, desfacer...): agora fainos o trasno no seu globo.

import { trasnoSay } from "./agent";

export function toast(text: string, opts: { kind?: "info" | "warn"; action?: { label: string; run: () => void }; ms?: number } = {}) {
  trasnoSay(text, opts.kind ?? "info", opts.ms ?? (opts.action ? 10000 : 7000), opts.action);
}
