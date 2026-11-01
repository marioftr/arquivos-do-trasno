<script lang="ts">
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { mediaTime, trimRange } from "../../files";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const total = file.duration ?? 0;
  // 0 en «Fin» significa ata o final.
  const form = createForm({ start: 0, end: total });
  onMount(() => useForm(form));

  // O visor móstrase o corte sobre a liña de tempo e as súas asas móvense: sempre sincronizado cos campos.
  const r1 = (v: number) => Math.round(v * 10) / 10;
  $: if (total) {
    const r = { start: $form.start, end: $form.end || total };
    const cur = get(trimRange);
    if (!cur || cur.start !== r.start || cur.end !== r.end) trimRange.set(r);
  }
  onMount(() =>
    trimRange.subscribe((r) => {
      if (!r || !total) return;
      const start = r1(r.start);
      const end = r1(r.end) >= r1(total) ? total : r1(r.end);
      const f = get(form);
      if (f.start !== start || (f.end || total) !== end) form.update((v) => ({ ...v, start, end }));
    }),
  );
  onDestroy(() => trimRange.set(null));

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
    <div class="pair">
      <button class="btn" on:click={() => form.update((v) => ({ ...v, start: Math.min($mediaTime, v.end || total || Infinity) }))}>{$t("Inicio: onde está o visor")}</button>
      <button class="btn" on:click={() => form.update((v) => ({ ...v, end: Math.max($mediaTime, v.start) }))}>{$t("Fin: onde está o visor")}</button>
    </div>
    {#if total}
      <p class="group-note">
        {$t("Duración total: {s} s", { s: total.toFixed(1) })}{#if kept !== null}&nbsp;·&nbsp;{$t("Queda: {s} s", { s: kept.toFixed(1) })}{/if}
      </p>
    {/if}
  </div>

  <button class="btn primary block" on:click={trim} disabled={$busy}>{$t("Gardar o corte")}</button>
  <JobStatus {job} />
</div>
