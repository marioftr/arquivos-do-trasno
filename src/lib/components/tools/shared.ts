import { save } from "@tauri-apps/plugin-dialog";
import { withExtension } from "../../formats";
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
    throw new Error("non se sobrescribe o ficheiro orixinal; escolle outro nome");
  }
  return out;
}
