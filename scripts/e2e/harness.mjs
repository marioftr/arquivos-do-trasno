// Banco de probas da app real: lanza o executable de depuración cun perfil illado (nada do usuario se toca), conéctase por CDP
// (WebView2) e ofrece axudas para facer o que faría unha persoa: soltar ficheiros, premer pestanas, mover desprazadores, ler o
// que se ve e comprobar o resultado. Cada escenario (scenarios/*.mjs) usa isto e falla con código ≠ 0 se algo non é como debe.
//
// Uso: cargo build -p trasno && node scripts/e2e/run.mjs [escenario...]
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync, execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
export const repo = path.resolve(here, "..", "..");
const PORT = 9222;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
export { sleep };

async function httpOk(url) {
  try {
    return (await fetch(url)).ok;
  } catch {
    return false;
  }
}

/** FFmpeg: o do PATH ou o que usa a app. */
export function ffmpegBin() {
  for (const c of ["ffmpeg", "C:\\ffmpeg\\bin\\ffmpeg.exe"]) {
    if (spawnSync(c, ["-version"]).status === 0) return c;
  }
  throw new Error("Non atopo FFmpeg para crear os ficheiros de proba.");
}
export function ffprobeDuration(file) {
  const bin = ffmpegBin().replace(/ffmpeg(\.exe)?$/, "ffprobe$1");
  const out = execFileSync(bin, ["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", file]).toString().trim();
  return Number(out);
}

export class Check {
  constructor(name) {
    this.name = name;
    this.fails = [];
    this.passes = 0;
  }
  ok(cond, what, detail = "") {
    if (cond) this.passes++;
    else this.fails.push(`${what}${detail ? " → " + detail : ""}`);
    console.log(`  ${cond ? "✓" : "✗"} ${what}${!cond && detail ? " → " + detail : ""}`);
    return !!cond;
  }
  near(value, expected, tol, what) {
    return this.ok(Math.abs(value - expected) <= tol, what, `${value} (agardado ${expected} ±${tol})`);
  }
}

