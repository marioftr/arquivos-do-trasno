<script lang="ts">
  import { onMount } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import StudioHeader from "../lib/components/StudioHeader.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import * as engine from "../lib/engine";
  import { extensionOf, targetsForCategory, withExtension, CATEGORY_LABELS } from "../lib/formats";
  import type { Category } from "../lib/engine";
  import { pendingFiles } from "../lib/router";

  interface Item {
    path: string;
    category: Category;
    targetExt: string;
    status: "pendente" | "convertendo" | "feito" | "erro";
    outputPath?: string;
    error?: string;
  }

  let items: Item[] = [];
  let ffmpegOk: boolean | null = null;
  let quality = 85;

  engine.ffmpegAvailable().then((v) => (ffmpegOk = v));

  async function addPaths(paths: string[]) {
    for (const path of paths) {
      const category = await engine.detectCategory(path);
      const targets = targetsForCategory(category);
      const currentExt = extensionOf(path);
      const targetExt = targets.find((t) => t !== currentExt) ?? targets[0] ?? "";
      items = [...items, { path, category, targetExt, status: "pendente" }];
    }
  }

  onMount(() => {
    const initial = $pendingFiles;
    if (initial.length) {
      addPaths(initial);
      pendingFiles.set([]);
    }
  });

  async function engadirFicheiros() {
    const selected = await open({ multiple: true });
    if (!selected) return;
    await addPaths(Array.isArray(selected) ? selected : [selected]);
  }

  function quitar(index: number) {
    items = items.filter((_, i) => i !== index);
  }

  function precisaFfmpeg(category: Category): boolean {
    return category === "audio" || category === "video";
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

    const defaultOutput = withExtension(item.path, item.targetExt);
    try {
      const chosen = await save({ defaultPath: defaultOutput });
      if (!chosen) {
        items[index] = { ...item, status: "pendente" };
        items = [...items];
        return;
      }

      const result = await engine.convertFile(item.path, chosen, {
        overwrite: true,
        quality: item.category === "imaxe" ? quality : undefined,
      });
      items[index] = { ...item, status: "feito", outputPath: result.output_path };
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
</script>

<StudioHeader title="Conversor universal" icon="convert" accent="var(--accent-conversor)" />

<div class="body scrollbar-thin">
  {#if ffmpegOk === false}
    <div class="aviso card">
      <Icon name="warning" size={17} />
      <span>
        Non se atopou <code>ffmpeg</code> neste equipo. As imaxes conviértense igualmente (motor propio en Rust),
        pero para audio e vídeo hai que instalar ffmpeg e volver abrir a aplicación.
      </span>
    </div>
  {/if}

  <div class="toolbar">
    <button class="btn primary" on:click={engadirFicheiros}>
      <Icon name="plus" size={15} />
      Engadir ficheiros
    </button>
    <button class="btn" on:click={converterTodo} disabled={items.length === 0}>Converter todo</button>
    <label class="calidade">
      Calidade imaxe
      <input type="range" min="10" max="100" bind:value={quality} />
      <span>{quality}</span>
    </label>
  </div>

  {#if items.length === 0}
    <div class="card baleiro muted">
      Engade unha imaxe, un audio, un vídeo, un PDF ou un modelo 3D, e escolle a que formato o queres converter.
    </div>
  {:else}
    <div class="lista">
      {#each items as item, i (item.path)}
        <div class="card fila">
          <div class="info">
            <div class="nome" title={item.path}>{item.path.split(/[\\/]/).pop()}</div>
            <div class="muted categoria">{CATEGORY_LABELS[item.category]} · {extensionOf(item.path) || "?"}</div>
          </div>

          <Icon name="arrow-right" size={15} strokeWidth={1.6} />

          <select bind:value={item.targetExt} disabled={item.status === "convertendo"}>
            {#each targetsForCategory(item.category) as ext}
              <option value={ext}>{ext.toUpperCase()}</option>
            {/each}
          </select>

          <div class="estado">
            {#if item.status === "pendente"}
              <button class="btn" on:click={() => converterUn(i)}>Converter</button>
            {:else if item.status === "convertendo"}
              <span class="muted">Convertendo…</span>
            {:else if item.status === "feito"}
              <span class="ok"><Icon name="check" size={15} /> Feito</span>
            {:else if item.status === "erro"}
              <span class="erro" title={item.error}><Icon name="warning" size={15} /> Erro</span>
            {/if}
          </div>

          <button class="btn quitar" on:click={() => quitar(i)} title="Quitar da lista" aria-label="Quitar da lista">
            <Icon name="close" size={15} />
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 1.1rem 1.25rem 2rem;
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
    gap: 0.75rem;
    flex-wrap: wrap;
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
  }

  .lista {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
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

  .categoria {
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
    min-width: 6.5rem;
    text-align: center;
  }

  .ok,
  .erro {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.86rem;
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
</style>
