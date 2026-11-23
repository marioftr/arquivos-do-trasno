// Quita de `dist` / `dist-web` a variante de ONNX Runtime para WebGPU (27 MB) que Vite copia pero ninguén pide: a app e a web
// traballan na CPU (`device: "wasm"`; ver mt.ts). Uso: node scripts/prune-dist.mjs dist | dist-web
import fs from "node:fs";
import path from "node:path";

if (process.env.TRASNO_NO_PRUNE) process.exit(0);
const dir = path.resolve(process.argv[2] ?? "dist");
const assets = path.join(dir, "assets");
if (!fs.existsSync(path.join(dir, "ort", "ort-wasm-simd-threaded.wasm"))) {
  console.error(`prune-dist: falta ${path.join(dir, "ort", "ort-wasm-simd-threaded.wasm")}; non se quita nada.`);
  process.exit(0);
}
let saved = 0;
for (const f of fs.readdirSync(assets)) {
  if (/^ort-wasm.*asyncify.*\.(wasm|mjs)$/.test(f)) {
    saved += fs.statSync(path.join(assets, f)).size;
    fs.rmSync(path.join(assets, f));
  }
}
console.log(`prune-dist: ${(saved / 1048576).toFixed(1)} MB menos en ${path.basename(dir)}`);
