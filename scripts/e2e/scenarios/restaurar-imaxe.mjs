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
    // Foto apagada: un degradado de 90 a 130 de gris.
    const made = await app.ev(`(()=>{const k=document.createElement('canvas'); k.width=600; k.height=300; const g=k.getContext('2d'); const gr=g.createLinearGradient(0,0,600,0); gr.addColorStop(0,'rgb(90,90,90)'); gr.addColorStop(1,'rgb(130,130,130)'); g.fillStyle=gr; g.fillRect(0,0,600,300); return k.toDataURL('image/png').split(',')[1];})()`);
    const src = path.join(app.media, "apagada.png");
    fs.writeFileSync(src, Buffer.from(made, "base64"));
    const before = sha(src);
    const profile = () => app.ev(`document.querySelector('.stack select[aria-label="Perfil"]')?.selectedOptions[0]?.textContent.trim()`);

    await app.open(src);
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Restaurar"), "as imaxes teñen a pestana Restaurar", tabs.join(","));
    c.ok(tabs.indexOf("Editar") < tabs.indexOf("Restaurar") && tabs.indexOf("Restaurar") < tabs.indexOf("Quitar fondo"), "Restaurar vai despois de Editar", tabs.join(","));
    await app.click("Restaurar");
    await app.waitFor(`!!document.querySelector('.stack select[aria-label="Perfil"]')`, "o selector de perfil");

    c.ok((await profile()) === "Automático", "o perfil de partida é «Automático»", await profile());
    const names = await app.ev(`[...document.querySelectorAll('.stack select[aria-label="Perfil"] option')].map(o=>o.textContent.trim()).join()`);
    c.ok(names === "Automático,Máis luz,Documento,Branco e negro,Menos ruído,Máis nitidez,Personalizado", "perfis e «Personalizado»", names);

    // A vista previa sae no visor (unha segunda imaxe por riba).
    await app.waitFor(`!!document.querySelector('img.cut')`, "a vista previa no visor", 15000);
    c.ok(true, "con «Automático» o visor amosa o resultado");

    // Tocar un axuste leva a «Personalizado».
    await app.slider("Sombras", 40);
    await app.waitFor(`document.querySelector('.stack select[aria-label="Perfil"]')?.selectedOptions[0]?.textContent.trim() === 'Personalizado'`, "tocar un axuste marca «Personalizado»");
    // Elixir un perfil pon os seus valores.
    await app.select("Perfil", "document");
    await app.waitFor(`document.querySelector('.stack select[aria-label="Tipo"], .stack [role=radiogroup][aria-label="Tipo"] [aria-checked=true]')`, "o tipo");
    c.ok(/Documento/.test(await app.ev(`document.querySelector('.stack [role=radiogroup][aria-label="Tipo"] [aria-checked=true]')?.textContent||''`)), "o perfil «Documento» pon o tipo Documento");
    await app.select("Perfil", "auto");
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
