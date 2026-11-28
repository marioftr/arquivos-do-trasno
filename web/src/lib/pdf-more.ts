/** Máis PDF no navegador con pdf-lib: ordenar, recortar, marca de auga, numerar, información e limpeza. */
import { tr } from "@app/lib/i18n";
import { stemOf, UserError, formatBytes, type Output } from "./files";
import { parseRanges } from "./pdf";

const lib = () => import("pdf-lib");

async function load(file: File, opts: { updateMetadata?: boolean } = {}) {
  const { PDFDocument } = await lib();
  try {
    return await PDFDocument.load(await file.arrayBuffer(), opts);
  } catch (e) {
    if (/encrypt/i.test(String((e as Error)?.message ?? e))) throw new UserError(tr("«{name}» ten contrasinal e non se pode abrir.", { name: file.name }));
    throw new UserError(tr("«{name}» non parece un PDF válido.", { name: file.name }));
  }
}
const toBlob = async (doc: { save: () => Promise<Uint8Array> }) => new Blob([(await doc.save()) as BlobPart], { type: "application/pdf" });
const pagesText = (n: number) => (n === 1 ? tr("1 páxina") : tr("{n} páxinas", { n }));
const expand = (ranges: [number, number][]) => ranges.flatMap(([a, b]) => Array.from({ length: b - a + 1 }, (_, k) => a + k));

/** Quita páxinas, dálle a volta á orde ou escribe unha orde nova (pódense repetir). */
export async function reorder(file: File, mode: "remove" | "reverse" | "order", spec: string): Promise<Output> {
  const { PDFDocument } = await lib();
  const src = await load(file);
  const n = src.getPageCount();
  let keep: number[];
  if (mode === "reverse") keep = src.getPageIndices().reverse();
  else if (mode === "order") keep = expand(parseRanges(spec, n));
  else {
    const drop = new Set(expand(parseRanges(spec, n)));
    keep = src.getPageIndices().filter((i) => !drop.has(i));
    if (!keep.length) throw new UserError(tr("Non pode quedar un PDF sen páxinas."));
  }
  const doc = await PDFDocument.create();
  for (const p of await doc.copyPages(src, keep)) doc.addPage(p);
  return { name: `${stemOf(file.name)}-ordenado.pdf`, blob: await toBlob(doc), note: pagesText(keep.length) };
}

/** Recorta `mm` milímetros de cada lado en todas as páxinas (o contido non se borra: a páxina só amosa menos). */
export async function cropMargins(file: File, mm: number): Promise<Output> {
  const doc = await load(file);
  const m = (mm * 72) / 25.4;
  for (const p of doc.getPages()) {
    const { x, y, width, height } = p.getCropBox();
    if (width - 2 * m < 36 || height - 2 * m < 36) throw new UserError(tr("A marxe é maior que a páxina. Elixe unha máis pequena."));
    p.setCropBox(x + m, y + m, width - 2 * m, height - 2 * m);
  }
  return { name: `${stemOf(file.name)}-recortado.pdf`, blob: await toBlob(doc) };
}

/** Un texto en diagonal sobre cada páxina. */
export async function watermark(file: File, text: string, opacity: number): Promise<Output> {
  const { StandardFonts, rgb, degrees } = await lib();
  const clean = text.trim();
  if (!clean) throw new UserError(tr("Escribe o texto da marca."));
  const doc = await load(file);
  const font = await doc.embedFont(StandardFonts.HelveticaBold);
  try {
    font.widthOfTextAtSize(clean, 10);
  } catch {
    throw new UserError(tr("A marca ten letras que o PDF non admite. Usa letras latinas."));
  }
  const angle = Math.PI / 4;
  for (const p of doc.getPages()) {
    const { width: pw, height: ph } = p.getSize();
    const size = Math.min(160, ((Math.hypot(pw, ph) * 0.7) / font.widthOfTextAtSize(clean, 100)) * 100);
    const w = font.widthOfTextAtSize(clean, size);
    const dx = w / 2;
    const dy = size * 0.35;
    p.drawText(clean, {
      x: pw / 2 - (dx * Math.cos(angle) - dy * Math.sin(angle)),
      y: ph / 2 - (dx * Math.sin(angle) + dy * Math.cos(angle)),
      size,
      font,
      color: rgb(0.45, 0.45, 0.45),
      opacity,
      rotate: degrees(45),
    });
  }
  return { name: `${stemOf(file.name)}-marca.pdf`, blob: await toBlob(doc) };
}

