<script lang="ts">
  import StudioHeader from "./StudioHeader.svelte";
  import { goTo } from "../router";
  import type { View } from "../router";

  export let title: string;
  export let accent: string;
  export let icona: string;
  export let features: string[];
  export let conversorView: View = "conversor";
</script>

<StudioHeader {title} {accent} />

<div class="body scrollbar-thin">
  <div class="card intro" style={`border-top: 2px solid ${accent}`}>
    <div class="icona">{icona}</div>
    <h2 style={`color:${accent}`}>{title} está en construción</h2>
    <p class="muted">
      Este estudo forma parte da folla de ruta de Trasno. Mentres se desenvolve, xa podes usar o
      <button class="link" on:click={() => goTo(conversorView)}>conversor universal</button>
      para cambiar de formato os teus ficheiros desta categoría.
    </p>
  </div>

  <div class="card">
    <h3 class="muted">Previsto para este estudo</h3>
    <ul>
      {#each features as f}
        <li>{f}</li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 1rem 1.25rem 2rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-width: 640px;
  }
  .intro {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .icona {
    font-size: 2rem;
  }
  .intro h2 {
    margin: 0;
  }
  .intro p {
    margin: 0;
  }
  .link {
    background: none;
    border: none;
    padding: 0;
    color: var(--accent-conversor);
    text-decoration: underline;
    font: inherit;
    cursor: pointer;
  }
  ul {
    margin: 0;
    padding-left: 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
</style>
