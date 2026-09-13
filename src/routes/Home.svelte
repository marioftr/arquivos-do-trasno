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

  const categorias = [
    { label: "Imaxe", icon: "image", accent: "var(--accent-imaxe)" },
    { label: "Audio", icon: "audio", accent: "var(--accent-audio)" },
    { label: "PDF", icon: "document", accent: "var(--accent-pdf)" },
    { label: "Vídeo", icon: "video", accent: "var(--accent-video)" },
    { label: "3D", icon: "layers", accent: "var(--accent-conversor)" },
  ];
</script>

<div class="home scrollbar-thin">
  <div class="hero">
    <h1>Que queres crear hoxe?</h1>
    <p class="muted">Catro estudos, un conversor universal, a mesma aplicación.</p>

    <div class="hero-actions">
      <button class="btn primary" on:click={abrirFicheiro}>
        <Icon name="folder" size={16} />
        Abrir ficheiro
      </button>
    </div>
  </div>

  <section class="card converter-panel">
    <div class="converter-icon">
      <Icon name="convert" size={22} />
    </div>
    <div class="converter-body">
      <h2>Conversor universal</h2>
      <p class="muted">
        Converte entre formatos de imaxe, audio, vídeo, PDF e modelos 3D — sen abrir ningún estudo.
      </p>
      <ul class="chip-row">
        {#each categorias as c}
          <li class="chip" style={`color:${c.accent}`}>
            <Icon name={c.icon} size={14} />
            {c.label}
          </li>
        {/each}
      </ul>
    </div>
  </section>

  <section class="recentes">
    <h3 class="muted">Recentes</h3>
    <div class="card recentes-baleiro muted">Aínda non abriches ningún ficheiro.</div>
  </section>
</div>

<style>
  .home {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 2.2rem 2rem 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.75rem;
    max-width: 760px;
  }

  .hero h1 {
    margin: 0 0 0.3rem;
    font-size: 1.6rem;
    font-weight: 650;
  }

  .hero p {
    margin: 0;
  }

  .hero-actions {
    margin-top: 1.1rem;
  }

  .converter-panel {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }

  .converter-icon {
    width: 2.6rem;
    height: 2.6rem;
    flex-shrink: 0;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--accent-conversor) 16%, transparent);
    color: var(--accent-conversor);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .converter-body h2 {
    margin: 0 0 0.3rem;
    font-size: 1rem;
  }

  .converter-body p {
    margin: 0;
    max-width: 52ch;
  }

  .chip-row {
    list-style: none;
    margin: 0.9rem 0 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.3rem 0.6rem;
    border-radius: 999px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    font-size: 0.78rem;
  }

  .recentes h3 {
    margin: 0 0 0.6rem;
    font-size: 0.72rem;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    font-weight: 600;
  }

  .recentes-baleiro {
    min-height: 84px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.85rem;
  }
</style>
