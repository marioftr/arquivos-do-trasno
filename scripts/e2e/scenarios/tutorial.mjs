// O titorial: na primeira execución sae primeiro a benvida; «Ver o titorial» abre os pasos; avanza e retrocede, pecha con Esc, non volve saír e pódese ver outra vez desde Axustes.
export default {
  about: "Titorial: benvida na primeira execución, pasos, saltar e velo de novo desde Axustes",
  async run(app, c) {
    const dialog = () => app.ev(`!!document.querySelector('[role=dialog][aria-labelledby=tut-title]')`);
    const counter = () => app.ev(`document.querySelector('.say .n')?.textContent.trim() ?? ''`);
    const shots = [];

    // Primeira execución: sen a marca de «visto», sae a benvida (e non os pasos).
    await app.ev("localStorage.removeItem('arquivos-do-trasno.tutorial-visto'); localStorage.removeItem('arquivos-do-trasno.extensions-seen'); location.reload()");
    await app.sleep(4500);
    c.ok(await dialog(), "na primeira execución sae unha xanela");
    c.ok(await app.ev(`!!document.querySelector('.panel.welcome') && !document.querySelector('.say')`), "primeiro é a benvida, non os pasos");
    c.ok(await app.ev(`document.querySelector('.welcome .big img')?.width === 192`), "o trasno vai en grande");
    shots.push(await app.shot("titorial-benvida.png"));
    c.ok(await app.ev(`document.activeElement?.classList.contains('primary')`), "o foco está no botón principal da benvida");
    // Antes do titorial, as ventás de personalización: tamaño da letra e tema.
    await app.click("Comezar");
    await app.sleep(500);
    c.ok(await app.ev(`document.querySelector('#tut-title')?.textContent.trim() === 'Tamaño da letra' && !!document.querySelector('.welcome [role=radiogroup], .welcome .seg, .welcome button[aria-pressed], .welcome button[role=radio]')`), "despois da benvida sae o tamaño da letra");
    shots.push(await app.shot("titorial-letra.png"));
    await app.click("Seguinte");
    await app.sleep(500);
    c.ok(await app.ev(`document.querySelector('#tut-title')?.textContent.trim() === 'Tema' && document.querySelectorAll('.welcome .theme').length === 3`), "despois sae o tema, con tres opcións");
    shots.push(await app.shot("titorial-tema.png"));
    c.ok(await app.ev(`document.querySelectorAll('.trasno:not(.away)').length === 0`), "con estas ventás tampouco hai dous trasnos");
    await app.click("Ver o titorial");
    await app.sleep(800);
    c.ok((await counter()) === "1 / 9", "empeza no paso 1 de 9", await counter());
    await app.sleep(1200);
    c.ok(await app.ev(`document.querySelectorAll('.tr .sprite').length === 1 && !document.querySelector('.zone .sprite') && document.querySelector('.trasno')?.classList.contains('away')`), "no paso 1 só hai un trasno en pantalla (o do titorial)");
    c.ok(await app.ev(`getComputedStyle(document.querySelector('.tr')).width === '64px'`), "o trasno do titorial mide un múltiplo de 32 (píxeles nítidos)");
    // Cada paso ten a súa animación: o que se ve na maqueta cambia co tempo (ficheiro elixido, ferramenta, nivel, cursor, páxina…).
    const snap = {
      2: `[...document.querySelectorAll('.row')].findIndex((r) => r.classList.contains('sel'))`,
      3: `[...document.querySelectorAll('.menu span')].findIndex((r) => r.classList.contains('hot'))`,
      4: `document.querySelector('.rail')?.children.length + ':' + document.querySelector('.vh b')?.textContent`,
      5: `[...document.querySelectorAll('.seg span')].findIndex((r) => r.classList.contains('on'))`,
      6: `document.querySelector('.jtext em')?.textContent`,
      7: `document.querySelector('.cur')?.getAttribute('style')`,
      8: `document.querySelector('.dlg')?.innerText`,
      9: `document.querySelector('.pt')?.textContent`,
    };
    for (let k = 2; k <= 9; k++) {
      await app.click("Seguinte");
      await app.sleep(1200);
      c.ok((await counter()) === `${k} / 9`, `paso ${k} de 9`, await counter());
      shots.push(await app.shot(`titorial-paso-${k}.png`));
      const x = await app.ev(snap[k]);
      await app.sleep(k === 6 ? 2600 : 3000);
      const y = await app.ev(snap[k]);
      c.ok(x !== y, `o paso ${k} anima a maqueta`, `${x} → ${y}`);
    }
    c.ok(await app.ev(`document.querySelector('.go .primary')?.textContent.trim() === 'Rematar'`), "o último paso ten o botón Rematar");
    c.ok(await app.ev(`!/repet/i.test(document.querySelector('.say')?.textContent ?? '')`), "o último texto non fala de repetir");
    // O trasno para xunto ao que explica o paso (aqui, ao carón da xanela de Axustes), sen taparo.
    const near = await app.ev(`(()=>{const t=document.querySelector('.tr').getBoundingClientRect(), p=document.querySelector('.dlg').getBoundingClientRect(); return [Math.round(t.left-p.right), Math.round(Math.abs((t.top+t.bottom)/2-(p.top+p.bottom)/2))]})()`);
    c.ok(near[0] >= -4 && near[0] < 40 && near[1] < 80, "o trasno está ao carón da xanela de Axustes, sen taparla", JSON.stringify(near));
    c.ok(await app.ev(`document.querySelector('[role=dialog] .wrap')?.getAttribute('aria-hidden') === 'true' && !!document.querySelector('[role=dialog] .say[aria-live=polite]')`), "o escenario é decorativo e o texto vai nunha rexión en vivo");
    // Os círculos dos pasos están centrados no seu contido.
    const dots = await app.ev(`(()=>{const d=document.querySelector('.dots').getBoundingClientRect(), n=document.querySelector('.nav').getBoundingClientRect(); return Math.abs((d.left+d.width/2)-(n.left+n.width/2))})()`);
    c.ok(dots < 2, "os puntos de paso están centrados na barra", String(dots));
    await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))`);
    await app.sleep(300);
    c.ok((await counter()) === "8 / 9", "a frecha esquerda volve un paso", await counter());
    await app.click("Anterior");
    await app.sleep(300);
    c.ok((await counter()) === "7 / 9", "Anterior volve un paso", await counter());
    // Tab non saca o foco da xanela.
    for (let k = 0; k < 8; k++) await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }))`);
    c.ok(await app.ev(`document.querySelector('[role=dialog]')?.contains(document.activeElement) ?? false`), "o foco queda dentro do titorial");
    // Quen xa usaba a app (marca das extensións de versións anteriores) non ve o titorial tras actualizar, aínda sen a súa marca.
    await app.ev("localStorage.removeItem('arquivos-do-trasno.tutorial-visto'); localStorage.setItem('arquivos-do-trasno.extensions-seen', '1'); location.reload()");
    await app.sleep(4500);
    c.ok(!(await dialog()), "unha instalación anterior non ve o titorial tras actualizar");
    await app.ev("localStorage.removeItem('arquivos-do-trasno.extensions-seen'); localStorage.removeItem('arquivos-do-trasno.tutorial-visto'); location.reload()");
    await app.sleep(4500);
    c.ok(await dialog(), "e na primeira execución de verdade si sae");
    // Esc pecha e xa non volve saír.
    await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
    await app.sleep(400);
    c.ok(!(await dialog()), "Esc pecha o titorial");
    c.ok((await app.ev(`localStorage.getItem('arquivos-do-trasno.tutorial-visto')`)) === "1", "queda marcado como visto");
    await app.ev("location.reload()");
    await app.sleep(4500);
    c.ok(!(await dialog()), "ao abrir de novo non volve saír");

    // «Agora non» na benvida tamén o deixa como visto.
    await app.ev("localStorage.removeItem('arquivos-do-trasno.tutorial-visto'); localStorage.removeItem('arquivos-do-trasno.extensions-seen'); location.reload()");
    await app.sleep(4500);
    await app.click("Agora non");
    await app.sleep(400);
    c.ok(!(await dialog()), "Agora non pecha a benvida");
    c.ok((await app.ev(`localStorage.getItem('arquivos-do-trasno.tutorial-visto')`)) === "1", "e queda marcado como visto");

    // Desde Axustes vai directo aos pasos, sen a benvida.
    await app.ev(`document.querySelector('button[aria-label="Axustes"], [title="Axustes"]')?.click()`);
    await app.sleep(500);
    if (!(await app.ev(`!!document.querySelector('[role=dialog]')`))) await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true }))`);
    await app.sleep(500);
    c.ok(await app.ev(`/Titorial/.test(document.body.innerText) && !/Volta rápida/.test(document.body.innerText)`), "Axustes chama Titorial á sección");
    await app.click("Ver o titorial");
    await app.sleep(600);
    c.ok(await dialog(), "Ver o titorial abre o titorial");
    c.ok((await counter()) === "1 / 9", "desde Axustes vai directo aos pasos", await counter());
    await app.click("Saltar");
    await app.sleep(400);
    c.ok(!(await dialog()), "Saltar pecha o titorial");
    console.log("  capturas:", shots.join(", "));
  },
};
