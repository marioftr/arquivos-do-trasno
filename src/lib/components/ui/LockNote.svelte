<script lang="ts">
  import InfoTip from "./InfoTip.svelte";

  /** Texto curto visible (p. ex. «Só modo lectura»). */
  export let label: string;
  /** Explicación completa: vai no «i», e pódese seleccionar e copiar. */
  export let text: string;
</script>

<p class="lock-note" role="note"><span>{label}</span><InfoTip {text} /></p>

<style>
  .lock-note { display: inline-flex; align-items: center; align-self: flex-start; margin: 0; padding: 0.25rem 0.35rem 0.25rem 0.65rem; border: 1px solid color-mix(in srgb, var(--accent-pdf) 55%, var(--border-strong)); border-radius: 999px; background: color-mix(in srgb, var(--accent-pdf) 10%, transparent); font-size: var(--fs-sm); line-height: 1.2; color: var(--text); }
</style>
