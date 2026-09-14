<script lang="ts">
  import StudioHeader from "./StudioHeader.svelte";
  import Icon from "./Icon.svelte";
  import { goTo } from "../router";
  import type { View } from "../router";

  export let title: string;
  export let accent: string;
  export let icon: string;
  export let features: string[];
  export let conversorView: View = "conversor";
</script>

<StudioHeader {title} {icon} {accent} />

<div class="body scrollbar-thin">
  <div class="content">
    <div class="card intro">
      <h2>
        <span style={`color:${accent}`}><Icon name={icon} size={17} /></span>
        {title} está en obras
      </h2>
      <p class="muted">
        Este estudo aínda non ten mans (ou gadoupas). Mentres tanto, o
        <button class="link" on:click={() => goTo(conversorView)}>conversor universal</button>
        xa che cambia o formato destes ficheiros.
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
</div>

<style>
  .body {
    flex: 1;
    min-height: 0;
    width: 100%;
    overflow-y: auto;
    padding: 1.1rem 1.25rem 2rem;
  }
  .content {
    max-width: 640px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .intro {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .intro h2 {
    margin: 0;
    font-size: 1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
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
  h3 {
    margin: 0 0 0.6rem;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
  }
  ul {
    margin: 0;
    padding-left: 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
</style>
