<script lang="ts">
  export let label = "";
  export let value: string;
  export let placeholder = "";
  export let hint = "";
</script>

<div class="field">
  {#if label}<span class="lab">{label}</span>{/if}
  <input type="text" bind:value {placeholder} aria-label={label} />
  {#if hint}<span class="hint">{hint}</span>{/if}
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .lab { font-size: 0.82rem; color: var(--text-muted); }
  input { width: 100%; text-align: left; }
  .hint { font-size: 0.76rem; color: var(--text-faint); }
</style>
