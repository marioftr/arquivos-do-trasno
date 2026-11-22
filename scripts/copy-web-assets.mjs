// Copia a web/public o que a versión web serve ela mesma (para que funcione sen conexión e sen CDN): o motor de OCR con tres idiomas
// e o ONNX Runtime que usan os modelos. Executase antes de compilar; os ficheiros non van a git (véñense en node_modules).
import { copyFileSync, mkdirSync, existsSync } from "node:fs";

const at = (p) => new URL(`../${p}`, import.meta.url);
const copies = [
  ["web/public/ocr", [
    ["node_modules/tesseract.js/dist/worker.min.js", "worker.min.js"],
    ["node_modules/tesseract.js-core/tesseract-core-simd-lstm.wasm.js", "tesseract-core-simd-lstm.wasm.js"],
    ["node_modules/tesseract.js-core/tesseract-core-lstm.wasm.js", "tesseract-core-lstm.wasm.js"],
    ...["eng", "spa", "glg", "cat"].map((l) => [`node_modules/@tesseract.js-data/${l}/4.0.0_best_int/${l}.traineddata.gz`, `${l}.traineddata.gz`]),
  ]],
  ["web/public/ort", [
    ["node_modules/onnxruntime-web/dist/ort-wasm-simd-threaded.mjs", "ort-wasm-simd-threaded.mjs"],
    ["node_modules/onnxruntime-web/dist/ort-wasm-simd-threaded.wasm", "ort-wasm-simd-threaded.wasm"],
  ]],
];
for (const [dir, files] of copies) {
  mkdirSync(at(dir), { recursive: true });
  for (const [src, name] of files) {
    if (!existsSync(at(src))) {
      console.error(`Falta ${src}: executa «npm install».`);
      process.exit(1);
    }
    copyFileSync(at(src), new URL(`${dir}/${name}`, at("")));
  }
}
console.log("Web: OCR e ONNX Runtime copiados a web/public");
