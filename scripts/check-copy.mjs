// Regras de texto da interface (ver docs/texto.md): se alguén as rompe, falla.
// Uso: npm run check:copy
//   1. Sen recheo: frases que o usuario xa dá por sabidas.
//   2. O trasno fala en primeira persoa nas mensaxes, avisos e erros («Non puiden…», non «O trasno non puido…»).
//      O historial de accións (noteAction) e a páxina «Sobre o trasno» están en terceira por deseño.
//   3. «Subtítulos», nunca «lexendas».
//   4. Unha «i» vai ao grano: máximo 140 caracteres, e é impersoal (nunca en primeira persoa).
//   5. Nin «Personalizado» con texto, nin explicacións en hover (`title`) que repitan o botón.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const root = path.join(repo, "src");
const MAX_INFO = 140;
// Unha «i» é impersoal: nunca en primeira persoa («busco», «convirto», «só sei»…).
const FIRST_PERSON = /\b(busco|convirto|quito|axusto|comento|teño|preparo|parto|detecto|deixo|gardo|descargo|fago|vexo|estou|podo|atopo|intento|calculo|escribo|ignoro|elimino|engado|recoñezo|miro|só sei|non sei|uso (unha|a|o|un))\b/i;

const BANNED = [
  [/orixinal non se toca/i, "«o orixinal non se toca» sobreentendese: todo resultado é un ficheiro novo"],
  [/orixinal (queda intacto|queda como estaba|non cambia|non se modifica)/i, "«o orixinal non cambia» sobreentendese"],
  [/(son|audio|imaxe|vídeo|video) (non se toca|non se recodifica|cópiase sen tocar|queda igual)/i, "«o son non se toca» e semellantes sobreentendense"],
  [/recodifícase con calidade alta/i, "«recodifícase con calidade alta» é un detalle técnico que non axuda a quen usa a ferramenta"],
  [/crea un ficheiro novo/i, "«crea un ficheiro novo» sobreentendese"],
  [/m[áa]is informaci[óo]n/i, "liñas xenéricas de «Máis información»: a «i» vai ao carón do que explica"],
  [/recodif[íi]case con moita calidade/i, "recheo sobre a recodificación da imaxe"],
  [/Valores propios/, "«Personalizado» xa se entende sen texto de máis"],
  [/value: "custom", label: \$t\("Personalizado"\), hint/, "«Personalizado» non leva explicación"],
  [/title=\{\$t\(HINTS\[/, "as explicacións van nunha «i», non en hover"],
  [/lexenda/i, "en galego, «subtítulos» (nunca «lexendas»)"],
  [/\b[Oo] trasno (non|só|está|ten|vai|usa|garda|busca|precisa|descarg\w+|comprim\w+|convirt\w+|prepara|ajusta|quita|atopa|atopou|leu|ve|fai|di|pregunta|comenta|volve|rebuscou|deixou|rematou)\b/, "o trasno fala en primeira persoa («Non atopo…», «Deixei…»)"],
];

const files = [];
(function walk(d, ext) {
  if (!fs.existsSync(d)) return;
  for (const f of fs.readdirSync(d, { withFileTypes: true })) {
    const p = path.join(d, f.name);
    if (f.isDirectory()) {
      if (f.name !== "locales" && f.name !== "node_modules") walk(p, ext);
    } else if (ext.test(f.name) && !/(legal|changelog)[.]ts$/.test(f.name)) files.push(p);
  }
})(root, /[.](svelte|ts)$/);
for (const d of ["crates/trasno-engine/src", "crates/trasno-engine/tests", "src-tauri/src"]) {
  const dir = path.join(repo, d);
  if (fs.existsSync(dir)) for (const f of fs.readdirSync(dir)) if (f.endsWith(".rs")) files.push(path.join(dir, f));
}

// A terceira persoa é correcta no historial de accións, na descrición do trasno (Créditos) e nos consellos que o nomean.
const THIRD_OK = /noteAction\(|\bnote:|^\s*(\/\/|\*|\/\*)|faga o trasno|fai o trasno ao premelo|sobre o trasno|O trasno é un duende|Consellos do trasno|Comentario do trasno|Descargado polo trasno/i;
const ALLOW_FILE = /CreditsPanel[.]svelte$/;

let bad = 0;
const report = (f, i, why) => {
  console.log(`✗ ${path.relative(repo, f)}:${i + 1}  ${why}`);
  bad++;
};
for (const f of files) {
  const rs = f.endsWith(".rs");
  fs.readFileSync(f, "utf8")
    .split("\n")
    .forEach((l, i) => {
      const comment = /^\s*(\/\/|\*|\/\*)/.test(l);
      for (const [re, why] of BANNED) {
        if (!re.test(l)) continue;
        const voice = why.startsWith("o trasno fala");
        const lex = why.startsWith("en galego");
        if (voice && (THIRD_OK.test(l) || ALLOW_FILE.test(f))) continue;
        if (comment && !lex) continue; // os comentarios do código só se controlan polo vocabulario
        if (rs && !lex && !voice) continue;
        report(f, i, why);
      }
    });
}

// Unha «i» curta: hint/info/text das compoñentes e as entradas de hints.ts.
const INFO_RE = /(?:\bhint|\binfo|\btip)(?:=\{|: ?)\$?t?\(?\s*"((?:[^"\\]|\\.)*)"|<GroupNote[^>]*\btext=\{\$t\(\s*"((?:[^"\\]|\\.)*)"/g;
for (const f of files.filter((x) => !x.endsWith(".rs"))) {
  const s = fs.readFileSync(f, "utf8");
  const lines = s.split("\n");
  for (const m of s.matchAll(INFO_RE)) {
    const text = m[1] ?? m[2];
    const at = s.slice(0, m.index).split("\n").length - 1;
    if (text.length > MAX_INFO) report(f, at, `«i» de ${text.length} caracteres (máximo ${MAX_INFO}): «${text.slice(0, 50)}…»`);
    if (FIRST_PERSON.test(text)) report(f, at, `«i» en primeira persoa: «${text.slice(0, 50)}…»`);
  }
  if (f.endsWith("hints.ts")) {
    lines.forEach((l, i) => {
      const m = l.match(/^\s+"[^"]+":\s+"((?:[^"\\]|\\.)*)"/);
      if (m && m[1].length > MAX_INFO) report(f, i, `«i» de ${m[1].length} caracteres (máximo ${MAX_INFO}): «${m[1].slice(0, 50)}…»`);
      if (m && FIRST_PERSON.test(m[1])) report(f, i, `«i» en primeira persoa: «${m[1].slice(0, 50)}…»`);
    });
  }
}
console.log(bad ? `${bad} problema(s) de texto.` : "Textos en regra.");
process.exit(bad ? 1 : 0);
