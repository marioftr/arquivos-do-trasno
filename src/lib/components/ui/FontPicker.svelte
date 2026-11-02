<script lang="ts">
  import FontSelect from "./FontSelect.svelte";
  import { t } from "../../i18n";

  /** Familia da letra e botóns de negriña e cursiva (os mesmos en PDF e en imaxes). */
  export let label = "";
  export let family: string;
  export let bold: boolean;
  export let italic: boolean;
  export let defFamily: string | undefined = undefined;
</script>

<div class="row">
  <FontSelect {label} bind:value={family} def={defFamily} />
  <button class="btn tog" class:on={bold} aria-pressed={bold} on:click={() => (bold = !bold)} title={`${$t("Negriña")} (Ctrl+B)`}><b>N</b></button>
  <button class="btn tog" class:on={italic} aria-pressed={italic} on:click={() => (italic = !italic)} title={`${$t("Cursiva")} (Ctrl+I)`}><i>C</i></button>
</div>

<style>
  .row { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; gap: 0.4rem; align-items: end; }
  .tog { width: var(--control-h); padding: 0; }
  .tog.on { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 18%, transparent); }
</style>
