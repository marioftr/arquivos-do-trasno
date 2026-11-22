// Descargar por ligazón con listas: unha lista de verdade (só se le, sen descargar nada) amósase enteira e deixa marcar vídeos; e a descarga
// por lotes (un vídeo bo e outro que falla) segue co resto e di o que non puido. Precisa a caché de extensións (TRASNO_E2E_TOOLS) e rede.
import fs from "node:fs";
import path from "node:path";

export default {
  about: "Descargar por ligazón: listas enteiras, marcar vídeos e descarga por lotes",
  async run(app, c) {
    if (!app.useTools()) {
      c.ok(true, "sen caché de extensións (TRASNO_E2E_TOOLS): non se proba");
      return;
    }
    await app.reload();
    await app.radial("Descargar por ligazón");
    const list = process.env.TRASNO_E2E_LIST ?? "https://www.youtube.com/playlist?list=PLMC9KNkIncKtPzgY-5rmhvj7fax8fdxoj";
    await app.ev(`(()=>{const i=document.querySelector('[role=dialog] input[type=text]'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i, ${JSON.stringify(list)}); i.dispatchEvent(new Event('input',{bubbles:true}));})()`);
    await app.sleep(1500);
    c.ok(!(await app.ev(`!!document.querySelector('[data-testid=media-info], [data-testid=playlist]')`)) && !(await app.ev(`document.body.innerText.includes('Comprobando')`)), "escribir a ligazón non a busca soa");
    await app.ev(`document.querySelector('[data-testid=download-search]')?.click()`);
    await app.waitFor(`document.querySelector('[role=dialog] [data-testid=playlist], [role=dialog] .warn')`, "ler a lista", 120000);
    // A lista vai nunha columna á dereita, a toda a altura, sen que a xanela sexa máis alta que a pantalla.
    c.ok(await app.ev(`(()=>{const a=document.querySelector('[data-testid=media-info]').getBoundingClientRect(), b=document.querySelector('[data-testid=playlist]').getBoundingClientRect(); return b.left >= a.right && b.top <= a.top + 40})()`), "a lista queda nunha columna á dereita dos detalles");
    const header = () => app.ev(`document.querySelector('[data-testid=playlist] .all')?.innerText.replace(/\\s+/g,' ').trim() ?? ''`);
    const rows = await app.ev(`document.querySelectorAll('[data-testid=playlist] li').length`);
    c.ok(rows >= 10, "a lista amósase enteira, cos seus vídeos", String(rows));
    const first = await header();
    c.ok(new RegExp(`^${rows} de ${rows} vídeos$`).test(first), "ao principio van todos marcados", first);
    const go = () => app.ev(`document.querySelector('[data-testid=download-go]')?.innerText.trim()`);
    c.ok((await go()) === `Descargar ${rows} vídeos`, "e o botón di cantos descarga", await go());
    await app.ev(`document.querySelector('[data-testid=playlist] .all input')?.click()`);
    await app.sleep(200);
    c.ok(/^0 de /.test(await header()) && (await app.ev(`document.querySelector('[data-testid=download-go]')?.disabled`)) === true, "desmarcar todos deixa o botón desactivado");
    await app.ev(`(()=>{const l=[...document.querySelectorAll('[data-testid=playlist] li input')]; l[0].click(); l[2].click();})()`);
    await app.sleep(200);
    c.ok(/^2 de /.test(await header()) && (await go()) === "Descargar 2 vídeos", "marcar dous vídeos a man", `${await header()} / ${await go()}`);
    c.ok(await app.ev(`document.querySelector('[data-testid=playlist] ul').scrollHeight > 0 && document.querySelector('.box').getBoundingClientRect().height <= window.innerHeight`), "a xanela non desborda");
    await app.shot("descarga-lista.png");
    await app.click("Pechar");

    // ---- por lotes: un vídeo que existe e outro que non
    const good = "https://www.youtube.com/watch?v=jNQXAC9IVRw";
    const bad = "https://www.youtube.com/watch?v=aaaaaaaaaaa";
    const r = await app.ev(`window.__TAURI_INTERNALS__.invoke('ytdlp_download', { urls: [${JSON.stringify(bad)}, ${JSON.stringify(good)}], dir: ${JSON.stringify(app.media)}, options: { kind: 'audio', max_height: null, container: 'mp4', audio_format: 'mp3', audio_kbps: 128 } })`);
    c.ok(r?.done?.length === 1 && r?.failed?.length === 1, "o que falla non para os demais e queda anotado", JSON.stringify(r));
    c.ok(r?.failed?.[0]?.url === bad && !!r?.failed?.[0]?.detail, "dicindo cal e por que");
    const got = fs.readdirSync(app.media).find((n) => /Me at the zoo.*\.mp3$/.test(n));
    c.ok(!!got && fs.statSync(path.join(app.media, got)).size > 10_000, "o vídeo bo descargouse", fs.readdirSync(app.media).join(", "));
    c.ok(!fs.readdirSync(app.media).some((n) => n.startsWith(".trasno-descarga")), "sen cartafoles temporais");
    const only = await app.ev(`window.__TAURI_INTERNALS__.invoke('ytdlp_download', { urls: [${JSON.stringify(bad)}], dir: ${JSON.stringify(app.media)}, options: { kind: 'audio', max_height: null, container: 'mp4', audio_format: 'mp3', audio_kbps: null } }).then(()=>'ok', (e)=>String(e?.message ?? e))`);
    c.ok(only !== "ok" && only.length > 3, "se só hai un e falla, dá erro en vez de fingir que foi ben", only);
  },
};
