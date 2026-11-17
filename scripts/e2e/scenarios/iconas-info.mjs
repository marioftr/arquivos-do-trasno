// A «i» explica o que ten ao carón: nunca queda unha liña xenérica de «Máis información» e a «i» dun grupo vai pegada ao seu título.
export default {
  about: "Pestanas dun vídeo: sen «Máis información» e coa «i» xunto ao título",
  async run(app, c) {
    await app.open(app.video("a.mp4", 4));
    for (const tab of ["Converter", "Editar", "Son", "Extraer"]) {
      await app.click(tab);
      await app.sleep(700);
      const modes = await app.ev(`[...document.querySelectorAll('.modes [role=radio]')].map(b=>b.textContent.trim())`);
      for (const mode of modes.length ? modes : [null]) {
        if (mode) await app.click(mode);
        await app.sleep(500);
        const label = `${tab}${mode ? " · " + mode : ""}`;
        c.ok(!(await app.ev(`/Máis información/.test(document.querySelector('.stack')?.innerText ?? '')`)), `${label}: sen liña xenérica de «Máis información»`);
        // Cada «i» pegada a un título de grupo está a menos de 40 px do final do texto do título.
        const far = await app.ev(`[...document.querySelectorAll('.group > .gn.title')].filter(g=>{const t=g.parentElement.querySelector(':scope > .group-title'); const r=document.createRange(); r.selectNodeContents(t); const end=r.getBoundingClientRect().right; return g.getBoundingClientRect().left-end>40}).length`);
        c.ok(far === 0, `${label}: as «i» de grupo van xunto ao título`, `${far} lonxe`);
      }
    }
  },
};
