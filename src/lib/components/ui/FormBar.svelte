<script lang="ts">
  import { readable } from "svelte/store";
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import type { Form } from "../../forms";

  /** Desfacer, refacer e restablecer: sempre os mesmos tres botóns no mesmo sitio, estean ou non activos.
   *  Cos axustes dun formulario usan o seu historial; noutros casos (as anotacións) pásanse as accións. */
  export let form: Form<any> | null = null;
  export let custom: { canUndo: any; canRedo: any; dirty: any; undo: () => void; redo: () => void; reset: () => void; resetTitle?: string } | null = null;
  const off = readable(false);
  $: src = custom ?? (form ? { canUndo: form.canUndo, canRedo: form.canRedo, dirty: form.dirty, undo: () => form!.undo(), redo: () => form!.redo(), reset: () => form!.resetAll() } : null);
  $: canUndo = src?.canUndo ?? off;
  $: canRedo = src?.canRedo ?? off;
  $: dirty = src?.dirty ?? off;
</script>

<div class="bar">
  <button class="btn ghost icon" on:click={() => src?.undo()} disabled={!$canUndo} title={$t("Desfacer (Ctrl+Z)")} aria-label={$t("Desfacer (Ctrl+Z)")}><Icon name="undo" size={15} /></button>
  <button class="btn ghost icon" on:click={() => src?.redo()} disabled={!$canRedo} title={$t("Refacer (Ctrl+Y)")} aria-label={$t("Refacer (Ctrl+Y)")}><Icon name="redo" size={15} /></button>
  <button class="btn ghost icon" on:click={() => src?.reset()} disabled={!$dirty} title={$t("Restablecer valores por defecto")} aria-label={$t("Restablecer")}><Icon name="reset" size={15} /></button>
</div>

<style>
  .bar { display: flex; align-items: center; gap: 0.1rem; }
</style>
