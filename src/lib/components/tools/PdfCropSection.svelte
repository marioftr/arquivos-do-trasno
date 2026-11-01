<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import { pdfOverlay, type FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput, safePages, parsePageSpec } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import TextField from "../ui/TextField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const PT_PER_MM = 72 / 25.4;
  const form = createForm({ left: 0, right: 0, top: 0, bottom: 0, same: false, pageSpec: "" });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  $: f = $form;
  // «Todos igual»: un só valor para os catro márxenes.
  $: if (f.same && (f.right !== f.left || f.top !== f.left || f.bottom !== f.left)) {
    form.update((v) => ({ ...v, right: v.left, top: v.left, bottom: v.left }));
  }
  $: pages = safePages(f.pageSpec, file.pages);
  $: pdfOverlay.set({ crop: { l: f.left * PT_PER_MM, r: f.right * PT_PER_MM, t: f.top * PT_PER_MM, b: f.bottom * PT_PER_MM }, select: pages ?? undefined });
  onDestroy(() => pdfOverlay.set(null));

  async function apply() {
    try {
      const sel = parsePageSpec(f.pageSpec, file.pages);
      const out = await askOutput(file, `-${tr("recortado")}`, "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const done = await engine.pdfCrop(
          file.path,
          out,
          { left: f.left * PT_PER_MM, bottom: f.bottom * PT_PER_MM, right: f.right * PT_PER_MM, top: f.top * PT_PER_MM },
          sel,
          true,
        );
        return `${listo()} ${fileNameOf(done)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Márxenes que se quitan")}</h4>
    <p class="group-note">{$t("As zonas escuras do visor son o que se vai cortar.")}</p>
    <Toggle label={$t("Os catro márxenes iguais")} bind:checked={$form.same} />
    <Slider label={$t("Esquerda")} unit="mm" min={0} max={100} step={0.5} bind:value={$form.left} def={0} />
    {#if !f.same}
      <Slider label={$t("Dereita")} unit="mm" min={0} max={100} step={0.5} bind:value={$form.right} def={0} />
      <Slider label={$t("Arriba")} unit="mm" min={0} max={100} step={0.5} bind:value={$form.top} def={0} />
      <Slider label={$t("Abaixo")} unit="mm" min={0} max={100} step={0.5} bind:value={$form.bottom} def={0} />
    {/if}
    <TextField label={$t("Páxinas (baleiro = todas)")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} />
  </div>
  <button class="btn primary block" on:click={apply} disabled={$busy || (f.left + f.right + f.top + f.bottom === 0)}>{$t("Gardar unha copia recortada")}</button>
  <JobStatus {job} />
</div>
