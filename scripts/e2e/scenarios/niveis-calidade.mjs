// Niveis de calidade con nome en Comprimir (imaxe e vídeo) e en Converter, sempre con «Personalizado»; e o remate lido polos lectores de pantalla.
const levels = (app) => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Nivel de calidade"] button')].map(b=>({t:b.innerText.split(String.fromCharCode(10)).join(' ').trim(), on:b.getAttribute('aria-checked')==='true'}))`);
const pick = async (app, name) => { const r = await app.ev(`(()=>{const b=[...document.querySelectorAll('.stack [role=radiogroup][aria-label="Nivel de calidade"] button')].find(b=>b.innerText.split(String.fromCharCode(10))[0].trim()===${JSON.stringify(name)}); if(!b) return 'sen '+${JSON.stringify(name)}; b.click(); return 'ok'})()`); if (r !== "ok") throw new Error(r); await app.sleep(300); };
const slider = (app, label) => app.ev(`Number([...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')===${JSON.stringify(label)})?.value)`);

export default {
  about: "Niveis de calidade (CRF e %), Personalizado en todos e o resultado dito polo trasno nunha rexión viva",
  async run(app, c) {
    // ---- imaxe PNG: os mesmos niveis ca en Converter (55, 75 e 95 %)
    const png = await app.textImage("niveis.png", ["Calidade"]);
    await app.open(png);
    await app.click("Comprimir");
    let l = await levels(app);
    c.ok(l.length === 4 && l[3].t === "Personalizado", "a imaxe ten tres niveis e Personalizado", l.map((x) => x.t).join(" | "));
    c.ok(l[0].t === "WhatsApp 55 %" && l[1].t === "Web 75 %" && l[2].t === "Arquivo 95 %", "cada botón leva a súa calidade (os mesmos 55, 75 e 95 % ca en Converter)", l.map((x) => x.t).join(" | "));
    c.ok(l[3].on, "o PNG sen perdas (100 %) non é de ningún nivel: Personalizado");
    await pick(app, "Web");
    c.near(await slider(app, "Calidade"), 75, 0.1, "escoller «Web» pon o desprazador en 75");
    l = await levels(app);
    c.ok(l[1].on, "e «Web» queda marcado");
    // Mentres se arrastra o desprazador a selección non cambia (sen parpadeos); ao soltalo, si.
    await app.ev(`(()=>{const i=[...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Calidade'); i.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true})); const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set; set.call(i,'95'); i.dispatchEvent(new Event('input',{bubbles:true}));})()`);
    await app.sleep(300);
    l = await levels(app);
    c.ok(l[1].on && !l[2].on, "ao pasar por 95 arrastrando, «Arquivo» aínda non se marca");
    await app.ev(`(()=>{const i=[...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')==='Calidade'); i.dispatchEvent(new Event('change',{bubbles:true}));})()`);
    await app.sleep(300);
    l = await levels(app);
    c.ok(l[2].on, "ao soltalo en 95, «Arquivo» queda marcado");
    await app.slider("Calidade", 70);
    l = await levels(app);
    c.ok(l[3].on && !l[1].on, "un valor que non é de ningún nivel (70) deixa marcado «Personalizado»");
    await pick(app, "Web");
    await pick(app, "Personalizado");
    l = await levels(app);
    c.ok(l[3].on && !l[1].on, "«Personalizado» queda marcado ao premelo aínda que os valores coincidan con «Web»");
    c.ok(!(await app.panel()).includes("Valores propios"), "Personalizado non leva texto de máis");

    // ---- vídeo: o CRF vai nos botóns
    await app.open(app.video("niveis.mp4", 3));
    await app.click("Converter");
    l = await levels(app);
    c.ok(l.length === 4 && l[0].t === "Alta CRF 23" && l[1].t === "Equilibrada CRF 28" && l[2].t === "Forte CRF 33" && l[3].t === "Personalizado", "o vídeo mostra Alta/Equilibrada/Forte co seu CRF", l.map((x) => x.t).join(" | "));
    c.ok(l[3].on, "o perfil por defecto (CRF 20) non coincide con ningún nivel: Personalizado");
    await pick(app, "Equilibrada");
    c.near(await slider(app, "Calidade (CRF)"), 28, 0.1, "«Equilibrada» pon o CRF en 28");
    l = await levels(app);
    c.ok(l[1].on, "e queda marcada");
    const loose = await app.ev(`[...document.querySelectorAll('.stack .group-note')].map(n=>n.textContent.trim())`);
    c.ok(loose.length === 0, "sen liñas soltas de explicación no perfil de vídeo", loose.join(" | "));

    // ---- o resultado di o trasno nunha rexión viva (lectores de pantalla)
    await app.click("Extraer");
    await app.click("Gardar este fotograma");
    await app.waitFor(`/./.test(document.querySelector('.sr-only[role=status]')?.textContent||'') || /./.test(document.querySelector('.sr-only[role=alert]')?.textContent||'')`, "o remate na rexión viva", 40000);
    const said = await app.ev(`(document.querySelector('.sr-only[role=status]')?.textContent||document.querySelector('.sr-only[role=alert]')?.textContent||'').trim()`);
    c.ok(said.length > 5 && !/O trasno /.test(said), "o trasno di o resultado en primeira persoa", said);

    // ---- Converter imaxe: mesmo selector, con Personalizado
    await app.open(await app.textImage("niveis2.jpg", ["Converter"]));
    await app.click("Converter");
    l = await levels(app);
    c.ok(l.length === 4 && l[3].t === "Personalizado" && l[0].t === "WhatsApp 55 %", "Converter ten os mesmos niveis e Personalizado", l.map((x) => x.t).join(" | "));
  },
};
