<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import { t, tr } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import Seg from "./Seg.svelte";
  import Info from "./Info.svelte";
  import { thumbnail } from "../lib/image";
  import { accepts, canShare, download, formatBytes, save, share, UserError, type Output } from "../lib/files";
  import { downloadedModels } from "../lib/models";
  import type { Tool, Values } from "../lib/tools";

  /** La pantalla de calquera ferramenta: elixir ficheiros, axustes, o botón verde e os resultados. */
  export let tool: Tool;

  interface Item {
    id: number;
    file: File;
    /** Miniatura (só imaxes). */
    url: string;
  }
  interface Done extends Output {
    url: string;
  }

  const KEY = `arquivos-do-trasno.web.${tool.id}`;
  const saved = (() => {
    try {
      return JSON.parse(localStorage.getItem(KEY) ?? "{}") as Values;
    } catch {
      return {};
    }
  })();
  // Só se lembran as opcións que se elixen (formato, calidade…), non o texto escrito (intervalos de páxinas).
  const keep = tool.options.filter((o) => o.type === "seg").map((o) => o.id);
  let values: Values = { ...tool.defaults, ...Object.fromEntries(Object.entries(saved).filter(([k]) => keep.includes(k))) };
  $: try {
    localStorage.setItem(KEY, JSON.stringify(Object.fromEntries(keep.map((k) => [k, values[k]]))));
  } catch {
    // sen almacenamento: vale ata pechar
  }

  const isImage = tool.accept.startsWith("image");
  let items: Item[] = [];
  let nextId = 1;
  let results: Done[] = [];
  let failed: { name: string; why: string }[] = [];
  let skipped = 0;
  let error = "";
  let busy = false;
  let step = 0;
  let frac: number | null = null;
  let label = "";
  const ctx = { progress: (f: number | null, l?: string) => ((frac = f), l !== undefined && (label = l)) };
  let over = false;
  let input: HTMLInputElement;
  let resultsEl: HTMLElement;

  // Modelos de IA xa gardados no navegador: as opcións que os usan amósano.
  let have = new Set<string>();
  const usesModels = tool.options.some((o) => o.type === "seg" && o.choices().some((c) => c.model));
  const checkModels = () => usesModels && void downloadedModels().then((s) => (have = s));
  checkModels();

  const min = tool.min ?? 1;
  $: ready = items.length >= min;
  $: visible = tool.options.filter((o) => !o.when || o.when(values));

  function forgetResults() {
    for (const r of results) if (r.url) URL.revokeObjectURL(r.url);
    results = [];
    failed = [];
    error = "";
  }
  function add(list: Iterable<File> | null | undefined) {
    const all = [...(list ?? [])];
    const ok = all.filter((f) => accepts(tool.accept, f));
    skipped = all.length - ok.length;
    if (!ok.length) return;
    forgetResults();
    if (!tool.multiple) clear();
    const fresh = (tool.multiple ? ok : ok.slice(0, 1)).map((file) => ({ id: nextId++, file, url: "" }));
    items = [...items, ...fresh];
    if (isImage) for (const it of fresh) void thumbnail(it.file).then((url) => (items.some((x) => x.id === it.id) ? (items = items.map((x) => (x.id === it.id ? { ...x, url } : x))) : url && URL.revokeObjectURL(url)));
  }
  function clear() {
    for (const i of items) if (i.url) URL.revokeObjectURL(i.url);
    items = [];
  }
  function remove(id: number) {
    const it = items.find((i) => i.id === id);
    if (it?.url) URL.revokeObjectURL(it.url);
    items = items.filter((i) => i.id !== id);
    forgetResults();
  }
  function move(id: number, d: number) {
    const i = items.findIndex((x) => x.id === id);
    const j = i + d;
    if (i < 0 || j < 0 || j >= items.length) return;
    const next = [...items];
    [next[i], next[j]] = [next[j], next[i]];
    items = next;
    forgetResults();
  }
  function picked() {
    add(input.files);
    input.value = ""; // a mesma foto pódese volver elixir
  }
  const onDrop = (e: DragEvent) => {
    over = false;
    add(e.dataTransfer?.files);
  };
  // Pegar unha imaxe copiada (escritorio).
  const onPaste = (e: ClipboardEvent) => e.clipboardData?.files.length && add(e.clipboardData.files);

  const nap = () => new Promise<void>((r) => setTimeout(r, 0));
  async function run() {
    if (!ready || busy) return;
    busy = true;
    forgetResults();
    step = 0;
    frac = null;
    label = "";
    await tick();
    await nap();
    const out: Output[] = [];
    try {
      const files = items.map((i) => i.file);
      if (tool.all) out.push(...(await tool.all(files, values, ctx)));
      else if (tool.each) {
        const taken = new Set<string>();
        for (const [i, f] of files.entries()) {
          step = i + 1;
          await nap();
          try {
            out.push(await tool.each(f, values, taken, ctx));
          } catch (e) {
            failed = [...failed, { name: f.name, why: e instanceof UserError ? e.message : tr("Non puiden abrir esta imaxe. O navegador non admite ese formato.") }];
          }
        }
      }
    } catch (e) {
      error = e instanceof UserError ? e.message : tr("Algo fallou ao procesar. Proba con outro ficheiro.");
      if (!(e instanceof UserError)) console.error(e);
    }
    results = out.map((o) => ({ ...o, url: o.blob.type.startsWith("image/") ? URL.createObjectURL(o.blob) : "" }));
    busy = false;
    checkModels();
    if (results.length || failed.length || error) {
      await tick();
      resultsEl?.scrollIntoView({ behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth", block: "start" });
      resultsEl?.focus({ preventScroll: true });
    }
  }

  let copied = "";
  async function copy(r: Output) {
    try {
      await navigator.clipboard.writeText(r.text ?? "");
      copied = r.name;
      setTimeout(() => (copied = ""), 1500);
    } catch {
      // o texto tamén se pode seleccionar a man
    }
  }
  let shareErr = "";
  /** Todos de vez: no móbil pola folla de compartir (permite gardalos en Fotos), no escritorio descargándoos. */
  async function saveAll() {
    shareErr = "";
    if (!canShare(results)) return results.forEach((r) => download(r));
    try {
      await share(results);
    } catch {
      shareErr = tr("Non se puido abrir a folla de compartir. Usa Gardar.");
    }
  }

  // Ver o resultado a pantalla completa.
  let viewer: HTMLDialogElement;
  let viewing: Done | null = null;
  function view(r: Done) {
    viewing = r;
    viewer.showModal();
  }
  function again() {
    forgetResults();
    clear();
    skipped = 0;
  }
  const total = (l: Output[]) => l.reduce((a, o) => a + o.blob.size, 0);

  onDestroy(() => {
    clear();
    forgetResults();
  });
</script>

<svelte:window on:paste={onPaste} />

<div class="wb">
  <section class="files" aria-label={$t("Ficheiros")} class:over on:dragover|preventDefault={() => (over = true)} on:dragleave={() => (over = false)} on:drop|preventDefault={onDrop}>
    <input bind:this={input} type="file" class="sr-only" accept={tool.accept} multiple={tool.multiple} on:change={picked} tabindex="-1" aria-hidden="true" />
    {#if !items.length}
      <button type="button" class="drop" on:click={() => input.click()}>
        <Icon name="load" size={30} />
        <span class="big">{tool.multiple ? $t("Elixir ficheiros") : $t("Elixir ficheiro")}</span>
        <span class="drag">{$t("Tamén podes arrastralos aquí ou pegalos.")}</span>
      </button>
    {/if}
    {#if skipped}<p class="warn" role="status">{skipped === 1 ? $t("Saltouse un ficheiro que non vale para esta ferramenta.") : $t("Saltáronse {n} ficheiros que non valen para esta ferramenta.", { n: skipped })}</p>{/if}

    {#if items.length}
      <ul class="list">
        {#each items as it, i (it.id)}
          <li>
            {#if it.url}<img src={it.url} alt="" class="thumb" decoding="async" />{:else}<span class="thumb ico"><Icon name="document" size={22} /></span>{/if}
            <span class="meta"><span class="name">{it.file.name}</span><span class="sub">{formatBytes(it.file.size)}</span></span>
            {#if tool.ordered && items.length > 1}
              <button type="button" class="ib" disabled={i === 0} on:click={() => move(it.id, -1)} aria-label={$t("Subir {name}", { name: it.file.name })}><Icon name="up" size={20} /></button>
              <button type="button" class="ib" disabled={i === items.length - 1} on:click={() => move(it.id, 1)} aria-label={$t("Baixar {name}", { name: it.file.name })}><Icon name="down" size={20} /></button>
            {/if}
            <button type="button" class="ib" on:click={() => remove(it.id)} aria-label={$t("Quitar {name}", { name: it.file.name })}><Icon name="close" size={20} /></button>
          </li>
        {/each}
      </ul>
      {#if items.length < min}<p class="warn" role="status">{$t("Engade polo menos {n} ficheiros.", { n: min })}</p>{/if}
      <button type="button" class="btn add" on:click={() => input.click()}>
        <Icon name="plus" size={20} />
        <span>{tool.multiple ? $t("Engadir máis") : $t("Cambiar ficheiro")}</span>
      </button>
    {/if}
  </section>

  {#if visible.length}
    <section class="opts" aria-label={$t("Axustes")}>
      {#each visible as o (o.id)}
        {#if o.type === "seg"}
          <Seg label={o.label} name={`${tool.id}-${o.id}`} choices={o.choices()} bind:value={values[o.id]} info={o.info ?? ""} ready={have} />
        {:else}
          <div class="field">
            <label for={`${tool.id}-${o.id}`}>{o.label}{#if o.info}<Info text={o.info} />{/if}</label>
            <input id={`${tool.id}-${o.id}`} type="text" inputmode="text" autocomplete="off" autocapitalize="off" spellcheck="false" placeholder={o.placeholder} bind:value={values[o.id]} />
          </div>
        {/if}
      {/each}
    </section>
  {/if}

  <section class="results" bind:this={resultsEl} tabindex="-1" aria-label={$t("Resultado")} hidden={!results.length && !failed.length && !error}>
    {#if error}<p class="warn" role="alert">{error}</p>{/if}
    {#if failed.length}
      <div class="warn" role="alert">
        {#each failed as f}<p>{tr("«{name}»: {why}", { name: f.name, why: f.why })}</p>{/each}
      </div>
    {/if}
    {#if results.length}
      <h2>{$t("Feito")} <span class="count">{results.length === 1 ? $t("1 ficheiro") : $t("{n} ficheiros", { n: results.length })} · {formatBytes(total(results))}</span></h2>
      <ul class="list">
        {#each results as r (r.name)}
          <li class="res">
            {#if r.url}<button type="button" class="thumb-btn" on:click={() => view(r)} aria-label={$t("Ver {name} a pantalla completa", { name: r.name })}><img src={r.url} alt="" class="thumb" decoding="async" /></button>{:else}<span class="thumb ico"><Icon name="document" size={22} /></span>{/if}
            <span class="meta"><span class="name">{r.name}</span>{#if r.note}<span class="sub">{r.note}</span>{/if}</span>
            {#if r.text}<textarea class="text" readonly rows="6" aria-label={$t("Texto")} value={r.text}></textarea>{/if}
            {#if r.text}<button type="button" class="btn save" on:click={() => copy(r)}><Icon name={copied === r.name ? "check" : "copy"} size={20} /><span>{copied === r.name ? $t("Copiado") : $t("Copiar")}</span></button>{/if}
            <button type="button" class="btn save" on:click={() => save(r).catch(() => (shareErr = tr("Non se puido gardar. Proba de novo.")))} aria-label={$t("Gardar {name}", { name: r.name })}><Icon name="download" size={20} /><span>{$t("Gardar")}</span></button>
          </li>
        {/each}
      </ul>
      <div class="after">
        {#if results.length > 1}
          <button type="button" class="btn" on:click={saveAll}><Icon name="download" size={20} /><span>{$t("Gardar todo")}</span></button>
        {/if}
        <button type="button" class="btn ghost" on:click={again}><Icon name="reset" size={20} /><span>{$t("Empezar de novo")}</span></button>
      </div>
      {#if shareErr}<p class="warn" role="alert">{shareErr}</p>{/if}
    {/if}
  </section>

  {#if busy && (label || frac !== null)}
    <div class="prog" role="status">
      <span>{label}</span>
      <progress max="1" value={frac ?? undefined} aria-label={label || $t("Procesando…")}></progress>
    </div>
  {/if}

  <div class="bar">
    <button type="button" class="btn primary go" disabled={!ready || busy} on:click={run} data-testid="go">
      {busy ? tr("Procesando {i} de {n}…", { i: Math.max(step, 1), n: items.length }) : tool.action(items.length || 1)}
    </button>
  </div>
</div>

<dialog bind:this={viewer} class="viewer" aria-label={viewing?.name ?? ""} on:close={() => (viewing = null)} on:click={() => viewer.close()}>
  {#if viewing}<img src={viewing.url} alt={viewing.name} />{/if}
  <button type="button" class="x" on:click={() => viewer.close()} aria-label={$t("Pechar")}><Icon name="close" size={24} /></button>
</dialog>

<div class="sr-only" role="status" aria-live="polite">
  {#if busy}{$t("Procesando…")}{:else if results.length}{$t("Feito")}: {results.length === 1 ? $t("1 ficheiro") : $t("{n} ficheiros", { n: results.length })}{/if}
</div>

<style>
  .wb { display: grid; grid-template-columns: minmax(0, 1fr); gap: 1.25rem; align-items: start; }
  section { min-width: 0; }
  .files { display: flex; flex-direction: column; gap: 0.75rem; border-radius: var(--radius-lg); }
  .files.over { outline: 2px dashed var(--accent); outline-offset: 4px; }
  /* Sen ficheiros, toda a zona é o botón: un só marco, grande e fácil de tocar. */
  .drop { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.35rem; min-height: 9rem; padding: 1.25rem 1rem; text-align: center; color: var(--text); background: var(--bg-card); border: 2px dashed var(--border-strong); border-radius: var(--radius-lg); -webkit-tap-highlight-color: transparent; }
  .drop:hover { background: var(--bg-hover); }
  .drop:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  .drop :global(svg) { color: var(--accent); }
  .big { font-size: var(--fs-lg); font-weight: var(--fw-strong); }
  .drag { color: var(--text-muted); font-size: var(--fs-sm); font-weight: var(--fw-regular); }
  @media (hover: none) { .drag { display: none; } }
  .add { width: 100%; }
  .opts { display: flex; flex-direction: column; gap: 1.1rem; }
  .field { display: flex; flex-direction: column; gap: 0.4rem; }
  .field label { display: flex; align-items: center; gap: 0.4rem; font-size: var(--fs-sm); font-weight: var(--fw-strong); color: var(--text-muted); }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.5rem; }
  .list li { display: flex; align-items: center; gap: 0.7rem; padding: 0.5rem 0.6rem; background: var(--bg); border: 1px solid var(--border); border-radius: var(--radius-md); min-width: 0; }
  .thumb { flex: none; width: 3rem; height: 3rem; object-fit: cover; border-radius: var(--radius-sm); background: var(--bg-hover); }
  .thumb.ico { display: grid; place-items: center; color: var(--text-muted); }
  .meta { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: var(--fw-strong); }
  .sub { font-size: var(--fs-sm); color: var(--text-muted); overflow-wrap: anywhere; }
  .ib { flex: none; display: grid; place-items: center; width: 44px; height: 44px; padding: 0; background: none; border: 0; border-radius: var(--radius-md); color: var(--text-muted); }
  .ib:hover:not(:disabled) { background: var(--bg-hover); color: var(--text); }
  .ib:disabled { opacity: 0.35; cursor: default; }
  .ib:focus-visible { outline: 3px solid var(--accent); outline-offset: -3px; }
  .warn { margin: 0; color: var(--accent-pdf); font-size: var(--fs-sm); }
  .warn p { margin: 0 0 0.25rem; }
  .results { display: flex; flex-direction: column; gap: 0.75rem; scroll-margin-top: 4.5rem; outline: none; }
  .results[hidden] { display: none; }
  h2 { margin: 0; font-size: var(--fs-lg); display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.2rem 0.7rem; }
  .count { font-size: var(--fs-sm); font-weight: var(--fw-regular); color: var(--text-muted); }
  .save { flex: none; padding: 0 1rem; }
  .after { display: flex; flex-wrap: wrap; gap: 0.5rem; }
  .after .btn { flex: 1 1 12rem; }
  .prog { display: flex; flex-direction: column; gap: 0.4rem; font-size: var(--fs-sm); color: var(--text-muted); }
  progress { width: 100%; height: 0.6rem; accent-color: var(--accent); }
  .res { flex-wrap: wrap; }
  .thumb-btn { flex: none; padding: 0; background: none; border: 0; border-radius: var(--radius-sm); cursor: zoom-in; }
  .thumb-btn:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  .res .thumb { width: 4rem; height: 4rem; display: block; }
  /* Pantalla completa: a imaxe enteira sobre fondo escuro; un toque en calquera parte pecha. */
  .viewer { position: fixed; inset: 0; width: 100%; height: 100%; max-width: none; max-height: none; margin: 0; padding: 0; border: 0; background: #1c1c1c conic-gradient(#2a2a2a 25%, #1c1c1c 0 50%, #2a2a2a 0 75%, #1c1c1c 0) 0 0 / 20px 20px; display: none; cursor: zoom-out; }
  .viewer[open] { display: block; }
  .viewer::backdrop { background: transparent; }
  /* Ocupa todo o espazo (as imaxes pequenas amplíanse) sen deformarse; o fondo de cadros deixa ver as transparencias. */
  .viewer img { display: block; width: 100%; height: 100%; object-fit: contain; }
  .viewer .x { position: absolute; top: max(0.75rem, env(safe-area-inset-top)); right: 0.75rem; display: grid; place-items: center; width: 48px; height: 48px; padding: 0; color: #fff; background: rgb(0 0 0 / 0.6); border: 1px solid rgb(255 255 255 / 0.5); border-radius: 50%; }
  .viewer .x:focus-visible { outline: 3px solid #fff; outline-offset: 2px; }
  .text { order: 9; flex: 1 0 100%; margin: 0; padding: 0.7rem 0.8rem; resize: vertical; width: 100%; color: var(--text); background: var(--bg); border: 1px solid var(--border); border-radius: var(--radius-md); font: inherit; font-size: var(--fs-sm); user-select: text; }

  /* Móbil: o botón verde queda sempre á man, pegado abaixo. */
  .bar { position: sticky; bottom: 0; z-index: 10; margin: 0 -1rem; padding: 0.75rem 1rem calc(0.75rem + env(safe-area-inset-bottom)); background: var(--bg); border-top: 1px solid var(--border); }
  .go { width: 100%; }

  @media (min-width: 860px) {
    .wb { grid-template-columns: minmax(0, 1fr) 21rem; grid-template-areas: "files opts" "files bar" "results results"; gap: 1.5rem; }
    .files { grid-area: files; }
    .opts { grid-area: opts; }
    .bar { grid-area: bar; position: static; margin: 0; padding: 0; background: none; border: 0; }
    .results { grid-area: results; }
    /* Sen axustes, o botón queda na primeira fila. */
    .wb:not(:has(.opts)) { grid-template-areas: "files bar" "results results"; }
  }
</style>
