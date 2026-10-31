<script lang="ts">
  import { errText } from "../../errors";
  import { onMount } from "svelte";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo, AUDIO_TARGETS } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const isVideo = file.category === "video";

  const form = createForm({
    frameAt: 0,
    speed: 1,
    volume: 1,
    height: Math.min(720, file.height ?? 720),
    audioExt: "mp3",
    audioKbps: 192,
  });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  async function run(suffix: string, ext: string, work: (out: string) => Promise<string>) {
    try {
      const out = await askOutput(file, tr(suffix), ext);
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => `${listo()} ${fileNameOf(await work(out))}`);
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }

  const frame = () => run("-fotograma", "png", (o) => engine.mediaEdit(file.path, o, "frame", $form.frameAt, true, true));
  const speed = () => run("-velocidade", file.ext, (o) => engine.mediaEdit(file.path, o, "speed", $form.speed, isVideo, true));
  const volume = () => run("-volume", file.ext, (o) => engine.mediaEdit(file.path, o, "volume", $form.volume, isVideo, true));
  const mute = () => run("-mudo", file.ext, (o) => engine.mediaEdit(file.path, o, "mute", 0, true, true));
  const scale = () => run(`-${$form.height}p`, file.ext, (o) => engine.mediaEdit(file.path, o, "scale", $form.height, true, true));
  const audio = () =>
    run("", $form.audioExt, async (o) => (await engine.convertFile(file.path, o, { overwrite: true, bitrateKbps: $form.audioKbps })).output_path);
</script>

<div class="stack">
  <FormBar {form} />

  <div class="group">
    <h4 class="group-title">{$t("Velocidade")}</h4>
    <Slider label={$t("Factor de velocidade")} unit="×" min={0.25} max={4} step={0.05} bind:value={$form.speed} def={1} hint={$t("2 = o dobre de rápido; 0,5 = a metade")} />
    <button class="btn block" on:click={speed} disabled={$busy || $form.speed === 1}>{$t("Aplicar velocidade ×{v}", { v: $form.speed })}</button>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Volume")}</h4>
    <Slider label={$t("Multiplicador de volume")} unit="×" min={0} max={4} step={0.05} bind:value={$form.volume} def={1} hint={$t("1 = igual que o orixinal")} />
    <button class="btn block" on:click={volume} disabled={$busy || $form.volume === 1}>{$t("Aplicar volume ×{v}", { v: $form.volume })}</button>
    {#if isVideo}
      <button class="btn block" on:click={mute} disabled={$busy}>{$t("Quitar o son")}</button>
    {/if}
  </div>

  {#if isVideo}
    <div class="group">
      <h4 class="group-title">{$t("Imaxe")}</h4>
      <div class="pair">
        <NumberField label={$t("Instante do fotograma")} unit="s" min={0} step={0.1} bind:value={$form.frameAt} def={0} />
        <NumberField label={$t("Nova altura")} unit="px" min={16} step={2} bind:value={$form.height} />
      </div>
      <div class="pair">
        <button class="btn" on:click={frame} disabled={$busy}>{$t("Gardar fotograma")}</button>
        <button class="btn" on:click={scale} disabled={$busy}>{$t("Reducir vídeo")}</button>
      </div>
    </div>
  {/if}

  {#if isVideo}
    <div class="group">
      <h4 class="group-title">{$t("Extraer audio")}</h4>
      <div class="pair">
        <SelectField label={$t("Formato")} bind:value={$form.audioExt} options={AUDIO_TARGETS.map((x) => ({ value: x, label: x.toUpperCase() }))} def="mp3" />
        <NumberField label={$t("Bitrate de audio")} unit="kbps" min={32} max={320} bind:value={$form.audioKbps} def={192} />
      </div>
      <button class="btn block" on:click={audio} disabled={$busy}>{$t("Extraer o audio")}</button>
    </div>
  {/if}

  <JobStatus {job} />
</div>
