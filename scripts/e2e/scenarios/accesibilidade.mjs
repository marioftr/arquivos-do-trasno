// Ideas de ui-sdk (vercel-labs/ui): toda control ten nome accesible, as «i» ligan co seu globo e as pestanas e os selectores seguen o patrón WAI-ARIA.
export default {
  about: "Accesibilidade: nomes nos controis, «i» ligadas ao seu texto e pestanas con ARIA correcta",
  async run(app, c) {
    await app.open(app.video("a.mp4", 4));
    for (const tab of ["Converter", "Editar", "Extraer", "Información"]) {
      await app.click(tab);
      await app.sleep(600);
      const unnamed = await app.ev(`[...document.querySelectorAll('.tools button, .tools input, .tools select')].filter(e=>{ if(e.type==='hidden') return false; const r=e.getBoundingClientRect(); if(!r.width) return false; const name=(e.getAttribute('aria-label')||e.textContent||e.title||(e.labels&&e.labels[0]&&e.labels[0].textContent)||'').trim(); return !name}).map(e=>e.outerHTML.slice(0,90))`);
      c.ok(unnamed.length === 0, `${tab}: todos os controis visibles teñen nome`, unnamed.join(" | "));
    }
    await app.click("Editar");
    const tabs = await app.ev(`(()=>{const l=document.querySelector('[role=tablist]'); const t=[...l.querySelectorAll('[role=tab]')]; return {n:t.length, selected:t.filter(x=>x.getAttribute('aria-selected')==='true').length, tab0:t.filter(x=>x.tabIndex===0).length}})()`);
    c.ok(tabs.n > 1 && tabs.selected === 1 && tabs.tab0 === 1, "a barra de ferramentas ten unha soa pestana activa e unha soa parada de Tab (roving tabindex)", JSON.stringify(tabs));
    const radios = await app.ev(`[...document.querySelectorAll('[role=radiogroup]')].every(g=>{const r=[...g.querySelectorAll('[role=radio]')]; return r.filter(x=>x.getAttribute('aria-checked')==='true').length===1 && r.filter(x=>x.tabIndex===0).length===1})`);
    c.ok(radios, "cada selector ten unha opción marcada e unha soa parada de Tab");
    // A «i» enlaza co seu globo ao abrirse.
    await app.waitFor(`!!document.querySelector('.stack button.tip')`, "a primeira «i»");
    await app.sleep(600);
    await app.ev(`document.querySelector('.stack button.tip').focus()`);
    await app.sleep(500);
    const linked = await app.ev(`(()=>{const b=document.querySelector('.stack button.tip'); const id=b.getAttribute('aria-describedby'); return !!id && !!document.getElementById(id) && document.getElementById(id).getAttribute('role')==='tooltip'})()`);
    c.ok(linked, "ao enfocar unha «i», o globo está ligado con aria-describedby", await app.ev(`JSON.stringify({focus: document.hasFocus(), act: document.activeElement?.className, ex: document.querySelector(".stack button.tip")?.getAttribute("aria-expanded"), pops: document.querySelectorAll("[role=tooltip]").length})`));
    await app.ev(`document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))`);
    await app.sleep(250);
    c.ok(!(await app.ev(`!!document.querySelector('[role=tooltip]')`)), "Esc pecha o globo");
  },
};
