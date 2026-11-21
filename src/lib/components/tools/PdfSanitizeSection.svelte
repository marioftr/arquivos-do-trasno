<script lang="ts">
  import { onMount } from "svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** PDF seguro: quita do ficheiro o que se pode compartir sen querer (datos do autor, scripts, anexos, ligazóns, comentarios). */
  export let file: FileEntry;

  const form = createForm({ metadata: true, scripts: true, attachments: true, links: false, comments: false }, "pdf-limpar", "pdflimpar|" + file.path);
  onMount(() => useForm(form));
  $: v = $form;

  const job = createJob();
  const busy = job.busy;
  let report: engine.SanitizeReport | null = null;

  const ROWS: { key: keyof engine.SanitizeOptions; label: string }[] = [
    { key: "metadata", label: "Datos do documento" },
    { key: "scripts", label: "Scripts e accións automáticas" },
    { key: "attachments", label: "Ficheiros anexos" },
    { key: "links", label: "Ligazóns externas" },
    { key: "comments", label: "Comentarios e notas" },
  ];

  async function run() {
    try {
      const out = await askOutput(file, tr("-limpo"), "pdf");
      if (!out) return;
      report = null;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.pdfSanitize(file.path, out, { metadata: v.metadata, scripts: v.scripts, attachments: v.attachments, links: v.links, comments: v.comments }, true);
        report = r;
        return `${listo()} ${fileNameOf(r.output_path)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
  $: any = v.metadata || v.scripts || v.attachments || v.links || v.comments;
</script>

<div class="stack">
  <div class="group">
    <Toggle label={$t("Datos do documento")} bind:checked={$form.metadata} hint={$t("Autor, título, programa que o creou, datas e a información XMP.")} />
    <Toggle label={$t("Scripts e accións automáticas")} bind:checked={$form.scripts} hint={$t("JavaScript e accións que se executan ao abrir o PDF ou ao premer un botón.")} />
    <Toggle label={$t("Ficheiros anexos")} bind:checked={$form.attachments} hint={$t("Ficheiros metidos dentro do PDF que non se ven nas páxinas.")} />
    <Toggle label={$t("Ligazóns externas")} bind:checked={$form.links} hint={$t("Quita as ligazóns a páxinas web e a outros ficheiros; as internas quedan.")} />
    <Toggle label={$t("Comentarios e notas")} bind:checked={$form.comments} hint={$t("Notas, resaltados e demais anotacións de revisión.")} />
  </div>

  <button class="btn primary block" on:click={run} disabled={$busy || !any}>{$t("Limpar PDF")}</button>

  {#if report}
    <ul class="results">
      {#each ROWS.filter((r) => v[r.key]) as r}
        <li class={report[r.key] ? "feito" : ""}>
          <span class="rname">{$t(r.label)}</span>
          <span class="rstate">{report[r.key] ? $t("quitados: {n}", { n: report[r.key] }) : $t("non había")}</span>
        </li>
      {/each}
    </ul>
  {/if}
  <JobStatus {job} />
</div>
