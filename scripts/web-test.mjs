// Proba da versión web con Edge sen ventá (CDP): constrúe, serve dist-web e fai o que faría unha persoa nun móbil (390 px) e nun
// ordenador (1280 px): abrir ferramentas, elixir ficheiros, converter, xuntar PDF, comprobar que non hai desprazamento lateral, que
// os controis miden polo menos 44 px e que o texto chega ao contraste AA nos dous temas.
// Uso: npm run test:web
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const EDGE = ["C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe", "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe", "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"].find((p) => fs.existsSync(p));
if (!EDGE) throw new Error("Non atopo Edge nin Chrome.");

const build = spawnSync(process.platform === "win32" ? "npm.cmd" : "npm", ["run", "build:web"], { cwd: repo, stdio: "ignore", shell: process.platform === "win32" });
if (build.status !== 0) throw new Error("Non compila a versión web.");
const preview = spawn(process.platform === "win32" ? "npx.cmd" : "npx", ["vite", "preview", "-c", "vite.web.config.ts", "--port", "1422", "--strictPort"], { cwd: repo, stdio: "ignore", shell: process.platform === "win32" });
for (let i = 0; i < 40; i++) {
  try {
    if ((await fetch("http://localhost:1422/")).ok) break;
  } catch {
    await sleep(250);
  }
}
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "trasno-web-"));
const edge = spawn(EDGE, ["--headless=new", "--remote-debugging-port=9333", `--user-data-dir=${profile}`, "--no-first-run", "--disable-gpu", "about:blank"], { stdio: "ignore" });
let page = null;
for (let i = 0; i < 60 && !page; i++) {
  await sleep(300);
  try {
    page = (await (await fetch("http://127.0.0.1:9333/json")).json()).find((t) => t.type === "page");
  } catch {
    // aínda non arrancou
  }
}
if (!page) throw new Error("Edge non abriu.");
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0;
const pend = new Map();
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id && pend.has(d.id)) (pend.get(d.id)(d), pend.delete(d.id));
};
const send = (method, params = {}) => new Promise((r) => { const i = ++id; pend.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = async (expr) => {
  const r = await send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails) throw new Error("JS: " + (r.result.exceptionDetails.exception?.description ?? "erro"));
  return r.result?.result?.value;
};
const view = (w, h, mobile) => send("Emulation.setDeviceMetricsOverride", { width: w, height: h, deviceScaleFactor: mobile ? 2 : 1, mobile });
const go = async (hash) => (await ev(`location.hash=${JSON.stringify(hash)}`), await sleep(500));

const TOOLS = ["converter", "redimensionar", "comprimir", "mellorar", "xirar", "ocr", "quitar-fondo", "unir-pdf", "dividir-pdf", "xirar-pdf", "imaxes-a-pdf", "pdf-imaxes", "pdf-texto", "comprimir-pdf", "audio", "cortar", "gif", "transcribir", "traducir"];
const NET = !!process.env.TRASNO_WEB_NET; // as probas que descargan motores e modelos (uns cantos centos de MB) só se piden
let fails = 0;
const ok = (cond, what, detail = "") => {
  if (!cond) fails++;
  console.log(`  ${cond ? "✓" : "✗"} ${what}${!cond && detail ? " → " + detail : ""}`);
};

// ---- axudas dentro da páxina
const HELP = `window.__t = {
  sleep: (ms) => new Promise((r) => setTimeout(r, ms)),
  png: (w, h, c, name) => new Promise((r) => { const k = document.createElement('canvas'); k.width = w; k.height = h; const g = k.getContext('2d'); g.fillStyle = '#2a6fd0'; g.fillRect(0, 0, w, h); g.fillStyle = c; g.beginPath(); g.arc(w / 2, h / 2, h / 3, 0, 7); g.fill(); k.toBlob((b) => r(new File([b], name, { type: 'image/png' })), 'image/png'); }),
  feed: async (files) => { const dt = new DataTransfer(); files.forEach((f) => dt.items.add(f)); const i = document.querySelector('input[type=file]'); i.files = dt.files; i.dispatchEvent(new Event('change', { bubbles: true })); await window.__t.sleep(600); },
  go: async () => { document.querySelector('[data-testid=go]').click(); await window.__t.sleep(2200); },
  results: () => [...document.querySelectorAll('.results li')].map((l) => l.innerText.replace(/\\s+/g, ' ')),
};`;
const MEASURE = fs.readFileSync(path.join(repo, "scripts", "e2e", "scenarios", "contraste.mjs"), "utf8").match(/const MEASURE = `([\s\S]*?)`;/)[1].replace(/\\\\/g, "\\");

try {
  await send("Page.enable");
  await send("Page.navigate", { url: "http://localhost:1422/" });
  await sleep(1500);
  await ev(HELP);

  for (const [label, w, h, mobile] of [["móbil 390 px", 390, 844, true], ["móbil estreito 320 px", 320, 640, true], ["ordenador 1280 px", 1280, 800, false]]) {
    console.log(`\n▶ ${label}`);
    await view(w, h, mobile);
    await go("");
    await ev("localStorage.clear(); document.documentElement.lang='gl'");
    await sleep(200);
    const cards = await ev(`document.querySelectorAll('a.card').length`);
    ok(cards === 19, "o inicio ten as 19 ferramentas", String(cards));
    for (const hash of ["", ...TOOLS.map((t) => `#/${t}`)]) {
      await go(hash);
      const m = await ev(`({ over: document.documentElement.scrollWidth - innerWidth, small: [...document.querySelectorAll('button:not(:disabled):not(.sr-only), a.card, select, input[type=text]')].filter((e) => { const r = e.getBoundingClientRect(); return r.width > 0 && (r.height < 43.5 || r.width < 43.5) }).map((e) => (e.getAttribute('aria-label') || e.textContent).trim().slice(0, 24)) })`);
      ok(m.over <= 0, `${hash || "inicio"}: sen desprazamento lateral`, `${m.over}px`);
      ok(m.small.length === 0, `${hash || "inicio"}: controis de 44 px ou máis`, m.small.join(", "));
    }
  }

  console.log("\n▶ converter imaxes (móbil)");
  await view(390, 844, true);
  await go("#/converter");
  await ev(`(async () => { await __t.feed([await __t.png(1200, 800, '#d01818', 'foto.png'), await __t.png(800, 1000, '#e8c020', 'retrato.png')]); })()`);
  ok((await ev(`document.querySelectorAll('.files .list li').length`)) === 2, "elixe dúas imaxes");
  await sleep(600);
  ok((await ev(`[...document.querySelectorAll('.files .thumb')].filter((i) => i.tagName === 'IMG' && i.naturalWidth > 0).length`)) === 2, "cada unha ten a súa miniatura");
  ok((await ev(`document.querySelector('[data-testid=go]').innerText.trim()`)) === "Converter 2 imaxes", "o botón di cantas converte");
  await ev(`__t.go()`);
  const res = await ev(`__t.results()`);
  ok(res.length === 2 && res[0].startsWith("foto.jpg") && res[1].startsWith("retrato.jpg"), "saen dous JPG cos nomes novos", res.join(" | "));
  ok(/→/.test(res[0] ?? ""), "cada resultado di canto pesaba e canto pesa agora", res[0]);

  console.log("\n▶ imaxes a PDF, dividir e unir");
  await go("#/imaxes-a-pdf");
  await ev(`(async () => { await __t.feed([await __t.png(1200, 800, '#d01818', 'a.png'), await __t.png(800, 1000, '#e8c020', 'b.png'), await __t.png(900, 900, '#20c040', 'c.png')]); })()`);
  ok((await ev(`document.querySelectorAll('.files .list li').length`)) === 3, "tres imaxes na lista");
  await ev(`document.querySelector('.files .list li:last-child .ib:nth-last-child(3)')?.click()`);
  await sleep(150);
  ok((await ev(`document.querySelector('.files .list li:nth-child(2) .name').textContent`)) === "c.png", "subir unha imaxe cambia a orde");
  await ev(`__t.go()`);
  const pdf = await ev(`__t.results()`);
  ok(pdf.length === 1 && /3 páxinas/.test(pdf[0]), "sae un PDF de 3 páxinas", pdf.join(" | "));
  await ev(`(async () => { window.__grab = []; HTMLAnchorElement.prototype.click = function () { if (this.download) window.__grab.push(this.href); }; document.querySelector('.results .save').click(); await __t.sleep(200); window.__pdf = new File([await (await fetch(window.__grab[0])).blob()], 'tres.pdf', { type: 'application/pdf' }); })()`);
  ok((await ev(`window.__pdf.size`)) > 1000, "o botón Gardar entrega o ficheiro");
  await go("#/dividir-pdf");
  await ev(`(async () => { await __t.feed([window.__pdf]); const t = document.querySelector('input[type=text]'); t.value = '2-3'; t.dispatchEvent(new Event('input', { bubbles: true })); await __t.go(); })()`);
  const sp = await ev(`__t.results()`);
  ok(sp.length === 1 && /2 páxinas/.test(sp[0]), "dividir 2-3 deixa un PDF de 2 páxinas", sp.join(" | "));
  await ev(`(async () => { const t = document.querySelector('input[type=text]'); t.value = '9'; t.dispatchEvent(new Event('input', { bubbles: true })); await __t.go(); })()`);
  ok(/sae do documento/.test(await ev(`document.querySelector('.results .warn')?.innerText ?? ''`)), "un intervalo fóra do documento di o motivo");
  await go("#/pdf-imaxes");
  await ev(`(async () => { await __t.feed([window.__pdf]); await __t.go(); })()`);
  const pi = await ev(`__t.results()`);
  ok(pi.length === 3 && pi[0].startsWith("tres-p1.jpg"), "PDF a imaxes deixa unha por páxina", pi.join(" | "));
  await go("#/comprimir-pdf");
  await ev(`(async () => { await __t.feed([window.__pdf]); await __t.go(); })()`);
  ok((await ev(`__t.results()`)).length === 1, "comprimir PDF devolve un ficheiro (menor ou igual)");
  await go("#/unir-pdf");
  await ev(`(async () => { await __t.feed([window.__pdf, window.__pdf]); await __t.go(); })()`);
  ok(/6 páxinas/.test((await ev(`__t.results()`)).join()), "unir dous PDF de 3 páxinas dá 6");

  console.log("\n▶ mellorar imaxes e PDF a texto");
  await go("#/mellorar");
  await ev(`(async () => { await __t.feed([await __t.png(600, 400, '#808080', 'escura.png')]); await __t.go(); })()`);
  const me = await ev(`__t.results()`);
  ok(me.length === 1 && me[0].startsWith("escura-mellorado.png"), "mellorar (automático) dá un ficheiro novo", me.join(" | "));
  for (const mode of ["light", "doc", "bw", "denoise", "sharp"]) {
    await ev(`(async () => { document.querySelector('input[name=mellorar-mode][value=${mode}]').click(); await __t.sleep(100); await __t.go(); })()`);
    const r = await ev(`__t.results()`);
    ok(r.length === 1 && /×/.test(r[0]), `mellorar: ${mode}`, r.join(" | "));
  }
  await go("#/pdf-texto");
  await ev(`(async () => { await __t.feed([window.__pdf]); await __t.go(); })()`);
  ok(/Non atopei texto/.test(await ev(`document.querySelector('.results .warn')?.innerText ?? ''`)), "PDF a texto avisa se o PDF só ten imaxes");

  console.log("\n▶ modelos descargados");
  await ev(`(async () => { for (const n of await caches.keys()) if (!n.startsWith('trasno-web')) await caches.delete(n); })()`);
  await go("#/quitar-fondo");
  await sleep(400);
  ok(/Descárgase a primeira vez/.test(await ev(`document.querySelector('.model')?.innerText ?? ''`)), "sen modelo, avisa de que se descarga");
  await ev(`(async () => { const c = await caches.open('trasno-models'); await c.put('https://huggingface.co/BritishWerewolf/U-2-Netp/resolve/main/onnx/model.onnx', new Response('x')); })()`);
  await go("");
  await go("#/quitar-fondo");
  await sleep(500);
  ok(/Xa descargado/.test(await ev(`document.querySelector('.model.done')?.innerText ?? ''`)), "co modelo gardado, di que xa está descargado");
  ok(/Descargado/.test(await ev(`document.querySelector('.seg label.on')?.innerText ?? ''`)), "e a opción elixida o amosa");
  await ev(`(async () => { await caches.delete('trasno-models'); })()`);

  console.log("\n▶ recoñecer texto (OCR, sen rede)");
  await go("#/ocr");
  await ev(`(async () => { const f = await new Promise((r) => { const k = document.createElement('canvas'); k.width = 1000; k.height = 300; const g = k.getContext('2d'); g.fillStyle = '#fff'; g.fillRect(0, 0, 1000, 300); g.fillStyle = '#000'; g.font = '56px Arial'; g.fillText('Ola mundo, isto é unha proba', 40, 120); g.fillText('de recoñecemento de texto.', 40, 220); k.toBlob((b) => r(new File([b], 'proba.png', { type: 'image/png' }))); }); await __t.feed([f]); document.querySelector('[data-testid=go]').click(); })()`);
  await ev(`(async () => { for (let i = 0; i < 90 && !document.querySelector('.results textarea') && !document.querySelector('.results .warn'); i++) await __t.sleep(1000); })()`);
  const ocr = await ev(`document.querySelector('.results textarea')?.value ?? document.querySelector('.results .warn')?.innerText ?? ''`);
  ok(/Ola mundo/.test(ocr) && /proba/.test(ocr), "le o texto en galego", ocr);
  ok((await ev(`document.querySelector('.results .sub')?.innerText`)).includes("palabras"), "di cantas palabras leu");

  if (NET) {
    console.log("\n▶ audio e tradución (con rede: motores e modelos de internet)");
    await go("#/audio");
    await ev(`(async () => { const sr = 16000, n = sr * 2, buf = new ArrayBuffer(44 + n * 2), dv = new DataView(buf); const w = (o, s) => [...s].forEach((c, i) => dv.setUint8(o + i, c.charCodeAt(0))); w(0, 'RIFF'); dv.setUint32(4, 36 + n * 2, true); w(8, 'WAVEfmt '); dv.setUint32(16, 16, true); dv.setUint16(20, 1, true); dv.setUint16(22, 1, true); dv.setUint32(24, sr, true); dv.setUint32(28, sr * 2, true); dv.setUint16(32, 2, true); dv.setUint16(34, 16, true); w(36, 'data'); dv.setUint32(40, n * 2, true); for (let i = 0; i < n; i++) dv.setInt16(44 + i * 2, Math.sin(i * 2 * Math.PI * 440 / sr) * 12000, true); await __t.feed([new File([buf], 'tono.wav', { type: 'audio/wav' })]); document.querySelector('[data-testid=go]').click(); for (let i = 0; i < 120 && !document.querySelector('.results li') && !document.querySelector('.results .warn'); i++) await __t.sleep(1000); })()`);
    const au = await ev(`__t.results()`);
    ok(au.length === 1 && au[0].startsWith("tono.mp3"), "converte un WAV a MP3 co motor de FFmpeg", au.join(" | "));
    await go("#/traducir");
    await ev(`(async () => { await __t.feed([new File(['1\\n00:00:01,000 --> 00:00:03,000\\nGood morning, how are you today?\\n'], 'a.srt', { type: 'application/x-subrip' })]); document.querySelector('input[name=traducir-to][value=spa]').click(); document.querySelector('[data-testid=go]').click(); for (let i = 0; i < 240 && !document.querySelector('.results textarea') && !document.querySelector('.results .warn'); i++) await __t.sleep(1000); })()`);
    const tr = await ev(`document.querySelector('.results textarea')?.value ?? document.querySelector('.results .warn')?.innerText ?? ''`);
    ok(/buenos d[ií]as/i.test(tr), "traduce subtítulos do inglés ao castelán", tr);
  }

  console.log("\n▶ contraste e idioma");
  for (const [hash, label] of [["", "inicio"], ["#/converter", "converter"], ["#/dividir-pdf", "dividir"]]) {
    await go(hash);
    for (const theme of ["dark", "light"]) {
      await ev(`document.documentElement.dataset.theme = ${JSON.stringify(theme)}`);
      await sleep(150);
      const bad = await ev(MEASURE);
      ok(bad.length === 0, `${label} (${theme === "dark" ? "escuro" : "claro"}): contraste AA`, bad.slice(0, 3).map((b) => `«${b.text}» ${b.ratio}:1`).join(" | "));
    }
  }
  await go("");
  await ev(`localStorage.setItem('arquivos-do-trasno.lang', 'en'); location.reload()`);
  await sleep(1500);
  ok(/Convert images/.test(await ev(`document.body.innerText`)), "o inglés tradúcese");
} finally {
  try {
    ws.close();
  } catch {
    // xa pechado
  }
  if (process.platform === "win32") {
    spawnSync("taskkill", ["/F", "/T", "/PID", String(edge.pid)]);
    spawnSync("taskkill", ["/F", "/T", "/PID", String(preview.pid)]);
  } else {
    edge.kill();
    preview.kill();
  }
  await sleep(500);
  fs.rmSync(profile, { recursive: true, force: true });
}
console.log(fails ? `\n✗ ${fails} comprobación(s) fallaron.` : "\n✓ Todo en orde.");
process.exit(fails ? 1 : 0);
