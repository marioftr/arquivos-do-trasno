<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { trimRange } from "../../files";
  import { t } from "../../i18n";

  /** Liña de tempo: onda opcional, posición actual e, se hai corte, as partes que se perden e as asas para movelo. */
  export let duration: number;
  export let now = 0;
  export let peaks: number[] = [];
  export let height = 56;

  const dispatch = createEventDispatcher<{ seek: number }>();
  let box: HTMLDivElement;
  let canvas: HTMLCanvasElement;

  $: pct = (v: number) => (duration ? Math.min(100, Math.max(0, (v / duration) * 100)) : 0);
  $: if (canvas) draw(peaks, now, duration, $trimRange);

  function draw(p: number[], at: number, dur: number, tr: { start: number; end: number } | null) {
    const w = (canvas.width = Math.max(1, canvas.clientWidth * devicePixelRatio));
    const h = (canvas.height = Math.max(1, canvas.clientHeight * devicePixelRatio));
    const g = canvas.getContext("2d")!;
    g.clearRect(0, 0, w, h);
    if (!p.length) return;
    const css = getComputedStyle(canvas);
    const accent = css.getPropertyValue("--accent").trim() || "#7aa2f7";
    const muted = css.getPropertyValue("--text-muted").trim() || "#9a9aa8";
    const played = dur ? at / dur : 0;
    const bw = w / p.length;
    g.globalAlpha = 1;
    for (let i = 0; i < p.length; i++) {
      const pos = i / p.length;
      const inside = !tr || (pos * dur >= tr.start && pos * dur <= tr.end);
      const bh = Math.max(1.5, p[i] * h * 0.92);
      g.fillStyle = pos <= played ? accent : muted;
      g.globalAlpha = inside ? 1 : 0.25;
      g.fillRect(i * bw, (h - bh) / 2, Math.max(1, bw - 0.8), bh);
    }
  }

  const timeAt = (ev: PointerEvent) => {
    const r = box.getBoundingClientRect();
    return Math.min(duration, Math.max(0, ((ev.clientX - r.left) / r.width) * duration));
  };

  type Mode = "seek" | "start" | "end" | null;
  let mode: Mode = null;
  function down(ev: PointerEvent, m: Mode) {
    if (!duration) return;
    mode = m;
    (ev.currentTarget as HTMLElement).setPointerCapture(ev.pointerId);
    ev.stopPropagation();
    if (m === "seek") dispatch("seek", timeAt(ev));
  }
  function move(ev: PointerEvent) {
    if (!mode) return;
    const v = timeAt(ev);
    if (mode === "seek") dispatch("seek", v);
    else if ($trimRange) {
      const r = $trimRange;
      if (mode === "start") trimRange.set({ start: Math.min(v, r.end - 0.1), end: r.end });
      else trimRange.set({ start: r.start, end: Math.max(v, r.start + 0.1) });
      dispatch("seek", v);
    }
  }
  const up = () => (mode = null);
  const round = (v: number) => Math.round(v * 10) / 10;
</script>

<div class="tl" style={`height:${height}px`} bind:this={box} on:pointerdown={(e) => down(e, "seek")} on:pointermove={move} on:pointerup={up} on:pointercancel={up} role="presentation">
  <canvas bind:this={canvas}></canvas>
  {#if $trimRange && duration}
    <div class="cut" style={`left:0;width:${pct($trimRange.start)}%`}></div>
    <div class="cut" style={`left:${pct($trimRange.end)}%;right:0`}></div>
    <div class="keep" style={`left:${pct($trimRange.start)}%;width:${pct($trimRange.end) - pct($trimRange.start)}%`}></div>
    <div class="handle" style={`left:${pct($trimRange.start)}%`} on:pointerdown={(e) => down(e, "start")} on:pointermove={move} on:pointerup={up} role="presentation" title={$t("Inicio do corte")}>
      <span class="tag">{round($trimRange.start)} s</span>
    </div>
    <div class="handle" style={`left:${pct($trimRange.end)}%`} on:pointerdown={(e) => down(e, "end")} on:pointermove={move} on:pointerup={up} role="presentation" title={$t("Fin do corte")}>
      <span class="tag">{round($trimRange.end)} s</span>
    </div>
  {/if}
  <div class="head" style={`left:${pct(now)}%`}></div>
</div>

<style>
  .tl { position: relative; width: 100%; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); cursor: pointer; touch-action: none; user-select: none; overflow: visible; }
  canvas { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
  .cut { position: absolute; top: 0; bottom: 0; background: repeating-linear-gradient(45deg, rgba(10, 10, 14, 0.7) 0 6px, rgba(10, 10, 14, 0.55) 6px 12px); pointer-events: none; }
  .keep { position: absolute; top: 0; bottom: 0; border-top: 2px solid var(--accent); border-bottom: 2px solid var(--accent); pointer-events: none; }
  .head { position: absolute; top: -3px; bottom: -3px; width: 2px; margin-left: -1px; background: var(--text); box-shadow: 0 0 0 1px var(--bg); pointer-events: none; }
  .handle { position: absolute; top: -4px; bottom: -4px; width: 14px; margin-left: -7px; cursor: ew-resize; display: flex; justify-content: center; }
  .handle::before { content: ""; width: 4px; height: 100%; border-radius: 2px; background: var(--accent); box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.45); }
  .tag { position: absolute; bottom: calc(100% + 2px); font-size: var(--fs-xs); background: rgba(0, 0, 0, 0.75); color: #fff; padding: 1px 5px; border-radius: 4px; white-space: nowrap; pointer-events: none; }
</style>
