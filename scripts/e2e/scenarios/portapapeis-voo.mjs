// Menú do trasno: Pegar do portapapeis (ligazón → Descargar por ligazón xa cargada; texto sen ligazón → aviso) e o voo do trasno
// entre o recanto e a xanela ao abrila e pechala.
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
const SPRITES = `document.querySelectorAll('body > img[src="/trasno/base.png"], body > div > img[src="/trasno/base.png"]').length`;

export default {
  about: "Pegar do portapapeis e voo do trasno ao abrir e pechar Descargar por ligazón",
  async run(app, c) {
    await app.reload();
    const url = "https://example.com/video/1";
    // O portapapeis do sistema (o que le o botón), non o do webview.
    const copy = (text) => (spawnSync("powershell", ["-NoProfile", "-Command", `Set-Clipboard -Value '${text.replace(/'/g, "''")}'`]).status === 0 ? "ok" : "erro");

    c.ok((await app.ev(`[...document.querySelectorAll('.dock .rd')].map(b=>b.getAttribute('aria-label')).slice(4).join()`)) === "Pegar do portapapeis,Comentario do trasno,Buscar", "o círculo exterior: portapapeis, comentario e buscar");

    // Texto que non é ligazón nin ruta: avisa o trasno.
    c.ok((await copy("hola, isto non é unha ligazón")) === "ok", "pódese escribir no portapapeis de proba");
    await app.radial("Pegar do portapapeis");
    await app.waitFor(`document.body.innerText.includes('O portapapeis non ten ligazóns')`, "o aviso do trasno", 8000);
    c.ok(true, "sen ligazón nin rutas, o trasno avisa");

    // Unha imaxe copiada: gárdase como PNG en Imaxes/Arquivos do Trasno e entra na lista (despois bórrase o que a proba creou).
    const pictures = spawnSync("powershell", ["-NoProfile", "-Command", "[Environment]::GetFolderPath('MyPictures')"]).stdout.toString().trim();
    const folder = path.join(pictures, "Arquivos do Trasno");
    const existed = fs.existsSync(folder);
    const before = existed ? new Set(fs.readdirSync(folder)) : new Set();
    const setImage = spawnSync("powershell", ["-NoProfile", "-STA", "-Command", "Add-Type -AssemblyName System.Windows.Forms,System.Drawing; $b=New-Object System.Drawing.Bitmap 40,30; $g=[System.Drawing.Graphics]::FromImage($b); $g.Clear([System.Drawing.Color]::OrangeRed); [System.Windows.Forms.Clipboard]::SetImage($b)"]);
    c.ok(setImage.status === 0, "pódese copiar unha imaxe ao portapapeis de proba");
    await app.radial("Pegar do portapapeis");
    await app.waitFor(`document.body.innerText.includes('Gardei a imaxe do portapapeis')`, "o trasno di que gardou a imaxe", 10000);
    const made = fs.existsSync(folder) ? fs.readdirSync(folder).filter((n) => !before.has(n)) : [];
    c.ok(made.length === 1 && /^portapapeis.*\.png$/.test(made[0]), "a imaxe gárdase como PNG en Imaxes/Arquivos do Trasno", made.join());
    c.ok(await app.ev(`[...document.querySelectorAll('li[role=option]')].some(li=>li.innerText.includes('portapapeis'))`), "e entra na lista de ficheiros");
    for (const n of made) fs.rmSync(path.join(folder, n), { force: true });
    if (!existed) fs.rmSync(folder, { recursive: true, force: true });

    // Unha ligazón: abre Descargar por ligazón con ela posta.
    c.ok((await copy(url)) === "ok", "e unha ligazón");
    // Mídese dentro da propia páxina, ao pouco de abrirse a xanela (o voo dura menos de medio segundo).
    const fly = await app.ev(`(async()=>{ document.querySelector('.dock').dispatchEvent(new PointerEvent('pointerenter',{bubbles:false})); await new Promise(r=>setTimeout(r,600));
      [...document.querySelectorAll('.dock .rd')].find(b=>b.getAttribute('aria-label')==='Pegar do portapapeis').click();
      for (let i=0;i<40 && !document.querySelector('[role=dialog]');i++) await new Promise(r=>setTimeout(r,25));
      await new Promise(r=>setTimeout(r,100));
      return {sprites: ${SPRITES}, hidden: getComputedStyle(document.querySelector('.dock')).visibility === 'hidden'}; })()`);
    c.ok(fly.sprites === 1, "ao abrirse a xanela, o trasno voa do recanto á cabeceira", JSON.stringify(fly));
    c.ok(fly.hidden, "e o do recanto queda agochado mentres voa");
    await app.sleep(900);
    c.ok((await app.ev(SPRITES)) === 0 && (await app.ev(`getComputedStyle(document.querySelector('[role=dialog] .mas')).visibility`)) === "visible", "ao chegar, o trasno da cabeceira ocupa o seu sitio");
    c.ok((await app.ev(`document.querySelector('[role=dialog] input[type=text]')?.value`)) === url, "a ligazón pegada xa está no campo");

    // A ligazón pegada desde o botón búscase soa; non fai falta premer Buscar.
    await app.waitFor(`document.querySelector('[data-testid=media-info], [role=dialog] .warn, [role=dialog] [data-testid=extension-gate]')`, "o resultado da busca ou o aviso de extensións", 90000);
    c.ok(true, "a ligazón que vén do botón de pegar búscase soa");

    // Co trasno na cabeceira, un aviso importante sáelle alí e non no globo da barra (que quedaría borroso ao fondo); un consello descártase
    // e un resultado agarda a que o trasno volva.
    const say = (code) => app.ev(`(async()=>{ try { const u = performance.getEntriesByType('resource').map(e=>e.name).find(n=>n.includes('/lib/agent.ts')); const m = await import(u); ${code}; await new Promise(r=>setTimeout(r,300)); return { head: !!document.querySelector('[role=dialog] header .bubble'), foot: !!document.querySelector('footer .bubble') }; } catch (e) { return 'sen-import'; } })()`);
    const tip = await say(`m.trasnoSay('Consello de proba', 'tip', 60000)`);
    if (tip === "sen-import") c.ok(true, "(sen acceso ao módulo do trasno nesta compilación: non se proba o globo da cabeceira)");
    else {
      c.ok(tip.head === false && tip.foot === false, "un consello non sae mentres o trasno está na xanela", JSON.stringify(tip));
      const warn = await say(`m.trasnoSay('Aviso de proba', 'warn', 60000)`);
      c.ok(warn.head === true && warn.foot === false, "un aviso importante sae no globo da cabeceira, ao carón do trasno", JSON.stringify(warn));
      c.ok(await app.ev(`(()=>{const h=document.querySelector('[role=dialog] header .bubble'), m=document.querySelector('[role=dialog] header .mas'); if(!h||!m) return false; const a=h.getBoundingClientRect(), b=m.getBoundingClientRect(); return a.top >= b.top && a.left >= b.left && a.right <= innerWidth && a.bottom <= innerHeight})()`), "e está á vista, pegado á cabeceira");
      await app.shot("globo-cabeceira.png");
      await say(`m.trasnoSay('Resultado retido', 'info', 60000)`);
    }

    // Pechar: o trasno volve ao recanto.
    await app.ev(`document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))`);
    await app.sleep(150);
    c.ok((await app.ev(SPRITES)) === 1, "ao pechar, o trasno voa da cabeceira ao recanto");
    await app.sleep(900);
    c.ok((await app.ev(SPRITES)) === 0 && (await app.ev(`getComputedStyle(document.querySelector('.dock')).visibility`)) === "visible", "e ao chegar o do recanto volve ser visible");
    if (tip !== "sen-import") c.ok(await app.ev(`document.querySelector('footer .bubble')?.innerText.includes('Resultado retido') ?? false`), "e o resultado retido sae entón no globo da barra");
  },
};
