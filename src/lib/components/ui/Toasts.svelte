<script lang="ts">
  import { toasts, dismiss } from "../../toasts";
  import Icon from "../Icon.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each $toasts as t (t.id)}
    <div class="toast" class:warn={t.kind === "warn"}>
      <Icon name={t.kind === "warn" ? "warning" : "info"} size={15} />
      <span class="tx">{t.text}</span>
      {#if t.action}<button class="btn" on:click={() => { t.action?.run(); dismiss(t.id); }}>{t.action.label}</button>{/if}
      <button class="btn ghost icon" on:click={() => dismiss(t.id)} aria-label="×"><Icon name="close" size={13} /></button>
    </div>
  {/each}
</div>

<style>
  .toasts { position: fixed; right: 1rem; bottom: 1rem; z-index: 60; display: flex; flex-direction: column; gap: 0.4rem; align-items: flex-end; pointer-events: none; }
  .toast { pointer-events: auto; display: flex; align-items: center; gap: 0.6rem; max-width: min(460px, 90vw); padding: 0.5rem 0.6rem 0.5rem 0.8rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-left: 3px solid var(--accent); border-radius: var(--radius-md); box-shadow: 0 8px 26px rgba(0, 0, 0, 0.4); font-size: 0.85rem; }
  .toast.warn { border-left-color: var(--accent-pdf); }
  .tx { flex: 1; min-width: 0; word-break: break-word; }
  .toast :global(.btn) { height: 28px; min-height: 28px; padding-block: 0; }
  .toast :global(.btn.icon) { width: 28px; }
</style>
