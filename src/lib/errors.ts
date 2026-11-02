// Os erros do motor chegan como `@@{"k": plantilla, "p": parámetros}` para
// poder traducilos aquí; o resto son frases fixas ou texto do sistema.

import { invoke, type InvokeArgs, type InvokeOptions } from "@tauri-apps/api/core";
import { tr } from "./i18n";

export function localizeError(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  if (!raw.startsWith("@@")) return tr(raw);
  try {
    const { k, p } = JSON.parse(raw.slice(2)) as { k: string; p: Record<string, unknown> };
    const params: Record<string, string | number> = {};
    for (const [key, v] of Object.entries(p ?? {})) params[key] = typeof v === "number" ? v : String(v);
    if (p?.dk && typeof p.detail === "string") params.detail = tr(p.detail);
    return tr(k, params);
  } catch {
    return raw;
  }
}

/** Texto de erro listo para amosar, sexa o que sexa o que se capturou. */
export const errText = localizeError;

/** Quen quere saber os ficheiros de saída que van creando os comandos (o rexistro de procesos). */
export const outputHook: { fn: ((path: string) => void) | null } = { fn: null };

export async function call<T>(cmd: string, args?: InvokeArgs, options?: InvokeOptions): Promise<T> {
  try {
    const r = await invoke<T>(cmd, args, options);
    const out = (args as { output?: unknown } | undefined)?.output;
    if (typeof out === "string" && out) {
      // O motor pode ter cambiado o nome (para non pisar nada): vale o camiño que devolve.
      const ext = out.slice(out.lastIndexOf(".")).toLowerCase();
      outputHook.fn?.(typeof r === "string" && /[\\/]/.test(r) && r.toLowerCase().endsWith(ext) ? r : out);
    } else if (cmd === "save_bytes" && typeof r === "string" && r) outputHook.fn?.(r);
    return r;
  } catch (e) {
    const err = new Error(localizeError(e)) as MissingToolError;
    const raw = e instanceof Error ? e.message : String(e);
    if (raw.includes("Non atopo ffmpeg")) err.missingTool = "ffmpeg";
    else if (raw.includes("Falta MKVToolNix")) err.missingTool = "mkvmerge";
    throw err;
  }
}

export type MissingToolError = Error & { missingTool?: "ffmpeg" | "mkvmerge" };
/** Se o erro é porque falta FFmpeg ou MKVToolNix, cal dos dous. */
export const missingToolOf = (e: unknown): "ffmpeg" | "mkvmerge" | null => (e as MissingToolError)?.missingTool ?? null;

/** Se o erro é só que o usuario cancelou o traballo (non é un fallo). */
export function isCancelled(e: unknown): boolean {
  const raw = typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
  return raw.includes("Cancelouse o traballo.");
}
