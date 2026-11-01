<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { t } from "../../i18n";
  import Icon from "../Icon.svelte";

  /** Barra de busca dos visores (Ctrl+F): texto, número de resultados e anterior/seguinte. */
  export let query = "";
  export let count = 0;
  export let current = 0; // 1..count, 0 se non hai
  export let busy = false;

  const dispatch = createEventDispatcher<{ next: void; prev: void; close: void }>();
  let input: HTMLInputElement;
  onMount(() => {
    input.focus();
    input.select();
  });

  function key(e: KeyboardEvent) {
    if (e.key === "Enter") dispatch(e.shiftKey ? "prev" : "next");
    else if (e.key === "Escape") dispatch("close");
  }
</script>

<div class="find" role="search">
  <input bind:this={input} bind:value={query} on:keydown={key} placeholder={$t("Buscar…")} aria-label={$t("Buscar…")} />
  <span class="muted n">{busy ? $t("Buscando…") : query ? (count ? `${current} / ${count}` : $t("Sen resultados")) : ""}</span>
  <button class="btn ghost icon" on:click={() => dispatch("prev")} disabled={!count} aria-label={$t("Anterior")} title={$t("Anterior")}><Icon name="up" size={14} /></button>
  <button class="btn ghost icon" on:click={() => dispatch("next")} disabled={!count} aria-label={$t("Seguinte")} title={$t("Seguinte")}><Icon name="down" size={14} /></button>
  <button class="btn ghost icon" on:click={() => dispatch("close")} aria-label={$t("Pechar")} title={$t("Pechar")}><Icon name="close" size={14} /></button>
</div>

<style>
  .find { position: absolute; top: 0.6rem; right: 1rem; z-index: 5; display: flex; align-items: center; gap: 0.3rem; padding: 0.3rem 0.4rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.35); }
  input { width: 12rem; height: 28px; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: 5px; padding: 0 0.5em; font: inherit; font-size: 0.85rem; }
  input:focus { outline: none; border-color: var(--accent); }
  .n { font-size: 0.78rem; min-width: 4.5rem; text-align: center; font-variant-numeric: tabular-nums; }
  .find :global(.btn) { height: 28px; min-height: 28px; padding-block: 0; }
  .find :global(.btn.icon) { width: 28px; }
</style>
