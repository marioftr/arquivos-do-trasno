<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import Icon from "../lib/components/Icon.svelte";
  import { openInConverter } from "../lib/router";

  async function abrirFicheiro() {
    const selected = await open({ multiple: true });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    openInConverter(paths);
  }
</script>

<div class="home scrollbar-thin">
  <div class="content">
    <header class="topbar">
      <div class="brand">
        <span class="name">Trasno</span>
        <span class="muted tagline">catro estudos, un conversor — muda de forma coma un trasno</span>
      </div>
      <button class="btn primary" on:click={abrirFicheiro}>
        <Icon name="folder" size={16} />
        Abrir ficheiro
      </button>
    </header>

    <section class="card converter-panel">
      <div class="converter-icon">
        <Icon name="convert" size={20} />
      </div>
      <div class="converter-body">
        <h2>Conversor universal</h2>
        <p class="muted">
          Imaxe, audio, vídeo, PDF e modelos 3D entre formatos — arrastra ficheiros directamente ao
          conversor ou ábreos dende aquí.
        </p>
      </div>
    </section>

    <section class="recentes">
      <h3 class="muted">Recentes</h3>
      <div class="card recentes-baleiro muted">Aínda non abriches ningún ficheiro.</div>
    </section>
  </div>
</div>

<style>
  .home {
    flex: 1;
    min-height: 0;
    width: 100%;
    overflow-y: auto;
    padding: 1.75rem 2rem 2rem;
  }

  .content {
    max-width: 900px;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .topbar {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .brand {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .name {
    font-size: 1.3rem;
    font-weight: 650;
  }

  .tagline {
    font-size: 0.85rem;
  }

  .converter-panel {
    display: flex;
    gap: 0.9rem;
    align-items: flex-start;
    border-left: 2px solid var(--accent-conversor);
  }

  .converter-icon {
    color: var(--accent-conversor);
    padding-top: 0.1rem;
  }

  .converter-body h2 {
    margin: 0 0 0.3rem;
    font-size: 0.95rem;
  }

  .converter-body p {
    margin: 0;
    max-width: 56ch;
  }

  .recentes h3 {
    margin: 0 0 0.6rem;
    font-size: 0.72rem;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    font-weight: 600;
  }

  .recentes-baleiro {
    min-height: 76px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.85rem;
  }
</style>
