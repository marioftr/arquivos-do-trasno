import { save } from "@tauri-apps/plugin-dialog";
import { withExtension } from "../../formats";
import { tr } from "../../i18n";
import type { FileEntry } from "../../files";

export function samePath(a: string, b: string): boolean {
  return a.replace(/\\/g, "/").toLowerCase() === b.replace(/\\/g, "/").toLowerCase();
}

/** Pide onde gardar o resultado. Nunca devolve a ruta do orixinal. */
export async function askOutput(f: FileEntry, suffix: string, ext: string = f.ext): Promise<string | null> {
  const stem = f.path.replace(/\.[^.\\/]+$/, "");
  const out = await save({ defaultPath: withExtension(`${stem}${suffix}`, ext) });
  if (!out) return null;
  if (samePath(out, f.path)) {
    throw new Error(tr("Non vou sobrescribir o ficheiro orixinal. Escolle outro nome."));
  }
  return out;
}

/** «1,3,5-7» → [1,3,5,6,7]. Texto baleiro → null (= todas as páxinas). */
export function parsePageSpec(spec: string, total?: number): number[] | null {
  if (!spec.trim()) return null;
  const pages: number[] = [];
  for (const part of spec.split(",")) {
    const p = part.trim();
    if (!p) continue;
    const m = p.match(/^(\d+)-(\d+)$/);
    if (m) {
      const a = Number(m[1]);
      const b = Number(m[2]);
      for (let n = a; a <= b ? n <= b : n >= b; n += a <= b ? 1 : -1) pages.push(n);
    } else if (/^\d+$/.test(p)) pages.push(Number(p));
    else throw new Error(tr("«{p}» non é unha páxina válida", { p }));
  }
  const bad = total ? pages.find((n) => n < 1 || n > total) : undefined;
  if (bad !== undefined) throw new Error(tr("A páxina {page} non existe: o PDF só ten {total}.", { page: bad, total: total! }));
  return pages;
}

/** Igual que `parsePageSpec`, pero sen lanzar erros (para vistas previas). */
export function safePages(spec: string, total?: number): number[] | null {
  try {
    return parsePageSpec(spec, total);
  } catch {
    return null;
  }
}
