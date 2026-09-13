<script lang="ts">
  import { currentView, goTo } from "../router";
  import type { View } from "../router";
  import Icon from "./Icon.svelte";
  import BrandMark from "./BrandMark.svelte";

  const items: { id: View; label: string; icon: string; accent: string }[] = [
    { id: "conversor", label: "Conversor", icon: "convert", accent: "var(--accent-conversor)" },
    { id: "imaxe", label: "Imaxe", icon: "image", accent: "var(--accent-imaxe)" },
    { id: "audio", label: "Audio", icon: "audio", accent: "var(--accent-audio)" },
    { id: "pdf", label: "PDF", icon: "document", accent: "var(--accent-pdf)" },
    { id: "video", label: "Vídeo", icon: "video", accent: "var(--accent-video)" },
  ];
</script>

<nav class="sidebar">
  <button class="brand" on:click={() => goTo("inicio")} title="Inicio" aria-label="Ir ao inicio">
    <BrandMark size={22} />
  </button>

  <ul>
    {#each items as item}
      <li>
        <button
          class="item"
          class:active={$currentView === item.id}
          style={`--item-accent:${item.accent}`}
          on:click={() => goTo(item.id)}
          title={item.label}
          aria-label={item.label}
          aria-current={$currentView === item.id ? "page" : undefined}
        >
          <Icon name={item.icon} size={19} />
        </button>
      </li>
    {/each}
  </ul>
</nav>

<style>
  .sidebar {
    width: var(--sidebar-w);
    flex-shrink: 0;
    background: var(--bg-rail);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.7rem 0;
    gap: 0.6rem;
  }

  .brand {
    background: transparent;
    border: none;
    padding: 0.4rem;
    border-radius: var(--radius-sm);
    display: flex;
    margin-bottom: 0.3rem;
  }
  .brand:hover {
    background: var(--bg-hover);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    width: 100%;
    align-items: center;
  }

  .item {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    position: relative;
  }

  .item:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .item.active {
    background: var(--bg-hover);
    color: var(--item-accent);
  }

  .item.active::before {
    content: "";
    position: absolute;
    left: -0.7rem;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 18px;
    border-radius: 2px;
    background: var(--item-accent);
  }
</style>
