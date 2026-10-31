<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, formatBytes, withExtension, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { samePath } from "./shared";

  export let selection: FileEntry[];
  export let category: "imaxe" | "video";

  $: multi = selection.length > 1;
  $: first = selection[0];
  $: isImage = category === "imaxe";

  const LEVELS = [
    { label: "Calidade alta", crf: 23, hint: "Case idéntico ao orixinal, pesa menos" },
    { label: "Equilibrada", crf: 28, hint: "O mellor equilibrio entre peso e calidade" },
    { label: "Forte", crf: 33, hint: "Moi pequeno; nótanse perdas de nitidez" },
  ];
  const HEIGHTS = [0, 1080, 720, 480, 360];
  const UNITS = [
    { id: "KB", mult: 1024 },
    { id: "MB", mult: 1024 ** 2 },
    { id: "GB", mult: 1024 ** 3 },
  ];

  let mode: "quality" | "size" = "quality";
  let quality = 75;
  let maxSide: number | null = null;
  let crf = 28;
  let maxHeight = 0;
  let audioKbps = 128;
  let sizeValue = 10;
  let unit = "MB";
  let outputDir: string | null = null;

  let busy = false;
  let estimating = false;
  let estimate: number | null = null;
  let message: { ok: boolean; text: string } | null = null;
  let results: { name: string; state: "agarda" | "feito" | "erro"; text?: string }[] = [];

  $: targetBytes = Math.max(1, Math.round(sizeValue * (UNITS.find((u) => u.id === unit)?.mult ?? 1)));
  $: selKey = selection.map((f) => f.path).join("|");
  $: paramsKey = `${selKey}|${mode}|${quality}|${maxSide}|${crf}|${maxHeight}|${audioKbps}`;
  $: invalidate(paramsKey);
  $: if (isImage && !multi && mode === "quality") scheduleImageEstimate(paramsKey);

  let lastKey = "";
  function invalidate(key: string) {
    if (key === lastKey) return;
    lastKey = key;
    estimate = null;
  }
  $: if (selKey) {
    message = null;
    results = [];
  }

  let timer: ReturnType<typeof setTimeout>;
  function scheduleImageEstimate(key: string) {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      if (key !== paramsKey) return;
      try {
        const bytes = await engine.estimateImage(first.path, quality, maxSide || undefined);
        if (key === paramsKey) estimate = bytes;
      } catch {
        estimate = null;
      }
    }, 350);
  }

  async function estimateVideo() {
    estimating = true;
    try {
      estimate = await engine.estimateVideo(first.path, crf, maxHeight || undefined, audioKbps);
    } catch (e) {
      message = { ok: false, text: String(e) };
    } finally {
      estimating = false;
    }
  }

  const pct = (from: number, to: number) => (from > 0 ? `${to <= from ? "−" : "+"}${Math.abs(Math.round((1 - to / from) * 100))} %` : "");

  // Un vídeo con pouquísimo bitrate queda moi feo: avísase antes de comezar.
  $: tooSmall = !isImage && mode === "size" && !multi && !!first?.duration && (targetBytes * 8 * 0.97) / first.duration / 1000 - audioKbps < 150;

  async function compressOne(f: FileEntry, out: string, overwrite: boolean): Promise<number> {
    if (isImage) {
      const r = await engine.compressImage(
        f.path,
        out,
        { quality: mode === "quality" ? quality : undefined, targetBytes: mode === "size" ? targetBytes : undefined, maxSide: maxSide || undefined },
        overwrite,
      );
      return r.bytes_written;
    }
    const done = await engine.compressVideo(
      f.path,
      out,
      { crf, targetBytes: mode === "size" ? targetBytes : undefined, maxHeight: maxHeight || undefined, audioKbps },
      overwrite,
    );
    return (await engine.fileSize(done)) as number;
  }

  const outExt = () => (isImage ? "jpg" : "mp4");

  async function runSingle() {
    const f = first;
    const stem = f.path.replace(/\.[^.\\/]+$/, "");
    const out = await save({ defaultPath: withExtension(`${stem}${tr("-comprimido")}`, outExt()) });
    if (!out) return;
    if (samePath(out, f.path)) {
      message = { ok: false, text: tr("Non se sobrescribe o ficheiro orixinal. Escolle outro nome.") };
      return;
    }
    busy = true;
    message = null;
    try {
      const bytes = await compressOne(f, out, true);
      message = {
        ok: true,
        text: tr("¡Feito! {from} → {to} ({pct}).", { from: formatBytes(f.size ?? 0), to: formatBytes(bytes), pct: pct(f.size ?? 0, bytes) }),
      };
    } catch (e) {
      message = { ok: false, text: `${tr("¡Vaites! O trasno tropezou durante a compresión.")}\n${e}` };
    } finally {
      busy = false;
    }
  }

  async function chooseDir() {
    const chosen = await open({ directory: true });
    if (typeof chosen === "string") outputDir = chosen;
  }

  async function runBatch() {
    const dir = outputDir ?? ((await open({ directory: true })) as string | null);
    if (!dir) return;
    outputDir = dir;
    const list = [...selection];
    results = list.map((f) => ({ name: f.name, state: "agarda" }));
    busy = true;
    message = null;
    let before = 0;
    let after = 0;
    let cursor = 0;
    const worker = async () => {
      while (cursor < list.length) {
        const i = cursor++;
        const f = list[i];
        try {
          const dot = f.name.lastIndexOf(".");
          const base = dot > 0 ? f.name.slice(0, dot) : f.name;
          const out = await join(dir, `${base}${tr("-comprimido")}.${outExt()}`);
          if (samePath(out, f.path)) throw new Error(tr("coincidiría co orixinal"));
          const bytes = await compressOne(f, out, false);
          before += f.size ?? 0;
          after += bytes;
          results[i] = { name: f.name, state: "feito", text: `${formatBytes(f.size ?? 0)} → ${formatBytes(bytes)}` };
        } catch (e) {
          results[i] = { name: f.name, state: "erro", text: String(e) };
        }
        results = results;
      }
    };
    // As imaxes van de tres en tres; o vídeo codifica de un en un (xa usa todos os núcleos).
    await Promise.all(Array.from({ length: Math.min(isImage ? 3 : 1, list.length) }, worker));
    const bad = results.filter((r) => r.state === "erro").length;
    message = bad
      ? { ok: false, text: tr("¡Vaites! {bad} de {total} ficheiros escapáronlle ao trasno.", { bad, total: list.length }) }
      : { ok: true, text: tr("¡Feito! {n} ficheiros comprimidos en {dir}: {from} → {to}.", { n: list.length, dir: fileNameOf(dir), from: formatBytes(before), to: formatBytes(after) }) };
    busy = false;
  }
