// Modelo 3D: a pestana Escena (tres grupos: obxectos e aspecto, materiais e texturas, luz) controla o visor sen exportar nada.
import fs from "node:fs";
import path from "node:path";

export default {
  about: "Modelo 3D: pestana Escena en tres grupos con lista de obxectos en árbore, botóns que alternan, malla, materiais e luz",
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
    // Os tres grupos do panel van arriba, como Protexer e Desbloquear nos PDF.
    const part = (name) => app.click(name);
    const parts = () => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup] button')].map(b=>b.innerText.split(String.fromCharCode(10))[0].trim()).join()`);
    const malla = () => app.ev(`[...document.querySelectorAll('.stack label.chk')].find(l=>/Malla/.test(l.textContent)).querySelector('input').click()`);

    await app.open(path.join(dir, "cubo.obj"));
    await app.waitFor(`document.querySelector('.canvas canvas')`, "o visor 3D", 20000);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Escena"), "hai unha pestana Escena", tabs.join(","));
    c.ok(!(await app.ev(`!!document.querySelector('[id=modelo], .side')`)) && !(await app.ev(`/Agochar panel|Panel do modelo/.test(document.body.innerText)`)), "o visor xa non leva panel flotante");
    c.ok(!(await app.ev(`!!document.querySelector('.bar input[type=checkbox]')`)), "a barra do visor xa non leva a malla nin a cuadrícula");
    await app.click("Escena");
    await app.waitFor(`document.querySelector('.stack [role=radiogroup]')`, "os grupos do panel");
    c.ok((await parts()) === "Obxectos e aspecto,Materiais e texturas,Luz", "tres grupos arriba: obxectos e aspecto, materiais e texturas, luz", await parts());
    c.ok(!(await app.ev(`[...document.querySelectorAll('.stack .group-title')].some(b=>b.tagName==='BUTTON')`)), "os grupos non se pregan: o título é só texto");
    // O grupo de obxectos e aspecto: a malla e a cuadrícula.
    // A malla debuxa os 12 bordos do cubo (polígonos), non os 18 das diagonais dos triángulos.
    await malla();
    await app.sleep(500);
    const wire = await app.ev(`window.__trasnoWire?.()`);
    c.ok(wire && wire.tris === 12 && wire.lines === 12, "a malla amosa 12 arestas de polígono nun cubo de 12 triángulos", JSON.stringify(wire));
    if (process.env.TRASNO_SHOTS) await app.shot("malla.png");
    await malla();
    await app.sleep(300);
    c.ok((await app.ev(`window.__trasnoWire?.().lines`)) === 0, "sen malla non hai liñas");
    c.ok(await app.ev(`[...document.querySelectorAll('.stack label.chk')].some(l=>/Cuadrícula/.test(l.textContent))`), "a cuadrícula está no mesmo grupo");
    c.ok(!(await app.ev(`!!document.querySelector('.stack .nodes')`)), "con un só obxecto non hai lista de obxectos");
    c.ok(!(await app.ev(`!!document.querySelector('.stack input[aria-label=Azimut]')`)) && !(await app.ev(`!!document.querySelector('.stack .mat')`)), "a luz e os materiais non se ven neste grupo");

    // Materiais e texturas.
    await part("Materiais e texturas");
    await app.waitFor(`document.querySelectorAll('.stack .mat').length === 2`, "os dous materiais", 15000);
    const text = await app.panel();
    c.ok(/Materiais[\s|]*2/i.test(text) && /Texturas/i.test(text), "materiais e texturas van xuntos", text.slice(0, 200));
    c.ok(!/Exportar|Converter a/.test(text), "e ningún botón de exportar");
    // Illar un material desde a pestana mostra só ese no visor.
    await app.ev(`document.querySelectorAll('.stack .mat')[1].click()`);
    await app.sleep(300);
    c.ok(await app.ev(`document.querySelectorAll('.stack .mat')[1].classList.contains('on')`), "premer un material déixao marcado");

    // A luz ten o seu grupo e móvese co desprazador.
    await part("Luz");
    await app.waitFor(`document.querySelector('.stack input[aria-label=Azimut]')`, "a luz");
    await app.slider("Azimut", 120);
    c.near(await app.ev(`Number([...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Azimut').value)`), 120, 0.1, "o desprazador da luz fica onde se puxo");
    await part("Materiais e texturas");
    await part("Luz");
    c.near(await app.ev(`Number([...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Azimut').value)`), 120, 0.1, "ao volver á luz segue onde estaba");

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
    await part("Obxectos e aspecto");
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

    // Pregar un grupo quita as súas filas. Os botóns da barra alternan: só hai o que serve en cada momento.
    c.ok(await app.ev(`!!document.querySelector('[title="Amosar todos os obxectos"]') && !document.querySelector('[title="Agochar todos os obxectos"]')`), "con obxectos agochados só hai o botón de amosar todos");
    await app.ev(`document.querySelector('[title="Amosar todos os obxectos"]').click()`);
    await app.sleep(300);
    c.ok(await app.ev(`!!document.querySelector('[title="Agochar todos os obxectos"]') && !document.querySelector('[title="Amosar todos os obxectos"]')`), "co todo á vista só hai o botón de agochar todos");
    await click(0, "button.fold");
    await app.sleep(200);
    c.ok((await app.ev(`document.querySelectorAll('.stack .nodes li').length`)) === 3, "pregar un grupo agocha as súas filas");
    await app.ev(`document.querySelector('[title="Pregar todos os grupos"]').click()`);
    await app.sleep(200);
    c.ok((await app.ev(`document.querySelectorAll('.stack .nodes li').length`)) === 2 && (await app.ev(`!document.querySelector('[title="Pregar todos os grupos"]') && !!document.querySelector('[title="Despregar todos os grupos"]')`)), "pregar todos deixa só os dous grupos e o botón pasa a despregar");
    await app.ev(`document.querySelector('[title="Despregar todos os grupos"]').click()`);
    await app.sleep(200);
    c.ok((await app.ev(`document.querySelectorAll('.stack .nodes li').length`)) === 5, "despregar todos devolve as filas");
    c.ok(await app.ev(`!document.querySelector('[title="Ampliar a lista"]')`), "con poucas filas non hai botón de ampliar a lista (non hai desprazamento)");

    // Con moitas filas a lista desprázase e aparece o botón de ampliala.
    const many = { asset: { version: "2.0" }, scene: 0, scenes: [{ nodes: Array.from({ length: 30 }, (_, i) => i) }], nodes: Array.from({ length: 30 }, (_, i) => ({ name: `m${i + 1}`, mesh: 0 })), meshes: [{ primitives: [{ attributes: { POSITION: 0 } }] }], accessors: [acc(0)], bufferViews: [{ buffer: 0, byteLength: 108 }], buffers: [{ byteLength: 108, uri: `data:application/octet-stream;base64,${buf.toString("base64")}` }] };
    fs.writeFileSync(path.join(dir, "moitos.gltf"), JSON.stringify(many));
    await app.open(path.join(dir, "moitos.gltf"));
    await app.waitFor(`document.querySelector('.canvas canvas')`, "o visor 3D");
    await app.click("Escena");
    await part("Obxectos e aspecto");
    await app.waitFor(`document.querySelectorAll('.stack .nodes li').length === 30`, "trinta obxectos", 15000);
    await app.waitFor(`document.querySelector('[title="Ampliar a lista"]')`, "o botón de ampliar a lista");
    await app.ev(`document.querySelector('[title="Ampliar a lista"]').click()`);
    await app.sleep(200);
    c.ok(await app.ev(`document.querySelector('.stack .nodes').classList.contains('wide') && !!document.querySelector('[title="Limitar a altura da lista"]')`), "a lista pódese ampliar sen desprazamento e o botón pasa a limitar");
    if (process.env.TRASNO_SHOTS) await app.shot("escena.png");
  },
};