/** Arranca a app (e vite se fai falta) e devolve o obxecto `app` coas axudas. */
export async function launch({ tessdata = false } = {}) {
  const exeSrc = path.join(repo, "target", "debug", "trasno.exe");
  if (!fs.existsSync(exeSrc)) throw new Error("Falta target/debug/trasno.exe: executa antes «cargo build -p trasno».");
  const root = path.join(os.tmpdir(), "trasno-e2e");
  fs.rmSync(root, { recursive: true, force: true });
  for (const d of ["tmp", "lad", "wv", "media"]) fs.mkdirSync(path.join(root, d), { recursive: true });
  const exe = path.join(root, "trasno-e2e.exe");
  fs.copyFileSync(exeSrc, exe);
  if (tessdata) {
    const src = path.join(process.env.LOCALAPPDATA ?? "", "ArquivosDoTrasno", "tools", "tessdata");
    if (fs.existsSync(src)) fs.cpSync(src, path.join(root, "lad", "ArquivosDoTrasno", "tools", "tessdata"), { recursive: true });
  }

  // A app de depuración carga a interface do servidor de vite.
  let vite = null;
  if (!(await httpOk("http://localhost:1420"))) {
    vite = spawn(process.platform === "win32" ? "npx.cmd" : "npx", ["vite", "--port", "1420", "--strictPort"], { cwd: repo, stdio: "ignore", shell: process.platform === "win32" });
    for (let i = 0; i < 40 && !(await httpOk("http://localhost:1420")); i++) await sleep(500);
  }

  const child = spawn(exe, [], {
    cwd: root,
    env: {
      ...process.env,
      TEMP: path.join(root, "tmp"),
      TMP: path.join(root, "tmp"),
      LOCALAPPDATA: path.join(root, "lad"),
      WEBVIEW2_USER_DATA_FOLDER: path.join(root, "wv"),
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT} --remote-allow-origins=*`,
    },
    stdio: "ignore",
  });
  let page = null;
  for (let i = 0; i < 60 && !page; i++) {
    await sleep(500);
    try {
      page = (await (await fetch(`http://127.0.0.1:${PORT}/json`)).json()).find((t) => t.type === "page");
    } catch {
      // aínda non arrancou
    }
  }
  if (!page) throw new Error("A app non abriu a tempo.");
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let id = 0;
  const pend = new Map();
  ws.onmessage = (m) => {
    const d = JSON.parse(m.data);
    if (d.id && pend.has(d.id)) (pend.get(d.id)(d), pend.delete(d.id));
  };
  const send = (method, params = {}) => new Promise((r) => { const i = ++id; pend.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
  const ev = async (expr, gesture = false) => {
    const r = await send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true, userGesture: gesture });
    if (r.result?.exceptionDetails) throw new Error("JS: " + (r.result.exceptionDetails.exception?.description ?? "erro"));
    return r.result?.result?.value;
  };

  // A ventá de proba pode non ter o foco do sistema (outro programa diante): finxe que o ten para que os eventos de foco cheguen.
  await send("Emulation.setFocusEmulationEnabled", { enabled: true });

  const app = {
    root,
    media: path.join(root, "media"),
    ev,
    sleep,
    /** Nivel (RMS, 0 a 1) do son que sae agora do visor, medido no grafo de efectos; só en builds de desenvolvemento. */
    async level(ms = 700) {
      return ev(`(async()=>{const f=window.__trasnoFx; if(!f||!f.probe) return -1; const b=new Float32Array(f.probe.fftSize); let acc=0,n=0; const t0=performance.now(); while(performance.now()-t0<${ms}){f.probe.getFloatTimeDomainData(b); for(const x of b){acc+=x*x;n++} await new Promise(r=>setTimeout(r,40))} return Math.sqrt(acc/n)})()`);
    },
    /** Espera ata que `fn` (executada na páxina, devolve valor verdadeiro) se cumpra. */
    async waitFor(js, what, ms = 20000) {
      const t0 = Date.now();
      while (Date.now() - t0 < ms) {
        const v = await ev(js);
        if (v) return v;
        await sleep(250);
      }
      throw new Error(`Esperando: ${what} (pasaron ${ms} ms)`);
    },
    async shot(name) {
      const r = await send("Page.captureScreenshot", { format: "png" });
      const p = path.join(root, name);
      fs.writeFileSync(p, Buffer.from(r.result.data, "base64"));
      return p;
    },
    /** Preme un botón/pestana polo seu texto, título ou aria-label. */
    async click(text) {
      const r = await ev(`(()=>{const t=${JSON.stringify(text)};const b=[...document.querySelectorAll('button,[role=tab],[role=radio]')].find(b=>b.textContent.trim()===t||b.title===t||b.getAttribute('aria-label')===t); if(!b) return 'sen: '+t; if(b.disabled) return 'desactivado: '+t; b.click(); return 'ok';})()`);
      if (r !== "ok") throw new Error(`Non se puido premer «${text}»: ${r}`);
      await sleep(350);
    },
    /** Acende ou apaga un interruptor polo seu texto. */
    async toggle(text) {
      const r = await ev(`(()=>{const l=[...document.querySelectorAll('.toggle')].find(x=>x.querySelector('.lab')?.textContent.trim()===${JSON.stringify(text)}); if(!l) return 'sen interruptor: '+${JSON.stringify(text)}; l.querySelector('input').click(); return 'ok';})()`);
      if (r !== 'ok') throw new Error(r);
      await sleep(250);
    },
    /** Mete un ficheiro como se se soltase sobre a ventá. */
    async open(file) {
      await ev(`window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {event:'tauri://drag-drop', payload:{type:'drop', paths:[${JSON.stringify(file)}], position:{x:300,y:300}}})`);
      await sleep(3000);
    },
    async slider(label, value) {
      const r = await ev(`(()=>{const i=[...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')===${JSON.stringify(label)}); if(!i) return 'sen slider '+${JSON.stringify(label)}; Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i,String(${value})); i.dispatchEvent(new Event('input',{bubbles:true})); i.dispatchEvent(new Event('change',{bubbles:true})); return 'ok';})()`);
      if (r !== "ok") throw new Error(r);
      await sleep(200);
    },
    /** Texto do panel de ferramentas, liñas separadas por « | ». */
    panel: () => ev(`(document.querySelector('.stack')?.innerText||'').split(String.fromCharCode(10)).filter(Boolean).join(' | ')`),
    tabs: () => ev(`[...document.querySelectorAll('[role=tab]')].map(b=>b.title)`),
    /** Imaxe de proba co texto dado (debuxada no propio navegador: non fai falta ningunha ferramenta máis). */
    async textImage(name, lines) {
      const b64 = await ev(`(()=>{const c=document.createElement('canvas');c.width=900;c.height=60+${lines.length}*80;const g=c.getContext('2d');g.fillStyle='#fff';g.fillRect(0,0,c.width,c.height);g.fillStyle='#000';g.font='44px Arial';${JSON.stringify(lines)}.forEach((t,i)=>g.fillText(t,30,80+i*80));return c.toDataURL('image/png').split(',')[1];})()`);
      const p = path.join(root, "media", name);
      fs.writeFileSync(p, Buffer.from(b64, "base64"));
      return p;
    },
    /** Vídeo de proba (testsrc + tono); `rate` en fotogramas por segundo (baixo para vídeos longos). */
    video(name, seconds, { rate = 25, size = "320x240" } = {}) {
      const p = path.join(root, "media", name);
      const r = spawnSync(ffmpegBin(), ["-v", "error", "-y", "-f", "lavfi", "-i", `testsrc=s=${size}:r=${rate}:d=${seconds}`, "-f", "lavfi", "-i", `sine=f=440:r=44100:d=${seconds}`, "-c:v", "libx264", "-preset", "ultrafast", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "48k", p]);
      if (r.status !== 0) throw new Error("FFmpeg non puido crear o vídeo de proba: " + r.stderr);
      return p;
    },
    async reload() {
      await ev("localStorage.setItem('arquivos-do-trasno.output-mode','beside'); location.reload()");
      await sleep(4500);
    },
    async close() {
      try {
        ws.close();
      } catch {
        // xa pechado
      }
      if (process.platform === "win32") spawnSync("taskkill", ["/F", "/T", "/PID", String(child.pid)]);
      else child.kill();
      if (vite) spawnSync("taskkill", ["/F", "/T", "/PID", String(vite.pid)]);
      await sleep(500);
    },
  };
  // Espera a que a ventá cargue a interface de verdade (ao principio é unha páxina baleira).
  await app.waitFor(`location.href.startsWith('http') && document.readyState === 'complete' && !!document.querySelector('#app, body > *')`, "a interface", 40000);
  await app.reload();
  return app;
}
