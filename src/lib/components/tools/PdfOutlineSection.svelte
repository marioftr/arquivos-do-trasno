<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { errText, isCancelled } from "../../errors";
  import { jumpToPdfPage, type FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { openPdf, readPages, readOutline, detectHeadings } from "../../pdfText";
  import { askOutput } from "./shared";
  import Icon from "../Icon.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Índice (marcadores) dun PDF: o que xa ten, editable; ou os títulos que se detectan polo tamaño da letra. */
  export let file: FileEntry;

  const form = createForm({ entries: [] as engine.OutlineEntry[] }, undefined, "pdfindice|" + file.path);
  onMount(() => useForm(form));
  $: entries = $form.entries;

  const job = createJob();
  const busy = job.busy;
  let total = file.pages ?? 0;
  let hadOutline = false;
  let selected = -1;
  let loaded = false;

  // Ao abrir, o índice que xa leva o PDF (se non se tocou antes nesta sesión).
  onMount(async () => {
    try {
      const doc = await openPdf(file.path);
      try {
        total = doc.numPages;
        const current = await readOutline(doc);
        hadOutline = current.length > 0;
        if (!$form.entries.length && current.length) form.restore({ entries: current });
      } finally {
        await doc.destroy();
      }
    } catch {
      // sen índice que ler: lista baleira
    }
    loaded = true;
  });
  onDestroy(() => job.clear());

  const mutate = (fn: (list: engine.OutlineEntry[]) => engine.OutlineEntry[]) => form.update((f) => ({ ...f, entries: fn(f.entries.map((e) => ({ ...e }))) }));
  function indent(i: number, d: number) {
    mutate((l) => {
      l[i].level = Math.max(0, Math.min(l[i].level + d, i === 0 ? 0 : l[i - 1].level + 1));
      // Os fillos seguen ao pai.
      for (let k = i + 1; k < l.length && l[k].level > l[i].level - d; k++) l[k].level = Math.max(0, l[k].level + d);
      return l;
    });
  }
  const del = (i: number) => mutate((l) => l.filter((_, k) => k !== i));
  function addAfter() {
    const at = selected >= 0 && selected < entries.length ? selected : entries.length - 1;
    const ref = entries[at];
    mutate((l) => {
      l.splice(at + 1, 0, { title: "", page: ref?.page ?? 1, level: ref?.level ?? 0 });
      return l;
    });
    selected = at + 1;
  }

  async function detect() {
    try {
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const doc = await openPdf(file.path);
        try {
          const all = Array.from({ length: doc.numPages }, (_, i) => i + 1);
          const pages = await readPages(doc, all, (n) => job.progress.set(n / all.length));
          const found = detectHeadings(pages);
          if (!found.length) return tr("Non atopei títulos: a letra é igual en todo o texto. Engádeos á man.");
          form.update((f) => ({ ...f, entries: found }));
          return tr("Títulos atopados: {n}. Revísaos antes de gardar.", { n: found.length });
        } finally {
          await doc.destroy();
        }
      });
    } catch (e) {
      if (!isCancelled(e)) job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }

  $: bad = entries.some((e) => !e.title.trim() || !(e.page >= 1 && (!total || e.page <= total)));
  async function save() {
    try {
      const out = await askOutput(file, entries.length ? tr("-con-indice") : tr("-sen-indice"), "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const done = await engine.pdfSetOutline(file.path, out, entries.map((e) => ({ title: e.title.trim(), page: Math.round(e.page), level: e.level })), true);
        return `${listo()} ${fileNameOf(done)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <PanelInfo text={$t("Son os marcadores do panel lateral do lector: cada entrada leva a unha páxina. As frechas cambian o nivel.")} />
    <div class="estimate">
      <button class="btn" on:click={detect} disabled={$busy}>{$t("Detectar títulos")}</button>
    </div>
    {#if entries.length}
      <ul class="outline">
        <!-- A cabeceira vai dentro da lista (pegada arriba): así as columnas coinciden aínda que haxa barra de desprazamento. -->
        <li class="ohead"><span>{$t("Nivel")}</span><span class="tt">{$t("Título")}</span><span>{$t("Páxina")}</span><span></span></li>
        {#each entries as e, i}
          <li class:sel={selected === i}>
            <span class="lv">
              <button class="btn ghost icon" on:click={() => indent(i, -1)} disabled={e.level === 0} title={$t("Subir un nivel")} aria-label={`${$t("Subir un nivel")}: ${e.title}`}><Icon name="back" size={13} /></button>
              <button class="btn ghost icon" on:click={() => indent(i, 1)} disabled={i === 0 || e.level > entries[i - 1].level} title={$t("Baixar un nivel")} aria-label={`${$t("Baixar un nivel")}: ${e.title}`}><Icon name="chevron-right" size={13} /></button>
            </span>
            <input class="otitle" style={`margin-left:${e.level * 0.8}rem`} bind:value={$form.entries[i].title} on:focus={() => (selected = i)} placeholder={$t("Título")} aria-label={`${$t("Título")} ${i + 1}`} />
            <input class="opage" type="number" min="1" max={total || undefined} bind:value={$form.entries[i].page} on:focus={() => ((selected = i), jumpToPdfPage(e.page))} on:change={() => jumpToPdfPage(e.page)} aria-label={`${$t("Páxina")} ${i + 1}`} />
            <button class="btn ghost icon" on:click={() => del(i)} title={$t("Quitar esta entrada")} aria-label={`${$t("Quitar esta entrada")}: ${e.title}`}><Icon name="trash" size={13} /></button>
          </li>
        {/each}
      </ul>
    {:else if loaded}
      <p class="muted">{hadOutline ? $t("O índice está baleiro: ao gardar quítase.") : $t("Este PDF non ten índice.")}</p>
    {/if}
    <div class="pair">
      <button class="btn" on:click={addAfter} disabled={$busy}><Icon name="plus" size={14} /> {$t("Engadir entrada")}</button>
      <button class="btn" on:click={() => form.update((f) => ({ ...f, entries: [] }))} disabled={$busy || !entries.length}>{$t("Baleirar")}</button>
    </div>
  </div>

  <button class="btn primary block" on:click={save} disabled={$busy || bad || (!entries.length && !hadOutline)}>
    {entries.length ? $t("Gardar copia co índice") : $t("Gardar copia sen índice")}
  </button>
  <JobStatus {job} />
</div>

<style>
  .outline { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; max-height: 22rem; overflow: auto; }
  .outline li { display: grid; grid-template-columns: 52px 1fr 4.4rem 26px; align-items: center; gap: 3px; }
  .ohead { position: sticky; top: 0; z-index: 1; background: var(--bg); font-size: var(--fs-xs); color: var(--text-faint); text-transform: uppercase; letter-spacing: 0.04em; }
  .ohead span { text-align: center; white-space: nowrap; overflow: hidden; }
  .ohead .tt { text-align: left; padding-left: 0.5em; }
  .lv { display: flex; }
  .outline li.sel .otitle { border-color: var(--accent); }
  .otitle, .opage { height: var(--control-h); background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0 0.5em; font-family: inherit; font-size: var(--fs-md); min-width: 0; }
  .opage { text-align: center; }
  .otitle:focus, .opage:focus { outline: none; border-color: var(--accent); }
</style>
