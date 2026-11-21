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
  import JobStatus from "../ui/JobStatus.svelte";
  import FramePreview from "../ui/FramePreview.svelte";

  export let file: FileEntry;
  /** «frame»: gardar un fotograma como imaxe; «audio»: sacar o son a un ficheiro. */
  export let mode: "frame" | "audio" = "frame";

  const form = createForm({ frameAt: 0, audioExt: "mp3", audioKbps: 192 }, undefined, "extract|" + file.path);
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
  {#if mode === "frame"}
    <div class="group">
      <NumberField label={$t("Instante")} unit="s" min={0} step={0.1} bind:value={$form.frameAt} def={0} />
      <div class="estimate">
        <button class="btn" on:click={() => ($form.frameAt = Math.round($mediaTime * 10) / 10)}>{$t("Fotograma do visor")}</button>
      </div>
      <FramePreview path={file.path} time={$form.frameAt ?? 0} />
    </div>
    <button class="btn primary block" on:click={() => frame($form.frameAt ?? 0)} disabled={$busy}>{$t("Gardar fotograma")}</button>
  {:else}
    <div class="group">
      <div class="pair">
        <SelectField label={$t("Formato")} bind:value={$form.audioExt} options={AUDIO_TARGETS.map((x) => ({ value: x, label: x.toUpperCase() }))} def="mp3" />
        <NumberField label={$t("Bitrate de audio")} unit="kbps" min={32} max={320} bind:value={$form.audioKbps} def={192} />
      </div>
    </div>
    <button class="btn primary block" on:click={audio} disabled={$busy}>{$t("Extraer audio")}</button>
  {/if}

  <JobStatus {job} />
</div>
