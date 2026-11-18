// Cortar en partes na app real: plantillas, zona sobre a imaxe no visor con cada parte numerada, ficheiros novos, formatos e orixinal intacto.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");
const ffprobe = () => ffmpegBin().replace(/ffmpeg(\.exe)?$/i, "ffprobe$1");
const dims = (f) => spawnSync(ffprobe(), ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", f], { encoding: "utf8" }).stdout.trim();
// Cor do píxel do centro (RGB).
const centre = (f) => {
  const r = spawnSync(ffmpegBin(), ["-v", "error", "-i", f, "-vf", "crop=1:1:iw/2:ih/2", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"], { encoding: "buffer" });
  return [...r.stdout.subarray(0, 3)];
};

export default {
  about: "Cortar en partes: plantillas, zona e números no visor, partes exactas, nomes novos, formatos e orixinal intacto",
  async run(app, c) {
    // 1200 × 400: tres cadrados de 400 (vermello, verde, azul).
    const made = await app.ev(`(()=>{const k=document.createElement('canvas'); k.width=1200; k.height=400; const g=k.getContext('2d'); ['#e61414','#14b414','#1414e6'].forEach((col,i)=>{g.fillStyle=col; g.fillRect(i*400,0,400,400);}); return k.toDataURL('image/png').split(',')[1];})()`);
    const src = path.join(app.media, "foto.png");
    fs.writeFileSync(src, Buffer.from(made, "base64"));
    const before = sha(src);
    const num = (label, v) => app.ev(`(()=>{const i=document.querySelector('.stack input[type=number][aria-label=${JSON.stringify(label)}]'); if(!i) return 'sen '+${JSON.stringify(label)}; Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i,String(${v})); i.dispatchEvent(new Event('input',{bubbles:true})); i.dispatchEvent(new Event('change',{bubbles:true})); i.dispatchEvent(new Event('blur')); return 'ok'})()`);
    const select = (label, v) => app.ev(`(()=>{const s=document.querySelector('.stack select[aria-label=${JSON.stringify(label)}]'); s.value=${JSON.stringify(v)}; s.dispatchEvent(new Event('change',{bubbles:true}));})()`);
    const preset = (name) => app.ev(`(()=>{const b=[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Plantilla"] button')].find(b=>b.innerText.split(String.fromCharCode(10))[0].trim()===${JSON.stringify(name)}); if(!b) return 'sen '+${JSON.stringify(name)}; b.click(); return 'ok'})()`);
    const shape = (name) => app.ev(`(()=>{const b=[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Forma de cada parte"] button')].find(b=>b.innerText.trim()===${JSON.stringify(name)}); if(!b) return 'sen '+${JSON.stringify(name)}; b.click(); return 'ok'})()`);
    const marked = () => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Plantilla"] button')].find(b=>b.getAttribute('aria-checked')==='true')?.innerText.split(String.fromCharCode(10))[0].trim()`);
    const row = (name) => app.ev(`[...document.querySelectorAll('.stack .results li')].find(li=>li.querySelector('.rname')?.textContent.trim()===${JSON.stringify(name)})?.querySelector('.rstate')?.textContent.trim()`);
    const badges = () => app.ev(`document.querySelectorAll('.crop .part b').length`);
    const label = () => app.ev(`document.querySelector('.crop .size')?.textContent.trim()`);

    await app.open(src);
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Cortar en partes"), "as imaxes teñen a pestana Cortar en partes", tabs.join(","));
    c.ok(tabs.indexOf("Editar") < tabs.indexOf("Redimensionar") && tabs.indexOf("Redimensionar") < tabs.indexOf("Cortar en partes"), "Redimensionar vai por debaixo de Editar", tabs.join(","));
    await app.click("Cortar en partes");
    await app.waitFor(`document.querySelectorAll('.crop .part b').length === 3`, "a zona dividida no visor con tres números");

    // Plantilla de partida: carrusel cadrado, 3 × 1.
    c.ok((await marked()) === "Carrusel", "a plantilla de partida é «Carrusel»", await marked());
    c.ok((await app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Plantilla"] button')].map(b=>b.innerText.split(String.fromCharCode(10))[0].trim()).join()`)) === "Carrusel,Vertical,Cuadrícula,Personalizado", "tres plantillas e «Personalizado»");
    c.ok(await app.ev(`document.querySelectorAll('.stack [role=radiogroup][aria-label="Plantilla"] button svg').length`) === 3, "cada plantilla leva o seu debuxo");
    c.ok((await row("Partes")) === "3 × 1 = 3" && (await row("Cada parte")) === "400 × 400 px" && (await row("Zona")) === "1200 × 400 px", "a táboa di partes, tamaño de cada unha e zona", [await row("Partes"), await row("Cada parte"), await row("Zona")].join(" | "));
    c.ok((await label()) === "3 × 1 · 400 × 400", "o visor di as columnas × filas e o tamaño de cada parte", await label());
    c.ok(await app.ev(`document.querySelectorAll('.crop .handle').length`) === 4, "coa forma fixa só hai as catro esquinas para escalar (manteñen a proporción)");
    if (process.env.TRASNO_SHOTS) await app.shot("cortar.png");

    // Outras plantillas cambian a zona no visor.
    await preset("Vertical");
    await app.waitFor(`document.querySelector('.crop .size')?.textContent.includes('300 × 400')`, "partes de 3:4");
    c.ok((await row("Zona")) === "900 × 400 px", "con 3:4 a zona encolle ata ter esa proporción", await row("Zona"));
    await preset("Cuadrícula");
    await app.waitFor(`document.querySelectorAll('.crop .part b').length === 9`, "nove partes");
    c.ok((await row("Cada parte")) === "133 × 133 px", "a cuadrícula 3×3 dá cadrados", await row("Cada parte"));
    await num("Filas", 2);
    await app.waitFor(`document.querySelectorAll('.crop .part b').length === 6`, "seis partes");
    c.ok((await marked()) === "Personalizado", "tocar as filas marca «Personalizado»", await marked());
    await shape("Libre");
    await app.waitFor(`document.querySelectorAll('.crop .handle').length === 8`, "coa forma libre hai oito asas");
    await num("Columnas", 25);
    await app.sleep(400);
    c.ok(/Cortar en 40 partes/.test(await app.panel()), "o máximo son 20 columnas: o botón di 40 partes", await app.panel());

    // Cortar: tres cadrados.
    await preset("Carrusel");
    await app.waitFor(`document.querySelector('.crop .size')?.textContent.includes('3 × 1 · 400 × 400')`, "de volta ao carrusel");

    // A zona escálase tirando dunha esquina (co rato de verdade) e mantén a proporción dos cadrados; móvese arrastrándoa.
    const centreOf = (sel, i) => app.ev(`(()=>{const r=document.querySelectorAll(${JSON.stringify(sel)})[${i}].getBoundingClientRect(); return [r.left+r.width/2, r.top+r.height/2];})()`);
    const zoneW = async () => Number(((await row("Zona")) ?? "").split(" ")[0]);
    const piece = async () => ((await row("Cada parte")) ?? "").replace(" px", "").split(" × ").map(Number);
    // O asa da esquina cae no bordo da imaxe (medio fóra): agárrase pola parte de dentro.
    const brHandle = await centreOf(".crop .handle", 2);
    await app.drag([brHandle[0] - 3, brHandle[1] - 3], [brHandle[0] - 123, brHandle[1] - 3]);
    const w1 = await zoneW();
    const [pw1, ph1] = await piece();
    c.ok(w1 < 1200 && w1 > 600, "tirar dunha esquina encolle a zona", String(w1));
    c.ok(Math.abs(pw1 - ph1) <= 1 && Math.abs(w1 / 3 - pw1) < 1, "e as partes seguen sendo cadradas", `${pw1} × ${ph1}`);
    const before1 = await app.ev(`document.querySelector('.crop').style.left`);
    const body = await centreOf(".crop", 0);
    await app.drag(body, [body[0] + 60, body[1]]);
    const after1 = await app.ev(`document.querySelector('.crop').style.left`);
    c.ok(parseFloat(after1) > parseFloat(before1), "arrastrar a zona móvea", `${before1} → ${after1}`);
    c.ok((await zoneW()) === w1, "sen mudar o tamaño");
    await app.click("Axustar á imaxe");
    await app.waitFor(`document.querySelector('.crop .size')?.textContent.includes('3 × 1 · 400 × 400')`, "a zona volta a cubrir a imaxe");
    c.ok(/Cortar en 3 partes/.test(await app.panel()), "o botón di en cantas partes corta");
    await app.click("Cortar en 3 partes");
    const out = (n, ext = "png", tail = "") => path.join(app.media, `foto-${n}${tail}.${ext}`);
    for (let i = 0; i < 60 && !fs.existsSync(out(3)); i++) await app.sleep(300);
    c.ok([1, 2, 3].every((n) => fs.existsSync(out(n))), "gardáronse foto-1.png, foto-2.png e foto-3.png ao carón");
    c.ok([1, 2, 3].every((n) => dims(out(n)) === "400,400"), "cada parte mide 400 × 400", [1, 2, 3].map((n) => dims(out(n))).join(" "));
    const cols = [1, 2, 3].map((n) => centre(out(n)));
    c.ok(cols[0][0] > 200 && cols[0][1] < 60 && cols[1][1] > 150 && cols[1][0] < 60 && cols[2][2] > 200 && cols[2][0] < 60, "e cada unha ten o seu anaco: vermello, verde e azul", JSON.stringify(cols));
    c.ok(sha(src) === before, "o orixinal queda igual, byte a byte");
    await app.waitFor(`/Listo! Deixei 3 partes de 400×400 px/.test(document.body.innerText)`, "o trasno di o que fixo", 5000).catch(() => {});
    c.ok(/Deixei 3 partes de 400×400 px/.test(await app.ev(`document.body.innerText`)), "o trasno resume o corte");

    // Outra vez: non pisa nada, todo o xogo pasa a «(2)».
    await app.click("Cortar en 3 partes");
    for (let i = 0; i < 60 && !fs.existsSync(out(3, "png", " (2)")); i++) await app.sleep(300);
    c.ok([1, 2, 3].every((n) => fs.existsSync(out(n, "png", " (2)"))) && fs.existsSync(out(1)), "a segunda vez non pisa as partes: o xogo novo leva «(2)» ao final, antes da extensión (foto-1 (2).png)");

    // Outro formato e numeración por columnas.
    c.ok((await app.ev(`[...document.querySelectorAll('.stack select[aria-label=Formato] option')][0]?.textContent.trim()`)) === "Igual que o orixinal (.png)", "o formato di «Igual que o orixinal (.png)»");
    await select("Formato", "jpg");
    await app.waitFor(`!!document.querySelector('.stack input[type=range][aria-label=Calidade]')`, "a calidade do JPG");
    await app.click("Por columnas");
    await app.waitFor(`document.querySelectorAll('.crop .part b').length === 3`, "tres partes");
    await app.click("Cortar en 3 partes");
    for (let i = 0; i < 60 && !fs.existsSync(out(3, "jpg")); i++) await app.sleep(300);
    c.ok([1, 2, 3].every((n) => fs.existsSync(out(n, "jpg"))), "en JPG saen os ficheiros .jpg", fs.readdirSync(app.media).join(","));
  },
};
