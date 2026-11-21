<script lang="ts" context="module">
  /** Debuxos para as opcións dos selectores: cada un di algo que o texto soo non amosa (a forma, a orde, o sentido…). */
  export type GlyphSpec =
    | { k: "ratio"; w: number; h: number } // rectángulo coa proporción ancho:alto
    | { k: "free" } // proporción libre: rectángulo en trazos con asas nas esquinas
    | { k: "grid"; cols: number; rows: number; w?: number; h?: number } // zona dividida; w:h é a forma de cada parte
    | { k: "order"; by: "rows" | "cols" } // numeración das partes
    | { k: "stack"; dir: "v" | "h" } // dúas pezas unha debaixo ou ao carón da outra
    | { k: "turn"; deg: number } // páxina xirada (a frecha marca a parte de arriba)
    | { k: "swatch"; color: string } // mostra de cor
    | { k: "match"; to: "small" | "large"; dir: "v" | "h" } // igualar o ancho (v) ou o alto (h) á peza máis pequena ou á máis grande
    | { k: "fit"; mode: "crop" | "stretch" | "contain" }; // meter unha imaxe nun cadro: cortando polo centro, deformándoa ou deixándoa enteira con marxes
</script>

<script lang="ts">
  export let spec: GlyphSpec;
  const W = 40;
  const H = 30;

  /** Rectángulo de proporción `r` (ancho/alto) que cabe en `mw × mh`, centrado en W × H. */
  function fit(r: number, mw: number, mh: number) {
    let w = mw;
    let h = w / r;
    if (h > mh) {
      h = mh;
      w = h * r;
    }
    return { x: (W - w) / 2, y: (H - h) / 2, w, h };
  }

  $: box = spec.k === "ratio" ? fit(spec.w / spec.h, 26, 22) : spec.k === "grid" ? fit((spec.cols * (spec.w ?? 1)) / (spec.rows * (spec.h ?? 1)), 32, 24) : { x: 0, y: 0, w: 0, h: 0 };
  $: cells = spec.k === "grid" ? { cw: box.w / spec.cols, ch: box.h / spec.rows } : { cw: 0, ch: 0 };
  // Igualar medidas: o debuxo móstrase o resultado. Unha das pezas cambia de tamaño sen cortarse nin deformarse (mesma proporción):
  // o contorno a trazos é o que medía antes e a frecha di cara a onde se escala.
  $: match = spec.k === "match" ? matchShapes(spec.to, spec.dir) : null;
  function matchShapes(to: "small" | "large", dir: "v" | "h") {
    const R = 2.8; // proporción das pezas
    const big = 26;
    const small = 16;
    const target = to === "small" ? small : big;
    const solid: Rc[] = [];
    const ghost: Rc[] = [];
    let arrow: [number, number, number, number];
    if (dir === "v") {
      // pezas unha debaixo da outra, igualadas en ancho; a grande é a de arriba se se reduce e a de abaixo se se amplía
      const x = 7;
      const hBig = big / R;
      const hSmall = small / R;
      const hT = target / R;
      const y1 = 3;
      const y2 = y1 + (to === "small" ? hBig : hT) + 3.5;
      if (to === "small") {
        solid.push({ x, y: y1, w: target, h: hT }, { x, y: y2, w: small, h: hSmall });
        ghost.push({ x, y: y1, w: big, h: hBig });
        arrow = [x + big - 1, y1 + hBig - 0.5, x + small - 0.5, y1 + hT - 0.5];
      } else {
        solid.push({ x, y: y1, w: big, h: hBig }, { x, y: y2, w: target, h: hT });
        ghost.push({ x, y: y2, w: small, h: hSmall });
        arrow = [x + small - 0.5, y2 + hSmall - 0.5, x + big - 1, y2 + hT - 0.5];
      }
    } else {
      // pezas ao carón, igualadas en alto
      const y = 3;
      const wBig = big / R;
      const wSmall = small / R;
      const wT = target / R;
      const x1 = 10;
      const x2 = x1 + (to === "small" ? wBig : wT) + 3.5;
      if (to === "small") {
        solid.push({ x: x1, y, w: wT, h: target }, { x: x2, y, w: wSmall, h: small });
        ghost.push({ x: x1, y, w: wBig, h: big });
        arrow = [x1 + wBig - 0.5, y + big - 1, x1 + wT - 0.5, y + small - 0.5];
      } else {
        solid.push({ x: x1, y, w: wBig, h: big }, { x: x2, y, w: wT, h: target });
        ghost.push({ x: x2, y, w: wSmall, h: small });
        arrow = [x2 + wSmall - 0.5, y + small - 0.5, x2 + wT - 1, y + big - 1];
      }
    }
    return { solid, ghost, arrow };
  }
  type Rc = { x: number; y: number; w: number; h: number };
  /** Frecha de (x1,y1) a (x2,y2) co seu remate. */
  function arrowPath([x1, y1, x2, y2]: [number, number, number, number]) {
    const a = Math.atan2(y2 - y1, x2 - x1);
    const L = 3.4;
    const p = (d: number) => `${(x2 - L * Math.cos(a + d)).toFixed(2)} ${(y2 - L * Math.sin(a + d)).toFixed(2)}`;
    return `M${x1} ${y1} L${x2} ${y2} M${p(0.5)} L${x2} ${y2} L${p(-0.5)}`;
  }
  // Cadrícula de numeración: 3 columnas × 2 filas.
  const nums = (by: "rows" | "cols") => Array.from({ length: 6 }, (_, i) => {
    const c = by === "rows" ? i % 3 : Math.floor(i / 2);
    const r = by === "rows" ? Math.floor(i / 3) : i % 2;
    return { n: i + 1, x: 5 + c * 10 + 5, y: 5 + r * 10 + 5 };
  });
