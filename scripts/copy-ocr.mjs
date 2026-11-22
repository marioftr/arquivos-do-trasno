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

// O ONNX Runtime (WebAssembly) que usa a tradución de subtítulos: serve o a propia app, sen CDN.
const ort = new URL("../public/ort/", import.meta.url);
mkdirSync(ort, { recursive: true });
for (const name of ["ort-wasm-simd-threaded.mjs", "ort-wasm-simd-threaded.wasm"]) {
  const s = new URL(`../node_modules/onnxruntime-web/dist/${name}`, import.meta.url);
  if (!existsSync(s)) {
    console.error(`Falta ${name}: executa «npm install».`);
    process.exit(1);
  }
  copyFileSync(s, new URL(name, ort));
}
console.log("ONNX Runtime: ficheiros copiados a public/ort");
