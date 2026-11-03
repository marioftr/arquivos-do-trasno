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
  import { actions, tipsOn, liveProgress, bubble, hideBubble, trasnoSay, pickComment, trasnoClick, trasnoAway, viewCtx, noteAction, CLICK_ITEMS, revertTo, revertCount, type ClickAction } from "./lib/agent";
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
  $: running = [...$procs].filter((p) => p.state === "running" && !cancelling.has(p.id)).sort((x, y) => (y.started ?? 0) - (x.started ?? 0));
  $: lastRun = running[0];
  $: pct = lastRun ? (lastRun.source === "app" && $liveProgress !== null ? $liveProgress : lastRun.progress) : 0;
  // O trasno-axente: avisa do importante (sempre) e, de cando en vez, comenta algo sobre o que hai á vista.
  $: important = ffmpegOk === false && needsFfmpeg ? { text: $t("Falta FFmpeg: sen el non podo traballar con audio nin vídeo. Prémeme para instalalo."), run: () => requestTool("ffmpeg") } : $updateAvailable ? { text: $t("Hai unha versión nova. Prémeme para ver as novidades."), run: () => openHub("news.updates") } : null;
  let tipTimer: ReturnType<typeof setTimeout>;
  function sayNow() {
    if (dockOpen) return;
    const c = pickComment($viewCtx.category, $viewCtx.tool);
    trasnoSay(c.text, c.kind, 11000);
  }
  function scheduleTip(first = false) {
    clearTimeout(tipTimer);
    tipTimer = setTimeout(() => {
      if ($tipsOn && !running.length && !important && $files.length && !$bubble && !dockOpen) sayNow();
      scheduleTip();
    }, first ? 18000 : 80000 + Math.random() * 60000);
  }
  $: if (!$tipsOn) hideBubble();

  // ---- menú radial do trasno: sae ao pasar o rato (ou co teclado) e péchase en canto o punteiro deixa as súas ruxas
  // Tres botóns polo interior e dous un pouco máis fóra, arredor do trasno (esquina de abaixo á esquerda).
  const ITEMS = CLICK_ITEMS;
  const BTN = 60;
  const RINGS = [
    { r: 110, a: 76 },
    { r: 110, a: 42 },
    { r: 110, a: 8 },
    { r: 160, a: 59 },
    { r: 160, a: 25 },
  ];
  const pos = RINGS.map((q) => ({ x: q.r * Math.cos((q.a * Math.PI) / 180), y: q.r * Math.sin((q.a * Math.PI) / 180) }));
  function runItem(id: ClickAction) {
    if (id === "palette") dialog.set("palette");
    else if (id === "procs") openHub("procs.running");
    else if (id === "news") openHub("news.changelog");
    else if (id === "settings") openHub("settings.general");
    else sayNow();
  }
  let dockEl: HTMLDivElement;
  let dockOpen = false;
  let hov = -1;
  function dockShow() {
    clearTimeout(closeTimer);
    closeTimer = undefined;
    dockOpen = true;
    hideBubble();
  }
  function dockClose() {
    dockOpen = false;
    hov = -1;
  }
  const segDist = (px: number, py: number, ax: number, ay: number, bx: number, by: number) => {
    const dx = bx - ax;
    const dy = by - ay;
    const l2 = dx * dx + dy * dy;
    const t = l2 ? Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / l2)) : 0;
    return Math.hypot(px - (ax + t * dx), py - (ay + t * dy));
  };
  let closeTimer: ReturnType<typeof setTimeout> | undefined;
  /** O menú segue aberto mentres o punteiro estea sobre o trasno, sobre un botón ou nos raios que os unen, e aínda
   *  aguanta un instante fóra para non pechar por pasar rozando. */
  function dockTrack(e: PointerEvent) {
    if (!dockOpen || !dockEl) return;
    const r = dockEl.getBoundingClientRect();
    const px = e.clientX - (r.left + 32);
    const py = r.top + 32 - e.clientY;
    const near = Math.hypot(px, py) <= 46 || pos.some((q) => Math.hypot(px - q.x, py - q.y) <= BTN / 2 + 22 || segDist(px, py, 0, 0, q.x, q.y) <= 24);
    if (near) {
      clearTimeout(closeTimer);
      closeTimer = undefined;
    } else if (!closeTimer) closeTimer = setTimeout(() => ((closeTimer = undefined), dockClose()), 320);
  }
  function pick(id: ClickAction) {
    dockClose();
    (document.activeElement as HTMLElement | null)?.blur?.();
    runItem(id);
  }
  function clickTrasno() {
    dockClose();
    runItem($trasnoClick);
  }
  $: if (dockOpen) hideBubble();

  let lastShown = "";
  $: if (active && active.path !== lastShown) {
    if (lastShown && $files.length > 1) noteAction("O trasno mostrou «{name}»", { name: active.name });
    lastShown = active.path;
  }
  /** Ao abrir o historial, déixase á vista o máis recente (abaixo). */
  const toEnd = (n: HTMLElement) => {
    const box = n.closest(".hist") as HTMLElement | null;
    if (box) box.scrollTop = box.scrollHeight;
  };
  let histOpen = false;
  let revertIdx: number | null = null;
  $: if (!histOpen) revertIdx = null;
  let askCancel = false;
  $: if (!lastRun) askCancel = false;
  // Ao confirmar, o proceso sae da barra ao instante (se non, quedaba «cargando» ata que a lista se refrescaba).
  let cancelling = new Set<string>();
  $: {
    const gone = [...cancelling].filter((id) => !$procs.some((p) => p.id === id && p.state === "running"));
    if (gone.length) cancelling = new Set([...cancelling].filter((id) => !gone.includes(id)));
  }
  async function doCancel() {
    askCancel = false;
    const r = lastRun;
    if (!r) return;
    cancelling = new Set(cancelling).add(r.id);
    setTimeout(() => {
      cancelling.delete(r.id);
      cancelling = cancelling;
    }, 6000);
    if (r.source === "app") await engine.cancelJob().catch(() => false);
    else await requestCancel(r.id).catch(() => {});
    refreshProcs();
  }
  const hhmm = (n: number) => new Date(n).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  $: lastAct = $actions[0];
  $: voice = lastRun?.subtitle ?? (cancelling.size ? $t("O trasno está parando o traballo…") : null) ?? lastAct?.text ?? ($files.length ? $t("Listo cando queiras.") : $t("Arrastra algún ficheiro e poñémonos a elo."));
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

