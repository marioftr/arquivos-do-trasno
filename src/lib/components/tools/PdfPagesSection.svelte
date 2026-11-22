<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { claim } from "../../claim";
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { pdfOverlay, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput, safePages } from "./shared";
  import TextField from "../ui/TextField.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import PageGrid from "./PageGrid.svelte";

  export let file: FileEntry;
  /** Extraer páxinas a un PDF novo, escollendo nas miniaturas ou polos números. */
  const form = createForm({ pageSpec: "" }, undefined, "pdfpages|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();

  // Vista previa: páxinas escollidas e xiro, antes de gardar nada.
  // O número de páxinas pode non vir co ficheiro: tómase das miniaturas.
  let gridTotal = 0;
  $: total = file.pages ?? (gridTotal || undefined);
  $: sel = safePages($form.pageSpec, total);
  $: pdfOverlay.set({ select: sel ?? undefined });
  const own = claim("pdfOverlay");
  onDestroy(() => own.release(() => pdfOverlay.set(null)));
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

  /** Marca ou desmarca unha páxina nas miniaturas; o campo de texto recibe o resultado como intervalos. */
  function togglePage(n: number) {
    const cur = new Set(sel ?? []);
    if (cur.has(n)) cur.delete(n);
    else cur.add(n);
    const list = [...cur].sort((a, b) => a - b);
    const parts: string[] = [];
    for (let i = 0; i < list.length; ) {
      let j = i;
      while (j + 1 < list.length && list[j + 1] === list[j] + 1) j++;
      parts.push(j > i + 1 ? `${list[i]}-${list[j]}` : list.slice(i, j + 1).join(","));
      i = j + 1;
    }
    form.update((v) => ({ ...v, pageSpec: parts.join(",") }));
  }
  const selectAll = () => total && form.update((v) => ({ ...v, pageSpec: `1-${total}` }));
  const selectNone = () => form.update((v) => ({ ...v, pageSpec: "" }));

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
</script>

<div class="stack">

  <div class="group">
    <PanelInfo text={$t("As páxinas escóllense nas miniaturas ou escribindo os números.")} />
    <div class="pair">
      <button class="btn" on:click={selectAll} disabled={!total}>{$t("Todas")}</button>
      <button class="btn" on:click={selectNone}>{$t("Ningunha")}</button>
    </div>
    <PageGrid {file} selected={sel ?? []} on:toggle={(e) => togglePage(e.detail)} on:total={(e) => (gridTotal = e.detail)} />
    <TextField label={$t("Páxinas")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} hint={file.pages ? $t("O PDF ten {n} páxinas.", { n: file.pages }) : ""} />
    <button class="btn block" on:click={extract} disabled={$busy || !$form.pageSpec}>{$t("Extraer a PDF novo")}</button>
  </div>

  <JobStatus {job} />
</div>
