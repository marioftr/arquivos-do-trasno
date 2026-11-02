<script lang="ts">
  import { onDestroy } from "svelte";

  /** O trasno en pixel art. Os fotogramas son provisionais (só para probar o sitio e o movemento):
   *  a arte definitiva substitúe `BASE` e as variantes sen tocar o resto da aplicación. */
  export let state: "idle" | "working" | "done" | "warn" = "idle";
  /** Tamaño total en px (a figura é de 16 × 16 píxeles de arte, así que convén un múltiplo de 16). */
  export let size = 32;

  const PAL: Record<string, string> = {
    h: "#4aa84a",
    H: "#2f7a35",
    s: "#f0c9a0",
    e: "#1d1d1d",
    c: "#e39a8a",
    b: "#dedede",
    m: "#a04030",
    w: "#6ec1ff",
  };
  const BASE = [
    ".......hh.......",
    "......hhhh......",
    ".....hhhhhh.....",
    "....hhhhhhhh....",
    "...hhhhhhhhhh...",
    "..HHHHHHHHHHHH..",
    "...ssssssssss...",
    "..ssssssssssss..",
    "..sseesssseess..",
    "..ssssssssssss..",
    "..sscsssssscss..",
    "..bbbbbbbbbbbb..",
    "...bbbbmmbbbb...",
    "....bbbbbbbb....",
    ".....bbbbbb.....",
    "......bbbb......",
  ];
  const patch = (rows: Record<number, string>) => BASE.map((r, i) => rows[i] ?? r);
  const BLINK = patch({ 8: "..ssssssssssss.." });
  const WARN = patch({ 6: "...sssssssssw...", 7: "..sssssssssssw..", 12: "...bbbbmmbbbb..." });
  const DONE = patch({ 8: "..ssesssssesss..", 12: "...bbbmmmmbbb..." });

  // Cada estado é unha secuencia de (fotograma, desprazamento vertical, milisegundos).
  type Step = [string[], number, number];
  const SEQ: Record<string, Step[]> = {
    idle: [[BASE, 0, 2600], [BLINK, 0, 140]],
    working: [[BASE, 0, 260], [BASE, 1, 260]],
    done: [[DONE, -1, 300], [DONE, 0, 300]],
    warn: [[WARN, 0, 420], [WARN, 1, 420]],
  };

  let i = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const still = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  $: seq = SEQ[state] ?? SEQ.idle;
  $: if (state) i = 0;
  function tick() {
    clearTimeout(timer);
    if (still) return;
    timer = setTimeout(() => {
      i = (i + 1) % seq.length;
      tick();
    }, seq[i % seq.length][2]);
  }
  $: seq, tick();
  onDestroy(() => clearTimeout(timer));

  $: [rows, dy] = seq[i % seq.length];
  $: cells = rows.flatMap((r, y) => [...r].map((c, x) => ({ c, x, y }))).filter((p) => p.c !== ".");
</script>

<svg class="sprite" width={size} height={size} viewBox="0 0 16 16" shape-rendering="crispEdges" aria-hidden="true">
  <g transform={`translate(0 ${dy})`}>
    {#each cells as p}<rect x={p.x} y={p.y} width="1" height="1" fill={PAL[p.c]} />{/each}
  </g>
</svg>

<style>
  .sprite { display: block; image-rendering: pixelated; flex-shrink: 0; overflow: visible; }
</style>
