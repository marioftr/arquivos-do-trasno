// Transcribir, Quitar fondo e Descargar por ligazón con programas e modelos de verdade (voz sintética de Windows, unha imaxe debuxada e a
// ligazón de TRASNO_E2E_URL). Precisa a caché de extensións (TRASNO_E2E_TOOLS, ver harness.mjs `useTools`) e rede para a descarga.
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

/** Mínimo e máximo da canle alfa dunha imaxe (0 a 255), medidos con FFmpeg. */
function alphaRange(file) {
  const r = spawnSync(ffmpegBin(), ["-hide_banner", "-i", file, "-vf", "alphaextract,signalstats,metadata=print:file=-", "-f", "null", "-"], { encoding: "utf8" });
  const text = r.stdout + r.stderr;
  const num = (k) => Number(new RegExp(`lavfi.signalstats.${k}=([0-9.]+)`).exec(text)?.[1] ?? NaN);
  return { min: num("YMIN"), max: num("YMAX") };
}

export default {
  about: "Transcribir, Quitar fondo e Descargar por ligazón con modelos e programas de verdade",
  async run(app, c) {
    if (!app.useTools()) {
      c.ok(true, "sen caché de extensións (TRASNO_E2E_TOOLS): non se proba");
      return;
    }
    await app.reload();
    const gate = () => app.ev(`!!document.querySelector('[data-testid=extension-gate]')`);
    const green = (text) => app.ev(`[...document.querySelectorAll('.btn.primary.block')].find(b=>b.textContent.trim()===${JSON.stringify(text)})?.disabled`);
    const press = (text) => app.ev(`[...document.querySelectorAll('.btn.primary.block')].find(b=>b.textContent.trim()===${JSON.stringify(text)})?.click()`);
    const pickLevel = (name) => app.ev(`[...document.querySelectorAll('.stack [role=radio]')].find(b=>b.innerText.startsWith(${JSON.stringify(name)}))?.click()`);

    // ---- Transcribir
    const voz = app.speech("voz.wav", "Hello, this is a test of the transcription. The quick brown fox jumps over the lazy dog.");
    await app.open(voz);
    await app.click("Transcribir");
    await pickLevel("Rápida");
    await app.sleep(300);
    c.ok(!(await gate()), "co modelo descargado non sae o aviso");
    await app.select("Idioma da voz", "en");
    c.ok((await green("Transcribir")) === false, "sae o botón verde Transcribir");
    // Un modelo que non está descargado leva o aviso e quita o botón.
    await pickLevel("Equilibrada");
    await app.sleep(300);
    c.ok(await gate(), "un modelo sen descargar leva o aviso");
    c.ok((await green("Transcribir")) === undefined, "e non hai botón verde");
    await pickLevel("Rápida");
    await app.sleep(300);
    await press("Transcribir");
    const srt = path.join(app.media, "voz.srt");
    await app.waitFor(`document.body.innerText.includes('Transcritos')`, "a transcrición", 180000).catch(() => {});
    c.ok(fs.existsSync(srt), "gárdase voz.srt ao carón do audio");
    const text = fs.existsSync(srt) ? fs.readFileSync(srt, "utf8").toLowerCase() : "";
    c.ok(/-->/.test(text) && /fox/.test(text), "o SRT ten tempos e a frase dita", text.slice(0, 200));
    c.ok(!fs.readdirSync(app.media).some((n) => n.startsWith(".")), "non quedan temporais á beira");

    // ---- Quitar fondo: un cadrado vermello grande sobre un fondo de cores
    const draw = await app.ev(`(()=>{const c=document.createElement('canvas');c.width=640;c.height=480;const g=c.getContext('2d');const gr=g.createLinearGradient(0,0,640,480);gr.addColorStop(0,'#173a73');gr.addColorStop(1,'#0e8a6b');g.fillStyle=gr;g.fillRect(0,0,640,480);g.fillStyle='#f2c200';g.beginPath();g.ellipse(320,250,150,170,0,0,Math.PI*2);g.fill();g.fillStyle='#c4161c';g.beginPath();g.ellipse(320,250,95,110,0,0,Math.PI*2);g.fill();g.fillStyle='#fff';g.beginPath();g.arc(285,215,24,0,Math.PI*2);g.arc(355,215,24,0,Math.PI*2);g.fill();return c.toDataURL('image/png').split(',')[1]})()`);
    const pic = path.join(app.media, "obxecto.png");
    fs.writeFileSync(pic, Buffer.from(draw, "base64"));
    await app.open(pic);
    await app.click("Quitar fondo");
    c.ok(!(await gate()), "co modelo descargado non sae o aviso");
    // A vista previa non sae soa: hai un botón pequeno encima de Quitar fondo. Ao premelo, o modelo analiza a imaxe e o visor amosa o recorte
    // en directo (un PNG transparente por riba da imaxe).
    c.ok(!(await app.ev(`!!document.querySelector('img.cut')`)), "a vista previa non empeza soa");
    c.ok(await app.ev(`(()=>{const b=[...document.querySelectorAll('.stack .btn')]; const p=b.findIndex(x=>x.textContent.trim()==='Previsualizar'); const g=b.findIndex(x=>x.textContent.trim()==='Quitar fondo'); return p>=0 && p<g && !b[p].classList.contains('primary');})()`), "hai un botón Previsualizar pequeno encima do verde");
    c.ok(await app.ev(`[...document.querySelectorAll('.stack input[type=range]')].every(i=>!i.disabled)`), "os axustes están dispoñibles desde o principio");
    await app.ev(`[...document.querySelectorAll('.stack .btn')].find(x=>x.textContent.trim()==='Previsualizar')?.click()`);
    await app.waitFor(`!!document.querySelector('img.cut')`, "a vista previa do recorte", 120000);
    c.ok(true, "o visor amosa o recorte en directo");
    const cutSrc = () => app.ev(`document.querySelector('img.cut')?.getAttribute('src') ?? ''`);
    const sliders = await app.ev(`[...document.querySelectorAll('.stack .field .lab, .stack label')].map(l=>l.textContent.trim()).join('|')`);
    c.ok(/Recorte/.test(sliders) && /Suavidade/.test(sliders) && /Detalle/.test(sliders), "hai axustes de recorte, suavidade e detalle", sliders);
    const radios = await app.ev(`[...document.querySelectorAll('.stack [role=radiogroup] [role=radio]')].map(b=>b.textContent.trim()).join()`);
    c.ok(radios === "Transparente,Branco,Negro,Gris,Personalizado", "o selector de fondo ten os mesmos botóns grandes que Converter", radios);
    const viewBg = () => app.ev(`document.querySelector('.canvas .view')?.style.background ?? ''`);
    await app.ev(`[...document.querySelectorAll('.stack [role=radio]')].find(b=>b.textContent.trim()==='Branco')?.click()`);
    await app.sleep(300);
    c.ok(/255, 255, 255/.test(await viewBg()), "o fondo de cor vese no visor, detrás do recorte", await viewBg());
    await app.ev(`[...document.querySelectorAll('.stack [role=radio]')].find(b=>b.textContent.trim()==='Personalizado')?.click()`);
    await app.sleep(300);
    c.ok(await app.ev(`!!document.querySelector('.stack input[aria-label$="(hex)"]')`), "«Personalizado» ensina o selector de cor");
    await app.shot("cutout-vivo.png");
    const before = await cutSrc();
    await app.slider("Recorte", 90);
    await app.waitFor(`document.querySelector('img.cut')?.getAttribute('src') !== ${JSON.stringify(before)}`, "a vista previa nova ao mover o axuste", 30000);
    c.ok(true, "mover un axuste actualiza a vista previa");
    await app.ev(`[...document.querySelectorAll('.stack [role=radio]')].find(b=>b.textContent.trim()==='Transparente')?.click()`);
    await app.sleep(200);
    c.ok((await viewBg()) === "", "co fondo transparente o visor volve ao xadrez", await viewBg());
    await app.slider("Recorte", 50);
    await app.sleep(1200);
    await press("Quitar fondo");
    const out = path.join(app.media, "obxecto-sen-fondo.png");
    await app.waitFor(`document.body.innerText.includes('Quedou o')`, "quitar o fondo", 120000).catch(() => {});
    c.ok(fs.existsSync(out), "gárdase obxecto-sen-fondo.png");
    if (fs.existsSync(out)) {
      const r = alphaRange(out);
      c.ok(r.min <= 10 && r.max >= 245, "o resultado ten fondo transparente e obxecto opaco", JSON.stringify(r));
    }
    // Ao saír da ferramenta o visor volve á imaxe orixinal.
    await app.click((await app.tabs())[0]);
    c.ok(!(await app.ev(`!!document.querySelector('img.cut')`)), "ao deixar a ferramenta o visor volve á imaxe orixinal");

    // ---- Descargar por ligazón (só audio, que pesa pouco)
    const url = process.env.TRASNO_E2E_URL ?? "https://www.youtube.com/watch?v=jNQXAC9IVRw";
    await app.ev(`localStorage.setItem('arquivos-do-trasno.output-mode','folder'); localStorage.setItem('arquivos-do-trasno.output-folder', ${JSON.stringify(app.media)}); localStorage.setItem('arquivos-do-trasno.descarga', JSON.stringify({ kind: 'audio', audio: 'mp3' })); location.reload()`);
    await app.sleep(2500);
    await app.radial("Descargar por ligazón");
    c.ok(!(await app.ev(`!!document.querySelector('[role=dialog] [data-testid=extension-gate]')`)), "con yt-dlp e FFmpeg non sae o aviso");
    c.ok(await app.ev(`!!document.querySelector('[role=dialog] header img')`), "o trasno vai na xanela, non borroso ao fondo");
    c.ok(await app.ev(`getComputedStyle(document.querySelector('.dock')).visibility === 'hidden'`), "e o do recanto queda agochado mentres tanto");
    c.ok(await app.ev(`document.querySelector('[role=dialog] [role=dialog], .box')?.getBoundingClientRect().width <= 540`), "a xanela é estreita");
    await app.ev(`(()=>{const i=document.querySelector('[role=dialog] input[type=text]'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i, ${JSON.stringify(url)}); i.dispatchEvent(new Event('input',{bubbles:true}));})()`);
    await app.sleep(1500);
    c.ok(!(await app.ev(`!!document.querySelector('[data-testid=media-info]')`)), "ao escribir a ligazón non se busca soa");
    await app.ev(`document.querySelector('[data-testid=download-search]')?.click()`);
    await app.waitFor(`document.querySelector('[role=dialog] [data-testid=media-info], [role=dialog] .warn')`, "comprobar a ligazón", 90000);
    await app.waitFor(`document.querySelector('[data-testid=media-thumb]')?.naturalWidth > 0`, "a miniatura", 20000).then(() => c.ok(true, "sae a miniatura do vídeo á esquerda dos detalles"), () => c.ok(false, "sae a miniatura do vídeo"));
    const checked = await app.ev(`document.querySelector('[role=dialog] [data-testid=media-info]')?.innerText.replace(/\\s+/g,' ')`);
    c.ok(/Me at the zoo/.test(checked ?? ""), "comproba soa e ensina o título", checked);
    c.ok(/YouTube/.test(checked ?? ""), "di en que plataforma está", checked);
    c.ok(/0:19|0:1\d/.test(checked ?? "") && /\b\d{1,2} (xan|feb|mar|abr|mai|xuñ|xul|ago|set|out|nov|dec) 2005\b/.test(checked ?? ""), "e a duración e a data, lexible (día, mes e ano)", checked);
    c.ok(/Sairá en\s*MP3/.test(await app.ev(`document.querySelector('[data-testid=audio-format]')?.innerText.replace(/\\s+/g,' ') ?? ''`)), "en audio di o formato que sairá");
    const kbpsOpts = await app.ev(`[...document.querySelectorAll('[role=dialog] select[aria-label="Calidade"] option')].map(o=>o.textContent).join()`);
    c.ok(/320 kbps/.test(kbpsOpts) && /128 kbps/.test(kbpsOpts), "e deixa elixir a calidade do audio", kbpsOpts);
    c.ok(/Orixinal/.test(kbpsOpts), "a primeira calidade di «Orixinal»", kbpsOpts);
    await app.ev(`[...document.querySelectorAll('[role=dialog] [role=radio]')].find(b=>b.textContent.trim()==='Vídeo')?.click()`);
    await app.sleep(300);
    const order = await app.ev(`[...document.querySelectorAll('[role=dialog] .form .opts > *')].map(e=>e.getAttribute('aria-label') || e.querySelector('select')?.getAttribute('aria-label') || e.querySelector('[role=radiogroup]')?.getAttribute('aria-label') || '').filter(Boolean).join()`);
    c.ok(/Formato,Calidade/.test(order), "en vídeo, Formato vai por riba de Calidade", order);
    c.ok(/^Orixinal \(\d+p\)$/.test(await app.ev(`document.querySelector('[role=dialog] select[aria-label="Calidade"] option')?.textContent ?? ''`)), "e a primeira calidade di «Orixinal» coa altura entre parénteses");
    const fmt = await app.ev(`document.querySelector('[data-testid=video-format]')?.innerText.replace(/\\s+/g,' ') ?? ''`);
    c.ok(/Sairá en\s*MP4\s*·\s*\S+\s*·\s*\d+p/.test(fmt), "en vídeo di contedor, códec e altura", fmt);
    await app.shot("descarga-video.png");
    await app.ev(`[...document.querySelectorAll('[role=dialog] [role=radio]')].find(b=>b.textContent.trim()==='Audio')?.click()`);
    await app.sleep(300);
    c.ok(await app.ev(`document.querySelector('[data-testid=download-go]')?.disabled === false`), "sae o botón verde Descargar");
    await app.ev(`document.querySelector('[data-testid=download-go]')?.click()`);
    await app.sleep(500);
    c.ok(await app.ev(`[...document.querySelectorAll('body > div')].some(d=>d.style.zIndex==='300' && d.textContent.includes('Descargando'))`), "o trasno leva o billete da descarga á barra de abaixo");
    await app.waitFor(`!document.querySelector('[role=dialog]')`, "que a xanela se peche", 15000);
    c.ok(true, "ao descargar péchase a xanela e a descarga segue en segundo plano");
    const running = await app.ev(`document.body.innerText`);
    c.ok(/Me at the zoo/.test(running) || /Descargando|Traballando|Procesos|1/.test(running), "queda un proceso en marcha á vista");
    await app.waitFor(`document.body.innerText.includes('Me at the zoo.mp3') || document.body.innerText.includes('Listo') || document.body.innerText.includes('Feito')`, "a descarga", 180000).catch(() => {});
    await app.sleep(2500);
    const got = fs.readdirSync(app.media).find((n) => /Me at the zoo.*\.mp3$/.test(n));
    c.ok(!!got, "o audio gárdase no cartafol elixido", fs.readdirSync(app.media).join(", "));
    c.ok(!fs.readdirSync(app.media).some((n) => n.startsWith(".trasno-descarga")), "sen cartafol temporal da descarga");
    const listed = await app.ev(`[...document.querySelectorAll('li[role=option]')].some(li=>li.innerText.includes('Me at the zoo'))`);
    c.ok(listed, "o ficheiro descargado entra na lista");
  },
};
