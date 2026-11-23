<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  /** O trasno en grande, vivo: pestanexa de cando en cando e, se o tocas, bota un pulo e queda sorprendido. Os fotogramas son os mesmos do programa. */
  export let size = 160;

  const F = { base: "/trasno/base.png", half: "/trasno/medio.png", shut: "/trasno/pechado.png", mid: "/trasno/intermedio.png", wow: "/trasno/sorpresa.png" };
  const FRAMES = Object.values(F);
  type Step = [string, number];
  const BLINK: Step[] = [[F.half, 80], [F.shut, 110], [F.half, 80], [F.base, 0]];
  const DOUBLE: Step[] = [[F.half, 70], [F.shut, 90], [F.half, 70], [F.base, 130], [F.half, 70], [F.shut, 90], [F.half, 70], [F.base, 0]];
  const STARTLE: Step[] = [[F.half, 80], [F.shut, 100], [F.half, 80], [F.mid, 110], [F.wow, 1100], [F.mid, 110], [F.base, 0]];

  let frame = F.base;
  let jump = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let busy = false;
  let still = false;
  let alive = true;

  const wait = (ms: number) => new Promise<void>((r) => (timer = setTimeout(r, ms)));
  async function play(seq: Step[]) {
    busy = true;
    for (const [f, ms] of seq) {
      if (!alive) return;
      frame = f;
      if (ms) await wait(ms);
    }
    busy = false;
  }
  // Pestanexa cada 2–5 segundos; unha de cada catro veces, dúas seguidas.
  async function loop() {
    while (alive && !still) {
      await wait(2000 + Math.random() * 3000);
      if (!alive || still) return;
      if (!busy) await play(Math.random() < 0.25 ? DOUBLE : BLINK);
    }
  }
  async function poke() {
    if (still || busy) return;
    jump = true;
    setTimeout(() => (jump = false), 450);
    await play(STARTLE);
  }

  onMount(() => {
    const m = matchMedia("(prefers-reduced-motion: reduce)");
    still = m.matches;
    // Todos os fotogramas descodificados de antemán: así a pestana non parpadea ao cambiar.
    for (const src of FRAMES) new Image().src = src;
    if (!still) void loop();
  });
  onDestroy(() => {
    alive = false;
    clearTimeout(timer);
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="trasno" class:jump on:click={poke} aria-hidden="true" style={`--s:${size}px`}>
  <span class="bob">
    {#each FRAMES as src (src)}
      <img {src} alt="" width={size} height={size} draggable="false" class:on={frame === src} />
    {/each}
  </span>
  <span class="shadow"></span>
</div>

<style>
  .trasno { position: relative; display: block; flex: none; width: var(--s); cursor: pointer; -webkit-tap-highlight-color: transparent; user-select: none; }
  .bob { position: relative; display: block; width: var(--s); height: var(--s); animation: bob 3.2s ease-in-out infinite; }
  .jump .bob { animation: hop 0.45s cubic-bezier(0.2, 0.8, 0.3, 1); }
  img { position: absolute; inset: 0; width: 100%; height: 100%; image-rendering: pixelated; opacity: 0; pointer-events: none; }
  img.on { opacity: 1; }
  .shadow { display: block; width: 60%; height: calc(var(--s) * 0.06); margin: calc(var(--s) * 0.04) auto 0; border-radius: 50%; background: var(--text); opacity: 0.14; animation: shade 3.2s ease-in-out infinite; }
  @keyframes bob { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(calc(var(--s) * -0.05)); } }
  @keyframes hop { 0% { transform: translateY(0); } 40% { transform: translateY(calc(var(--s) * -0.2)); } 100% { transform: translateY(0); } }
  @keyframes shade { 0%, 100% { transform: scaleX(1); opacity: 0.14; } 50% { transform: scaleX(0.88); opacity: 0.09; } }
</style>
