<script lang="ts">
  import InfoTip from "./InfoTip.svelte";
  import { HINTS } from "../../hints";
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";

  export let label = "";
  export let value: string | number;
  export let options: { value: string | number; label: string }[];
  /** Opcións agrupadas (en vez de `options`). */
  export let groups: { label: string; options: { value: string | number; label: string }[] }[] | null = null;
  export let def: string | number | undefined = undefined;
  export let hint = "";
  export let disabled = false;
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher<{ change: string | number }>();

  $: tip = hint || (label && HINTS[label] ? $t(HINTS[label]) : "");
  $: changed = def !== undefined && value !== def;
</script>

<div class="field">
  {#if label}
    <div class="head">
      <span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>
      {#if changed}
        <button class="reset" on:click={() => ((value = def as string | number), dispatch("change", value))} title={$t("Restaurar o valor por defecto ({v})", { v: String(def) })} aria-label={$t("Restaurar o valor por defecto ({v})", { v: String(def) })}>
          <Icon name="reset" size={13} />
        </button>
      {/if}
    </div>
  {/if}
  <select bind:value {disabled} aria-label={label} on:change={() => dispatch("change", value)}>
    {#if groups}
      {#each groups as g}
        <optgroup label={g.label}>{#each g.options as o}<option value={o.value}>{o.label}</option>{/each}</optgroup>
      {/each}
    {:else}
      {#each options as o}<option value={o.value}>{o.label}</option>{/each}
    {/if}
  </select>
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .head { display: flex; align-items: center; justify-content: space-between; min-height: 18px; }
  .lab { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
  .reset { display: inline-flex; background: none; border: none; color: var(--text-faint); padding: 2px; border-radius: 4px; }
  .reset:hover { color: var(--accent); background: var(--bg-hover); }
  select { width: 100%; }
</style>
