// Vídeo .ts (MPEG-TS), o que gardan Ace Stream e os reproductores de emisións: abre, móstrase no visor e convértese a MP4.
// A mostra imita unha gravación real: H.264 + AAC, comeza no medio dun paquete e acaba cortada.
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { ffmpegBin, ffprobeDuration } from "../harness.mjs";

export default {
  about: "Vídeo .ts (Ace Stream): abrir, ver no visor, e converter a MP4",
  async run(app, c) {
    const full = path.join(app.media, "completo.ts");
    const r = spawnSync(ffmpegBin(), ["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc=s=640x360:r=25:d=8", "-f", "lavfi", "-i", "sine=f=440:r=48000:d=8", "-c:v", "libx264", "-preset", "ultrafast", "-g", "50", "-pix_fmt", "yuv420p", "-c:a", "aac", "-f", "mpegts", full]);
    if (r.status !== 0) throw new Error("FFmpeg non puido crear o .ts: " + r.stderr);
    const bytes = fs.readFileSync(full);
    const cut = Math.floor((bytes.length * 3) / 4 / 188) * 188 + 77;
    const src = path.join(app.media, "directo.ts");
    fs.writeFileSync(src, Buffer.concat([Buffer.alloc(100, 0x55), bytes.subarray(0, cut)]));
    const before = fs.readFileSync(src);

    await app.open(src);
    c.ok(await app.ev(`document.body.innerText.includes('directo.ts')`), "o .ts entra na lista de ficheiros");
    const tabs = await app.tabs();
    c.ok(tabs.includes("Converter") && tabs.includes("Editar") && tabs.includes("Son"), "trátase como vídeo: Converter, Editar e Son", tabs.join(","));

    // O visor non le MPEG-TS directamente: a app prepara unha copia que si pode reproducir.
    await app.waitFor(`(()=>{const v=document.querySelector('video'); return !!v && v.readyState>=1 && v.duration>1;})()`, "o vídeo no visor", 60000);
    const dur = await app.ev(`document.querySelector('video').duration`);
    c.ok(dur > 3, "o visor amosa o vídeo con duración", String(dur));
    if (process.env.TRASNO_SHOTS) await app.shot("video-ts.png");

    // Converter a MP4 (o formato por defecto) e que saia un ficheiro que dura o que se gravou.
    await app.click("Converter");
    await app.waitFor(`!![...document.querySelectorAll('.stack button.btn.primary')].find(b=>/Converter/.test(b.textContent))`, "o botón de converter");
    const offered = await app.ev(`[...document.querySelectorAll('.stack [role=radio]')].map(b=>b.textContent.trim().split('\\n')[0].toUpperCase())`);
    c.ok(!offered.includes("TS"), "o .ts non se ofrece como formato de saída", offered.join(","));
    await app.ev(`[...document.querySelectorAll('.stack button.btn.primary')].find(b=>/Converter/.test(b.textContent)).click()`);
    await app.waitFor(`/Feito|Listo|Vaites/.test(document.body.innerText)`, "o remate da conversión", 90000);
    const mp4 = fs.readdirSync(app.media).find((f) => /^directo.*\.mp4$/.test(f));
    c.ok(!!mp4, "gardouse un MP4 ao carón do orixinal", fs.readdirSync(app.media).join(","));
    if (mp4) c.ok(ffprobeDuration(path.join(app.media, mp4)) > 4, "o MP4 dura o que tiña a gravación", String(ffprobeDuration(path.join(app.media, mp4))));
    c.ok(Buffer.compare(fs.readFileSync(src), before) === 0, "o orixinal non se tocou");
  },
};
