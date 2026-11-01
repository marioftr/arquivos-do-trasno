<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { t } from "../../i18n";
  import Icon from "../Icon.svelte";

  /** Panel acoplado ao lado dun visor: empurra o contido en vez de taparo. Leva as súas seccións dentro. */
  export let title: string;
  /** Nome co que se lembra o ancho (un por tipo de panel). */
  export let id = "panel";
  export let min = 220;
  export let max = 520;
  export let initial = 272;
  const dispatch = createEventDispatcher<{ close: void }>();

  const KEY = "arquivos-do-trasno.ancho-paneis";
  const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v)));
  let width = (() => {
    try {
      const v = JSON.parse(localStorage.getItem(KEY) ?? "{}")[id];
      return typeof v === "number" ? clamp(v) : initial;
    } catch {
      return initial;
    }
  })();
  function save() {
    try {
      const all = JSON.parse(localStorage.getItem(KEY) ?? "{}");
      all[id] = width;
      localStorage.setItem(KEY, JSON.stringify(all));
    } catch {
      // sen almacenamento: vale mentres a app está aberta
    }
  }

  let drag: { x: number; w: number } | null = null;
  function down(e: PointerEvent) {
    if (e.button !== 0) return;
    drag = { x: e.clientX, w: width };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (drag) width = clamp(drag.w - (e.clientX - drag.x));
  }
  function up() {
    if (drag) save();
    drag = null;
  }
  function key(e: KeyboardEvent) {
    const step = e.shiftKey ? 48 : 16;
    if (e.key === "ArrowLeft") width = clamp(width + step);
    else if (e.key === "ArrowRight") width = clamp(width - step);
    else if (e.key === "Home") width = max;
    else if (e.key === "End") width = min;
    else return;
    e.preventDefault();
    save();
  }
</script>

<aside class="sp" style={`--sp-w:${width}px`} aria-label={title}>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="grip"
    class:drag={!!drag}
    role="separator"
    aria-orientation="vertical"
    aria-label={$t("Ancho do panel")}
    aria-valuenow={width}
    aria-valuemin={min}
    aria-valuemax={max}
    tabindex="0"
    on:pointerdown={down}
    on:pointermove={move}
    on:pointerup={up}
    on:pointercancel={up}
    on:dblclick={() => ((width = initial), save())}
    on:keydown={key}
  ></div>
  <header>
    <h3>{title}</h3>
    <button class="x" on:click={() => dispatch("close")} aria-label={$t("Agochar panel")} title={$t("Agochar panel")}><Icon name="close" size={14} /></button>
  </header>
  <div class="body scrollbar-thin"><slot /></div>
</aside>

<style>
  .sp { position: relative; flex: 0 0 var(--sp-w); width: var(--sp-w); max-width: 60%; min-width: 0; display: flex; flex-direction: column; background: var(--bg-rail); border-left: 1px solid var(--border); font-size: 0.84rem; }
  .grip { position: absolute; left: -3px; top: 0; bottom: 0; width: 7px; cursor: col-resize; touch-action: none; z-index: 2; outline: none; }
  .grip::after { content: ""; position: absolute; left: 3px; top: 0; bottom: 0; width: 1px; background: transparent; }
  .grip:hover::after, .grip.drag::after, .grip:focus-visible::after { width: 2px; left: 2px; background: var(--accent); }
  header { display: flex; align-items: center; justify-content: space-between; padding: 0.55rem 0.7rem 0.55rem 0.9rem; border-bottom: 1px solid var(--border); }
  h3 { margin: 0; font-size: 0.86rem; font-weight: 600; }
  .x { display: grid; place-items: center; width: 26px; height: 26px; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-faint); }
  .x:hover { background: var(--bg-hover); color: var(--text); }
  .body { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; }
</style>