</script>

<svg class="glyph" viewBox="0 0 {W} {H}" width="46" height="34" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round">
  {#if spec.k === "ratio"}
    <rect x={box.x} y={box.y} width={box.w} height={box.h} rx="1.5" fill="currentColor" fill-opacity="0.14" />
  {:else if spec.k === "free"}
    <rect x="7" y="5" width="26" height="20" stroke-dasharray="3 2.4" />
    {#each [[7, 5], [33, 5], [7, 25], [33, 25]] as [x, y]}
      <rect x={x - 2.2} y={y - 2.2} width="4.4" height="4.4" rx="0.8" fill="currentColor" stroke="none" />
    {/each}
  {:else if spec.k === "grid"}
    <rect x={box.x} y={box.y} width={box.w} height={box.h} rx="1.5" fill="currentColor" fill-opacity="0.1" />
    {#each Array.from({ length: spec.cols - 1 }, (_, i) => i + 1) as i}
      {#if cells.cw >= 2}<line x1={box.x + i * cells.cw} y1={box.y} x2={box.x + i * cells.cw} y2={box.y + box.h} stroke-width="1.1" />{/if}
    {/each}
    {#each Array.from({ length: spec.rows - 1 }, (_, i) => i + 1) as i}
      {#if cells.ch >= 2}<line x1={box.x} y1={box.y + i * cells.ch} x2={box.x + box.w} y2={box.y + i * cells.ch} stroke-width="1.1" />{/if}
    {/each}
  {:else if spec.k === "order"}
    <rect x="5" y="5" width="30" height="20" rx="1.5" stroke-width="1.2" />
    <line x1="15" y1="5" x2="15" y2="25" stroke-width="0.9" />
    <line x1="25" y1="5" x2="25" y2="25" stroke-width="0.9" />
    <line x1="5" y1="15" x2="35" y2="15" stroke-width="0.9" />
    {#each nums(spec.by) as c}
      <text x={c.x} y={c.y + 0.4} text-anchor="middle" dominant-baseline="central" font-size="8.5" font-weight="700" fill="currentColor" stroke="none">{c.n}</text>
    {/each}
  {:else if spec.k === "stack"}
    {#if spec.dir === "v"}
      <rect x="9" y="3.5" width="22" height="10" rx="1.5" fill="currentColor" fill-opacity="0.14" />
      <rect x="9" y="16.5" width="22" height="10" rx="1.5" fill="currentColor" fill-opacity="0.14" />
    {:else}
      <rect x="3.5" y="7" width="15" height="16" rx="1.5" fill="currentColor" fill-opacity="0.14" />
      <rect x="21.5" y="7" width="15" height="16" rx="1.5" fill="currentColor" fill-opacity="0.14" />
    {/if}
  {:else if spec.k === "turn"}
    <g transform="rotate({spec.deg} 20 15)">
      <rect x="12.5" y="4.5" width="15" height="21" rx="1.5" fill="currentColor" fill-opacity="0.14" />
      <polygon points="20,8.5 15.8,15.5 24.2,15.5" fill="currentColor" stroke-width="1" />
    </g>
  {:else if spec.k === "match" && match}
    {#each match.ghost as r}<rect x={r.x} y={r.y} width={r.w} height={r.h} rx="1.2" fill="currentColor" fill-opacity="0.05" stroke-dasharray="2.2 1.8" stroke-width="1.1" stroke-opacity="0.75" />{/each}
    {#each match.solid as r}<rect x={r.x} y={r.y} width={r.w} height={r.h} rx="1.2" fill="currentColor" fill-opacity="0.2" />{/each}
    <path d={arrowPath(match.arrow)} stroke="var(--accent)" stroke-width="1.9" />
  {:else if spec.k === "fit"}
    <!-- Os tres levan o mesmo cadro (18 × 18) e o mesmo contido, unha elipse ancha, para que se vexa só o que muda. -->
    <defs><clipPath id="glyph-fit-frame"><rect x="11" y="6" width="18" height="18" rx="1.2" /></clipPath></defs>
    {#if spec.mode === "crop"}
      <!-- o contido sobra polos lados: o que cae fóra do cadro córtase -->
      <rect x="11" y="6" width="18" height="18" rx="1.2" fill="currentColor" fill-opacity="0.14" />
      <ellipse cx="20" cy="15" rx="14.5" ry="9" stroke-dasharray="2.2 1.8" stroke-width="1.1" stroke-opacity="0.75" />
      <ellipse cx="20" cy="15" rx="14.5" ry="9" clip-path="url(#glyph-fit-frame)" />
    {:else if spec.mode === "stretch"}
      <!-- o contido énchese o cadro deformándose: a elipse vólvese círculo -->
      <rect x="11" y="6" width="18" height="18" rx="1.2" fill="currentColor" fill-opacity="0.14" />
      <ellipse cx="20" cy="15" rx="9" ry="9" />
      <path d="M2.5 15 H8 M5.4 12.6 L8 15 L5.4 17.4 M37.5 15 H32 M34.6 12.6 L32 15 L34.6 17.4" stroke="var(--accent)" stroke-width="1.6" />
    {:else}
      <!-- o contido cabe enteiro e sobran marxes (transparentes) arriba e abaixo -->
      <rect x="11" y="6" width="18" height="18" rx="1.2" stroke-dasharray="2.2 1.8" stroke-width="1.1" stroke-opacity="0.75" />
      <ellipse cx="20" cy="15" rx="9" ry="5.2" />
    {/if}
  {:else if spec.k === "swatch"}
    <rect x="11" y="5" width="18" height="20" rx="3" fill={spec.color} stroke="var(--border-strong)" stroke-width="1.2" />
  {/if}
</svg>

<style>
  .glyph { display: block; flex: none; }
</style>
