<script lang="ts">
  export let tabs: { id: string; label: string }[];
  export let value: string;
</script>

<div class="tabs" role="tablist">
  {#each tabs as tab}
    <button role="tab" aria-selected={value === tab.id} class:on={value === tab.id} on:click={() => (value = tab.id)}>{tab.label}</button>
  {/each}
</div>

<style>
  /* As pestanas non fan scroll: se non caben, pasan á liña seguinte. */
  .tabs { display: flex; flex-wrap: wrap; gap: 0 0.1rem; padding: 0 0.6rem; border-bottom: 1px solid var(--border); flex-shrink: 0; overflow: hidden; }
  button { position: relative; background: none; border: none; color: var(--text-muted); font-size: 0.9rem; font-weight: 500; padding: 0.8rem 0.6rem 0.7rem; white-space: nowrap; }
  button:hover { color: var(--text); }
  button.on { color: var(--text); font-weight: 600; }
  button.on::after { content: ""; position: absolute; left: 0.5rem; right: 0.5rem; bottom: -1px; height: 2px; border-radius: 2px; background: var(--accent); }
</style>
