<script lang="ts">
  import { errText } from "../../errors";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, formatBytes, withExtension, TRASNO, fallo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { savedProfiles } from "../../settings";
  import { toast } from "../../toasts";
  import { samePath, askPath, askDir } from "./shared";
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

  // Perfís de saída para vídeo: cada un fixa a calidade e os axustes de compatibilidade.
  // `deinterlace`: 0 non, 1 só se está entrelazado, 2 sempre (cadencia dobre, como facía o DV orixinal).
  const PROFILES: Record<string, { label: string; hint: string; v: { crf: number; maxHeight: number; audioKbps: number; deinterlace: number; compat: boolean } }> = {
    universal: { label: "Universal optimizado", hint: "MP4 H.264 que abre en móbiles, televisións e navegadores. Case idéntico ao orixinal e bastante máis lixeiro.", v: { crf: 20, maxHeight: 0, audioKbps: 160, deinterlace: 1, compat: true } },
    minidv: { label: "Mini DV a MP4 (sen perda visible)", hint: "Para cintas dixitalizadas en AVI/DV: desentrelaza, calidade moi alta e MP4 que se ve en todas partes. Pesa moito menos que o AVI.", v: { crf: 18, maxHeight: 0, audioKbps: 192, deinterlace: 2, compat: true } },
    youtube: { label: "Para subir a YouTube", hint: "Calidade alta e MP4 con «faststart», que é o que YouTube recomenda. Non reduce a resolución.", v: { crf: 18, maxHeight: 0, audioKbps: 192, deinterlace: 1, compat: true } },
    movil: { label: "Móbil e mensaxería", hint: "Máis pequeno para enviar por WhatsApp ou correo: 720p e calidade equilibrada.", v: { crf: 26, maxHeight: 720, audioKbps: 128, deinterlace: 1, compat: true } },
    custom: { label: "Personalizado", hint: "Axustas ti a calidade, a altura e o audio.", v: { crf: 28, maxHeight: 0, audioKbps: 128, deinterlace: 0, compat: false } },
  };
  const initialProfile = isImage ? "custom" : "universal";
  const DARS = ["", "4/3", "16/9"];
  const UNITS = [
    { id: "KB", mult: 1024 },
    { id: "MB", mult: 1024 ** 2 },
    { id: "GB", mult: 1024 ** 3 },
  ];

  const form = createForm({
    mode: "quality" as "quality" | "bitrate" | "size",
    quality: 75,
    maxSide: null as number | null,
    profile: initialProfile,
    deinterlace: PROFILES[initialProfile].v.deinterlace,
    compat: PROFILES[initialProfile].v.compat,
    dar: "",
    crf: PROFILES[initialProfile].v.crf,
    maxHeight: PROFILES[initialProfile].v.maxHeight,
    audioKbps: PROFILES[initialProfile].v.audioKbps,
    videoKbps: 2000,
    sizeValue: 10,
    unit: "MB",
  }, `comprimir2-${category}`);
  onMount(() => {
    // Un perfil gardado que xa se borrou: volve ao de partida.
    if (!isImage && !known(get(form).profile)) {
      appliedProfile = initialProfile;
      form.update((f) => ({ ...f, profile: initialProfile, ...PROFILES[initialProfile].v }));
    }
    return useForm(form);
  });

  const job = createJob();
  const busy = job.busy;
  let estimating = false;
  let estimate: number | null = null;
  let results: { name: string; state: "agarda" | "feito" | "erro"; text?: string }[] = [];

  $: v = $form;
  $: targetBytes = Math.max(1, Math.round((v.sizeValue || 1) * (UNITS.find((u) => u.id === v.unit)?.mult ?? 1)));
  $: paramsKey = `${v.mode}|${v.quality}|${v.maxSide}|${v.crf}|${v.maxHeight}|${v.audioKbps}|${v.videoKbps}|${v.deinterlace}|${v.compat}|${v.dar}`;
  $: invalidate(paramsKey);
  $: if (isImage && !multi && v.mode === "quality") scheduleImageEstimate(paramsKey);

  // Ao escoller un perfil aplícanse os seus valores; despois pódese seguir afinando.
  let appliedProfile = get(form).profile;
  function applyProfile(id: string) {
    if (isImage || id === appliedProfile) return;
    appliedProfile = id;
    if (id.startsWith("user:")) {
      const mine = get(savedProfiles).find((x) => `user:${x.id}` === id);
      if (mine) form.update((f) => ({ ...f, ...mine.v }));
      return;
    }
    const p = PROFILES[id];
    if (p) form.update((f) => ({ ...f, mode: "quality" as const, dar: "", ...p.v }));
  }

  // ---- perfís gardados polo usuario
  let naming = false;
  let newName = "";
  function saveProfile() {
    const name = newName.trim();
    if (!name) return;
    const f = get(form);
    const id = Math.random().toString(36).slice(2, 9);
    savedProfiles.update((l) => [
      ...l,
      { id, name, v: { mode: f.mode === "bitrate" ? "bitrate" : "quality", crf: f.crf, videoKbps: f.videoKbps, maxHeight: f.maxHeight, audioKbps: f.audioKbps, deinterlace: f.deinterlace, compat: f.compat, dar: f.dar } },
    ]);
    appliedProfile = `user:${id}`;
    form.update((x) => ({ ...x, profile: `user:${id}` }));
    toast(tr("O trasno gardou o perfil «{name}».", { name }));
    naming = false;
    newName = "";
  }
  $: known = (id: string) => !!PROFILES[id] || $savedProfiles.some((x) => `user:${x.id}` === id);
  $: profileOptions = [
    ...Object.entries(PROFILES).map(([id, p]) => ({ value: id, label: $t(p.label) })),
    ...$savedProfiles.map((x) => ({ value: `user:${x.id}`, label: `★ ${x.name}` })),
  ];
  $: profileHint = v.profile.startsWith("user:") ? $t("Perfil gardado por ti. Podes xestionalo en Axustes → Perfís de saída.") : $t(PROFILES[v.profile]?.hint ?? "");
  $: applyProfile(v.profile);

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
      estimate = await engine.estimateVideo(first.path, v.crf, v.maxHeight || undefined, v.audioKbps, { deinterlace: v.deinterlace, compat: v.compat, dar: v.dar });
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
      {
        crf: v.crf,
        targetBytes: v.mode === "size" ? targetBytes : undefined,
        maxHeight: v.maxHeight || undefined,
        audioKbps: v.audioKbps,
        videoKbps: v.mode === "bitrate" ? v.videoKbps : undefined,
        deinterlace: v.deinterlace as 0 | 1 | 2,
        compat: v.compat,
        dar: v.dar || undefined,
      },
      overwrite,
    );
    return await engine.fileSize(done);
  }

  const outExt = () => (isImage ? "jpg" : "mp4");

  async function runSingle() {
    const stem = first.path.replace(/\.[^.\\/]+$/, "");
    const out = await askPath(withExtension(`${stem}${tr("-comprimido")}`, outExt()));
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

  async function runBatch() {
    const dir = await askDir(selection[0].path);
    if (!dir) return;
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

  {#if !isImage}
    <div class="group">
      <SelectField
        label={$t("Perfil de saída")}
        bind:value={$form.profile}
        options={profileOptions}
        def={initialProfile}
      />
      <p class="group-note">{profileHint}</p>
      {#if naming}
        <div class="savebox">
          <input class="nameinp" bind:value={newName} placeholder={$t("Nome do perfil")} aria-label={$t("Nome do perfil")} maxlength="40" on:keydown={(e) => e.key === "Enter" && saveProfile()} />
          <button class="btn primary" on:click={saveProfile} disabled={!newName.trim()}>{$t("Gardar")}</button>
          <button class="btn" on:click={() => ((naming = false), (newName = ""))}>{$t("Cancelar")}</button>
        </div>
      {:else}
        <button class="btn" on:click={() => (naming = true)}><Icon name="plus" size={14} /> {$t("Gardar estes axustes como perfil…")}</button>
      {/if}
      {#if v.profile !== "custom"}
        <SelectField
          label={$t("Relación de aspecto")}
          bind:value={$form.dar}
          options={DARS.map((d) => ({ value: d, label: d ? d.replace("/", ":") : $t("orixinal") }))}
          def=""
          hint={$t("Só se o vídeo se ve esticado ou aplastado; no resto déixao en orixinal.")}
        />
      {/if}
    </div>
  {/if}

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
        {#if !isImage}
          <span class="muted tiny">{$t("Estimación por mostra: pode variar un pouco.")}</span>
          <button class="btn ghost" on:click={estimateVideo} disabled={estimating || $busy}>{estimating ? $t("Estimando…") : $t("Estimar de novo")}</button>
        {/if}
      {:else if !isImage && v.mode === "quality"}
        <button class="btn" on:click={estimateVideo} disabled={estimating || $busy}>{estimating ? $t("Estimando…") : $t("Estimar o tamaño")}</button>
      {:else}
        <span class="muted">{$t("Estimando…")}</span>
      {/if}
    </div>
  {/if}

  {#if multi}
    <button class="btn primary block" on:click={runBatch} disabled={$busy}>
      {isImage ? $t("Comprimir {n} imaxes", { n: selection.length }) : $t("Converter {n} vídeos a MP4", { n: selection.length })}
    </button>
  {:else}
    <button class="btn primary block" on:click={runSingle} disabled={$busy}>
      {isImage ? $t("Comprimir imaxe") : $t("Converter a MP4")}
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
  .savebox { display: flex; gap: 0.4rem; align-items: center; }
  .nameinp { flex: 1; min-width: 0; height: var(--control-h); background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0 0.6rem; font: inherit; }
  .nameinp:focus { outline: none; border-color: var(--accent); }
  .tiny { font-size: 0.76rem; }
  .estimate { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.2rem 0.7rem; min-height: var(--control-h); align-content: center; }
  .est-main { font-size: 1.05rem; font-weight: 600; }
</style>
