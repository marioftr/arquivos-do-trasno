<script lang="ts">
  import { onMount } from "svelte";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { mediaTime, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo, AUDIO_TARGETS } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import FramePreview from "../ui/FramePreview.svelte";

  export let file: FileEntry;

  const form = createForm({ frameMode: "viewer", frameAt: 0, audioExt: "mp3", audioKbps: 192 });
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

  const frame = (at: number) => run("-fotograma", "png", (o) => engine.mediaEdit(file.path, o, "frame", at, true, true));
  const audio = () => run("", $form.audioExt, async (o) => (await engine.convertFile(file.path, o, { overwrite: true, bitrateKbps: $form.audioKbps })).output_path);
</script>

<div class="stack">
  <FormBar {form} />

  <div class="group">
    <h4 class="group-title">{$t("Fotograma")}</h4>
    <Segmented
      options={[
        { value: "viewer", label: $t("Onde está o visor") },
        { value: "exact", label: $t("Instante exacto") },
      ]}
      bind:value={$form.frameMode}
    />
    {#if $form.frameMode === "exact"}
      <NumberField label={$t("Instante")} unit="s" min={0} step={0.1} bind:value={$form.frameAt} def={0} />
    {/if}
    <FramePreview path={file.path} time={$form.frameMode === "viewer" ? $mediaTime : $form.frameAt} />
    <p class="group-note">{$t("Esta é a imaxe que se vai gardar.")}</p>
    <button class="btn primary block" on:click={() => frame($form.frameMode === "viewer" ? $mediaTime : $form.frameAt)} disabled={$busy}>{$t("Gardar este fotograma")}</button>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Extraer audio")}</h4>
    <div class="pair">
      <SelectField label={$t("Formato")} bind:value={$form.audioExt} options={AUDIO_TARGETS.map((x) => ({ value: x, label: x.toUpperCase() }))} def="mp3" />
      <NumberField label={$t("Bitrate de audio")} unit="kbps" min={32} max={320} bind:value={$form.audioKbps} def={192} />
    </div>
    <button class="btn block" on:click={audio} disabled={$busy}>{$t("Extraer o audio")}</button>
  </div>

  <JobStatus {job} />
</div>
