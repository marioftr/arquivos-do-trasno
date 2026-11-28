/** PDF no navegador con pdf-lib (descárgase só cando se abre unha ferramenta de PDF). */
import { tr } from "@app/lib/i18n";
import { render } from "./image";
import { stemOf, uniqueName, UserError, type Ctx, type Output } from "./files";

const lib = () => import("pdf-lib");
const pagesText = (n: number) => (n === 1 ? tr("1 páxina") : tr("{n} páxinas", { n }));

async function load(file: File) {
  const { PDFDocument } = await lib();
  try {
    return await PDFDocument.load(await file.arrayBuffer());
  } catch (e) {
    const msg = String((e as Error)?.message ?? e);
    if (/encrypt/i.test(msg)) throw new UserError(tr("«{name}» ten contrasinal e non se pode abrir.", { name: file.name }));
    throw new UserError(tr("«{name}» non parece un PDF válido.", { name: file.name }));
  }
}

const pdfBlob = async (doc: { save: () => Promise<Uint8Array> }) => new Blob([(await doc.save()) as BlobPart], { type: "application/pdf" });

export async function merge(files: File[], ctx: Ctx): Promise<Output[]> {
  const { PDFDocument } = await lib();
  const out = await PDFDocument.create();
  let pages = 0;
  for (const [i, f] of files.entries()) {
    ctx.progress(i / files.length);
    const src = await load(f);
    for (const p of await out.copyPages(src, src.getPageIndices())) {
      out.addPage(p);
      pages++;
    }
  }
  return [{ name: "unido.pdf", blob: await pdfBlob(out), note: pagesText(pages) }];
}

/** «1-3, 5, 8-» → intervalos de índices (desde 0). «-3» vai da primeira á 3; «8-» da 8 á última. */
export function parseRanges(text: string, pages: number): [number, number][] {
  const out: [number, number][] = [];
  for (const raw of text.split(/[,;\s]+/).filter(Boolean)) {
    const m = /^(\d*)-(\d*)$/.exec(raw) ?? /^(\d+)$/.exec(raw);
    if (!m) throw new UserError(tr("O intervalo «{r}» non é válido. Exemplo: 1-3, 5, 8-", { r: raw }));
    const a = m.length === 2 ? Number(m[1]) : m[1] ? Number(m[1]) : 1;
    const b = m.length === 2 ? Number(m[1]) : m[2] ? Number(m[2]) : pages;
    if (a < 1 || b < a || b > pages) throw new UserError(tr("O intervalo «{r}» sae do documento, que ten {n} páxinas.", { r: raw, n: pages }));
    out.push([a - 1, b - 1]);
  }
  if (!out.length) throw new UserError(tr("Escribe as páxinas que queres. Exemplo: 1-3, 5, 8-"));
  return out;
}

export type SplitMode = "extract" | "ranges" | "each";

export async function split(file: File, mode: SplitMode, ranges: string): Promise<Output[]> {
  const { PDFDocument } = await lib();
  const src = await load(file);
  const total = src.getPageCount();
  const groups: { label: string; pages: number[] }[] =
    mode === "each"
      ? src.getPageIndices().map((i) => ({ label: `p${i + 1}`, pages: [i] }))
      : mode === "ranges"
        ? parseRanges(ranges, total).map(([a, b]) => ({ label: a === b ? `p${a + 1}` : `p${a + 1}-${b + 1}`, pages: Array.from({ length: b - a + 1 }, (_, k) => a + k) }))
        : [{ label: "extracto", pages: parseRanges(ranges, total).flatMap(([a, b]) => Array.from({ length: b - a + 1 }, (_, k) => a + k)) }];
  const taken = new Set<string>();
  const outs: Output[] = [];
  for (const g of groups) {
    const doc = await PDFDocument.create();
    for (const p of await doc.copyPages(src, g.pages)) doc.addPage(p);
    outs.push({ name: uniqueName(`${stemOf(file.name)}-${g.label}.pdf`, taken), blob: await pdfBlob(doc), note: pagesText(g.pages.length) });
  }
  return outs;
}

export async function rotate(file: File, angle: number): Promise<Output> {
  const { degrees } = await lib();
  const doc = await load(file);
  for (const p of doc.getPages()) p.setRotation(degrees((((p.getRotation().angle + angle) % 360) + 360) % 360));
  return { name: `${stemOf(file.name)}-xirado.pdf`, blob: await pdfBlob(doc) };
}

const A4: [number, number] = [595.28, 841.89];

/** Unha imaxe por páxina. As fotos pasan polo canvas: así respectan a orientación da cámara e non pesan de máis. */
export async function imagesToPdf(files: File[], page: "image" | "a4", ctx: Ctx): Promise<Output[]> {
  const { PDFDocument } = await lib();
  const doc = await PDFDocument.create();
  for (const [i, f] of files.entries()) {
    ctx.progress(i / files.length);
    const png = f.type === "image/png" || f.type === "image/gif" || f.type === "image/webp";
    let r;
    try {
      r = await render(f, { mime: png ? "image/png" : "image/jpeg", quality: 0.9, maxSide: 2600 });
    } catch {
      throw new UserError(tr("Non se puido abrir «{name}».", { name: f.name }));
    }
    const bytes = new Uint8Array(await r.blob.arrayBuffer());
    const img = png ? await doc.embedPng(bytes) : await doc.embedJpg(bytes);
    if (page === "a4") {
      const [pw, ph] = r.width > r.height ? [A4[1], A4[0]] : A4;
      const margin = 24;
      const k = Math.min((pw - margin * 2) / r.width, (ph - margin * 2) / r.height);
      const w = r.width * k;
      const h = r.height * k;
      doc.addPage([pw, ph]).drawImage(img, { x: (pw - w) / 2, y: (ph - h) / 2, width: w, height: h });
    } else {
      const w = r.width * 0.5;
      const h = r.height * 0.5;
      doc.addPage([w, h]).drawImage(img, { x: 0, y: 0, width: w, height: h });
    }
  }
  return [{ name: `${stemOf(files[0]?.name ?? "imaxes")}.pdf`, blob: await pdfBlob(doc), note: pagesText(files.length) }];
}
