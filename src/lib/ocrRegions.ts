// Zonas de texto para o OCR: sen DOM, só números, para poder probalo á parte.
// A lectura de toda a imaxe falla coas letras pequenas (capturas, visores 3D, etiquetas): estas funcións din ONDE hai texto
// para que o OCR o lea de novo só nesa zona, ampliada.

export interface Box {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}
export interface Region {
  box: Box;
  /** Altura típica das letras (px da imaxe orixinal), se se sabe. */
  lineH?: number;
  /** Veu dunha lectura xa feita (hai texto seguro), non só da busca de bordos. */
  seeded: boolean;
  /** Canto texto parece haber (para ordenar). */
  score: number;
}

const area = (b: Box) => Math.max(0, b.x1 - b.x0) * Math.max(0, b.y1 - b.y0);
function inter(a: Box, b: Box): number {
  return area({ x0: Math.max(a.x0, b.x0), y0: Math.max(a.y0, b.y0), x1: Math.min(a.x1, b.x1), y1: Math.min(a.y1, b.y1) });
}
const union = (a: Box, b: Box): Box => ({ x0: Math.min(a.x0, b.x0), y0: Math.min(a.y0, b.y0), x1: Math.max(a.x1, b.x1), y1: Math.max(a.y1, b.y1) });

/** Busca zonas con pinta de texto (moitos bordos pequenos e xuntos, en liñas) nunha imaxe en grises. As caixas saen en píxeles da
 *  imaxe dada. Non distingue texto de textura: o OCR é quen decide despois se as letras son de verdade. */
export function detectTextRegions(lum: Uint8Array, w: number, h: number): Region[] {
  const CELL = 6;
  // Gradiente (diferenzas entre veciños) e limiar adaptado: nunha imaxe plana, calquera bordo marcado; nunha foto, só os máis fortes.
  const grad = new Uint8Array(w * h);
  const hist = new Uint32Array(256);
  for (let y = 1; y < h - 1; y++) {
    for (let x = 1; x < w - 1; x++) {
      const i = y * w + x;
      const g = Math.min(255, Math.abs(lum[i + 1] - lum[i - 1]) + Math.abs(lum[i + w] - lum[i - w]));
      grad[i] = g;
      hist[g]++;
    }
  }
  let acc = 0;
  let p = 0;
  const target = (w * h) * 0.9;
  for (; p < 255 && acc + hist[p] < target; p++) acc += hist[p];
  const T = Math.max(48, p);

  const cw = Math.ceil(w / CELL);
  const ch = Math.ceil(h / CELL);
  const cnt = new Uint16Array(cw * ch);
  for (let y = 1; y < h - 1; y++) {
    const row = y * w;
    const cy = ((y / CELL) | 0) * cw;
    for (let x = 1; x < w - 1; x++) if (grad[row + x] >= T) cnt[cy + ((x / CELL) | 0)]++;
  }
  // Cela con texto: algúns bordos, pero non todo bordos (un bloque macizo ou unha reixa non son letras).
  const on = new Uint8Array(cw * ch);
  for (let i = 0; i < on.length; i++) on[i] = cnt[i] >= 3 && cnt[i] <= CELL * CELL * 0.7 ? 1 : 0;

  // Xúntanse as celas próximas (máis en horizontal: unha liña de letras; algo en vertical: as liñas dun bloque).
  const HX = 3;
  const HY = 2;
  const dil = new Uint8Array(cw * ch);
  for (let y = 0; y < ch; y++) {
    for (let x = 0; x < cw; x++) {
      if (!on[y * cw + x]) continue;
      for (let dy = -HY; dy <= HY; dy++) {
        const yy = y + dy;
        if (yy < 0 || yy >= ch) continue;
        for (let dx = -HX; dx <= HX; dx++) {
          const xx = x + dx;
          if (xx >= 0 && xx < cw) dil[yy * cw + xx] = 1;
        }
      }
    }
  }
  // Compoñentes conexas da máscara dilatada; a caixa sae só das celas orixinais.
  const seen = new Uint8Array(cw * ch);
  const out: Region[] = [];
  const stack: number[] = [];
  for (let s = 0; s < dil.length; s++) {
    if (!dil[s] || seen[s]) continue;
    let x0 = cw, y0 = ch, x1 = -1, y1 = -1, cells = 0, edges = 0;
    stack.push(s);
    seen[s] = 1;
    while (stack.length) {
      const i = stack.pop()!;
      const x = i % cw;
      const y = (i / cw) | 0;
      if (on[i]) {
        cells++;
        edges += cnt[i];
        if (x < x0) x0 = x;
        if (x > x1) x1 = x;
        if (y < y0) y0 = y;
        if (y > y1) y1 = y;
      }
      if (x > 0 && dil[i - 1] && !seen[i - 1]) (seen[i - 1] = 1), stack.push(i - 1);
      if (x < cw - 1 && dil[i + 1] && !seen[i + 1]) (seen[i + 1] = 1), stack.push(i + 1);
      if (y > 0 && dil[i - cw] && !seen[i - cw]) (seen[i - cw] = 1), stack.push(i - cw);
      if (y < ch - 1 && dil[i + cw] && !seen[i + cw]) (seen[i + cw] = 1), stack.push(i + cw);
    }
    if (x1 < 0) continue;
    const box: Box = { x0: x0 * CELL, y0: y0 * CELL, x1: Math.min(w, (x1 + 1) * CELL), y1: Math.min(h, (y1 + 1) * CELL) };
    const bw = box.x1 - box.x0;
    const bh = box.y1 - box.y0;
    // Demasiado pequena (unha mancha), demasiado grande (a imaxe enteira: ou é unha páxina, que xa le a primeira pasada, ou é textura)
    // ou moi pouco densa (bordos soltos): fóra.
    if (bw < 24 || bh < 9 || area(box) > w * h * 0.5 || cells < 4) continue;
    if (cells / ((bw / CELL) * (bh / CELL)) < 0.12) continue;
    out.push({ box, seeded: false, score: edges });
  }
  return out.sort((a, b) => b.score - a.score);
}

