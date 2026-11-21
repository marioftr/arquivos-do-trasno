// PDF: Markdown, táboas, índice, limpar, tapar e formularios na app real, cun PDF con títulos, lista, táboa, correo e campos.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");

/** PDF feito á man: `objs` son os corpos dos obxectos desde o 1; o último é o catálogo. */
function build(objs) {
  let out = "%PDF-1.5\n";
  const off = [];
  objs.forEach((o, i) => {
    off.push(out.length);
    out += `${i + 1} 0 obj\n${o}\nendobj\n`;
  });
  const xref = out.length;
  out += `xref\n0 ${objs.length + 1}\n0000000000 65535 f \n${off.map((x) => String(x).padStart(10, "0") + " 00000 n \n").join("")}`;
  return out + `trailer\n<< /Size ${objs.length + 1} /Root ${objs.length} 0 R /Info 3 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
}
const stream = (s, extra = "") => `<< /Length ${s.length} ${extra} >>\nstream\n${s}\nendstream`;
const t = (size, x, y, txt) => `BT /F1 ${size} Tf ${x} ${y} Td (${txt}) Tj ET\n`;

function report(font = "Helvetica") {
  const p1 =
    t(24, 50, 780, "Informe anual") +
    t(18, 50, 740, "Resumo") +
    t(11, 50, 715, "Este documento resume o ano que pasou.") +
    t(11, 50, 700, "Contacto: ana@example.com e telefono 600 123 456.") +
    t(11, 50, 685, "Mais texto para que o corpo sexa o maior grupo.") +
    t(11, 50, 670, "Outra liña de texto normal do informe.") +
    t(11, 50, 650, "- Primeiro punto") +
    t(11, 50, 635, "- Segundo punto") +
    [["Nome", "Idade", "Cidade"], ["Ana", "31", "Vigo"], ["Brais", "28", "Lugo"], ["Carme", "45", "Ourense"]].map((r, i) => r.map((c, k) => t(11, 50 + k * 150, 590 - i * 16, c)).join("")).join("");
  const p2 = t(18, 50, 780, "Anexo") + t(11, 50, 750, "Datos finais do informe nunha segunda pagina.") + t(11, 50, 735, "Segredo da pagina dous.");
  return build([
    "<< /Type /Pages /Kids [4 0 R 6 0 R] /Count 2 >>", // 1
    `<< /Type /Font /Subtype /Type1 /BaseFont /${font} >>`, // 2
    "<< /Author (Marta Autora) /Title (Informe) >>", // 3
    "<< /Type /Page /Parent 1 0 R /MediaBox [0 0 595 842] /Contents 5 0 R /Resources << /Font << /F1 2 0 R >> >> >>", // 4
    stream(p1), // 5
    "<< /Type /Page /Parent 1 0 R /MediaBox [0 0 595 842] /Contents 7 0 R /Resources << /Font << /F1 2 0 R >> >> >>", // 6
    stream(p2), // 7
    "<< /Type /Catalog /Pages 1 0 R >>", // 8
  ]);
}

function formPdf() {
  return build([
    "<< /Type /Pages /Kids [4 0 R] /Count 1 >>", // 1
    "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>", // 2
    "<< /Title (Formulario) >>", // 3
    "<< /Type /Page /Parent 1 0 R /MediaBox [0 0 595 842] /Contents 5 0 R /Resources << /Font << /F1 2 0 R >> >> /Annots [6 0 R 7 0 R] >>", // 4
    stream(t(16, 50, 780, "Solicitude")), // 5
    "<< /Type /Annot /Subtype /Widget /FT /Tx /T (nome) /V () /Rect [50 700 300 722] /DA (/F1 0 Tf 0 g) /P 4 0 R >>", // 6
    "<< /Type /Annot /Subtype /Widget /FT /Btn /T (acepto) /V /Off /AS /Off /Rect [50 660 62 672] /AP << /N << /Yes 8 0 R >> >> /P 4 0 R >>", // 7
    stream("0 0 12 12 re S", "/Type /XObject /Subtype /Form /BBox [0 0 12 12]"), // 8
    "<< /Type /Catalog /Pages 1 0 R /AcroForm << /Fields [6 0 R 7 0 R] /DA (/F1 0 Tf 0 g) >> >>", // 9
  ]);
}

export default {
  about: "PDF: Markdown, táboas, índice, limpar, tapar e formularios con ficheiros novos e orixinal intacto",
  async run(app, c) {
    const pdf = path.join(app.media, "informe.pdf");
    fs.writeFileSync(pdf, report(), "latin1");
    const before = sha(pdf);
    const read = (n) => fs.readFileSync(path.join(app.media, n), "latin1");
    const exists = async (n) => {
      for (let i = 0; i < 60 && !fs.existsSync(path.join(app.media, n)); i++) await app.sleep(300);
      return fs.existsSync(path.join(app.media, n));
    };
    const mode = (name) => app.ev(`(()=>{const b=[...document.querySelectorAll('.modes [role=radio], [role=radiogroup] button')].find(b=>b.getAttribute('aria-label')===${JSON.stringify(name)}||b.innerText.split(String.fromCharCode(10))[0].trim()===${JSON.stringify(name)}); if(!b) return 'sen '+${JSON.stringify(name)}; b.click(); return 'ok'})()`);
    const row = (label) => app.ev(`[...document.querySelectorAll('.stack .results li')].find(li=>li.querySelector('.rname')?.textContent.trim()===${JSON.stringify(label)})?.querySelector('.rstate')?.textContent.replace(/\\s+/g,' ').trim()`);

    // Os paneis de PDF cargan á demanda: agárdase polo botón antes de premelo.
    const go = async (name) => {
      await app.waitFor(`[...document.querySelectorAll('.stack button')].some(b=>b.textContent.trim()===${JSON.stringify(name)}&&!b.disabled)`, "o botón «" + name + "»", 20000);
      await app.click(name);
    };
    await app.open(pdf);
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    const tabs = await app.tabs();
    c.ok(tabs.includes("Completar") && tabs.includes("Seguridade"), "o PDF ten as pestanas Completar e Seguridade", tabs.join(","));
    c.ok(!tabs.includes("Formulario") && !tabs.includes("Anexos"), "Formulario e Anexos xa non son pestanas soltas", tabs.join(","));
    c.ok(tabs.indexOf("Completar") < tabs.indexOf("Marcas"), "Completar vai por riba de Marcas", tabs.join(","));
    c.ok(tabs.indexOf("Recoñecer texto (OCR)") === tabs.length - 2 && tabs[tabs.length - 1] === "Información", "Recoñecer texto vai xusto antes de Información", tabs.join(","));

    // ---- Markdown
    await app.click("Converter");
    await app.waitFor(`!!document.querySelector('.stack select[aria-label="Formato de saída"]')`, "o selector de formato de saída", 20000);
    const fmts = await app.ev(`[...document.querySelectorAll('.stack select[aria-label="Formato de saída"] option')].map(o=>o.textContent.trim()).slice(0,4).join()`);
    c.ok(fmts === "PNG,JPG,TXT,Markdown", "un só selector de formato: imaxes, texto e Markdown", fmts);
    c.ok(!(await app.panel()).includes("A Markdown"), "xa non hai modos separados para Markdown nin «A outro formato»");
    await app.ev(`(()=>{const s=document.querySelector('.stack select[aria-label="Formato de saída"]'); s.value='md'; s.dispatchEvent(new Event('change',{bubbles:true}));})()`);
    await go("Gardar como Markdown");
    c.ok(await exists("informe.md"), "gardouse informe.md ao carón");
    const md = fs.readFileSync(path.join(app.media, "informe.md"), "utf8");
    c.ok(/^# Informe anual$/m.test(md) && /^## Resumo$/m.test(md), "os títulos saen pola letra: # e ##", md.slice(0, 120));
    c.ok(/^- Primeiro punto\n- Segundo punto$/m.test(md), "a lista queda seguida", md);
    c.ok(/\| Nome \| Idade \| Cidade \|\n\| --- \| --- \| --- \|\n\| Ana \| 31 \| Vigo \|/.test(md), "a táboa sae como táboa de Markdown", md);
    c.ok(/Este documento resume o ano que pasou\./.test(md) && /# Anexo/.test(md), "o texto e a segunda páxina están");
    c.ok(sha(pdf) === before, "o PDF orixinal queda igual, byte a byte");

    // ---- Táboas
    c.ok((await mode("Táboas")) === "ok", "Converter ten o modo «Táboas»");
    await go("Buscar táboas");
    await app.waitFor(`document.querySelectorAll('.tbls li').length === 1`, "unha táboa atopada", 20000);
    c.ok(/4 × 3/.test(await app.ev(`document.querySelector('.tbls .head').innerText.replace(/\\s+/g,' ')`)), "4 filas × 3 columnas");
    await go("Gardar a táboa en CSV");
    c.ok(await exists("informe-taboa.csv"), "gardouse informe-taboa.csv");
    const csv = fs.readFileSync(path.join(app.media, "informe-taboa.csv"), "utf8");
    c.ok(csv.charCodeAt(0) === 0xfeff && csv.includes("Nome;Idade;Cidade\r\nAna;31;Vigo\r\n"), "CSV con BOM e punto e coma", JSON.stringify(csv));

    // ---- Índice
    await app.click("Páxinas");
    c.ok((await mode("Índice")) !== "ok", "Páxinas xa non ten o modo «Índice»");
    await app.click("Completar");
    c.ok((await mode("Índice")) === "ok", "Completar ten o modo «Índice»");
    await app.waitFor(`/Este PDF non ten índice/.test(document.body.innerText)`, "sen índice de partida");
    await go("Detectar títulos");
    await app.waitFor(`document.querySelectorAll('.outline li').length >= 3`, "títulos detectados", 20000);
    const titles = await app.ev(`[...document.querySelectorAll('.outline .otitle')].map(i=>i.value+'@'+i.closest('li').querySelector('.opage').value+'/'+parseFloat(i.style.marginLeft||0)).join(' | ')`);
    c.ok(/Informe anual@1\/0/.test(titles) && /Resumo@1\//.test(titles) && /Anexo@2\//.test(titles), "Informe anual, Resumo e Anexo coas súas páxinas", titles);
    // «Resumo» pasa a fillo de «Informe anual» (a letra menor): nivel 1.
    c.ok(/Resumo@1\/0\.8/.test(titles) || /Resumo@1\/1\.6/.test(titles), "o nivel sae do tamaño da letra", titles);
    if (process.env.TRASNO_SHOTS) await app.shot("indice.png");
    await go("Gardar unha copia co índice");
    c.ok(await exists("informe-con-indice.pdf"), "gardouse informe-con-indice.pdf");
    const withIdx = read("informe-con-indice.pdf");
    c.ok(/\/Outlines/.test(withIdx) && withIdx.includes("Resumo"), "o PDF novo leva o índice");
    // Ao abrir o PDF novo, o índice sae xa cargado.
    await app.open(path.join(app.media, "informe-con-indice.pdf"));
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    await app.click("Completar");
    await mode("Índice");
    await app.waitFor(`document.querySelectorAll('.outline li').length >= 3`, "o índice que xa levaba", 20000);
    c.ok(true, "ao abrir o PDF co índice, as entradas aparecen para editalas");

    // ---- Limpar
    await app.open(pdf);
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    await app.click("Seguridade");
    c.ok((await mode("Limpar")) === "ok", "Seguridade ten o modo «Limpar»");
    await go("Limpar o PDF");
    c.ok(await exists("informe-limpo.pdf"), "gardouse informe-limpo.pdf");
    c.ok(!read("informe-limpo.pdf").includes("Marta Autora"), "o autor xa non está no ficheiro");
    c.ok(/quitados: \d+/.test((await row("Datos do documento")) ?? ""), "a táboa di cantos datos quitou", await row("Datos do documento"));
    c.ok(read("informe.pdf").includes("Marta Autora"), "o orixinal segue co autor");

    // ---- Recortar: ao baixar as marxes a 0 non queda sombra
    await app.click("Xirar e recortar");
    await mode("Recortar");
    await app.waitFor(`document.querySelector('.stack input[role=switch][aria-label="Os catro márxenes iguais"]')`, "o recorte", 15000);
    await app.toggle("Os catro márxenes iguais");
    await app.slider("Marxe", 25);
    await app.sleep(400);
    const shade = () => app.ev(`[...document.querySelectorAll('.pages .pg div')].filter(d=>d.style.background.startsWith('rgba(10, 10, 14')).length`);
    c.ok((await shade()) >= 4, "con marxe hai sombra nos catro lados", String(await shade()));
    await app.slider("Marxe", 0);
    await app.sleep(500);
    c.ok((await shade()) === 0, "ao volver a 0 non queda sombra en ningún lado", String(await shade()));

    // ---- Anexos: pestana propia (non vai dentro de Seguridade)
    c.ok((await mode("Anexos")) !== "ok", "Seguridade xa non ten o modo «Anexos»");
    await app.click("Completar");
    c.ok((await mode("Anexos")) === "ok", "Completar ten o modo «Anexos»");
    await app.waitFor(`/non ten anexos/.test(document.body.innerText)`, "sen anexos de partida", 15000);
    c.ok(true, "o PDF de partida non ten anexos");
    await app.click("Seguridade");

    // ---- Tapar
    c.ok((await mode("Tapar")) === "ok", "Seguridade ten o modo «Tapar»");
    await app.waitFor(`/zonas a tapar/i.test(document.body.innerText)`, "o panel de tapar");
    await go("Marcar coincidencias");
    await app.waitFor(`document.querySelectorAll('.stack .results li').length === 1`, "unha páxina con zonas", 30000);
    c.ok(/Zonas: 1/.test((await row("Páxina 1")) ?? ""), "o correo dá unha zona na páxina 1", await row("Páxina 1"));
    // Debuxar outra zona co rato de verdade sobre a páxina 2 (onde está «Segredo da pagina dous»).
    await app.ev(`document.querySelector('.pages .pg:nth-child(2)')?.scrollIntoView()`);
    await app.sleep(600);
    const pg = await app.ev(`(()=>{const r=document.querySelector('.pages .pg:nth-child(2)').getBoundingClientRect(); return [r.left,r.top,r.width,r.height];})()`);
    await app.drag([pg[0] + pg[2] * 0.05, pg[1] + pg[3] * 0.06], [pg[0] + pg[2] * 0.7, pg[1] + pg[3] * 0.11]);
    await app.waitFor(`document.querySelectorAll('.stack .results li').length === 2`, "unha zona debuxada co rato", 10000);
    c.ok(/Zonas: 1/.test((await row("Páxina 2")) ?? ""), "arrastrar sobre a páxina 2 engade unha zona", await row("Páxina 2"));
    // A zona da páxina 2 móvese arrastrándoa e a súa × quítaa (antes non respondían).
    const box = () => app.ev(`(()=>{const b=document.querySelectorAll('.pages .pg:nth-child(2) .ov')[0]; if(!b) return null; const r=b.getBoundingClientRect(); return [r.left,r.top,r.width,r.height];})()`);
    const b0 = await box();
    await app.drag([b0[0] + b0[2] / 2, b0[1] + b0[3] / 2], [b0[0] + b0[2] / 2 + 60, b0[1] + b0[3] / 2 + 80]);
    await app.sleep(400);
    const b1 = await box();
    c.ok(b1 && b1[0] - b0[0] > 40 && b1[1] - b0[1] > 60, "arrastrar a zona lévaa a outro sitio", JSON.stringify([b0, b1]));
    await app.drag([b1[0] + b1[2] - 2, b1[1] + b1[3] - 2], [b1[0] + b1[2] + 40, b1[1] + b1[3] + 40]);
    await app.sleep(400);
    const b2 = await box();
    c.ok(b2 && b2[2] > b1[2] + 25 && b2[3] > b1[3] + 25, "tirar dunha esquina cambia o tamaño", JSON.stringify([b1, b2]));
    // Premer o nome da páxina na lista leva o visor ata ela.
    await app.ev(`document.querySelector('.pages').scrollTop = 0`);
    await app.click("Páxina 2");
    await app.sleep(700);
    c.ok(await app.ev(`(()=>{const r=document.querySelector('.pages .pg:nth-child(2)').getBoundingClientRect(); const c=document.querySelector('.pages').getBoundingClientRect(); return Math.abs(r.top-c.top) < 60;})()`), "premer «Páxina 2» na lista leva o visor ata ela");
    // Quitar unha zona co seu ×, e volver debuxala.
    c.ok(await app.ev(`(()=>{const x=document.querySelector('.pages .pg:nth-child(2) .ov button'); if(!x) return false; x.click(); return true;})()`), "a × da zona responde");
    await app.waitFor(`![...document.querySelectorAll('.stack .results li .rname')].some(n=>n.textContent.trim()==='Páxina 2')`, "a zona quitouse", 8000);
    await app.ev(`document.querySelector('.pages .pg:nth-child(2)')?.scrollIntoView()`);
    await app.sleep(600);
    const pg2 = await app.ev(`(()=>{const r=document.querySelector('.pages .pg:nth-child(2)').getBoundingClientRect(); return [r.left,r.top,r.width,r.height];})()`);
    await app.drag([pg2[0] + pg2[2] * 0.05, pg2[1] + pg2[3] * 0.06], [pg2[0] + pg2[2] * 0.7, pg2[1] + pg2[3] * 0.11]);
    await app.waitFor(`document.querySelectorAll('.stack .results li').length === 2`, "outra zona debuxada", 10000);
    c.ok(!/Calidade das páxinas tapadas|Resolución/.test(await app.panel()), "tapar xa non ten axustes de calidade");
    if (process.env.TRASNO_SHOTS) await app.shot("tapar.png");
    await go("Tapar as zonas (2)");
    c.ok(await exists("informe-tapado.pdf"), "gardouse informe-tapado.pdf", 20000);
    const red = read("informe-tapado.pdf");
    c.ok(!red.includes("ana@example.com") && !red.includes("Informe anual") && !red.includes("Segredo da pagina dous"), "o texto das páxinas tapadas xa non está no ficheiro");
    c.ok(/pasar a imaxe para tapar con seguridade/.test(await app.ev(`document.body.innerText`)), "con tipografías sen anchos a páxina pasa a imaxe e o di");
    c.ok(sha(pdf) === before, "o orixinal segue igual");
    // As páxinas tapadas son imaxe: aberto de novo, o buscador non atopa o correo.
    await app.open(path.join(app.media, "informe-tapado.pdf"));
    await app.sleep(1500);
    c.ok(/pax|páx/i.test(await app.ev(`document.body.innerText`)), "o PDF tapado abre");

    // ---- Tapar sen converter a páxina en imaxe (tipografía de anchos exactos): só desaparece o que hai baixo a zona
    const cour = path.join(app.media, "carta.pdf");
    fs.writeFileSync(cour, report("Courier"), "latin1");
    await app.open(cour);
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    await app.click("Seguridade");
    await mode("Tapar");
    await app.waitFor(`/zonas a tapar/i.test(document.body.innerText)`, "o panel de tapar outra vez");
    await go("Marcar coincidencias");
    await app.waitFor(`document.querySelectorAll('.stack .results li').length === 1`, "a zona do correo", 30000);
    await go("Tapar as zonas (1)");
    c.ok(await exists("carta-tapado.pdf"), "gardouse carta-tapado.pdf");
    const clean = read("carta-tapado.pdf");
    c.ok(!clean.includes("ana@example.com"), "o correo xa non está no ficheiro");
    c.ok(/O resto do contido quedou como estaba/.test(await app.ev(`document.body.innerText`)), "di que o resto quedou como estaba");
    await app.open(path.join(app.media, "carta-tapado.pdf"));
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    await app.click("Converter");
    await mode("Formatos");
    await app.waitFor(`!!document.querySelector('.stack select[aria-label="Formato de saída"]')`, "o selector de formato de saída", 20000);
    await app.ev(`(()=>{const s=document.querySelector('.stack select[aria-label="Formato de saída"]'); s.value='md'; s.dispatchEvent(new Event('change',{bubbles:true}));})()`);
    await go("Gardar como Markdown");
    c.ok(await exists("carta-tapado.md"), "o PDF tapado aínda ten texto que se pode converter");
    const cm = fs.readFileSync(path.join(app.media, "carta-tapado.md"), "utf8");
    c.ok(/Informe anual/.test(cm) && /Segredo da pagina dous/.test(cm) && !/ana@example\.com/.test(cm), "o resto do texto segue seleccionable e o correo non está", cm.slice(0, 200));

    // ---- Formulario
    const form = path.join(app.media, "solicitude.pdf");
    fs.writeFileSync(form, formPdf(), "latin1");
    await app.open(form);
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    await app.click("Completar");
    await mode("Formulario");
    await app.waitFor(`document.querySelector('.stack input[aria-label=nome]')`, "o campo nome", 15000);
    c.ok(await app.ev(`!!document.querySelector('.stack input[role=switch][aria-label=acepto]')`), "a casilla sae como interruptor");
    await app.ev(`(()=>{const i=document.querySelector('.stack input[aria-label=nome]'); i.value='Marta Lopez'; i.dispatchEvent(new Event('input',{bubbles:true}));})()`);
    await app.toggle("acepto");
    await go("Gardar unha copia chea");
    c.ok(await exists("solicitude-cheo.pdf"), "gardouse solicitude-cheo.pdf");
    const filled = read("solicitude-cheo.pdf");
    c.ok(filled.includes("Marta Lopez") && /\/AcroForm/.test(filled), "o valor entrou e o formulario segue editable");
    await app.toggle("Aplanar o formulario");
    await go("Gardar unha copia aplanada");
    c.ok(await exists("solicitude-aplanado.pdf"), "gardouse solicitude-aplanado.pdf");
    c.ok(!/\/AcroForm/.test(read("solicitude-aplanado.pdf")), "aplanado: xa non hai formulario");
    c.ok(!read("solicitude.pdf").includes("Marta Lopez"), "o orixinal segue baleiro");
  },
};
