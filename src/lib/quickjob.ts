// Traballos rápidos do clic dereito: o que se fai con cada ficheiro (convertir, comprimir, cambiar tamaño, xirar).
import * as engine from "./engine";
import { withExtension, sameFormat, cannotSaveNotice, noCompressionNotice, animatedNotice } from "./formats";
import { tr } from "./i18n";

const extOf = (p: string) => (p.match(/\.([^.\\/]+)$/)?.[1] ?? "").toLowerCase();

const pctLess = (before: number, after: number) => (before > 0 ? Math.round((1 - after / before) * 100) : 0);

/** Comproba que unha ferramenta que garda no formato do orixinal pode tratar esta imaxe sen perder nada (formato que só se sabe ler,
 *  sen compresión propia ou animación que se perdería). Se non, falla co motivo en vez de gardar algo distinto do esperado. */
async function guardImage(path: string, ext: string, tool: "editar" | "comprimir") {
  if (!sameFormat(ext)) throw new Error(cannotSaveNotice(ext, tool));
  if (tool === "comprimir" && !engine.COMPRESS_KEEP.includes(ext)) throw new Error(noCompressionNotice(ext));
  if (ext !== "gif" || tool === "editar") {
    if (["gif", "webp", "png", "apng"].includes(ext) && (await engine.imageIsAnimated(path))) throw new Error(animatedNotice(ext, tool));
  }
}

/** Fai o traballo cun ficheiro e devolve a ruta de saída e, se vén ao caso, unha nota (p. ex. «62 % menos»). */
export async function runOne(job: engine.StartupJob, path: string): Promise<{ out: string; note?: string }> {
  const it = { path };
  const ext = extOf(it.path);
  const cat = await engine.detectCategory(it.path);
  const stem = it.path.replace(/\.[^.\\/]+$/, "");
  const arg = job.target ?? "";

  if (job.action === "convert") {
    const out = await engine.uniquePath(withExtension(it.path, arg));
    if (cat === "imaxe" && arg === "pdf") await engine.imagesToPdf([it.path], out, false);
    else await engine.convertFile(it.path, out, { overwrite: false });
    return { out };
  }

  if (job.action === "compress") {
    // O nivel é a calidade que se quere manter: 75, 50 ou 25 %.
    const level = Math.min(100, Math.max(10, Number(arg) || 75));
    const before = await engine.fileSize(it.path);
    let out: string;
    if (cat === "imaxe") {
      await guardImage(it.path, ext, "comprimir");
      out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, ext));
      await engine.compressImage(it.path, out, { quality: level }, false);
    } else if (cat === "video") {
      out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, "mp4"));
      const crf = level >= 75 ? 24 : level >= 50 ? 28 : 34;
      await engine.compressVideo(it.path, out, { crf, audioKbps: level >= 75 ? 160 : 128 }, false);
    } else {
      out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, "pdf"));
      const [q, sc] = level >= 75 ? [70, 75] : level >= 50 ? [55, 60] : [40, 50];
      await engine.pdfCompress(it.path, out, q, sc, false);
    }
    const after = await engine.fileSize(out);
    const less = pctLess(before, after);
    return { out, note: less > 0 ? tr("{pct} % menos", { pct: less }) : tr("xa estaba moi comprimido") };
  }

  if (job.action === "resize") {
    const before = await engine.fileSize(it.path);
    if (cat === "imaxe") {
      await guardImage(it.path, ext, "editar");
      const p = Number(arg.slice(1)) || 50;
      const [w, h] = await engine.imageDimensions(it.path);
      const out = await engine.uniquePath(withExtension(`${stem}-${p}pc`, ext));
      await engine.editImage(it.path, out, { resize: [Math.max(1, Math.round((w * p) / 100)), Math.max(1, Math.round((h * p) / 100))] }, false);
      return { out, note: `${w}×${h} → ${Math.round((w * p) / 100)}×${Math.round((h * p) / 100)}` };
    }
    const hgt = Number(arg.slice(1)) || 720;
    const out = await engine.uniquePath(withExtension(`${stem}-${hgt}p`, ext));
    await engine.mediaEdit(it.path, out, "scale", hgt, true, false);
    const less = pctLess(before, await engine.fileSize(out));
    return { out, note: less > 0 ? tr("{pct} % menos", { pct: less }) : undefined };
  }

  // rotate
  const deg = Number(arg) || 90;
  const out = await engine.uniquePath(withExtension(`${stem}${tr("-xirado")}`, ext));
  if (cat === "imaxe") {
    await guardImage(it.path, ext, "editar");
    await engine.editImage(it.path, out, { rotate: deg }, false);
  } else await engine.pdfRotate(it.path, null, deg, out, false);
  return { out };
}

