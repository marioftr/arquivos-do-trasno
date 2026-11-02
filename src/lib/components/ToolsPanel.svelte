<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import * as engine from "../engine";
  import { openLink, type LinkId } from "../engine";
  import { errText, isCancelled } from "../errors";
  import { t } from "../i18n";
  import { tools, refreshTools } from "../settings";
  import Icon from "./Icon.svelte";
  import { toast } from "../toasts";
  import { tr } from "../i18n";

  let checking = false;
  async function recheck() {
    checking = true;
    const before = $tools;
    try {
      const s = await refreshTools();
      const found = [!before?.ffmpeg && s.ffmpeg ? "FFmpeg" : "", !before?.mkvmerge && s.mkvmerge ? "MKVToolNix" : ""].filter(Boolean);
      if (found.length) toast(tr("O trasno atopou {list}.", { list: found.join(" e ") }));
      else if (s.ffmpeg && s.mkvmerge) toast(tr("O trasno comprobouno: está todo instalado."));
      else toast(tr("O trasno non atopa o que falta. Se o acabas de instalar, pecha e abre a aplicación unha vez."), { kind: "warn", ms: 8000 });
    } catch {
      toast(tr("O trasno non puido comprobalo."), { kind: "warn" });
    } finally {
      checking = false;
    }
  }

  // Descarga e instalación desde a propia app.
  type Key = "ffmpeg" | "mkvmerge";
  let confirming: Key | null = null;
  let installing: Key | null = null;
  let phase: "download" | "verify" | "extract" = "download";
  let progress = 0;
  let failure: { key: Key; text: string } | null = null;
  let unlisten: UnlistenFn | undefined;

  onMount(async () => {
    refreshTools();
    unlisten = await listen<{ tool: string; phase: typeof phase; progress: number }>("trasno://tool-install", (e) => {
      phase = e.payload.phase;
      progress = e.payload.progress;
    });
  });
  onDestroy(() => unlisten?.());

  async function install(key: Key) {
    confirming = null;
    failure = null;
    installing = key;
    phase = "download";
    progress = 0;
    try {
      tools.set(await engine.installTool(key));
      toast(tr("{name} instalado. Xa se pode usar.", { name: items.find((i) => i.key === key)!.name }));
    } catch (e) {
      if (isCancelled(e)) toast(tr("Cancelaches a descarga."));
      else failure = { key, text: errText(e) };
    } finally {
      installing = null;
    }
  }
  async function remove(key: Key) {
    try {
      tools.set(await engine.removeTool(key));
    } catch (e) {
      failure = { key, text: errText(e) };
    }
  }
  const phaseText = (p: typeof phase) => (p === "download" ? tr("Descargando…") : p === "verify" ? tr("Comprobando a descarga…") : tr("Instalando…"));
  /** Avance global: a descarga é case todo o tempo. */
  $: overall = phase === "download" ? progress * 0.8 : phase === "verify" ? 0.8 : 0.8 + progress * 0.2;

  const items: { key: Key; name: string; why: string; unlocks: string; link: LinkId; winget: string; size: string; source: string }[] = [
    {
      key: "ffmpeg",
      name: "FFmpeg",
      why: "Lee e escribe case todos os formatos de audio e vídeo.",
      unlocks: "Convertir, comprimir, editar, cortar e unir audio e vídeo, extraer fotogramas e audio, efectos de son e vistas previas de MKV, AVI ou AIFF.",
      link: "ffmpeg",
      winget: "winget install ffmpeg",
      size: "115 MB",
      source: "gyan.dev",
    },
    {
      key: "mkvmerge",
      name: "MKVToolNix",
      why: "Edita os contedores Matroska (MKV) sen recodificar nada, así que non hai perda de calidade.",
      unlocks: "Quitar ou reordenar pistas de audio e subtítulos nun MKV, cambiarlles o nome e o idioma e marcar cal vai por defecto.",
      link: "mkvtoolnix",
      winget: "winget install MoritzBunkus.MKVToolNix",
      size: "104 MB",
      source: "mkvtoolnix.download",
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
      {#if installing === it.key}
        <div class="prog">
          <div class="line"><span>{phaseText(phase)}</span><span class="muted pct">{Math.round(overall * 100)} %</span></div>
          <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(overall * 100)}><div class="fill" style={`width:${Math.round(overall * 100)}%`}></div></div>
          <div class="acts"><button class="btn" on:click={() => engine.cancelToolInstall()}>{$t("Cancelar descarga")}</button></div>
        </div>
      {:else if confirming === it.key}
        <div class="ask">
          <p>{$t("O trasno vai descargar {name} (uns {size}) de {source}, comprobar que o ficheiro está íntegro e deixalo na súa carpeta de datos. Non fai falta ser administrador.", { name: it.name, size: it.size, source: it.source })}</p>
          <div class="acts">
            <button class="btn primary" on:click={() => install(it.key)}>{$t("Aceptar e descargar")}</button>
            <button class="btn" on:click={() => (confirming = null)}>{$t("Agora non")}</button>
          </div>
        </div>
      {:else if !ok}
        <div class="acts">
          <button class="btn primary" on:click={() => (confirming = it.key)} disabled={installing !== null}>{$t("Descargar e instalar")} <span class="muted">· {it.size}</span></button>
          <button class="btn ghost" on:click={() => openLink(it.link)}>{$t("Abrir a páxina de descarga")}</button>
        </div>
        <p class="muted small">{$t("ou, en Windows, no terminal:")} <code class="selectable">{it.winget}</code></p>
      {:else if $tools?.[`${it.key}_own`]}
        <div class="acts">
          <span class="muted small">{$t("Descargado polo trasno.")}</span>
          <button class="btn ghost" on:click={() => remove(it.key)} disabled={installing !== null}>{$t("Quitar")}</button>
        </div>
      {/if}
      {#if failure?.key === it.key}<p class="fail selectable">{failure.text}</p>{/if}
    </section>
  {/each}

  <div class="foot">
    <button class="btn" on:click={recheck} disabled={checking}><Icon name="reset" size={14} /> {checking ? $t("Comprobando…") : $t("Volver comprobar")}</button>
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
  .small { font-size: 0.8rem; }
  .fail { color: var(--accent-video); font-size: 0.84rem; }
  .prog { display: flex; flex-direction: column; gap: 0.4rem; }
  .line { display: flex; justify-content: space-between; font-size: 0.84rem; }
  .pct { font-variant-numeric: tabular-nums; }
  .bar { height: 7px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); border-radius: 99px; transition: width 0.2s ease; }
  .ask p { margin: 0 0 0.5rem; }
  .acts { display: flex; flex-wrap: wrap; align-items: center; gap: 0.7rem; }
  code { background: var(--bg-input); border: 1px solid var(--border); border-radius: 4px; padding: 1px 6px; font-size: 0.82rem; }
  .foot { display: flex; flex-wrap: wrap; align-items: center; gap: 0.8rem; font-size: 0.82rem; }
</style>
