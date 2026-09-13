<script lang="ts">
  import { goTo } from "../router";

  export let title: string;
  export let accent: string = "var(--accent-conversor)";
</script>

<header class="header">
  <button class="back" on:click={() => goTo("inicio")} title="Volver ao inicio" aria-label="Volver ao inicio">
    ←
  </button>
  <span class="title" style={`color:${accent}`}>{title}</span>
  <div class="spacer"></div>
  <slot name="actions" />
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--bg-elevated);
  }
  .back {
    background: transparent;
    border: 1px solid var(--border-strong);
    color: var(--text);
    border-radius: var(--radius-sm);
    width: 2rem;
    height: 2rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .back:hover {
    border-color: var(--text-faint);
  }
  .title {
    font-weight: 700;
    font-size: 1.05rem;
  }
  .spacer {
    flex: 1;
  }
</style>
