<script lang="ts">
  import { onDestroy } from "svelte";

  /** O trasno en pixel art (32 × 32). Os fotogramas viven en `public/trasno/`: substituír os PNG
   *  (mesmos nomes) cambia a arte en toda a aplicación sen tocar nada máis. */
  export let state: "idle" | "working" | "done" | "warn" | "notice" = "idle";
  /** Tamaño en px; convén un múltiplo de 32 para que os píxeles queden nítidos. */
  export let size = 32;

  const F = { base: "/trasno/base.png", half: "/trasno/medio.png", shut: "/trasno/pechado.png", mid: "/trasno/intermedio.png", wow: "/trasno/sorpresa.png" };

  // Cada estado é unha secuencia de (fotograma, desprazamento vertical, milisegundos).
  type Step = [string, number, number];
  const SEQ: Record<string, Step[]> = {
    idle: [[F.base, 0, 2800], [F.half, 0, 90], [F.shut, 0, 110], [F.half, 0, 90]],
    working: [[F.base, 0, 260], [F.half, 0, 90], [F.base, -1, 260]],
    done: [[F.base, -2, 260], [F.base, 0, 260]],
    // Un aviso: pestanexa, o fotograma de transición e queda sorprendido (milisegundos 0 = quedar nese fotograma).
    notice: [[F.base, 0, 140], [F.half, 0, 80], [F.shut, 0, 100], [F.half, 0, 80], [F.mid, 0, 110], [F.wow, 0, 0]],
  };
  SEQ.warn = SEQ.notice;

  let i = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const still = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  $: seq = SEQ[state] ?? SEQ.idle;
  $: if (state) i = still && (state === "notice" || state === "warn") ? seq.length - 1 : 0;
  function tick() {
    clearTimeout(timer);
    if (still) return;
    const ms = seq[i % seq.length][2];
    if (!ms) return;
    timer = setTimeout(() => {
      i = (i + 1) % seq.length;
      tick();
    }, ms);
  }
  $: seq, tick();
  onDestroy(() => clearTimeout(timer));

  $: [src, dy] = seq[i % seq.length];
</script>

<img class="sprite" {src} width={size} height={size} alt="" aria-hidden="true" draggable="false" style={`transform: translateY(${(dy * size) / 32}px)`} />

<style>
  .sprite { display: block; image-rendering: pixelated; flex-shrink: 0; user-select: none; }
</style>
