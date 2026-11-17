// Lista todas as «i» da interface (hint, info, notas de grupo e hints.ts), das máis longas ás máis curtas, para revisalas xuntas.
// Uso: npm run list:info [-- N]   (N = cantas amosar; por defecto todas)
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.join(path.resolve(path.dirname(fileURLToPath(import.meta.url)), ".."), "src");
const files = [];
(function walk(d) {
  for (const f of fs.readdirSync(d, { withFileTypes: true })) {
    const p = path.join(d, f.name);
    if (f.isDirectory()) {
      if (f.name !== "locales") walk(p);
    } else if (/\.(svelte|ts)$/.test(f.name)) files.push(p);
  }
})(root);
const found = [];
const re = /(?:\bhint|\binfo|\btip)(?:=\{|: ?)\$?t?\(?\s*"((?:[^"\\]|\\.)*)"|<GroupNote[^>]*\btext=\{\$t\(\s*"((?:[^"\\]|\\.)*)"/g;
for (const f of files) {
  const s = fs.readFileSync(f, "utf8");
  for (const m of s.matchAll(re)) found.push([m[1] ?? m[2], path.relative(root, f)]);
}
for (const m of fs.readFileSync(path.join(root, "lib/hints.ts"), "utf8").matchAll(/^\s+"([^"]+)":\s+"((?:[^"\\]|\\.)*)"/gm)) found.push([m[2], `hints.ts · ${m[1]}`]);
found.sort((a, b) => b[0].length - a[0].length);
const n = Number(process.argv[2]) || found.length;
console.log(`${found.length} «i»\n`);
for (const [t, f] of found.slice(0, n)) console.log(`${String(t.length).padStart(3)}  ${f.replace(/^lib[\\/]components[\\/]/, "")}\n     ${t}`);
