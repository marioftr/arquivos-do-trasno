<script lang="ts">
  /** Carga un compoñente pesado só cando se usa (p. ex. as ferramentas de PDF, que traen PDF.js). */
  export let loader: () => Promise<{ default: any }>;
  let C: any = null;
  loader().then((m) => (C = m.default));
</script>

{#if C}<svelte:component this={C} {...$$restProps} />{/if}
