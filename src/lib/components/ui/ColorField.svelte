<script lang="ts">
  import { onDestroy } from "svelte";
  import InfoTip from "./InfoTip.svelte";
  import Icon from "../Icon.svelte";
  import { HINTS } from "../../hints";
  import { t } from "../../i18n";
  import { toast } from "../../toasts";
  import { tr } from "../../i18n";

  /** Selector de cor propio: paleta, código hexadecimal e coagotas para tomar unha cor de calquera sitio da pantalla. */
  export let label = "";
  export let value: string;
  $: tip = label && HINTS[label] ? $t(HINTS[label]) : "";

  const PALETTE = [
    "#000000", "#4d4d4d", "#808080", "#bfbfbf", "#ffffff", "#7f1d1d",
    "#ff3b30", "#ff9500", "#ffcc00", "#34c759", "#00a86b", "#00c7be",
    "#32ade6", "#007aff", "#5856d6", "#af52de", "#ff2d92", "#a2845e",
  ];

  let open = false;
  let btn: HTMLButtonElement;
  let pop: HTMLDivElement | undefined;
  let at = { x: 0, y: 0 };
  let hex = value;
  $: if (!picking) hex = value;
  const norm = (s: string) => {
    const m = /^#?([0-9a-f]{6}|[0-9a-f]{3})$/i.exec(s.trim());
    if (!m) return null;
    let h = m[1];
    if (h.length === 3) h = [...h].map((c) => c + c).join("");
    return `#${h.toLowerCase()}`;
  };
  function typed() {
    const n = norm(hex);
    if (n) value = n;
    else hex = value;
  }
  function toggle() {
    if (open) return (open = false);
    const r = btn.getBoundingClientRect();
    const w = 232;
    const below = r.bottom + 250 < window.innerHeight;
    at = { x: Math.min(Math.max(8, r.left), window.innerWidth - w - 8), y: below ? r.bottom + 4 : Math.max(8, r.top - 254) };
    open = true;
  }

  // Coagotas: o selector do sistema (Chromium) colle unha cor de calquera punto da pantalla, tamén de fóra da xanela.
  // Escape cancela; o clic dereito tamén (se o sistema deixa pasar o evento).
  const canPick = typeof window !== "undefined" && "EyeDropper" in window;
  let picking = false;
  let abort: AbortController | null = null;
  const cancelRight = (e: Event) => {
    e.preventDefault();
    abort?.abort();
  };
  async function pick() {
    if (!canPick || picking) return;
    picking = true;
    abort = new AbortController();
    window.addEventListener("contextmenu", cancelRight, true);
    window.addEventListener("mousedown", rightDown, true);
    try {
      // @ts-expect-error EyeDropper non está nos tipos de TypeScript
      const r: { sRGBHex: string } = await new window.EyeDropper().open({ signal: abort.signal });
      value = r.sRGBHex.toLowerCase();
    } catch {
      // cancelado
    } finally {
      picking = false;
      abort = null;
      window.removeEventListener("contextmenu", cancelRight, true);
      window.removeEventListener("mousedown", rightDown, true);
    }
  }
  const rightDown = (e: MouseEvent) => {
    if (e.button === 2) abort?.abort();
  };
  onDestroy(() => abort?.abort());
</script>

<svelte:window
  on:pointerdown={(e) => {
    if (open && !pop?.contains(e.target as Node) && !btn?.contains(e.target as Node)) open = false;
  }}
  on:keydown={(e) => e.key === "Escape" && open && !picking && (open = false)}
/>

<div class="field">
  {#if label}<span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>{/if}
  <div class="row">
    <button bind:this={btn} class="sw" type="button" on:click={toggle} aria-label={label || $t("Cor")} aria-expanded={open}>
      <span class="chip" style={`background:${value}`}></span>
    </button>
    <input class="hex" type="text" bind:value={hex} on:change={typed} on:keydown={(e) => e.key === "Enter" && typed()} maxlength="7" spellcheck="false" aria-label={`${label || $t("Cor")} (hex)`} />
    {#if canPick}
      <button class="drop" class:on={picking} type="button" on:click={pick} title={$t("Coller unha cor da pantalla (Esc ou clic dereito para cancelar)")} aria-label={$t("Coller unha cor da pantalla (Esc ou clic dereito para cancelar)")}>
        <Icon name="pipette" size={15} />
      </button>
    {/if}
  </div>
</div>

{#if open}
  <div class="pop" bind:this={pop} style={`left:${at.x}px;top:${at.y}px`} role="dialog" aria-label={$t("Escoller unha cor")}>
    <div class="grid">
      {#each PALETTE as c}
        <button class="cell" class:cur={value.toLowerCase() === c} style={`background:${c}`} on:click={() => { value = c; open = false; }} aria-label={c} title={c}></button>
      {/each}
    </div>
    {#if !canPick}<p class="muted note">{$t("Escribe o código hexadecimal, por exemplo #ff9500.")}</p>{/if}
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
  .pop { position: fixed; z-index: 95; width: 232px; padding: 0.6rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: 0 10px 28px rgba(0, 0, 0, 0.45); }
  .grid { display: grid; grid-template-columns: repeat(6, 1fr); gap: 6px; }
  .cell { aspect-ratio: 1; border-radius: 5px; border: 1px solid rgba(128, 128, 128, 0.5); cursor: pointer; padding: 0; }
  .cell:hover { transform: scale(1.1); }
  .cell.cur { outline: 2px solid var(--accent); outline-offset: 1px; }
  .note { margin: 0.5rem 0 0; font-size: 0.76rem; }
</style>