/** Agrupa as palabras xa lidas en zonas (palabras da mesma altura e preto unhas doutras) e dálles marxe, sobre todo á dereita e abaixo,
 *  onde adoitan estar as cifras que acompañan unha etiqueta. */
export function seedRegions(words: Box[], w: number, h: number): Region[] {
  if (!words.length) return [];
  const hs = words.map((b) => b.y1 - b.y0);
  const parent = words.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  for (let i = 0; i < words.length; i++) {
    for (let j = i + 1; j < words.length; j++) {
      const a = words[i];
      const b = words[j];
      const lh = Math.max(hs[i], hs[j]);
      const dx = Math.max(0, Math.max(a.x0, b.x0) - Math.min(a.x1, b.x1));
      const dy = Math.max(0, Math.max(a.y0, b.y0) - Math.min(a.y1, b.y1));
      const ratio = Math.max(hs[i], hs[j]) / Math.max(1, Math.min(hs[i], hs[j]));
      if (ratio < 2 && dx < lh * 2.5 && dy < lh * 1.6) parent[find(i)] = find(j);
    }
  }
  const groups = new Map<number, number[]>();
  words.forEach((_, i) => groups.set(find(i), [...(groups.get(find(i)) ?? []), i]));
  const out: Region[] = [];
  for (const ids of groups.values()) {
    let box = words[ids[0]];
    for (const i of ids) box = union(box, words[i]);
    const lh = ids.map((i) => hs[i]).sort((a, b) => a - b)[ids.length >> 1];
    const padX = lh * 10;
    const padY = lh * 1.5;
    out.push({
      box: { x0: Math.max(0, box.x0 - lh * 2), y0: Math.max(0, box.y0 - padY), x1: Math.min(w, box.x1 + padX), y1: Math.min(h, box.y1 + padY) },
      lineH: lh,
      seeded: true,
      score: ids.length * 1000,
    });
  }
  return out;
}

/** Xunta as zonas que se solapan (unha do OCR e outra dos bordos para o mesmo texto) e queda coas `max` máis prometedoras. As zonas con
 *  texto seguro van primeiro. */
export function planRegions(list: Region[], w: number, h: number, max = 10): Region[] {
  const regs = list.map((r) => ({ ...r }));
  let changed = true;
  while (changed) {
    changed = false;
    outer: for (let i = 0; i < regs.length; i++) {
      for (let j = i + 1; j < regs.length; j++) {
        const sm = Math.min(area(regs[i].box), area(regs[j].box));
        if (sm > 0 && inter(regs[i].box, regs[j].box) / sm > 0.3) {
          const a = regs[i];
          const b = regs[j];
          regs[i] = { box: union(a.box, b.box), lineH: a.lineH && b.lineH ? Math.min(a.lineH, b.lineH) : a.lineH ?? b.lineH, seeded: a.seeded || b.seeded, score: a.score + b.score };
          regs.splice(j, 1);
          changed = true;
          break outer;
        }
      }
    }
  }
  // Fóra as zonas que, unidas, se comen a imaxe (o texto en páxina xa o le a primeira pasada).
  return regs
    .filter((r) => area(r.box) <= w * h * 0.6)
    .sort((a, b) => Number(b.seeded) - Number(a.seeded) || b.score - a.score)
    .slice(0, max);
}

export interface GlyphCandidate {
  box: Box;
  /** Bordos por píxel na marxe arredor (0 = illado de todo). */
  ring: number;
}

