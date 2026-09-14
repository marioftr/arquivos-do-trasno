<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { join } from "@tauri-apps/api/path";
  import StudioHeader from "../lib/components/StudioHeader.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import * as engine from "../lib/engine";
  import { extensionOf, targetsForCategory, withExtension, fileNameOf, formatBytes, CATEGORY_LABELS } from "../lib/formats";
  import type { Category } from "../lib/engine";
  import { pendingFiles } from "../lib/router";

  interface Item {
    path: string;
    category: Category;
    targetExt: string;
    status: "pendente" | "convertendo" | "feito" | "erro";
    outputPath?: string;
    error?: string;
    sizeBytes?: number;
    outputBytes?: number;
    pageCount?: number;
    showOptions: boolean;
    resizeW: number | null;
    resizeH: number | null;
  }

  let items: Item[] = [];
  let ffmpegOk: boolean | null = null;
  let quality = 85;
  let outputDir: string | null = null;
  let dragging = false;
  let unlistenDrop: (() => void) | null = null;

  engine.ffmpegAvailable().then((v) => (ffmpegOk = v));

  async function addPaths(paths: string[]) {
    for (const path of paths) {
      if (items.some((i) => i.path === path)) continue;
      const category = await engine.detectCategory(path);
      const targets = targetsForCategory(category);
      const currentExt = extensionOf(path);
      const targetExt = targets.find((t) => t !== currentExt) ?? targets[0] ?? "";
      const item: Item = {
        path,
        category,
        targetExt,
        status: "pendente",
        showOptions: false,
        resizeW: null,
        resizeH: null,
      };
      items = [...items, item];

      engine.fileSize(path).then((s) => updateItem(path, { sizeBytes: s })).catch(() => {});
      if (category === "documento") {
        engine.pdfPageCount(path).then((n) => updateItem(path, { pageCount: n })).catch(() => {});
      }
    }
  }

  function updateItem(path: string, patch: Partial<Item>) {
    items = items.map((i) => (i.path === path ? { ...i, ...patch } : i));
  }

  onMount(async () => {
    const initial = $pendingFiles;
    if (initial.length) {
      addPaths(initial);
      pendingFiles.set([]);
    }

    const webview = getCurrentWebview();
    unlistenDrop = await webview.onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        dragging = false;
        addPaths(event.payload.paths);
      } else if (event.payload.type === "enter" || event.payload.type === "over") {
        dragging = true;
      } else {
        dragging = false;
      }
    });
  });

  onDestroy(() => {
    unlistenDrop?.();
  });

  async function escollerCartafol() {
    const chosen = await open({ directory: true });
    if (typeof chosen === "string") outputDir = chosen;
  }

  function quitar(index: number) {
    items = items.filter((_, i) => i !== index);
  }

  function precisaFfmpeg(category: Category): boolean {
    return category === "audio" || category === "video";
  }

  async function resolveOutputPath(item: Item): Promise<string | null> {
    const targetName = withExtension(fileNameOf(item.path), item.targetExt);
    if (outputDir) {
      return join(outputDir, targetName);
    }
    const defaultOutput = withExtension(item.path, item.targetExt);
    const chosen = await save({ defaultPath: defaultOutput });
    return chosen ?? null;
  }

  async function converterUn(index: number) {
    const item = items[index];
    if (!item.targetExt) return;
    if (precisaFfmpeg(item.category) && ffmpegOk === false) {
      items[index] = { ...item, status: "erro", error: "Non se atopou ffmpeg neste equipo." };
      items = [...items];
      return;
    }

    items[index] = { ...item, status: "convertendo", error: undefined };
    items = [...items];

    try {
      const outputPath = await resolveOutputPath(item);
      if (!outputPath) {
        items[index] = { ...item, status: "pendente" };
        items = [...items];
        return;
      }

      const resize: [number, number] | undefined =
        item.resizeW && item.resizeH ? [item.resizeW, item.resizeH] : undefined;

      const result = await engine.convertFile(item.path, outputPath, {
        overwrite: true,
        quality: item.category === "imaxe" ? quality : undefined,
        resize,
      });
      items[index] = { ...item, status: "feito", outputPath: result.output_path, outputBytes: result.bytes_written };
    } catch (err) {
      items[index] = { ...item, status: "erro", error: String(err) };
    }
    items = [...items];
  }

  async function converterTodo() {
    for (let i = 0; i < items.length; i++) {
      if (items[i].status === "pendente") {
        await converterUn(i);
      }
    }
  }

  function toggleOpcions(index: number) {
    items[index] = { ...items[index], showOptions: !items[index].showOptions };
    items = [...items];
  }
</script>

<StudioHeader title="Conversor universal" icon="convert" accent="var(--accent-conversor)" />

