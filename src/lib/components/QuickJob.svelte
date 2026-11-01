<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import * as engine from "../engine";
  import { errText } from "../errors";
  import { fallo, fileNameOf, listo, TRASNO, withExtension } from "../formats";
  import { t, tr } from "../i18n";
  import BrandMark from "./BrandMark.svelte";
  import Icon from "./Icon.svelte";

  /** Traballo lanzado desde o clic dereito do Explorador: xanela pequena que fai a faena e se pecha soa. */
  export let job: engine.StartupJob;

  interface Item {
    path: string;
    state: "agarda" | "traballa" | "feito" | "erro";
    out?: string;
    error?: string;
  }
  let items: Item[] = job.paths.map((path) => ({ path, state: "agarda" }));
  let headline = tr(job.action === "convert" ? TRASNO.convertendo : TRASNO.comprimindo);
  let finished = false;
  let failedAny = false;
  let keepOpen = false;
  let countdown = 6;
  const win = getCurrentWindow();

  const extOf = (p: string) => (p.match(/\.([^.\\/]+)$/)?.[1] ?? "").toLowerCase();

  async function one(it: Item) {
    const ext = extOf(it.path);
    const cat = await engine.detectCategory(it.path);
    if (job.action === "convert") {
      const out = await engine.uniquePath(withExtension(it.path, job.target!));
      if (cat === "imaxe" && job.target === "pdf") await engine.imagesToPdf([it.path], out, false);
      else await engine.convertFile(it.path, out, { overwrite: false });
      return out;
    }
    // Comprimir con axustes recomendados; o orixinal non se toca.
    const stem = it.path.replace(/\.[^.\\/]+$/, "");
    if (cat === "imaxe") {
      const out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, ext === "png" ? "jpg" : ext));
      await engine.compressImage(it.path, out, { quality: 75 }, false);
      return out;
    }
    if (cat === "video") {
      const out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, "mp4"));
      await engine.compressVideo(it.path, out, { crf: 26, audioKbps: 128 }, false);
      return out;
    }
    const out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, "pdf"));
    await engine.pdfCompress(it.path, out, 70, 75, false);
    return out;
  }

  onMount(async () => {
    try {
      await win.setTitle("Arquivos do Trasno");
      await win.setSize(new LogicalSize(480, 230));
      await win.center();
      await win.setAlwaysOnTop(true);
    } catch {
      // sen permiso para axustar a xanela: segue igual
    }
    for (const it of items) {
      it.state = "traballa";
      items = items;
      try {
        it.out = await one(it);
        it.state = "feito";
      } catch (e) {
        it.state = "erro";
        it.error = errText(e);
        failedAny = true;
      }
      items = items;
    }
    finished = true;
    headline = failedAny ? fallo(job.action === "convert" ? "convertendo" : "comprimindo") : listo();
    if (!failedAny) {
      const timer = setInterval(() => {
        if (keepOpen) return clearInterval(timer);
        countdown -= 1;
        if (countdown <= 0) {
          clearInterval(timer);
          win.close();
        }
      }, 1000);
    }
  });

  const reveal = (p: string) => engine.revealInFolder(p);
  const done = items.length;
</script>

<svelte:window on:pointerdown={() => (keepOpen = true)} on:keydown={(e) => e.key === "Escape" && win.close()} />

<main>
  <header>
    <BrandMark size={22} />
    <strong>Arquivos do Trasno</strong>
  </header>
  <p class="head" class:bad={finished && failedAny}>{headline}</p>
  <ul class="selectable">
    {#each items as it}
      <li class={it.state}>
        <span class="n" title={it.path}>{fileNameOf(it.path)}</span>
        {#if it.state === "traballa"}
          <span class="muted">{$t("Traballando…")}</span>
        {:else if it.state === "feito" && it.out}
          <span class="muted">→ {fileNameOf(it.out)}</span>
          <button class="btn ghost" on:click={() => reveal(it.out!)} title={$t("Amosar no cartafol")}><Icon name="folder" size={14} /></button>
        {:else if it.state === "erro"}
          <span class="err">{it.error}</span>
        {:else}
          <span class="muted">{$t("Agardando")}</span>
        {/if}
      </li>
    {/each}
  </ul>
  <footer>
    {#if finished && !failedAny && !keepOpen}<span class="muted">{$t("Péchase en {s} s…", { s: countdown })}</span>{/if}
    <span class="grow"></span>
    <button class="btn" on:click={() => win.close()}>{$t("Pechar")}</button>
  </footer>
</main>

<style>
  main { height: 100vh; display: flex; flex-direction: column; gap: 0.6rem; padding: 0.9rem 1.1rem; background: var(--bg); }
  header { display: flex; align-items: center; gap: 0.5rem; font-size: 0.85rem; }
  .head { margin: 0; font-size: 0.95rem; font-weight: 600; color: var(--accent); }
  .head.bad { color: var(--accent-video); }
  ul { list-style: none; margin: 0; padding: 0; flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; }
  li { display: flex; align-items: center; gap: 0.5rem; min-width: 0; }
  .n { font-weight: 550; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 45%; }
  .err { color: var(--accent-video); word-break: break-word; }
  footer { display: flex; align-items: center; gap: 0.5rem; font-size: 0.8rem; }
  .grow { flex: 1; }
</style>
