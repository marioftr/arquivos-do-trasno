/** PDF a imaxes e compresión de PDF (pasando as páxinas por imaxe) con pdf.js; só se descarga ao usar estas ferramentas. */
import { tr } from "@app/lib/i18n";
import { stemOf, uniqueName, UserError, type Ctx, type Output } from "./files";

/** Máximo de píxeles por páxina (o canvas dun móbil non aguanta máis). */
const MAX_AREA = 12_000_000;
const MAX_PAGES = 150;

async function open(file: File) {
  const pdfjs = await import("pdfjs-dist");
  const worker = (await import("pdfjs-dist/build/pdf.worker.min.mjs?url")).default;
  pdfjs.GlobalWorkerOptions.workerSrc = worker;
  try {
    return await pdfjs.getDocument({ data: new Uint8Array(await file.arrayBuffer()) }).promise;
  } catch (e) {
    if ((e as { name?: string })?.name === "PasswordException") throw new UserError(tr("«{name}» ten contrasinal e non se pode abrir.", { name: file.name }));
    throw new UserError(tr("«{name}» non parece un PDF válido.", { name: file.name }));
  }
}

type Doc = Awaited<ReturnType<typeof open>>;

/** Debuxa unha páxina nun canvas a `ppp` (72 é o tamaño do PDF), sen pasar do límite de píxeles. */
async function paint(doc: Doc, n: number, ppp: number) {
  const page = await doc.getPage(n);
  const base = page.getViewport({ scale: 1 });
  let scale = ppp / 72;
  if (base.width * scale * base.height * scale > MAX_AREA) scale = Math.sqrt(MAX_AREA / (base.width * base.height));
  const vp = page.getViewport({ scale });
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.floor(vp.width));
  canvas.height = Math.max(1, Math.floor(vp.height));
  const g = canvas.getContext("2d")!;
  g.fillStyle = "#fff";
  g.fillRect(0, 0, canvas.width, canvas.height);
  await page.render({ canvasContext: g, viewport: vp, canvas } as never).promise;
  page.cleanup();
  return { canvas, w: base.width, h: base.height };
}

const toBlob = (c: HTMLCanvasElement, mime: string, q: number) => new Promise<Blob | null>((r) => c.toBlob(r, mime, q));

export async function pdfToImages(file: File, mime: "image/jpeg" | "image/png", ppp: number, ctx: Ctx): Promise<Output[]> {
  const doc = await open(file);
  const total = Math.min(doc.numPages, MAX_PAGES);
  const taken = new Set<string>();
  const ext = mime === "image/png" ? "png" : "jpg";
  const out: Output[] = [];
  for (let n = 1; n <= total; n++) {
    ctx.progress((n - 1) / total, tr("Páxina {n} de {total}…", { n, total }));
    await new Promise((r) => setTimeout(r));
    const { canvas } = await paint(doc, n, ppp);
    const blob = await toBlob(canvas, mime, 0.88);
    canvas.width = canvas.height = 0;
    if (!blob) throw new UserError(tr("Algo fallou ao procesar. Proba con outro ficheiro."));
    out.push({ name: uniqueName(`${stemOf(file.name)}-p${n}.${ext}`, taken), blob, note: `${tr("Páxina {n}", { n })}` });
  }
  if (doc.numPages > MAX_PAGES) out[out.length - 1].note += ` · ${tr("O PDF ten {n} páxinas: só se fixeron as primeiras {max}.", { n: doc.numPages, max: MAX_PAGES })}`;
  return out;
}

/** O texto seleccionable do PDF, páxina a páxina (cada unha co seu salto de liña). */
export async function pdfToText(file: File, ctx: Ctx): Promise<Output> {
  const doc = await open(file);
  const parts: string[] = [];
  for (let n = 1; n <= doc.numPages; n++) {
    ctx.progress((n - 1) / doc.numPages, tr("Páxina {n} de {total}…", { n, total: doc.numPages }));
    const page = await doc.getPage(n);
    const content = await page.getTextContent();
    let text = "";
    for (const it of content.items as { str: string; hasEOL?: boolean }[]) text += it.str + (it.hasEOL ? "\n" : "");
    page.cleanup();
    parts.push(text.replace(/[ \t]+\n/g, "\n").trim());
  }
  const text = parts.filter(Boolean).join("\n\n");
  if (!text) throw new UserError(tr("Non atopei texto neste PDF. Se é unha dixitalización, saca as imaxes e usa Recoñecer texto."));
  const words = text.split(/\s+/).filter(Boolean).length;
  return { name: `${stemOf(file.name)}.txt`, blob: new Blob([text], { type: "text/plain;charset=utf-8" }), text, note: tr("{n} palabras", { n: words }) };
}

/** Comprime un PDF volvendo gardar cada páxina como imaxe JPG: pesa moito menos (sobre todo dixitalizacións), pero perde o texto seleccionable. */
export async function compressPdf(file: File, ppp: number, quality: number, ctx: Ctx): Promise<Output> {
  const [{ PDFDocument }, doc] = await Promise.all([import("pdf-lib"), open(file)]);
  const total = Math.min(doc.numPages, MAX_PAGES);
  const out = await PDFDocument.create();
  for (let n = 1; n <= total; n++) {
    ctx.progress((n - 1) / total, tr("Páxina {n} de {total}…", { n, total }));
    await new Promise((r) => setTimeout(r));
    const { canvas, w, h } = await paint(doc, n, ppp);
    const blob = await toBlob(canvas, "image/jpeg", quality);
    canvas.width = canvas.height = 0;
    if (!blob) throw new UserError(tr("Algo fallou ao procesar. Proba con outro ficheiro."));
    const img = await out.embedJpg(new Uint8Array(await blob.arrayBuffer()));
    out.addPage([w, h]).drawImage(img, { x: 0, y: 0, width: w, height: h });
  }
  const bytes = await out.save();
  return { name: `${stemOf(file.name)}-comprimido.pdf`, blob: new Blob([bytes as BlobPart], { type: "application/pdf" }) };
}
