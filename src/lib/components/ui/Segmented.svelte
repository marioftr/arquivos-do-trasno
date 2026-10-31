<script lang="ts">
  export let options: { value: string | number; label: string; hint?: string }[];
  export let value: string | number;
  export let label = "";
</script>

<div class="wrap">
  {#if label}<span class="lab">{label}</span>{/if}
  <div class="seg" role="group" aria-label={label}>
    {#each options as o}
      <button class:on={value === o.value} title={o.hint} on:click={() => (value = o.value)}>{o.label}</button>
    {/each}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { font-size: 0.82rem; color: var(--text-muted); }
  .seg { display: flex; border: 1px solid var(--border-strong); border-radius: var(--radius-sm); overflow: hidden; height: var(--control-h); background: var(--bg-input); }
  button { flex: 1; background: transparent; border: none; border-right: 1px solid var(--border-strong); color: var(--text-muted); font-size: 0.85rem; padding: 0 0.5em; white-space: nowrap; }
  button:last-child { border-right: none; }
  button:hover { color: var(--text); background: var(--bg-hover); }
  button.on { background: color-mix(in srgb, var(--accent) 20%, transparent); color: var(--text); font-weight: 600; }
</style>
