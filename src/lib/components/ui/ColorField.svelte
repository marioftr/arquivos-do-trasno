<script lang="ts">
  import { tick } from "svelte";
  import InfoTip from "./InfoTip.svelte";
  import Icon from "../Icon.svelte";
  import { HINTS } from "../../hints";
  import { t, tr } from "../../i18n";
  import { toast } from "../../toasts";
  import { pickScreenColor } from "../../eyedropper";
  import { errText } from "../../errors";

  /** Selector de cor: cadrado de cor e matiz (calquera cor), paleta, código hexadecimal e coagotas da pantalla. */
  export let label = "";
  export let value: string;
  $: tip = label && HINTS[label] ? $t(HINTS[label]) : "";

  const PALETTE = [
    "#000000", "#4d4d4d", "#808080", "#bfbfbf", "#ffffff", "#7f1d1d", "#ff3b30", "#ff9500", "#ffcc00",
    "#34c759", "#00a86b", "#00c7be", "#32ade6", "#007aff", "#5856d6", "#af52de", "#ff2d92", "#a2845e",
  ];

  // ---- conversións HSV <-> hexadecimal
  const clamp = (v: number, a = 0, b = 1) => Math.min(b, Math.max(a, v));
  function toHsv(hex: string): [number, number, number] {
    const n = parseInt(hex.slice(1), 16);
    const r = ((n >> 16) & 255) / 255;
    const g = ((n >> 8) & 255) / 255;
    const b = (n & 255) / 255;
    const max = Math.max(r, g, b);
    const d = max - Math.min(r, g, b);
    let h = 0;
    if (d) h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
    return [((h * 60) % 360 + 360) % 360, max ? d / max : 0, max];
  }
  function toHex(h: number, s: number, v: number): string {
    const f = (n: number) => {
      const k = (n + h / 60) % 6;
      return Math.round((v - v * s * Math.max(0, Math.min(k, 4 - k, 1))) * 255).toString(16).padStart(2, "0");
    };
    return `#${f(5)}${f(3)}${f(1)}`;
  }
  const norm = (s: string) => {
    const m = /^#?([0-9a-f]{6}|[0-9a-f]{3})$/i.exec(s.trim());
    if (!m) return null;
    let h = m[1];
    if (h.length === 3) h = [...h].map((c) => c + c).join("");
    return `#${h.toLowerCase()}`;
  };

  let open = false;
  let btn: HTMLButtonElement;
  let pop: HTMLDivElement | undefined;
  let pos = { left: 0, top: 0 };
  let hex = value;
  let [H, S, V] = toHsv(value);
  let dragging: "sv" | "hue" | null = null;
  let svEl: HTMLDivElement | undefined;
  let hueEl: HTMLDivElement | undefined;

  // O valor cambia por fóra (paleta, hexadecimal, coagotas): o cadrado sincronízase.
  $: if (!dragging && norm(value)) {
    hex = value;
    const [h, s, v] = toHsv(value);
    // Se é gris ou negro, o matiz que escolleu o usuario consérvase.
    if (s > 0.01 && v > 0.01) H = h;
    S = s;
    V = v;
  }
  function typed() {
    const n = norm(hex);
    if (n) value = n;
    else hex = value;
  }
  function drag(e: PointerEvent, which: "sv" | "hue") {
    dragging = which;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    update(e);
  }
  function update(e: PointerEvent) {
    if (!dragging) return;
    if (dragging === "sv" && svEl) {
      const r = svEl.getBoundingClientRect();
      S = clamp((e.clientX - r.left) / r.width);
      V = 1 - clamp((e.clientY - r.top) / r.height);
    } else if (dragging === "hue" && hueEl) {
      const r = hueEl.getBoundingClientRect();
      H = clamp((e.clientX - r.left) / r.width) * 360;
    }
    value = toHex(H, S, V);
    hex = value;
  }
  const release = () => (dragging = null);

  async function toggle() {
    if (open) return (open = false);
    open = true;
    await tick();
    place();
  }
  /** Coloca a paleta xunto ao botón: debaixo se cabe e, senón, enriba, sempre dentro da xanela. */
  function place() {
    if (!pop || !btn) return;
    const r = btn.getBoundingClientRect();
    const w = pop.offsetWidth;
    const h = pop.offsetHeight;
    const below = window.innerHeight - r.bottom - 8 >= h;
    const top = below ? r.bottom + 6 : Math.max(8, r.top - h - 6);
    pos = { left: Math.min(Math.max(8, r.left), window.innerWidth - w - 8), top };
  }

  let picking = false;
  async function pick() {
    if (picking) return;
    picking = true;
    const keep = open;
    open = false;
    try {
      const c = await pickScreenColor();
      if (c) value = c;
    } catch (e) {
      toast(tr("O trasno non puido coller a cor da pantalla: {e}", { e: errText(e) }), { kind: "warn" });
    } finally {
      picking = false;
      open = keep;
      if (open) {
        await tick();
        place();
      }
    }
  }
