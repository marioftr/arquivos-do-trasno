// Textos de recheo que non din nada (o usuario xa os dá por sabidos): se alguén os volve poñer, falla.
// Uso: npm run check:copy
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "src");
const BANNED = [
  [/orixinal non se toca/i, "«o orixinal non se toca» sobreentendese: todo resultado é un ficheiro novo"],
  [/orixinal queda (intacto|como estaba)/i, "«o orixinal queda intacto» sobreentendese"],
  [/crea un ficheiro novo/i, "«crea un ficheiro novo» sobreentendese"],
  [/m[áa]is informaci[óo]n/i, "liñas xenéricas de «Máis información»: a «i» vai ao carón do que explica"],
  [/recodif[íi]case con moita calidade/i, "recheo sobre a recodificación da imaxe"],
  [/Valores propios/, "«Personalizado» xa se entende sen texto de máis"],
];
const files = [];
(function walk(d) {
  for (const f of fs.readdirSync(d, { withFileTypes: true })) {
    const p = path.join(d, f.name);
    if (f.isDirectory()) {
      if (f.name !== "locales") walk(p);
    } else if (/[.](svelte|ts)$/.test(f.name) && !/(legal|changelog)[.]ts$/.test(f.name)) files.push(p);
  }
})(root);
// Os avisos de que unha vista previa é unha copia temporal viven dentro dunha «i» e si din algo.
const ALLOW = /(ImageViewer|MediaViewer)[.]svelte$/;
let bad = 0;
for (const f of files) {
  fs.readFileSync(f, "utf8")
    .split(String.fromCharCode(10))
    .forEach((l, i) => {
      for (const [re, why] of BANNED) {
        if (re.test(l) && !(ALLOW.test(f) && /orixinal/.test(l)) && !/^\s*(\/\/|\*|\/\*)/.test(l)) {
          console.log(`✗ ${path.relative(root, f)}:${i + 1}  ${why}`);
          bad++;
        }
      }
    });
}
console.log(bad ? `${bad} texto(s) de recheo.` : "Sen textos de recheo.");
process.exit(bad ? 1 : 0);