/** Signos soltos (unha cifra, unha letra: o número de páxina, os eixos «x y z» dun visor 3D...) que non forman palabras e polo tanto non
 *  entran nunha lectura de texto. Búscanse polos seus bordos en cor (non só en brillo: unha letra verde sobre gris case non cambia de
 *  brillo): cada signo é unha mancha de bordos con altura parecida á das letras xa lidas (`lineH`) e rodeada de pouco máis. Non di que
 *  sexa unha letra; iso decídeo despois o OCR, signo a signo. Devolve ata `max`, os máis illados primeiro. */
export function isolatedGlyphs(rgba: Uint8ClampedArray, w: number, h: number, lineH: number, max = 24, radius?: number): GlyphCandidate[] {
  if (w * h > 8_000_000 || lineH < 4) return [];
  const n = w * h;
  const grad = new Uint8Array(n);
  const hist = new Uint32Array(256);
  for (let y = 1; y < h - 1; y++) {
    for (let x = 1; x < w - 1; x++) {
      const i = y * w + x;
      let g = 0;
      for (let c = 0; c < 3; c++) {
        const v = Math.abs(rgba[(i + 1) * 4 + c] - rgba[(i - 1) * 4 + c]) + Math.abs(rgba[(i + w) * 4 + c] - rgba[(i - w) * 4 + c]);
        if (v > g) g = v;
      }
      g = Math.min(255, g);
      grad[i] = g;
      hist[g]++;
    }
  }
  let acc = 0;
  let p = 0;
  for (; p < 255 && acc + hist[p] < n * 0.9; p++) acc += hist[p];
  const T = Math.max(48, p);
  // Imaxe integral dos bordos: contar bordos nunha caixa é O(1).
  const W1 = w + 1;
  const integ = new Int32Array(W1 * (h + 1));
  for (let y = 0; y < h; y++) {
    let row = 0;
    for (let x = 0; x < w; x++) {
      row += grad[y * w + x] >= T ? 1 : 0;
      integ[(y + 1) * W1 + x + 1] = integ[y * W1 + x + 1] + row;
    }
  }
  const count = (x0: number, y0: number, x1: number, y1: number) => {
    x0 = Math.max(0, x0);
    y0 = Math.max(0, y0);
    x1 = Math.min(w, x1);
    y1 = Math.min(h, y1);
    if (x1 <= x0 || y1 <= y0) return 0;
    return integ[y1 * W1 + x1] - integ[y0 * W1 + x1] - integ[y1 * W1 + x0] + integ[y0 * W1 + x0];
  };
  // Os bordos dun signo groso quedan separados uns píxeles (os dous lados do trazo): únense cunha dilatación segundo o tamaño das letras.
  const r = radius ?? Math.max(1, Math.round(lineH * 0.22));
  const dil = new Uint8Array(n);
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) if (count(x - r, y - r, x + r + 1, y + r + 1) > 0) dil[y * w + x] = 1;
  const seen = new Uint8Array(n);
  const out: (GlyphCandidate & { s: number })[] = [];
  const st: number[] = [];
  for (let s0 = 0; s0 < n; s0++) {
    if (!dil[s0] || seen[s0]) continue;
    let x0 = w, y0 = h, x1 = -1, y1 = -1;
    st.push(s0);
    seen[s0] = 1;
    while (st.length) {
      const i = st.pop()!;
      const x = i % w;
      const y = (i / w) | 0;
      if (x < x0) x0 = x;
      if (x > x1) x1 = x;
      if (y < y0) y0 = y;
      if (y > y1) y1 = y;
      for (let dy = -1; dy <= 1; dy++) {
        const yy = y + dy;
        if (yy < 0 || yy >= h) continue;
        for (let dx = -1; dx <= 1; dx++) {
          const xx = x + dx;
          if (xx < 0 || xx >= w) continue;
          const k = yy * w + xx;
          if (dil[k] && !seen[k]) (seen[k] = 1), st.push(k);
        }
      }
    }
    // A caixa dos bordos de verdade: a da mancha dilatada menos a dilatación.
    const bx0 = x0 + r, by0 = y0 + r, bx1 = x1 + 1 - r, by1 = y1 + 1 - r;
    const bw = bx1 - bx0;
    const bh = by1 - by0;
    if (bw < 2 || bh < 6 || bh < lineH * 0.5 || bh > lineH * 3.5 || bw > bh * 1.6 || bw < bh * 0.12) continue;
    const edges = count(bx0, by0, bx1, by1);
    if (edges < 6) continue;
    // Illamento: bordos na marxe (unha beira de ~0,9 alturas arredor) por píxel.
    const m = Math.max(3, Math.round(bh * 0.9));
    const ringEdges = count(bx0 - m, by0 - m, bx1 + m, by1 + m) - edges;
    const ringArea = (Math.min(w, bx1 + m) - Math.max(0, bx0 - m)) * (Math.min(h, by1 + m) - Math.max(0, by0 - m)) - bw * bh;
    const ring = ringArea > 0 ? ringEdges / ringArea : 1;
    if (ring > 0.12) continue;
    // Prefírense os máis illados e de tamaño parecido ao das letras.
    out.push({ box: { x0: bx0, y0: by0, x1: bx1, y1: by1 }, ring, s: ring + Math.abs(Math.log(bh / lineH)) * 0.05 });
  }
  return out.sort((a, b) => a.s - b.s).slice(0, max);
}

