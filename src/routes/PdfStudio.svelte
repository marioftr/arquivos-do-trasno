<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import StudioHeader from "../lib/components/StudioHeader.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import * as engine from "../lib/engine";
  import { fileNameOf } from "../lib/formats";

  const PDF_FILTER = [{ name: "PDF", extensions: ["pdf"] }];

  type Status = { ok: boolean; text: string } | null;

  let dragging = false;
  let unlistenDrop: (() => void) | null = null;

  onMount(async () => {
    const webview = getCurrentWebview();
    unlistenDrop = await webview.onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        dragging = false;
        addMergeFiles(event.payload.paths);
      } else if (event.payload.type === "enter" || event.payload.type === "over") {
        dragging = true;
      } else {
        dragging = false;
      }
    });
  });

  onDestroy(() => unlistenDrop?.());

  // ---------------------------------------------------------------- Unir --

  let mergeFiles: string[] = [];
  let mergeStatus: Status = null;
  let merging = false;

  function addMergeFiles(paths: string[]) {
    const pdfs = paths.filter((p) => p.toLowerCase().endsWith(".pdf"));
    const novos = pdfs.filter((p) => !mergeFiles.includes(p));
    if (novos.length) mergeFiles = [...mergeFiles, ...novos];
  }

  async function escollerParaUnir() {
    const chosen = await open({ multiple: true, filters: PDF_FILTER });
    if (!chosen) return;
    addMergeFiles(Array.isArray(chosen) ? chosen : [chosen]);
  }

  function moverArriba(i: number) {
    if (i === 0) return;
    const copia = [...mergeFiles];
    [copia[i - 1], copia[i]] = [copia[i], copia[i - 1]];
    mergeFiles = copia;
  }

  function moverAbaixo(i: number) {
    if (i === mergeFiles.length - 1) return;
    const copia = [...mergeFiles];
    [copia[i + 1], copia[i]] = [copia[i], copia[i + 1]];
    mergeFiles = copia;
  }

  function quitarDeUnir(i: number) {
    mergeFiles = mergeFiles.filter((_, idx) => idx !== i);
    mergeStatus = null;
  }

  async function unir() {
    if (mergeFiles.length < 2) return;
    const output = await save({ defaultPath: "unido.pdf", filters: PDF_FILTER });
    if (!output) return;
    merging = true;
    mergeStatus = null;
    try {
      await engine.pdfMerge(mergeFiles, output, true);
      mergeStatus = { ok: true, text: `Feito: ${fileNameOf(output)}` };
    } catch (err) {
      mergeStatus = { ok: false, text: String(err) };
    }
    merging = false;
  }

  // ------------------------------------------------------ Extraer/dividir --

  let extractInput: string | null = null;
  let extractPageCount: number | null = null;
  let extractSpec = "";
  let extractStatus: Status = null;
  let extracting = false;

  async function escollerParaExtraer() {
    const chosen = await open({ filters: PDF_FILTER });
    if (typeof chosen !== "string") return;
    extractInput = chosen;
    extractSpec = "";
    extractStatus = null;
    try {
      extractPageCount = await engine.pdfPageCount(chosen);
    } catch (err) {
      extractPageCount = null;
      extractStatus = { ok: false, text: String(err) };
    }
  }

  /** Interpreta "1,3,5-7" nunha lista de páxinas (1-indexada), na orde escrita. */
  function parsePageSpec(spec: string, max: number): number[] | null {
    const partes = spec
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    if (partes.length === 0) return null;
    const out: number[] = [];
    for (const parte of partes) {
      const rango = parte.match(/^(\d+)-(\d+)$/);
      if (rango) {
        const a = Number(rango[1]);
        const b = Number(rango[2]);
        if (a < 1 || b < a || b > max) return null;
        for (let n = a; n <= b; n++) out.push(n);
      } else if (/^\d+$/.test(parte)) {
        const n = Number(parte);
        if (n < 1 || n > max) return null;
        out.push(n);
      } else {
        return null;
      }
    }
    return out;
  }

  async function extraer() {
    if (!extractInput || extractPageCount === null) return;
    const pages = parsePageSpec(extractSpec, extractPageCount);
    if (!pages) {
      extractStatus = {
        ok: false,
        text: `Escribe páxinas válidas entre 1 e ${extractPageCount} (ex: "1,3,5-7")`,
      };
      return;
    }
    const base = fileNameOf(extractInput).replace(/\.pdf$/i, "");
    const output = await save({ defaultPath: `${base}-extracto.pdf`, filters: PDF_FILTER });
    if (!output) return;
    extracting = true;
    extractStatus = null;
    try {
      await engine.pdfExtractPages(extractInput, pages, output, true);
      extractStatus = { ok: true, text: `Feito: ${fileNameOf(output)} (${pages.length} páx.)` };
    } catch (err) {
      extractStatus = { ok: false, text: String(err) };
    }
    extracting = false;
  }

  // ----------------------------------------------------------- Rotar ------

  let rotateInput: string | null = null;
  let rotatePageCount: number | null = null;
  let rotateAllPages = true;
  let rotatePage = 1;
  let rotateDegrees = 90;
  let rotateStatus: Status = null;
  let rotating = false;

  async function escollerParaRotar() {
    const chosen = await open({ filters: PDF_FILTER });
    if (typeof chosen !== "string") return;
    rotateInput = chosen;
    rotateStatus = null;
    rotateAllPages = true;
    rotatePage = 1;
    try {
      rotatePageCount = await engine.pdfPageCount(chosen);
    } catch (err) {
      rotatePageCount = null;
      rotateStatus = { ok: false, text: String(err) };
    }
  }

  async function rotar() {
    if (!rotateInput) return;
    const base = fileNameOf(rotateInput).replace(/\.pdf$/i, "");
    const output = await save({ defaultPath: `${base}-rotado.pdf`, filters: PDF_FILTER });
    if (!output) return;
    rotating = true;
    rotateStatus = null;
    try {
      const page = rotateAllPages ? null : rotatePage;
      await engine.pdfRotate(rotateInput, page, rotateDegrees, output, true);
      rotateStatus = { ok: true, text: `Feito: ${fileNameOf(output)}` };
    } catch (err) {
      rotateStatus = { ok: false, text: String(err) };
    }
    rotating = false;
  }
