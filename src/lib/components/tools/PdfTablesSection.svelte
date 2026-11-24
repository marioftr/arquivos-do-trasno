<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { onMount, onDestroy } from "svelte";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import { errText, isCancelled } from "../../errors";
  import { jumpToPdfPage, type FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { openPdf, readPages, findTables, csvOf, type Table } from "../../pdfText";
  import { askOutput, askDir, parsePageSpec } from "./shared";
  import Segmented from "../ui/Segmented.svelte";
  import TextField from "../ui/TextField.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Táboas dun PDF a CSV (ábrense en Excel, Calc ou Google Sheets). O texto sae do propio PDF (non é OCR). */
  export let file: FileEntry;

  const form = createForm({ sep: ";", pageSpec: "" }, "pdf-taboas", "pdftaboas|" + file.path);
  onMount(() => useForm(form));
  $: v = $form;

  const job = createJob();
  const busy = job.busy;
  let tables: Table[] = [];
  let on: boolean[] = [];
  let searched = false;
  onDestroy(() => job.clear());

  async function find() {
    try {
      searched = false;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const doc = await openPdf(file.path);
        try {
          const wanted = parsePageSpec(v.pageSpec, doc.numPages) ?? Array.from({ length: doc.numPages }, (_, i) => i + 1);
          const pages = await readPages(doc, wanted, (n) => job.progress.set(n / wanted.length));
          tables = pages.flatMap((p) => findTables(p).tables);
          on = tables.map(() => true);
          searched = true;
          return tables.length
            ? tr("Táboas atopadas: {n}. Marca as que queres gardar.", { n: tables.length })
            : tr("Non atopei táboas. Se o PDF é unha dixitalización, primeiro hai que lelo en Recoñecer texto (OCR).");
        } finally {
          await doc.destroy();
        }
      });
    } catch (e) {
      if (!isCancelled(e)) job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }

  $: chosen = tables.filter((_, i) => on[i]);
  const bom = "﻿"; // sen isto Excel le mal os acentos
  const enc = (rows: string[][]) => new TextEncoder().encode(bom + csvOf(rows, v.sep === "tab" ? "\t" : v.sep));

  async function save() {
    try {
      if (chosen.length === 1) {
        const out = await askOutput(file, tr("-taboa"), "csv");
        if (!out) return;
        await job.run(TRASNO.traballando, fallo("editando"), async () => {
          await engine.saveBytes(out, enc(chosen[0].rows));
          return `${listo()} ${fileNameOf(out)}`;
        });
      } else {
        const dir = await askDir(file.path);
        if (!dir) return;
        const stem = file.name.replace(/\.[^.]+$/, "");
        await job.run(TRASNO.traballando, fallo("editando"), async () => {
          for (let i = 0; i < chosen.length; i++) {
            const out = await engine.uniquePath(await join(dir, `${stem}${tr("-taboa")}-${i + 1}.csv`));
            await engine.saveBytes(out, enc(chosen[i].rows));
          }
          return `${listo()} ${tr("Deixei {n} ficheiros CSV en {dir}.", { n: chosen.length, dir: fileNameOf(dir) })}`;
        });
      }
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <PanelInfo text={$t("Atopa táboas polas columnas do texto. Funciona mellor con táboas sen celas fusionadas.")} />
    <TextField label={$t("Páxinas (baleiro = todas)")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} />
    <div class="estimate">
      <button class="btn" on:click={find} disabled={$busy}>{$t("Buscar táboas")}</button>
    </div>
  </div>

  {#if tables.length}
    <div class="group">
      <h4 class="group-title">{$t("Atopadas")}</h4>
      <ul class="tbls">
        {#each tables as tb, i}
          <li>
            <label class="head">
              <input type="checkbox" bind:checked={on[i]} aria-label={`${$t("Táboa")} ${i + 1}`} />
              <span>{$t("Táboa {n}", { n: i + 1 })}</span>
              <span class="muted"><button type="button" class="pg" on:click|preventDefault={() => jumpToPdfPage(tb.page)} title={$t("Ir á páxina")}>{$t("páxina {p}", { p: tb.page })}</button> · {tb.rows.length} × {tb.rows[0]?.length ?? 0}</span>
            </label>
            <div class="scroll">
              <table>
                <tbody>
                  {#each tb.rows.slice(0, 4) as r}
                    <tr>{#each r as c}<td>{c}</td>{/each}</tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </li>
        {/each}
      </ul>
    </div>
    <Segmented
      label={$t("Separador")}
      info={$t("Excel en galego ou castelán espera punto e coma; o resto de programas, coma.")}
      options={[
        { value: ";", label: $t("Punto e coma") },
        { value: ",", label: $t("Coma") },
        { value: "tab", label: $t("Tabulador") },
      ]}
      bind:value={$form.sep}
    />
    <button class="btn primary block" on:click={save} disabled={$busy || !chosen.length}>
      {chosen.length > 1 ? $t("Gardar {n} táboas en CSV", { n: chosen.length }) : $t("Gardar táboa en CSV")}
    </button>
  {:else if searched}
    <p class="muted">{$t("Non hai táboas que gardar.")}</p>
  {/if}
  <JobStatus {job} />
</div>

<style>
  .tbls { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.7rem; }
  .head { display: flex; align-items: center; gap: 0.5rem; font-size: var(--fs-md); }
  .head .muted { margin-left: auto; font-size: var(--fs-sm); }
  .pg { background: none; border: none; padding: 0; color: inherit; font: inherit; cursor: pointer; }
  .pg:hover { color: var(--accent); text-decoration: underline; }
  .scroll { overflow: auto; margin-top: 0.3rem; border: 1px solid var(--border); border-radius: var(--radius-sm); }
  table { border-collapse: collapse; width: 100%; font-size: var(--fs-sm); }
  td { padding: 0.2rem 0.45rem; border-bottom: 1px solid var(--border); white-space: nowrap; }
  tr:last-child td { border-bottom: none; }
  tr:first-child td { font-weight: var(--fw-strong); background: var(--bg-hover); }
</style>
