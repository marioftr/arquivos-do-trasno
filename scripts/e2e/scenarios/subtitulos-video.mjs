// Subtítulos no vídeo: incrustar (pista con idioma), sacar a ficheiro en UTF-8 e queimar na imaxe.
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { ffmpegBin } from "../harness.mjs";

const probe = (file) => {
  const r = spawnSync(ffmpegBin().replace(/ffmpeg(\.exe)?$/i, "ffprobe$1"), ["-v", "error", "-show_entries", "stream=codec_type,codec_name:stream_tags=language", "-of", "json", file], { encoding: "utf8" });
  return JSON.parse(r.stdout).streams;
};

export default {
  about: "Vídeo → Subtítulos: incrustar, sacar e queimar con ficheiros reais",
  async run(app, c) {
    const video = app.video("peli.mp4", 4);
    // Subtítulos antigos en Windows-1252: o «ñ» ten que sobrevivir ao paso a UTF-8.
    const srt = path.join(app.media, "peli.srt");
    fs.writeFileSync(srt, Buffer.from("1\n00:00:00,500 --> 00:00:03,000\nOl\xE1, Ni\xF1o\n\n2\n00:00:03,100 --> 00:00:03,900\nAdeus\n\n", "latin1"));
    await app.open(video);
    await app.click("Subtítulos");
    const modes = await app.ev(`[...document.querySelectorAll('[role=radio]')].map(b=>b.textContent.trim())`);
    c.ok(["Incrustar", "Queimar", "Extraer"].every((m) => modes.includes(m)), "tres modos: Incrustar, Queimar e Extraer", modes.join(","));
    c.ok(await app.ev(`document.querySelector('.stack .btn.primary').disabled`), "sen subtítulos escollidos non se pode incrustar");

    // ---- incrustar
    await app.waitFor(`typeof window.__trasnoSubs === 'function'`, "o selector de subtítulos");
    await app.ev(`window.__trasnoSubs(${JSON.stringify(srt)})`);
    await app.waitFor(`/2 falas/.test(document.querySelector('.stack').innerText)`, "a lectura dos subtítulos");
    c.ok(/Windows-1252|ISO|CP1252|1252/i.test(await app.panel()), "avisa da codificación orixinal", await app.panel());
    await app.ev(`(()=>{const i=document.querySelector('.stack input[type=text], .stack input:not([type])'); const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set; set.call(i,'glg'); i.dispatchEvent(new Event('input',{bubbles:true}));})()`);
    await app.click("Incrustar");
    await app.sleep(100);
    await app.ev(`[...document.querySelectorAll('.stack .btn.primary')].find(b=>b.textContent.trim()==='Incrustar').click()`);
    const emb = path.join(app.media, "peli-subtitulado.mp4");
    await app.waitFor(`/Feito|Vaites|Listo/.test(document.body.innerText)`, "o remate", 60000);
    c.ok(fs.existsSync(emb), "gardouse peli-subtitulado.mp4 ao carón do orixinal");
    const st = fs.existsSync(emb) ? probe(emb) : [];
    c.ok(st.some((s) => s.codec_type === "subtitle" && s.tags?.language === "glg"), "o MP4 ten unha pista de subtítulos en galego", JSON.stringify(st.map((s) => [s.codec_type, s.tags?.language])));

    // ---- sacar a ficheiro desde o vídeo xa subtitulado
    await app.open(emb);
    await app.click("Subtítulos");
    await app.ev(`[...document.querySelectorAll('[role=radio]')].find(b=>b.textContent.trim()==='Extraer').click()`);
    await app.waitFor(`document.querySelectorAll('.stack [role=radio][aria-checked]').length >= 1`, "a lista de pistas");
    c.ok(/GLG/.test(await app.panel()), "a pista aparece co seu idioma", await app.panel());
    await app.ev(`[...document.querySelectorAll('.stack .btn.primary')].find(b=>/Sacar/.test(b.textContent)).click()`);
    const back = path.join(app.media, "peli-subtitulado-subtitulos.srt");
    for (let i = 0; i < 60 && !fs.existsSync(back); i++) await app.sleep(500);
    c.ok(fs.existsSync(back) && fs.readFileSync(back, "utf8").includes("Olá, Niño"), "o ficheiro saído está en UTF-8 co «ñ» ben");

    // ---- queimar
    await app.open(video);
    await app.click("Subtítulos");
    await app.click("Queimar");
    await app.waitFor(`typeof window.__trasnoSubs === 'function'`, "o selector de subtítulos");
    await app.ev(`window.__trasnoSubs(${JSON.stringify(srt)})`);
    await app.waitFor(`/2 falas/.test(document.querySelector('.stack').innerText)`, "a lectura dos subtítulos");
    await app.ev(`[...document.querySelectorAll('.stack .btn.primary')].find(b=>b.textContent.trim()==='Queimar').click()`);
    const burned = path.join(app.media, "peli-queimado.mp4");
    for (let i = 0; i < 180 && !fs.existsSync(burned); i++) await app.sleep(500);
    await app.sleep(500);
    c.ok(fs.existsSync(burned), "gardouse peli-queimado.mp4");
    c.ok(fs.existsSync(burned) && !probe(burned).some((s) => s.codec_type === "subtitle"), "queimados: non queda pista que apagar");
  },
};