/** O número de páxina ao pé, á esquerda, no centro ou á dereita. */
export async function numberPages(file: File, side: "left" | "center" | "right", withTotal: boolean): Promise<Output> {
  const { StandardFonts, rgb } = await lib();
  const doc = await load(file);
  const font = await doc.embedFont(StandardFonts.Helvetica);
  const pages = doc.getPages();
  pages.forEach((p, i) => {
    const { width } = p.getSize();
    const label = withTotal ? `${i + 1} / ${pages.length}` : String(i + 1);
    const w = font.widthOfTextAtSize(label, 10);
    const x = side === "left" ? 36 : side === "right" ? width - 36 - w : (width - w) / 2;
    p.drawText(label, { x, y: 22, size: 10, font, color: rgb(0.25, 0.25, 0.25) });
  });
  return { name: `${stemOf(file.name)}-numerado.pdf`, blob: await toBlob(doc), note: pagesText(pages.length) };
}

/** Quita os metadatos e/ou aplana o formulario. */
export async function clean(file: File, meta: boolean, flat: boolean): Promise<Output> {
  const { PDFName } = await lib();
  const doc = await load(file, { updateMetadata: false });
  if (flat) {
    try {
      doc.getForm().flatten();
    } catch {
      throw new UserError(tr("Non se puido aplanar o formulario deste PDF."));
    }
  }
  if (meta) {
    const info = (doc as unknown as { getInfoDict(): { keys(): never[]; delete(k: never): boolean } }).getInfoDict();
    for (const k of info.keys()) info.delete(k);
    doc.catalog.delete(PDFName.of("Metadata"));
  }
  return { name: `${stemOf(file.name)}-limpo.pdf`, blob: await toBlob(doc) };
}

const dateText = (d: Date | undefined) => (d && !Number.isNaN(d.getTime()) ? d.toLocaleString() : "");

/** O que leva o PDF: páxinas, tamaño, autor, datas. */
export async function info(file: File): Promise<Output> {
  const doc = await load(file, { updateMetadata: false });
  const first = doc.getPage(0).getSize();
  const mm = (pt: number) => Math.round((pt * 25.4) / 72);
  const rows: [string, string][] = [
    [tr("Nome"), file.name],
    [tr("Tamaño do ficheiro"), formatBytes(file.size)],
    [tr("Páxinas"), String(doc.getPageCount())],
    [tr("Tamaño da páxina"), `${mm(first.width)} × ${mm(first.height)} mm`],
    [tr("Título"), doc.getTitle() ?? ""],
    [tr("Autor"), doc.getAuthor() ?? ""],
    [tr("Asunto"), doc.getSubject() ?? ""],
    [tr("Palabras clave"), doc.getKeywords() ?? ""],
    [tr("Creado con"), doc.getCreator() ?? ""],
    [tr("Producido con"), doc.getProducer() ?? ""],
    [tr("Data de creación"), dateText(doc.getCreationDate())],
    [tr("Última modificación"), dateText(doc.getModificationDate())],
    [tr("Formulario"), doc.getForm().getFields().length ? tr("Si") : tr("Non")],
  ];
  const text = rows.filter(([, v]) => v).map(([k, v]) => `${k}: ${v}`).join("\n");
  return { name: `${stemOf(file.name)}-info.txt`, blob: new Blob([text], { type: "text/plain;charset=utf-8" }), text };
}
