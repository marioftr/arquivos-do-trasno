<script lang="ts">
  /** Regra de ampliación: escala logarítmica de 12,5 % a 800 %, co 100 % xusto no medio. Marcas visibles nos
   *  valores habituais; ao arrastrar pégase a elas se está preto. Teclado: frechas, Inicio (100 %) e Fin (o máximo). */
  export let value = 100;
  export let onChange: (percent: number) => void;
  export let label = "";
  /** Límites e marcas da regra. Por defecto, os das imaxes; cada visor pode pasar os seus. */
  export let min = 12.5;
  export let max = 800;
  export let ticks: number[] = [12.5, 25, 50, 100, 200, 400, 800];
  export let labeled: number[] = [25, 50, 100, 200, 400, 800];

  const pos = (z: number) => Math.max(0, Math.min(1, Math.log(Math.max(min, z) / min) / Math.log(max / min)));
  const zoomAt = (p: number) => min * Math.exp(Math.max(0, Math.min(1, p)) * Math.log(max / min));
  $: at = pos(value) * 100;

  let rail: HTMLDivElement;
  let dragging = false;
  function fromEvent(ev: PointerEvent) {
    const r = rail.getBoundingClientRect();
    let p = (ev.clientX - r.left) / r.width;
    // Pégase ás marcas se se solta a menos dun 2,5 % do longo da regra.
    for (const t of ticks) if (Math.abs(pos(t) - p) < 0.025) p = pos(t);
    onChange(zoomAt(p));
  }
  function down(ev: PointerEvent) {
    if (ev.button !== 0) return;
    dragging = true;
    (ev.currentTarget as HTMLElement).setPointerCapture(ev.pointerId);
    fromEvent(ev);
  }
  function move(ev: PointerEvent) {
    if (dragging) fromEvent(ev);
  }
  function up() {
    dragging = false;
  }
  function key(ev: KeyboardEvent) {
    const step = ev.shiftKey ? 0.1 : 0.03;
    if (ev.key === "ArrowRight" || ev.key === "ArrowUp") onChange(zoomAt(pos(value) + step));
    else if (ev.key === "ArrowLeft" || ev.key === "ArrowDown") onChange(zoomAt(pos(value) - step));
    else if (ev.key === "Home") onChange(100);
    else if (ev.key === "End") onChange(max);
    else return;
    ev.preventDefault();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="zr" role="slider" tabindex="0" aria-label={label} aria-valuemin={min} aria-valuemax={max} aria-valuenow={Math.round(value)} aria-valuetext={`${Math.round(value)} %`} title={label} on:pointerdown={down} on:pointermove={move} on:pointerup={up} on:pointercancel={up} on:keydown={key}>
  <div class="rail" bind:this={rail}>
    <div class="line"></div>
    <div class="fill" style={`width:${at}%`}></div>
    {#each ticks as t}
      <span class="tick" class:major={labeled.includes(t)} class:on={t === 100} style={`left:${pos(t) * 100}%`}>
        {#if labeled.includes(t)}<span class="tl">{t}</span>{/if}
      </span>
    {/each}
    <span class="thumb" class:drag={dragging} style={`left:${at}%`}></span>
  </div>
</div>

<style>
  .zr { position: relative; flex: none; width: 230px; height: 34px; padding: 0 14px; box-sizing: border-box; cursor: pointer; touch-action: none; user-select: none; border-radius: var(--radius-sm); }
  .zr:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  .rail { position: relative; height: 100%; }
  .line { position: absolute; left: 0; right: 0; top: 11px; height: 4px; border-radius: 4px; background: var(--border-strong); }
  .fill { position: absolute; left: 0; top: 11px; height: 4px; border-radius: 4px; background: color-mix(in srgb, var(--accent) 70%, var(--border-strong)); pointer-events: none; }
  .tick { position: absolute; top: 18px; width: 1px; height: 4px; background: var(--text-faint); transform: translateX(-0.5px); }
  .tick.major { height: 6px; background: var(--text-muted); }
  .tick.on { height: 8px; background: var(--text); }
  .tl { position: absolute; top: 8px; left: 50%; transform: translateX(-50%); font-size: 0.62rem; line-height: 1; color: var(--text-faint); font-variant-numeric: tabular-nums; pointer-events: none; }
  .tick.on .tl { color: var(--text); font-weight: 600; top: 10px; }
  .thumb { position: absolute; top: 6px; width: 14px; height: 14px; margin-left: -7px; border-radius: 50%; background: var(--accent); border: 2px solid var(--bg-rail); box-shadow: 0 1px 3px rgba(0, 0, 0, 0.5); box-sizing: border-box; transition: transform 0.12s; pointer-events: none; }
  .thumb.drag { transform: scale(1.2); }
</style>
