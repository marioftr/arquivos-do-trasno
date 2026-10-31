<script lang="ts">
  import { errText } from "../../errors";
  import { onMount } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, formatBytes, withExtension, TRASNO, fallo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { samePath } from "./shared";
  import Icon from "../Icon.svelte";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let selection: FileEntry[];
  export let category: "imaxe" | "video";

  const multi = selection.length > 1;
  const first = selection[0];
  const isImage = category === "imaxe";

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

  const form = createForm({
    mode: "quality" as "quality" | "bitrate" | "size",
    quality: 75,
    maxSide: null as number | null,
    crf: 28,
    maxHeight: 0,
    audioKbps: 128,
    videoKbps: 2000,
    sizeValue: 10,
    unit: "MB",
  });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;
  let outputDir: string | null = null;
  let estimating = false;
  let estimate: number | null = null;
  let results: { name: string; state: "agarda" | "feito" | "erro"; text?: string }[] = [];

  $: v = $form;
  $: targetBytes = Math.max(1, Math.round((v.sizeValue || 1) * (UNITS.find((u) => u.id === v.unit)?.mult ?? 1)));
  $: paramsKey = `${v.mode}|${v.quality}|${v.maxSide}|${v.crf}|${v.maxHeight}|${v.audioKbps}|${v.videoKbps}`;
  $: invalidate(paramsKey);
  $: if (isImage && !multi && v.mode === "quality") scheduleImageEstimate(paramsKey);

  let lastKey = "";
  function invalidate(key: string) {
    if (key !== lastKey) {
      lastKey = key;
      estimate = null;
    }
  }

  let timer: ReturnType<typeof setTimeout>;
  function scheduleImageEstimate(key: string) {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      try {
        const bytes = await engine.estimateImage(first.path, v.quality, v.maxSide || undefined);
        if (key === paramsKey) estimate = bytes;
      } catch {
        estimate = null;
      }
    }, 350);
  }

  async function estimateVideo() {
    estimating = true;
    try {
      estimate = await engine.estimateVideo(first.path, v.crf, v.maxHeight || undefined, v.audioKbps);
    } catch (e) {
      job.say(false, errText(e));
    } finally {
      estimating = false;
    }
  }

  const pct = (from: number, to: number) => (from > 0 ? `${to <= from ? "−" : "+"}${Math.abs(Math.round((1 - to / from) * 100))} %` : "");

  // Un vídeo con pouquísimo bitrate queda moi feo: avísase antes de comezar.
  $: tooSmall = !isImage && v.mode === "size" && !multi && !!first.duration && (targetBytes * 8 * 0.97) / first.duration / 1000 - v.audioKbps < 150;

  async function compressOne(f: FileEntry, out: string, overwrite: boolean): Promise<number> {
    if (isImage) {
      const r = await engine.compressImage(
        f.path,
        out,
        { quality: v.mode === "quality" ? v.quality : undefined, targetBytes: v.mode === "size" ? targetBytes : undefined, maxSide: v.maxSide || undefined },
        overwrite,
      );
      return r.bytes_written;
    }
    const done = await engine.compressVideo(
      f.path,
      out,
      { crf: v.crf, targetBytes: v.mode === "size" ? targetBytes : undefined, maxHeight: v.maxHeight || undefined, audioKbps: v.audioKbps, videoKbps: v.mode === "bitrate" ? v.videoKbps : undefined },
      overwrite,
    );
    return await engine.fileSize(done);
  }

  const outExt = () => (isImage ? "jpg" : "mp4");

  async function runSingle() {
    const stem = first.path.replace(/\.[^.\\/]+$/, "");
    const out = await save({ defaultPath: withExtension(`${stem}${tr("-comprimido")}`, outExt()) });
    if (!out) return;
    if (samePath(out, first.path)) {
      job.say(false, tr("Non vou sobrescribir o ficheiro orixinal. Escolle outro nome."));
      return;
    }
    await job.run(TRASNO.comprimindo, fallo("comprimindo"), async () => {
      const bytes = await compressOne(first, out, true);
      return tr("Listo! O trasno deixou o ficheiro en {to} ({pct}; antes eran {from}).", { from: formatBytes(first.size ?? 0), to: formatBytes(bytes), pct: pct(first.size ?? 0, bytes) });
    });
  }

  async function chooseDir() {
    const chosen = await open({ directory: true });
    if (typeof chosen === "string") outputDir = chosen;
  }

  async function runBatch() {
    const dir = outputDir ?? ((await open({ directory: true })) as string | null);
    if (!dir) return;
    outputDir = dir;
    results = selection.map((f) => ({ name: f.name, state: "agarda" }));
    job.busy.set(true);
    job.text.set(tr(TRASNO.comprimindo));
    job.clear();
    let before = 0;
    let after = 0;
    let cursor = 0;
    const worker = async () => {
      while (cursor < selection.length) {
        const i = cursor++;
        const f = selection[i];
        try {
          const dot = f.name.lastIndexOf(".");
          const base = dot > 0 ? f.name.slice(0, dot) : f.name;
          const out = await join(dir, `${base}${tr("-comprimido")}.${outExt()}`);
          if (samePath(out, f.path)) throw new Error(tr("Coincidiría co ficheiro orixinal."));
          const bytes = await compressOne(f, out, false);
          before += f.size ?? 0;
          after += bytes;
          results[i] = { name: f.name, state: "feito", text: `${formatBytes(f.size ?? 0)} → ${formatBytes(bytes)}` };
        } catch (e) {
          results[i] = { name: f.name, state: "erro", text: errText(e) };
        }
        results = results;
      }
    };
    // As imaxes van de tres en tres; o vídeo codifica de un en un (xa usa todos os núcleos).
    await Promise.all(Array.from({ length: Math.min(isImage ? 3 : 1, selection.length) }, worker));
    const bad = results.filter((r) => r.state === "erro").length;
    job.say(!bad, bad
      ? tr("Vaites! {bad} de {total} ficheiros escapáronlle ao trasno.", { bad, total: selection.length })
      : tr("Feito! O trasno comprimiu {n} ficheiros en {dir}: de {from} a {to}.", { n: selection.length, dir: fileNameOf(dir), from: formatBytes(before), to: formatBytes(after) }));
    job.busy.set(false);
  }
