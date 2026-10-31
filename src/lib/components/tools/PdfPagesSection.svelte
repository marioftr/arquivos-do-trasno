<script lang="ts">
  import { errText } from "../../errors";
  import { onMount } from "svelte";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import NumberField from "../ui/NumberField.svelte";
  import TextField from "../ui/TextField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const form = createForm({ pageSpec: "", rotateAll: true, rotatePage: 1, degrees: 90 });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  function parsePages(spec: string): number[] {
    const pages: number[] = [];
    for (const part of spec.split(",")) {
      const p = part.trim();
      if (!p) continue;
      const m = p.match(/^(\d+)-(\d+)$/);
      if (m) {
        const a = Number(m[1]);
        const b = Number(m[2]);
        for (let n = a; a <= b ? n <= b : n >= b; n += a <= b ? 1 : -1) pages.push(n);
      } else if (/^\d+$/.test(p)) pages.push(Number(p));
      else throw new Error(tr("«{p}» non é unha páxina válida", { p }));
    }
    if (!pages.length) throw new Error(tr("Indica algunha páxina."));
    return pages;
  }

  async function extract() {
    try {
      const pages = parsePages($form.pageSpec);
      const out = await askOutput(file, `-${tr("paxinas")}`, "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => `${listo()} ${fileNameOf(await engine.pdfExtractPages(file.path, pages, out, true))}`);
    } catch (e) {
      job.say(false, errText(e));
    }
  }

  async function rotate() {
    try {
      const out = await askOutput(file, `-${tr("rotado")}`, "pdf");
      if (!out) return;
      const f = $form;
      await job.run(TRASNO.traballando, fallo("editando"), async () =>
        `${listo()} ${fileNameOf(await engine.pdfRotate(file.path, f.rotateAll ? null : f.rotatePage, f.degrees, out, true))}`,
      );
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <FormBar {form} />

  <div class="group">
    <h4 class="group-title">{$t("Extraer páxinas")}</h4>
    <TextField label={$t("Páxinas")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} hint={file.pages ? $t("O PDF ten {n} páxinas. Serve tamén para reordenar.", { n: file.pages }) : ""} />
    <button class="btn block" on:click={extract} disabled={$busy || !$form.pageSpec}>{$t("Extraer a un novo PDF")}</button>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Rotar")}</h4>
    <Toggle label={$t("Todas as páxinas")} bind:checked={$form.rotateAll} />
    {#if !$form.rotateAll}
      <NumberField label={$t("Páxina")} min={1} max={file.pages} bind:value={$form.rotatePage} def={1} unit={file.pages ? `/ ${file.pages}` : ""} />
    {/if}
    <Segmented label={$t("Ángulo")} options={[{ value: 90, label: "90°" }, { value: 180, label: "180°" }, { value: 270, label: "270°" }]} bind:value={$form.degrees} />
    <button class="btn block" on:click={rotate} disabled={$busy}>{$t("Rotar a un novo PDF")}</button>
  </div>

  <JobStatus {job} />
</div>
