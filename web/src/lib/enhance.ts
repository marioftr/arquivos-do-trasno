/** Restaurar fotos sen IA xerativa: axustes de luz, cor e ruído feitos píxel a píxel no navegador (nada se inventa: só se corrixe o que xa hai). */

export type Mode = "auto" | "light" | "doc" | "bw" | "denoise" | "sharp";
export type Amount = "soft" | "normal" | "strong";

/** Píxeles máximos que se tratan (o móbil aguanta mal máis, con varias copias en memoria). */
export const MAX_AREA = 12_000_000;

const clamp = (v: number) => (v < 0 ? 0 : v > 255 ? 255 : v);
const luma = (r: number, g: number, b: number) => 0.299 * r + 0.587 * g + 0.114 * b;

/** Desenfoque de caixa de 3×3 (separable) nas tres canles de cor; o alfa non se toca. */
function blur3(px: Uint8ClampedArray, w: number, h: number): Uint8ClampedArray {
  const tmp = new Uint8ClampedArray(px.length);
  const out = new Uint8ClampedArray(px.length);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4;
      const a = (y * w + Math.max(0, x - 1)) * 4;
      const c = (y * w + Math.min(w - 1, x + 1)) * 4;
      for (let k = 0; k < 3; k++) tmp[i + k] = (px[a + k] + px[i + k] + px[c + k]) / 3;
      tmp[i + 3] = px[i + 3];
    }
  }
  for (let y = 0; y < h; y++) {
    const up = Math.max(0, y - 1);
    const dn = Math.min(h - 1, y + 1);
    for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4;
      const a = (up * w + x) * 4;
      const c = (dn * w + x) * 4;
      for (let k = 0; k < 3; k++) out[i + k] = (tmp[a + k] + tmp[i + k] + tmp[c + k]) / 3;
      out[i + 3] = tmp[i + 3];
    }
  }
  return out;
}

/** Estira a luz: o 0,5 % máis escuro pasa a negro e o 0,5 % máis claro a branco, e a cor equilíbrase un pouco (gris medio). */
function auto(d: ImageData) {
  const px = d.data;
  const n = px.length / 4;
  const hist = new Uint32Array(256);
  let sr = 0;
  let sg = 0;
  let sb = 0;
  for (let i = 0; i < px.length; i += 4) {
    hist[Math.round(luma(px[i], px[i + 1], px[i + 2]))]++;
    sr += px[i];
    sg += px[i + 1];
    sb += px[i + 2];
  }
  let lo = 0;
  let hi = 255;
  for (let acc = 0; lo < 255 && (acc += hist[lo]) <= n * 0.005; lo++);
  for (let acc = 0; hi > 0 && (acc += hist[hi]) <= n * 0.005; hi--);
  if (hi - lo < 40) hi = Math.min(255, lo + 40);
  const mean = (sr + sg + sb) / (3 * n);
  // Equilibrio de cor: acerca cada canle á media, sen pasarse (un cambio de ata un 12 %).
  const bal = [sr, sg, sb].map((s) => Math.max(0.88, Math.min(1.12, 1 + (mean / (s / n) - 1) * 0.5)));
  const span = hi - lo;
  for (let i = 0; i < px.length; i += 4) {
    let r = clamp(((px[i] * bal[0] - lo) / span) * 255);
    let g = clamp(((px[i + 1] * bal[1] - lo) / span) * 255);
    let b = clamp(((px[i + 2] * bal[2] - lo) / span) * 255);
    // Un pouco máis de cor.
    const l = luma(r, g, b);
    r = clamp(l + (r - l) * 1.096);
    g = clamp(l + (g - l) * 1.096);
    b = clamp(l + (b - l) * 1.096);
    px[i] = r;
    px[i + 1] = g;
    px[i + 2] = b;
  }
}

/** Levanta as sombras sen queimar as luces (curva gamma). */
function light(d: ImageData) {
  const lut = new Uint8ClampedArray(256);
  for (let v = 0; v < 256; v++) lut[v] = 255 * Math.pow(v / 255, 0.72);
  const px = d.data;
  for (let i = 0; i < px.length; i += 4) (px[i] = lut[px[i]]), (px[i + 1] = lut[px[i + 1]]), (px[i + 2] = lut[px[i + 2]]);
}

