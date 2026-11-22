// Restaurar imaxes na app real: pestana, perfis con «Personalizado», vista previa no visor, ficheiro novo e orixinal intacto.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");
// Cor media (RGB) dunha imaxe, con FFmpeg.
const mean = (f) => {
  const r = spawnSync(ffmpegBin(), ["-v", "error", "-i", f, "-vf", "scale=1:1:flags=area", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"], { encoding: "buffer" });
  return [...r.stdout.subarray(0, 3)];
};

export default {
  about: "Restaurar imaxes: pestana, perfis (Automático… Personalizado), vista previa no visor, resultado novo e orixinal intacto",
  async run(app, c) {
    // Foto apagada: un degradado de 60 a 190 de gris.
    const made = await app.ev(`(()=>{const k=document.createElement('canvas'); k.width=600; k.height=300; const g=k.getContext('2d'); const gr=g.createLinearGradient(0,0,600,0); gr.addColorStop(0,'rgb(60,60,60)'); gr.addColorStop(1,'rgb(190,190,190)'); g.fillStyle=gr; g.fillRect(0,0,600,300); return k.toDataURL('image/png').split(',')[1];})()`);
    const src = path.join(app.media, "apagada.png");
    fs.writeFileSync(src, Buffer.from(made, "base64"));
    const before = sha(src);
    const PERFIL = `.stack [role=radiogroup][aria-label="Perfil"]`;
    const profile = () => app.ev(`document.querySelector('${PERFIL} [aria-checked=true]')?.textContent.trim()`);
    const pick = (name) => app.ev(`[...document.querySelectorAll('${PERFIL} button')].find(b=>b.textContent.trim()===${JSON.stringify(name)})?.click()`);

    await app.open(src);
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Restaurar"), "as imaxes teñen a pestana Restaurar", tabs.join(","));
    c.ok(tabs.indexOf("Transformar") < tabs.indexOf("Restaurar") && tabs.indexOf("Restaurar") < tabs.indexOf("Quitar fondo"), "Restaurar vai despois de Transformar", tabs.join(","));
    await app.click("Restaurar");
    await app.waitFor(`!!document.querySelector('${PERFIL}')`, "o selector de perfil");

    c.ok((await profile()) === "Automático", "o perfil de partida é «Automático»", await profile());
    const names = await app.ev(`[...document.querySelectorAll('${PERFIL} button')].map(o=>o.textContent.trim()).join()`);
    c.ok(names === "Máis luz,Documento,Branco e negro,Menos ruído,Máis nitidez,Automático,Personalizado", "perfís e «Personalizado», todos en botóns", names);
    const rowsY = await app.ev(`[...new Set([...document.querySelectorAll('${PERFIL} button')].map(b=>Math.round(b.getBoundingClientRect().top)))].length`);
    c.ok(rowsY === 2, "os perfís van en dúas filas (cinco arriba; Automático e Personalizado abaixo)", String(rowsY));
    c.ok((await app.ev(`document.querySelectorAll('${PERFIL} button svg').length`)) === 7, "cada perfil leva a súa icona");
    c.ok(!(await app.ev(`!!document.querySelector('.stack [role=radiogroup][aria-label="Tipo"]')`)), "non hai un selector de Tipo aparte");
    const panel = await app.panel();
    c.ok(/Brillo/.test(panel) && /Contraste/.test(panel) && /Desenfoque/.test(panel) && /Inverter cores/.test(panel), "cor e efectos (brillo, contraste, desenfoque, inverter) están en Restaurar");

    // A vista previa sae no visor (unha segunda imaxe por riba).
    await app.waitFor(`!!document.querySelector('img.cut')`, "a vista previa no visor", 15000);
    c.ok(true, "con «Automático» o visor amosa o resultado");
    await app.shot("restaurar-imx.png");

    // Tocar un axuste leva a «Personalizado».
    await app.slider("Sombras", 40);
    await app.waitFor(`document.querySelector('${PERFIL} [aria-checked=true]')?.textContent.trim() === 'Personalizado'`, "tocar un axuste marca «Personalizado»");
    // Ver orixinal: ao soltar, o resultado volve á vez (xa está en memoria).
    await app.waitFor(`!!document.querySelector('img.cut')`, "a vista previa cos novos axustes", 15000);
    await app.ev(`document.querySelector('.stack .estimate button').dispatchEvent(new PointerEvent('pointerdown',{bubbles:true,pointerId:1}))`);
    await app.sleep(150);
    c.ok(!(await app.ev(`!!document.querySelector('img.cut')`)), "mantendo premido, o visor amosa o orixinal");
    await app.ev(`document.querySelector('.stack .estimate button').dispatchEvent(new PointerEvent('pointerup',{bubbles:true,pointerId:1}))`);
    await app.sleep(80);
    c.ok(await app.ev(`!!document.querySelector('img.cut')`), "ao soltar, o resultado volve á vez, sen agardar");
    // Elixir un perfil pon os seus valores.
    await pick("Documento");
    await app.sleep(300);
    c.ok((await profile()) === "Documento", "elixir «Documento» queda marcado (e pon o tipo)", await profile());
    c.ok(await app.ev(`[...document.querySelectorAll('.stack [role=switch], .stack input[type=checkbox]')].some(x=>x.disabled || x.getAttribute('aria-disabled')==='true')`), "co tipo Documento, os controis de cor quedan desactivados");
    await pick("Personalizado");
    await app.sleep(300);
    c.ok((await profile()) === "Personalizado", "«Personalizado» pódese elixir e queda marcado", await profile());
    await pick("Automático");
    await app.sleep(300);
    c.ok((await profile()) === "Automático", "e Automático volve ao seu perfil", await profile());
    await app.sleep(500);

    // Gardar: ficheiro novo ao carón; o orixinal non se toca e a luz estírase.
    await app.click("Restaurar imaxe");
    const out = path.join(app.media, "apagada-restaurado.png");
    for (let i = 0; i < 40 && !fs.existsSync(out); i++) await app.sleep(300);
    c.ok(fs.existsSync(out), "gardouse apagada-restaurado.png ao carón");
    c.ok(sha(src) === before, "o orixinal queda intacto");
    if (fs.existsSync(out)) {
      const [a, b] = [mean(src)[0], mean(out)[0]];
      c.ok(b !== a, "o resultado non é igual ao orixinal", `${a} → ${b}`);
      const probe = spawnSync(ffmpegBin(), ["-v", "error", "-i", out, "-vf", "crop=1:1:0:150", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"], { encoding: "buffer" });
      c.ok(probe.stdout[0] < 25, "o escuro do extremo vai case a negro (niveis automáticos)", String(probe.stdout[0]));
    }
  },
};
