// Os selectores con perfís levan a información dentro de cada botón, non nunha liña solta cunha «i».
export default {
  about: "Silencios: cada perfil mostra os seus valores no botón",
  async run(app, c) {
    await app.open(app.video("a.mp4", 4));
    await app.click("Editar");
    await app.click("Silencios");
    const opts = await app.ev(`[...document.querySelectorAll('.stack [role=radiogroup] button')].map(b=>b.innerText.split(String.fromCharCode(10)).join(' ').trim())`);
    c.ok(opts.length === 4 && opts[0].includes("−40 dB") && opts[1].includes("−35 dB · 0,5 s") && opts[2].includes("−30 dB") && /Personalizado/.test(opts[3]), "os catro botóns levan nome e valores", opts.join(" | "));
    const text = await app.ev(`document.querySelector('.stack').innerText`);
    c.ok(!/orixinal non se toca|recodifícase a imaxe/.test(text), "sen mensaxes de recheo");
  },
};
