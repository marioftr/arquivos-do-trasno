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
  let total = 0;
  let thumbs: Record<number, string> = {};
  let error = "";
  let alive = true;
  let doc: pdfjs.PDFDocumentProxy | null = null;
  let grid: HTMLDivElement;
  let io: IntersectionObserver | null = null;
  const wanted = new Set<number>();
  let drawing = false;

  // Só se debuxan as miniaturas que se ven (ou están a piques de verse): un PDF de 300 páxinas non conxela a xanela.
  async function drain() {
    if (drawing) return;
    drawing = true;
    while (alive && doc && wanted.size) {
      const n = [...wanted].sort((x, y) => x - y)[0];
      wanted.delete(n);
      if (thumbs[n]) continue;
      try {
        const page = await doc.getPage(n);
        const vp = page.getViewport({ scale: 0.3 });
        const canvas = document.createElement("canvas");
        canvas.width = vp.width;
        canvas.height = vp.height;
        const ctx = canvas.getContext("2d")!;
        ctx.fillStyle = "#fff";
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        await page.render({ canvasContext: ctx, viewport: vp }).promise;
        thumbs[n] = canvas.toDataURL("image/jpeg", 0.7);
        thumbs = thumbs;
      } catch {
        // unha páxina que non se debuxa queda en branco
      }
    }
    drawing = false;
  }

  function seen(node: HTMLElement, n: number) {
    io?.observe(node);
    node.dataset.n = String(n);
    return { destroy: () => io?.unobserve(node) };
  }

  onMount(async () => {
    try {
      const res = await fetch(await viewUrl(file.path));
      doc = await pdfjs.getDocument({ data: new Uint8Array(await res.arrayBuffer()) }).promise;
      io = new IntersectionObserver(
        (entries) => {
          for (const e of entries) if (e.isIntersecting) wanted.add(Number((e.target as HTMLElement).dataset.n));
          drain();
        },
        { rootMargin: "300px" },
      );
      total = doc.numPages;
      dispatch("total", total);
    } catch (e) {
      error = errText(e);
    }
  });
  onDestroy(() => {
    alive = false;
    io?.disconnect();
    doc?.destroy();
  });

  $: set = new Set(selected);
</script>

{#if error}
  <p class="group-note">{$t("Non puiden abrir o PDF: {error}", { error })}</p>
{:else}
  <div class="grid" bind:this={grid}>
    {#each Array.from({ length: total }, (_, k) => k + 1) as n (n)}
      <button class="tile" class:on={set.has(n)} on:click={() => dispatch("toggle", n)} aria-pressed={set.has(n)} title={$t("Páxina {n}", { n })} use:seen={n}>
        <span class="thumb">{#if thumbs[n]}<img src={thumbs[n]} alt={String(n)} draggable="false" on:dragstart|preventDefault />{/if}</span>
        <span class="num">{n}</span>
      </button>
    {/each}
  </div>
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
