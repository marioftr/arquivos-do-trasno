// Comproba as traducións: todo texto usado coa función t()/tr() (e os textos
// indirectos: frases do trasno, categorías, predefinidos, canles de textura...)
// ten que existir en locales/es.json e en.json cos mesmos marcadores {nome}.
// Avisa tamén de entradas sen usar. Uso: node scripts/check-i18n.mjs

import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const root = new URL("../src", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const load = (f) => JSON.parse(readFileSync(join(root, "lib/locales", f), "utf8"));
const dicts = { es: load("es.json"), en: load("en.json") };

function walk(dir) {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? (n === "locales" ? [] : walk(p)) : /\.(svelte|ts)$/.test(n) ? [p] : [];
  });
}

const used = new Map(); // chave -> onde
const add = (key, where) => key && !used.has(key) && used.set(key, where);
const str = String.raw`"((?:[^"\\\n]|\\.)*)"`;
const unesc = (s) => JSON.parse(`"${s}"`);

for (const file of walk(root)) {
  if (file.endsWith("i18n.ts")) continue;
  const src = readFileSync(file, "utf8");
  const rel = file.slice(root.length + 1);

  // t("...") / $t("...") / tr("...")
  for (const m of src.matchAll(new RegExp(String.raw`(?:\$t|\btr)\(\s*${str}`, "g"))) add(unesc(m[1]), rel);

  // Indirectos: valores de TRASNO e CATEGORY_LABELS en formats.ts
  if (rel.endsWith("formats.ts")) {
    for (const block of ["TRASNO", "CATEGORY_LABELS"]) {
      const b = src.match(new RegExp(`const ${block}\\b[^=]*=\\s*\\{([\\s\\S]*?)\\n\\};`));
      if (b) for (const m of b[1].matchAll(new RegExp(String.raw`:\s*${str}`, "g"))) add(unesc(m[1]), rel);
    }
    // Listas de frases do trasno (éxito e fallo)
    for (const block of ["TRASNO_LISTO", "TRASNO_FALLO"]) {
      const b = src.match(new RegExp(`const ${block}\\b[^=]*=\\s*[\\[{]([\\s\\S]*?)\\n[\\]}];`));
      if (b) for (const m of b[1].matchAll(new RegExp(str, "g"))) add(unesc(m[1]), rel);
    }
  }
  // Predefinidos de calidade (Tools): label e hint
  if (rel.endsWith("ConvertSection.svelte")) {
    for (const m of src.matchAll(new RegExp(String.raw`label:\s*${str}\s*,\s*value:\s*\d+\s*,\s*hint:\s*${str}`, "g"))) {
      add(unesc(m[1]), rel);
      add(unesc(m[2]), rel);
    }
  }
  // Etiquetas de textura (model3d) e canles do visor 3D
  if (rel.endsWith("model3d.ts")) for (const m of src.matchAll(new RegExp(String.raw`label:\s*${str}`, "g"))) add(unesc(m[1]), rel);
  if (rel.endsWith("ModelViewer.svelte")) for (const m of src.matchAll(new RegExp(String.raw`id:\s*"[a-z]+",\s*label:\s*${str}`, "g"))) add(unesc(m[1]), rel);
  // Etiquetas da información do ficheiro
  if (rel.endsWith("InfoSection.svelte")) {
    for (const m of src.matchAll(new RegExp(String.raw`(?:rows|extra)\.push\(\[\s*${str}`, "g"))) add(unesc(m[1]), rel);
    add("Ruta", rel);
  }
  // Pestanas das ferramentas
  if (rel.endsWith("Tools.svelte")) for (const m of src.matchAll(new RegExp(String.raw`tab\(\s*"[a-z]+"\s*,\s*${str}`, "g"))) add(unesc(m[1]), rel);
  // Niveis de compresión de vídeo (label e hint)
  if (rel.endsWith("CompressSection.svelte")) {
    for (const m of src.matchAll(new RegExp(String.raw`label:\s*${str}\s*,\s*crf:\s*\d+\s*,\s*hint:\s*${str}`, "g"))) {
      add(unesc(m[1]), rel);
      add(unesc(m[2]), rel);
    }
  }
  // Sufixos de nome de ficheiro da edición de audio e vídeo
  if (rel.endsWith("MediaEditSection.svelte")) for (const m of src.matchAll(new RegExp(String.raw`run\(\s*${str}`, "g"))) if (m[1].startsWith("-")) add(unesc(m[1]), rel);
}

// Textos que nacen en Rust: plantillas de erro, detalles e mensaxes dos comandos.
const rustFiles = [
  ...readdirSync(new URL("../crates/trasno-engine/src", import.meta.url)).filter((f) => f.endsWith(".rs")).map((f) => new URL(`../crates/trasno-engine/src/${f}`, import.meta.url)),
  new URL("../src-tauri/src/commands.rs", import.meta.url),
];
for (const url of rustFiles) {
  const src = readFileSync(url, "utf8");
  const where = url.pathname.split("/").slice(-1)[0];
  if (where === "error.rs") {
    for (const m of src.matchAll(new RegExp(String.raw`=>\s*\(\s*${str}`, "g"))) add(unesc(m[1]), where);
    for (const m of src.matchAll(new RegExp(String.raw`\n\s+${str},\n\s+json!`, "g"))) add(unesc(m[1]), where);
  }
  for (const m of src.matchAll(new RegExp(String.raw`EngineError::(?:IntegrityCheck|Pdf|Model3d)\(\s*${str}`, "g"))) add(unesc(m[1]), where);
  for (const m of src.matchAll(new RegExp(String.raw`\bbad\(\s*${str}`, "g"))) add(unesc(m[1]), where);
  for (const m of src.matchAll(new RegExp(String.raw`\bmsg\(\s*${str}`, "g"))) add(unesc(m[1]), where);
}

const placeholders = (s) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort().join(",");
let errors = 0;
const fail = (msg) => {
  errors++;
  console.error("✗ " + msg);
};

for (const [key, where] of used) {
  for (const [lang, dict] of Object.entries(dicts)) {
    if (!(key in dict)) {
      fail(`[${lang}] falta a tradución de «${key}» (${where})`);
      continue;
    }
    if (!dict[key].trim()) fail(`[${lang}] tradución baleira de «${key}»`);
    if (placeholders(key) !== placeholders(dict[key])) fail(`[${lang}] marcadores distintos en «${key}» → «${dict[key]}»`);
  }
}
for (const [lang, dict] of Object.entries(dicts)) {
  for (const key of Object.keys(dict)) if (!used.has(key)) console.warn(`! [${lang}] entrada sen usar: «${key}»`);
}

console.log(`${used.size} textos comprobados en ${Object.keys(dicts).length} idiomas.`);
if (errors) {
  console.error(`${errors} problema(s).`);
  process.exit(1);
}
console.log("Traducións completas.");
