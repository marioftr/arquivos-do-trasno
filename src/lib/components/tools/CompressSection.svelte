<script lang="ts">
  import InfoTip from "../ui/InfoTip.svelte";
    import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import { claim } from "../../claim";
  import { videoDar, type FileEntry } from "../../files";
  import { fileNameOf, formatBytes, withExtension, TRASNO, fallo, sameFormat, cannotSaveNotice, noCompressionNotice, animatedNotice, readOnlyLabel } from "../../formats";
  import LockNote from "../ui/LockNote.svelte";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { savedProfiles } from "../../settings";
  import { toast } from "../../toasts";
  import { samePath, askPath, askDir } from "./shared";
  import Icon from "../Icon.svelte";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import PhotoMeta from "../ui/PhotoMeta.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import { IMAGE_LEVELS, VIDEO_LEVELS } from "../../levels";
  import Levels from "../ui/Levels.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let selection: FileEntry[];
  export let category: "imaxe" | "video";

  const multi = selection.length > 1;
  const first = selection[0];
  const isImage = category === "imaxe";

  const HEIGHTS = [0, 1080, 720, 480, 360];

  // Perfís de saída para vídeo: cada un fixa a calidade e os axustes de compatibilidade.
  // `deinterlace`: 0 non, 1 só se está entrelazado, 2 sempre (cadencia dobre, como facía o DV orixinal).
  const PROFILES: Record<string, { label: string; hint?: string; v: { crf: number; maxHeight: number; audioKbps: number; deinterlace: number; compat: boolean } }> = {
    universal: { label: "Universal optimizado", hint: "MP4 H.264 que abre en móbiles, televisións e navegadores.", v: { crf: 20, maxHeight: 0, audioKbps: 160, deinterlace: 1, compat: true } },
    minidv: { label: "Mini DV a MP4 (sen perda visible)", hint: "Para cintas dixitalizadas en AVI/DV: desentrelaza e dá calidade moi alta.", v: { crf: 18, maxHeight: 0, audioKbps: 192, deinterlace: 2, compat: true } },
    youtube: { label: "Para subir a YouTube", hint: "Calidade alta e MP4 con «faststart», o que recomenda YouTube. Non reduce a resolución.", v: { crf: 18, maxHeight: 0, audioKbps: 192, deinterlace: 1, compat: true } },
    movil: { label: "Móbil e mensaxería", hint: "720p e calidade equilibrada, para WhatsApp ou correo.", v: { crf: 26, maxHeight: 720, audioKbps: 128, deinterlace: 1, compat: true } },
    custom: { label: "Personalizado", v: { crf: 28, maxHeight: 0, audioKbps: 128, deinterlace: 0, compat: false } },
  };
  const initialProfile = isImage ? "custom" : "universal";
  // Opcións segundo a orientación do vídeo: en vertical, as proporcións «de pé».
  $: portrait = !!first && (first as any).width && (first as any).height ? (first as any).height > (first as any).width : false;
  $: DARS = portrait ? ["", "3/4", "9/16", "1/1"] : ["", "4/3", "16/9", "21/9"];
  const UNITS = [
    { id: "KB", mult: 1024 },
    { id: "MB", mult: 1024 ** 2 },
    { id: "GB", mult: 1024 ** 3 },
  ];

  const form = createForm({
    mode: "quality" as "quality" | "bitrate" | "size",
    quality: 75,
    pngQuality: 100,
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
    keepMeta: true,
    keepGps: true,
  }, `comprimir2-${category}`, "compress|" + selection.map((f) => f.path).join("|"));
  $: videoDar.set(isImage ? "" : v.dar ?? "");
  const own = claim("videoDar");
  onDestroy(() => own.release(() => videoDar.set("")));
  onMount(() => {
    return useForm(form);
  });

  const job = createJob();
  const busy = job.busy;
  let estimating = false;
  let estimate: number | null = null;
  let results: { name: string; state: "agarda" | "feito" | "erro"; text?: string }[] = [];

  $: v = $form;
  // Cada formato ten a súa calidade: en PNG, 100 é sen perdas (por defecto) e menos reduce a paleta de cores.
  const isPngExt = (ext: string) => engine.compressExt(ext) === "png";
  // O resultado mantén sempre o formato do orixinal (para cambiar de formato está Converter). Hai ficheiros que non se poden
  // comprimir así: formatos sen compresión propia (BMP, TIFF…), que só se saben ler (HEIC…) e imaxes animadas que non son GIF.
  let animatedPaths = new Set<string>();
  $: void checkAnimated(isImage ? selection : []);
  async function checkAnimated(list: FileEntry[]) {
    const found = new Set<string>();
    await Promise.all(
      list
        .filter((f) => ["webp", "png", "apng", "gif"].includes(f.ext.toLowerCase()))
        .map((f) => engine.imageIsAnimated(f.path).then((a) => a && found.add(f.path)).catch(() => {})),
    );
    if (list === selection || !isImage) animatedPaths = found;
  }
  const keepsFormat = (f: FileEntry) => engine.COMPRESS_KEEP.includes(f.ext.toLowerCase());
  const isBlocked = (f: FileEntry, anim: Set<string>) => isImage && (!keepsFormat(f) || (anim.has(f.path) && f.ext.toLowerCase() !== "gif"));
  $: blocked = selection.filter((f) => isBlocked(f, animatedPaths));
  $: usable = selection.filter((f) => !isBlocked(f, animatedPaths));
  $: allBlocked = isImage && usable.length === 0;
  function whyBlocked(f: FileEntry): string {
    if (!sameFormat(f.ext)) return cannotSaveNotice(f.ext, "comprimir");
    if (!keepsFormat(f)) return noCompressionNotice(f.ext);
    return animatedNotice(f.ext, "comprimir");
  }
  $: blockedText = blocked.length === 1 ? whyBlocked(blocked[0]) : tr("{n} ficheiros non se comprimen: ou o seu formato non ten compresión propia, ou só se saben ler, ou son imaxes animadas que non son GIF. Convérteos antes a PNG ou JPG na pestana Converter.", { n: blocked.length }) + " " + [...new Set(blocked.map((f) => f.ext.toUpperCase()))].join(", ");
  $: hasPng = isImage && usable.some((f) => isPngExt(f.ext));
  $: hasLossy = isImage && usable.some((f) => !isPngExt(f.ext));
  $: hasGif = isImage && usable.some((f) => f.ext.toLowerCase() === "gif");
  $: targetBytes = Math.max(1, Math.round((v.sizeValue || 1) * (UNITS.find((u) => u.id === v.unit)?.mult ?? 1)));
  $: paramsKey = `${v.mode}|${v.quality}|${v.pngQuality}|${v.maxSide}|${v.crf}|${v.maxHeight}|${v.audioKbps}|${v.videoKbps}|${v.deinterlace}|${v.compat}|${v.dar}`;
  $: invalidate(paramsKey);
  $: if (isImage && !multi && !allBlocked && v.mode === "quality") scheduleImageEstimate(paramsKey);

  // Ao escoller un perfil aplícanse os seus valores; despois pódese seguir afinando.
  // Os valores que definen un perfil. Se non coinciden cos do perfil elixido, o perfil pasa a «Personalizado».
  const keyOf = (f: any) => JSON.stringify([f.mode, f.crf, f.maxHeight, f.audioKbps, f.deinterlace, f.compat, f.dar, f.mode === "bitrate" ? f.videoKbps : 0]);
  const valuesOf = (id: string): Record<string, any> | null => {
    if (id.startsWith("user:")) {
      const mine = get(savedProfiles).find((x) => `user:${x.id}` === id);
      return mine ? { ...mine.v } : null;
    }
    return PROFILES[id] && id !== "custom" ? { mode: "quality", dar: "", ...PROFILES[id].v } : null;
  };
  /** Elixe un perfil: cambia o perfil e os seus valores dunha soa vez. */
  function pickProfile(id: string) {
    if (isImage) return;
    form.update((f) => ({ ...f, ...(valuesOf(id) ?? {}), profile: id }));
  }
  $: if (!isImage && v.profile !== "custom") {
    const want = valuesOf(v.profile);
    if (!want || keyOf(v) !== keyOf({ ...v, ...want })) form.update((x) => ({ ...x, profile: "custom" }));
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
    form.update((x) => ({ ...x, profile: `user:${id}` }));
    toast(tr("Perfil «{name}» gardado.", { name }));
    naming = false;
    newName = "";
  }
  $: profileOptions = [
    ...Object.entries(PROFILES).map(([id, p]) => ({ value: id, label: $t(p.label) })),
    ...$savedProfiles.map((x) => ({ value: `user:${x.id}`, label: `★ ${x.name}` })),
  ];
  // A «i» do perfil explica cada perfil fixo, un por liña (Personalizado e os teus perfís non levan texto).
  $: profilesTip = Object.values(PROFILES).filter((p) => p.hint).map((p) => `${$t(p.label)}: ${$t(p.hint ?? "")}`).join(String.fromCharCode(10));

  // Nivel de calidade con nome: só conta o valor dos formatos que hai na selección.
  $: imageValues = { ...(hasLossy ? { quality: v.quality } : {}), ...(hasPng ? { pngQuality: v.pngQuality } : {}) };
  $: imageLevels = IMAGE_LEVELS.map((l) => ({ id: l.id, label: $t(l.label), detail: `${l.q} %`, hint: $t(l.hint), values: { ...(hasLossy ? { quality: l.q } : {}), ...(hasPng ? { pngQuality: l.q } : {}) } }));
  $: videoLevels = VIDEO_LEVELS.map((l) => ({ id: String(l.crf), label: $t(l.label), detail: `CRF ${l.crf}`, hint: $t(l.hint), values: { crf: l.crf } }));

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
        const bytes = await engine.estimateImage(first.path, first.ext.toLowerCase(), isPngExt(first.ext) ? v.pngQuality : v.quality, v.maxSide || undefined);
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
        { quality: v.mode === "quality" ? (isPngExt(f.ext) ? v.pngQuality : v.quality) : undefined, targetBytes: v.mode === "size" ? targetBytes : undefined, maxSide: v.maxSide || undefined, metadata: engine.photoMetadata(v.keepMeta, v.keepGps) },
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

  const outExt = (f: FileEntry) => (isImage ? f.ext.toLowerCase() : "mp4");

  async function runSingle() {
    const stem = first.path.replace(/\.[^.\\/]+$/, "");
    const out = await askPath(withExtension(`${stem}${tr("-comprimido")}`, outExt(first)));
    if (!out) return;
    if (samePath(out, first.path)) {
      job.say(false, tr("Non vou sobrescribir o ficheiro orixinal. Escolle outro nome."));
      return;
    }
    await job.run(TRASNO.comprimindo, fallo("comprimindo"), async () => {
      const before = (await engine.fileSize(first.path).catch(() => 0)) || (first.size ?? 0);
      const bytes = await compressOne(first, out, true);
      if (!before) return tr("Listo! Deixei o ficheiro en {to}.", { to: formatBytes(bytes) });
      return tr("Listo! Deixei o ficheiro en {to} ({pct}; antes eran {from}).", { from: formatBytes(before), to: formatBytes(bytes), pct: pct(before, bytes) });
    });
  }

  async function runBatch() {
    // As imaxes que non se poden comprimir mantendo o formato quedan fóra do lote (avísase arriba).
    const files = isImage ? usable : selection;
    const dir = await askDir(files[0].path);
    if (!dir) return;
    results = files.map((f) => ({ name: f.name, state: "agarda" }));
    job.busy.set(true);
    job.text.set(tr(TRASNO.comprimindo));
    job.clear();
    let before = 0;
    let after = 0;
    let cursor = 0;
    const worker = async () => {
      while (cursor < files.length) {
        const i = cursor++;
        const f = files[i];
        try {
          const dot = f.name.lastIndexOf(".");
          const base = dot > 0 ? f.name.slice(0, dot) : f.name;
          const out = await join(dir, `${base}${tr("-comprimido")}.${outExt(f)}`);
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
    await Promise.all(Array.from({ length: Math.min(isImage ? 3 : 1, files.length) }, worker));
    const bad = results.filter((r) => r.state === "erro").length;
    job.say(!bad, bad
      ? tr("Vaites! {bad} de {total} ficheiros escapáronseme.", { bad, total: files.length })
      : tr("Feito! Comprimín {n} ficheiros en {dir}: de {from} a {to}.", { n: files.length, dir: fileNameOf(dir), from: formatBytes(before), to: formatBytes(after) }));
    job.busy.set(false);
  }
</script>

<div class="stack">
  {#if isImage && blocked.length}<LockNote label={allBlocked ? readOnlyLabel() : tr("{n} sen comprimir", { n: blocked.length })} text={blockedText} />{/if}
  <div class="locked-tools" class:on={allBlocked} inert={allBlocked}>

  {#if !isImage}
    <div class="group">
      <SelectField
        label={$t("Perfil de saída")}
        value={v.profile}
        on:change={(e) => pickProfile(String(e.detail))}
        options={profileOptions}
        def={initialProfile}
        hint={profilesTip}
      />
      {#if !isImage}
        <Segmented
          label={$t("Relación de aspecto")}
          info={$t("Só para vídeos que se ven esticados ou aplastados.")}
          options={DARS.map((d) => ({ value: d, label: d ? d.replace("/", ":") : $t("orixinal") }))}
          bind:value={$form.dar}
        />
      {/if}
    </div>
  {/if}

  <Segmented
    label={$t("Como se axusta o peso")}
    info={isImage
      ? $t("Por calidade: calidade fixa; o peso depende da imaxe. Tamaño máximo: búscase a calidade máis alta que quede por debaixo do límite.")
      : $t("Por calidade: calidade fixa; o peso depende do vídeo. Por bitrate: datos por segundo fixos. Tamaño máximo: búscase a calidade máis alta que quede por debaixo do límite.")}
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
      {#if tooSmall}<p class="warn">{$t("Aviso: con este tamaño o vídeo quedará con moi pouca calidade.")}</p>{/if}
    {:else if v.mode === "bitrate" && !isImage}
      <Slider label={$t("Bitrate de vídeo")} unit="kbps" min={200} max={12000} step={100} bind:value={$form.videoKbps} def={form.defaults.videoKbps} hint={$t("Datos por segundo da imaxe: máis alto, máis calidade e máis peso.")} />
    {:else if isImage}
      <Levels label={$t("Nivel de calidade")} levels={imageLevels} values={imageValues} on:apply={(e) => form.update((f) => ({ ...f, ...e.detail }))} />
      {#if hasLossy}
        <Slider label={hasPng ? $t("Calidade (JPG, WebP…)") : $t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={form.defaults.quality} hint={hasGif ? $t("Nos GIF, menos calidade son menos cores; os fotogramas non cambian.") : undefined} />
      {/if}
      {#if hasPng}
        <Slider label={hasLossy ? $t("Calidade do PNG") : $t("Calidade")} unit="%" min={10} max={100} bind:value={$form.pngQuality} def={form.defaults.pngQuality} hint={$t("100 %: non cambia ningún píxel. Menos: reduce as cores para pesar menos, sen perder a transparencia.")} />
      {/if}
    {:else}
      <Levels label={$t("Nivel de calidade")} levels={videoLevels} values={{ crf: v.crf }} on:apply={(e) => form.update((f) => ({ ...f, ...e.detail }))} />
      <Slider label={$t("Calidade (CRF)")} min={18} max={40} bind:value={$form.crf} def={form.defaults.crf} />
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
  </div>

  {#if !multi && !allBlocked}
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

  {#if !isImage && v.profile === "custom"}
    <div class="savewrap">
      {#if naming}
        <div class="savebox">
          <input class="nameinp" bind:value={newName} placeholder={$t("Nome do perfil")} aria-label={$t("Nome do perfil")} maxlength="40" on:keydown={(e) => e.key === "Enter" && saveProfile()} />
          <button class="btn primary" on:click={saveProfile} disabled={!newName.trim()}>{$t("Gardar")}</button>
          <button class="btn" on:click={() => ((naming = false), (newName = ""))}>{$t("Cancelar")}</button>
        </div>
      {:else}
        <button class="btn block" on:click={() => (naming = true)}><Icon name="plus" size={14} /> {$t("Gardar estes axustes como perfil…")}</button>
      {/if}
    </div>
  {/if}

  {#if multi}
    <button class="btn primary block" on:click={runBatch} disabled={$busy || allBlocked}>
      {isImage ? $t("Comprimir {n} imaxes", { n: usable.length }) : $t("Converter {n} vídeos a MP4", { n: selection.length })}
    </button>
  {:else}
    <button class="btn primary block" on:click={runSingle} disabled={$busy || allBlocked}>
      {isImage ? $t("Comprimir imaxe") : $t("Converter a MP4")}
    </button>
  {/if}
  {#if isImage && usable.some((f) => engine.META_EXTS.includes(f.ext.toLowerCase()))}
    <PhotoMeta bind:keep={$form.keepMeta} bind:gps={$form.keepGps} disabled={$busy} />
  {/if}

  </div>

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
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); }
  .savewrap { display: flex; flex-direction: column; }
  .savebox { display: flex; gap: 0.4rem; align-items: center; }
  .nameinp { flex: 1; min-width: 0; height: var(--control-h); background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0 0.6rem; font: inherit; }
  .nameinp:focus { outline: none; border-color: var(--accent); }
  .tiny { font-size: var(--fs-xs); }
</style>
