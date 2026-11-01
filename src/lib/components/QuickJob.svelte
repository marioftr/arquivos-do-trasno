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
    note?: string;
    error?: string;
  }
  let items: Item[] = job.paths.map((path) => ({ path, state: "agarda" }));
  let headline = tr(job.action === "convert" ? TRASNO.convertendo : job.action === "compress" ? TRASNO.comprimindo : TRASNO.traballando);
  let finished = false;
  let failedAny = false;
  let keepOpen = false;
  let countdown = 6;
  const win = getCurrentWindow();

  const extOf = (p: string) => (p.match(/\.([^.\\/]+)$/)?.[1] ?? "").toLowerCase();

  const pctLess = (before: number, after: number) => (before > 0 ? Math.round((1 - after / before) * 100) : 0);

  /** Fai o traballo cun ficheiro e devolve a ruta de saída e, se vén ao caso, unha nota (p. ex. «62 % menos»). */
  async function one(it: Item): Promise<{ out: string; note?: string }> {
    const ext = extOf(it.path);
    const cat = await engine.detectCategory(it.path);
    const stem = it.path.replace(/\.[^.\\/]+$/, "");
    const arg = job.target ?? "";

    if (job.action === "convert") {
      const out = await engine.uniquePath(withExtension(it.path, arg));
      if (cat === "imaxe" && arg === "pdf") await engine.imagesToPdf([it.path], out, false);
      else await engine.convertFile(it.path, out, { overwrite: false });
      return { out };
    }

    if (job.action === "compress") {
      // O nivel é a calidade que se quere manter: 75, 50 ou 25 %.
      const level = Math.min(100, Math.max(10, Number(arg) || 75));
      const before = await engine.fileSize(it.path);
      let out: string;
      if (cat === "imaxe") {
        out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, ext === "png" ? "jpg" : ext));
        await engine.compressImage(it.path, out, { quality: level }, false);
      } else if (cat === "video") {
        out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, "mp4"));
        const crf = level >= 75 ? 24 : level >= 50 ? 28 : 34;
        await engine.compressVideo(it.path, out, { crf, audioKbps: level >= 75 ? 160 : 128 }, false);
      } else {
        out = await engine.uniquePath(withExtension(`${stem}${tr("-comprimido")}`, "pdf"));
        const [q, sc] = level >= 75 ? [70, 75] : level >= 50 ? [55, 60] : [40, 50];
        await engine.pdfCompress(it.path, out, q, sc, false);
      }
      const after = await engine.fileSize(out);
      const less = pctLess(before, after);
      return { out, note: less > 0 ? tr("{pct} % menos", { pct: less }) : tr("xa estaba moi comprimido") };
    }

    if (job.action === "resize") {
      const before = await engine.fileSize(it.path);
      if (cat === "imaxe") {
        const p = Number(arg.slice(1)) || 50;
        const [w, h] = await engine.imageDimensions(it.path);
        const out = await engine.uniquePath(withExtension(`${stem}-${p}pc`, ext));
        await engine.editImage(it.path, out, { resize: [Math.max(1, Math.round((w * p) / 100)), Math.max(1, Math.round((h * p) / 100))] }, false);
        return { out, note: `${w}×${h} → ${Math.round((w * p) / 100)}×${Math.round((h * p) / 100)}` };
      }
      const hgt = Number(arg.slice(1)) || 720;
      const out = await engine.uniquePath(withExtension(`${stem}-${hgt}p`, ext));
      await engine.mediaEdit(it.path, out, "scale", hgt, true, false);
      const less = pctLess(before, await engine.fileSize(out));
      return { out, note: less > 0 ? tr("{pct} % menos", { pct: less }) : undefined };
    }

    // rotate
    const deg = Number(arg) || 90;
    const out = await engine.uniquePath(withExtension(`${stem}${tr("-xirado")}`, ext));
    if (cat === "imaxe") await engine.editImage(it.path, out, { rotate: deg }, false);
    else await engine.pdfRotate(it.path, null, deg, out, false);
    return { out };
  }

  onMount(async () => {
    try {
      await win.setTitle("Arquivos do Trasno");
      await win.setSize(new LogicalSize(520, Math.min(560, 190 + job.paths.length * 28)));
      await win.center();
      await win.setAlwaysOnTop(true);
    } catch {
      // sen permiso para axustar a xanela: segue igual
    }
    for (const it of items) {
      it.state = "traballa";
      items = items;
      try {
        const r = await one(it);
        it.out = r.out;
        it.note = r.note;
        it.state = "feito";
      } catch (e) {
        it.state = "erro";
        it.error = errText(e);
        failedAny = true;
      }
      items = items;
    }
    finished = true;
    headline = failedAny ? fallo(job.action === "convert" ? "convertendo" : job.action === "compress" ? "comprimindo" : "editando") : listo();
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
          {#if it.note}<strong class="note">{it.note}</strong>{/if}
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
  .note { color: var(--accent); font-size: 0.8rem; white-space: nowrap; }
  .err { color: var(--accent-video); word-break: break-word; }
  footer { display: flex; align-items: center; gap: 0.5rem; font-size: 0.8rem; }
  .grow { flex: 1; }
</style>
