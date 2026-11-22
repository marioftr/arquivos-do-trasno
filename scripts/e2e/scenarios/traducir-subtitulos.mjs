// Subtítulos → Traducir: un SRT en inglés pasa a español co modelo Opus-MT (transformers.js no webview). Precisa rede (descarga uns 120 MB
// de Hugging Face a primeira vez): só corre con TRASNO_E2E_NET=1.
import fs from "node:fs";
import path from "node:path";

export default {
  about: "Subtítulos: traducir un SRT de inglés a español, con tempos e orixinal intactos",
  async run(app, c) {
    if (!process.env.TRASNO_E2E_NET) {
      c.ok(true, "sen TRASNO_E2E_NET=1: non se proba (descarga un modelo de internet)");
      return;
    }
    const srt = path.join(app.media, "proba.srt");
    const original = "1\n00:00:01,000 --> 00:00:03,500\nGood morning, how are you today?\n\n2\n00:00:04,000 --> 00:00:06,000\nI would like a cup of coffee, please.\n\n";
    fs.writeFileSync(srt, original);
    await app.open(srt);
    const tabs = await app.tabs();
    c.ok(tabs.length >= 2, "un SRT ten pestanas de ferramentas", tabs.join());
    await app.click(tabs[1]); // a segunda: axustar os subtítulos (resincronizar e traducir)
    await app.waitFor(`/2 falas/.test(document.querySelector('.stack')?.innerText ?? '')`, "a lectura dos subtítulos");
    const modes = await app.ev(`[...document.querySelectorAll('[role=radio]')].map(b=>b.textContent.trim())`);
    c.ok(modes.includes("Resincronizar") && modes.includes("Traducir"), "hai dous modos: Resincronizar e Traducir", modes.join(","));
    await app.click("Traducir");
    c.ok(await app.ev(`!!document.querySelector('.stack')?.innerText.includes('Está en')`), "no modo Traducir pídense os dous idiomas");
    // Os botóns, en orde: Resincronizar, Traducir, «Está en» (Galego, Español, English, Català) e «Traducir a» (os mesmos catro).
    await app.ev(`(()=>{ const r=[...document.querySelectorAll('.stack [role=radio]')]; r[4].click(); r[7].click(); })()`);
    await app.sleep(300);
    await app.ev(`[...document.querySelectorAll('.stack .btn.primary')].find(b=>b.textContent.trim()==='Traducir')?.click()`);
    const out = path.join(app.media, "proba-es.srt");
    await app.waitFor(`/Feito|Vaites|Listo|Todo en orde/.test(document.body.innerText)`, "o remate", 300000);
    c.ok(fs.existsSync(out), "gardouse proba-es.srt ao carón do orixinal");
    const text = fs.existsSync(out) ? fs.readFileSync(out, "utf8") : "";
    c.ok(/buenos d[ií]as/i.test(text) && /caf[ée]/i.test(text), "a tradución sae en español", text.replace(/\n/g, " | "));
    c.ok(text.includes("00:00:01,000 --> 00:00:03,500") && text.includes("00:00:04,000 --> 00:00:06,000"), "cos mesmos tempos");
    c.ok(fs.readFileSync(srt, "utf8") === original, "e o orixinal queda igual");
  },
};
