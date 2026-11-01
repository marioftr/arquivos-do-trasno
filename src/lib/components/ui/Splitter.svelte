<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { t } from "../../i18n";
  import Icon from "../Icon.svelte";

  /** Divisoria entre un panel lateral e o centro: arrastra para cambiar o ancho; o botón pliega ou despliega o panel. */
  export let side: "left" | "right";
  export let width: number;
  export let min: number;
  export let max: number;
  export let collapsed: boolean;
  export let initial: number;

  const dispatch = createEventDispatcher<{ toggle: void }>();
  let drag: { x: number; w: number } | null = null;
  const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v)));

  function down(e: PointerEvent) {
    if (collapsed || e.button !== 0) return;
    drag = { x: e.clientX, w: width };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!drag) return;
    const dx = e.clientX - drag.x;
    width = clamp(drag.w + (side === "left" ? dx : -dx));
  }
  const up = () => (drag = null);
  function key(e: KeyboardEvent) {
    if (e.key === "Enter") return dispatch("toggle");
    if (collapsed) return;
    const step = e.shiftKey ? 48 : 16;
    if (e.key === "ArrowLeft") width = clamp(width + (side === "left" ? -step : step));
    else if (e.key === "ArrowRight") width = clamp(width + (side === "left" ? step : -step));
    else if (e.key === "Home") width = min;
    else if (e.key === "End") width = max;
    else return;
    e.preventDefault();
  }
  // Frecha do botón: apunta cara onde se agocha o panel (ou cara o centro se está pregado).
  $: pointsLeft = side === "left" ? !collapsed : collapsed;
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="split"
  class:drag={!!drag}
  class:collapsed
  class:left={side === "left"}
  class:right={side === "right"}
  role="separator"
  aria-orientation="vertical"
  aria-label={side === "left" ? $t("Ancho do panel de ficheiros") : $t("Ancho do panel de ferramentas")}
  aria-valuenow={collapsed ? 0 : width}
  aria-valuemin={min}
  aria-valuemax={max}
  tabindex="0"
  on:pointerdown={down}
  on:pointermove={move}
  on:pointerup={up}
  on:pointercancel={up}
  on:dblclick={() => (width = initial)}
  on:keydown={key}
>
  <button
    class="fold"
    on:pointerdown|stopPropagation
    on:click|stopPropagation={() => dispatch("toggle")}
    aria-label={collapsed ? $t("Mostrar o panel") : $t("Agochar o panel")}
    title={collapsed ? $t("Mostrar o panel") : $t("Agochar o panel")}
  >
    <span class:flip={pointsLeft}><Icon name="arrow-right" size={12} /></span>
  </button>
</div>

<style>
  .split { position: relative; z-index: 3; cursor: col-resize; touch-action: none; background: var(--border); width: 1px; justify-self: center; outline: none; }
  .split::before { content: ""; position: absolute; inset: 0 -4px; }
  .split:hover, .split.drag, .split:focus-visible { background: var(--accent); width: 2px; }
  .split.collapsed { cursor: default; }
  .fold { position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); display: grid; place-items: center; width: 16px; height: 44px; padding: 0; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: 8px; color: var(--text-muted); opacity: 0; transition: opacity 0.12s ease; cursor: pointer; }
  .fold { pointer-events: none; }
  .split:hover .fold, .split:focus-visible .fold, .split.drag .fold, .split.collapsed .fold, .fold:focus-visible { opacity: 1; pointer-events: auto; }
  /* Pregado, a divisoria queda no bordo da xanela: o botón vai cara dentro para non quedar cortado. */
  .split.collapsed.left .fold { left: 0; transform: translate(0, -50%); border-left: none; border-radius: 0 8px 8px 0; }
  .split.collapsed.right .fold { left: auto; right: 0; transform: translate(0, -50%); border-right: none; border-radius: 8px 0 0 8px; }
  .fold:hover { color: var(--accent); border-color: var(--accent); }
  .flip { display: inline-flex; transform: scaleX(-1); }
  .fold span { display: inline-flex; }
</style>