<div class="body scrollbar-thin" class:dragging>
  <div class="content">
    {#if ffmpegOk === false}
      <div class="aviso card">
        <Icon name="warning" size={17} />
        <span>
          Non se atopou <code>ffmpeg</code> neste equipo. As imaxes conviértense igualmente (motor propio en
          Rust), pero para audio e vídeo hai que instalar ffmpeg e volver abrir a aplicación.
        </span>
      </div>
    {/if}

    <div class="toolbar">
      <button class="btn primary" on:click={() => open({ multiple: true }).then((s) => s && addPaths(Array.isArray(s) ? s : [s]))}>
        <Icon name="plus" size={15} />
        Engadir ficheiros
      </button>
      <button class="btn" on:click={converterTodo} disabled={items.length === 0}>Converter todo</button>

      <button class="btn folder" on:click={escollerCartafol} title="Gardar todo neste cartafol en vez de preguntar cada vez">
        <Icon name="folder" size={15} />
        {outputDir ? fileNameOf(outputDir) : "Escoller cartafol de saída"}
      </button>
      {#if outputDir}
        <button class="btn quitar" on:click={() => (outputDir = null)} title="Deixar de usar este cartafol" aria-label="Quitar cartafol">
          <Icon name="close" size={14} />
        </button>
      {/if}

      <label class="calidade">
        Calidade imaxe
        <input type="range" min="10" max="100" bind:value={quality} />
        <input class="num-input" type="number" min="10" max="100" bind:value={quality} />
      </label>
    </div>

    {#if items.length === 0}
      <div class="card baleiro muted" class:dragging>
        <Icon name="folder" size={22} />
        <p>
          Arrastra aquí imaxes, audio, vídeo, PDF ou modelos 3D — o trasno encárgase de trocarlles a
          forma. Tamén podes usar "Engadir ficheiros".
        </p>
      </div>
    {:else}
      <div class="lista">
        {#each items as item, i (item.path)}
          <div class="card fila-wrap">
            <div class="fila">
              <div class="info">
                <div class="nome" title={item.path}>{fileNameOf(item.path)}</div>
                <div class="muted detalles">
                  {CATEGORY_LABELS[item.category]} · {extensionOf(item.path) || "?"}
                  {#if item.sizeBytes !== undefined} · {formatBytes(item.sizeBytes)}{/if}
                  {#if item.pageCount !== undefined} · {item.pageCount} páx.{/if}
                </div>
              </div>

              <Icon name="arrow-right" size={15} strokeWidth={1.6} />

              <select bind:value={item.targetExt} disabled={item.status === "convertendo"}>
                {#each targetsForCategory(item.category) as ext}
                  <option value={ext}>{ext.toUpperCase()}</option>
                {/each}
              </select>

              {#if item.category === "imaxe"}
                <button class="btn opcions" on:click={() => toggleOpcions(i)} title="Redimensionar">
                  <Icon name="chevron-down" size={14} />
                </button>
              {/if}

              <div class="estado">
                {#if item.status === "pendente"}
                  <button class="btn" on:click={() => converterUn(i)}>Converter</button>
                {:else if item.status === "convertendo"}
                  <span class="muted">Convertendo…</span>
                {:else if item.status === "feito"}
                  <span class="ok">
                    <Icon name="check" size={15} /> Feito{#if item.outputBytes !== undefined} · {formatBytes(item.outputBytes)}{/if}
                  </span>
                {:else if item.status === "erro"}
                  <span class="erro" title={item.error}><Icon name="warning" size={15} /> Erro</span>
                {/if}
              </div>

              <button class="btn quitar" on:click={() => quitar(i)} title="Quitar da lista" aria-label="Quitar da lista">
                <Icon name="close" size={15} />
              </button>
            </div>

            {#if item.showOptions}
              <div class="opcions-panel muted">
                Redimensionar (opcional):
                <input class="num-input" type="number" min="1" placeholder="ancho" bind:value={item.resizeW} />
                ×
                <input class="num-input" type="number" min="1" placeholder="alto" bind:value={item.resizeH} />
                px — deixa baleiro para manter o tamaño orixinal.
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
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
    border-color: var(--accent-conversor);
  }

  .content {
    /* A lista de conversión é traballo, non prosa: que aproveite todo o
       ancho da xanela en vez de quedar apertada nun lado. */
    max-width: 1400px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .aviso {
    border-color: var(--accent-pdf);
    color: var(--accent-pdf);
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
  }

  .aviso span {
    color: var(--text);
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .folder {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .calidade {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-left: auto;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .baleiro {
    padding: 2rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
  }

  .baleiro p {
    margin: 0;
    max-width: 46ch;
  }

  .lista {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .fila-wrap {
    padding: 0;
    overflow: hidden;
  }

  .fila {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    color: var(--text-faint);
  }

  .info {
    flex: 1;
    min-width: 0;
    color: var(--text);
  }

  .nome {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .detalles {
    font-size: 0.8rem;
  }

  select {
    background: var(--bg-elevated);
    color: var(--text);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    padding: 0.4em 0.6em;
  }

  .estado {
    min-width: 8rem;
    text-align: center;
  }

  .ok,
  .erro {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.86rem;
    white-space: nowrap;
  }

  .ok {
    color: var(--accent-imaxe);
  }

  .erro {
    color: var(--accent-video);
  }

  .quitar {
    background: transparent;
    border: none;
    color: var(--text-faint);
    padding: 0.3rem;
  }

  .quitar:hover {
    color: var(--text);
  }

  .opcions-panel {
    padding: 0 1rem 0.75rem 1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.82rem;
    flex-wrap: wrap;
  }

  .opcions-panel input {
    width: 5.5em;
  }
</style>
