// Redimensionar imaxes (soa e en lote) na app real: previsualización do tamaño, niveis con «Personalizado», modos, ficheiros novos e orixinais intactos.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");
const dims = (f) => {
  const r = spawnSync(ffmpegBin().replace(/ffmpeg(\.exe)?$/i, "ffprobe$1"), ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", f], { encoding: "utf8" });
  return r.stdout.trim();
};

export default {
  about: "Redimensionar imaxes: unha e varias, previsualización, tamaños con nome, campos ligados pola proporción, porcentaxe, tamaño exacto, nomes novos e orixinais intactos",
  async run(app, c) {
    const a = await app.textImage("uno.png", ["Un"]); // 900 × 140
    const b = await app.textImage("dous.png", ["Dous", "liñas"]); // 900 × 220
    const t = await app.textImage("tres.png", ["Tres", "a", "b"]); // 900 × 300
    const hashes = [a, b, t].map(sha);
    const row = (i) => app.ev(`document.querySelectorAll('.stack .results li')[${i}]?.innerText.replace(/\\s+/g,' ').trim()||''`);
    const level = (name) => app.ev(`(()=>{const b=[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Tamaño"] button')].find(b=>b.innerText.split(String.fromCharCode(10))[0].trim()===${JSON.stringify(name)}); if(!b) return 'sen '+${JSON.stringify(name)}; b.click(); return 'ok'})()`);
    const marked = () => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Tamaño"] button')].find(b=>b.getAttribute('aria-checked')==='true')?.innerText.split(String.fromCharCode(10))[0].trim()`);
    const num = (label, v) => app.ev(`(()=>{const i=document.querySelector('.stack input[type=number][aria-label=${JSON.stringify(label)}]'); if(!i) return 'sen '+${JSON.stringify(label)}; Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i,String(${v})); i.dispatchEvent(new Event('input',{bubbles:true})); i.dispatchEvent(new Event('change',{bubbles:true})); i.dispatchEvent(new Event('blur')); return 'ok'})()`);

    // Marca só estes ficheiros (a lista pode ter restos doutros escenarios).
    const pick = async (names) => {
      await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ const want=${JSON.stringify(names)}.some(n=>x.getAttribute('aria-label')==='Marcar '+n); if(x.checked!==want) x.click(); })`);
      await app.sleep(500);
    };

    // ---- unha imaxe
    await app.open(a);
    await pick([]);
    c.ok((await app.tabs()).includes("Redimensionar"), "as imaxes teñen a pestana Redimensionar", (await app.tabs()).join(","));
    await app.click("Redimensionar");
    await app.waitFor(`document.querySelectorAll('.stack .results li').length === 1 && /→/.test(document.querySelector('.stack .results li').innerText)`, "o tamaño que quedará");
    const modes = () => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Medida"] button')].map(b=>b.innerText.split(String.fromCharCode(10))[0].trim()).join()`);
    const levelNames = () => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Tamaño"] button')].map(b=>b.innerText.split(String.fromCharCode(10))[0].trim()).join()`);
    const field = (label) => app.ev(`document.querySelector('.stack input[type=number][aria-label=${JSON.stringify(label)}]')?.value`);
    c.ok((await modes()) === "Porcentaxe,Píxeles", "dúas formas de medir: porcentaxe e píxeles", await modes());
    c.ok((await levelNames()) === "Pequeno,Mediano,Grande,Personalizado", "tres tamaños con nome (masculino) máis «Personalizado»", await levelNames());
    c.ok(!/Esticar/.test(await app.panel()), "xa non hai «Esticar» mal escrito");
    c.ok((await row(0)).includes("900 × 140 → 900 × 140"), "por defecto («Grande», sen ampliar) unha imaxe pequena non cambia", await row(0));
    c.ok((await marked()) === "Grande", "o tamaño marcado é «Grande»", await marked());
    c.ok((await field("Ancho")) === "1920" && (await field("Alto")) === "299", "con «Grande» o lado maior vale 1920 e o alto calcúlase no campo", `${await field("Ancho")} × ${await field("Alto")}`);
    await level("Pequeno");
    await app.waitFor(`/900 × 140 → 640 × 100/.test(document.querySelector('.stack .results li')?.innerText||'')`, "a previsualización de 640");
    c.ok((await field("Ancho")) === "640" && (await field("Alto")) === "100", "«Pequeno» deixa 640 × 100 nos campos", `${await field("Ancho")} × ${await field("Alto")}`);
    // Tocar un valor á man deixa «Personalizado», e o outro campo cámbiase á vista pola proporción.
    await num("Ancho", 500);
    await app.sleep(500);
    c.ok((await marked()) === "Personalizado", "cambiar o ancho á man marca «Personalizado»", await marked());
    c.ok((await field("Alto")) === "78", "e o campo do alto cambia a 78", await field("Alto"));
    await app.waitFor(`/900 × 140 → 500 × 78/.test(document.querySelector('.stack .results li')?.innerText||'')`, "500 de ancho");
    await num("Alto", 70);
    await app.waitFor(`/→ 450 × 70/.test(document.querySelector('.stack .results li')?.innerText||'')`, "70 de alto");
    c.ok((await field("Ancho")) === "450", "cambiar o alto cambia o campo do ancho", await field("Ancho"));
    // Porcentaxe
    await app.click("Porcentaxe");
    await app.slider("Porcentaxe", 50);
    await app.waitFor(`/→ 450 × 70/.test(document.querySelector('.stack .results li')?.innerText||'')`, "50 % = 450 × 70");
    c.ok(!(await levelNames()), "co modo de porcentaxe non hai tamaños con nome");
    await app.click("Píxeles");
    // Sen manter a proporción é un tamaño exacto, e aí pregúntase que facer.
    c.ok(await app.ev(`!!document.querySelector('.stack .locked-tools.on [role=radiogroup][aria-label="Se a proporción cambia"]')`), "coa proporción mantida «Se a proporción cambia» queda apagado, non agochado");
    await app.toggle("Manter proporción");
    await num("Ancho", 300);
    await num("Alto", 300);
    await app.waitFor(`/→ 300 × 300/.test(document.querySelector('.stack .results li')?.innerText||'')`, "300 × 300 exacto");
    c.ok((await levelNames()) === "Pequeno,Mediano,Grande,Personalizado", "sen manter a proporción nada desaparece: os tamaños con nome seguen aí");
    c.ok(await app.ev(`[...document.querySelectorAll('.stack .toggle')].find(x=>x.querySelector('.lab')?.textContent.trim()==='Non ampliar as imaxes pequenas')?.querySelector('input').disabled`), "e «Non ampliar» queda apagado, non agochado");
    c.ok(/Recortar polo centro/.test(await app.panel()) && /Estirar/.test(await app.panel()), "o tamaño exacto pregunta: recortar polo centro ou estirar");
    // Un valor imposible non deixa continuar.
    await num("Ancho", 0);
    await app.sleep(700);
    c.ok(await app.ev(`[...document.querySelectorAll('.stack .btn.primary')].find(b=>/^Redimensionar/.test(b.textContent))?.disabled`), "con un tamaño imposible o botón queda desactivado");
    // Ao volver activar a proporción, as medidas axústanse ao cadro e hai tamaños con nome outra vez.
    await num("Ancho", 640);
    await app.toggle("Manter proporción");
    await app.sleep(500);
    c.ok((await field("Ancho")) === "640" && (await field("Alto")) === "100", "ao manter a proporción o cadro 640 × 300 queda en 640 × 100", `${await field("Ancho")} × ${await field("Alto")}`);
    c.ok((await marked()) === "Pequeno", "e o tamaño marcado é «Pequeno»", await marked());
    await app.waitFor(`/→ 640 × 100/.test(document.querySelector('.stack .results li')?.innerText||'')`, "volta a 640");
    if (process.env.TRASNO_SHOTS) await app.shot("redimensionar.png");
    await app.click("Redimensionar imaxe");
    const out1 = path.join(app.media, "uno-redimensionado.png");
    for (let i = 0; i < 40 && !fs.existsSync(out1); i++) await app.sleep(300);
    c.ok(fs.existsSync(out1), "gardouse uno-redimensionado.png ao carón");
    c.ok(dims(out1) === "640,100", "co tamaño previsto", dims(out1));
    c.ok(sha(a) === hashes[0], "o orixinal queda igual, byte a byte");
    await app.waitFor(`/Listo/.test(document.body.innerText)`, "a mensaxe de remate", 5000).catch(() => {});
    c.ok(/640×100 px/.test(await app.ev(`document.body.innerText`)), "e o trasno di que tamaño quedou");

    // ---- imaxe vertical: «Grande» fixa o lado maior (o alto) e o ancho calcúlase
    const made = await app.ev(`(()=>{const k=document.createElement('canvas'); k.width=300; k.height=600; const g=k.getContext('2d'); g.fillStyle='#14b414'; g.fillRect(0,0,300,600); return k.toDataURL('image/png').split(',')[1];})()`);
    const tall = path.join(app.media, "alta.png");
    fs.writeFileSync(tall, Buffer.from(made, "base64"));
    await app.open(tall);
    await pick([]);
    await app.click("Redimensionar");
    await app.waitFor(`/300 × 600/.test(document.querySelector('.stack .results li')?.innerText||'')`, "a imaxe vertical");
    await app.toggle("Non ampliar as imaxes pequenas");
    await level("Grande");
    await app.waitFor(`/→ 960 × 1920/.test(document.querySelector('.stack .results li')?.innerText||'')`, "960 × 1920");
    c.ok((await field("Ancho")) === "960" && (await field("Alto")) === "1920", "nunha imaxe vertical o lado maior é o alto e o ancho calcúlase", `${await field("Ancho")} × ${await field("Alto")}`);
    await level("Pequeno");
    await app.waitFor(`/→ 320 × 640/.test(document.querySelector('.stack .results li')?.innerText||'')`, "320 × 640");
    c.ok((await field("Ancho")) === "320" && (await field("Alto")) === "640", "«Pequeno» nunha vertical: 320 × 640", `${await field("Ancho")} × ${await field("Alto")}`);
    await num("Alto", 1000);
    await app.sleep(400);
    c.ok((await field("Ancho")) === "500", "e cambiar o alto cambia o ancho no campo", await field("Ancho"));

    // ---- varias: o lote
    await app.open(b);
    await app.open(t);
    await pick(["uno.png", "dous.png", "tres.png"]);
    c.ok((await app.tabs()).join(",").includes("Redimensionar") && (await app.tabs()).includes("Xuntar capturas"), "con varias imaxes hai Redimensionar e Xuntar capturas", (await app.tabs()).join(","));
    await app.click("Redimensionar");
    await app.waitFor(`document.querySelectorAll('.stack .results li').length === 3`, "tres filas");
    await level("Pequeno");
    await app.waitFor(`/→ 640 × 213/.test(document.querySelectorAll('.stack .results li')[2]?.innerText||'')`, "a previsualización das tres");
    c.ok(/→ 640 × 100/.test(await row(0)) && /→ 640 × 156/.test(await row(1)) && /→ 640 × 213/.test(await row(2)), "cada unha ten o seu tamaño", [await row(0), await row(1), await row(2)].join(" | "));
    await app.click("Redimensionar 3 imaxes");
    await app.waitFor(`document.querySelectorAll('.stack .results li.feito').length === 3`, "as tres feitas", 60000);
    const dir = fs.readdirSync(app.media);
    c.ok(dir.includes("dous-redimensionado.png") && dir.includes("tres-redimensionado.png"), "gardáronse os novos co sufixo", dir.join(","));
    c.ok(dir.includes("uno-redimensionado (2).png") && fs.existsSync(out1), "o que xa existía non se pisou: o novo leva «(2)»", dir.join(","));
    c.ok(dims(path.join(app.media, "dous-redimensionado.png")) === "640,156" && dims(path.join(app.media, "tres-redimensionado.png")) === "640,213", "cos tamaños previstos");
    c.ok([a, b, t].every((f, i) => sha(f) === hashes[i]), "os tres orixinais quedan iguais, byte a byte");
    c.ok(/Feito! Redimensionei 3 imaxes/.test(await app.ev(`document.body.innerText`)), "o trasno resume o lote");
  },
};
