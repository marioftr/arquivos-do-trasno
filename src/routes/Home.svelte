<script lang="ts">
  import { goTo } from "../lib/router";

  const modulos = [
    {
      id: "imaxe" as const,
      nome: "Trasno Imaxe",
      accent: "var(--accent-imaxe)",
      icona: "🖼️",
      descricion: "Deseña, retoca e compón con capas.",
    },
    {
      id: "audio" as const,
      nome: "Trasno Audio",
      accent: "var(--accent-audio)",
      icona: "🎚️",
      descricion: "Edita voces e mestura o teu podcast.",
    },
    {
      id: "pdf" as const,
      nome: "Trasno PDF",
      accent: "var(--accent-pdf)",
      icona: "📄",
      descricion: "Revisa, organiza e asina documentos.",
    },
    {
      id: "video" as const,
      nome: "Trasno Vídeo",
      accent: "var(--accent-video)",
      icona: "🎬",
      descricion: "Monta planos, escenas e son.",
    },
  ];
</script>

<div class="home scrollbar-thin">
  <header class="topbar">
    <div class="brand">
      <span class="logo">🔥</span>
      <span class="name">Trasno</span>
    </div>
    <div class="actions">
      <button class="btn">📁 Abrir ficheiro</button>
      <button class="btn">⬇️ Instalar neste equipo</button>
    </div>
  </header>

  <section class="intro">
    <h1>Que queres crear hoxe?</h1>
    <p class="muted">Un só espazo · catro estudos · un conversor universal. Mesma linguaxe, mesmos xestos.</p>
  </section>

  <section class="grid">
    {#each modulos as m}
      <div class="card modulo" style={`--accent:${m.accent}`}>
        <div class="icona">{m.icona}</div>
        <h2 style={`color:${m.accent}`}>{m.nome}</h2>
        <p class="muted">{m.descricion}</p>
        <button class="btn abrir" on:click={() => goTo(m.id)}>› Abrir</button>
      </div>
    {/each}
  </section>

  <section class="card conversor-destacado">
    <div class="conversor-texto">
      <h2 style="color: var(--accent-conversor)">🔁 Conversor universal</h2>
      <p class="muted">
        Converte imaxes, audio, vídeo, documentos e modelos 3D entre formatos —
        PNG→JPEG, WAV→MP3, MOV→MP4, e moito máis — sen abrir ningún dos estudos.
      </p>
    </div>
    <button class="btn primary" on:click={() => goTo("conversor")}>Abrir conversor</button>
  </section>

  <footer class="footer">
    <span class="muted">Imaxe · Audio · PDF · Vídeo</span>
    <span class="muted">Trasno Docs chega máis adiante.</span>
  </footer>

  <section class="recentes">
    <h3 class="muted">RECENTES</h3>
    <div class="card recentes-baleiro muted">Aínda non abriches ningún ficheiro.</div>
  </section>
</div>

<style>
  .home {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 1.25rem 1.5rem 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 800;
    font-size: 1.2rem;
  }

  .logo {
    filter: drop-shadow(0 0 6px rgba(110, 214, 90, 0.4));
  }

  .actions {
    display: flex;
    gap: 0.6rem;
  }

  .intro h1 {
    margin: 0 0 0.35rem;
    font-size: 1.8rem;
  }

  .intro p {
    margin: 0;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 1rem;
  }

  .modulo {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    border-top: 2px solid var(--accent);
  }

  .modulo .icona {
    font-size: 1.6rem;
  }

  .modulo h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  .modulo p {
    margin: 0 0 0.5rem;
    flex: 1;
  }

  .abrir {
    align-self: flex-start;
  }

  .conversor-destacado {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
    border: 1px solid var(--accent-conversor);
    background: linear-gradient(135deg, rgba(79, 179, 245, 0.08), var(--bg-card));
  }

  .conversor-texto h2 {
    margin: 0 0 0.25rem;
  }

  .conversor-texto p {
    margin: 0;
    max-width: 46ch;
  }

  .footer {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .recentes h3 {
    margin: 0 0 0.5rem;
    font-size: 0.75rem;
    letter-spacing: 0.08em;
  }

  .recentes-baleiro {
    min-height: 90px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
