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
    | { k: "match"; to: "small" | "large"; dir: "v" | "h" }; // igualar o ancho (v) ou o alto (h) á peza máis pequena ou á máis grande
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
  // Igualar medidas: dúas pezas de distinto ancho (ou alto) quedan como están; unha liña de guía marca a medida á que se igualan
  // (a da máis pequena ou a da máis grande). O que sobra da grande vai atenuado; o que falta á pequena, a trazos.
  $: match = spec.k === "match" ? matchShapes(spec.to, spec.dir) : null;
  function matchShapes(to: "small" | "large", dir: "v" | "h") {
    const big = dir === "v" ? 28 : 22;
    const small = dir === "v" ? 18 : 14;
    const at = dir === "v" ? 6 : 4; // onde empezan as pezas no eixe da medida
    const guide = at + (to === "small" ? small : big);
    if (dir === "v") {
      return {
        solid: [{ x: at, y: 4, w: to === "small" ? small : big, h: 9 }, { x: at, y: 17, w: small, h: 9 }],
        faded: to === "small" ? [{ x: guide, y: 4, w: big - small, h: 9 }] : [],
        dashed: to === "large" ? [{ x: at + small, y: 17, w: big - small, h: 9 }] : [],
        line: { x1: guide, y1: 2, x2: guide, y2: 28 },
      };
    }
    return {
      solid: [{ x: 5, y: at, w: 14, h: to === "small" ? small : big }, { x: 21, y: at, w: 14, h: small }],
      faded: to === "small" ? [{ x: 5, y: guide, w: 14, h: big - small }] : [],
      dashed: to === "large" ? [{ x: 21, y: at + small, w: 14, h: big - small }] : [],
      line: { x1: 3, y1: guide, x2: 37, y2: guide },
    };
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
    {#each match.solid as r}<rect x={r.x} y={r.y} width={r.w} height={r.h} rx="1.5" fill="currentColor" fill-opacity="0.14" />{/each}
    {#each match.faded as r}<rect x={r.x} y={r.y} width={r.w} height={r.h} rx="1.5" fill="currentColor" fill-opacity="0.06" stroke-opacity="0.35" />{/each}
    {#each match.dashed as r}<rect x={r.x} y={r.y} width={r.w} height={r.h} rx="1.5" stroke-dasharray="2.4 2" stroke-width="1.1" />{/each}
    <line x1={match.line.x1} y1={match.line.y1} x2={match.line.x2} y2={match.line.y2} stroke="var(--accent)" stroke-width="1.6" />
  {:else if spec.k === "swatch"}
    <rect x="11" y="5" width="18" height="20" rx="3" fill={spec.color} stroke="var(--border-strong)" stroke-width="1.2" />
  {/if}
</svg>

<style>
  .glyph { display: block; flex: none; }
</style>
