<script lang="ts">
  import { askPath, askDir } from "./shared";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import Icon from "../Icon.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import TextField from "../ui/TextField.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const form = createForm({ mode: "each" as "each" | "every" | "ranges", every: 2, ranges: "" });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  $: total = file.pages ?? 0;

  /** Cada grupo vai a un PDF distinto. */
  function groups(): number[][] {
    const f = $form;
    if (!total) throw new Error(tr("Aínda non sei cantas páxinas ten o PDF."));
    const all = Array.from({ length: total }, (_, i) => i + 1);
    if (f.mode === "each") return all.map((n) => [n]);
    if (f.mode === "every") {
      const k = Math.max(1, Math.floor(f.every || 1));
      const out: number[][] = [];
      for (let i = 0; i < all.length; i += k) out.push(all.slice(i, i + k));
      return out;
    }
    const out: number[][] = [];
    for (const part of f.ranges.split(";").flatMap((s) => s.split(" ")).filter(Boolean)) {
      const pages: number[] = [];
      for (const piece of part.split(",")) {
        const m = piece.match(/^(\d+)-(\d+)$/);
        if (m) for (let n = Number(m[1]); n <= Number(m[2]); n++) pages.push(n);
        else if (/^\d+$/.test(piece)) pages.push(Number(piece));
        else throw new Error(tr("«{p}» non é unha páxina válida", { p: piece }));
      }
      if (pages.some((n) => n < 1 || n > total)) throw new Error(tr("A páxina {page} non existe: o PDF só ten {total}.", { page: pages.find((n) => n < 1 || n > total)!, total }));
      if (pages.length) out.push(pages);
    }
    if (!out.length) throw new Error(tr("Indica polo menos un intervalo."));
    return out;
  }

  $: preview = (() => {
    try {
      return groups().length;
    } catch {
      return null;
    }
  })();

  async function run() {
    let list: number[][];
    try {
      list = groups();
    } catch (e) {
      job.say(false, errText(e));
      return;
    }
    const dir = await askDir(file.path);
    if (!dir) return;
    const dot = file.name.lastIndexOf(".");
    const base = dot > 0 ? file.name.slice(0, dot) : file.name;
    job.busy.set(true);
    job.clear();
    try {
      for (let i = 0; i < list.length; i++) {
        job.text.set(tr("O trasno está cortando o PDF: parte {n} de {total}…", { n: i + 1, total: list.length }));
        const out = await join(dir, `${base}-${tr("parte")}-${String(i + 1).padStart(String(list.length).length, "0")}.pdf`);
        await engine.pdfExtractPages(file.path, list[i], out, false);
      }
      job.say(true, `${listo()} ${tr("O trasno cortou o PDF en {n} partes en {dir}.", { n: list.length, dir: fileNameOf(dir) })}`);
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    } finally {
      job.busy.set(false);
    }
  }
</script>

<div class="stack">
  <FormBar {form} />
  <Segmented
    options={[
      { value: "each", label: $t("Cada páxina") },
      { value: "every", label: $t("Cada N páxinas") },
      { value: "ranges", label: $t("Intervalos") },
    ]}
    bind:value={$form.mode}
  />
  <div class="group">
    {#if $form.mode === "each"}
      <p class="group-note">{$t("Sairá un PDF por cada páxina.")}</p>
    {:else if $form.mode === "every"}
      <NumberField label={$t("Páxinas por parte")} min={1} bind:value={$form.every} def={2} />
    {:else}
      <TextField label={$t("Intervalos")} placeholder={$t("ex: 1-3; 4-6; 7")} bind:value={$form.ranges} hint={$t("Separa cada parte con «;». Cada parte vai a un PDF.")} />
    {/if}
    {#if preview !== null}<p class="group-note">{$t("Resultado: {n} PDF", { n: preview })} · {$t("O PDF ten {n} páxinas.", { n: total })}</p>{/if}
  </div>
  <button class="btn primary block" on:click={run} disabled={$busy || preview === null}>{$t("Dividir o PDF")}</button>
  <JobStatus {job} />
</div>
