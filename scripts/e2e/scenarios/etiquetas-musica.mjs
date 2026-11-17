// Etiquetas de música: ler as que ten, editalas, poñer portada e gardar unha copia sen tocar o orixinal nin o son.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

const probeJson = (file, entries) => {
  const r = spawnSync(ffmpegBin().replace(/ffmpeg(\.exe)?$/i, "ffprobe$1"), ["-v", "error", ...entries, "-of", "json", file], { encoding: "utf8" });
  return JSON.parse(r.stdout);
};
const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");

export default {
  about: "Audio → Etiquetas: ler, editar, portada e gardar unha copia con ficheiros reais",
  async run(app, c) {
    const song = path.join(app.media, "03 - Os Cañotas - A Rula.mp3");
    const r = spawnSync(ffmpegBin(), ["-v", "error", "-y", "-f", "lavfi", "-i", "sine=f=440:r=44100:d=3", "-metadata", "title=Vella", "-metadata", "album=Álbum Vello", song]);
    if (r.status !== 0) throw new Error("non puiden crear a canción de proba");
    const before = sha(song);
    const cover = await app.textImage("portada.png", ["PORTADA"]);

    await app.open(song);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Etiquetas"), "os audios teñen a pestana Etiquetas", tabs.join(","));
    await app.click("Etiquetas");
    await app.waitFor(`document.querySelector('.stack input[aria-label="Título"]')`, "o formulario de etiquetas");
    const val = (label) => app.ev(`document.querySelector('.stack input[aria-label=${JSON.stringify(label)}], .stack textarea[aria-label=${JSON.stringify(label)}]')?.value`);
    c.ok((await val("Título")) === "Vella" && (await val("Álbum")) === "Álbum Vello", "le as etiquetas que xa tiña", `${await val("Título")} / ${await val("Álbum")}`);
    c.ok(await app.ev(`[...document.querySelectorAll('.stack .btn.primary')].find(b=>/Gardar etiquetas/.test(b.textContent)).disabled`), "sen cambios non se pode gardar");

    const set = (label, v) =>
      app.ev(`(()=>{const i=document.querySelector('.stack input[aria-label=${JSON.stringify(label)}], .stack textarea[aria-label=${JSON.stringify(label)}]'); const proto=i.tagName==='TEXTAREA'?HTMLTextAreaElement:HTMLInputElement; Object.getOwnPropertyDescriptor(proto.prototype,'value').set.call(i,${JSON.stringify(v)}); i.dispatchEvent(new Event('input',{bubbles:true}));})()`);
    await set("Título", "A Rula dos Trasnos");
    await set("Letra", "Primeira liña\nSegunda liña ñ");
    await set("Ano", "2024");
    // «Sacar do nome do ficheiro» enche só o que está baleiro: o artista e a pista (o título xa estaba posto).
    await app.click("Sacar do nome do ficheiro");
    c.ok((await val("Artista")) === "Os Cañotas" && (await val("Título")) === "A Rula dos Trasnos" && (await val("Pista")) === "3", "o nome do ficheiro enche o artista e a pista sen pisar o título", `${await val("Artista")} / ${await val("Título")} / ${await val("Pista")}`);

    await app.waitFor(`typeof window.__trasnoCover === 'function'`, "o selector de portada");
    await app.ev(`window.__trasnoCover(${JSON.stringify(cover)})`);
    await app.waitFor(`document.querySelector('.stack .cover img')`, "a portada nova no panel");
    if (process.env.TRASNO_SHOTS) await app.shot("etiquetas.png"); // TRASNO_SHOTS=1: capturas en %TEMP%	rasno-e2e para revisalas á vista
    await app.ev(`[...document.querySelectorAll('.stack .btn.primary')].find(b=>/Gardar etiquetas/.test(b.textContent)).click()`);
    const out = path.join(app.media, "03 - Os Cañotas - A Rula-etiquetado.mp3");
    for (let i = 0; i < 40 && !fs.existsSync(out); i++) await app.sleep(300);
    await app.sleep(500);
    c.ok(fs.existsSync(out), "gardouse unha copia «-etiquetado» ao carón");
    c.ok(sha(song) === before, "o ficheiro orixinal queda igual, byte a byte");
    if (fs.existsSync(out)) {
      const info = probeJson(out, ["-show_entries", "format=duration:format_tags:stream=codec_type,codec_name,duration"]);
      const tags = Object.fromEntries(Object.entries(info.format.tags ?? {}).map(([k, v]) => [k.toLowerCase(), v]));
      c.ok(tags.title === "A Rula dos Trasnos" && tags.artist === "Os Cañotas" && tags.album === "Álbum Vello", "as etiquetas novas len ben noutro programa (ffprobe)", JSON.stringify(tags));
      c.ok(String(tags.date ?? tags.year ?? "").startsWith("2024") && /Segunda liña ñ/.test(tags.lyrics ?? tags["lyrics-eng"] ?? tags["unsyncedlyrics"] ?? JSON.stringify(tags)), "ano e letra gardados", JSON.stringify(tags));
      c.ok(info.streams.some((s) => s.codec_type === "video"), "leva portada (fluxo de imaxe)", JSON.stringify(info.streams));
      c.ok(Math.abs(Number(info.format.duration) - 3) < 0.15, "o son dura o mesmo", info.format.duration);
      const dec = spawnSync(ffmpegBin(), ["-v", "error", "-i", out, "-f", "null", "-"], { encoding: "utf8" });
      c.ok(dec.status === 0 && !dec.stderr.trim(), "decodifica sen queixas", dec.stderr.trim().slice(0, 100));
    }
    c.ok(/Feito|Listo|Xa est|Rematad|Todo correcto/i.test(await app.ev(`document.body.innerText`)), "o trasno avisa de que rematou");

    // Un formato sen soporte (WMA) dá unha mensaxe clara e non deixa ficheiros.
    const wma = path.join(app.media, "vella.wma");
    spawnSync(ffmpegBin(), ["-v", "error", "-y", "-f", "lavfi", "-i", "sine=f=440:d=1", wma]);
    await app.open(wma);
    await app.click("Etiquetas");
    await app.waitFor(`/Non sei ler as etiquetas/.test(document.querySelector('.stack')?.innerText||'')`, "o aviso de formato sen soporte", 15000);
    c.ok(true, "WMA: avisa de que non sabe editalo");
  },
};
