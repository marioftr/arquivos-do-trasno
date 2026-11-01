<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { tick } from "svelte";
  import { getPdf, cachedThumbs, thumbVersion, wantThumb, dropThumb, prefetchThumbs } from "../../pdfCache";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Icon from "../Icon.svelte";
  import JobStatus from "../ui/JobStatus.svelte";


  export let file: FileEntry;

  interface Tile {
    page: number;
    rotate: number;
    removed: boolean;
  }

  let tiles: Tile[] = [];
  let original: Tile[] = [];
  let loading = true;
  let loadError = "";
  const job = createJob();
  const busy = job.busy;

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
      const list: Tile[] = Array.from({ length: doc.numPages }, (_, k) => ({ page: k + 1, rotate: 0, removed: false }));
      tiles = list;
      original = list.map((x) => ({ ...x }));
      loading = false;
      await tick();
      start(doc.numPages);
    } catch (e) {
      loadError = errText(e);
      loading = false;
    }
  });

  function move(i: number, d: -1 | 1) {
    const j = i + d;
    if (j < 0 || j >= tiles.length) return;
    const next = [...tiles];
    [next[i], next[j]] = [next[j], next[i]];
    tiles = next;
  }
  /** Move a páxina da posición `from` á posición `to` (empezando en 0). */
  function moveTo(from: number, to: number) {
    to = Math.max(0, Math.min(tiles.length - 1, to));
    if (from === to || from < 0) return;
    const next = [...tiles];
    const [t] = next.splice(from, 1);
    next.splice(to, 0, t);
    tiles = next;
  }
  const setPos = (i: number, ev: Event) => {
    const el = ev.currentTarget as HTMLInputElement;
    const v = Math.round(Number(el.value));
    if (Number.isFinite(v)) moveTo(i, v - 1);
    el.value = String(tiles.findIndex((x) => x === tiles[i]) + 1);
  };

  // Arrastrar con punteiro (o arrastrar e soltar do navegador non vale: a ventá captura os ficheiros).
  let dragFrom = -1;
  let dragOver = -1;
  let dragging = false;
  let sx = 0, sy = 0, ox = 0, oy = 0;
  function pdown(e: PointerEvent, i: number) {
    if (e.button !== 0) return;
    dragFrom = i; dragOver = i; sx = e.clientX; sy = e.clientY; ox = 0; oy = 0;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function pmove(e: PointerEvent) {
    if (dragFrom < 0) return;
    if (!dragging && Math.hypot(e.clientX - sx, e.clientY - sy) > 6) dragging = true;
    if (!dragging) return;
    ox = e.clientX - sx; oy = e.clientY - sy;
    const el = document.elementFromPoint(e.clientX, e.clientY)?.closest("[data-i]") as HTMLElement | null;
    if (el) dragOver = Number(el.dataset.i);
  }
  function pup() {
    if (dragging && dragOver >= 0) moveTo(dragFrom, dragOver);
    dragFrom = -1; dragOver = -1; dragging = false; ox = 0; oy = 0;
  }
  const rotate = (i: number) => (tiles[i] = { ...tiles[i], rotate: (tiles[i].rotate + 90) % 360 }, (tiles = tiles));
  const toggle = (i: number) => (tiles[i] = { ...tiles[i], removed: !tiles[i].removed }, (tiles = tiles));
  const reverse = () => (tiles = [...tiles].reverse());
  const reset = () => (tiles = original.map((x) => ({ ...x })));

  $: kept = tiles.filter((x) => !x.removed);
  $: changed = JSON.stringify(tiles.map((x) => [x.page, x.rotate, x.removed])) !== JSON.stringify(original.map((x) => [x.page, x.rotate, x.removed]));

  async function save() {
    try {
      const out = await askOutput(file, `-${tr("ordenado")}`, "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const done = await engine.pdfArrange(file.path, out, kept.map((x) => [x.page, x.rotate] as [number, number]), true);
        return `${listo()} ${fileNameOf(done)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Ordenar páxinas")}</h4>
    <p class="group-note">{$t("Arrastra as páxinas ou escribe a posición onde queres poñelas. Tamén podes xirar ou quitar. O PDF orixinal non se toca: gárdase unha copia.")}</p>
    <div class="row">
      <button class="btn" on:click={reverse} disabled={loading}>{$t("Invertir a orde")}</button>
      <button class="btn ghost" on:click={reset} disabled={!changed}><Icon name="reset" size={14} /> {$t("Restablecer")}</button>
    </div>
    {#if loadError}
      <p class="group-note">{$t("Non puiden abrir o PDF: {error}", { error: loadError })}</p>
    {:else}
      <div class="grid">
        {#each tiles as tile, i (tile.page)}
          <div
            class="tile"
            class:removed={tile.removed}
            class:lifted={dragging && dragFrom === i}
            class:target={dragging && dragOver === i && dragFrom !== i}
            data-i={i}
            use:seen={tile.page}
            style={dragging && dragFrom === i ? `transform: translate(${ox}px, ${oy}px)` : ""}
          >
            <div class="thumb" on:pointerdown={(e) => pdown(e, i)} on:pointermove={pmove} on:pointerup={pup} on:pointercancel={pup} role="presentation">{#if thumbs[tile.page]}<img src={thumbs[tile.page]} alt={String(tile.page)} draggable="false" on:dragstart|preventDefault style={`transform: rotate(${tile.rotate}deg)`} />{/if}</div>
            <label class="num" title={$t("Posición")}><input class="pos" type="number" min="1" max={tiles.length} value={i + 1} on:change={(e) => setPos(i, e)} /><span>({tile.page})</span></label>
            <div class="acts">
              <button class="mini" on:click={() => move(i, -1)} disabled={i === 0} aria-label={$t("Mover cara atrás")} title={$t("Mover cara atrás")}><Icon name="back" size={13} /></button>
              <button class="mini" on:click={() => rotate(i)} aria-label={$t("Xirar 90° á dereita")} title={$t("Xirar 90° á dereita")}><Icon name="redo" size={13} /></button>
              <button class="mini" on:click={() => toggle(i)} aria-label={$t("Quitar")} title={$t("Quitar")}><Icon name="close" size={13} /></button>
              <button class="mini" on:click={() => move(i, 1)} disabled={i === tiles.length - 1} aria-label={$t("Mover cara adiante")} title={$t("Mover cara adiante")}><Icon name="arrow-right" size={13} /></button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
  <button class="btn primary block" on:click={save} disabled={$busy || loading || !changed || kept.length === 0}>
    {$t("Gardar con {n} páxinas", { n: kept.length })}
  </button>
  <JobStatus {job} />
</div>

<style>
  .row { display: flex; gap: 0.5rem; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(92px, 1fr)); gap: 0.6rem; }
  .tile { display: flex; flex-direction: column; align-items: center; gap: 0.2rem; padding: 0.4rem; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); }
  .tile.removed { opacity: 0.3; }
  .thumb { width: 100%; height: 96px; display: flex; align-items: center; justify-content: center; overflow: hidden; }
  .thumb img { max-width: 100%; max-height: 100%; background: #fff; transition: transform 0.15s ease; }
  .num { font-size: 0.74rem; color: var(--text-muted); display: flex; align-items: center; gap: 0.25rem; }
  .pos { width: 3.2rem; height: 1.7rem; text-align: center; background: var(--bg); color: var(--text); border: 1px solid var(--border-strong); border-radius: 4px; font: inherit; font-size: 0.8rem; }
  .pos:focus { outline: none; border-color: var(--accent); }
  .thumb { cursor: grab; touch-action: none; user-select: none; }
  .thumb img { -webkit-user-drag: none; pointer-events: none; }
  .tile.lifted { position: relative; z-index: 5; pointer-events: none; opacity: 0.85; box-shadow: 0 6px 18px rgba(0, 0, 0, 0.45); }
  .tile.target { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 14%, var(--bg-input)); }
  .acts { display: flex; gap: 0.1rem; }
  .mini { display: flex; background: none; border: none; color: var(--text-muted); padding: 0.25rem; border-radius: 4px; }
  .mini:hover:not(:disabled) { color: var(--text); background: var(--bg-hover); }
  .mini:disabled { opacity: 0.25; cursor: default; }
</style>
