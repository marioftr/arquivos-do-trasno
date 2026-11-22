import { defineConfig, type Plugin } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";
import fs from "node:fs";
import path from "node:path";

const at = (p: string) => fileURLToPath(new URL(p, import.meta.url));

/** As cabeceiras de seguridade de Vercel (vercel.json), tamén na vista previa local: así a proba ve o mesmo que o visitante. */
const headers = Object.fromEntries(
  (JSON.parse(fs.readFileSync(at("./vercel.json"), "utf8")).headers[0].headers as { key: string; value: string }[]).map((h) => [h.key, h.value]),
);

/** Textos que usa a versión web (`t("…")`, `$t("…")`, `tr("…")`): o resto do dicionario do programa non vai no paquete. */
function webKeys(): Set<string> {
  const keys = new Set<string>();
  const walk = (dir: string): string[] =>
    fs.readdirSync(dir, { withFileTypes: true }).flatMap((d) => (d.isDirectory() ? walk(path.join(dir, d.name)) : /\.(svelte|ts)$/.test(d.name) ? [path.join(dir, d.name)] : []));
  for (const f of [...walk(at("./web/src")), at("./src/lib/mt.ts")]) {
    const src = fs.readFileSync(f, "utf8");
    for (const m of src.matchAll(/(?:\$t|\btr)\(\s*"((?:[^"\\\n]|\\.)*)"/g)) keys.add(JSON.parse(`"${m[1]}"`));
  }
  return keys;
}

/** Os dicionarios de castelán e inglés levan só os textos da web (o do programa de escritorio ten máis de mil). */
function webLocales(): Plugin {
  const keys = webKeys();
  return {
    name: "web-locales",
    enforce: "pre",
    resolveId(source) {
      const m = /locales\/(es|en)\.json$/.exec(source);
      return m ? `\0web-locales:${m[1]}` : null;
    },
    load(id) {
      const m = /^\0web-locales:(es|en)$/.exec(id);
      if (!m) return null;
      const all = JSON.parse(fs.readFileSync(at(`./src/lib/locales/${m[1]}.json`), "utf8")) as Record<string, string>;
      const out: Record<string, string> = {};
      for (const k of keys) if (k in all) out[k] = all[k];
      return `export default ${JSON.stringify(out)};`;
    },
  };
}

/** Versión web (PWA): carpeta `web/`, comparte co programa de escritorio os textos, as iconas e as cores. */
export default defineConfig({
  root: at("./web"),
  publicDir: at("./web/public"),
  // Cache propia: o servidor do programa de escritorio usa node_modules/.vite e os dous non se pisan.
  cacheDir: at("./node_modules/.vite-web"),
  plugins: [webLocales(), svelte({ configFile: at("./svelte.config.js") })],
  resolve: { alias: { "@app": at("./src") } },
  // ffmpeg.wasm carga o seu propio traballador: non se pre-empaqueta.
  optimizeDeps: { exclude: ["@ffmpeg/ffmpeg", "@ffmpeg/util"] },
  server: { port: 1421, strictPort: true },
  preview: { headers },
  build: { outDir: at("./dist-web"), emptyOutDir: true, target: ["es2022", "safari15"] },
});
