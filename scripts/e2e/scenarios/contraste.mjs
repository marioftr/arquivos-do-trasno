// Contraste do texto en toda a app (WCAG AA: 4,5:1 no texto normal, 3:1 no grande), no tema escuro e no claro, nas pantallas principais.
// Mide cada texto visible contra o fondo real que ten detrás (compoñendo as capas con transparencia). Os controis desactivados non contan.
import path from "node:path";
import fs from "node:fs";

const MEASURE = `(() => {
  const cv = document.createElement('canvas'); cv.width = cv.height = 1;
  const g = cv.getContext('2d', { willReadFrequently: true });
  const rgba = (css) => { g.clearRect(0, 0, 1, 1); g.fillStyle = '#010203'; g.fillStyle = css; g.fillRect(0, 0, 1, 1); const d = g.getImageData(0, 0, 1, 1).data; return [d[0], d[1], d[2], d[3] / 255]; };
  const over = (t, b) => [t[0] * t[3] + b[0] * (1 - t[3]), t[1] * t[3] + b[1] * (1 - t[3]), t[2] * t[3] + b[2] * (1 - t[3]), 1];
  const lum = ([r, g2, b]) => { const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(r) + 0.7152 * f(g2) + 0.0722 * f(b); };
  const ratio = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
  const out = new Map();
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  let n;
  while ((n = walker.nextNode())) {
    const text = n.textContent.trim();
    if (!text) continue;
    const el = n.parentElement;
    if (!el || el.closest('[inert],[disabled],[aria-hidden=true],script,style,.sr-only,[hidden]')) continue;
    const r = document.createRange(); r.selectNodeContents(n);
    const box = r.getBoundingClientRect();
    if (box.width < 2 || box.height < 2 || box.bottom < 0 || box.top > innerHeight) continue;
    const cs = getComputedStyle(el);
    if (cs.visibility !== 'visible' || cs.display === 'none') continue;
    // Fondo real: as capas desde o elemento ata a raíz; se hai imaxe ou degradado antes dun fondo opaco, non se pode medir.
    const layers = [];
    let skip = false, opacity = 1;
    for (let e = el; e; e = e.parentElement) {
      const s = getComputedStyle(e);
      opacity *= parseFloat(s.opacity);
      if (s.backgroundImage !== 'none') { skip = true; break; }
      const bg = rgba(s.backgroundColor);
      if (bg[3] > 0) layers.push(bg);
      if (bg[3] >= 1) break;
    }
    if (skip) continue;
    let bg = layers.length && layers[layers.length - 1][3] >= 1 ? layers.pop() : rgba(getComputedStyle(document.documentElement).getPropertyValue('--bg') || '#000');
    while (layers.length) bg = over(layers.pop(), bg);
    const fg0 = rgba(cs.color);
    const fg = over([fg0[0], fg0[1], fg0[2], fg0[3] * opacity], bg);
    if (opacity < 0.1) continue;
    const px = parseFloat(cs.fontSize);
    const large = px >= 24 || (px >= 18.66 && parseInt(cs.fontWeight) >= 700);
    const need = large ? 3 : 4.5;
    const rt = ratio(fg, bg);
    if (rt < need) {
      const key = (el.className || el.tagName) + '|' + fg.map(Math.round) + '|' + bg.map(Math.round);
      if (!out.has(key)) out.set(key, { text: text.slice(0, 32), cls: String(el.className || el.tagName).slice(0, 40), fg: fg.slice(0, 3).map(Math.round).join(','), bg: bg.slice(0, 3).map(Math.round).join(','), ratio: Math.round(rt * 100) / 100, need });
    }
  }
  return [...out.values()];
})()`;

export default {
  about: "Contraste do texto (WCAG AA) nas pantallas principais, en tema escuro e claro",
  async run(app, c) {
    const pic = await (async () => {
      const b64 = await app.ev(`(()=>{const k=document.createElement('canvas');k.width=200;k.height=120;const g=k.getContext('2d');g.fillStyle='#d01818';g.beginPath();g.arc(100,60,40,0,Math.PI*2);g.fill();return k.toDataURL('image/png').split(',')[1]})()`);
      const p = path.join(app.media, "contraste.png");
      fs.writeFileSync(p, Buffer.from(b64, "base64"));
      return p;
    })();
    await app.open(app.video("c.mp4", 3));
    await app.open(pic);

    const audit = async (label) => {
      for (const theme of ["dark", "light"]) {
        await app.ev(`document.documentElement.dataset.theme = ${JSON.stringify(theme)}`);
        await app.sleep(250);
        const bad = await app.ev(MEASURE);
        c.ok(bad.length === 0, `${label} (${theme === "dark" ? "escuro" : "claro"}): todo o texto chega ao contraste mínimo`, bad.slice(0, 6).map((b) => `«${b.text}» [${b.cls}] ${b.fg} sobre ${b.bg} = ${b.ratio}:1 (<${b.need})`).join(" || "));
      }
    };

    // Imaxe: pestanas de ferramentas.
    await app.ev(`[...document.querySelectorAll('li[role=option]')].find(li=>li.innerText.includes('contraste.png'))?.querySelector('button.row')?.click()`);
    await app.sleep(600);
    for (const tab of ["Converter", "Transformar", "Quitar fondo"]) {
      await app.click(tab);
      await app.sleep(700);
      await audit(`Imaxe · ${tab}`);
    }
    // Vídeo.
    await app.ev(`[...document.querySelectorAll('li[role=option]')].find(li=>li.innerText.includes('c.mp4'))?.querySelector('button.row')?.click()`);
    await app.sleep(600);
    for (const tab of ["Converter", "Editar", "Son", "Restaurar", "Transcribir", "Subtítulos", "Extraer", "Información"]) {
      await app.click(tab);
      await app.sleep(700);
      await audit(`Vídeo · ${tab}`);
    }
    // Menú do trasno e xanelas.
    await app.ev(`document.querySelector('.dock')?.dispatchEvent(new PointerEvent('pointerenter',{bubbles:false}))`);
    await app.sleep(700);
    await audit("Menú do trasno");
    await app.radial("Descargar por ligazón");
    await app.sleep(800);
    await audit("Descargar por ligazón");
    await app.ev(`document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))`);
    await app.sleep(700);
    await app.radial("Axustes");
    for (const tab of ["Xeral", "Extensións"]) {
      await app.click(tab).catch(() => {});
      await app.sleep(700);
      await audit(`Axustes · ${tab}`);
    }
  },
};
