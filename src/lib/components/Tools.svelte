<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../engine";
  import type { Category } from "../engine";
  import type { FileEntry } from "../files";
  import Icon from "./Icon.svelte";
  import { convertModel } from "../model3d";
  import { t, tr } from "../i18n";
  import InfoSection from "./tools/InfoSection.svelte";
  import ImageEditSection from "./tools/ImageEditSection.svelte";
  import MediaEditSection from "./tools/MediaEditSection.svelte";
  import CompressSection from "./tools/CompressSection.svelte";
  import {
    targetsForCategory,
    withExtension,
    fileNameOf,
    formatBytes,
    TRASNO,
    CATEGORY_LABELS,
  } from "../formats";

  export let selection: FileEntry[] = [];

  const MAX_CONCURRENT = 3;
  const PRESETS = [
    { label: "WhatsApp", value: 55, hint: "Máis lixeiro, ideal para enviar rápido" },
    { label: "Web", value: 75, hint: "Equilibrio entre peso e nitidez" },
    { label: "Arquivo", value: 95, hint: "Case sen perdas, para gardar" },
  ];

  let targetExt = "";
  let quality = 85;
  let bitrate = 0;
  let resizeW: number | null = null;
  let resizeH: number | null = null;
  let keepRatio = true;
  let maxSide: number | null = null;
  let trimStart = 0;
  let trimEnd: number | null = null;
  let pageSpec = "";
  let rotateAll = true;
  let rotatePage = 1;
  let rotateDegrees = 90;
  let outputDir: string | null = null;

  let busy = false;
  let busyText = "";
  let message: { ok: boolean; text: string } | null = null;
  let results: { name: string; state: "agarda" | "convertendo" | "feito" | "erro"; text?: string }[] = [];

  $: multi = selection.length > 1;
  $: first = selection[0] as FileEntry | undefined;
  $: category = (selection.length && selection.every((f) => f.category === first!.category)
    ? first!.category
    : null) as Category | null;
  $: isPdf = category === "documento" && selection.every((f) => f.ext === "pdf");
  $: isMedia = category === "audio" || category === "video";
  $: targets = first && category ? targetsForCategory(category, first.ext) : [];
  $: selectionKey = selection.map((f) => f.path).join("|");
  $: resetFor(selectionKey, category);

  let lastKey = "";
  function resetFor(key: string, cat: Category | null) {
    if (key === lastKey) return;
    lastKey = key;
    message = null;
    results = [];
    resizeW = resizeH = null;
    maxSide = null;
    trimStart = 0;
    trimEnd = null;
    pageSpec = "";
    rotatePage = 1;
    if (first && cat) {
      const tg = targetsForCategory(cat, first.ext);
      targetExt = multi && cat === "imaxe" && tg.includes(first.ext) ? first.ext : (tg.find((x) => x !== first!.ext) ?? tg[0] ?? "");
      bitrate = cat === "audio" ? 192 : cat === "video" ? 4000 : 0;
    }
  }

  function samePath(a: string, b: string) {
    return a.replace(/\\/g, "/").toLowerCase() === b.replace(/\\/g, "/").toLowerCase();
  }

  function onResizeW(raw: string) {
    resizeW = raw === "" ? null : Number(raw);
    if (resizeW && keepRatio && first?.width && first.height) {
      resizeH = Math.max(1, Math.round((resizeW * first.height) / first.width));
    }
  }
  function onResizeH(raw: string) {
    resizeH = raw === "" ? null : Number(raw);
    if (resizeH && keepRatio && first?.width && first.height) {
      resizeW = Math.max(1, Math.round((resizeH * first.width) / first.height));
    }
  }

  /** Executa un traballo; se falla, o trasno avisa co texto `fail` e despois vai o erro. */
  async function guard(working: string, fail: string, job: () => Promise<string | void>) {
    busy = true;
    busyText = tr(working);
    message = null;
    try {
      const done = await job();
      if (done) message = { ok: true, text: done };
    } catch (e) {
      message = { ok: false, text: `${tr(fail)}\n${e}` };
    } finally {
      busy = false;
    }
  }

  const refuseOriginal = () => ({ ok: false, text: tr("Non se sobrescribe o ficheiro orixinal. Escolle outro nome.") });

  // En lote o tamaño dáse como lado máximo; cada imaxe escálase mantendo a súa proporción.
  function resizeFor(f: FileEntry, single: boolean): [number, number] | undefined {
    if (single) return resizeW && resizeH ? [resizeW, resizeH] : undefined;
    if (!maxSide || !f.width || !f.height) return undefined;
    const k = Math.min(1, maxSide / Math.max(f.width, f.height));
    if (k >= 1) return undefined;
    return [Math.max(1, Math.round(f.width * k)), Math.max(1, Math.round(f.height * k))];
  }

  function optionsFor(f: FileEntry, single: boolean): engine.ConvertOptions {
    return {
      quality: f.category === "imaxe" ? quality : undefined,
      resize: f.category === "imaxe" ? resizeFor(f, single) : undefined,
      bitrateKbps: (f.category === "audio" || f.category === "video") && bitrate ? bitrate : undefined,
    };
  }

  const ENGINE_3D = ["obj", "stl", "ply"];

  /** 3D: o motor Rust para OBJ/STL/PLY entre si; o resto, polo visor (three.js). */
  async function convertOne(f: FileEntry, out: string, overwrite: boolean): Promise<{ output_path: string; bytes_written: number }> {
    if (f.category === "modelo3d" && !(ENGINE_3D.includes(f.ext) && ENGINE_3D.includes(targetExt))) {
      const bytes = await convertModel(f.path, f.ext, out, targetExt, overwrite);
      return { output_path: out, bytes_written: bytes };
    }
    return engine.convertFile(f.path, out, { ...optionsFor(f, !multi), overwrite });
  }

  async function convertSingle() {
    const f = first!;
    const out = await save({ defaultPath: withExtension(f.path, targetExt) });
    if (!out) return;
    if (samePath(out, f.path)) {
      message = refuseOriginal();
      return;
    }
    await guard(TRASNO.convertendo, TRASNO.tropezouConvertendo, async () => {
      const r = await convertOne(f, out, true);
      return `${tr(TRASNO.listo)} ${fileNameOf(r.output_path)} · ${formatBytes(r.bytes_written)}`;
    });
  }

  async function chooseDir() {
    const chosen = await open({ directory: true });
    if (typeof chosen === "string") outputDir = chosen;
  }

  async function convertBatch() {
    const dir = outputDir ?? ((await open({ directory: true })) as string | null);
    if (!dir) return;
    outputDir = dir;
    const list = [...selection];
    results = list.map((f) => ({ name: f.name, state: "agarda" }));
    busy = true;
    busyText = tr(TRASNO.convertendo);
    message = null;
    let cursor = 0;
    const worker = async () => {
      while (cursor < list.length) {
        const i = cursor++;
        const f = list[i];
        results[i] = { ...results[i], state: "convertendo" };
        results = results;
        try {
          const dot = f.name.lastIndexOf(".");
          const base = dot > 0 ? f.name.slice(0, dot) : f.name;
          // Mesmo formato (p. ex. comprimir un JPG): outro nome, nunca o orixinal.
          const name = targetExt === f.ext ? `${base}-trasno.${targetExt}` : `${base}.${targetExt}`;
          const out = await join(dir, name);
          if (samePath(out, f.path)) throw new Error(tr("coincidiría co orixinal"));
          const r = await convertOne(f, out, false);
          const before = f.size ? ` (${formatBytes(f.size)} → ${formatBytes(r.bytes_written)})` : "";
          results[i] = { name: f.name, state: "feito", text: before };
        } catch (e) {
          results[i] = { name: f.name, state: "erro", text: String(e) };
        }
        results = results;
      }
    };
    await Promise.all(Array.from({ length: Math.min(MAX_CONCURRENT, list.length) }, worker));
    const bad = results.filter((r) => r.state === "erro").length;
    message = bad
      ? { ok: false, text: tr("¡Vaites! {bad} de {total} ficheiros escapáronlle ao trasno.", { bad, total: list.length }) }
      : { ok: true, text: tr("¡Feito! {n} ficheiros gardados en {dir}.", { n: list.length, dir: fileNameOf(dir) }) };
    busy = false;
  }

  async function merge() {
    const f = first!;
    const asPdf = isPdf || category === "imaxe";
    const out = await save({
      defaultPath: await join(f.path.replace(/[\\/][^\\/]*$/, ""), `${tr("unido")}.${asPdf ? "pdf" : f.ext}`),
    });
    if (!out) return;
    const inputs = selection.map((s) => s.path);
    if (inputs.some((p) => samePath(p, out))) {
      message = { ok: false, text: tr("Non se sobrescribe un dos ficheiros de orixe. Escolle outro nome.") };
      return;
    }
    await guard(TRASNO.unindo, TRASNO.tropezouUnindo, async () => {
      const done = isPdf
        ? await engine.pdfMerge(inputs, out, true)
        : category === "imaxe"
          ? await engine.imagesToPdf(inputs, out, true)
          : await engine.concatMedia(inputs, out, true);
      return `${tr(TRASNO.listo)} ${fileNameOf(done)}`;
    });
  }

  function parsePages(spec: string): number[] {
    const pages: number[] = [];
    for (const part of spec.split(",")) {
      const p = part.trim();
      if (!p) continue;
      const m = p.match(/^(\d+)-(\d+)$/);
      if (m) {
        const a = Number(m[1]);
        const b = Number(m[2]);
        for (let n = a; a <= b ? n <= b : n >= b; n += a <= b ? 1 : -1) pages.push(n);
      } else if (/^\d+$/.test(p)) pages.push(Number(p));
      else throw new Error(tr("«{p}» non é unha páxina válida", { p }));
    }
    if (!pages.length) throw new Error(tr("indica algunha páxina"));
    return pages;
  }

  async function extractPages() {
    const f = first!;
    let pages: number[];
    try {
      pages = parsePages(pageSpec);
    } catch (e) {
      message = { ok: false, text: String(e) };
      return;
    }
    const out = await save({ defaultPath: withExtension(f.path.replace(/\.pdf$/i, `-${tr("paxinas")}`), "pdf") });
    if (!out) return;
    if (samePath(out, f.path)) {
      message = refuseOriginal();
      return;
    }
    await guard(TRASNO.traballando, TRASNO.tropezouEditando, async () => `${tr(TRASNO.listo)} ${fileNameOf(await engine.pdfExtractPages(f.path, pages, out, true))}`);
  }

  async function rotatePdf() {
    const f = first!;
    const out = await save({ defaultPath: withExtension(f.path.replace(/\.pdf$/i, `-${tr("rotado")}`), "pdf") });
    if (!out) return;
    if (samePath(out, f.path)) {
      message = refuseOriginal();
      return;
    }
    await guard(TRASNO.traballando, TRASNO.tropezouEditando, async () =>
      `${tr(TRASNO.listo)} ${fileNameOf(await engine.pdfRotate(f.path, rotateAll ? null : rotatePage, rotateDegrees, out, true))}`,
    );
  }

  async function trim() {
    const f = first!;
    const out = await save({ defaultPath: withExtension(f.path.replace(/\.[^.\\/]+$/, `-${tr("corte")}`), f.ext) });
    if (!out) return;
    if (samePath(out, f.path)) {
      message = refuseOriginal();
      return;
    }
    await guard(TRASNO.traballando, TRASNO.tropezouEditando, async () =>
      `${tr(TRASNO.listo)} ${fileNameOf(await engine.trimMedia(f.path, out, trimStart || 0, trimEnd || null, true))}`,
    );
  }