</script>

<svelte:window
  on:pointerdown={(e) => {
    if (open && !pop?.contains(e.target as Node) && !btn?.contains(e.target as Node)) open = false;
  }}
  on:keydown={(e) => e.key === "Escape" && open && !picking && (open = false)}
  on:resize={place}
/>

<div class="field">
  {#if label}<span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>{/if}
  <div class="row">
    <button bind:this={btn} class="sw" type="button" on:click={toggle} aria-label={label || $t("Cor")} aria-expanded={open}>
      <span class="chip" style={`background:${value}`}></span>
    </button>
    <input class="hex" type="text" bind:value={hex} on:change={typed} on:keydown={(e) => e.key === "Enter" && typed()} maxlength="7" spellcheck="false" aria-label={`${label || $t("Cor")} (hex)`} />
    <button class="drop" class:on={picking} type="button" on:click={pick} title={$t("Coller unha cor de calquera sitio da xanela (Esc ou clic dereito para cancelar)")} aria-label={$t("Coller unha cor de calquera sitio da xanela (Esc ou clic dereito para cancelar)")}>
      <Icon name="pipette" size={15} />
    </button>
  </div>
</div>

{#if open}
  <div class="pop" bind:this={pop} style={`left:${pos.left}px;top:${pos.top}px`} role="dialog" aria-label={$t("Escoller unha cor")}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="sv" bind:this={svEl} style={`--hue:${H}`} on:pointerdown={(e) => drag(e, "sv")} on:pointermove={update} on:pointerup={release} on:pointercancel={release}>
      <span class="knob" style={`left:${S * 100}%;top:${(1 - V) * 100}%`}></span>
    </div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="hue" bind:this={hueEl} on:pointerdown={(e) => drag(e, "hue")} on:pointermove={update} on:pointerup={release} on:pointercancel={release}>
      <span class="hknob" style={`left:${(H / 360) * 100}%`}></span>
    </div>
    <div class="grid">
      {#each PALETTE as c}
        <button class="cell" class:cur={value.toLowerCase() === c} style={`background:${c}`} on:click={() => (value = c)} aria-label={c} title={c}></button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .field { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .lab { display: inline-flex; align-items: center; font-size: 0.82rem; color: var(--text-muted); }
  .row { display: flex; align-items: center; gap: 0.4rem; }
  .sw { display: grid; place-items: center; width: var(--control-h); height: var(--control-h); padding: 0; background: var(--bg-input); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); cursor: pointer; }
  .sw:hover { border-color: var(--text-faint); }
  .chip { width: 20px; height: 20px; border-radius: 4px; box-shadow: inset 0 0 0 1px rgba(128, 128, 128, 0.55); }
  .hex { flex: 1; min-width: 0; width: 5.6rem; height: var(--control-h); background: var(--bg-input); color: var(--text-muted); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0 0.6em; font-family: Consolas, monospace; font-size: 0.82rem; text-transform: uppercase; }
  .hex:focus { outline: none; border-color: var(--accent); color: var(--text); }
  .drop { display: grid; place-items: center; width: var(--control-h); height: var(--control-h); padding: 0; background: var(--bg-input); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); color: var(--text-muted); cursor: pointer; }
  .drop:hover, .drop.on { color: var(--accent); border-color: var(--accent); }
  .pop { position: fixed; z-index: 95; width: 240px; display: flex; flex-direction: column; gap: 0.6rem; padding: 0.65rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: 0 10px 28px rgba(0, 0, 0, 0.45); }
  .sv { position: relative; height: 150px; border-radius: 6px; cursor: crosshair; touch-action: none; background: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, hsl(var(--hue), 100%, 50%)); }
  .knob { position: absolute; width: 14px; height: 14px; margin: -7px 0 0 -7px; border-radius: 50%; border: 2px solid #fff; box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.6); pointer-events: none; }
  .hue { position: relative; height: 14px; border-radius: 7px; cursor: pointer; touch-action: none; background: linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00); }
  .hknob { position: absolute; top: -2px; width: 8px; height: 18px; margin-left: -4px; border-radius: 4px; background: #fff; box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.6); pointer-events: none; }
  .grid { display: grid; grid-template-columns: repeat(9, 1fr); gap: 5px; }
  .cell { aspect-ratio: 1; border-radius: 5px; border: 1px solid rgba(128, 128, 128, 0.5); cursor: pointer; padding: 0; }
  .cell:hover { transform: scale(1.12); }
  .cell.cur { outline: 2px solid var(--accent); outline-offset: 1px; }
</style>
