// Recoñecer texto nunha imaxe: pestana propia, lectura, texto sobre a imaxe e nada de botóns de OCR na barra do visor.
export default {
  about: "Pestana «Recoñecer texto (OCR)» das imaxes",
  tessdata: true,
  async run(app, c) {
    const img = await app.textImage("texto.png", ["O trasno le este texto", "Arquivos do Trasno 2026"]);
    await app.open(img);
    c.ok((await app.tabs()).includes("Recoñecer texto (OCR)"), "a imaxe ten a pestana de OCR");
    const bar = await app.ev(`[...document.querySelectorAll('.bar button')].map(b=>b.textContent.trim()).join('|')`);
    c.ok(!/Texto da imaxe|Lendo|Copiar todo/.test(bar), "a barra do visor xa non leva botóns de OCR", bar);
    c.ok(!(await app.ev(`!!document.querySelector('.ocr')`)), "sen abrir a pestana non hai texto sobre a imaxe");

    await app.click("Recoñecer texto (OCR)");
    await app.click("Ler texto da imaxe");
    await app.waitFor(`!!document.querySelector('.ocr')`, "o texto lido sobre a imaxe", 60000);
    const panel = await app.panel();
    const n = Number(/(\d+) palabras\./.exec(panel)?.[1] ?? 0);
    c.ok(n >= 8, "le polo menos 8 das 9 palabras", panel);
    c.ok(/Copiar todo o texto/.test(panel), "ofrece copiar todo o texto");

    await app.click("Información");
    c.ok(!(await app.ev(`!!document.querySelector('.ocr')`)), "ao saír da pestana o texto deixa de estar sobre a imaxe");
    await app.click("Recoñecer texto (OCR)");
    c.ok(await app.ev(`!!document.querySelector('.ocr')`), "ao volver, o texto xa lido volve sen ler de novo");
  },
};
