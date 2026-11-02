// Copia a `public/ocr` o necesario para o recoñecemento de texto (Tesseract en WebAssembly), para que vaia dentro da app.
// Executase antes de cada compilación; os ficheiros non se gardan en git (vense en node_modules).

import { copyFileSync, mkdirSync, existsSync } from "node:fs";

const out = new URL("../public/ocr/", import.meta.url);
mkdirSync(out, { recursive: true });
const from = [
  ["../node_modules/tesseract.js/dist/worker.min.js", "worker.min.js"],
  ["../node_modules/tesseract.js-core/tesseract-core-simd-lstm.wasm.js", "tesseract-core-simd-lstm.wasm.js"],
];
for (const [src, name] of from) {
  const s = new URL(src, import.meta.url);
  if (!existsSync(s)) {
    console.error(`Falta ${src}: executa «npm install».`);
    process.exit(1);
  }
  copyFileSync(s, new URL(name, out));
}
console.log("OCR: ficheiros copiados a public/ocr");
