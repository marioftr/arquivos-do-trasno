// Avisos curtos (ficheiro non soportado, xa estaba na lista, desfacer...): fainos o trasno no seu globo.
// Os erros e avisos quedan tamén no historial da barra inferior, con botón de copiar.

import { trasnoSay, noteAction } from "./agent";

export function toast(text: string, opts: { kind?: "info" | "warn"; action?: { label: string; run: () => void }; ms?: number } = {}) {
  const kind = opts.kind ?? "info";
  trasnoSay(text, kind, opts.ms ?? (kind === "warn" ? 40000 : opts.action ? 18000 : 12000), opts.action, { copy: kind === "warn" ? text : undefined, notify: true });
  if (kind === "warn") noteAction(text, undefined, undefined, text);
}