</script>

<section>
  <h3>{$t("Comprimir")}</h3>

  <div class="seg">
    <button class:on={mode === "quality"} on:click={() => (mode = "quality")}>{$t("Por calidade")}</button>
    <button class:on={mode === "size"} on:click={() => (mode = "size")}>{$t("Tamaño máximo")}</button>
  </div>

  {#if mode === "size"}
    <div class="row">
      <span>{$t("Pesar menos de")}</span>
      <input class="num-input wide" type="number" min="1" step="any" bind:value={sizeValue} />
      <select bind:value={unit}>{#each UNITS as u}<option value={u.id}>{u.id}</option>{/each}</select>
    </div>
    <p class="muted tiny">
      {multi ? $t("Cada ficheiro quedará por debaixo de {size}.", { size: `${sizeValue} ${unit}` }) : $t("O resultado quedará por debaixo de {size}.", { size: `${sizeValue} ${unit}` })}
    </p>
    {#if tooSmall}<p class="warn">{$t("Aviso: con este tamaño o vídeo quedará con moi pouca calidade.")}</p>{/if}
  {:else if isImage}
    <div class="row">
      <span>{$t("Calidade")}</span>
      <input type="range" min="10" max="100" bind:value={quality} />
      <input class="num-input" type="number" min="10" max="100" bind:value={quality} />
    </div>
  {:else}
    <div class="levels">
      {#each LEVELS as l}
        <button class="lvl" class:on={crf === l.crf} title={$t(l.hint)} on:click={() => (crf = l.crf)}>{$t(l.label)}</button>
      {/each}
    </div>
    <div class="row">
      <span>{$t("Calidade (CRF)")}</span>
      <input type="range" min="18" max="40" bind:value={crf} />
      <input class="num-input" type="number" min="18" max="40" bind:value={crf} />
    </div>
    <p class="muted tiny">{$t("Menos CRF = máis calidade e máis peso")}</p>
  {/if}

  {#if isImage}
    <div class="row">
      <span>{$t("Lado máx.")}</span>
      <input class="num-input wide" type="number" min="1" placeholder={$t("orixinal")} bind:value={maxSide} />
      <span class="muted">px</span>
    </div>
  {:else}
    <div class="row">
      <span>{$t("Altura máxima")}</span>
      <select bind:value={maxHeight}>
        {#each HEIGHTS as h}<option value={h}>{h ? `${h}p` : $t("orixinal")}</option>{/each}
      </select>
    </div>
    <div class="row">
      <span>{$t("Audio (kbps)")}</span>
      <input type="range" min="48" max="256" step="8" bind:value={audioKbps} />
      <input class="num-input" type="number" min="48" max="256" bind:value={audioKbps} />
    </div>
  {/if}

  <p class="muted tiny">{isImage ? $t("As imaxes gárdanse como JPG.") : $t("Os vídeos gárdanse como MP4 (H.264).")}</p>

  {#if !multi}
    {#if mode === "size"}
      <p class="est">{$t("Resultado aproximado: {size} (orixinal {orig}, {pct})", { size: `≤ ${sizeValue} ${unit}`, orig: formatBytes(first?.size ?? 0), pct: pct(first?.size ?? 0, targetBytes) })}</p>
    {:else if estimate !== null}
      <p class="est">{$t("Resultado aproximado: {size} (orixinal {orig}, {pct})", { size: `≈ ${formatBytes(estimate)}`, orig: formatBytes(first?.size ?? 0), pct: pct(first?.size ?? 0, estimate) })}</p>
      {#if !isImage}<p class="muted tiny">{$t("Estimación por mostra: pode variar un pouco.")}</p>{/if}
    {:else if !isImage}
      <button class="btn" on:click={estimateVideo} disabled={estimating || busy}>{estimating ? $t("Estimando…") : $t("Estimar")}</button>
    {/if}
  {/if}

  {#if multi}
    <div class="row">
      <button class="btn" on:click={chooseDir} disabled={busy} title={$t("Gardar todo neste cartafol")}>
        {outputDir ? fileNameOf(outputDir) : $t("Cartafol de saída")}
      </button>
    </div>
    <button class="btn primary full" on:click={runBatch} disabled={busy}>
      {busy ? $t(TRASNO.traballando) : isImage ? $t("Comprimir {n} imaxes", { n: selection.length }) : $t("Comprimir {n} vídeos", { n: selection.length })}
    </button>
  {:else}
    <button class="btn primary full" on:click={runSingle} disabled={busy}>
      {busy ? $t("O trasno está comprimindo… ¡paciencia!") : isImage ? $t("Comprimir imaxe") : $t("Comprimir vídeo")}
    </button>
  {/if}

  {#if results.length}
    <ul class="results">
      {#each results as r}
        <li class={r.state}>
          <span class="rname" title={r.text}>{r.name}</span>
          <span class="rstate">{r.state === "feito" ? r.text : r.state === "erro" ? $t("erro") : ""}</span>
        </li>
      {/each}
    </ul>
  {/if}

  {#if message}<p class="msg" class:ok={message.ok} class:erro={!message.ok}>{message.text}</p>{/if}
</section>

<style>
  section { padding: 1rem 0; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 0.6rem; }
  h3 { margin: 0; font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-muted); font-weight: 600; }
  .seg { display: flex; border: 1px solid var(--border-strong); border-radius: var(--radius-sm); overflow: hidden; }
  .seg button { flex: 1; background: transparent; border: none; color: var(--text-muted); padding: 0.4em 0; font-size: 0.82rem; }
  .seg button.on { background: var(--bg-hover); color: var(--text); }
  .levels { display: flex; gap: 0.3rem; }
  .lvl { flex: 1; background: transparent; border: 1px solid var(--border-strong); color: var(--text-muted); border-radius: var(--radius-sm); padding: 0.3em 0; font-size: 0.78rem; }
  .lvl.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .row { display: flex; align-items: center; gap: 0.5rem; font-size: 0.86rem; }
  .row > span:first-child { min-width: 5.2rem; color: var(--text-muted); }
  .row input[type="range"] { flex: 1; min-width: 0; }
  .row select { flex: 1; }
  select { background: var(--bg-elevated); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0.4em 0.6em; font-size: 0.85rem; }
  .num-input.wide { width: 5.5em; }
  .tiny { margin: 0; font-size: 0.78rem; }
  .warn { margin: 0; font-size: 0.8rem; color: var(--accent-pdf); }
  .est { margin: 0; font-size: 0.86rem; color: var(--text); }
  .full { width: 100%; justify-content: center; }
  .results { list-style: none; margin: 0.3rem 0 0; padding: 0; font-size: 0.82rem; }
  .results li { display: flex; justify-content: space-between; gap: 0.6rem; padding: 0.35rem 0; border-bottom: 1px solid var(--border); }
  .rname { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rstate { white-space: nowrap; color: var(--text-muted); }
  .results li.feito .rstate { color: var(--accent-imaxe); }
  .results li.erro .rstate { color: var(--accent-video); }
  .msg { margin: 0; font-size: 0.85rem; white-space: pre-wrap; word-break: break-word; }
  .msg.ok { color: var(--accent-imaxe); }
  .msg.erro { color: var(--accent-video); }
</style>
