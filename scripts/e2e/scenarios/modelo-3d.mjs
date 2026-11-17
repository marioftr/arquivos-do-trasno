// Modelo 3D: a vista (luz, canles, materiais, texturas) é unha pestana máis das ferramentas e só afecta ao visor.
import fs from "node:fs";
import path from "node:path";

export default {
  about: "Modelo 3D: pestana Vista nas ferramentas (sen panel flotante nin exportación) que controla o visor",
  async run(app, c) {
    const dir = app.media;
    fs.writeFileSync(path.join(dir, "cubo.mtl"), "newmtl vermello\nKd 0.9 0.1 0.1\nnewmtl azul\nKd 0.1 0.1 0.9\n");
    const quad = (z, m) => `usemtl ${m}\nf ${1 + z * 4} ${2 + z * 4} ${3 + z * 4} ${4 + z * 4}\n`;
    const verts = [0, 1].flatMap((z) => [[0, 0, z], [1, 0, z], [1, 1, z], [0, 1, z]]).map((v) => `v ${v.join(" ")}`).join("\n");
    fs.writeFileSync(path.join(dir, "cubo.obj"), `mtllib cubo.mtl\n${verts}\n${quad(0, "vermello")}${quad(1, "azul")}`);
    await app.open(path.join(dir, "cubo.obj"));
    await app.waitFor(`document.querySelector('.canvas canvas')`, "o visor 3D", 20000);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Vista"), "hai unha pestana Vista", tabs.join(","));
    c.ok(!(await app.ev(`!!document.querySelector('[id=modelo], .side')`)) && !(await app.ev(`/Agochar panel|Panel do modelo/.test(document.body.innerText)`)), "o visor xa non leva panel flotante");
    await app.click("Vista");
    await app.waitFor(`document.querySelectorAll('.stack .mat').length === 2`, "os dous materiais na pestana", 15000);
    const text = await app.panel();
    c.ok(/Azimut/i.test(text) && /Materiais · 2/i.test(text) && /Texturas/i.test(text), "Vista ten luz, materiais e texturas", text.slice(0, 160));
    c.ok(!/Exportar|Converter a/.test(text), "e ningún botón de exportar");
    // Illar un material desde a pestana mostra só ese no visor (as outras mallas ocúltanse).
    await app.ev(`document.querySelectorAll('.stack .mat')[1].click()`);
    await app.sleep(300);
    c.ok(await app.ev(`document.querySelectorAll('.stack .mat')[1].classList.contains('on')`), "premer un material déixao marcado");
    // A luz móvese co desprazador.
    await app.slider("Azimut", 120);
    c.near(await app.ev(`Number([...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Azimut').value)`), 120, 0.1, "o desprazador da luz fica onde se puxo");
  },
};
