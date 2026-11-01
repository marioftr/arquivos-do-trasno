<script lang="ts">
  import { onMount } from "svelte";
  import { openLink, type LinkId } from "../engine";
  import { t } from "../i18n";
  import { tools, refreshTools } from "../settings";
  import Icon from "./Icon.svelte";

  onMount(() => {
    refreshTools();
  });

  const items: { key: "ffmpeg" | "mkvmerge"; name: string; why: string; unlocks: string; link: LinkId; winget: string }[] = [
    {
      key: "ffmpeg",
      name: "FFmpeg",
      why: "Lee e escribe case todos os formatos de audio e vídeo.",
      unlocks: "Convertir, comprimir, editar, cortar e unir audio e vídeo, extraer fotogramas e audio, efectos de son e vistas previas de MKV, AVI ou AIFF.",
      link: "ffmpeg",
      winget: "winget install ffmpeg",
    },
    {
      key: "mkvmerge",
      name: "MKVToolNix",
      why: "Edita os contedores Matroska (MKV) sen recodificar nada, así que non hai perda de calidade.",
      unlocks: "Quitar ou reordenar pistas de audio e subtítulos nun MKV, cambiarlles o nome e o idioma e marcar cal vai por defecto.",
      link: "mkvtoolnix",
      winget: "winget install MoritzBunkus.MKVToolNix",
    },
  ];
</script>

<div class="panel">
  <p class="muted lead">
    {$t("Arquivos do Trasno traballa só coas imaxes, os PDF e o 3D. Para audio, vídeo e MKV axúdase de dous programas gratuítos que non van incluídos: instálaos e a app detectaos soa.")}
  </p>

  {#each items as it}
    {@const ok = $tools?.[it.key]}
    <section class="card" class:ok>
      <div class="head">
        <strong>{it.name}</strong>
        <span class="state" class:ok>
          <Icon name={ok ? "check" : "warning"} size={13} />
          {$tools === null ? $t("Comprobando…") : ok ? $t("Instalado") : $t("Non atopado")}
        </span>
      </div>
      <p class="muted">{$t(it.why)}</p>
      <p><span class="muted">{$t("Desbloquea:")}</span> {$t(it.unlocks)}</p>
      {#if !ok}
        <div class="acts">
          <button class="btn primary" on:click={() => openLink(it.link)}>{$t("Abrir a páxina de descarga")}</button>
          <span class="muted">{$t("ou, en Windows, no terminal:")} <code class="selectable">{it.winget}</code></span>
        </div>
      {/if}
    </section>
  {/each}

  <div class="foot">
    <button class="btn" on:click={refreshTools}><Icon name="reset" size={14} /> {$t("Volver comprobar")}</button>
    <span class="muted">{$t("Tamén se aceptan ao carón do executable da app (ffmpeg.exe, ffprobe.exe, mkvmerge.exe).")}</span>
  </div>
</div>

<style>
  .panel { display: flex; flex-direction: column; gap: 1rem; }
  .lead { margin: 0; }
  .card { border: 1px solid var(--border); border-radius: var(--radius-md); padding: 0.85rem 1rem; display: flex; flex-direction: column; gap: 0.5rem; background: var(--bg-card); }
  .card p { margin: 0; }
  .head { display: flex; align-items: center; justify-content: space-between; }
  .state { display: inline-flex; align-items: center; gap: 0.3rem; font-size: 0.8rem; color: var(--accent-pdf); }
  .state.ok { color: var(--accent-imaxe); }
  .acts { display: flex; flex-wrap: wrap; align-items: center; gap: 0.7rem; }
  code { background: var(--bg-input); border: 1px solid var(--border); border-radius: 4px; padding: 1px 6px; font-size: 0.82rem; }
  .foot { display: flex; flex-wrap: wrap; align-items: center; gap: 0.8rem; font-size: 0.82rem; }
</style>
