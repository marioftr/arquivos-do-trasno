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
  import { procs, refreshProcs, requestCancel } from "./lib/procs";
  import { actions, tipsOn, TIPS, liveProgress } from "./lib/agent";
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
  $: running = [...$procs].filter((p) => p.state === "running").sort((x, y) => (y.started ?? 0) - (x.started ?? 0));
  $: lastRun = running[0];
  $: pct = lastRun ? (lastRun.source === "app" && $liveProgress !== null ? $liveProgress : lastRun.progress) : 0;
  // O trasno-axente: avisa do importante (sempre) e, de cando en vez, solta un consello (se non está apagado).
  $: important = ffmpegOk === false && needsFfmpeg ? { text: $t("Falta ffmpeg: instalalo agora"), run: () => requestTool("ffmpeg") } : $updateAvailable ? { text: $t("Hai unha versión nova"), run: () => openHub("news.updates") } : null;
  let tip = "";
  let tipIdx = -1;
  let tipTimer: ReturnType<typeof setTimeout>;
  let tipHide: ReturnType<typeof setTimeout>;
  function scheduleTip(first = false) {
    clearTimeout(tipTimer);
    tipTimer = setTimeout(() => {
      if ($tipsOn && !running.length && !important && $files.length) {
        tipIdx = (tipIdx + 1 + Math.floor(Math.random() * (TIPS.length - 1))) % TIPS.length;
        tip = TIPS[tipIdx];
        clearTimeout(tipHide);
        tipHide = setTimeout(() => (tip = ""), 13000);
      }
      scheduleTip();
    }, first ? 18000 : 80000 + Math.random() * 60000);
  }
  $: if (!$tipsOn) tip = "";
  let askCancel = false;
  $: if (!lastRun) askCancel = false;
  async function doCancel() {
    askCancel = false;
    if (!lastRun) return;
    if (lastRun.source === "app") await engine.cancelJob().catch(() => false);
    else await requestCancel(lastRun.id).catch(() => {});
  }
  const hhmm = (n: number) => new Date(n).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  $: lastAct = $actions[0];
  $: voice = lastRun?.subtitle ?? lastAct?.text ?? ($files.length ? $t("Listo cando queiras.") : $t("Arrastra algún ficheiro e poñémonos a elo."));
  $: history = $actions.length ? `${$t("Últimas accións")}:\n` + $actions.map((x) => `${hhmm(x.at)}  ${x.text}`).join("\n") : "";
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
      scheduleTip(true);
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
    {#if important}
      <button class="bubble" on:click={important.run}><span class="bx">!</span> {important.text}</button>
    {:else if tip}
      <button class="bubble tipb" on:click={() => (tip = "")} title={$t("Pechar")}><span class="bx">i</span> <span><b>{$t("Sabías que…")}</b> {$t(tip)}</span></button>
    {/if}
    <button class="trasno" on:click={() => dialog.set("palette")} title={$t("Buscar unha acción (Ctrl+K)")} aria-label={$t("Buscar unha acción (Ctrl+K)")}>
      <TrasnoSprite state={important ? "warn" : lastRun ? "working" : "idle"} size={64} />
      {#if important}<span class="alert" aria-hidden="true">!</span>{/if}
    </button>
    <div class="voice" title={history}>
      <span class="vt">{voice}</span>
      {#if !lastRun && lastAct}<span class="muted vh">{hhmm(lastAct.at)}</span>{/if}
    </div>
    {#if lastRun}
      <div class="jobs">
        <button class="jb" on:click={() => openHub("procs.running")} title={`${lastRun.title} — ${$t("Procesos")}`} aria-label={$t("Procesos")}>
          <span class="jt">{lastRun.title}</span>
          <span class="track"><span class="fill" style={`width:${Math.round(Math.max(0.03, pct) * 100)}%`}></span></span>
          <span class="muted pct">{Math.round(pct * 100)} %{running.length > 1 ? ` · +${running.length - 1} ${$t("máis en marcha")}` : ""}</span>
        </button>
        {#if askCancel}
          <span class="cq">{$t("Cancelar?")}</span>
          <button class="cb danger" on:click={doCancel}>{$t("Si, cancelar")}</button>
          <button class="cb" on:click={() => (askCancel = false)}>{$t("Non")}</button>
        {:else}
          <button class="cb x" on:click={() => (askCancel = true)} title={$t("Cancelar")} aria-label={$t("Cancelar")}><Icon name="close" size={14} /></button>
        {/if}
      </div>
    {/if}
    <span class="grow"></span>
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
  .status { position: relative; z-index: 20; flex: 0 0 auto; display: flex; align-items: center; gap: 0.7rem; height: 2.7rem; padding: 0 0.9rem 0 0.5rem; background: var(--bg-rail); border-top: 1px solid var(--border); font-size: 0.8rem; }
  .trasno { position: relative; flex: none; align-self: flex-end; width: 4rem; height: 4rem; margin-bottom: -0.1rem; display: grid; place-items: center; background: none; border: none; border-radius: var(--radius-md); padding: 0; filter: drop-shadow(0 2px 0 rgba(0, 0, 0, 0.35)); transition: transform 0.12s; }
  .trasno:hover { transform: translateY(-2px); }
  .alert { position: absolute; top: 0.1rem; right: 0; width: 1.15rem; height: 1.15rem; border-radius: 50%; background: var(--accent-pdf); color: #111; font-size: 0.78rem; font-weight: 800; display: grid; place-items: center; box-shadow: 0 0 0 2px var(--bg-rail); animation: hop 0.9s ease-in-out infinite; }
  @keyframes hop { 50% { transform: translateY(-3px); } }
  .bubble { position: absolute; left: 4.9rem; bottom: calc(100% + 0.25rem); display: inline-flex; align-items: center; gap: 0.5rem; max-width: min(34rem, 70%); padding: 0.5rem 0.8rem; text-align: left; color: var(--text); background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); font-size: 0.82rem; line-height: 1.35; animation: rise 0.25s ease-out; }
  .bubble:hover { border-color: var(--accent); }
  .bubble::before { content: ""; position: absolute; left: -6px; bottom: 0.9rem; width: 10px; height: 10px; background: var(--bg-elevated); border-left: 1px solid var(--border-strong); border-bottom: 1px solid var(--border-strong); transform: rotate(45deg); }
  @keyframes rise { from { opacity: 0; transform: translateY(6px); } }
  .bx { flex: none; display: grid; place-items: center; width: 1.2rem; height: 1.2rem; border-radius: 50%; background: var(--accent-pdf); color: #111; font-weight: 800; font-size: 0.78rem; }
  .tipb .bx { background: var(--accent); color: var(--on-accent); }
  .voice { flex: 1 1 8rem; min-width: 0; display: flex; align-items: baseline; gap: 0.55rem; }
  .vt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text); }
  .vh { flex: none; font-size: 0.72rem; font-variant-numeric: tabular-nums; }
  .jobs { flex: 0 1 28rem; min-width: 0; display: flex; align-items: center; gap: 0.35rem; }
  .jb { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.6rem; height: 1.9rem; padding: 0 0.55rem; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text); font-size: 0.78rem; }
  .jb:hover { border-color: var(--accent); }
  .jt { flex: 0 1 9rem; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 550; }
  .track { flex: 1 1 4rem; min-width: 2.5rem; height: 5px; border-radius: 5px; background: var(--border-strong); overflow: hidden; }
  .fill { display: block; height: 100%; background: var(--accent); border-radius: 5px; transition: width 0.3s; }
  .pct { white-space: nowrap; font-variant-numeric: tabular-nums; }
  .cq { font-size: 0.78rem; white-space: nowrap; }
  .cb { height: 1.9rem; padding: 0 0.6rem; background: none; border: 1px solid var(--border-strong); border-radius: var(--radius-sm); color: var(--text); font-size: 0.78rem; }
  .cb.x { width: 1.9rem; padding: 0; display: grid; place-items: center; border-color: transparent; color: var(--text-muted); }
  .cb:hover { background: var(--bg-hover); }
  .cb.danger { color: var(--accent-video); border-color: var(--accent-video); }
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
