<script lang="ts">
  export let tabs: { id: string; label: string }[];
  export let value: string;

  let bar: HTMLDivElement;
  function key(e: KeyboardEvent) {
    const i = tabs.findIndex((x) => x.id === value);
    const to = e.key === "ArrowRight" ? i + 1 : e.key === "ArrowLeft" ? i - 1 : e.key === "Home" ? 0 : e.key === "End" ? tabs.length - 1 : -1;
    if (to < 0) return;
    e.preventDefault();
    const j = (to + tabs.length) % tabs.length;
    value = tabs[j].id;
    requestAnimationFrame(() => bar?.querySelectorAll<HTMLElement>('[role="tab"]')[j]?.focus());
  }
</script>

<!-- Frechas para moverse entre pestanas; só a activa entra co Tab (como nos controis nativos). -->
<!-- svelte-ignore a11y_interactive_supports_focus -->
<div class="tabs" role="tablist" on:keydown={key} bind:this={bar}>
  {#each tabs as tab}
    <button role="tab" tabindex={value === tab.id ? 0 : -1} aria-selected={value === tab.id} class:on={value === tab.id} on:click={() => (value = tab.id)}>{tab.label}</button>
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
