// Clona la app original, la compila en modo "standalone" y deja el resultado en ./bundle
import { execSync } from "node:child_process";
import { cpSync, existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const REPO = "https://github.com/oozmakafa/stremio-account-addon-cloner.git";
const src = join(import.meta.dirname, "app-src");
const out = join(import.meta.dirname, "bundle");
const run = (cmd, cwd) => execSync(cmd, { cwd, stdio: "inherit" });

if (!existsSync(src)) run(`git clone --depth 1 ${REPO} "${src}"`);
else run("git pull --ff-only", src);

// Activar la salida standalone de Next.js
const cfgPath = join(src, "next.config.ts");
let cfg = readFileSync(cfgPath, "utf8");
if (!cfg.includes("standalone")) {
  cfg = cfg.replace("/* config options here */", 'output: "standalone",');
  writeFileSync(cfgPath, cfg);
}

run("npm ci", src);
run("npx next build", src);

rmSync(out, { recursive: true, force: true });
cpSync(join(src, ".next/standalone"), out, { recursive: true });
cpSync(join(src, ".next/static"), join(out, ".next/static"), { recursive: true });
cpSync(join(src, "public"), join(out, "public"), { recursive: true });
console.log("\nBundle listo en", out);
