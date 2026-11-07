<script lang="ts">
  import GroupNote from "../ui/GroupNote.svelte";
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { Category } from "../../engine";
  import type { FileEntry } from "../../files";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { targetsForCategory, OFFICE_EXTS, PDF_TO_DOC, withExtension, fileNameOf, formatBytes, AUDIO_TARGETS, TRASNO, fallo, listo, firstFrameNotice } from "../../formats";
  import LockNote from "../ui/LockNote.svelte";
  import { samePath, askPath, askDir } from "./shared";
  import Icon from "../Icon.svelte";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import { NO_ALPHA, BACKGROUNDS, anyTransparent, backdrop, cssOf, type Bg } from "../../transparency";
  import JobStatus from "../ui/JobStatus.svelte";

  export let selection: FileEntry[];
  export let category: Category;

  const MAX_CONCURRENT = 3;
  const PRESETS = [
    { label: "WhatsApp", value: 55, hint: "Máis lixeiro, ideal para enviar rápido" },
    { label: "Web", value: 75, hint: "Equilibrio entre peso e nitidez" },
    { label: "Arquivo", value: 95, hint: "Case sen perdas, para gardar" },
  ];

  const first = selection[0];
  const multi = selection.length > 1;
  let targets = targetsForCategory(category, first.ext);
  // Documentos de Office (e PDF → documento): as saídas dependen do que haxa instalado (Microsoft Office / LibreOffice).
  const viaOffice = (ext: string, to: string) => OFFICE_EXTS.includes(ext) || (ext === "pdf" && PDF_TO_DOC.includes(to));
  const hasOffice = category === "documento" && selection.some((f) => OFFICE_EXTS.includes(f.ext) || f.ext === "pdf");
  onMount(async () => {
    if (!hasOffice) return;
    try {
      const lists = await Promise.all(selection.map((f) => engine.officeTargets(f.ext)));
      const common = lists.reduce((a, l) => a.filter((x) => l.includes(x)));
      const base = targetsForCategory(category, first.ext);
      targets = [...base.filter((x) => selection.every((f) => targetsForCategory(category, f.ext).includes(x))), ...common.filter((x) => !base.includes(x))];
      if (!targets.includes($form.targetExt)) form.update((f) => ({ ...f, targetExt: targets.find((x) => x !== first.ext) ?? targets[0] ?? "" }));
    } catch {
      /* sen Office: só as saídas de sempre */
    }
  });
  const isMedia = category === "audio" || category === "video";

  const form = createForm({
    targetExt: targets.find((x) => x !== first.ext) ?? targets[0] ?? "",
    quality: 85,
    bitrate: category === "audio" ? 192 : category === "video" ? 4000 : 0,
    audioBitrate: 192,
    resizeW: null as number | null,
    resizeH: null as number | null,
    keepRatio: true,
    maxSide: null as number | null,
    lossless: false,
    bg: "white" as Bg,
  }, undefined, "convert|" + selection.map((f) => f.path).join("|"));
  onMount(() => useForm(form));

  // Sen perdas: só onde o formato de destino o permite.
  $: canLossless = (category === "imaxe" && ["jxl", "jp2", "avif"].includes($form.targetExt)) || (category === "video" && ["mp4", "mkv", "mov"].includes($form.targetExt));

  // Se hai imaxes con transparencia e o formato de destino non a admite, hai que escoller o fondo.
  $: needsBg = category === "imaxe" && NO_ALPHA.includes($form.targetExt);
  let transparent = false;
  $: checkTransparency(needsBg, selection);
  async function checkTransparency(on: boolean, sel: FileEntry[]) {
    if (!on) {
      transparent = false;
      return;
    }
    const r = await anyTransparent(sel.map((f) => f.path));
    if (on === needsBg && sel === selection) transparent = r;
  }

  // Unha imaxe animada só mantén a animación ao pasar a GIF ou WebP (desde GIF ou PNG animado); a calquera outro formato
  // só se garda o primeiro fotograma, e iso avísase antes de converter.
  let animatedFiles: FileEntry[] = [];
  $: void findAnimated(category === "imaxe" ? selection : []);
  async function findAnimated(list: FileEntry[]) {
    const out: FileEntry[] = [];
    await Promise.all(
      list.filter((f) => ["gif", "webp", "png", "apng"].includes(f.ext.toLowerCase())).map((f) => engine.imageIsAnimated(f.path).then((a) => a && out.push(f)).catch(() => {})),
    );
    if (list === selection || category !== "imaxe") animatedFiles = out;
  }
  const keepsAnimation = (f: FileEntry, to: string) => ["gif", "png", "apng"].includes(f.ext.toLowerCase()) && ["gif", "webp"].includes(to) && f.ext.toLowerCase() !== to;
  $: losing = category === "imaxe" ? animatedFiles.filter((f) => !keepsAnimation(f, $form.targetExt)) : [];

  $: backdrop.set(needsBg && transparent && selection.length === 1 ? cssOf($form.bg) : null);
  onDestroy(() => backdrop.set(null));

  // Con «sen perdas» o resultado é o orixinal tal cal: calidade, bitrate e tamaño deixan de ter sentido.
  $: exact = canLossless && $form.lossless;

  const job = createJob();
  const busy = job.busy;
  let results: { name: string; state: "agarda" | "feito" | "erro"; text?: string }[] = [];

  function onW(raw: string) {
    const w = raw === "" ? null : Number(raw);
    form.update((f) => ({
      ...f,
      resizeW: w,
      resizeH: w && f.keepRatio && first.width && first.height ? Math.max(1, Math.round((w * first.height) / first.width)) : f.resizeH,
    }));
  }
  function onH(raw: string) {
    const h = raw === "" ? null : Number(raw);
    form.update((f) => ({
      ...f,
      resizeH: h,
      resizeW: h && f.keepRatio && first.width && first.height ? Math.max(1, Math.round((h * first.width) / first.height)) : f.resizeW,
    }));
  }

  function resizeFor(f: FileEntry): [number, number] | undefined {
    const v = $form;
    if (!multi) return v.resizeW && v.resizeH ? [v.resizeW, v.resizeH] : undefined;
    if (!v.maxSide || !f.width || !f.height) return undefined;
    const k = Math.min(1, v.maxSide / Math.max(f.width, f.height));
    return k >= 1 ? undefined : [Math.max(1, Math.round(f.width * k)), Math.max(1, Math.round(f.height * k))];
  }

  const ENGINE_3D = ["obj", "stl", "ply"];

  async function convertOne(f: FileEntry, out: string, overwrite: boolean) {
    const v = $form;
    if (f.category === "documento" && viaOffice(f.ext, v.targetExt)) {
      const done = await engine.convertOffice(f.path, out, overwrite);
      return { output_path: done, bytes_written: await engine.fileSize(done) };
    }
    if (f.category === "modelo3d" && !(ENGINE_3D.includes(f.ext) && ENGINE_3D.includes(v.targetExt))) {
      return { output_path: out, bytes_written: await (await import("../../model3d")).convertModel(f.path, f.ext, out, v.targetExt, overwrite) };
    }
    if (f.category === "imaxe" && v.targetExt === "pdf") {
      const done = await engine.imagesToPdf([f.path], out, overwrite);
      return { output_path: done, bytes_written: await engine.fileSize(done) };
    }
    // Un vídeo que se converte só a audio leva o bitrate de audio.
    const toAudio = f.category === "video" && AUDIO_TARGETS.includes(v.targetExt);
    return engine.convertFile(f.path, out, {
      overwrite,
      quality: f.category === "imaxe" ? v.quality : undefined,
      resize: f.category === "imaxe" && !exact ? resizeFor(f) : undefined,
      bitrateKbps: canLossless && v.lossless ? undefined : toAudio ? v.audioBitrate : isMedia && v.bitrate ? v.bitrate : undefined,
      lossless: canLossless && v.lossless,
      background: f.category === "imaxe" && NO_ALPHA.includes(v.targetExt) ? BACKGROUNDS[v.bg] : undefined,
    });
  }

  async function convertSingle() {
    const out = await askPath(withExtension(first.path, $form.targetExt));
    if (!out) return;
    if (samePath(out, first.path)) {
      job.say(false, tr("Non vou sobrescribir o ficheiro orixinal. Escolle outro nome."));
      return;
    }
    await job.run(TRASNO.convertendo, fallo("convertendo"), async () => {
      const r = await convertOne(first, out, true);
      return `${listo()} ${fileNameOf(r.output_path)} · ${formatBytes(r.bytes_written)}`;
    });
  }

  async function convertBatch() {
    const dir = await askDir(selection[0].path);
    if (!dir) return;
    const ext = $form.targetExt;
    results = selection.map((f) => ({ name: f.name, state: "agarda" }));
    job.busy.set(true);
    job.text.set(tr(TRASNO.convertendo));
    job.clear();
    let cursor = 0;
    const worker = async () => {
      while (cursor < selection.length) {
        const i = cursor++;
        const f = selection[i];
        try {
          const dot = f.name.lastIndexOf(".");
          const base = dot > 0 ? f.name.slice(0, dot) : f.name;
          // Mesmo formato (p. ex. comprimir un JPG): outro nome, nunca o orixinal.
          const out = await join(dir, ext === f.ext ? `${base}-trasno.${ext}` : `${base}.${ext}`);
          if (samePath(out, f.path)) throw new Error(tr("Coincidiría co ficheiro orixinal."));
          const r = await convertOne(f, out, false);
          results[i] = { name: f.name, state: "feito", text: f.size ? `${formatBytes(f.size)} → ${formatBytes(r.bytes_written)}` : formatBytes(r.bytes_written) };
        } catch (e) {
          results[i] = { name: f.name, state: "erro", text: errText(e) };
        }
        results = results;
      }
    };
    await Promise.all(Array.from({ length: Math.min(MAX_CONCURRENT, selection.length) }, worker));
    const bad = results.filter((r) => r.state === "erro").length;
    job.say(!bad, bad
      ? tr("Vaites! {bad} de {total} ficheiros escapáronlle ao trasno.", { bad, total: selection.length })
      : tr("Feito! O trasno gardou os {n} ficheiros en {dir} sen perder nin un.", { n: selection.length, dir: fileNameOf(dir) }));
    job.busy.set(false);
  }