/** Pasa a gris e, se `flat`, iguala a iluminación (divide pola luz de fondo): páxinas fotografadas con sombras quedan brancas e o texto negro. */
function gray(d: ImageData, flat: boolean) {
  const { width: w, height: h, data: px } = d;
  const g = new Float32Array(w * h);
  for (let i = 0, j = 0; i < px.length; i += 4, j++) g[j] = luma(px[i], px[i + 1], px[i + 2]);
  let out = g;
  if (flat) {
    // Fondo: media por bloques sobre unha imaxe pequena, suavizada e ampliada con interpolación.
    const f = Math.max(1, Math.ceil(Math.max(w, h) / 240));
    const sw = Math.ceil(w / f);
    const sh = Math.ceil(h / f);
    let small = new Float32Array(sw * sh);
    const cnt = new Uint16Array(sw * sh);
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) (small[Math.floor(y / f) * sw + Math.floor(x / f)] += g[y * w + x]), cnt[Math.floor(y / f) * sw + Math.floor(x / f)]++;
    for (let i = 0; i < small.length; i++) small[i] /= cnt[i] || 1;
    // O texto fino non debe contar como fondo: toma o máximo local (dilatación) antes de suavizar.
    for (let pass = 0; pass < 2; pass++) {
      const next = new Float32Array(small.length);
      for (let y = 0; y < sh; y++)
        for (let x = 0; x < sw; x++) {
          let m = 0;
          for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) m = Math.max(m, small[Math.min(sh - 1, Math.max(0, y + dy)) * sw + Math.min(sw - 1, Math.max(0, x + dx))]);
          next[y * sw + x] = m;
        }
      small = next;
    }
    for (let pass = 0; pass < 3; pass++) {
      const next = new Float32Array(small.length);
      for (let y = 0; y < sh; y++)
        for (let x = 0; x < sw; x++) {
          let s = 0;
          for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) s += small[Math.min(sh - 1, Math.max(0, y + dy)) * sw + Math.min(sw - 1, Math.max(0, x + dx))];
          next[y * sw + x] = s / 9;
        }
      small = next;
    }
    out = new Float32Array(w * h);
    for (let y = 0; y < h; y++) {
      const fy = Math.min(sh - 1, Math.max(0, y / f - 0.5));
      const y0 = Math.floor(fy);
      const y1 = Math.min(sh - 1, y0 + 1);
      const ty = fy - y0;
      for (let x = 0; x < w; x++) {
        const fx = Math.min(sw - 1, Math.max(0, x / f - 0.5));
        const x0 = Math.floor(fx);
        const x1 = Math.min(sw - 1, x0 + 1);
        const tx = fx - x0;
        const bg = (small[y0 * sw + x0] * (1 - tx) + small[y0 * sw + x1] * tx) * (1 - ty) + (small[y1 * sw + x0] * (1 - tx) + small[y1 * sw + x1] * tx) * ty;
        out[y * w + x] = Math.min(255, (g[y * w + x] * 255) / Math.max(bg, 24));
      }
    }
  }
  // Niveis: o papel pasa a branco e o texto escurece.
  let lo = 0;
  let hi = 255;
  if (flat) (lo = 85), (hi = 225);
  else {
    const hist = new Uint32Array(256);
    for (const v of out) hist[Math.round(v)]++;
    const n = out.length;
    for (let acc = 0; lo < 255 && (acc += hist[lo]) <= n * 0.005; lo++);
    for (let acc = 0; hi > 0 && (acc += hist[hi]) <= n * 0.005; hi--);
    if (hi - lo < 40) hi = Math.min(255, lo + 40);
  }
  for (let i = 0, j = 0; i < px.length; i += 4, j++) {
    const v = clamp(((out[j] - lo) / (hi - lo)) * 255);
    px[i] = px[i + 1] = px[i + 2] = v;
  }
}

const NOISE: Record<Amount, { limit: number; passes: number }> = { soft: { limit: 14, passes: 1 }, normal: { limit: 26, passes: 1 }, strong: { limit: 40, passes: 2 } };

/** Quita ruído suavizando só onde a diferenza co contorno é pequena: as beiras e o detalle fino quedan. */
function denoise(d: ImageData, amount: Amount) {
  const { limit, passes } = NOISE[amount];
  const px = d.data;
  for (let p = 0; p < passes; p++) {
    const soft = blur3(px, d.width, d.height);
    for (let i = 0; i < px.length; i += 4) {
      for (let k = 0; k < 3; k++) {
        const diff = Math.abs(px[i + k] - soft[i + k]);
        const w = diff >= limit ? 0 : 1 - diff / limit;
        px[i + k] = px[i + k] + (soft[i + k] - px[i + k]) * w;
      }
    }
  }
}

const SHARP: Record<Amount, number> = { soft: 0.5, normal: 0.9, strong: 1.5 };

/** Máscara de enfoque: realza a diferenza co desenfoque, sen tocar as zonas lisas (ruído) por debaixo dun limiar. */
function sharpen(d: ImageData, amount: Amount) {
  const px = d.data;
  const soft = blur3(blur3(px, d.width, d.height), d.width, d.height);
  const k = SHARP[amount];
  for (let i = 0; i < px.length; i += 4) {
    for (let c = 0; c < 3; c++) {
      const diff = px[i + c] - soft[i + c];
      if (Math.abs(diff) > 3) px[i + c] = clamp(px[i + c] + diff * k);
    }
  }
}

/** Aplica o axuste elixido aos píxeles. */
export function enhance(d: ImageData, mode: Mode, amount: Amount) {
  if (mode === "auto") auto(d);
  else if (mode === "light") light(d);
  else if (mode === "doc") gray(d, true);
  else if (mode === "bw") gray(d, false);
  else if (mode === "denoise") denoise(d, amount);
  else sharpen(d, amount);
}