<svelte:window on:keydown={onKey} on:pointermove={dockTrack} on:blur={dockClose} on:pointerdown={(e) => { if (histOpen && !(e.target as HTMLElement).closest(".hist, .voice")) histOpen = false; }} on:keydown={(e) => e.key === "Escape" && (histOpen = false)} />
<svelte:document on:mouseleave={dockClose} />

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
    {#if !dockOpen}
      {#if important}
        <button class="bubble warnb" on:click={important.run}><span class="bi"><Icon name="warning" size={17} /></span><span class="bt">{important.text}</span></button>
      {:else if $bubble}
        <button class="bubble" class:tipb={$bubble.kind === "tip"} on:click={hideBubble} title={$t("Pechar")}>
          <span class="bi"><Icon name={$bubble.kind === "tip" ? "bulb" : "sparkles"} size={17} /></span>
          <span class="bt">{#if $bubble.kind === "tip"}<b>{$t("Sabías que…")}</b>{" "}{/if}{$t($bubble.text)}</span>
        </button>
      {/if}
    {/if}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="dock" class:open={dockOpen} bind:this={dockEl} on:pointerenter={dockShow} on:focusout={(e) => { if (!dockEl.contains(e.relatedTarget as Node)) dockClose(); }} on:keydown={(e) => e.key === "Escape" && dockClose()} role="group" aria-label="Arquivos do Trasno">
      {#each ITEMS as it, i}
        <button
          class="rd"
          class:hot={hov === i}
          style={`left:${32 + pos[i].x - BTN / 2}px;bottom:${32 + pos[i].y - BTN / 2}px;--dx:${-pos[i].x}px;--dy:${pos[i].y}px;--n:${i};--c:${it.tone}`}
          tabindex={dockOpen ? 0 : -1}
          on:click={() => pick(it.id)}
          on:pointerenter={() => (hov = i)}
          on:pointerleave={() => (hov = -1)}
          on:focus={() => (hov = i)}
          on:blur={() => (hov = -1)}
          aria-label={$t(it.label)}
        >
          <Icon name={it.icon} size={32} strokeWidth={1.6} />
          {#if it.id === "procs" && runningProcs}<span class="badge">{runningProcs}</span>{/if}
          {#if it.id === "news" && $updateAvailable}<span class="newdot" aria-hidden="true"></span>{/if}
          <span class="rl">{$t(it.label)}</span>
        </button>
      {/each}
      <button class="trasno" class:away={$trasnoAway} on:click={clickTrasno} on:focus={(e) => e.currentTarget.matches(":focus-visible") && dockShow()} aria-label={$t("Procesos")}>
        <TrasnoSprite state={important ? "warn" : lastRun ? "working" : "idle"} size={64} />
        {#if important}<span class="alert" aria-hidden="true">!</span>{/if}
      </button>
    </div>
    <button class="voice" class:on={histOpen} on:click={() => (histOpen = !histOpen)} aria-expanded={histOpen} aria-haspopup="dialog" title={$t("Últimas accións")}>
      <span class="vt">{voice}</span>
      {#if !lastRun && lastAct}<span class="muted vh">{hhmm(lastAct.at)}</span>{/if}
    </button>
    {#if histOpen}
      <div class="hist" role="dialog" aria-label={$t("Últimas accións")}>
        <h4>{$t("Últimas accións")}</h4>
        {#if revertIdx !== null && $actions[revertIdx]}
          <div class="conf">
            <p>{$t("Volver ao estado de «{what}»?", { what: $actions[revertIdx].text })}</p>
            <p class="muted">{revertCount($actions, revertIdx) === 1 ? $t("Desfarase 1 cambio.") : $t("Desfaranse {n} cambios.", { n: revertCount($actions, revertIdx) })}</p>
            <div class="cacts">
              <button class="btn danger" on:click={() => { revertTo(revertIdx ?? 0); revertIdx = null; histOpen = false; }}>{$t("Si, volver")}</button>
              <button class="btn ghost" on:click={() => (revertIdx = null)}>{$t("Cancelar")}</button>
            </div>
          </div>
        {:else if $actions.length}
          <ul use:toEnd>
            {#each $actions.map((a, i) => ({ a, i })).reverse() as { a, i }}
              <li>
                <button class="hrow" disabled={!revertCount($actions, i)} on:click={() => (revertIdx = i)} title={revertCount($actions, i) ? $t("Volver a este punto") : ""}>
                  <time class="muted">{hhmm(a.at)}</time>
                  <span class="ht">{a.text}</span>
                  {#if revertCount($actions, i)}<span class="hu"><Icon name="undo" size={14} /></span>{/if}
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="muted">{$t("Aínda non hai accións.")}</p>
        {/if}
      </div>
    {/if}
    {#if lastRun && !$trasnoAway}
      <div class="jobs" class:ask={askCancel}>
        {#if askCancel}
          <span class="q"><Icon name="warning" size={14} /> {$t("Cancelar este proceso?")}</span>
          <span class="acts">
            <button class="btn danger" on:click={doCancel}>{$t("Si, cancelar")}</button>
            <button class="btn ghost" on:click={() => (askCancel = false)}>{$t("Seguir")}</button>
          </span>
        {:else}
          <button class="jmain" on:click={() => openHub("procs.running")} title={`${lastRun.title} — ${$t("Procesos")}`} aria-label={$t("Procesos")}>
            <span class="jt">{lastRun.title}</span>
            <span class="track"><span class="fill" style={`width:${Math.round(Math.max(0.03, pct) * 100)}%`}></span></span>
            <span class="muted pct">{Math.round(pct * 100)} %{running.length > 1 ? ` · +${running.length - 1} ${$t("máis en marcha")}` : ""}</span>
          </button>
          <button class="jx" on:click={() => (askCancel = true)} title={$t("Cancelar")} aria-label={$t("Cancelar")}><Icon name="close" size={14} /></button>
        {/if}
      </div>
    {/if}
    {#if !lastRun}
      <div class="jobs idle">
        <button class="imain" on:click={() => openHub("procs.running")} title={$t("Ver os procesos e o seu historial")}>
          <Icon name="layers" size={16} /><span>{$t("Sen procesos activos")}</span>
        </button>
        <button class="jx" on:click={() => openHub("procs.history")} title={$t("Ver o historial de procesos")} aria-label={$t("Ver o historial de procesos")}><Icon name="clock" size={15} /></button>
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
  .app { position: relative; display: flex; flex-direction: column; width: 100%; height: 100vh; overflow: clip; }
  .frame { flex: 1; min-height: 0; display: flex; }
  .stage { flex: 1; min-width: 0; min-height: 0; display: flex; flex-direction: column; }
  .status { position: relative; z-index: 20; flex: 0 0 auto; display: flex; align-items: center; gap: 0.7rem; height: 44px; padding: 0 0.9rem 0 84px; background: var(--bg-rail); border-top: 1px solid var(--border); font-size: 0.8rem; }
  .logo { display: grid; place-items: center; margin-left: auto; opacity: 0.55; }
  .dock { position: absolute; left: 6px; bottom: 0; width: 64px; height: 64px; }
  .trasno { position: absolute; inset: 0; z-index: 4; display: grid; place-items: center; background: none; border: none; border-radius: var(--radius-md); padding: 0; filter: drop-shadow(0 2px 0 rgba(0, 0, 0, 0.35)); transition: transform 0.38s cubic-bezier(0.3, 1.5, 0.5, 1), opacity 0.2s; }
  .trasno:hover:not(.away) { transform: translateY(-2px); }
  .trasno.away { transform: translateY(115%); opacity: 0; transition: transform 0.2s ease-in, opacity 0.2s; pointer-events: none; }
  .rd { --c: var(--accent); position: absolute; z-index: 1; display: grid; place-items: center; width: 60px; height: 60px; margin: 0; padding: 0; border-radius: 50%; color: var(--c); background: radial-gradient(circle at 32% 24%, color-mix(in srgb, var(--c) 30%, var(--bg-elevated)), var(--bg-elevated) 74%); border: 1px solid color-mix(in srgb, var(--c) 45%, var(--border-strong)); box-shadow: var(--shadow-pop), inset 0 0 0 3px var(--bg-elevated), inset 0 0 0 4px color-mix(in srgb, var(--c) 28%, transparent); opacity: 0; pointer-events: none; transform: translate(var(--dx), var(--dy)) scale(0.3); transition: opacity 0.16s, transform 0.26s cubic-bezier(0.3, 1.25, 0.5, 1), border-color 0.12s, box-shadow 0.12s; transition-delay: calc(var(--n) * 22ms), calc(var(--n) * 22ms), 0s, 0s; }
  .dock.open .rd { opacity: 1; pointer-events: auto; transform: none; }
  .dock.open .rd.hot { z-index: 3; transform: scale(1.1); border-color: var(--c); box-shadow: 0 0 0 4px color-mix(in srgb, var(--c) 24%, transparent), var(--shadow-pop), inset 0 0 0 3px var(--bg-elevated), inset 0 0 0 4px color-mix(in srgb, var(--c) 45%, transparent); transition-delay: 0s; }
  .rl { position: absolute; left: calc(100% + 12px); top: 50%; display: flex; align-items: center; padding: 0.3rem 0.7rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); box-shadow: var(--shadow-pop); color: var(--text); font-size: 0.78rem; font-weight: 550; white-space: nowrap; pointer-events: none; opacity: 0; transform: translate(-6px, -50%); transition: opacity 0.14s, transform 0.18s ease; }
  .rd.hot .rl { opacity: 1; transform: translate(0, -50%); }
  .badge { position: absolute; top: -4px; right: -4px; background: var(--accent); color: var(--on-accent); border-radius: 99px; font-size: 0.62rem; font-weight: 700; padding: 0 0.3em; min-width: 1.1em; line-height: 1.3; text-align: center; }
  .newdot { position: absolute; top: 1px; right: 1px; width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
  .alert { position: absolute; top: 0.1rem; right: 0; width: 1.15rem; height: 1.15rem; border-radius: 50%; background: var(--accent-pdf); color: #111; font-size: 0.78rem; font-weight: 800; display: grid; place-items: center; box-shadow: 0 0 0 2px var(--bg-rail); animation: hop 0.9s ease-in-out infinite; }
  @keyframes hop { 50% { transform: translateY(-3px); } }
  .bubble { --c: var(--accent-conversor); position: absolute; left: 80px; bottom: calc(100% + 0.5rem); display: flex; align-items: flex-start; gap: 0.65rem; max-width: min(30rem, calc(100vw - 7rem)); padding: 0.6rem 0.85rem 0.6rem 0.6rem; text-align: left; color: var(--text); background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); font-size: 0.82rem; line-height: 1.4; animation: rise 0.25s ease-out; }
  .bubble.tipb { --c: var(--accent); }
  .bubble.warnb { --c: var(--accent-pdf); }
  .bubble:hover { border-color: var(--c); }
  .bubble::before { content: ""; position: absolute; left: -6px; bottom: 0.75rem; width: 10px; height: 10px; background: var(--bg-elevated); border-left: 1px solid var(--border-strong); border-bottom: 1px solid var(--border-strong); transform: rotate(45deg); }
  @keyframes rise { from { opacity: 0; transform: translateY(6px); } }
  .bi { flex: none; display: grid; place-items: center; width: 28px; height: 28px; border-radius: 8px; background: color-mix(in srgb, var(--c) 16%, transparent); color: var(--c); }
  .bt { min-width: 0; align-self: center; }
  .voice { flex: 0 1 auto; min-width: 0; display: flex; align-items: center; gap: 0.6rem; height: 28px; padding: 0 0.6rem; background: none; border: 1px solid transparent; border-radius: var(--radius-sm); color: var(--text); text-align: left; font-size: 0.8rem; }
  .voice:hover, .voice.on { background: var(--bg-hover); }
  .hist { position: absolute; left: 84px; bottom: calc(100% + 0.4rem); width: min(30rem, calc(100vw - 8rem)); max-height: 50vh; overflow: auto; padding: 0.6rem 0.2rem 0.4rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); animation: rise 0.18s ease-out; }
  .hist h4 { margin: 0 0.8rem 0.4rem; font-size: 0.72rem; font-weight: 650; letter-spacing: 0.07em; text-transform: uppercase; color: var(--text-muted); }
  .hist ul { margin: 0; padding: 0; list-style: none; }
  .hist li { padding: 0; }
  .hrow { width: 100%; display: grid; grid-template-columns: 3.4rem 1fr auto; align-items: center; gap: 0.6rem; padding: 0.55rem 0.9rem; background: none; border: none; border-radius: 0; color: var(--text); text-align: left; font-size: 0.8rem; }
  .hrow:disabled { cursor: default; }
  .hrow:hover:not(:disabled) { background: var(--bg-hover); }
  .hu { color: var(--text-faint); display: inline-flex; }
  .hrow:hover .hu { color: var(--accent-video); }
  .conf { padding: 0.3rem 0.9rem 0.5rem; display: flex; flex-direction: column; gap: 0.4rem; font-size: 0.82rem; }
  .conf p { margin: 0; }
  .cacts { display: flex; gap: 0.4rem; margin-top: 0.3rem; }
  .hist time { font-variant-numeric: tabular-nums; }
  .ht { min-width: 0; }
  .hist p { margin: 0; padding: 0.3rem 0.8rem; font-size: 0.8rem; }
  .vt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
  .vh { flex: none; font-size: 0.72rem; font-variant-numeric: tabular-nums; }
  .jobs { margin-inline: auto; flex: 0 1 32rem; min-width: 0; display: flex; align-items: stretch; height: 32px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); overflow: hidden; transform-origin: left center; animation: grow 0.4s cubic-bezier(0.2, 0.8, 0.3, 1); }
  @keyframes grow { from { opacity: 0; transform: scaleX(0.15); } }
  .jobs.ask { border-color: color-mix(in srgb, var(--accent-video) 55%, var(--border)); background: color-mix(in srgb, var(--accent-video) 8%, var(--bg-input)); --control-h: 24px; }
  .jmain { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.6rem; padding: 0 0.7rem; background: none; border: none; color: var(--text); font-size: 0.78rem; }
  .jmain:hover { background: var(--bg-hover); }
  .jx { flex: none; display: grid; place-items: center; width: 32px; background: none; border: none; border-left: 1px solid var(--border); color: var(--text-muted); }
  .jx:hover { background: color-mix(in srgb, var(--accent-video) 16%, transparent); color: var(--accent-video); }
  .q { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.5rem; padding: 0 0.7rem; font-size: 0.78rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--text); }
  .acts { flex: none; display: flex; align-items: center; gap: 0.25rem; padding: 0 0.25rem; }
  .acts :global(.btn) { font-size: 0.76rem; padding: 0 0.7em; }
  .jt { flex: 0 1 9rem; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 550; }
  .track { flex: 1 1 4rem; min-width: 2.5rem; height: 5px; border-radius: 5px; background: var(--border-strong); overflow: hidden; }
  .fill { display: block; height: 100%; background: var(--accent); border-radius: 5px; transition: width 0.3s; }
  .pct { white-space: nowrap; font-variant-numeric: tabular-nums; }
  .jobs.idle { flex: 0 1 auto; border-color: transparent; background: none; animation: none; }
  .jobs.idle:hover { border-color: var(--border-strong); background: var(--bg-input); }
  .imain { display: inline-flex; align-items: center; gap: 0.45rem; padding: 0 0.8rem; background: none; border: none; color: var(--text-muted); font-size: 0.78rem; }
  .imain:hover { color: var(--text); }
  .jobs.idle .jx { border-left-color: transparent; }
  .jobs.idle:hover .jx { border-left-color: var(--border); }
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
  .left { padding-bottom: 28px; box-sizing: border-box; }
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
