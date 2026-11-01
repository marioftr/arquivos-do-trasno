<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import * as engine from "../engine";
  import { errText, isCancelled } from "../errors";
  import { fallo, fileNameOf, listo, TRASNO } from "../formats";
  import { t, tr } from "../i18n";
  import { runOne } from "../quickjob";
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
  let cancelled = false;
  let running = true;
  const win = getCurrentWindow();

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
  // A barra avanza por ficheiros rematados e medio máis mentres hai un en marcha (non hai progreso interno por ficheiro).
  // Avance real do ffmpeg para o ficheiro en marcha (0 a 1); as imaxes e os PDF non o dan e usan un avance estimado.
  let frac = 0;
  let realProgress = false;
  $: inner = realProgress ? frac : working ? 0.4 : 0;
  $: pct = finished ? 100 : Math.min(99, Math.round(((doneCount + inner) / total) * 100));
  $: itemPct = Math.round(frac * 100);

  /** Cancelar: para o ffmpeg, esquece o resto e borra o que este traballo leva creado (tamén os ficheiros xa rematados). */
  async function doCancel() {
    asking = false;
    cancelled = true;
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
    // Pechar coa X mentres traballa equivale a cancelar: antes pídese confirmación.
    await win.onCloseRequested((e) => {
      if (running && !cancelled) {
        e.preventDefault();
        asking = true;
      }
    });
    try {
      await win.setTitle("Arquivos do Trasno");
      await win.setResizable(false);
      await win.setMaximizable(false);
      await win.setSize(new LogicalSize(500, Math.min(560, 210 + items.length * 34)));
      await win.center();
    } catch {
      // sen permiso para axustar a xanela: segue igual
    }
    // A xanela naceu agochada: móstrase xa co tamaño bo e co contido pintado.
    await win.show();

    let unlisten: (() => void) | null = null;
    try {
      await engine.enableProgress();
      unlisten = await listen<number>("trasno://progress", (e) => {
        realProgress = true;
        frac = Math.max(0, Math.min(1, e.payload));
      });
    } catch {
      // sen avance real: a barra usa o estimado
    }
    for (const it of items) {
      if (cancelled) {
        it.state = "agarda";
        continue;
      }
      await engine.beginJob().catch(() => {});
      frac = 0;
      realProgress = false;
      it.state = "traballa";
      items = items;
      try {
        const r = await runOne(job, it.path);
        it.out = r.out;
        it.note = r.note;
        it.state = "feito";
      } catch (e) {
        if (cancelled || isCancelled(e)) {
          cancelled = true;
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
    running = false;
    if (cancelled) {
      await discardAll();
      items = items;
      finished = true;
      headline = tr("Cancelaches o traballo. Non quedou ningún ficheiro a medias.");
      setTimeout(() => win.destroy(), 2500);
      return;
    }
    finished = true;
    headline = failedAny ? fallo(job.action === "convert" ? "convertendo" : job.action === "compress" ? "comprimindo" : "editando") : listo();
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

  const firstOut = () => items.find((i) => i.out)?.out;
</script>

<svelte:window on:pointerdown={() => (keepOpen = true)} on:keydown={(e) => e.key === "Escape" && (running && !cancelled ? (asking = true) : win.destroy())} />

<main>
  <header>
    <BrandMark size={26} />
    <div class="titles">
      <strong>Arquivos do Trasno</strong>
      <span class="muted what">{what}</span>
    </div>
  </header>

  <p class="head" class:bad={finished && failedAny}>{headline}</p>

  <div class="bar" class:busy={!finished} role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct}>
    <div class="fill" class:bad={finished && failedAny} style={`width:${pct}%`}></div>
  </div>
  <p class="muted count">{total > 1 ? `${doneCount} / ${total} · ` : ""}{pct} %</p>

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
            <span class="muted sub">→ {fileNameOf(it.out)}{#if it.note} · <strong class="note">{it.note}</strong>{/if}</span>
          {:else if it.state === "erro"}
            <span class="err">{it.error}</span>
          {:else if it.state === "traballa"}
            <span class="muted sub">{realProgress ? `${itemPct} %` : $t("Traballando…")}</span>
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

  <footer>
    {#if finished && !failedAny && !cancelled && !keepOpen}<span class="muted">{$t("Péchase en {s} s…", { s: countdown })}</span>{/if}
    <span class="grow"></span>
    {#if finished && !cancelled && firstOut()}<button class="btn" on:click={() => engine.revealInFolder(firstOut()!)}><Icon name="folder" size={14} /> {$t("Amosar no cartafol")}</button>{/if}
    {#if asking}
      <span class="ask">{$t("Cancelar e borrar o que leva feito?")}</span>
      <button class="btn danger" on:click={doCancel}>{$t("Si, cancelar")}</button>
      <button class="btn" on:click={() => (asking = false)}>{$t("Seguir")}</button>
    {:else if running && !cancelled}
      <button class="btn" on:click={() => (asking = true)}>{$t("Cancelar")}</button>
    {:else}
      <button class="btn primary" on:click={() => win.destroy()}>{$t("Pechar")}</button>
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

  .bar { position: relative; height: 8px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; border-radius: 99px; background: var(--accent); transition: width 0.35s ease; }
  .fill.bad { background: var(--accent-video); }
  .bar.busy .fill { background-image: linear-gradient(110deg, transparent 30%, rgba(255, 255, 255, 0.4) 50%, transparent 70%); background-size: 200% 100%; background-color: var(--accent); animation: slide 1.2s linear infinite; }
  @keyframes slide { from { background-position: 200% 0; } to { background-position: -200% 0; } }
  .count { margin: -0.25rem 0 0; font-size: 0.75rem; text-align: right; font-variant-numeric: tabular-nums; }

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
  footer { display: flex; align-items: center; gap: 0.5rem; font-size: 0.8rem; padding-top: 0.2rem; border-top: 1px solid var(--border); }
  .grow { flex: 1; }
  .ask { font-size: 0.8rem; color: var(--text); }
</style>
