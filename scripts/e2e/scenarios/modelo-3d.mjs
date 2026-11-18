// Modelo 3D: a pestana Escena (obxectos, aspecto, materiais, texturas e luz) controla o visor sen exportar nada.
import fs from "node:fs";
import path from "node:path";

export default {
  about: "Modelo 3D: pestana Escena con lista de obxectos en árbore, malla, materiais e luz plegables que controlan o visor",
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
    const tabsIn = async () => (await app.tabs()).join(",");
    // Abre ou pecha unha xanela interna do panel polo seu título.
    const fold = (title) => app.ev(`[...document.querySelectorAll('.stack .group-title')].find(b=>b.textContent.trim().startsWith(${JSON.stringify(title)}))?.click()`);
    const isOpen = (title) => app.ev(`[...document.querySelectorAll('.stack .group-title')].find(b=>b.textContent.trim().startsWith(${JSON.stringify(title)}))?.getAttribute('aria-expanded')==='true'`);
    const malla = () => app.ev(`[...document.querySelectorAll('.stack label.chk')].find(l=>/Malla/.test(l.textContent)).querySelector('input').click()`);

    await app.open(path.join(dir, "cubo.obj"));
    await app.waitFor(`document.querySelector('.canvas canvas')`, "o visor 3D", 20000);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Escena"), "hai unha pestana Escena", tabs.join(","));
    c.ok(!(await app.ev(`!!document.querySelector('[id=modelo], .side')`)) && !(await app.ev(`/Agochar panel|Panel do modelo/.test(document.body.innerText)`)), "o visor xa non leva panel flotante");
    c.ok(!(await app.ev(`!!document.querySelector('.bar input[type=checkbox]')`)), "a barra do visor xa non leva a malla nin a cuadrícula");
    await app.click("Escena");
    await app.waitFor(`document.querySelectorAll('.stack .mat').length === 2`, "os dous materiais na pestana", 15000);
    // A malla debuxa os 12 bordos do cubo (polígonos), non os 18 das diagonais dos triángulos.
    await malla();
    await app.sleep(500);
    const wire = await app.ev(`window.__trasnoWire?.()`);
    c.ok(wire && wire.tris === 12 && wire.lines === 12, "a malla amosa 12 arestas de polígono nun cubo de 12 triángulos", JSON.stringify(wire));
    if (process.env.TRASNO_SHOTS) await app.shot("malla.png");
    await malla();
    await app.sleep(300);
    c.ok((await app.ev(`window.__trasnoWire?.().lines`)) === 0, "sen malla non hai liñas");
    c.ok(await app.ev(`!!document.querySelector('.stack label.chk input[type=checkbox]')`) && (await app.ev(`[...document.querySelectorAll('.stack label.chk')].some(l=>/Cuadrícula/.test(l.textContent))`)), "a cuadrícula está na pestana");

    // Cada categoría ten a súa xanela: a luz vai debaixo de materiais e texturas, e a xanela pregada non se amosa.
    const titles = await app.ev(`[...document.querySelectorAll('.stack .group-title')].map(b=>b.textContent.trim().split(/\\s+/)[0])`);
    c.ok(titles.join(",") === "Aspecto,Materiais,Texturas,Luz", "orde das xanelas: aspecto, materiais, texturas e luz", titles.join(","));
    c.ok(!(await isOpen("Luz")) && !(await app.ev(`!!document.querySelector('.stack input[aria-label=Azimut]')`)), "a luz empeza pregada");
    await fold("Luz");
    await app.waitFor(`document.querySelector('.stack input[aria-label=Azimut]')`, "a luz despregada");
    const text = await app.panel();
    c.ok(/Azimut/i.test(text) && /Materiais[\s|]*2/i.test(text) && /Texturas/i.test(text), "Escena ten luz, materiais e texturas", text.slice(0, 200));
    c.ok(!/Exportar|Converter a/.test(text), "e ningún botón de exportar");
    c.ok(!(await app.ev(`!!document.querySelector('.stack .nodes')`)), "con un só obxecto non hai lista de obxectos");
    // Illar un material desde a pestana mostra só ese no visor.
    await app.ev(`document.querySelectorAll('.stack .mat')[1].click()`);
    await app.sleep(300);
    c.ok(await app.ev(`document.querySelectorAll('.stack .mat')[1].classList.contains('on')`), "premer un material déixao marcado");
    // A luz móvese co desprazador.
    await app.slider("Azimut", 120);
    c.near(await app.ev(`Number([...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Azimut').value)`), 120, 0.1, "o desprazador da luz fica onde se puxo");
    await fold("Luz");
    c.ok(!(await app.ev(`!!document.querySelector('.stack input[aria-label=Azimut]')`)), "pregar a luz agocha os desprazadores");
    await fold("Luz");
    c.near(await app.ev(`Number([...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Azimut').value)`), 120, 0.1, "e ao despregala a luz segue onde estaba");

    // Un ficheiro glTF con grupos dentro de grupos: A (a1, a2) e B (b1). Cada malla leva un triángulo.
    const tri = (x) => [x, 0, 0, x + 1, 0, 0, x, 1, 0];
    const buf = Buffer.from(new Float32Array([...tri(0), ...tri(2), ...tri(4)]).buffer);
    const acc = (i) => ({ bufferView: 0, byteOffset: i * 36, componentType: 5126, count: 3, type: "VEC3", min: [0, 0, 0], max: [6, 1, 0] });
    const gltf = {
      asset: { version: "2.0" },
      scene: 0,
      scenes: [{ nodes: [0, 3] }],
      nodes: [
        { name: "Grupo A", children: [1, 2] },
        { name: "a1", mesh: 0 },
        { name: "a2", mesh: 1 },
        { name: "Grupo B", children: [4] },
        { name: "b1", mesh: 2 },
      ],
      meshes: [0, 1, 2].map((i) => ({ primitives: [{ attributes: { POSITION: i } }] })),
      accessors: [0, 1, 2].map(acc),
      bufferViews: [{ buffer: 0, byteLength: 108 }],
      buffers: [{ byteLength: 108, uri: `data:application/octet-stream;base64,${buf.toString("base64")}` }],
    };
    fs.writeFileSync(path.join(dir, "escena.gltf"), JSON.stringify(gltf));
    await app.open(path.join(dir, "escena.gltf"));
    await app.waitFor(`document.querySelector('.canvas canvas')`, "o visor 3D");
    await app.click("Escena");
    await app.waitFor(`document.querySelectorAll('.stack .nodes li').length === 5`, "cinco obxectos na lista", 15000);
    const names = await app.ev(`[...document.querySelectorAll('.stack .nodes .nn')].map(x=>x.textContent.trim())`);
    c.ok(["Grupo_A", "a1", "a2", "Grupo_B", "b1"].every((n) => names.includes(n)), "a lista leva os nomes do ficheiro", names.join(","));
    c.ok(/Triángulos/.test(await app.ev(`document.querySelector('.stack .cols')?.innerText||''`)), "a columna dos números ten a súa etiqueta");
    const row = (i) => `document.querySelectorAll('.stack .nodes li')[${i}]`;
    const box = (i) => app.ev(`(()=>{const x=${row(i)}.querySelector('input[type=checkbox]'); return x.indeterminate ? 'some' : x.checked ? 'on' : 'off'})()`);
    const states = async () => (await Promise.all([0, 1, 2, 3, 4].map(box))).join(",");
    const shown = () => app.ev(`window.__trasnoWire?.().shown`);
    const click = (i, sel) => app.ev(`${row(i)}.querySelector(${JSON.stringify(sel)}).click()`);
    c.ok((await shown()) === 3 && (await states()) === "on,on,on,on,on", "os tres obxectos están á vista", `${await shown()} ${await states()}`);

    await click(0, "input[type=checkbox]"); // desmarcar o grupo A
    await app.sleep(400);
    c.ok((await shown()) === 1 && (await states()) === "off,off,off,on,on", "desmarcar un grupo desmarca o que leva dentro", `${await shown()} ${await states()}`);
    await click(1, "input[type=checkbox]"); // marcar só a1
    await app.sleep(400);
    c.ok((await shown()) === 2 && (await states()) === "some,on,off,on,on", "marcar un obxecto de dentro deixa o grupo marcado (parcial)", `${await shown()} ${await states()}`);
    await click(0, "input[type=checkbox]"); // grupo parcial -> desmarcar todo
    await app.sleep(300);
    await click(0, "input[type=checkbox]"); // e outra vez -> marcar todo
    await app.sleep(400);
    c.ok((await shown()) === 3 && (await states()) === "on,on,on,on,on", "marcar un grupo marca todo o seu interior", `${await shown()} ${await states()}`);

    await click(1, ".solo"); // só a1
    await app.sleep(400);
    c.ok((await shown()) === 1 && (await states()) === "some,on,off,off,off", "ver só «a1» deixa só a1, non o grupo enteiro", `${await shown()} ${await states()}`);
    c.ok(await app.ev(`${row(1)}.querySelector('.solo').classList.contains('on')`), "o botón de ver só queda marcado");
    await click(1, ".solo");
    await app.sleep(400);
    c.ok((await shown()) === 3, "premer outra vez volve amosalo todo", String(await shown()));
    await click(0, ".solo"); // só o grupo A: a1 e a2
    await app.sleep(400);
    c.ok((await shown()) === 2 && (await states()) === "on,on,on,off,off", "ver só un grupo deixa as súas mallas", `${await shown()} ${await states()}`);

    // Pregar un grupo quita as súas filas; o botón de ver todo devolve todo.
    await app.ev(`document.querySelector('[title="Amosar todos os obxectos"]').click()`);
    await app.sleep(300);
    await click(0, "button.fold");
    await app.sleep(200);
    c.ok((await app.ev(`document.querySelectorAll('.stack .nodes li').length`)) === 3, "pregar un grupo agocha as súas filas");
    await app.ev(`document.querySelector('[title="Despregar todos os grupos"]').click()`);
    await app.sleep(200);
    c.ok((await app.ev(`document.querySelectorAll('.stack .nodes li').length`)) === 5, "despregar todos devolve as filas");
    await app.ev(`document.querySelector('[title="Ampliar a lista"]').click()`);
    await app.sleep(200);
    c.ok(await app.ev(`document.querySelector('.stack .nodes').classList.contains('wide')`), "a lista pódese ampliar sen desprazamento");
    if (process.env.TRASNO_SHOTS) await app.shot("escena.png");
  },
};
