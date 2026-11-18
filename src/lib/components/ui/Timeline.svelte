<script lang="ts">
  import { formatClock } from "../../formats";
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
    const muted = css.getPropertyValue("--text-faint").trim() || "#6c6c78";
    const played = dur ? at / dur : 0;
    // Barras redondeadas e simétricas (3 px con 2 de aire): cada unha leva o pico máis alto do seu tramo, realzado coa raíz para que o
    // son baixo tamén se vexa. Onde hai silencio queda un punto, non un oco.
    const unit = devicePixelRatio;
    const bar = 3 * unit;
    const gap = 2 * unit;
    const n = Math.max(1, Math.floor((w + gap) / (bar + gap)));
    const pad = 0.12 * h;
    const maxH = h - pad * 2;
    const radius = bar / 2;
    for (let i = 0; i < n; i++) {
      const from = Math.floor((i / n) * p.length);
      const to = Math.max(from + 1, Math.floor(((i + 1) / n) * p.length));
      let peak = 0;
      for (let j = from; j < to; j++) peak = Math.max(peak, p[j]);
      const bh = Math.max(bar, Math.sqrt(Math.min(1, peak)) * maxH);
      const x = i * (bar + gap);
      const pos = (i + 0.5) / n;
      const inside = !tr || (pos * dur >= tr.start && pos * dur <= tr.end);
      g.globalAlpha = inside ? 1 : 0.25;
      g.fillStyle = pos <= played ? accent : muted;
      g.beginPath();
      g.roundRect(x, (h - bh) / 2, bar, bh, radius);
      g.fill();
    }
    g.globalAlpha = 1;
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
  // Ao pasar o rato, unha liña fina marca a que instante iría o reprodutor se se preme.
  let hoverAt: number | null = null;
  function hover(ev: PointerEvent) {
    hoverAt = mode ? null : timeAt(ev);
  }
  const round = (v: number) => Math.round(v * 10) / 10;
  // Menos de un minuto: «12,3 s»; máis: reloxo («1:05:00»).
  const tag = (v: number) => (v < 60 ? `${round(v)} s` : formatClock(v));
</script>

<div class="tl" class:wave={peaks.length > 0} style={`height:${height}px`} bind:this={box} on:pointerdown={(e) => down(e, "seek")} on:pointermove={(e) => (hover(e), move(e))} on:pointerleave={() => (hoverAt = null)} on:pointerup={up} on:pointercancel={up} role="presentation">
  <canvas bind:this={canvas}></canvas>
  {#if hoverAt !== null && duration}<div class="hover" style={`left:${pct(hoverAt)}%`}><span class="tag">{tag(hoverAt)}</span></div>{/if}
  {#if $trimRange && duration}
    <div class="cut" style={`left:0;width:${pct($trimRange.start)}%`}></div>
    <div class="cut" style={`left:${pct($trimRange.end)}%;right:0`}></div>
    <div class="keep" style={`left:${pct($trimRange.start)}%;width:${pct($trimRange.end) - pct($trimRange.start)}%`}></div>
    <div class="handle" style={`left:${pct($trimRange.start)}%`} on:pointerdown={(e) => down(e, "start")} on:pointermove={move} on:pointerup={up} role="presentation" title={$t("Inicio do corte")}>
      <span class="tag">{tag($trimRange.start)}</span>
    </div>
    <div class="handle" style={`left:${pct($trimRange.end)}%`} on:pointerdown={(e) => down(e, "end")} on:pointermove={move} on:pointerup={up} role="presentation" title={$t("Fin do corte")}>
      <span class="tag">{tag($trimRange.end)}</span>
    </div>
  {/if}
  <div class="head" style={`left:${pct(now)}%`}></div>
</div>

<style>
  .tl { position: relative; width: 100%; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius); cursor: pointer; touch-action: none; user-select: none; overflow: visible; }
  /* A onda de son flota sen caixa: só as barras. */
  .tl.wave { background: transparent; border-color: transparent; }
  canvas { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
  .cut { position: absolute; top: 0; bottom: 0; background: repeating-linear-gradient(45deg, rgba(10, 10, 14, 0.7) 0 6px, rgba(10, 10, 14, 0.55) 6px 12px); pointer-events: none; }
  .keep { position: absolute; top: 0; bottom: 0; border-top: 2px solid var(--accent); border-bottom: 2px solid var(--accent); pointer-events: none; }
  .hover { position: absolute; top: 0; bottom: 0; width: 1px; background: var(--text-muted); opacity: 0.6; pointer-events: none; }
  .hover .tag { left: 50%; transform: translateX(-50%); top: calc(100% + 2px); bottom: auto; }
  .head { position: absolute; top: -3px; bottom: -3px; width: 2px; margin-left: -1px; background: var(--text); box-shadow: 0 0 0 1px var(--bg); pointer-events: none; }
  .handle { position: absolute; top: -4px; bottom: -4px; width: 14px; margin-left: -7px; cursor: ew-resize; display: flex; justify-content: center; }
  .handle::before { content: ""; width: 4px; height: 100%; border-radius: 2px; background: var(--accent); box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.45); }
  .tag { position: absolute; bottom: calc(100% + 2px); font-size: var(--fs-xs); background: rgba(0, 0, 0, 0.75); color: #fff; padding: 1px 5px; border-radius: 4px; white-space: nowrap; pointer-events: none; }
</style>
