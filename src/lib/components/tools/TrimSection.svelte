<script lang="ts">
  import { errText } from "../../errors";
  import { onMount } from "svelte";
  import { mediaSeek } from "../../files";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const total = file.duration ?? 0;
  // 0 en «Fin» significa ata o final.
  const form = createForm({ start: 0, end: total });
  onMount(() => useForm(form));

  // Ao mover o inicio ou o fin, o visor salta a ese instante para ver onde corta.
  let lastStart = $form.start;
  let lastEnd = $form.end;
  $: if ($form.start !== lastStart) {
    lastStart = $form.start;
    mediaSeek.set({ t: lastStart, stamp: Date.now() });
  }
  $: if ($form.end !== lastEnd) {
    lastEnd = $form.end;
    if (lastEnd) mediaSeek.set({ t: Math.max(0, lastEnd - 0.05), stamp: Date.now() });
  }

  const job = createJob();
  const busy = job.busy;

  $: kept = total ? Math.max(0, ($form.end || total) - $form.start) : null;

  async function trim() {
    try {
      const out = await askOutput(file, `-${tr("corte")}`, file.ext);
      if (!out) return;
      const f = $form;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const end = f.end && (!total || f.end < total) ? f.end : null;
        return `${listo()} ${fileNameOf(await engine.trimMedia(file.path, out, f.start || 0, end, true))}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <FormBar {form} />

  <div class="group">
    {#if total}
      <Slider label={$t("Inicio")} unit="s" min={0} max={total} step={0.1} bind:value={$form.start} def={0} />
      <Slider label={$t("Fin")} unit="s" min={0} max={total} step={0.1} bind:value={$form.end} def={total} />
    {:else}
      <div class="pair">
        <NumberField label={$t("Inicio")} unit="s" min={0} step={0.1} bind:value={$form.start} def={0} />
        <NumberField label={$t("Fin")} unit="s" min={0} step={0.1} placeholder={$t("fin")} bind:value={$form.end} def={0} />
      </div>
    {/if}
    {#if total}
      <p class="group-note">
        {$t("Duración total: {s} s", { s: total.toFixed(1) })}{#if kept !== null} · {$t("Queda: {s} s", { s: kept.toFixed(1) })}{/if}
      </p>
    {/if}
  </div>

  <button class="btn primary block" on:click={trim} disabled={$busy}>{$t("Gardar o corte")}</button>
  <JobStatus {job} />
</div>
