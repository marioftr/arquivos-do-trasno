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
  import { actions, tipsOn, liveProgress, bubble, hideBubble, trasnoSay, pickComment, trasnoClick, trasnoAway, viewCtx, noteAction } from "./lib/agent";
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
  // O trasno-axente: avisa do importante (sempre) e, de cando en vez, comenta algo sobre o que hai á vista.
  $: important = ffmpegOk === false && needsFfmpeg ? { text: $t("Falta ffmpeg: instalalo agora"), run: () => requestTool("ffmpeg") } : $updateAvailable ? { text: $t("Hai unha versión nova"), run: () => openHub("news.updates") } : null;
  let tipTimer: ReturnType<typeof setTimeout>;
  function sayNow() {
    const c = pickComment($viewCtx.category, $viewCtx.tool);
    trasnoSay(c.text, c.kind, 11000);
  }
  function scheduleTip(first = false) {
    clearTimeout(tipTimer);
    tipTimer = setTimeout(() => {
      if ($tipsOn && !running.length && !important && $files.length && !$bubble) sayNow();
      scheduleTip();
    }, first ? 18000 : 80000 + Math.random() * 60000);
  }
  $: if (!$tipsOn) hideBubble();
  function clickTrasno() {
    if ($trasnoClick === "palette") dialog.set("palette");
    else if ($trasnoClick === "say") sayNow();
    else openHub("procs.running");
  }
  // Menú radial: aparece ao pasar o rato (ou co teclado) polo trasno.
  let dockOpen = false;
  let dockTimer: ReturnType<typeof setTimeout>;
  const dockShow = () => {
    clearTimeout(dockTimer);
    dockOpen = true;
  };
  const dockHide = () => {
    clearTimeout(dockTimer);
    dockTimer = setTimeout(() => (dockOpen = false), 380);
  };
  const RAD = 92;
  const place = (i: number, n: number) => {
    const th = ((88 - (i * 80) / (n - 1)) * Math.PI) / 180;
    return `left:${32 + RAD * Math.cos(th) - 16}px;bottom:${32 + RAD * Math.sin(th) - 16}px;--dx:${-RAD * Math.cos(th)}px;--dy:${RAD * Math.sin(th)}px`;
  };
  let lastShown = "";
  $: if (active && active.path !== lastShown) {
    if (lastShown && $files.length > 1) noteAction("O trasno mostrou «{name}»", { name: active.name });
    lastShown = active.path;
  }
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
      <button class="bubble warnb" on:click={important.run}><span class="bx">!</span><span class="bt">{important.text}</span></button>
    {:else if $bubble}
      <button class="bubble" class:tipb={$bubble.kind === "tip"} on:click={hideBubble} title={$t("Pechar")}>
        <span class="bx">{$bubble.kind === "tip" ? "i" : "·"}</span>
        <span class="bt">{#if $bubble.kind === "tip"}<b>{$t("Sabías que…")}</b> {/if}{$t($bubble.text)}</span>
      </button>
    {/if}
    <div class="dock" class:open={dockOpen} on:mouseenter={dockShow} on:mouseleave={dockHide} on:focusin={dockShow} on:focusout={dockHide} role="group" aria-label="Arquivos do Trasno">
      <div class="bridge"></div>
      <button class="rd" style={place(0, 5)} tabindex={dockOpen ? 0 : -1} on:click={() => dialog.set("palette")} title={$t("Buscar unha acción (Ctrl+K)")} aria-label={$t("Buscar unha acción (Ctrl+K)")}><Icon name="search" size={16} /></button>
      <button class="rd" style={place(1, 5)} tabindex={dockOpen ? 0 : -1} on:click={() => openHub("procs.running")} title={$t("Procesos")} aria-label={$t("Procesos")}>
        <Icon name="layers" size={16} />{#if runningProcs}<span class="badge">{runningProcs}</span>{/if}
      </button>
      <button class="rd" style={place(2, 5)} tabindex={dockOpen ? 0 : -1} on:click={() => openHub("news.changelog")} title={$t("Novidades")} aria-label={$t("Novidades")}>
        <Icon name="info" size={16} />{#if $updateAvailable}<span class="newdot" aria-hidden="true"></span>{/if}
      </button>
      <button class="rd" style={place(3, 5)} tabindex={dockOpen ? 0 : -1} on:click={() => openHub("settings.general")} title={$t("Axustes")} aria-label={$t("Axustes")}><Icon name="gear" size={16} /></button>
      <button class="rd" style={place(4, 5)} tabindex={dockOpen ? 0 : -1} on:click={sayNow} title={$t("Comentario do trasno")} aria-label={$t("Comentario do trasno")}><Icon name="chat" size={16} /></button>
      <button class="trasno" class:away={$trasnoAway} on:click={clickTrasno} aria-label={$t("Procesos")}>
        <TrasnoSprite state={important ? "warn" : lastRun ? "working" : "idle"} size={64} />
        {#if important}<span class="alert" aria-hidden="true">!</span>{/if}
      </button>
    </div>
    <button class="voice" on:click={() => openHub("procs.running")} title={history ? `${history}\n\n${$t("Procesos")}` : $t("Procesos")}>
      <span class="vt">{voice}</span>
      {#if !lastRun && lastAct}<span class="muted vh">{hhmm(lastAct.at)}</span>{/if}
    </button>
    {#if lastRun}
      <div class="jobs" class:ask={askCancel}>
        {#if askCancel}
          <span class="cq">{$t("Cancelar este proceso?")}</span>
          <button class="cb danger" on:click={doCancel}>{$t("Si, cancelar")}</button>
          <button class="cb" on:click={() => (askCancel = false)}>{$t("Seguir")}</button>
        {:else}
          <button class="jb" on:click={() => openHub("procs.running")} title={`${lastRun.title} — ${$t("Procesos")}`} aria-label={$t("Procesos")}>
            <span class="jt">{lastRun.title}</span>
            <span class="track"><span class="fill" style={`width:${Math.round(Math.max(0.03, pct) * 100)}%`}></span></span>
            <span class="muted pct">{Math.round(pct * 100)} %{running.length > 1 ? ` · +${running.length - 1} ${$t("máis en marcha")}` : ""}</span>
          </button>
          <button class="cb" on:click={() => (askCancel = true)} title={$t("Cancelar")}><Icon name="close" size={13} /> {$t("Cancelar")}</button>
        {/if}
      </div>
    {/if}
    {#if ffmpegOk === false && needsFfmpeg}
      <button class="aviso" on:click={() => requestTool("ffmpeg")}><Icon name="warning" size={14} /> {$t("Falta ffmpeg: instalalo agora")}</button>
    {/if}
    {#if $updateAvailable}
      <button class="aviso up" on:click={() => openHub("news.updates")}><Icon name="download" size={14} /> {$t("Hai unha versión nova")}</button>
    {/if}
    <span class="logo" aria-hidden="true"><BrandMark size={18} /></span>
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
  .badge { position: absolute; top: 2px; right: 1px; background: var(--accent); color: var(--on-accent); border-radius: 99px; font-size: 0.64rem; font-weight: 700; padding: 0 0.35em; min-width: 1.2em; line-height: 1.35; text-align: center; }
  .newdot { position: absolute; top: 7px; right: 7px; width: 8px; height: 8px; border-radius: 50%; background: var(--accent); box-shadow: 0 0 0 2px var(--bg-rail); }
  .status { position: relative; z-index: 20; flex: 0 0 auto; display: flex; align-items: center; gap: 0.7rem; height: 2.7rem; padding: 0 0.9rem 0 5rem; background: var(--bg-rail); border-top: 1px solid var(--border); font-size: 0.8rem; }
  .logo { display: grid; place-items: center; margin-left: auto; opacity: 0.55; }
  .dock { position: absolute; left: 0.4rem; bottom: 0; width: 4rem; height: 4rem; }
  .trasno { position: absolute; inset: 0; display: grid; place-items: center; background: none; border: none; border-radius: var(--radius-md); padding: 0; filter: drop-shadow(0 2px 0 rgba(0, 0, 0, 0.35)); transition: transform 0.38s cubic-bezier(0.3, 1.5, 0.5, 1), opacity 0.2s; }
  .trasno:hover:not(.away) { transform: translateY(-2px); }
  .trasno.away { transform: translateY(115%); opacity: 0; transition: transform 0.2s ease-in, opacity 0.2s; pointer-events: none; }
  .bridge { position: absolute; left: 0; bottom: 0; width: 8.4rem; height: 8.4rem; border-top-right-radius: 100%; pointer-events: none; }
  .dock.open .bridge { pointer-events: auto; }
  .rd { position: absolute; z-index: 2; display: grid; place-items: center; width: 32px; height: 32px; margin: 0; padding: 0; border-radius: 50%; background: var(--bg-elevated); color: var(--text-muted); border: 1px solid var(--border-strong); box-shadow: var(--shadow-pop); opacity: 0; pointer-events: none; transform: translate(var(--dx), var(--dy)) scale(0.2); transition: opacity 0.16s, transform 0.22s cubic-bezier(0.3, 1.4, 0.5, 1), color 0.12s; }
  .dock.open .rd { opacity: 1; pointer-events: auto; transform: none; }
  .rd:hover { color: var(--accent); border-color: var(--accent); }
  .rd:nth-of-type(2) { transition-delay: 0.03s; }
  .rd:nth-of-type(3) { transition-delay: 0.06s; }
  .rd:nth-of-type(4) { transition-delay: 0.09s; }
  .rd:nth-of-type(5) { transition-delay: 0.12s; }
  .badge { position: absolute; top: -5px; right: -5px; background: var(--accent); color: var(--on-accent); border-radius: 99px; font-size: 0.62rem; font-weight: 700; padding: 0 0.3em; min-width: 1.1em; line-height: 1.3; text-align: center; }
  .newdot { position: absolute; top: 1px; right: 1px; width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
  .alert { position: absolute; top: 0.1rem; right: 0; width: 1.15rem; height: 1.15rem; border-radius: 50%; background: var(--accent-pdf); color: #111; font-size: 0.78rem; font-weight: 800; display: grid; place-items: center; box-shadow: 0 0 0 2px var(--bg-rail); animation: hop 0.9s ease-in-out infinite; }
  @keyframes hop { 50% { transform: translateY(-3px); } }
  .bubble { position: absolute; left: 5.1rem; bottom: calc(100% + 0.5rem); display: flex; align-items: flex-start; gap: 0.6rem; max-width: min(30rem, calc(100vw - 7rem)); padding: 0.6rem 0.85rem; text-align: left; color: var(--text); background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); font-size: 0.82rem; line-height: 1.4; animation: rise 0.25s ease-out; }
  .bubble:hover { border-color: var(--accent); }
  .bubble::before { content: ""; position: absolute; left: -6px; bottom: 0.75rem; width: 10px; height: 10px; background: var(--bg-elevated); border-left: 1px solid var(--border-strong); border-bottom: 1px solid var(--border-strong); transform: rotate(45deg); }
  @keyframes rise { from { opacity: 0; transform: translateY(6px); } }
  .bx { flex: none; display: grid; place-items: center; width: 1.2rem; height: 1.2rem; margin-top: 0.05rem; border-radius: 50%; background: var(--accent-pdf); color: #111; font-weight: 800; font-size: 0.78rem; line-height: 1; }
  .tipb .bx { background: var(--accent); color: var(--on-accent); }
  .bubble:not(.tipb) .bx { background: var(--bg-hover); color: var(--text-muted); }
  .warnb .bx { background: var(--accent-pdf) !important; color: #111 !important; }
  .bt { min-width: 0; }
  .voice { flex: 1 1 8rem; min-width: 0; display: flex; align-items: center; gap: 0.55rem; height: 1.9rem; padding: 0 0.5rem; margin-left: -0.5rem; background: none; border: none; border-radius: var(--radius-sm); color: var(--text); text-align: left; font-size: 0.8rem; }
  .voice:hover { background: var(--bg-hover); }
  .vt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
  .vh { flex: none; font-size: 0.72rem; font-variant-numeric: tabular-nums; }
  .jobs { flex: 0 1 30rem; min-width: 0; display: flex; align-items: center; gap: 0.4rem; }
  .jobs.ask { padding: 0 0.3rem 0 0.6rem; height: 1.9rem; border: 1px solid var(--accent-video); border-radius: var(--radius-sm); background: color-mix(in srgb, var(--accent-video) 10%, transparent); }
  .jb { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.6rem; height: 1.9rem; padding: 0 0.6rem; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text); font-size: 0.78rem; animation: grow 0.35s ease-out; transform-origin: left center; }
  @keyframes grow { from { opacity: 0; transform: scaleX(0.2); } }
  .jb:hover { border-color: var(--accent); }
  .jt { flex: 0 1 9rem; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 550; }
  .track { flex: 1 1 4rem; min-width: 2.5rem; height: 5px; border-radius: 5px; background: var(--border-strong); overflow: hidden; }
  .fill { display: block; height: 100%; background: var(--accent); border-radius: 5px; transition: width 0.3s; }
  .pct { white-space: nowrap; font-variant-numeric: tabular-nums; }
  .cq { font-size: 0.78rem; white-space: nowrap; }
  .cb { display: inline-flex; align-items: center; gap: 0.3rem; height: 1.9rem; padding: 0 0.65rem; background: none; border: 1px solid var(--border-strong); border-radius: var(--radius-sm); color: var(--text); font-size: 0.78rem; white-space: nowrap; }
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
  .left { padding-bottom: 1.5rem; box-sizing: border-box; }
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
