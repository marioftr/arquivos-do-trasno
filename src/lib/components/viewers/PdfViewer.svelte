<script lang="ts">
  import { errText } from "../../errors";
  import { onDestroy } from "svelte";
  import * as pdfjs from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
  import { viewUrl, type FileEntry } from "../../files";
  import { t } from "../../i18n";
  import Slider from "../ui/Slider.svelte";

  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

  export let file: FileEntry;

  let container: HTMLDivElement;
  let doc: pdfjs.PDFDocumentProxy | null = null;
  let scale = 80;
  let error = "";
  let token = 0;

  $: open(file.path);
  $: if (doc && container) render(scale);

  async function open(path: string) {
    const mine = ++token;
    error = "";
    try {
      await doc?.destroy();
      doc = null;
      const res = await fetch(await viewUrl(path));
      const data = new Uint8Array(await res.arrayBuffer());
      const loaded = await pdfjs.getDocument({ data }).promise;
      if (mine !== token) {
        loaded.destroy();
        return;
      }
      doc = loaded;
    } catch (e) {
      error = errText(e);
    }
  }

  async function render(percent: number) {
    const mine = ++token;
    const current = doc;
    if (!current) return;
    container.replaceChildren();
    const dpr = window.devicePixelRatio || 1;
    for (let n = 1; n <= current.numPages; n++) {
      if (mine !== token) return;
      const page = await current.getPage(n);
      const viewport = page.getViewport({ scale: (percent / 100) * 1.333 });
      const canvas = document.createElement("canvas");
      canvas.width = viewport.width * dpr;
      canvas.height = viewport.height * dpr;
      canvas.style.width = `${viewport.width}px`;
      canvas.style.height = `${viewport.height}px`;
      container.appendChild(canvas);
      await page.render({
        canvasContext: canvas.getContext("2d")!,
        viewport,
        transform: dpr !== 1 ? [dpr, 0, 0, dpr, 0, 0] : undefined,
      }).promise;
    }
  }

  onDestroy(() => {
    token++;
    doc?.destroy();
  });
</script>

<div class="wrap">
  {#if error}
    <p class="muted msg">{$t("Non puiden abrir o PDF: {error}", { error })}</p>
  {:else}
    <div class="pages scrollbar-thin" bind:this={container}></div>
  {/if}
  <div class="bar">
    <Slider compact label={$t("Zoom")} unit="%" min={40} max={250} bind:value={scale} />
    {#if doc}<span class="muted pag">{$t("{n} páxinas", { n: doc.numPages })}</span>{/if}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .pages {
    flex: 1; min-height: 0; overflow: auto;
    display: flex; flex-direction: column; align-items: flex-start; gap: 12px;
    padding: 16px; background: #0f0f12;
  }
  .pages :global(canvas) { background: #fff; flex-shrink: 0; margin-inline: auto; }
  .bar { display: flex; align-items: center; gap: 0.6rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .pag { margin-left: auto; }
  .msg { padding: 2rem; }
</style>
