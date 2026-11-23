// A volta rápida (tutorial): sae soa na primeira execución, avanza e retrocede, pecha con Esc, non volve saír e pódese repetir desde Axustes.
export default {
  about: "Tutorial de benvida: primeira execución, pasos, saltar e repetir desde Axustes",
  async run(app, c) {
    const open = () => app.ev(`!!document.querySelector('[role=dialog][aria-labelledby=tut-title]')`);
    const counter = () => app.ev(`document.querySelector('.say .n')?.textContent.trim() ?? ''`);

    // Primeira execución: sen a marca de «visto», ábrese soa.
    await app.ev("localStorage.removeItem('arquivos-do-trasno.tutorial-visto'); location.reload()");
    await app.sleep(4500);
    c.ok(await open(), "na primeira execución ábrese a volta rápida");
    c.ok((await counter()) === "1 / 9", "empeza no paso 1 de 9", await counter());
    c.ok(await app.ev(`document.activeElement?.classList.contains('primary')`), "o foco está no botón Seguinte");
    const shots = [];
    for (let k = 2; k <= 9; k++) {
      await app.click(k === 9 ? "Seguinte" : "Seguinte");
      await app.sleep(1100);
      c.ok((await counter()) === `${k} / 9`, `paso ${k} de 9`, await counter());
      if (k === 2 || k === 5 || k === 7 || k === 8) shots.push(await app.shot(`tutorial-paso-${k}.png`));
    }
    c.ok(await app.ev(`document.querySelector('.nav .primary')?.textContent.trim() === 'Empezar'`), "o último paso ten o botón Empezar");
    // O trasno e as escenas non se len: só a caixa de texto (aria-live) e os botóns.
    c.ok(await app.ev(`document.querySelector('[role=dialog] .stage')?.getAttribute('aria-hidden') === 'true' && !!document.querySelector('[role=dialog] .say[aria-live=polite]')`), "o escenario é decorativo e o texto vai nunha rexión en vivo");
    await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))`);
    await app.sleep(300);
    c.ok((await counter()) === "8 / 9", "a frecha esquerda volve un paso", await counter());
    await app.click("Anterior");
    await app.sleep(300);
    c.ok((await counter()) === "7 / 9", "Anterior volve un paso", await counter());
    // Tab non saca o foco da xanela.
    for (let k = 0; k < 8; k++) await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }))`);
    c.ok(await app.ev(`document.querySelector('[role=dialog]')?.contains(document.activeElement) ?? false`), "o foco queda dentro da volta rápida");
    // Esc pecha e xa non volve saír.
    await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
    await app.sleep(400);
    c.ok(!(await open()), "Esc pecha a volta rápida");
    c.ok((await app.ev(`localStorage.getItem('arquivos-do-trasno.tutorial-visto')`)) === "1", "queda marcada como vista");
    await app.ev("location.reload()");
    await app.sleep(4500);
    c.ok(!(await open()), "ao abrir de novo non volve saír");

    // Repetir desde Axustes.
    await app.ev(`document.querySelector('button[aria-label="Axustes"], [title="Axustes"]')?.click()`);
    await app.sleep(500);
    if (!(await app.ev(`!!document.querySelector('[role=dialog]')`))) await app.ev(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true }))`);
    await app.sleep(500);
    c.ok(await app.ev(`/Repetir a volta rápida/.test(document.body.innerText)`), "Axustes ten o botón Repetir a volta rápida");
    await app.click("Repetir a volta rápida");
    await app.sleep(600);
    c.ok(await open(), "Repetir a volta rápida abre o tutorial");
    c.ok(!(await app.ev(`!!document.querySelector('.hub, [aria-label="Axustes"][role=dialog]')`)), "Axustes pechou ao abrir o tutorial");
    await app.click("Saltar");
    await app.sleep(400);
    c.ok(!(await open()), "Saltar pecha a volta rápida");
    console.log("  capturas:", shots.join(", "));
  },
};
