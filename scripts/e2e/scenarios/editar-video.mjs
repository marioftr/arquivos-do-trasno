// Editar un vídeo: todos os axustes nunha pestana, vista previa e unha soa exportación.
import fs from "node:fs";
import path from "node:path";
import { ffprobeDuration } from "../harness.mjs";

export default {
  about: "Editar → Axustes: resumo, vista previa e exportar nunha vez (corte + velocidade)",
  async run(app, c) {
    const src = app.video("clip.mp4", 12);
    await app.open(src);
    c.ok((await app.tabs()).includes("Editar"), "o vídeo ten a pestana Editar");
    await app.click("Editar");
    const radios = await app.ev(`[...document.querySelectorAll('[role=radio]')].map(b=>b.textContent.trim())`);
    c.ok(radios.includes("Axustes") && radios.includes("Silencios") && !radios.includes("Cortar"), "os modos son Axustes e Silencios (non hai pestanas soltas de cortar/velocidade)", radios.join(","));

    const btn = `document.querySelector('.stack .btn.primary')`;
    c.ok(await app.ev(`${btn}?.disabled`), "Exportar está desactivado sen cambios");
    const [r, g, b] = (await app.ev(`getComputedStyle(${btn}).backgroundColor.match(/\\d+/g).map(Number)`));
    c.ok(g > r && g > b, "o botón de exportar é verde", `rgb(${r},${g},${b})`);
    const grey = await app.ev(`[...document.querySelectorAll('.stack .btn.block')].filter(b=>/Aplicar/.test(b.textContent)).length`);
    c.ok(grey === 0, "non hai botóns «Aplicar» soltos, un por axuste");

    await app.slider("Factor de velocidade", 2);
    await app.slider("Fin", 8);
    await app.slider("Inicio", 2);
    const sum = await app.ev(`document.querySelector('.stack .sum')?.innerText`);
    c.ok(/cortar de 0:02,0 a 0:08,0/.test(sum) && /velocidade ×2/.test(sum), "o resumo di o que se vai facer", sum);
    c.ok(!(await app.ev(`${btn}.disabled`)), "Exportar activo cos cambios");

    await app.click("Probar uns segundos (8 s)");
    await app.waitFor(`!!document.querySelector('.stack video.pv')`, "a vista previa");
    await app.sleep(800);
    c.near(await app.ev(`document.querySelector('.stack video.pv').duration`), 3, 0.4, "a vista previa dura 3 s (6 s a ×2)");

    await app.click("Exportar");
    await app.waitFor(`/Feito|Vaites/.test(document.body.innerText)`, "o remate", 60000);
    const out = path.join(app.media, "clip-editado.mp4");
    c.ok(fs.existsSync(out), "gardouse clip-editado.mp4 ao carón do orixinal");
    if (fs.existsSync(out)) c.near(ffprobeDuration(out), 3, 0.2, "o ficheiro dura 3 s");
    c.near(ffprobeDuration(src), 12, 0.1, "o orixinal non se tocou");
  },
};
