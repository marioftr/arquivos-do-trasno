<script lang="ts">
  import InfoTip from "./InfoTip.svelte";
  import { HINTS } from "../../hints";
  import { t } from "../../i18n";
  export let label = "";
  export let value: string;
  export let placeholder = "";
  export let hint = "";
  $: tip = label && HINTS[label] ? $t(HINTS[label]) : "";
</script>

<div class="field">
  {#if label}<span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>{/if}
  <input type="text" bind:value {placeholder} aria-label={label} />
  {#if hint}<span class="hint">{hint}</span>{/if}
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .lab { display: inline-flex; align-items: center; font-size: 0.82rem; color: var(--text-muted); }
  input { width: 100%; text-align: left; }
  .hint { font-size: 0.76rem; color: var(--text-faint); }
</style>