</script>

<StudioHeader title="Trasno PDF" icon="document" accent="var(--accent-pdf)" />

<div class="body scrollbar-thin" class:dragging>
  <div class="content">
    <section class="card">
      <h3>
        <Icon name="layers" size={16} />
        Unir varios PDF nun só
      </h3>
      <p class="muted hint">
        Arrastra PDFs aquí ou engádeos á man. A orde da lista é a orde final do documento.
      </p>

      <div class="toolbar">
        <button class="btn" on:click={escollerParaUnir}>
          <Icon name="plus" size={14} />
          Engadir PDFs
        </button>
        <button class="btn primary" on:click={unir} disabled={mergeFiles.length < 2 || merging}>
          {merging ? "Unindo…" : "Unir"}
        </button>
      </div>

      {#if mergeFiles.length === 0}
        <p class="muted empty">Ningún PDF engadido aínda.</p>
      {:else}
        <ol class="lista">
          {#each mergeFiles as path, i (path)}
            <li>
              <span class="nome" title={path}>{fileNameOf(path)}</span>
              <span class="fila-acciones">
                <button class="mini" on:click={() => moverArriba(i)} disabled={i === 0} title="Subir">↑</button>
                <button class="mini" on:click={() => moverAbaixo(i)} disabled={i === mergeFiles.length - 1} title="Baixar">↓</button>
                <button class="mini" on:click={() => quitarDeUnir(i)} title="Quitar" aria-label="Quitar">
                  <Icon name="close" size={13} />
                </button>
              </span>
            </li>
          {/each}
        </ol>
      {/if}

      {#if mergeStatus}
        <p class:ok={mergeStatus.ok} class:erro={!mergeStatus.ok}>{mergeStatus.text}</p>
      {/if}
    </section>

    <section class="card">
      <h3>
        <Icon name="convert" size={16} />
        Extraer ou dividir páxinas
      </h3>
      <p class="muted hint">Elixe as páxinas que queres nun novo PDF (serve tamén para reordenar).</p>

      <div class="toolbar">
        <button class="btn" on:click={escollerParaExtraer}>
          <Icon name="folder" size={14} />
          {extractInput ? fileNameOf(extractInput) : "Escoller PDF"}
        </button>
        {#if extractPageCount !== null}
          <span class="muted paxinas">{extractPageCount} páxinas</span>
          <input
            class="spec-input"
            type="text"
            placeholder="ex: 1,3,5-7"
            bind:value={extractSpec}
          />
          <button class="btn primary" on:click={extraer} disabled={extracting || !extractSpec}>
            {extracting ? "Extraendo…" : "Extraer"}
          </button>
        {/if}
      </div>

      {#if extractStatus}
        <p class:ok={extractStatus.ok} class:erro={!extractStatus.ok}>{extractStatus.text}</p>
      {/if}
    </section>

    <section class="card">
      <h3>
        <Icon name="back" size={16} />
        Rotar páxinas
      </h3>
      <p class="muted hint">Rota un PDF enteiro ou só unha páxina, en múltiplos de 90°.</p>

      <div class="toolbar">
        <button class="btn" on:click={escollerParaRotar}>
          <Icon name="folder" size={14} />
          {rotateInput ? fileNameOf(rotateInput) : "Escoller PDF"}
        </button>
        {#if rotatePageCount !== null}
          <label class="check">
            <input type="checkbox" bind:checked={rotateAllPages} />
            Todas as páxinas
          </label>
          {#if !rotateAllPages}
            <input
              class="num-input"
              type="number"
              min="1"
              max={rotatePageCount}
              bind:value={rotatePage}
            />
            <span class="muted">/ {rotatePageCount}</span>
          {/if}
          <select bind:value={rotateDegrees}>
            <option value={90}>90°</option>
            <option value={180}>180°</option>
            <option value={270}>270°</option>
          </select>
          <button class="btn primary" on:click={rotar} disabled={rotating}>
            {rotating ? "Rotando…" : "Rotar"}
          </button>
        {/if}
      </div>

      {#if rotateStatus}
        <p class:ok={rotateStatus.ok} class:erro={!rotateStatus.ok}>{rotateStatus.text}</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .body {
    flex: 1;
    min-height: 0;
    width: 100%;
    overflow-y: auto;
    padding: 1.1rem 1.25rem 2rem;
    border: 2px dashed transparent;
    transition: border-color 0.1s ease;
  }

  .body.dragging {
    border-color: var(--accent-pdf);
  }

  .content {
    max-width: 760px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  h3 {
    margin: 0 0 0.3rem;
    font-size: 0.92rem;
    display: flex;
    align-items: center;
    gap: 0.45rem;
    color: var(--accent-pdf);
  }

  .hint {
    margin: 0 0 0.8rem;
    font-size: 0.85rem;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .empty {
    margin: 0.8rem 0 0;
    font-size: 0.85rem;
  }

  .lista {
    list-style: none;
    margin: 0.9rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .lista li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 0.45rem 0.7rem;
  }

  .nome {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.86rem;
  }

  .fila-acciones {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    flex-shrink: 0;
  }

  .mini {
    background: transparent;
    border: none;
    color: var(--text-faint);
    padding: 0.2rem 0.4rem;
    border-radius: var(--radius-sm);
    font-size: 0.9rem;
    line-height: 1;
  }

  .mini:hover:not(:disabled) {
    color: var(--text);
    background: var(--bg-hover);
  }

  .mini:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .check {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .paxinas {
    font-size: 0.82rem;
  }

  .spec-input {
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--text);
    padding: 0.4em 0.6em;
    font-size: 0.85rem;
    width: 9em;
  }

  select {
    background: var(--bg-elevated);
    color: var(--text);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    padding: 0.4em 0.6em;
  }

  p.ok,
  p.erro {
    margin: 0.7rem 0 0;
    font-size: 0.85rem;
  }

  p.ok {
    color: var(--accent-imaxe);
  }

  p.erro {
    color: var(--accent-video);
  }
</style>
