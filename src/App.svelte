<script lang="ts">
  import TrasnoSprite from "./lib/components/ui/TrasnoSprite.svelte";
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import BrandMark from "./lib/components/BrandMark.svelte";
  import Icon from "./lib/components/Icon.svelte";
  import FileList from "./lib/components/FileList.svelte";
  import Viewer from "./lib/components/Viewer.svelte";
  import Tools from "./lib/components/Tools.svelte";
  import { files, activePath, checked, addPaths, removeFile, restoreSession } from "./lib/files";
  import Toasts from "./lib/components/ui/Toasts.svelte";
  import { activeForm } from "./lib/forms";
  import { get } from "svelte/store";
  import * as engine from "./lib/engine";
  import { t, lang, LANGS, type Lang } from "./lib/i18n";
  import { theme, dialog, openSettings, openHub, welcomeIfNeeded, type Theme } from "./lib/settings";
  import { updateAvailable, autoCheck, checkForUpdate, applyPendingMenuRefresh } from "./lib/updates";
  import { installExplorerMenu } from "./lib/explorerMenu";
  import QuickJob from "./lib/components/QuickJob.svelte";
  import HubDialog from "./lib/components/HubDialog.svelte";
  import { procs, refreshProcs } from "./lib/procs";
  import { reportProgress } from "./lib/job";
  import { requestTool } from "./lib/toolinstall";
  import ToolNeedDialog from "./lib/components/ToolNeedDialog.svelte";
  import { actionFor } from "./lib/keymap";
  import Splitter from "./lib/components/ui/Splitter.svelte";
  import CommandPalette from "./lib/components/CommandPalette.svelte";

  let dragging = false;
  let unlisten: (() => void) | null = null;
  let unlistenOpen: (() => void) | null = null;
  let unlistenProgress: (() => void) | null = null;
  let procTimer: ReturnType<typeof setInterval>;
  $: runningProcs = $procs.filter((p) => p.state === "running").length;
  let ffmpegOk: boolean | null = null;

  engine.ffmpegAvailable().then((v) => (ffmpegOk = v));
  // Se a app se abriu desde o clic dereito do Explorador, só se mostra a xanela pequena do traballo.
  let quickJob: engine.StartupJob | null | undefined = undefined;
  engine.startupJob().then((j) => (quickJob = j)).catch(() => (quickJob = null));
  const THEMES: { id: Theme; label: string }[] = [
    { id: "system", label: "Sistema" },
    { id: "light", label: "Claro" },
    { id: "dark", label: "Escuro" },
  ];

  async function escoller() {
    const selected = await open({ multiple: true });
    if (!selected) return;
    await addPaths(Array.isArray(selected) ? selected : [selected]);
  }

  // Ancho e estado (pregado ou non) dos dous paneis laterais; lémbranse entre sesións.
  const PANES_KEY = "arquivos-do-trasno.paneis";
  const L = { init: 250, min: 200, max: 420 };
  const R = { init: 450, min: 410, max: 760 };
  const saved = (() => {
    try {
      return JSON.parse(localStorage.getItem(PANES_KEY) ?? "{}");
    } catch {
      return {};
    }
  })();
  const fit = (v: unknown, d: { init: number; min: number; max: number }) => (typeof v === "number" ? Math.min(d.max, Math.max(d.min, v)) : d.init);
  let leftW = fit(saved.leftW, L);
  let rightW = fit(saved.rightW, R);
  let leftOff = !!saved.leftOff;
  let rightOff = !!saved.rightOff;
  $: try {
    localStorage.setItem(PANES_KEY, JSON.stringify({ leftW, rightW, leftOff, rightOff }));
  } catch {
    // sen almacenamento: vale só mentres a app está aberta
  }

  onMount(async () => {
    if ((await engine.startupJob().catch(() => null)) === null) {
      // Apertura normal: a xanela móstrase cando a interface xa está pintada.
      requestAnimationFrame(() => getCurrentWindow().show().catch(() => {}));
      welcomeIfNeeded();
      // Se se acaba de actualizar pedindo refrescar o menú do Explorador, faise agora coa versión nova.
      applyPendingMenuRefresh(installExplorerMenu);
      // Busca de versións novas, un pouco despois de abrir para non estorbar.
      setTimeout(() => {
        if (get(autoCheck)) checkForUpdate(true);
      }, 6000);
    }
    // Primeiro volve a lista da última vez; despois entran os ficheiros co que se abriu a aplicación.
    engine.startupJob().then((j) => {
      if (j !== null) return; // xanela rápida do clic dereito: non toca a lista
      restoreSession().then(() => engine.startupFiles().then((paths) => { if (paths.length) addPaths(paths); }));
    });
    // Outra apertura (clic dereito → «Abrir», «Abrir con») manda aquí os ficheiros en vez de abrir outra app.
    unlistenOpen = await listen<string[]>("trasno://open", (e) => {
      if (e.payload?.length) addPaths(e.payload);
    });
    // Avance real dos traballos desta aplicación e lista de procesos (tamén os das xanelas rápidas).
    if ((await engine.startupJob().catch(() => null)) === null) {
      engine.enableProgress().catch(() => {});
      unlistenProgress = await listen<number>("trasno://progress", (e) => reportProgress(e.payload));
      refreshProcs();
      procTimer = setInterval(refreshProcs, 2000);
    }
    unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === "drop") {
        dragging = false;
        if (p.paths.length) addPaths(p.paths);
      } else if (p.type === "enter") {
        // Arrastrar texto ou elementos da propia interface non trae ficheiros: nese caso non se amosa nada.
        dragging = p.paths.length > 0;
      } else if (p.type === "over") {
        // mantense o que decidiu «enter»
      } else {
        dragging = false;
      }
    });
  });

  function onKey(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    const id = actionFor(e, "general");
    if (id === "g.palette") {
      e.preventDefault();
      dialog.update((d) => (d === "palette" ? null : "palette"));
      return;
    }
    if (!id || t.closest("input, select, textarea")) return;
    if (id === "g.undo") {
      e.preventDefault();
      get(activeForm)?.undo();
    } else if (id === "g.redo") {
      e.preventDefault();
      get(activeForm)?.redo();
    } else if (id === "g.open") {
      e.preventDefault();
      escoller();
    } else if (id === "g.remove" && $activePath) {
      removeFile($activePath);
    }
  }

  onDestroy(() => {
    unlisten?.();
    unlistenOpen?.();
    unlistenProgress?.();
    clearInterval(procTimer);
  });

  $: active = $files.find((f) => f.path === $activePath) ?? null;
  // Con dous ou máis ficheiros marcados, as ferramentas traballan co lote;
  // se non, co ficheiro que se está vendo.
  $: batch = $files.filter((f) => $checked.has(f.path));
  $: selection = batch.length > 1 ? batch : active ? [active] : [];
  $: needsFfmpeg = selection.some((f) => f.category === "audio" || f.category === "video");
