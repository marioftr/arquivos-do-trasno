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
    fs.writeFileSync(path.join(dir, "cubo.obj"), `mtllib cubo.mtl\n${verts}\n${quad(0, "vermello")}${quad(1, "azul")}usemtl vermello
f 1 2 6 5
f 2 3 7 6
f 3 4 8 7
f 4 1 5 8
`);
    await app.open(path.join(dir, "cubo.obj"));
    await app.waitFor(`document.querySelector('.canvas canvas')`, "o visor 3D", 20000);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Escena"), "hai unha pestana Escena", tabs.join(","));
    c.ok(!(await app.ev(`!!document.querySelector('[id=modelo], .side')`)) && !(await app.ev(`/Agochar panel|Panel do modelo/.test(document.body.innerText)`)), "o visor xa non leva panel flotante");
    // A malla debuxa os 12 bordos do cubo (polígonos), non os 18 das diagonais dos triángulos.
    await app.ev(`document.querySelector('.bar input[type=checkbox]').click()`);
    await app.sleep(500);
    const wire = await app.ev(`window.__trasnoWire?.()`);
    c.ok(wire && wire.tris === 12 && wire.lines === 12, "a malla amosa 12 arestas de polígono nun cubo de 12 triángulos", JSON.stringify(wire));
    if (process.env.TRASNO_SHOTS) await app.shot("malla.png");
    await app.ev(`document.querySelector('.bar input[type=checkbox]').click()`);
    await app.sleep(300);
    c.ok((await app.ev(`window.__trasnoWire?.().lines`)) === 0, "sen malla non hai liñas");
    await app.click("Escena");
    await app.waitFor(`document.querySelectorAll('.stack .mat').length === 2`, "os dous materiais na pestana", 15000);
    const text = await app.panel();
    c.ok(/Azimut/i.test(text) && /Materiais · 2/i.test(text) && /Texturas/i.test(text), "Escena ten luz, materiais e texturas", text.slice(0, 160));
    c.ok(!/Exportar|Converter a/.test(text), "e ningún botón de exportar");
    // Illar un material desde a pestana mostra só ese no visor (as outras mallas ocúltanse).
    c.ok(!(await app.ev(`!!document.querySelector('.stack .nodes')`)), "con un só obxecto non hai lista de obxectos");
    await app.ev(`document.querySelectorAll('.stack .mat')[1].click()`);
    await app.sleep(300);
    c.ok(await app.ev(`document.querySelectorAll('.stack .mat')[1].classList.contains('on')`), "premer un material déixao marcado");
    // A luz móvese co desprazador.
    await app.slider("Azimut", 120);
    c.near(await app.ev(`Number([...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Azimut').value)`), 120, 0.1, "o desprazador da luz fica onde se puxo");

    // Un ficheiro con varios obxectos: lista na pestana, cada un agochable e «Só» para velo illado.
    const tri = (n, x) => {
      const b = (x / 2) * 3;
      return [`o ${n}`, `v ${x} 0 0`, `v ${x + 1} 0 0`, `v ${x} 1 0`, `usemtl vermello`, `f ${b + 1} ${b + 2} ${b + 3}`].join("\n") + "\n";
    };
    fs.writeFileSync(path.join(dir, "escena.obj"), `mtllib cubo.mtl\n${tri("alfa", 0)}${tri("beta", 2)}${tri("gama", 4)}`);
    await app.open(path.join(dir, "escena.obj"));
    await app.waitFor(`document.querySelector('.canvas canvas')`, "o visor 3D");
    await app.click("Escena");
    await app.waitFor(`document.querySelectorAll('.stack .nodes li').length === 3`, "tres obxectos na lista", 15000);
    const names = await app.ev(`[...document.querySelectorAll('.stack .nodes .nn')].map(x=>x.textContent.trim())`);
    c.ok(["alfa", "beta", "gama"].every((n) => names.includes(n)), "a lista leva os nomes do ficheiro", names.join(","));
    const shown = () => app.ev(`window.__trasnoWire?.().shown`);
    c.ok((await shown()) === 3, "os tres obxectos están no visor", String(await shown()));
    await app.ev(`document.querySelectorAll('.stack .nodes input[type=checkbox]')[1].click()`);
    await app.sleep(400);
    c.ok((await shown()) === 2, "agochar «beta» quítao do visor", String(await shown()));
    await app.ev(`document.querySelectorAll('.stack .nodes .solo')[2].click()`);
    await app.sleep(400);
    c.ok((await shown()) === 1, "«Só» en «gama» deixa un só obxecto", String(await shown()));
    await app.ev(`document.querySelectorAll('.stack .nodes .solo')[2].click()`);
    await app.sleep(400);
    c.ok((await shown()) === 3, "«Só» outra vez volve amosalos todos", String(await shown()));
  },
};
