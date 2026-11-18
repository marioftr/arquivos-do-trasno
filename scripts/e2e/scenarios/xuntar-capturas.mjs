// Xuntar capturas na app real: capturas de desprazamento con cabeceira e pé fixos que quedan como a páxina orixinal, orde, formatos e orixinais intactos.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");
const ffprobe = () => ffmpegBin().replace(/ffmpeg(\.exe)?$/i, "ffprobe$1");
const dims = (f) => spawnSync(ffprobe(), ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", f], { encoding: "utf8" }).stdout.trim();
// Diferenza máxima entre dúas imaxes do mesmo tamaño (0 = iguais píxel a píxel).
const maxDiff = (a, b) => {
  const r = spawnSync(ffmpegBin(), ["-v", "error", "-i", a, "-i", b, "-lavfi", "[0:v][1:v]blend=all_mode=difference,format=gray,signalstats,metadata=print:file=-", "-f", "null", "-"], { encoding: "utf8" });
  const vals = r.stdout.split("\n").filter((l) => l.startsWith("lavfi.signalstats.YMAX=")).map((l) => Number(l.split("=")[1]));
  return vals.length ? Math.max(...vals) : NaN;
};

export default {
  about: "Xuntar capturas: de desprazamento con cabeceira e pé fixos, orde, espazo, formatos e orixinais intactos",
  async run(app, c) {
    // Páxina de 1500 filas con contido distinto en cada fila; 3 capturas de 700 con cabeceira de 60 e pé de 40 (600 de contido), cada 450 filas.
    const W = 400, VIEW = 600, HEAD = 60, FOOT = 40, STEP = 450, PAGE = STEP * 2 + VIEW;
    const made = await app.ev(`(()=>{
      const page=document.createElement('canvas'); page.width=${W}; page.height=${PAGE}; const g=page.getContext('2d');
      let s=12345; const rnd=()=>{s=(s*1103515245+12345)&0x7fffffff; return s/0x7fffffff;};
      for(let y=0;y<${PAGE};y+=14){ g.fillStyle='hsl('+Math.floor(rnd()*360)+',60%,'+(35+Math.floor(rnd()*40))+'%)'; g.fillRect(0,y,${W},14); g.fillStyle='#111'; g.font='12px Arial'; g.fillText('Liña '+(y/14)+' '+Math.floor(rnd()*1e6),8+Math.floor(rnd()*80),y+11); for(let k=0;k<4;k++){g.fillStyle='hsl('+Math.floor(rnd()*360)+',70%,50%)'; g.fillRect(Math.floor(rnd()*${W - 30}),y+2,12+Math.floor(rnd()*18),9);} }
      const frame=(draw)=>{const c=document.createElement('canvas'); c.width=${W}; c.height=${HEAD + VIEW + FOOT}; const x=c.getContext('2d'); x.fillStyle='#2a5bd7'; x.fillRect(0,0,${W},${HEAD}); x.fillStyle='#fff'; x.font='20px Arial'; x.fillText('Cabeceira fixa',10,38); x.fillStyle='#d7432a'; x.fillRect(0,${HEAD + VIEW},${W},${FOOT}); x.fillStyle='#fff'; x.fillText('Pé fixo',10,${HEAD + VIEW + 28}); draw(x); return c.toDataURL('image/png').split(',')[1];};
      const shots=[0,1,2].map(i=>frame(x=>x.drawImage(page,0,i*${STEP},${W},${VIEW},0,${HEAD},${W},${VIEW})));
      const t=document.createElement('canvas'); t.width=${W}; t.height=${HEAD + PAGE + FOOT}; const x=t.getContext('2d'); x.fillStyle='#2a5bd7'; x.fillRect(0,0,${W},${HEAD}); x.fillStyle='#fff'; x.font='20px Arial'; x.fillText('Cabeceira fixa',10,38); x.drawImage(page,0,${HEAD}); x.fillStyle='#d7432a'; x.fillRect(0,${HEAD + PAGE},${W},${FOOT}); x.fillStyle='#fff'; x.fillText('Pé fixo',10,${HEAD + PAGE + 28});
      return {shots, truth:t.toDataURL('image/png').split(',')[1]};
    })()`);
    const files = made.shots.map((b64, i) => {
      const p = path.join(app.media, `captura-${i + 1}.png`);
      fs.writeFileSync(p, Buffer.from(b64, "base64"));
      return p;
    });
    const truth = path.join(app.media, "verdade.png");
    fs.writeFileSync(truth, Buffer.from(made.truth, "base64"));
    const hashes = files.map(sha);

    // Marca só estes ficheiros (a lista pode ter restos doutros escenarios).
    const pick = async (names) => {
      await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ const want=${JSON.stringify(names)}.some(n=>x.getAttribute('aria-label')==='Marcar '+n); if(x.checked!==want) x.click(); })`);
      await app.sleep(500);
    };

    // Aberto un só: non hai nada que xuntar. Aberto os tres e marcados: aparece a pestana.
    await app.open(files[0]);
    await pick([]);
    c.ok(!(await app.tabs()).includes("Xuntar capturas"), "cunha soa imaxe non hai pestana de xuntar");
    await app.open(files[1]);
    await app.open(files[2]);
    await pick(["captura-1.png", "captura-2.png", "captura-3.png"]);
    c.ok((await app.tabs()).includes("Xuntar capturas"), "con varias imaxes marcadas hai a pestana Xuntar capturas", (await app.tabs()).join(","));
    await app.click("Xuntar capturas");
    await app.waitFor(`document.querySelectorAll('.stack ol.order li').length === 3`, "a lista ordenable");
    c.ok(await app.ev(`[...document.querySelectorAll('.stack ol.order li .nm')].map(x=>x.textContent.trim()).join()`) === "captura-1.png,captura-2.png,captura-3.png", "a orde de partida é a da lista");

    // ---- xuntar de arriba abaixo quitando o repetido
    c.ok(/Xuntar 3 capturas/.test(await app.panel()), "o botón di cantas se xuntan");
    if (process.env.TRASNO_SHOTS) await app.shot("xuntar.png");
    await app.click("Xuntar 3 capturas");
    const out = path.join(app.media, "capturas-xuntadas.png");
    for (let i = 0; i < 60 && !fs.existsSync(out); i++) await app.sleep(300);
    c.ok(fs.existsSync(out), "gardouse capturas-xuntadas.png ao carón");
    await app.waitFor(`document.querySelectorAll('.stack .results li').length === 2`, "as dúas unións", 10000);
    const expected = `${W},${HEAD + PAGE + FOOT}`;
    c.ok(dims(out) === expected, "a altura é a da páxina enteira, sen filas repetidas nin cabeceiras no medio", `${dims(out)} (agardaba ${expected})`);
    c.ok(maxDiff(out, truth) === 0, "e queda igual á páxina orixinal, píxel a píxel", String(maxDiff(out, truth)));
    const joins = await app.ev(`[...document.querySelectorAll('.stack .results li')].map(x=>x.innerText.replace(/\\s+/g,' ').trim())`);
    c.ok(joins.every((j) => /px repetidos quitados/.test(j)), "cada unión di canto se quitou", joins.join(" | "));
    c.ok(files.every((f, i) => sha(f) === hashes[i]), "as capturas orixinais quedan iguais, byte a byte");
    c.ok(/Listo! Xunteinas nunha imaxe de 400×1600 px/.test(await app.ev(`document.body.innerText`)), "o trasno di o tamaño que saíu");

    // ---- sen quitar o repetido, con espazo: todo tal cal
    await app.toggle("Quitar o que se repite");
    await app.ev(`(()=>{const i=document.querySelector('.stack input[type=number][aria-label="Espazo entre capturas"]'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i,'20'); i.dispatchEvent(new Event('input',{bubbles:true})); i.dispatchEvent(new Event('change',{bubbles:true})); i.dispatchEvent(new Event('blur'));})()`);
    await app.sleep(300);
    await app.click("Xuntar 3 capturas");
    const out2 = path.join(app.media, "capturas-xuntadas (2).png");
    for (let i = 0; i < 60 && !fs.existsSync(out2); i++) await app.sleep(300);
    c.ok(fs.existsSync(out2) && fs.existsSync(out), "a segunda vez non pisa a primeira: leva «(2)»");
    c.ok(dims(out2) === `${W},${(HEAD + VIEW + FOOT) * 3 + 20 * 2}`, "sen quitar nada vai todo tal cal, con 20 px entre cada unha", dims(out2));

    // ---- a orde cámbiase na propia pestana (a lista de ficheiros non se toca)
    await app.ev(`document.querySelectorAll('.stack ol.order li .mini')[1].click()`); // baixar a primeira
    await app.sleep(300);
    c.ok(await app.ev(`[...document.querySelectorAll('.stack ol.order li .nm')].map(x=>x.textContent.trim()).join()`) === "captura-2.png,captura-1.png,captura-3.png", "baixar unha cambia a orde");
    c.ok(await app.ev(`[...document.querySelectorAll('li[role=option] .nome')].map(x=>x.textContent.trim()).filter(n=>/^captura-/.test(n)).join()`) === "captura-1.png,captura-2.png,captura-3.png", "sen mexer na lista de ficheiros");
    c.ok(/Xuntar 3 capturas/.test(await app.panel()), "o botón segue dicindo 3 capturas");
  },
};
