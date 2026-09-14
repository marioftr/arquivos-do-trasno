<script lang="ts">
  import Icon from "./Icon.svelte";

  export let title: string;
  export let icon: string;
  export let accent: string = "var(--accent-conversor)";
</script>

<header class="header" style={`--accent:${accent}`}>
  <span class="icon" style={`color:${accent}`}>
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
    gap: 0.55rem;
    padding: 0.7rem 1.1rem;
    border-bottom: 1px solid var(--border);
    box-shadow: inset 0 -2px 0 color-mix(in srgb, var(--accent) 45%, transparent);
    background: var(--bg-elevated);
  }
  .icon {
    display: inline-flex;
    align-items: center;
  }
  .title {
    font-weight: 600;
    font-size: 0.92rem;
    color: var(--text);
  }
  .spacer {
    flex: 1;
  }
</style>
