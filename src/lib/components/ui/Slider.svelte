<script lang="ts">
  import InfoTip from "./InfoTip.svelte";
  import { HINTS } from "../../hints";
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";

  export let label = "";
  export let value: number;
  export let min = 0;
  export let max = 100;
  export let step = 1;
  export let unit = "";
  /** Valor por defecto: se cambia, aparece o botón de restaurar. */
  export let def: number | undefined = undefined;
  export let hint = "";
  export let disabled = false;
  /** Sen etiqueta nin botón de restaurar: para barras de ferramentas. */
  export let compact = false;

  $: tip = hint || (label && HINTS[label] ? $t(HINTS[label]) : "");
  $: span = max - min;
  $: pct = span > 0 ? ((Math.min(Math.max(value, min), max) - min) / span) * 100 : 0;
  $: changed = def !== undefined && value !== def;

  function typed(e: Event) {
    const v = Number((e.currentTarget as HTMLInputElement).value);
    if (!Number.isNaN(v)) value = v;
  }
  function commit(e: Event) {
    const el = e.currentTarget as HTMLInputElement;
    const v = Number(el.value);
    value = Number.isNaN(v) || el.value === "" ? (def ?? min) : Math.min(max, Math.max(min, v));
    el.value = String(value);
  }
</script>

<div class="field" class:compact>
  <div class="head" class:hidden={compact}>
    <span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>
    {#if changed}
      <button class="reset" on:click={() => (value = def as number)} title={$t("Restaurar o valor por defecto ({v})", { v: def as number })} aria-label={$t("Restaurar o valor por defecto ({v})", { v: def as number })}>
        <Icon name="reset" size={13} />
      </button>
    {/if}
  </div>
  <div class="ctl">
    <input type="range" style={`--pct:${pct}%`} {min} {max} {step} bind:value {disabled} aria-label={label} on:input />
    <label class="num">
      <input type="number" {min} {max} {step} {value} {disabled} on:input={typed} on:input on:blur={commit} aria-label={label} />
      {#if unit}<span class="unit">{unit}</span>{/if}
    </label>
  </div>
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 0.2rem; }
  .head { display: flex; align-items: center; justify-content: space-between; min-height: 18px; }
  .lab { font-size: 0.82rem; color: var(--text-muted); }
  .reset { display: inline-flex; background: none; border: none; color: var(--text-faint); padding: 2px; border-radius: 4px; }
  .reset:hover { color: var(--accent); background: var(--bg-hover); }
  .hidden { display: none; }
  .compact .ctl { min-width: 230px; }
  .ctl { display: flex; align-items: center; gap: 0.7rem; }
  .ctl input[type="range"] { flex: 1; min-width: 0; }
  .num { display: inline-flex; align-items: center; height: var(--control-h); background: var(--bg-input); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding-right: 0.55em; }
  .num:hover { border-color: var(--text-faint); }
  .num:focus-within { border-color: var(--accent); }
  .num input { width: 4.2em; height: 100%; border: none; background: transparent; padding: 0 0.3em 0 0.6em; }
  .num input:focus { outline: none; border: none; }
  .unit { font-size: 0.78rem; color: var(--text-faint); min-width: 1.6em; }
</style>
