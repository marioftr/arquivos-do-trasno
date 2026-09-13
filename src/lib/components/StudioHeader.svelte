<script lang="ts">
  import Icon from "./Icon.svelte";

  export let title: string;
  export let icon: string;
  export let accent: string = "var(--accent-conversor)";
</script>

<header class="header">
  <span class="icon-badge" style={`color:${accent}; background:color-mix(in srgb, ${accent} 16%, transparent)`}>
    <Icon name={icon} size={17} />
  </span>
  <span class="title">{title}</span>
  <div class="spacer"></div>
  <slot name="actions" />
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.7rem 1.1rem;
    border-bottom: 1px solid var(--border);
    background: var(--bg-elevated);
  }
  .icon-badge {
    width: 1.9rem;
    height: 1.9rem;
    border-radius: var(--radius-sm);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .title {
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--text);
  }
  .spacer {
    flex: 1;
  }
</style>
