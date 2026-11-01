// Prepara unha versión: node scripts/release.mjs 0.2.0
// Cambia o número nos tres sitios, comproba que o rexistro de cambios ten a entrada, fai o commit e crea a etiqueta.
// Non sube nada: ao final imprime os dous comandos para facelo cando queiras.

import { readFileSync, writeFileSync } from "node:fs";
import { execSync } from "node:child_process";

const v = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(v ?? "")) {
  console.error("Uso: npm run release -- 0.2.0");
  process.exit(1);
}
const sh = (c) => execSync(c, { stdio: "pipe" }).toString().trim();

if (sh("git status --porcelain")) {
  console.error("Hai cambios sen commit. Fai commit primeiro e volve probar.");
  process.exit(1);
}
const changelog = readFileSync("src/lib/changelog.ts", "utf8");
if (!changelog.includes(`version: "${v}"`)) {
  console.error(`Falta a entrada da versión ${v} en src/lib/changelog.ts. Engádea arriba do todo e volve probar.`);
  process.exit(1);
}
const edit = (file, from, to) => {
  const s = readFileSync(file, "utf8");
  if (!from.test(s)) throw new Error(`Non atopei a versión en ${file}`);
  writeFileSync(file, s.replace(from, to));
};
edit("package.json", /"version": "[^"]+"/, `"version": "${v}"`);
edit("src-tauri/tauri.conf.json", /"version": "[^"]+"/, `"version": "${v}"`);
edit("Cargo.toml", /(\[workspace\.package\]\s*\nversion = )"[^"]+"/, `$1"${v}"`);
sh("npm install --package-lock-only --ignore-scripts");
sh("git add -A");
sh(`git commit -m "Versión ${v}"`);
sh(`git tag v${v}`);
const branch = sh("git rev-parse --abbrev-ref HEAD");
console.log(`\nListo: versión ${v} preparada no teu ordenador.\nPara publicala, executa:\n\n  git push origin ${branch}\n  git push origin v${v}\n`);