</script>

<div class="stack">
  <FormBar {form} />

  <Segmented
    options={isImage
      ? [
          { value: "quality", label: $t("Por calidade") },
          { value: "size", label: $t("Tamaño máximo") },
        ]
      : [
          { value: "quality", label: $t("Por calidade") },
          { value: "bitrate", label: $t("Por bitrate") },
          { value: "size", label: $t("Tamaño máximo") },
        ]}
    bind:value={$form.mode}
  />

  <div class="group">
    {#if v.mode === "size"}
      <div class="pair">
        <NumberField label={$t("Pesar menos de")} min={1} step="any" bind:value={$form.sizeValue} def={form.defaults.sizeValue} />
        <SelectField label={$t("Unidade")} bind:value={$form.unit} options={UNITS.map((u) => ({ value: u.id, label: u.id }))} def={form.defaults.unit} />
      </div>
      <p class="group-note">
        {multi ? $t("Cada ficheiro quedará por debaixo de {size}.", { size: `${v.sizeValue} ${v.unit}` }) : $t("O resultado quedará por debaixo de {size}.", { size: `${v.sizeValue} ${v.unit}` })}
      </p>
      {#if tooSmall}<p class="warn">{$t("Aviso: con este tamaño o vídeo quedará con moi pouca calidade.")}</p>{/if}
    {:else if v.mode === "bitrate" && !isImage}
      <Slider label={$t("Bitrate de vídeo")} unit="kbps" min={200} max={12000} step={100} bind:value={$form.videoKbps} def={form.defaults.videoKbps} hint={$t("Cantidade de datos por segundo da imaxe: máis alto = máis calidade e máis peso")} />
    {:else if isImage}
      <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={form.defaults.quality} />
    {:else}
      <Segmented options={LEVELS.map((l) => ({ value: l.crf, label: $t(l.label), hint: $t(l.hint) }))} bind:value={$form.crf} />
      <Slider label={$t("Calidade (CRF)")} min={18} max={40} bind:value={$form.crf} def={form.defaults.crf} hint={$t("Menos CRF = máis calidade e máis peso")} />
    {/if}

    {#if isImage}
      <NumberField label={$t("Lado máximo")} unit="px" min={1} placeholder={$t("orixinal")} bind:value={$form.maxSide} def={null} />
    {:else}
      <SelectField
        label={$t("Altura máxima")}
        bind:value={$form.maxHeight}
        options={HEIGHTS.map((h) => ({ value: h, label: h ? `${h}p` : $t("orixinal") }))}
        def={form.defaults.maxHeight}
      />
      <Slider label={$t("Bitrate de audio")} unit="kbps" min={48} max={256} step={8} bind:value={$form.audioKbps} def={form.defaults.audioKbps} />
    {/if}
    <p class="group-note">{isImage ? $t("As imaxes gárdanse como JPG.") : $t("Os vídeos gárdanse como MP4 (H.264).")}</p>
  </div>

  {#if !multi}
    <div class="estimate">
      {#if v.mode === "size"}
        <span class="est-main">≤ {v.sizeValue} {v.unit}</span>
        <span class="muted">{$t("orixinal {orig}, {pct}", { orig: formatBytes(first.size ?? 0), pct: pct(first.size ?? 0, targetBytes) })}</span>
      {:else if v.mode === "bitrate" && !isImage && first.duration}
        <span class="est-main">≈ {formatBytes(((v.videoKbps + v.audioKbps) * 1000 * first.duration) / 8)}</span>
        <span class="muted">{$t("orixinal {orig}, {pct}", { orig: formatBytes(first.size ?? 0), pct: pct(first.size ?? 0, ((v.videoKbps + v.audioKbps) * 1000 * first.duration) / 8) })}</span>
      {:else if estimate !== null}
        <span class="est-main">≈ {formatBytes(estimate)}</span>
        <span class="muted">{$t("orixinal {orig}, {pct}", { orig: formatBytes(first.size ?? 0), pct: pct(first.size ?? 0, estimate) })}</span>
        {#if !isImage}<span class="muted tiny">{$t("Estimación por mostra: pode variar un pouco.")}</span>{/if}
      {:else if !isImage && v.mode === "quality"}
        <button class="btn" on:click={estimateVideo} disabled={estimating || $busy}>{estimating ? $t("Estimando…") : $t("Estimar o tamaño")}</button>
      {:else}
        <span class="muted">{$t("Estimando…")}</span>
      {/if}
    </div>
  {/if}

  {#if multi}
    <button class="btn" on:click={chooseDir} disabled={$busy} title={$t("Gardar todo neste cartafol")}>
      <Icon name="folder" size={14} />
      {outputDir ? fileNameOf(outputDir) : $t("Cartafol de saída")}
    </button>
    <button class="btn primary block" on:click={runBatch} disabled={$busy}>
      {isImage ? $t("Comprimir {n} imaxes", { n: selection.length }) : $t("Comprimir {n} vídeos", { n: selection.length })}
    </button>
  {:else}
    <button class="btn primary block" on:click={runSingle} disabled={$busy}>
      {isImage ? $t("Comprimir imaxe") : $t("Comprimir vídeo")}
    </button>
  {/if}

  {#if results.length}
    <ul class="results">
      {#each results as r}
        <li class={r.state}>
          <span class="rname" title={r.text}>{r.name}</span>
          <span class="rstate">{r.state === "feito" ? r.text : r.state === "erro" ? $t("erro") : "…"}</span>
        </li>
      {/each}
    </ul>
  {/if}

  <JobStatus {job} />
</div>

<style>
  .warn { margin: 0; font-size: 0.8rem; color: var(--accent-pdf); }
  .tiny { font-size: 0.76rem; }
  .estimate { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.2rem 0.7rem; min-height: var(--control-h); align-content: center; }
  .est-main { font-size: 1.05rem; font-weight: 600; }
</style>
