<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import * as engine from "../engine";
  import { errText, isCancelled, missingToolOf } from "../errors";
  import { TOOL_INFO, installing, installError, installToolNow, overall, phaseText } from "../toolinstall";
  import type { ToolId } from "../engine";
  import { fallo, fileNameOf, listo, TRASNO } from "../formats";
  import { t, tr } from "../i18n";
  import { runOne } from "../quickjob";
  import { track, cancelRequested, type Tracker } from "../procs";
  import BrandMark from "./BrandMark.svelte";
  import Icon from "./Icon.svelte";

  /** Traballo lanzado desde o clic dereito do Explorador: xanela pequena que fai a faena e se pecha soa. */
  export let job: engine.StartupJob;

  interface Item {
    path: string;
    state: "agarda" | "traballa" | "feito" | "erro";
    out?: string;
    note?: string;
    error?: string;
  }
  let items: Item[] = job.paths.map((path) => ({ path, state: "agarda" }));
  let headline = tr(job.action === "convert" ? TRASNO.convertendo : job.action === "compress" ? TRASNO.comprimindo : TRASNO.traballando);
  let finished = false;
  let failedAny = false;
  let keepOpen = false;
  let countdown = 8;
  let asking = false; // pedindo confirmación para cancelar
  let cancelMode: "partial" | "all" | null = null;
  let cancelPct = 0; // onde estaba a barra ao cancelar
  let running = true;
  const win = getCurrentWindow();

  // Se un ficheiro precisa FFmpeg e non está, o traballo agarda e pregúntase se se descarga.
  let missing: ToolId | null = null;
  let answer: ((install: boolean) => void) | null = null;
  const askInstall = (tool: ToolId) =>
    new Promise<boolean>((resolve) => {
      missing = tool;
      installError.set(null);
      win.setSize(new LogicalSize(520, Math.min(640, 400 + items.length * 34))).catch(() => {});
      answer = (v) => {
        missing = null;
        answer = null;
        win.setSize(new LogicalSize(520, Math.min(580, 230 + items.length * 34))).catch(() => {});
        resolve(v);
      };
    });
  async function acceptInstall() {
    if (!missing) return;
    const ok = await installToolNow(missing);
    if (ok) answer?.(true);
  }
  let tracker: Tracker | null = null;
  let poll: ReturnType<typeof setInterval> | null = null;

  // Que se lle vai facer aos ficheiros, en claro.
  $: what = (() => {
    const a = job.target ?? "";
    switch (job.action) {
      case "convert":
        return tr("Converter a {ext}", { ext: a.toUpperCase() });
      case "compress":
        return tr("Comprimir") + " · " + tr("Calidade {p} %", { p: a });
      case "resize":
        return tr("Cambiar tamaño") + " · " + (a.startsWith("p") ? tr("{p} %", { p: a.slice(1) }) : `${a.slice(1)}p`);
      default:
        return tr("Xirar") + " · " + (a === "270" ? tr("Á esquerda") : a === "180" ? tr("Dar a volta (180°)") : tr("Á dereita"));
    }
  })();

  $: total = items.length;
  $: doneCount = items.filter((i) => i.state === "feito" || i.state === "erro").length;
  $: working = items.some((i) => i.state === "traballa");
  // Avance real do ffmpeg para o ficheiro en marcha (0 a 1). As imaxes e os PDF non o dan: a barra móvese sen número.
  let frac = 0;
  let realProgress = false;
  $: indeterminate = working && !realProgress && !cancelMode;
  $: pct = cancelMode ? cancelPct : finished ? 100 : Math.min(99, Math.round(((doneCount + (realProgress ? frac : 0)) / total) * 100));
  $: itemPct = Math.round(frac * 100);
  $: tracker?.update({ progress: pct / 100, outputs: items.filter((i) => i.out).map((i) => i.out!) });

  const CANCEL_PHRASES = [
    "Vale, o trasno déixao estar e recolle as ferramentas.",
    "Proceso cancelado: o trasno gárdase o gorro e aquí non pasou nada.",
    "Cancelado! O trasno borra as pegadas e non di nada.",
  ];

  /** Cancelar: para o ffmpeg e o resto da lista. `partial` borra só o que quedou a medias; `all` tamén o que xa estaba rematado. */
  async function doCancel(mode: "partial" | "all") {
    if (cancelMode) return;
    asking = false;
    cancelPct = pct;
    cancelMode = mode;
    headline = tr("Cancelando…");
    await engine.cancelJob().catch(() => false);
  }
  async function discardAll() {
    for (const it of items) {
      if (it.out) await engine.discardOutput(it.out).catch(() => false);
      it.out = undefined;
      it.note = undefined;
    }
  }

  onMount(async () => {
    // Pechar coa X mentres traballa equivale a querer cancelar: antes pídese confirmación.
    await win.onCloseRequested((e) => {
      if (running && !cancelMode) {
        e.preventDefault();
        asking = true;
      }
    });
    try {
      await win.setTitle("Arquivos do Trasno");
      await win.setResizable(false);
      await win.setMaximizable(false);
      await win.setSize(new LogicalSize(520, Math.min(580, 230 + items.length * 34)));
      await win.center();
    } catch {
      // sen permiso para axustar a xanela: segue igual
    }
    // A xanela naceu agochada: móstrase xa co tamaño bo e co contido pintado.
    await win.show();

    tracker = track({ title: items.length === 1 ? fileNameOf(items[0].path) : tr("{n} ficheiros", { n: items.length }), subtitle: what, source: "quick" });
    // A aplicación principal pode pedir cancelar este traballo desde a lista de procesos.
    poll = setInterval(async () => {
      if (!cancelMode && tracker && (await cancelRequested(tracker.id).catch(() => false))) doCancel("partial");
    }, 700);

    let unlisten: (() => void) | null = null;
    try {
      await engine.enableProgress();
      unlisten = await listen<number>("trasno://progress", (e) => {
        realProgress = true;
        frac = Math.max(0, Math.min(1, e.payload));
      });
    } catch {
      // sen avance real: a barra móvese sen número
    }
    for (const it of items) {
      if (cancelMode) break;
      await engine.beginJob().catch(() => {});
      frac = 0;
      realProgress = false;
      it.state = "traballa";
      items = items;
      try {
        let r;
        for (;;) {
          try {
            r = await runOne(job, it.path);
            break;
          } catch (e) {
            const tool = missingToolOf(e);
            if (!tool || cancelMode || !(await askInstall(tool))) throw e;
          }
        }
        it.out = r.out;
        it.note = r.note;
        it.state = "feito";
      } catch (e) {
        if (cancelMode || isCancelled(e)) {
          if (!cancelMode) await doCancel("partial");
          it.state = "agarda";
        } else {
          it.state = "erro";
          it.error = errText(e);
          failedAny = true;
        }
      }
      items = items;
    }
    unlisten?.();
    if (poll) clearInterval(poll);
    running = false;
    if (cancelMode) {
      if (cancelMode === "all") await discardAll();
      items = items;
      finished = true;
      headline = tr(CANCEL_PHRASES[Math.floor(Math.random() * CANCEL_PHRASES.length)]);
      tracker.finish("cancelled", { outputs: items.filter((i) => i.out).map((i) => i.out!) });
      countdown = 6;
    } else {
      finished = true;
      headline = failedAny ? fallo(job.action === "convert" ? "convertendo" : job.action === "compress" ? "comprimindo" : "editando") : listo();
      tracker.finish(failedAny ? "error" : "done", { detail: items.find((i) => i.error)?.error });
    }
    // Contador de peche: tamén cando se cancela. Un clic en calquera sitio ou un erro deixan a xanela aberta.
    if (!failedAny) {
      const timer = setInterval(() => {
        if (keepOpen) return clearInterval(timer);
        countdown -= 1;
        if (countdown <= 0) {
          clearInterval(timer);
          win.destroy();
        }
      }, 1000);
    }
  });

  onDestroy(() => {
    if (poll) clearInterval(poll);
  });

  const outs = () => items.filter((i) => i.out).map((i) => i.out!);
  async function openFull() {
    try {
      await engine.openInApp(outs());
      win.destroy();
    } catch {
      // se non se puido abrir, a xanela queda para amosar no cartafol
    }
  }
