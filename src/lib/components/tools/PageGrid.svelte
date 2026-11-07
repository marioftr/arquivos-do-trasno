<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import { tick } from "svelte";
  import { getPdf, cachedThumbs, thumbVersion, wantThumb, dropThumb, prefetchThumbs } from "../../pdfCache";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { t } from "../../i18n";


  /** Miniaturas das páxinas dun PDF; ao facer clic nunha avisa para marcala ou desmarcala. */
  export let file: FileEntry;
  export let selected: number[] = [];

  const dispatch = createEventDispatcher<{ toggle: number; total: number }>();
  let total = 0;
  let error = "";
  // Só se piden as miniaturas que se ven; o resto vai chegando en segundo plano e queda gardado para o outro modo.
  let thumbs: Record<number, string> = {};
  let io: IntersectionObserver | null = null;
  let alive = true;
  const sync = () => {
    const m = cachedThumbs(file.path);
    let changed = false;
    for (const [n, u] of m) if (thumbs[n] !== u) { thumbs[n] = u; changed = true; }
    if (changed) thumbs = thumbs;
  };
  const unsub = thumbVersion.subscribe(() => sync());
  function seen(node: HTMLElement, n: number) {
    node.dataset.page = String(n);
    io?.observe(node);
    return { destroy: () => io?.unobserve(node) };
  }
  function start(total: number) {
    io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          const n = Number((e.target as HTMLElement).dataset.page);
          if (e.isIntersecting) wantThumb(file.path, n);
          else dropThumb(file.path, n);
        }
      },
      { rootMargin: "400px" },
    );
    prefetchThumbs(file.path, total);
    sync();
  }
  onDestroy(() => {
    alive = false;
    io?.disconnect();
    unsub();
  });

  onMount(async () => {
    try {
      const doc = await getPdf(file.path);
      total = doc.numPages;
      dispatch("total", total);
      await tick();
      start(total);
    } catch (e) {
      error = errText(e);
    }
  });

  $: set = new Set(selected);
</script>

{#if error}
  <p class="group-note">{$t("Non puiden abrir o PDF: {error}", { error })}</p>
{:else}
  <div class="grid">
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
  .num { font-size: var(--fs-xs); }
</style>
