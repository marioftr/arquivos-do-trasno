// Extensións opcionais sen descargar: as tarxetas de Axustes, o aviso de descarga en Transcribir, Quitar fondo e Descargar por ligazón
// (en vez da ferramenta), a descarga real de yt-dlp desde o aviso e o círculo exterior do menú do trasno. Precisa rede para descargar yt-dlp (24 MB).
import fs from "node:fs";
import path from "node:path";

export default {
  about: "Extensións opcionais: Axustes, avisos de descarga nas ferramentas, descarga de yt-dlp e menú do trasno",
  async run(app, c) {
    app.noTools();
    await app.reload();
    const gate = () => app.ev(`document.querySelector('[data-testid=extension-gate]')?.innerText.replace(/\\s+/g,' ').trim() ?? ''`);
    const green = (text) => app.ev(`[...document.querySelectorAll('.btn.primary.block')].some(b=>b.textContent.trim()===${JSON.stringify(text)})`);

    // ---- Axustes → Extensións
    // O menú do trasno con segundo círculo (a captura serve para revisalo á vista).
    await app.ev(`document.querySelector('.dock')?.dispatchEvent(new PointerEvent('pointerenter',{bubbles:false}))`);
    await app.sleep(700);
    await app.ev(`[...document.querySelectorAll('.dock .rd')].find(b=>b.getAttribute('aria-label')==='Descargar por ligazón')?.dispatchEvent(new PointerEvent('pointerenter',{bubbles:false}))`);
    await app.sleep(400);
    await app.shot("ext-radial.png");
    c.ok(await app.ev(`[...document.querySelectorAll('.dock .rd')].map(b=>b.getAttribute('aria-label')).join()`) === "Engadir ficheiros,Descargar por ligazón,Procesos,Axustes,Pegar do portapapeis,Comentario do trasno,Buscar", "o menú ten dous círculos: Engadir, Ligazón, Procesos e Axustes por dentro; Pegar, Comentario e Buscar por fóra");
    await app.radial("Axustes");
    await app.click("Extensións");
    await app.shot("ext-axustes.png");
    const page = await app.ev(`document.body.innerText`);
    c.ok(/whisper\.cpp/.test(page) && /yt-dlp/.test(page), "Axustes → Extensións lista whisper.cpp e yt-dlp");
    c.ok(/Modelos de recoñecemento de voz/.test(page) && /Rápida/.test(page) && /Equilibrada/.test(page) && /Precisa/.test(page), "e os tres modelos de voz");
    c.ok(/IS-Net/.test(page) && !/Modelo para quitar fondos/.test(page), "e o modelo de quitar fondos co seu nome");
    await app.waitFor(`![...document.querySelectorAll('.card .state')].some(x=>x.textContent.includes('Comprobando'))`, "que Office acabe de comprobarse", 30000);
    const cards = await app.ev(`[...document.querySelectorAll('.card')].filter(k=>k.querySelector('.head .state')).map(k=>({ name: k.querySelector('.head strong').textContent.trim(), state: k.querySelector('.head .state').textContent.trim(), meta: k.querySelector('.meta')?.innerText.replace(/\\s+/g,' ') ?? '' }))`);
    c.ok(cards.filter((k) => k.state !== "Sempre dispoñible").every((k) => ["Instalado", "Sen instalar"].includes(k.state)), "todas as tarxetas dicen «Instalado» ou «Sen instalar», sen outras palabras", JSON.stringify(cards.map((k) => k.state)));
    const withMeta = cards.filter((k) => /\d+ MB/.test(k.meta) && /github\.com|gitlab\.com/.test(k.meta));
    c.ok(withMeta.length >= 5, "cada extensión di canto ocupa e leva a ligazón ao seu proxecto", JSON.stringify(cards.map((k) => k.meta)));
    c.ok(cards.some((k) => k.name === "yt-dlp" && /24 MB/.test(k.meta)) && cards.some((k) => k.name === "IS-Net" && /170 MB/.test(k.meta)), "yt-dlp e IS-Net tamén levan o tamaño");
    c.ok(!(await app.ev(`document.body.innerText`)).includes("Descargado\n"), "ningunha tarxeta usa «Descargado» no canto de «Instalado»");
    await app.click("Pechar");

    // ---- Transcribir: sen whisper.cpp nin modelo sae o aviso en vez do botón
    const video = app.video("a.mp4", 3);
    await app.open(video);
    c.ok((await app.tabs()).includes("Transcribir"), "os vídeos teñen a pestana Transcribir", (await app.tabs()).join(","));
    await app.click("Transcribir");
    let g = await gate();
    c.ok(/whisper\.cpp/.test(g) && /Whisper small/.test(g) && /Falta instalar/.test(g), "Transcribir pide whisper.cpp e o modelo elixido", g);
    c.ok(!(await green("Transcribir")), "sen as extensións non hai botón verde");
    await app.ev(`[...document.querySelectorAll('.stack [role=radio]')].find(b=>b.innerText.startsWith('Rápida'))?.click()`);
    await app.sleep(300);
    g = await gate();
    c.ok(/Whisper base/.test(g), "o aviso segue ao modelo que se escolle", g);
    c.ok(/Descargar e instalar · \d+ MB/.test(await app.ev(`document.querySelector('[data-testid=extension-gate] .btn.primary')?.textContent.replace(/\\s+/g,' ').trim() ?? ''`)), "o botón do aviso di canto se descarga en total");
    await app.shot("ext-transcribir.png");
    c.ok(await app.ev(`[...document.querySelectorAll('.stack .field .lab')].some(l=>l.textContent.includes('Idioma da voz'))`), "os axustes seguen á vista");
    c.ok(await app.ev(`!!document.querySelector('header .tt button.tip')`), "a descrición vai na «i» do título do panel");

    // ---- Quitar fondo
    const img = await app.textImage("fondo.png", ["Ola"]);
    await app.open(img);
    c.ok((await app.tabs()).includes("Quitar fondo"), "as imaxes teñen a pestana Quitar fondo", (await app.tabs()).join(","));
    await app.click("Quitar fondo");
    g = await gate();
    c.ok(/IS-Net/.test(g) && !/Modelo para quitar fondos/.test(g), "Quitar fondo pide o modelo co seu nome", g);
    c.ok(!(await app.ev(`!!document.querySelector('.stack [role=radiogroup]')`)), "sen o modelo, só sae o aviso e non os axustes");
    await app.shot("ext-fondo.png");

    // ---- Descargar por ligazón: círculo exterior do trasno
    await app.radial("Descargar por ligazón");
    c.ok(await app.ev(`document.querySelector('[role=dialog] h2')?.textContent.trim() === 'Descargar por ligazón'`), "o círculo exterior abre Descargar por ligazón");
    const dlgGate = () => app.ev(`document.querySelector('[role=dialog] [data-testid=extension-gate]')?.innerText.replace(/\s+/g,' ').trim() ?? ''`);
    g = await dlgGate();
    c.ok(/yt-dlp/.test(g) && !/Modelo/.test(g), "pide yt-dlp antes de nada", g);
    await app.shot("ext-dialogo.png");
    await app.ev(`[...document.querySelectorAll('[role=dialog] [data-testid=extension-gate] .btn.primary')].find(b=>b.textContent.trim().startsWith('Descargar e instalar'))?.click()`);
    await app.waitFor(`!document.querySelector('[role=dialog] [data-testid=extension-gate]')`, "a descarga de yt-dlp", 180000);
    const exe = path.join(app.root, "lad", "ArquivosDoTrasno", "tools", "yt-dlp", "yt-dlp.exe");
    c.ok(fs.existsSync(exe) && fs.statSync(exe).size > 10_000_000, "yt-dlp quedou instalado na carpeta da app");
    c.ok(await app.ev(`[...document.querySelectorAll('[role=dialog] .btn.primary.block')].find(b=>b.textContent.trim()==='Descargar')?.disabled === true`), "e sae o botón verde, desactivado mentres non hai ligazón");
    await app.click("Pechar");
  },
};