</script>

<svelte:window on:keydown={onKey} />

{#if quickJob === undefined}
  <div class="app"></div>
{:else if quickJob}
  <QuickJob job={quickJob} />
{:else}
<div class="app" class:dragging>

  <div class="frame">
  <nav class="rail" aria-label="Arquivos do Trasno">
    <span class="logo"><BrandMark size={26} /></span>
    <button class="rb add" on:click={escoller} title={$t("Escoller ficheiros")} aria-label={$t("Escoller ficheiros")}><Icon name="plus" size={20} /></button>
    <button class="rb" on:click={() => dialog.set("palette")} title={$t("Buscar unha acción (Ctrl+K)")} aria-label={$t("Buscar unha acción (Ctrl+K)")}><Icon name="search" size={20} /></button>
    <button class="rb" on:click={() => openHub("procs.running")} title={$t("Procesos")} aria-label={$t("Procesos")}>
      <Icon name="layers" size={20} />{#if runningProcs}<span class="badge">{runningProcs}</span>{/if}
    </button>
    <button class="rb" on:click={() => openHub("news.changelog")} title={$t("Novidades")} aria-label={$t("Novidades")}><Icon name="info" size={20} />{#if $updateAvailable}<span class="newdot" aria-hidden="true"></span>{/if}</button>
    <span class="grow"></span>
    <button class="rb" on:click={() => openHub("settings.general")} title={$t("Axustes")} aria-label={$t("Axustes")}><Icon name="gear" size={20} /></button>
  </nav>
  <div class="stage">
  {#if $files.length === 0}
    <main class="home">
      <div class="drop">
        <TrasnoSprite state="idle" size={64} />
        <h1>{$t("Arrastra os ficheiros aquí")}</h1>
        <button class="btn primary" on:click={escoller}><Icon name="folder" size={16} /> {$t("Escoller ficheiros")}</button>
        <p class="muted formatos">
          PNG · JPG · WEBP · GIF · MP3 · WAV · FLAC · MP4 · MKV · WEBM · PDF · TXT · FBX · GLB · OBJ · STL · PLY
        </p>
      </div>
    </main>
  {:else}
    <main class="work" style={`--lw:${leftOff ? 0 : leftW}px; --rw:${rightOff ? 0 : rightW}px`}>
      <aside class="left" class:off={leftOff} inert={leftOff}><FileList onAdd={escoller} /></aside>
      <Splitter side="left" bind:width={leftW} min={L.min} max={L.max} initial={L.init} collapsed={leftOff} on:toggle={() => (leftOff = !leftOff)} />
      <section class="center">
        {#if active}<Viewer file={active} />{/if}
      </section>
      <Splitter side="right" bind:width={rightW} min={R.min} max={R.max} initial={R.init} collapsed={rightOff} on:toggle={() => (rightOff = !rightOff)} />
      <aside class="right" class:off={rightOff} inert={rightOff}><Tools {selection} /></aside>
    </main>
  {/if}

  </div>
  </div>

  <footer class="status">
    <TrasnoSprite state={runningProcs ? "working" : "idle"} size={32} />
    <span class="smsg">{runningProcs ? `${$t("En marcha")}: ${runningProcs}` : $files.length ? $t("{n} ficheiros", { n: $files.length }) : $t("Listo")}</span>
    <span class="grow"></span>
    {#if active}<span class="muted sfile" title={active.path}>{active.name}</span>{/if}
    {#if ffmpegOk === false && needsFfmpeg}
      <button class="aviso" on:click={() => requestTool("ffmpeg")}><Icon name="warning" size={14} /> {$t("Falta ffmpeg: instalalo agora")}</button>
    {/if}
    {#if $updateAvailable}
      <button class="aviso up" on:click={() => openHub("news.updates")}><Icon name="download" size={14} /> {$t("Hai unha versión nova")}</button>
    {/if}
  </footer>

  <Toasts />

  {#if $dialog === "hub"}<HubDialog />{:else if $dialog === "palette"}<CommandPalette onAdd={escoller} />{/if}
  <ToolNeedDialog />

  {#if dragging}
    <div class="veil"><span>{$t("Solta os ficheiros para que o trasno os colla")}</span></div>
  {/if}
</div>
{/if}

<style>
  .app { position: relative; display: flex; flex-direction: column; width: 100%; height: 100vh; }
  .frame { flex: 1; min-height: 0; display: flex; }
  .stage { flex: 1; min-width: 0; min-height: 0; display: flex; flex-direction: column; }
  .rail { flex: 0 0 3.5rem; width: 3.5rem; display: flex; flex-direction: column; align-items: center; gap: 0.3rem; padding: 0.7rem 0 0.6rem; background: var(--bg-rail); border-right: 1px solid var(--border); }
  .logo { display: grid; place-items: center; margin-bottom: 0.6rem; }
  .rb { position: relative; display: grid; place-items: center; width: 2.5rem; height: 2.5rem; background: none; border: none; border-radius: 0.7rem; color: var(--text-muted); transition: background 0.12s ease, color 0.12s ease; }
  .rb:hover { background: var(--bg-hover); color: var(--text); }
  .rb.add { background: var(--accent); color: var(--on-accent); margin-bottom: 0.5rem; }
  .rb.add:hover { filter: brightness(1.1); background: var(--accent); color: var(--on-accent); }
  .grow { flex: 1; }
  .badge { position: absolute; top: 2px; right: 1px; background: var(--accent); color: var(--on-accent); border-radius: 99px; font-size: 0.64rem; font-weight: 700; padding: 0 0.35em; min-width: 1.2em; line-height: 1.35; text-align: center; }
  .newdot { position: absolute; top: 7px; right: 7px; width: 8px; height: 8px; border-radius: 50%; background: var(--accent); box-shadow: 0 0 0 2px var(--bg-rail); }
  .status { flex: 0 0 auto; display: flex; align-items: center; gap: 0.7rem; min-height: 2.5rem; padding: 0 0.9rem 0 0.7rem; background: var(--bg-rail); border-top: 1px solid var(--border); font-size: 0.78rem; }
  .smsg { color: var(--text); font-weight: 550; }
  .sfile { max-width: 38%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .aviso { background: none; border: none; display: inline-flex; align-items: center; gap: 0.4rem; color: var(--accent-pdf); font-size: 0.78rem; font-weight: 550; padding: 0.2rem 0.4rem; border-radius: var(--radius-sm); }
  .aviso:hover { background: var(--bg-hover); }
  .aviso.up { color: var(--accent); }

  .home { flex: 1; min-height: 0; display: flex; padding: 1.2rem; }
  .drop {
    flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 1rem;
    text-align: center; padding: 2rem; border: 1px dashed var(--border-strong); border-radius: var(--radius-lg);
    background: radial-gradient(ellipse at 50% 38%, color-mix(in srgb, var(--accent) 7%, transparent), transparent 62%);
  }
  .drop h1 { margin: 0.4rem 0 0; font-size: 1.2rem; font-weight: 600; letter-spacing: -0.01em; }
  .drop p { margin: 0; max-width: 54ch; }
  .formatos { font-size: 0.75rem; letter-spacing: 0.04em; margin-top: 0.8rem !important; }

  .work { flex: 1; min-height: 0; display: grid; grid-template-rows: minmax(0, 1fr); grid-template-columns: var(--lw) 1px minmax(0, 1fr) 1px var(--rw); }
  .off { overflow: hidden; visibility: hidden; }
  .left, .right { min-height: 0; min-width: 0; background: var(--bg-rail); }
  .center { background: var(--bg); }

  .center { min-width: 0; min-height: 0; overflow: hidden; }

  /* Co espazo xusto, os paneis ocupan o seu sitio de sempre e as divisorias agóchanse. */
  @media (max-width: 1050px) {
    .work { grid-template-columns: minmax(190px, 230px) minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) minmax(0, 42%); }
    .work > :global(.split) { display: none; }
    .left.off, .right.off { visibility: visible; }
    .right { grid-column: 1 / -1; border-top: 1px solid var(--border); }
  }

  .veil {
    position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
    background: color-mix(in srgb, var(--bg) 82%, transparent); border: 2px dashed var(--accent-conversor);
    color: var(--text); font-size: 1.05rem; pointer-events: none;
  }
</style>