/** Parte unha mancha de bordos en signos separados: dentro da caixa, o que se afasta en cor do fondo (a mediana da beira) é tinta, e
 *  cada compoñente conexa de tinta é un signo. Así as letras dun debuxo («x y z» ao pé das liñas dos eixos) saen cada unha á súa
 *  conta. Se non hai máis ca unha, devolve a propia caixa. */
export function segmentGlyphs(rgba: Uint8ClampedArray, w: number, h: number, box: Box, lineH: number): Box[] {
  const pad = 3;
  const x0 = Math.max(0, Math.floor(box.x0) - pad);
  const y0 = Math.max(0, Math.floor(box.y0) - pad);
  const x1 = Math.min(w, Math.ceil(box.x1) + pad);
  const y1 = Math.min(h, Math.ceil(box.y1) + pad);
  const bw = x1 - x0;
  const bh = y1 - y0;
  if (bw < 4 || bh < 4 || bw * bh > 40_000) return [box];
  const px = (x: number, y: number) => ((y0 + y) * w + x0 + x) * 4;
  const ring: number[][] = [[], [], []];
  const take = (x: number, y: number) => {
    const i = px(x, y);
    for (let c = 0; c < 3; c++) ring[c].push(rgba[i + c]);
  };
  for (let x = 0; x < bw; x++) (take(x, 0), take(x, bh - 1));
  for (let y = 0; y < bh; y++) (take(0, y), take(bw - 1, y));
  const bg = ring.map((a) => a.sort((p, q) => p - q)[a.length >> 1]);
  const dist = new Uint8Array(bw * bh);
  const hist = new Uint32Array(256);
  for (let y = 0; y < bh; y++) {
    for (let x = 0; x < bw; x++) {
      const i = px(x, y);
      const d = Math.min(255, Math.max(Math.abs(rgba[i] - bg[0]), Math.abs(rgba[i + 1] - bg[1]), Math.abs(rgba[i + 2] - bg[2])));
      dist[y * bw + x] = d;
      hist[d]++;
    }
  }
  // Otsu
  let sum = 0;
  for (let i = 0; i < 256; i++) sum += i * hist[i];
  let wB = 0, sB = 0, best = 0, thr = 40;
  const total = bw * bh;
  for (let t = 0; t < 256; t++) {
    wB += hist[t];
    if (!wB) continue;
    const wF = total - wB;
    if (!wF) break;
    sB += t * hist[t];
    const v = wB * wF * (sB / wB - (sum - sB) / wF) ** 2;
    if (v > best) (best = v), (thr = t);
  }
  if (thr < 12) return [box];
  const seen = new Uint8Array(total);
  const out: Box[] = [];
  const st: number[] = [];
  for (let s0 = 0; s0 < total; s0++) {
    if (seen[s0] || dist[s0] <= thr) continue;
    let a0 = bw, b0 = bh, a1 = -1, b1 = -1, n = 0;
    st.push(s0);
    seen[s0] = 1;
    while (st.length) {
      const i = st.pop()!;
      const x = i % bw;
      const y = (i / bw) | 0;
      n++;
      if (x < a0) a0 = x;
      if (x > a1) a1 = x;
      if (y < b0) b0 = y;
      if (y > b1) b1 = y;
      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          const xx = x + dx;
          const yy = y + dy;
          if (xx < 0 || yy < 0 || xx >= bw || yy >= bh) continue;
          const k = yy * bw + xx;
          if (!seen[k] && dist[k] > thr) (seen[k] = 1), st.push(k);
        }
      }
    }
    const gw = a1 - a0 + 1;
    const gh = b1 - b0 + 1;
    // Tamaño de letra: nin unha raia (os eixos) nin un tolo.
    if (gh < Math.max(5, lineH * 0.5) || gh > lineH * 3 || gw < 2 || gw > gh * 1.8 || n < gw * gh * 0.2) continue;
    out.push({ x0: x0 + a0, y0: y0 + b0, x1: x0 + a1 + 1, y1: y0 + b1 + 1 });
  }
  return out.length ? out : [box];
}
