// Executa os escenarios da app real (ver harness.mjs). Sen argumentos, todos; ou por nome: node scripts/e2e/run.mjs editar-video
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { launch, Check } from "./harness.mjs";

const dir = path.join(path.dirname(fileURLToPath(import.meta.url)), "scenarios");
const all = fs.readdirSync(dir).filter((f) => f.endsWith(".mjs")).map((f) => f.replace(/\.mjs$/, ""));
const wanted = process.argv.slice(2);
const names = wanted.length ? wanted : all;
for (const n of names) if (!all.includes(n)) (console.error(`Non hai o escenario «${n}». Os que hai: ${all.join(", ")}`), process.exit(2));

const mods = await Promise.all(names.map(async (n) => ({ n, mod: (await import(pathToFileURL(path.join(dir, n + ".mjs")).href)).default })));
const app = await launch({ tessdata: mods.some((m) => m.mod.tessdata) });
let failed = 0;
try {
  for (const { n, mod } of mods) {
    console.log(`\n▶ ${n}: ${mod.about}`);
    const check = new Check(n);
    try {
      await app.reload();
      await mod.run(app, check);
    } catch (e) {
      check.ok(false, "o escenario rematou sen erro", String(e.message ?? e));
      try {
        console.log("  captura:", await app.shot(`erro-${n}.png`));
      } catch {
        // sen captura
      }
    }
    failed += check.fails.length;
  }
} finally {
  await app.close();
}
console.log(failed ? `\n✗ ${failed} comprobación(s) fallaron.` : "\n✓ Todo en orde.");
process.exit(failed ? 1 : 0);
