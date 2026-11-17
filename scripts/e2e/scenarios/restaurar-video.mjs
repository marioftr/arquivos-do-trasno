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

    // ---- cor en directo no visor
    await app.slider("Brillo", 0.3);
    await app.sleep(300);
    const filter = await app.ev(`document.querySelector('video')?.style.filter || ''`);
    c.ok(/brightness\(1\.6/.test(filter), "o visor aplica o brillo en directo", filter);
    await app.slider("Brillo", 0);
    await app.sleep(300);
    c.ok(!(await app.ev(`document.querySelector('video')?.style.filter || ''`)), "sen cor cambiada o visor queda limpo");

    // ---- fotograma restaurado por riba do vídeo
    c.ok(await app.ev(`${btn("/Ver fotograma restaurado/")}.disabled`), "sen axustes de imaxe non hai fotograma que comparar");
    await level("Quitar ruído", "Normal");
    await app.sleep(200);
    await app.ev(`${btn("/Ver fotograma restaurado/")}.click()`);
    await app.waitFor(`document.querySelector('img.still')`, "o fotograma restaurado no visor", 20000);
    if (process.env.TRASNO_SHOTS) await app.shot("restaurar.png");
    c.ok(await app.ev(`(()=>{const i=document.querySelector('img.still'); return i.naturalWidth>100 && i.complete})()`), "o fotograma é unha imaxe de verdade");
    c.ok(/Ver o orixinal/.test(await app.ev(`document.querySelector('.stack').innerText`)), "o botón pasa a «Ver o orixinal»");
    await app.ev(`${btn("/Ver o orixinal/")}.click()`);
    await app.sleep(300);
    c.ok(!(await app.ev(`!!document.querySelector('img.still')`)), "«Ver o orixinal» quita o fotograma");
    await level("Quitar ruído", "Non");

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
