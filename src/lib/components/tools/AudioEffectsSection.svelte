<script lang="ts">
  import { onMount } from "svelte";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const form = createForm({ fade: 2, echo: 300, bass: 6, treble: 6 }, "audio-efectos");
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  async function apply(suffix: string, op: engine.MediaOp, value: number) {
    try {
      const out = await askOutput(file, tr(suffix), file.ext);
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => `${listo()} ${fileNameOf(await engine.mediaEdit(file.path, out, op, value, false, true))}`);
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <FormBar {form} />

  <div class="group">
    <h4 class="group-title">{$t("Son")}</h4>
    <div class="pair">
      <button class="btn" on:click={() => apply("-normalizado", "normalize", 0)} disabled={$busy} title={$t("Iguala o volume percibido")}>{$t("Normalizar")}</button>
      <button class="btn" on:click={() => apply("-limpo", "denoise", 0)} disabled={$busy} title={$t("Reduce o ruído de fondo")}>{$t("Quitar ruído")}</button>
    </div>
    <button class="btn block" on:click={() => apply("-do-revés", "reverse", 0)} disabled={$busy}>{$t("Dar a volta ao son")}</button>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Esvaecer")}</h4>
    <Slider label={$t("Duración")} unit="s" min={0.1} max={30} step={0.1} bind:value={$form.fade} def={2} />
    <div class="pair">
      <button class="btn" on:click={() => apply("-entrada", "fadein", $form.fade)} disabled={$busy}>{$t("Ao principio")}</button>
      <button class="btn" on:click={() => apply("-saída", "fadeout", $form.fade)} disabled={$busy}>{$t("Ao final")}</button>
    </div>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Ecualizador")}</h4>
    <Slider label={$t("Graves")} unit="dB" min={-12} max={12} step={1} bind:value={$form.bass} def={6} />
    <button class="btn block" on:click={() => apply("-graves", "bass", $form.bass)} disabled={$busy}>{$t("Aplicar graves")}</button>
    <Slider label={$t("Agudos")} unit="dB" min={-12} max={12} step={1} bind:value={$form.treble} def={6} />
    <button class="btn block" on:click={() => apply("-agudos", "treble", $form.treble)} disabled={$busy}>{$t("Aplicar agudos")}</button>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Eco")}</h4>
    <Slider label={$t("Atraso")} unit="ms" min={20} max={2000} step={10} bind:value={$form.echo} def={300} />
    <button class="btn block" on:click={() => apply("-eco", "echo", $form.echo)} disabled={$busy}>{$t("Aplicar eco")}</button>
  </div>

  <JobStatus {job} />
</div>
