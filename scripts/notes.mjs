// Escribe as notas dunha versión a partir de src/lib/changelog.ts: node scripts/notes.mjs v0.1.3
// A publicación en GitHub usa isto como texto da release, e a app le a sección do seu idioma
// (marcadores <!-- gl -->, <!-- es --> e <!-- en -->) para amosar un resumo antes de actualizar.

import { readFileSync } from "node:fs";

const version = (process.argv[2] ?? "").replace(/^v/, "");
const src = readFileSync(new URL("../src/lib/changelog.ts", import.meta.url), "utf8");
const body = src.slice(src.indexOf("export const CHANGELOG"));
const list = new Function(body.replace(/^export const CHANGELOG[^=]*=/, "return "))();
const rel = list.find((r) => r.version === version);
if (!rel) {
  console.error(`Non hai entrada para ${version} en changelog.ts`);
  process.exit(1);
}
const names = { gl: "Galego", es: "Español", en: "English" };
const out = [];
for (const lang of ["gl", "es", "en"]) {
  out.push(`<!-- ${lang} -->`, `### ${names[lang]} · ${rel.title[lang]}`, "", ...rel.changes.map((c) => `- ${c[lang]}`), "");
}
console.log(out.join("\n"));