</script>

<svelte:window
  on:pointerdown={() => (keepOpen = true)}
  on:keydown={(e) => e.key === "Escape" && (running && !cancelMode ? (asking = true) : win.destroy())}
/>

<main>
  <header>
    <BrandMark size={26} />
    <div class="titles">
      <strong>Arquivos do Trasno</strong>
      <span class="muted what">{what}</span>
    </div>
  </header>

  <p class="head" class:bad={finished && failedAny} class:cancel={!!cancelMode}>{headline}</p>

  <div class="bar" class:indet={indeterminate} class:cancel={!!cancelMode && finished} role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct}>
    <div class="fill" class:bad={finished && failedAny} style={`width:${pct}%`}></div>
  </div>
  <p class="muted count">
    {#if cancelMode && finished}{$t("Proceso cancelado")}
    {:else if realProgress || finished || doneCount > 0}{total > 1 ? `${doneCount} / ${total} · ` : ""}{pct} %
    {:else}&nbsp;{/if}
  </p>

  <ul class="selectable scrollbar-thin">
    {#each items as it}
      <li class={it.state}>
        <span class="st" aria-hidden="true">
          {#if it.state === "traballa"}<span class="spin"></span>
          {:else if it.state === "feito"}<Icon name="check" size={15} />
          {:else if it.state === "erro"}<Icon name="warning" size={15} />
          {:else}<span class="dot"></span>{/if}
        </span>
        <div class="txt">
          <span class="n" title={it.path}>{fileNameOf(it.path)}</span>
          {#if it.state === "feito" && it.out}
            <span class="muted sub">→ {fileNameOf(it.out)}{#if it.note}&nbsp;·&nbsp;<strong class="note">{it.note}</strong>{/if}</span>
          {:else if it.state === "erro"}
            <span class="err">{it.error}</span>
          {:else if it.state === "traballa"}
            <span class="muted sub">{realProgress ? `${itemPct} %` : $t("Traballando…")}</span>
          {:else if cancelMode && finished}
            <span class="muted sub">{$t("Cancelado")}</span>
          {:else}
            <span class="muted sub">{$t("Agardando")}</span>
          {/if}
        </div>
        {#if it.state === "feito" && it.out}
          <button class="btn ghost icon" on:click={() => engine.revealInFolder(it.out!)} title={$t("Amosar no cartafol")} aria-label={$t("Amosar no cartafol")}><Icon name="folder" size={15} /></button>
        {/if}
      </li>
    {/each}
  </ul>

  {#if missing}
    {@const info = TOOL_INFO[missing]}
    {@const busy = $installing?.tool === missing ? $installing : null}
    <section class="need">
      <p><strong>{$t("Falta un complemento")}</strong></p>
      <p class="muted">{$t("Para facer isto o trasno precisa {name}, que non está instalado.", { name: info.name })}</p>
      {#if busy}
        {@const p = Math.round(overall(busy) * 100)}
        <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={p}><div class="fill" style={`width:${p}%`}></div></div>
        <p class="muted count2">{phaseText(busy.phase)} · {p} %</p>
        <div class="acts"><button class="btn" on:click={() => engine.cancelToolInstall()}>{$t("Cancelar descarga")}</button></div>
      {:else}
        {#if $installError?.tool === missing}<p class="err">{$installError.text}</p>{/if}
        <div class="acts">
          <button class="btn primary" on:click={acceptInstall}>{$t("Descargar e instalar")} <span class="muted">· {info.size}</span></button>
          <button class="btn" on:click={() => engine.openLink(info.link)}><Icon name="open" size={14} /> {$t("Ir á páxina oficial")}</button>
          <button class="btn ghost" on:click={() => answer?.(false)}>{$t("Agora non")}</button>
        </div>
      {/if}
    </section>
  {/if}

  <footer>
    {#if finished && !failedAny && !keepOpen}<span class="muted">{$t("Péchase en {s} s…", { s: countdown })}</span>{/if}
    <span class="grow"></span>
    {#if asking}
      <span class="ask">{$t("Cancelar o traballo?")}</span>
      <div class="acts">
        <button class="btn danger" on:click={() => doCancel("partial")} title={$t("Para o traballo e borra só o ficheiro que quedaba a medias")}>{$t("Borrar só o incompleto")}</button>
        {#if doneCount > 0}
          <button class="btn danger" on:click={() => doCancel("all")} title={$t("Para o traballo e borra tamén o que xa estaba feito")}>{$t("Borrar todo")}</button>
        {/if}
        <button class="btn" on:click={() => (asking = false)}>{$t("Seguir")}</button>
      </div>
    {:else if running && !cancelMode}
      <div class="acts"><button class="btn" on:click={() => (asking = true)}>{$t("Cancelar")}</button></div>
    {:else if finished}
      <div class="acts">
        {#if !cancelMode && outs().length}<button class="btn" on:click={openFull} title={$t("Abre o resultado na aplicación completa, para velo ou seguir traballando con el")}><Icon name="open" size={14} /> {$t("Abrir co programa completo")}</button>{/if}
        <button class="btn primary" on:click={() => win.destroy()}>{$t("Pechar")}</button>
      </div>
    {/if}
  </footer>
</main>

<style>
  main { flex: 1; width: 100%; min-width: 0; height: 100vh; display: flex; flex-direction: column; gap: 0.55rem; padding: 1rem 1.2rem 0.9rem; background: var(--bg); }
  header { display: flex; align-items: center; gap: 0.7rem; }
  .titles { display: flex; flex-direction: column; line-height: 1.25; min-width: 0; }
  .titles strong { font-size: 0.9rem; }
  .what { font-size: 0.8rem; }
  .head { margin: 0.2rem 0 0; font-size: 1.02rem; font-weight: 600; color: var(--text); }
  .head.bad { color: var(--accent-video); }
  .head.cancel { color: var(--accent-pdf); }

  .bar { position: relative; height: 8px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; border-radius: 99px; background: var(--accent); transition: width 0.35s ease; }
  .fill.bad { background: var(--accent-video); }
  .bar.cancel .fill { background: var(--accent-pdf); }
  /* Sen avance medible (imaxes, PDF ou o ffmpeg aínda sen dato): unha franxa que vai e vén, sen porcentaxe inventada. */
  .bar.indet .fill { width: 30% !important; transition: none; animation: vaivén 1.1s ease-in-out infinite alternate; }
  @keyframes vaivén { from { margin-left: 0; } to { margin-left: 70%; } }
  .count { margin: -0.25rem 0 0; font-size: 0.75rem; text-align: right; font-variant-numeric: tabular-nums; min-height: 1.1em; }

  ul { list-style: none; margin: 0; padding: 0; flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 0.15rem; }
  li { display: flex; align-items: center; gap: 0.6rem; padding: 0.35rem 0.4rem; border-radius: var(--radius-sm); min-width: 0; }
  li.traballa { background: color-mix(in srgb, var(--accent) 10%, transparent); }
  .st { flex: 0 0 18px; display: flex; align-items: center; justify-content: center; color: var(--text-faint); }
  li.feito .st { color: var(--accent); }
  li.erro .st { color: var(--accent-video); }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
  .spin { width: 14px; height: 14px; border-radius: 50%; border: 2px solid var(--border-strong); border-top-color: var(--accent); animation: turn 0.8s linear infinite; }
  @keyframes turn { to { transform: rotate(360deg); } }
  .txt { flex: 1; min-width: 0; display: flex; flex-direction: column; line-height: 1.3; }
  .n { font-size: 0.86rem; font-weight: 550; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sub { font-size: 0.78rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .note { color: var(--accent); font-weight: 600; }
  .err { font-size: 0.78rem; color: var(--accent-video); word-break: break-word; }
  footer { display: flex; align-items: center; gap: 0.6rem; font-size: 0.8rem; padding-top: 0.5rem; border-top: 1px solid var(--border); flex-wrap: wrap; }
  .grow { flex: 1; }
  .ask { font-size: 0.8rem; color: var(--text); }
  /* Todos os botóns do pé miden o mesmo. */
  .need { display: flex; flex-direction: column; gap: 0.35rem; padding: 0.7rem 0.8rem; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--bg-card); }
  .need p { margin: 0; font-size: 0.84rem; }
  .need .count2 { font-size: 0.78rem; }
  .acts { display: flex; gap: 0.4rem; align-items: center; flex-wrap: wrap; }
  .acts :global(.btn) { height: var(--control-h); min-height: var(--control-h); padding-block: 0; box-sizing: border-box; }
</style>
