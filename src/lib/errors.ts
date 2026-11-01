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

export async function call<T>(cmd: string, args?: InvokeArgs, options?: InvokeOptions): Promise<T> {
  try {
    return await invoke<T>(cmd, args, options);
  } catch (e) {
    throw new Error(localizeError(e));
  }
}

/** Se o erro é só que o usuario cancelou o traballo (non é un fallo). */
export function isCancelled(e: unknown): boolean {
  const raw = typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
  return raw.includes("Cancelouse o traballo.");
}
