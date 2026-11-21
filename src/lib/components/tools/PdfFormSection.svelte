<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Segmented from "../ui/Segmented.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import TextField from "../ui/TextField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Encher un formulario PDF: cada campo co seu tipo (texto, casilla, opcións, lista). Pódese deixar editable ou aplanar. */
  export let file: FileEntry;

  const form = createForm({ flatten: false }, "pdf-formulario", "pdfform|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;
  let fields: engine.FormField[] = [];
  let vals: Record<string, string> = {};
  let checks: Record<string, boolean> = {};
  let loaded = false;
  let loadError = "";
  onDestroy(() => job.clear());

  onMount(async () => {
    try {
      fields = (await engine.pdfFormFields(file.path)).sort((a, b) => a.page - b.page || b.rect[3] - a.rect[3] || a.rect[0] - b.rect[0]);
      vals = Object.fromEntries(fields.map((f) => [f.name, f.value]));
      checks = Object.fromEntries(fields.filter((f) => f.kind === "check").map((f) => [f.name, f.value === "true"]));
    } catch (e) {
      loadError = errText(e);
    }
    loaded = true;
  });

  $: pages = [...new Set(fields.map((f) => f.page))];
  const current = (f: engine.FormField) => (f.kind === "check" ? String(!!checks[f.name]) : (vals[f.name] ?? ""));
  const countChanged = (fs: engine.FormField[], v: Record<string, string>, c: Record<string, boolean>) => fs.filter((f) => !f.read_only && (f.kind === "check" ? String(!!c[f.name]) : (v[f.name] ?? "")) !== f.value).length;
  $: changed = countChanged(fields, vals, checks);

  async function save() {
    try {
      const out = await askOutput(file, $form.flatten ? tr("-aplanado") : tr("-cheo"), "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const values = fields.filter((f) => !f.read_only).map((f) => ({ name: f.name, value: current(f) }));
        const r = await engine.pdfFillForm(file.path, out, values, $form.flatten, true);
        return `${listo()} ${fileNameOf(r.output_path)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  {#if !loaded}
    <p class="muted">{$t("Lendo os campos…")}</p>
  {:else if loadError}
    <p class="muted">{loadError}</p>
  {:else if !fields.length}
    <p class="muted">{$t("Este PDF non ten campos de formulario.")}</p>
  {:else}
    {#each pages as pg}
      <div class="group">
        {#if pages.length > 1}<h4 class="group-title">{$t("Páxina {n}", { n: pg })}</h4>{/if}
        {#each fields.filter((f) => f.page === pg) as f (f.name)}
          {#if f.kind === "text" && f.multiline}
            <label class="area">
              <span class="lab">{f.name}</span>
              <textarea rows="3" bind:value={vals[f.name]} disabled={f.read_only} aria-label={f.name}></textarea>
            </label>
          {:else if f.kind === "text"}
            <TextField label={f.name} bind:value={vals[f.name]} disabled={f.read_only} />
          {:else if f.kind === "check"}
            <Toggle label={f.name} bind:checked={checks[f.name]} disabled={f.read_only} />
          {:else if f.kind === "radio"}
            <Segmented label={f.name} options={f.options.map((o) => ({ value: o.value, label: o.label }))} bind:value={vals[f.name]} />
          {:else}
            <SelectField label={f.name} bind:value={vals[f.name]} options={[{ value: "", label: "" }, ...f.options.map((o) => ({ value: o.value, label: o.label }))]} disabled={f.read_only} />
          {/if}
        {/each}
      </div>
    {/each}

    <div class="group">
      <Toggle label={$t("Aplanar formulario")} bind:checked={$form.flatten} hint={$t("O que se escribiu pasa a ser parte da páxina e os campos deixan de ser editables.")} />
    </div>

    <button class="btn primary block" on:click={save} disabled={$busy || (!changed && !$form.flatten)}>
      {$form.flatten ? $t("Gardar copia aplanada") : $t("Gardar copia chea")}
    </button>
  {/if}
  <JobStatus {job} />
</div>

<style>
  .area { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { font-size: var(--fs-sm); color: var(--text-muted); }
  textarea { background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0.4em 0.65em; font-family: inherit; font-size: var(--fs-md); resize: vertical; }
  textarea:focus { outline: none; border-color: var(--accent); }
</style>
