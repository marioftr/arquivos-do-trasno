<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import * as pdfjs from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
  import { errText } from "../../errors";
  import { viewUrl, type FileEntry } from "../../files";
  import { t } from "../../i18n";

  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

  /** Miniaturas das páxinas dun PDF; ao facer clic nunha avisa para marcala ou desmarcala. */
  export let file: FileEntry;
  export let selected: number[] = [];

  const dispatch = createEventDispatcher<{ toggle: number; total: number }>();
  let thumbs: string[] = [];
  let loading = true;
  let error = "";
  let alive = true;

  onMount(async () => {
    try {
      const res = await fetch(await viewUrl(file.path));
      const doc = await pdfjs.getDocument({ data: new Uint8Array(await res.arrayBuffer()) }).promise;
      dispatch("total", doc.numPages);
      for (let n = 1; n <= doc.numPages && alive; n++) {
        const page = await doc.getPage(n);
        const vp = page.getViewport({ scale: 0.3 });
        const canvas = document.createElement("canvas");
        canvas.width = vp.width;
        canvas.height = vp.height;
        const ctx = canvas.getContext("2d")!;
        ctx.fillStyle = "#fff";
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        await page.render({ canvasContext: ctx, viewport: vp }).promise;
        thumbs = [...thumbs, canvas.toDataURL("image/jpeg", 0.7)];
      }
      await doc.destroy();
    } catch (e) {
      error = errText(e);
    } finally {
      loading = false;
    }
  });
  onDestroy(() => (alive = false));

  $: set = new Set(selected);
</script>

{#if error}
  <p class="group-note">{$t("Non puiden abrir o PDF: {error}", { error })}</p>
{:else}
  <div class="grid">
    {#each thumbs as src, i}
      <button class="tile" class:on={set.has(i + 1)} on:click={() => dispatch("toggle", i + 1)} aria-pressed={set.has(i + 1)} title={$t("Páxina {n}", { n: i + 1 })}>
        <span class="thumb"><img {src} alt={String(i + 1)} draggable="false" on:dragstart|preventDefault /></span>
        <span class="num">{i + 1}</span>
      </button>
    {/each}
  </div>
  {#if loading}<p class="group-note">{$t("O trasno está debuxando as miniaturas…")}</p>{/if}
{/if}

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(78px, 1fr)); gap: 0.5rem; }
  .tile { display: flex; flex-direction: column; align-items: center; gap: 0.2rem; padding: 0.35rem; border: 2px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); color: var(--text-muted); cursor: pointer; }
  .tile:hover { border-color: var(--border-strong); }
  .tile.on { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 16%, var(--bg-input)); color: var(--text); font-weight: 600; }
  .thumb { width: 100%; height: 84px; display: flex; align-items: center; justify-content: center; overflow: hidden; }
  .thumb img { max-width: 100%; max-height: 100%; background: #fff; user-select: none; -webkit-user-drag: none; pointer-events: none; }
  .tile:not(.on) .thumb img { opacity: 0.75; }
  .num { font-size: 0.74rem; }
</style>
