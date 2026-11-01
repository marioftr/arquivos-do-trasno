<script lang="ts">
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { mediaPreview, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const isVideo = file.category === "video";

  const form = createForm({
    speed: 1,
    volume: 1,
  });
  onMount(() => useForm(form));
  // O visor reproduce xa coa velocidade e o volume elixidos.
  $: mediaPreview.set({ speed: $form.speed, volume: $form.volume });
  onDestroy(() => mediaPreview.set(null));

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

  const speed = () => run("-velocidade", file.ext, (o) => engine.mediaEdit(file.path, o, "speed", $form.speed, isVideo, true));
  const volume = () => run("-volume", file.ext, (o) => engine.mediaEdit(file.path, o, "volume", $form.volume, isVideo, true));
  const mute = () => run("-mudo", file.ext, (o) => engine.mediaEdit(file.path, o, "mute", 0, true, true));
</script>

<div class="stack">

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

  <JobStatus {job} />
</div>
