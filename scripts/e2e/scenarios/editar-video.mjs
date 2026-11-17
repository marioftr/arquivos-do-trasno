// Editar un vídeo: todos os axustes nunha pestana, vista previa en directo no visor e unha soa exportación.
import fs from "node:fs";
import path from "node:path";
import { ffprobeDuration } from "../harness.mjs";

export default {
  about: "Editar → Axustes: resultado ordenado, vista previa en directo no visor e exportar nunha vez",
  async run(app, c) {
    const src = app.video("clip.mp4", 12);
    await app.open(src);
    await app.click("Editar");
    const radios = await app.ev(`[...document.querySelectorAll('[role=radio]')].map(b=>b.textContent.trim())`);
    c.ok(radios.includes("Axustes") && radios.includes("Silencios") && !radios.includes("Cortar"), "os modos son Axustes e Silencios", radios.join(","));

    const btn = `document.querySelector('.stack .btn.primary')`;
    c.ok(await app.ev(`${btn}?.disabled`), "Exportar está desactivado sen cambios");
    c.ok(!(await app.ev(`!!document.querySelector('.stack .rows')`)), "sen cambios non hai táboa de resultado");
    const [r, g, b] = await app.ev(`getComputedStyle(${btn}).backgroundColor.match(/[0-9]+/g).map(Number)`);
    c.ok(g > r && g > b, "o botón de exportar é verde", `rgb(${r},${g},${b})`);

    // Sen mensaxes de recheo nin botón de vista previa: a vista previa é o propio visor.
    const text = await app.ev(`document.querySelector('.stack').innerText`);
    c.ok(!/Probar uns segundos|Vaise aplicar|Crea un ficheiro novo|orixinal non se toca|recodifícase con moita calidade/.test(text), "sen texto de recheo nin botón de probar");

    // Vista previa en directo: velocidade no reprodutor e son que sae de verdade.
    await app.slider("Factor de velocidade", 2);
    c.near(await app.ev(`document.querySelector('video').playbackRate`), 2, 0.01, "o visor reproduce xa a ×2");
    await app.slider("Factor de velocidade", 1);
    await app.slider("Graves", 6);
    await app.ev(`document.querySelector('video').play()`, true);
    await app.sleep(600);
    const loud = await app.level();
    c.ok(loud > 0.02, "con graves o son sae polo grafo do visor (non en silencio)", String(loud));
    await app.slider("Multiplicador de volume", 3);
    const louder = await app.level();
    c.ok(louder > loud * 1.8, "volume ×3 soa máis alto que ×1 (pasa de 1 no visor)", `${louder} vs ${loud}`);
    await app.toggle("Quitar o son");
    c.ok((await app.level(500)) < 0.002, "«Quitar o son» silencia a vista previa");
    await app.toggle("Quitar o son");
    await app.slider("Graves", 0);
    await app.slider("Multiplicador de volume", 1);
    await app.ev(`document.querySelector('video').pause()`);

    await app.slider("Factor de velocidade", 2);
    await app.slider("Fin", 8);
    await app.slider("Inicio", 2);
    const rows = await app.ev(`[...document.querySelectorAll('.stack .rows dt')].map((d,i)=>d.textContent.trim()+'='+document.querySelectorAll('.stack .rows dd')[i].textContent.trim().split(String.fromCharCode(10)).join(' '))`);
    c.ok(rows.some((x) => /^Tramo=0:02,0 – 0:08,0$/.test(x)) && rows.some((x) => /^Velocidade=×2$/.test(x)), "o resultado lista tramo e velocidade como datos", rows.join(" | "));
    c.ok(rows.some((x) => /^Duración=0:12,0.→.0:03,0$/.test(x)), "e a duración de antes e despois", rows.join(" | "));
    c.ok(!(await app.ev(`${btn}.disabled`)), "Exportar activo cos cambios");

    await app.click("Exportar");
    await app.waitFor(`/Feito|Vaites/.test(document.body.innerText)`, "o remate", 60000);
    const out = path.join(app.media, "clip-editado.mp4");
    c.ok(fs.existsSync(out), "gardouse clip-editado.mp4 ao carón do orixinal");
    if (fs.existsSync(out)) c.near(ffprobeDuration(out), 3, 0.2, "o ficheiro dura 3 s");
    c.near(ffprobeDuration(src), 12, 0.1, "o orixinal non se tocou");
  },
};
