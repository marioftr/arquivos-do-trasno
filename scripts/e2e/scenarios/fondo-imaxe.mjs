// Fondo das imaxes con transparencia: o mesmo selector (botóns grandes con debuxo e Personalizado) en Converter, Editar e Quitar fondo, o fondo
// que se ve no visor en directo e a cor que sae no ficheiro.
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

/** Cor (R, G, B) do píxel (x, y) dunha imaxe, medida con FFmpeg. */
function pixel(file, x = 2, y = 2) {
  const r = spawnSync(ffmpegBin(), ["-v", "error", "-i", file, "-vf", `crop=1:1:${x}:${y},format=rgb24`, "-f", "rawvideo", "-"], { encoding: "buffer" });
  return [...r.stdout.subarray(0, 3)];
}

export default {
  about: "Selector de fondo unificado (Converter): botóns, cor personalizada, fondo no visor e cor do ficheiro",
  async run(app, c) {
    // Un círculo vermello sobre fondo transparente.
    const b64 = await app.ev(`(()=>{const c=document.createElement('canvas');c.width=120;c.height=80;const g=c.getContext('2d');g.fillStyle='#d01818';g.beginPath();g.arc(60,40,30,0,Math.PI*2);g.fill();return c.toDataURL('image/png').split(',')[1]})()`);
    const pic = path.join(app.media, "transparente.png");
    fs.writeFileSync(pic, Buffer.from(b64, "base64"));
    await app.open(pic);

    const radios = () => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Fondo"] [role=radio]')].map(b=>b.textContent.trim()).join()`);
    const pickRadio = (name) => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Fondo"] [role=radio]')].find(b=>b.textContent.trim()===${JSON.stringify(name)})?.click()`);
    const viewBg = () => app.ev(`document.querySelector('.canvas .view')?.style.background ?? ''`);

    // ---- Converter: PNG → JPG pide fondo
    await app.click("Converter");
    await app.select("Formato de saída", "jpg");
    await app.sleep(500);
    c.ok((await radios()) === "Branco,Negro,Gris,Personalizado", "Converter: botóns Branco, Negro, Gris e Personalizado (sen Transparente)", await radios());
    c.ok(/255, 255, 255/.test(await viewBg()), "o visor amosa o fondo branco", await viewBg());
    await pickRadio("Negro");
    await app.sleep(200);
    c.ok(/\(0, 0, 0\)/.test(await viewBg()), "e cambia a negro ao elixilo", await viewBg());
    await pickRadio("Personalizado");
    await app.sleep(300);
    c.ok(await app.ev(`!!document.querySelector('.stack input[aria-label$="(hex)"]')`), "Personalizado ensina o selector de cor");
    await app.ev(`(()=>{const i=document.querySelector('.stack input[aria-label$="(hex)"]'); i.value='#00ff00'; i.dispatchEvent(new Event('input',{bubbles:true})); i.dispatchEvent(new Event('change',{bubbles:true}));})()`);
    await app.sleep(300);
    c.ok(/\(0, 255, 0\)/.test(await viewBg()), "a cor escollida vese no visor", await viewBg());
    await app.shot("fondo-converter.png");
    const go = (text) => app.ev(`[...document.querySelectorAll('.btn.primary.block')].find(b=>b.textContent.trim().startsWith(${JSON.stringify(text)}))?.click()`);
    await go("Converter");
    const jpg = path.join(app.media, "transparente.jpg");
    await app.waitFor(`document.body.innerText.includes('transparente.jpg')`, "a conversión", 30000);
    await app.sleep(500);
    c.ok(fs.existsSync(jpg), "gárdase transparente.jpg");
    if (fs.existsSync(jpg)) {
      const [r, g, b] = pixel(jpg);
      c.ok(r < 40 && g > 215 && b < 40, "a esquina transparente sae da cor personalizada (verde)", `${r},${g},${b}`);
    }
  },
};
