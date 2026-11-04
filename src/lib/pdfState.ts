// Estado ligeiro dos PDF (sen cargar pdf.js): onde estaba cada un e quen o esquece ao quitalo da lista.
// Vive aparte de `pdfCache` para que o programa non cargue o lector de PDF ata que se abra un.

export const viewState = new Map<string, { page: number; scale: number; fit: "none" | "width" | "page" }>();

/** Funcións que limpan o que cada módulo pesado gardou dun PDF (rexístranse ao cargarse). */
const forgetHooks: ((path: string) => void)[] = [];
export const onForgetPdf = (f: (path: string) => void) => void forgetHooks.push(f);
export function forgetPdf(path: string) {
  viewState.delete(path);
  for (const f of forgetHooks) f(path);
}