</script>

<div class="tools scrollbar-thin">
  {#if !selection.length}
    <p class="muted hint">{$t("Abre un ficheiro para ver as ferramentas que o trasno ten para el.")}</p>
  {:else if !category}
    <p class="muted hint">
      {$t("Hai ficheiros de tipos distintos marcados. Marca só do mesmo tipo (por exemplo, varios PDF ou varias imaxes) para traballar con eles xuntos.")}
    </p>
  {:else}
    <header>
      <h2>{multi ? $t("{n} ficheiros", { n: selection.length }) : first?.name}</h2>
      <p class="muted sub">
        {$t(CATEGORY_LABELS[category])}{#if !multi && first?.size !== undefined} · {formatBytes(first.size)}{/if}{#if !multi && first?.pages} · {$t("{n} páx.", { n: first.pages })}{/if}
      </p>
    </header>

    {#if targets.length}
      <section>
        <h3>{multi ? $t("Converter todo") : $t("Converter")}</h3>
        <label class="row">
          <span>{$t("Formato")}</span>
          <select bind:value={targetExt} disabled={busy}>
            {#each targets as ext}<option value={ext}>{ext.toUpperCase()}</option>{/each}
          </select>
        </label>

        {#if category === "imaxe"}
          <div class="row">
            <span>{$t("Calidade")}</span>
            <input type="range" min="10" max="100" bind:value={quality} />
            <input class="num-input" type="number" min="10" max="100" bind:value={quality} />
          </div>
          <div class="presets">
            {#each PRESETS as p}
              <button class="preset" class:on={quality === p.value} title={$t(p.hint)} on:click={() => (quality = p.value)}>{$t(p.label)}</button>
            {/each}
          </div>
          {#if multi}
            <div class="row">
              <span>{$t("Lado máx.")}</span>
              <input class="num-input wide" type="number" min="1" placeholder={$t("orixinal")} bind:value={maxSide} />
              <span class="muted">px</span>
            </div>
          {:else}
            <div class="row">
              <span>{$t("Tamaño")}</span>
              <input class="num-input wide" type="number" min="1" placeholder={$t("ancho")} value={resizeW ?? ""} on:input={(e) => onResizeW(e.currentTarget.value)} />
              <span class="x">×</span>
              <input class="num-input wide" type="number" min="1" placeholder={$t("alto")} value={resizeH ?? ""} on:input={(e) => onResizeH(e.currentTarget.value)} />
            </div>
            <label class="check"><input type="checkbox" bind:checked={keepRatio} /> {$t("Manter proporción")}</label>
          {/if}
        {:else if isMedia}
          <div class="row">
            <span>{$t("Bitrate")}</span>
            <input type="range" min={category === "audio" ? 32 : 500} max={category === "audio" ? 320 : 12000} step={category === "audio" ? 8 : 100} bind:value={bitrate} />
            <input class="num-input wide" type="number" min="1" bind:value={bitrate} />
          </div>
          <p class="muted tiny">{$t("kbps — máis alto, máis calidade e máis tamaño.")}</p>
        {/if}

        {#if multi}
          <div class="row">
            <button class="btn" on:click={chooseDir} disabled={busy} title={$t("Gardar todo neste cartafol")}>
              <Icon name="folder" size={14} />
              {outputDir ? fileNameOf(outputDir) : $t("Cartafol de saída")}
            </button>
          </div>
          <button class="btn primary full" on:click={convertBatch} disabled={busy || !targetExt}>
            {$t("Converter {n} ficheiros", { n: selection.length })}
          </button>
        {:else}
          <button class="btn primary full" on:click={convertSingle} disabled={busy || !targetExt}>{$t("Converter a {ext}", { ext: targetExt.toUpperCase() })}</button>
        {/if}
      </section>
    {/if}

    {#if (category === "imaxe" || category === "video") && first}
      <CompressSection {selection} {category} />
    {/if}

    {#if multi && (isPdf || isMedia || category === "imaxe")}
      <section>
        <h3>{isPdf ? $t("Unir PDF") : category === "imaxe" ? $t("Imaxes a PDF") : category === "video" ? $t("Xuntar vídeos") : $t("Xuntar audios")}</h3>
        <p class="muted tiny">{$t("Vaise facer na orde da lista; usa as frechas para reordenar.")}</p>
        <button class="btn primary full" on:click={merge} disabled={busy}>
          <Icon name="layers" size={14} />
          {isPdf ? $t("Unir nun só PDF") : category === "imaxe" ? $t("Facer un PDF cunha imaxe por páxina") : $t("Xuntar nun só ficheiro")}
        </button>
      </section>
    {/if}

    {#if !multi && category === "imaxe" && first}
      <ImageEditSection file={first} />
    {/if}

    {#if !multi && isMedia && first}
      <MediaEditSection file={first} />
    {/if}

    {#if !multi && isPdf}
      <section>
        <h3>{$t("Extraer páxinas")}</h3>
        <div class="row">
          <input class="text-input" type="text" placeholder={$t("ex: 1,3,5-7")} bind:value={pageSpec} />
        </div>
        <button class="btn full" on:click={extractPages} disabled={busy || !pageSpec}>{$t("Extraer a un novo PDF")}</button>
      </section>
      <section>
        <h3>{$t("Rotar")}</h3>
        <label class="check"><input type="checkbox" bind:checked={rotateAll} /> {$t("Todas as páxinas")}</label>
        {#if !rotateAll}
          <div class="row">
            <span>{$t("Páxina")}</span>
            <input class="num-input" type="number" min="1" max={first?.pages} bind:value={rotatePage} />
            {#if first?.pages}<span class="muted">/ {first.pages}</span>{/if}
          </div>
        {/if}
        <div class="row">
          <span>{$t("Ángulo")}</span>
          <select bind:value={rotateDegrees}>
            <option value={90}>90°</option>
            <option value={180}>180°</option>
            <option value={270}>270°</option>
          </select>
        </div>
        <button class="btn full" on:click={rotatePdf} disabled={busy}>{$t("Rotar a un novo PDF")}</button>
      </section>
    {/if}

    {#if !multi && isMedia}
      <section>
        <h3>{$t("Cortar")}</h3>
        <div class="row">
          <span>{$t("Dende")}</span>
          {#if first?.duration}
            <input type="range" min="0" max={first.duration} step="0.1" bind:value={trimStart} />
          {/if}
          <input class="num-input wide" type="number" min="0" step="0.1" bind:value={trimStart} />
          <span class="muted">s</span>
        </div>
        <div class="row">
          <span>{$t("Ata")}</span>
          {#if first?.duration}
            <input type="range" min="0" max={first.duration} step="0.1" value={trimEnd ?? first.duration} on:input={(e) => (trimEnd = Number(e.currentTarget.value))} />
          {/if}
          <input class="num-input wide" type="number" min="0" step="0.1" placeholder={$t("fin")} bind:value={trimEnd} />
          <span class="muted">s</span>
        </div>
        {#if first?.duration}<p class="muted tiny">{$t("Duración total: {s} s", { s: first.duration.toFixed(1) })}</p>{/if}
        <button class="btn full" on:click={trim} disabled={busy}>{$t("Gardar o corte")}</button>
      </section>
    {/if}

    {#if !multi && first}
      <InfoSection file={first} />
    {/if}

    {#if busy && !results.length}<p class="status muted">{busyText}</p>{/if}

    {#if results.length}
      <ul class="results">
        {#each results as r}
          <li class={r.state}>
            <span class="rname" title={r.text}>{r.name}</span>
            <span class="rstate">
              {#if r.state === "feito"}<Icon name="check" size={13} />{r.text}
              {:else if r.state === "erro"}<Icon name="warning" size={13} /> {$t("erro")}
              {:else if r.state === "convertendo"}…
              {/if}
            </span>
          </li>
        {/each}
      </ul>
    {/if}

    {#if message}
      <p class="msg" class:ok={message.ok} class:erro={!message.ok}>{message.text}</p>
    {/if}
  {/if}
</div>

<style>
  .tools { padding: 1rem 1.1rem 1.5rem; overflow-y: auto; height: 100%; }
  header h2 { margin: 0; font-size: 0.95rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sub { margin: 0.2rem 0 0; font-size: 0.8rem; }
  section { padding: 1rem 0; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 0.6rem; }
  section:last-of-type { border-bottom: none; }
  h3 { margin: 0; font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-muted); font-weight: 600; }
  .row { display: flex; align-items: center; gap: 0.5rem; font-size: 0.86rem; }
  .row > span:first-child { min-width: 4.2rem; color: var(--text-muted); }
  .row select { flex: 1; }
  .row input[type="range"] { flex: 1; min-width: 0; }
  .x { color: var(--text-faint); min-width: 0 !important; }
  .num-input.wide { width: 5.5em; }
  .text-input, select {
    background: var(--bg-elevated); color: var(--text); border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm); padding: 0.4em 0.6em; font-size: 0.85rem;
  }
  .text-input { flex: 1; }
  .check { display: flex; align-items: center; gap: 0.4rem; font-size: 0.85rem; color: var(--text-muted); }
  .presets { display: flex; gap: 0.3rem; }
  .preset { flex: 1; background: transparent; border: 1px solid var(--border-strong); color: var(--text-muted); border-radius: var(--radius-sm); padding: 0.3em 0; font-size: 0.78rem; }
  .preset:hover { color: var(--text); }
  .preset.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .full { width: 100%; justify-content: center; }
  .tiny { margin: 0; font-size: 0.78rem; }
  .hint { padding: 0.5rem 0; font-size: 0.88rem; }
  .status { font-size: 0.85rem; }
  .results { list-style: none; margin: 0.5rem 0 0; padding: 0; font-size: 0.82rem; }
  .results li { display: flex; justify-content: space-between; gap: 0.6rem; padding: 0.35rem 0; border-bottom: 1px solid var(--border); }
  .rname { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rstate { display: inline-flex; align-items: center; gap: 0.3rem; white-space: nowrap; color: var(--text-muted); }
  .results li.feito .rstate { color: var(--accent-imaxe); }
  .results li.erro .rstate { color: var(--accent-video); }
  .msg { margin: 0.8rem 0 0; font-size: 0.85rem; white-space: pre-wrap; word-break: break-word; }
  .msg.ok { color: var(--accent-imaxe); }
  .msg.erro { color: var(--accent-video); }
</style>
