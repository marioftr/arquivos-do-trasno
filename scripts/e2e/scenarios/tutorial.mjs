// O titorial: na primeira execución sae primeiro a benvida; «Ver o titorial» abre os pasos; avanza e retrocede, pecha con Esc, non volve saír e pódese ver outra vez desde Axustes.
export default {
  about: "Titorial: benvida na primeira execución, pasos, saltar e velo de novo desde Axustes",
  async run(app, c) {
    const dialog = () => app.ev(`!!document.querySelector('[role=dialog][aria-labelledby=tut-title]')`);
    const counter = () => app.ev(`document.querySelector('.say .n')?.textContent.trim() ?? ''`);
    const shots = [];

    // Primeira execución: sen a marca de «visto», sae a benvida (e non os pasos).
    await app.ev("localStorage.removeItem('arquivos-do-trasno.tutorial-visto'); location.reload()");
    await app.sleep(4500);
    c.ok(await dialog(), "na primeira execución sae unha xanela");
    c.ok(await app.ev(`!!document.querySelector('.panel.welcome') && !document.querySelector('.say')`), "primeiro é a benvida, non os pasos");
    c.ok(await app.ev(`document.querySelector('.welcome .big img')?.width === 192`), "o trasno vai en grande");
    shots.push(await app.shot("titorial-benvida.png"));
    c.ok(await app.ev(`document.activeElement?.classList.contains('primary')`), "o foco está en Ver o titorial");
    await app.click("Ver o titorial");
    await app.sleep(800);
    c.ok((await counter()) === "1 / 9", "empeza no paso 1 de 9", await counter());
    for (let k = 2; k <= 9; k++) {
      await app.click("Seguinte");
      await app.sleep(1200);
      c.ok((await counter()) === `${k} / 9`, `paso ${k} de 9`, await counter());
      shots.push(await app.shot(`titorial-paso-${k}.png`));
    }
    c.ok(await app.ev(`document.querySelector('.go .primary')?.textContent.trim() === 'Rematar'`), "o último paso ten o botón Rematar");
    c.ok(await app.ev(`!/repet|Axustes/i.test(document.querySelector('.say')?.textContent ?? '')`), "o último texto non fala de repetir");
    // O trasno para xunto ao que explica o paso (aqui, debaixo do buscador), sen taparo.
    const near = await app.ev(`(()=>{const t=document.querySelector('.tr').getBoundingClientRect(), p=document.querySelector('.pal').getBoundingClientRect(); return [Math.round(t.top-p.bottom), Math.round(Math.abs((t.left+t.right)/2-(p.left+p.right)/2))]})()`);
    c.ok(near[0] >= -4 && near[0] < 40 && near[1] < 40, "o trasno está debaixo do buscador, sen taparo", JSON.stringify(near));
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
    // Esc pecha e xa non volve saír.
    await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
    await app.sleep(400);
    c.ok(!(await dialog()), "Esc pecha o titorial");
    c.ok((await app.ev(`localStorage.getItem('arquivos-do-trasno.tutorial-visto')`)) === "1", "queda marcado como visto");
    await app.ev("location.reload()");
    await app.sleep(4500);
    c.ok(!(await dialog()), "ao abrir de novo non volve saír");

    // «Agora non» na benvida tamén o deixa como visto.
    await app.ev("localStorage.removeItem('arquivos-do-trasno.tutorial-visto'); location.reload()");
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
