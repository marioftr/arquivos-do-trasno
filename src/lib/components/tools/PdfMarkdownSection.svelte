<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { errText, isCancelled } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { openPdf, readPages, toMarkdown, findTables } from "../../pdfText";
  import { askOutput, parsePageSpec } from "./shared";
  import GroupNote from "../ui/GroupNote.svelte";
  import TextField from "../ui/TextField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** PDF a Markdown: títulos pola letra, listas, párrafos e táboas. O texto sae do propio PDF (non é OCR). */
  export let file: FileEntry;

  const form = createForm({ tables: true, pageMarks: false, pageSpec: "" }, "pdf-markdown", "pdfmd|" + file.path);
  onMount(() => useForm(form));
  $: v = $form;

  const job = createJob();
  const busy = job.busy;
  let preview = "";
  let stats: { pages: number; tables: number; headings: number; chars: number } | null = null;
  onDestroy(() => job.clear());

  async function run() {
    try {
      const out = await askOutput(file, "", "md");
      if (!out) return;
      preview = "";
      stats = null;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const doc = await openPdf(file.path);
        try {
          const wanted = parsePageSpec(v.pageSpec, doc.numPages) ?? Array.from({ length: doc.numPages }, (_, i) => i + 1);
          const pages = await readPages(doc, wanted, (n) => job.progress.set(n / wanted.length));
          const text = pages.reduce((n, p) => n + p.lines.length, 0);
          if (!text) throw new Error(tr("Non hai texto neste PDF. Se é unha dixitalización, primeiro hai que lelo en Recoñecer texto (OCR)."));
          const md = toMarkdown(pages, { tables: v.tables, pageMarks: v.pageMarks });
          await engine.saveBytes(out, new TextEncoder().encode(md));
          preview = md.split("\n").slice(0, 40).join("\n");
          stats = { pages: pages.length, tables: v.tables ? pages.reduce((n, p) => n + findTables(p).tables.length, 0) : 0, headings: (md.match(/^#{1,6} /gm) ?? []).length, chars: md.length };
          return `${listo()} ${fileNameOf(out)}`;
        } finally {
          await doc.destroy();
        }
      });
    } catch (e) {
      if (!isCancelled(e)) job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Markdown")}</h4>
    <GroupNote text={$t("Detecta títulos pola letra, listas e táboas. Pensado para texto nunha columna; as imaxes non se levan.")} />
    <Toggle label={$t("Detectar táboas")} bind:checked={$form.tables} hint={$t("Convérteas en táboas de Markdown en vez de deixar cada liña solta.")} />
    <Toggle label={$t("Marcar as páxinas")} bind:checked={$form.pageMarks} hint={$t("Engade un comentario invisible onde empeza cada páxina.")} />
    <TextField label={$t("Páxinas (baleiro = todas)")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} />
  </div>

  <button class="btn primary block" on:click={run} disabled={$busy}>{$t("Gardar como Markdown")}</button>

  {#if stats}
    <ul class="results">
      <li><span class="rname">{$t("Páxinas")}</span><span class="rstate">{stats.pages}</span></li>
      <li><span class="rname">{$t("Títulos")}</span><span class="rstate">{stats.headings}</span></li>
      <li><span class="rname">{$t("Táboas")}</span><span class="rstate">{stats.tables}</span></li>
    </ul>
    <pre class="preview selectable">{preview}</pre>
  {/if}
  <JobStatus {job} />
</div>

<style>
  .preview { margin: 0; max-height: 18rem; overflow: auto; padding: 0.6rem 0.7rem; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); font-family: var(--mono, ui-monospace, monospace); font-size: var(--fs-sm); white-space: pre-wrap; word-break: break-word; }
</style>
