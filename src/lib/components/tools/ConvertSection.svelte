<script lang="ts">
  import { errText } from "../../errors";
  import { onMount } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { Category } from "../../engine";
  import type { FileEntry } from "../../files";
  import { convertModel } from "../../model3d";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { targetsForCategory, withExtension, fileNameOf, formatBytes, AUDIO_TARGETS, TRASNO, fallo, listo } from "../../formats";
  import { samePath, askPath, askDir } from "./shared";
  import Icon from "../Icon.svelte";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import FormBar from "../ui/FormBar.svelte";
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
  const targets = targetsForCategory(category, first.ext);
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
  });
  onMount(() => useForm(form));

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
    if (f.category === "modelo3d" && !(ENGINE_3D.includes(f.ext) && ENGINE_3D.includes(v.targetExt))) {
      return { output_path: out, bytes_written: await convertModel(f.path, f.ext, out, v.targetExt, overwrite) };
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
      resize: f.category === "imaxe" ? resizeFor(f) : undefined,
      bitrateKbps: toAudio ? v.audioBitrate : isMedia && v.bitrate ? v.bitrate : undefined,
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
  <FormBar {form} />

  <div class="group">
    <SelectField
      label={$t("Formato de saída")}
      bind:value={$form.targetExt}
      options={targets.map((e) => ({ value: e, label: e.toUpperCase() }))}
      def={form.defaults.targetExt}
      disabled={$busy}
    />

    {#if category === "imaxe" && $form.targetExt === "pdf"}
      <p class="group-note">{$t("Cada imaxe vai a unha páxina do PDF, co seu tamaño orixinal.")}</p>
    {:else if category === "imaxe"}
      <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={form.defaults.quality} hint={$t("Máis alto = mellor imaxe e máis peso (só JPG)")} />
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
      <p class="group-note">{$t("Só se garda o son do vídeo.")}</p>
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
      <p class="group-note">{$t("Os FBX e glTF conviértense co mesmo cargador do visor.")}</p>
    {/if}
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
