// O visor de vídeo e a liña de tempo amosan horas cando o vídeo dura máis dunha hora.
export default {
  about: "Vídeo de 1 h 05 min: reloxo do visor e etiquetas do corte con horas",
  async run(app, c) {
    await app.open(app.video("longo.mp4", 3900, { rate: 1, size: "160x120" }));
    await app.waitFor(`/\\d+:\\d\\d:\\d\\d$/.test(document.querySelector('.time')?.innerText ?? '')`, "o reloxo do visor", 15000);
    c.ok(/0:00 \/ 1:05:00/.test(await app.ev(`document.querySelector('.time').innerText`)), "o visor di 0:00 / 1:05:00");
    await app.click("Editar");
    await app.sleep(800);
    const tags = await app.ev(`[...document.querySelectorAll('.tag')].map(t=>t.textContent.trim())`);
    c.ok(tags.length >= 2 && tags.every((t) => !/^\d{4,} s$/.test(t)), "as etiquetas do corte non van en segundos soltos", tags.join(","));
    c.ok(tags.some((t) => t.startsWith("1:05:00")), "a etiqueta do fin pon 1:05:00", tags.join(","));
  },
};