</script>

<div class="stack">

  <div class="group">
    <SelectField
      label={$t("Formato de saída")}
      bind:value={$form.targetExt}
      options={targets.map((e) => ({ value: e, label: e.toUpperCase() }))}
      def={form.defaults.targetExt}
      disabled={$busy}
    />

    {#if losing.length}
      <LockNote
        label={losing.length === 1 ? $t("Só o primeiro fotograma") : $t("{n} perden a animación", { n: losing.length })}
        text={firstFrameNotice(losing[0].ext, $form.targetExt) + (losing.length > 1 ? " " + tr("Afecta a {n} ficheiros.", { n: losing.length }) : "")}
      />
    {/if}

    {#if needsBg && transparent}
      <Segmented
        label={$t("Fondo")}
        info={$t("A imaxe ten transparencia e este formato non a admite: hai que escoller un fondo sobre o que compoñela.")}
        options={[{ value: "white", label: $t("Branco") }, { value: "black", label: $t("Negro") }, { value: "gray", label: $t("Gris neutro") }]}
        bind:value={$form.bg}
      />
    {/if}

    {#if canLossless}
      <Toggle label={$t("Sen perdas")} bind:checked={$form.lossless} hint={$t("Conserva exactamente a calidade do orixinal, sen compresión con perdas. O ficheiro resultante pesa moito máis e o proceso tarda máis.")} />
    {/if}

    <div class="locked" class:on={exact} inert={exact}>
    {#if category === "imaxe" && $form.targetExt === "pdf"}
      <GroupNote text={$t("Cada imaxe vai a unha páxina do PDF, co seu tamaño orixinal.")} />
    {:else if category === "imaxe"}
      <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={form.defaults.quality} hint={$t("Máis alto = mellor imaxe e máis peso (JPG, WebP, AVIF e JPEG XL)")} />
      <Segmented options={PRESETS.map((p) => ({ value: p.value, label: $t(p.label), hint: $t(p.hint) }))} bind:value={$form.quality} />
      {#if multi}
        <NumberField label={$t("Lado máximo")} unit="px" min={1} placeholder={$t("orixinal")} bind:value={$form.maxSide} def={null} />
      {:else}
        <div class="pair">
          <NumberField label={$t("Ancho")} unit="px" min={1} value={$form.resizeW} onInput={onW} placeholder={first.width ? String(first.width) : ""} def={null} />
          <NumberField label={$t("Alto")} unit="px" min={1} value={$form.resizeH} onInput={onH} placeholder={first.height ? String(first.height) : ""} def={null} />
        </div>
        <Toggle label={$t("Manter proporción")} bind:checked={$form.keepRatio} />
      {/if}
    {:else if category === "video" && AUDIO_TARGETS.includes($form.targetExt)}
      <GroupNote text={$t("Só se garda o son do vídeo.")} />
      <Slider label={$t("Bitrate de audio")} unit="kbps" min={32} max={320} step={8} bind:value={$form.audioBitrate} def={form.defaults.audioBitrate} hint={$t("Máis alto = máis calidade e máis tamaño")} />
    {:else if isMedia}
      <Slider
        label={category === "audio" ? $t("Bitrate de audio") : $t("Bitrate de vídeo")}
        unit="kbps"
        min={category === "audio" ? 32 : 500}
        max={category === "audio" ? 320 : 12000}
        step={category === "audio" ? 8 : 100}
        bind:value={$form.bitrate}
        def={form.defaults.bitrate}
        hint={$t("Máis alto = máis calidade e máis tamaño")}
      />
    {:else if category === "modelo3d" && !ENGINE_3D.includes(first.ext)}
      <GroupNote text={$t("Os FBX e glTF conviértense co mesmo cargador do visor.")} />
    {/if}
    </div>
  </div>

  {#if multi}
    <button class="btn primary block" on:click={convertBatch} disabled={$busy || !$form.targetExt}>
      {$t("Converter {n} ficheiros", { n: selection.length })}
    </button>
  {:else}
    <button class="btn primary block" on:click={convertSingle} disabled={$busy || !$form.targetExt}>
      {$t("Converter a {ext}", { ext: $form.targetExt.toUpperCase() })}
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
  /* «Sen perdas»: os axustes que xa non valen quedan en gris e bloqueados (non desaparecen) */
  .locked { display: flex; flex-direction: column; gap: inherit; transition: opacity 0.2s, filter 0.2s; }
  .locked.on { opacity: 0.4; filter: grayscale(1); pointer-events: none; user-select: none; }
</style>
