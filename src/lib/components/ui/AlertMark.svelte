<script lang="ts">
  /** Sinal que salta un momento sobre o trasno cando hai un aviso. Pixel art, como o propio trasno:
   *  «error» (exclamación vermella), «warn» (triángulo ámbar) e «update» (frecha de descarga). */
  export let kind: "error" | "warn" | "update" = "error";

  // Corpo redondo de 16 × 15 píxeles: o = contorno, H = brillo, F = cor, S = sombra.
  const BODY = [
    "....oooooooo....",
    "..ooHHFFFFFFoo..",
    ".oHHFFFFFFFFFFo.",
    "oHFFFFFFFFFFFFFo",
    "oHFFFFFFFFFFFFFo",
    "oFFFFFFFFFFFFFFo",
    "oFFFFFFFFFFFFFFo",
    "oFFFFFFFFFFFFFFo",
    "oFFFFFFFFFFFFFFo",
    "oFFFFFFFFFFFFFFo",
    "oFFFFFFFFFFFFFFo",
    "oFFFFFFFFFFFFFSo",
    ".oSSFFFFFFFFSSo.",
    "..ooSSSSSSSSoo..",
    "....oooooooo....",
  ];
  // Triángulo: calculado por filas (cada fila ensancha medio píxel por lado).
  const TRI = Array.from({ length: 14 }, (_, y) => {
    const half = Math.floor(y * 0.55) + 1;
    const l = 8 - half;
    const r = 7 + half;
    return Array.from({ length: 16 }, (_, x) => (x < l || x > r ? "." : x === l || x === r || y === 13 ? "o" : x === l + 1 && y > 2 ? "H" : y === 12 ? "S" : "F")).join("");
  });
  // Marcas (x, y, ancho, alto) que se pintan por riba: o signo de cada caso.
  const MARKS: Record<string, number[][]> = {
    error: [[7, 3, 2, 5], [7, 9, 2, 2]],
    warn: [[7, 5, 2, 4], [7, 10, 2, 2]],
    update: [[7, 3, 2, 5], [5, 7, 6, 1], [6, 8, 4, 1], [7, 9, 2, 1], [4, 11, 8, 1]],
  };
  $: rows = kind === "warn" ? TRI : BODY;
  const PAD_X = 4;
  const PAD_Y = 6;
</script>

<svg class="mark {kind}" viewBox="0 0 24 21" width="48" height="42" shape-rendering="crispEdges" aria-hidden="true">
  <!-- raios que saltan ao aparecer -->
  <g class="rays">
    <rect x="11" y="0" width="2" height="3" />
    <rect x="3" y="2" width="2" height="2" />
    <rect x="19" y="2" width="2" height="2" />
    <rect x="1" y="6" width="2" height="1" />
    <rect x="21" y="6" width="2" height="1" />
  </g>
  <g transform={`translate(${PAD_X} ${PAD_Y})`}>
    {#each rows as row, y}
      {#each [...row] as c, x}
        {#if c !== "."}<rect class={`p-${c}`} x={x} y={y} width="1.02" height="1.02" />{/if}
      {/each}
    {/each}
    {#each MARKS[kind] as [x, y, w, h]}
      <rect class="sign" {x} {y} width={w} height={h} />
    {/each}
  </g>
</svg>

<style>
  .mark { display: block; overflow: visible; filter: drop-shadow(0 2px 0 rgba(0, 0, 0, 0.35)); transform-origin: 50% 85%; animation: pop 3.4s ease-out forwards; }
  .error { --o: #4d1015; --H: #ff9a8f; --F: #e5463f; --S: #b02620; --sign: #fff; --ray: #ff6a5f; }
  .warn { --o: #4f3406; --H: #ffe08a; --F: #f4b73e; --S: #c98b1b; --sign: #3d2803; --ray: #ffc94d; }
  .update { --o: color-mix(in srgb, var(--accent) 30%, #000); --H: color-mix(in srgb, var(--accent) 50%, #fff); --F: var(--accent); --S: color-mix(in srgb, var(--accent) 70%, #000); --sign: #fff; --ray: var(--accent); }
  .p-o { fill: var(--o); }
  .p-H { fill: var(--H); }
  .p-F { fill: var(--F); }
  .p-S { fill: var(--S); }
  .sign { fill: var(--sign); }
  .rays rect { fill: var(--ray); }
  .rays { animation: rays 0.9s steps(2, end) 2 0.12s both; }
  /* Entra de golpe, tembla coma un susto e vaise */
  @keyframes pop {
    0% { transform: scale(0.2) translateY(10px); opacity: 0; }
    8% { transform: scale(1.3) translateY(-8px) rotate(0deg); opacity: 1; }
    14% { transform: scale(0.95) translateY(0) rotate(-8deg); }
    20% { transform: scale(1) rotate(8deg); }
    26% { transform: rotate(-6deg); }
    32% { transform: rotate(6deg); }
    38% { transform: rotate(-3deg); }
    44% { transform: rotate(0deg); }
    82% { transform: scale(1) translateY(-2px); opacity: 1; }
    100% { transform: scale(0.8) translateY(-10px); opacity: 0; }
  }
  @keyframes rays {
    0% { opacity: 0; }
    30% { opacity: 1; }
    100% { opacity: 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .mark { animation: fade 3.4s ease-out forwards; }
    .rays { display: none; }
    @keyframes fade { 0%, 80% { opacity: 1; } 100% { opacity: 0; } }
  }
</style>
