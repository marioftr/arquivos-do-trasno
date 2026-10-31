<script lang="ts">
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import type { Form } from "../../forms";

  export let form: Form<any>;
  $: canUndo = form.canUndo;
  $: canRedo = form.canRedo;
  $: dirty = form.dirty;
</script>

<div class="bar">
  <button class="btn ghost icon" on:click={() => form.undo()} disabled={!$canUndo} title={$t("Desfacer (Ctrl+Z)")} aria-label={$t("Desfacer (Ctrl+Z)")}><Icon name="undo" size={15} /></button>
  <button class="btn ghost icon" on:click={() => form.redo()} disabled={!$canRedo} title={$t("Refacer (Ctrl+Y)")} aria-label={$t("Refacer (Ctrl+Y)")}><Icon name="redo" size={15} /></button>
  <span class="grow"></span>
  <button class="btn ghost" on:click={() => form.resetAll()} disabled={!$dirty} title={$t("Volver a todos os valores por defecto")}>
    <Icon name="reset" size={14} /> {$t("Restablecer")}
  </button>
</div>

<style>
  .bar { display: flex; align-items: center; gap: 0.2rem; margin-bottom: 0.2rem; }
  .grow { flex: 1; }
</style>
