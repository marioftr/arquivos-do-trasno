// PDF: Comprimir e OCR usan os mesmos niveis ca o resto (selección derivada dos valores, «Personalizado» e desprazadores sempre á vista).
import fs from "node:fs";
import path from "node:path";

const levels = (app, label) => app.ev(`[...document.querySelectorAll('.stack [role=radiogroup][aria-label=${JSON.stringify(label)}] button')].map(b=>({t:b.innerText.split(String.fromCharCode(10)).join(' ').trim(), on:b.getAttribute('aria-checked')==='true'}))`);
const pick = async (app, label, name) => {
  const r = await app.ev(`(()=>{const b=[...document.querySelectorAll('.stack [role=radiogroup][aria-label=${JSON.stringify(label)}] button')].find(b=>b.innerText.split(String.fromCharCode(10))[0].trim()===${JSON.stringify(name)}); if(!b) return 'sen '+${JSON.stringify(name)}; b.click(); return 'ok'})()`);
  if (r !== "ok") throw new Error(r);
  await app.sleep(300);
};
const val = (app, label) => app.ev(`Number([...document.querySelectorAll('.stack input[type=range]')].find(x=>x.getAttribute('aria-label')===${JSON.stringify(label)})?.value)`);

function minimalPdf() {
  const objs = ["<< /Type /Catalog /Pages 2 0 R >>", "<< /Type /Pages /Kids [3 0 R] /Count 1 >>", "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>", "", "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"];
  const stream = "BT /F1 18 Tf 20 50 Td (Ola) Tj ET";
  objs[3] = `<< /Length ${stream.length} >>\nstream\n${stream}\nendstream`;
  let out = "%PDF-1.4\n";
  const off = [];
  objs.forEach((o, i) => { off.push(out.length); out += `${i + 1} 0 obj\n${o}\nendobj\n`; });
  const xref = out.length;
  out += `xref\n0 ${objs.length + 1}\n0000000000 65535 f \n${off.map((x) => String(x).padStart(10, "0") + " 00000 n \n").join("")}trailer\n<< /Size ${objs.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return out;
}

export default {
  about: "PDF: niveis de compresión e de lectura OCR co mesmo comportamento ca o resto",
  async run(app, c) {
    const pdf = path.join(app.media, "niveis.pdf");
    fs.writeFileSync(pdf, minimalPdf());
    await app.open(pdf);
    await app.click("Comprimir e reparar");
    let l = await levels(app, "Nivel de compresión");
    c.ok(l.length === 4 && l[3].t === "Personalizado" && l[1].on, "Comprimir: tres niveis e Personalizado; «Recomendada» por defecto", l.map((x) => x.t).join(" | "));
    c.ok((await val(app, "Calidade")) === 70 && (await val(app, "Tamaño das imaxes")) === 75, "os desprazadores están sempre á vista cos valores do nivel");
    await pick(app, "Nivel de compresión", "Extrema");
    c.ok((await val(app, "Calidade")) === 45 && (await val(app, "Tamaño das imaxes")) === 50, "«Extrema» pon calidade 45 e tamaño 50");
    await app.slider("Calidade", 60);
    l = await levels(app, "Nivel de compresión");
    c.ok(l[3].on, "mover un desprazador a un valor sen nivel marca «Personalizado»");

    await app.click("Recoñecer texto (OCR)");
    l = await levels(app, "Resolución de lectura");
    c.ok(l.length === 4 && l[3].t === "Personalizado" && l[1].on, "OCR: Rápida, Normal, Alta e Personalizado; Normal por defecto", l.map((x) => x.t).join(" | "));
    c.ok((await val(app, "Resolución")) === 200, "o desprazador de resolución está á vista (200 ppp)");
    await pick(app, "Resolución de lectura", "Alta");
    c.ok((await val(app, "Resolución")) === 300, "«Alta» pon 300 ppp");
    await app.slider("Resolución", 250);
    l = await levels(app, "Resolución de lectura");
    c.ok(l[3].on, "250 ppp non é de ningún nivel: Personalizado");
  },
};
