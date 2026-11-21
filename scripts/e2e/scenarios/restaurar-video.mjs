// Restaurar vídeo: cor en directo no visor, fotograma restaurado para comparar e exportación con estabilización, nun ficheiro real.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

const probe = (file) => {
  const r = spawnSync(ffmpegBin().replace(/ffmpeg(\.exe)?$/i, "ffprobe$1"), ["-v", "error", "-show_entries", "stream=codec_type,codec_name,width,height,nb_frames:format=duration", "-of", "json", file], { encoding: "utf8" });
  return JSON.parse(r.stdout);
};
const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");

export default {
  about: "Vídeo → Restaurar: cor en directo, fotograma para comparar e exportar estabilizado con ficheiros reais",
  async run(app, c) {
    const src = path.join(app.media, "treme.mp4");
    const r = spawnSync(ffmpegBin(), ["-v", "error", "-y", "-f", "lavfi", "-i", "smptebars=s=480x360:r=25:d=4,crop=400:300:x='40+22*sin(t*37)':y='30+16*cos(t*53)'", "-f", "lavfi", "-i", "sine=f=440:r=44100:d=4", "-shortest", "-c:v", "libx264", "-crf", "14", "-pix_fmt", "yuv420p", "-c:a", "aac", src]);
    if (r.status !== 0) throw new Error("non puiden crear o vídeo de proba");
    const before = sha(src);
    await app.open(src);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Restaurar"), "os vídeos teñen a pestana Restaurar", tabs.join(","));
    await app.click("Restaurar");
    await app.waitFor(`document.querySelector('.stack [role=radiogroup][aria-label="Estabilizar"]')`, "o formulario de restaurar");
    const btn = (re) => `[...document.querySelectorAll('.stack .btn')].find(b=>${re}.test(b.textContent))`;
    c.ok(await app.ev(`${btn("/^Restaurar$/")}.disabled`), "sen axustes non se pode restaurar");

    const level = (group, text) => app.ev(`[...document.querySelector('.stack [role=radiogroup][aria-label=${JSON.stringify(group)}]').querySelectorAll('button')].find(b=>b.textContent.trim().split('\\n')[0]===${JSON.stringify(text)}).click()`);

    // O botón de comparar só vale mentres se mantén premido.
    const holdOriginal = (down) => app.ev(`(()=>{const b=[...document.querySelectorAll('.stack .btn')].find(x=>x.textContent.trim()==='Ver orixinal'); b.dispatchEvent(new PointerEvent(${JSON.stringify(down ? "pointerdown" : "pointerup")},{bubbles:true,pointerId:7})); return true;})()`);

    // ---- cor en directo no visor (filtro que imita o do motor) e sen fotograma preparado: non hai salto ao desmarcar «Ver o orixinal»
    await app.slider("Brillo", 0.3);
    await app.sleep(300);
    const filter = await app.ev(`document.querySelector('video')?.style.filter || ''`);
    const values = await app.ev(`document.querySelector('#trasno-look feColorMatrix')?.getAttribute('values') || ''`);
    c.ok(/trasno-look/.test(filter) && /0\.349/.test(values), "o visor aplica o brillo en directo", `${filter} ${values.slice(-24)}`);
    // O botón de restaurar de cada axuste leva a súa etiqueta ao pasar o rato.
    await app.ev(`document.querySelector('.stack .reset').dispatchEvent(new PointerEvent('pointerover',{bubbles:true}))`);
    await app.waitFor(`document.querySelector('.tt[role=tooltip]')`, "a etiqueta do botón de restaurar", 3000);
    c.ok(/Restaurar o valor por defecto \(0\)/.test(await app.ev(`document.querySelector('.tt[role=tooltip]').textContent`)), "a etiqueta di o valor por defecto");
    await app.ev(`document.body.dispatchEvent(new PointerEvent('pointerover',{bubbles:true}))`);
    await app.sleep(1200);
    c.ok(!(await app.ev(`!!document.querySelector('img.still')`)), "só con cor non hai fotograma preparado que substitúa o vídeo");
    await holdOriginal(true);
    await app.sleep(300);
    c.ok(!(await app.ev(`document.querySelector('video')?.style.filter || ''`)), "«Manter para ver o orixinal» quita a cor");
    await holdOriginal(false);
    await app.sleep(1200);
    c.ok(/trasno-look/.test(await app.ev(`document.querySelector('video')?.style.filter || ''`)) && !(await app.ev(`!!document.querySelector('img.still')`)), "ao volver, a cor está no vídeo ao momento e nada máis cambia");
    await app.slider("Brillo", 0);
    await app.sleep(300);
    c.ok(!(await app.ev(`document.querySelector('video')?.style.filter || ''`)), "sen cor cambiada o visor queda limpo");

    // ---- arrastrar un desprazador: os botóns de arriba e «Restaurar» non pestanexan e deciden ao soltar
    const state = () => app.ev(`[!document.querySelectorAll('header .bar button')[2].disabled, [...document.querySelectorAll('.stack .btn')].some(b=>/^Restaurar$/.test(b.textContent)&&!b.disabled)].join()`);
    const idle = await state();
    await app.ev(`(()=>{const i=[...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Brillo'); i.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));})()`);
    const seen = new Set();
    for (const v of [0.1, 0, -0.1, 0, 0.2]) {
      // Só `input` (sen `change`): o que pasa mentres o rato segue premido.
      await app.ev(`(()=>{const i=[...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Brillo'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i,String(${v})); i.dispatchEvent(new Event('input',{bubbles:true}));})()`);
      await app.sleep(60);
      seen.add(await state());
    }
    c.ok(seen.size === 1 && seen.has(idle), "arrastrando, os botóns non cambian de estado", [...seen].join(" | "));
    await app.ev(`window.dispatchEvent(new PointerEvent('pointerup'))`);
    await app.sleep(300);
    c.ok((await state()) !== idle, "ao soltar, os botóns miran o valor final", await state());
    await app.slider("Brillo", 0);
    await app.sleep(300);

    // ---- o visor amosa o resultado sen botóns: ao activar un axuste de imaxe sae o fotograma procesado
    c.ok(!(await app.ev(`!!document.querySelector('img.still')`)), "sen axustes de imaxe o visor amosa o vídeo tal cal");
    await level("Quitar ruído", "Normal");
    await app.waitFor(`document.querySelector('img.still')`, "o fotograma restaurado no visor, sen premer nada", 20000);
    if (process.env.TRASNO_SHOTS) await app.shot("restaurar.png");
    c.ok(await app.ev(`(()=>{const i=document.querySelector('img.still'); return i.naturalWidth>100 && i.complete})()`), "o fotograma é unha imaxe de verdade");
    const same = async () => app.ev(`(()=>{const i=document.querySelector('img.still'),v=document.querySelector('video'); return !!i && !!v})()`);
    c.ok(!/Comparar|Ver fotograma/.test(await app.ev(`document.querySelector('.stack').innerText`)), "non hai botón de «comparar»");
    // «Manter para ver o orixinal» vale só mentres está premido: quita o fotograma procesado e volve ao soltar.
    await holdOriginal(true);
    await app.sleep(500);
    c.ok(!(await app.ev(`!!document.querySelector('img.still')`)), "premido, amosa o vídeo sen axustes");
    await holdOriginal(false);
    await app.waitFor(`document.querySelector('img.still')`, "o fotograma procesado de volta", 20000);
    c.ok(await same(), "volve o fotograma procesado");
    await level("Quitar ruído", "Non");
    await app.sleep(600);
    c.ok(!(await app.ev(`!!document.querySelector('img.still')`)), "sen axustes de imaxe o fotograma procesado desaparece");

    // ---- os botóns de arriba (desfacer, refacer, restablecer) sobreviven ao cambiar de pestana
    await level("Nitidez", "Forte");
    await app.sleep(700);
    const lastLog = await app.ev(`document.body.innerText`);
    c.ok(/O trasno [^\n]*en «treme\.mp4» \(Restaurar\)/.test(lastLog), "o historial di en que ficheiro e menú se fixo o cambio", lastLog.split("\n").filter((l) => /O trasno/.test(l)).slice(0, 2).join(" / "));
    const bar = () => app.ev(`[...document.querySelectorAll('header .bar button')].map(b=>!b.disabled)`);
    c.ok(JSON.stringify(await bar()) === "[true,false,true]", "con axustes cambiados: desfacer e restablecer activos", JSON.stringify(await bar()));
    await app.click("Información");
    await app.click("Restaurar");
    await app.waitFor(`document.querySelector('.stack [role=radiogroup][aria-label="Nitidez"]')`, "volver a Restaurar");
    c.ok(JSON.stringify(await bar()) === "[true,false,true]", "ao volver, os botóns de arriba seguen activos", JSON.stringify(await bar()));
    c.ok(/Forte/.test(await app.ev(`document.querySelector('[role=radiogroup][aria-label="Nitidez"] .on, [role=radiogroup][aria-label="Nitidez"] [aria-checked=true]')?.textContent||''`)), "e a nitidez segue en «Forte»");
    await app.ev(`document.querySelectorAll('header .bar button')[2].click()`);
    await app.sleep(400);
    c.ok(/Non/.test(await app.ev(`document.querySelector('[role=radiogroup][aria-label="Nitidez"] [aria-checked=true]')?.textContent||''`)), "restablecer deixa a nitidez en «Non»");
    await app.ev(`document.querySelectorAll('header .bar button')[0].click()`);
    await app.sleep(400);
    c.ok(/Forte/.test(await app.ev(`document.querySelector('[role=radiogroup][aria-label="Nitidez"] [aria-checked=true]')?.textContent||''`)), "desfacer volve a poñela en «Forte» (o historial sobreviviu)");
    await level("Nitidez", "Non");

    // ---- exportar estabilizado
    await level("Estabilizar", "Normal");
    await app.sleep(200);
    await app.ev(`${btn("/^Restaurar$/")}.click()`);
    const out = path.join(app.media, "treme-restaurado.mp4");
    await app.waitFor(`/Feito|Listo|Xa est|Rematad|Todo correcto|Vaites|Ouh|Caramba/.test(document.body.innerText)`, "o remate", 120000);
    for (let i = 0; i < 40 && !fs.existsSync(out); i++) await app.sleep(300);
    c.ok(fs.existsSync(out), "gardouse treme-restaurado.mp4 ao carón");
    c.ok(sha(src) === before, "o orixinal queda igual, byte a byte");
    if (fs.existsSync(out)) {
      const a = probe(src), b = probe(out);
      const v = (i) => i.streams.find((s) => s.codec_type === "video");
      const au = (i) => i.streams.find((s) => s.codec_type === "audio");
      c.ok(v(a).width === v(b).width && v(a).height === v(b).height, "o tamaño non cambia", `${v(a).width}x${v(a).height} → ${v(b).width}x${v(b).height}`);
      c.ok(Math.abs(Number(a.format.duration) - Number(b.format.duration)) < 0.2, "a duración non cambia", `${a.format.duration} → ${b.format.duration}`);
      c.ok(Math.abs(Number(v(a).nb_frames) - Number(v(b).nb_frames)) <= 1, "os mesmos fotogramas", `${v(a).nb_frames} → ${v(b).nb_frames}`);
      c.ok(au(b)?.codec_name === au(a).codec_name, "o son segue igual (copiado)");
      const dec = spawnSync(ffmpegBin(), ["-v", "error", "-i", out, "-f", "null", "-"], { encoding: "utf8" });
      c.ok(dec.status === 0 && !dec.stderr.trim(), "decodifica sen queixas", dec.stderr.trim().slice(0, 100));
      // Menos tremor: a diferenza media entre fotogramas consecutivos cae.
      const jitter = (f) => {
        const o = spawnSync(ffmpegBin(), ["-v", "error", "-i", f, "-vf", "tblend=all_mode=difference,signalstats,metadata=print:file=-", "-f", "null", "-"], { encoding: "utf8" });
        const vals = o.stdout.split("\n").filter((l) => l.startsWith("lavfi.signalstats.YAVG=")).map((l) => Number(l.split("=")[1]));
        return vals.reduce((x, y) => x + y, 0) / vals.length;
      };
      const j0 = jitter(src), j1 = jitter(out);
      c.ok(j1 < j0 * 0.75, "a imaxe treme moito menos", `${j0.toFixed(2)} → ${j1.toFixed(2)}`);
    }

    // Ao saír da pestana o visor volve ao normal.
    await app.slider("Brillo", 0.3);
    await app.click("Información");
    await app.sleep(400);
    c.ok(!(await app.ev(`document.querySelector('video')?.style.filter || ''`)), "ao saír de Restaurar o visor non queda tinguido");
  },
};
