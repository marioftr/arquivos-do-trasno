// Os selectores con perfís levan o que significan dentro de cada botón e a explicación nunha «i», nunca en hover nin en liñas soltas.
export default {
  about: "Silencios: botóns con texto claro, «i» co detalle, botón Analizar no estilo de Estimar",
  async run(app, c) {
    await app.open(app.video("a.mp4", 4));
    await app.click("Son");
    await app.click("Silencios");
    const opts = await app.ev(`[...document.querySelectorAll('.stack [role=radiogroup] button')].map(b=>b.innerText.split(String.fromCharCode(10)).join(' ').trim())`);
    c.ok(opts.length === 4 && opts[0] === "Suave Pausas longas" && opts[1] === "Normal Pausas habituais" && opts[2] === "Forte Case todas" && opts[3] === "Personalizado", "os botóns din o que fan e Personalizado non leva texto de máis", opts.join(" | "));
    c.ok(await app.ev(`[...document.querySelectorAll('.stack [role=radiogroup] button')].every(b=>!b.title)`), "ningún botón leva información en hover (title)");
    c.ok(await app.ev(`!!document.querySelector('.stack .wrap .lab button.tip')`), "o selector ten a súa «i» ao carón do título");
    const btn = await app.ev(`(()=>{const b=[...document.querySelectorAll('.stack .estimate .btn')][0]; if(!b) return null; const r=b.getBoundingClientRect(), p=document.querySelector('.stack').getBoundingClientRect(); return {t:b.textContent.trim(), w:r.width, pw:p.width, l:r.left-p.left}})()`);
    c.ok(btn && btn.t === "Analizar silencios" && btn.w < btn.pw * 0.6 && btn.l < 4, "o botón é curto, a esquerda e leva «Analizar silencios»", JSON.stringify(btn));
    const text = await app.ev(`document.querySelector('.stack').innerText`);
    c.ok(!/orixinal non se toca|recodifícase a imaxe|Valores propios|Ver canto/.test(text), "sen texto de recheo");
  },
};
