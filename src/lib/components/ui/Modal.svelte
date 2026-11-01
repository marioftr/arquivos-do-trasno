<script lang="ts">
  import { t } from "../../i18n";
  import Icon from "../Icon.svelte";

  export let title: string;
  export let onClose: () => void;

  function key(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window on:keydown={key} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="veil" on:click={onClose}>
  <div class="box" role="dialog" tabindex="-1" aria-modal="true" aria-label={title} on:click|stopPropagation>
    <header>
      <h2>{title}</h2>
      <button class="btn ghost icon" on:click={onClose} aria-label={$t("Pechar")} title={$t("Pechar")}><Icon name="close" size={15} /></button>
    </header>
    <div class="content selectable scrollbar-thin"><slot /></div>
  </div>
</div>

<style>
  .veil { position: fixed; inset: 0; z-index: 50; background: rgba(0, 0, 0, 0.55); display: grid; place-items: center; padding: 1.5rem; }
  .box { width: min(860px, 100%); max-height: calc(100vh - 3rem); min-height: 0; display: flex; flex-direction: column; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); box-shadow: 0 18px 50px rgba(0, 0, 0, 0.5); }
  header { display: flex; align-items: center; justify-content: space-between; padding: 0.9rem 1.1rem 0.7rem; border-bottom: 1px solid var(--border); }
  h2 { margin: 0; font-size: 1.02rem; font-weight: 600; }
  .content { padding: 1rem 1.1rem 1.2rem; overflow-y: auto; min-height: 0; flex: 1; display: flex; flex-direction: column; gap: 1rem; }
  :global(.content:has(.split)) { overflow: hidden; height: min(560px, calc(100vh - 8rem)); }
</style>
