<script lang="ts">
  import InfoTip from "./InfoTip.svelte";
  import { HINTS } from "../../hints";
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";

  export let label = "";
  export let value: number | null;
  export let min: number | undefined = undefined;
  export let max: number | undefined = undefined;
  export let step: number | string = 1;
  export let unit = "";
  export let placeholder = "";
  export let def: number | null | undefined = undefined;
  export let hint = "";
  export let disabled = false;
  /** Se se pasa, avísase en cada cambio (útil para campos ligados entre si). */
  export let onInput: ((raw: string) => void) | undefined = undefined;

  $: changed = def !== undefined && value !== def;

  function input(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    if (onInput) onInput(raw);
    else value = raw === "" ? null : Number(raw);
  }
  $: tip = hint || (label && HINTS[label] ? $t(HINTS[label]) : "");
</script>

<div class="field">
  {#if label}
    <div class="head">
      <span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>
      {#if changed}
        <button class="reset" on:click={() => (value = def as number | null)} title={$t("Restaurar o valor por defecto ({v})", { v: def ?? "—" })} aria-label={$t("Restaurar o valor por defecto ({v})", { v: def ?? "—" })}>
          <Icon name="reset" size={13} />
        </button>
      {/if}
    </div>
  {/if}
  <label class="num">
    <input type="number" {min} {max} {step} {placeholder} {disabled} value={value ?? ""} on:input={input} aria-label={label} />
    {#if unit}<span class="unit">{unit}</span>{/if}
  </label>
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .head { display: flex; align-items: center; justify-content: space-between; min-height: 18px; }
  .lab { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
  .reset { display: inline-flex; background: none; border: none; color: var(--text-faint); padding: 2px; border-radius: 4px; }
  .reset:hover { color: var(--accent); background: var(--bg-hover); }
  .num { display: flex; align-items: center; height: var(--control-h); background: var(--bg-input); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding-right: 0.6em; }
  .num:hover { border-color: var(--text-faint); }
  .num:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent); }
  .num input { flex: 1; width: 100%; min-width: 0; height: 100%; border: none; background: transparent; padding: 0 0.4em 0 0.65em; text-align: left; }
  .num input:focus { outline: none; border: none; box-shadow: none; }
  .unit { font-size: var(--fs-sm); color: var(--text-faint); white-space: nowrap; }
</style>
