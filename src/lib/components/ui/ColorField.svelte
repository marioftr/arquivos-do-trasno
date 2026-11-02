<script lang="ts">
  import InfoTip from "./InfoTip.svelte";
  import { HINTS } from "../../hints";
  import { t } from "../../i18n";
  export let label = "";
  export let value: string;
  $: tip = label && HINTS[label] ? $t(HINTS[label]) : "";
</script>

<div class="field">
  {#if label}<span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>{/if}
  <label class="color">
    <input type="color" bind:value aria-label={label} />
    <span class="code">{value.toUpperCase()}</span>
  </label>
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .lab { display: inline-flex; align-items: center; font-size: 0.82rem; color: var(--text-muted); }
  .color { display: flex; align-items: center; gap: 0.6rem; height: var(--control-h); background: var(--bg-input); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0 0.5rem; cursor: pointer; }
  .color:hover { border-color: var(--text-faint); }
  input { width: 22px; height: 22px; padding: 0; border: none; background: none; cursor: pointer; }
  .code { font-size: 0.82rem; color: var(--text-muted); font-family: Consolas, monospace; }
</style>
