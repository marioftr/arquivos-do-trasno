<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { claim } from "../../claim";
  import * as engine from "../../engine";
  import { errText, isCancelled } from "../../errors";
  import { pdfOverlay, type FileEntry, type RedactBox } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { openPdf, readPages, findInPages, type FindKind } from "../../pdfText";
  import { askOutput } from "./shared";
  import Icon from "../Icon.svelte";
  import Levels from "../ui/Levels.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Slider from "../ui/Slider.svelte";
  import TextField from "../ui/TextField.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Tapar de verdade: as zonas que se debuxan no visor (ou que atopa unha busca) quedan en negro e o que había debaixo
   *  desaparece do ficheiro. As páxinas tapadas pasan a ser unha imaxe. */
  export let file: FileEntry;

  const LEVELS = [
    { id: "rapida", label: "Rápida", detail: "150 ppp", values: { dpi: 150 } },
    { id: "normal", label: "Normal", detail: "200 ppp", values: { dpi: 200 } },
    { id: "alta", label: "Alta", detail: "300 ppp", values: { dpi: 300 } },
  ];
  const KINDS: { value: FindKind; label: string }[] = [
    { value: "text", label: "Un texto" },
    { value: "email", label: "Correos electrónicos" },
    { value: "phone", label: "Teléfonos" },
    { value: "iban", label: "IBAN" },
    { value: "dni", label: "DNI ou NIE" },
  ];

  const form = createForm({ dpi: 200, kind: "email" as FindKind, query: "" }, "pdf-tapar", "pdftapar|" + file.path);
  onMount(() => useForm(form));
  $: v = $form;

  const job = createJob();
  const busy = job.busy;

  let boxes: RedactBox[] = [];
  let nextId = 1;
  const add = (page: number, x: number, y: number, w: number, h: number) => {
    boxes = [...boxes, { id: nextId++, page, x, y, w, h }];
  };
  const remove = (id: number) => (boxes = boxes.filter((b) => b.id !== id));

  const own = claim("pdfOverlay");
  $: pdfOverlay.set({ redact: { boxes, onAdd: add, onRemove: remove } });
  onDestroy(() => {
    job.clear();
    own.release(() => pdfOverlay.set(null));
  });

  $: byPage = [...boxes.reduce((m, b) => m.set(b.page, (m.get(b.page) ?? 0) + 1), new Map<number, number>())].sort((a, b) => a[0] - b[0]);

  async function search() {
    try {
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const doc = await openPdf(file.path);
        try {
          const all = Array.from({ length: doc.numPages }, (_, i) => i + 1);
          const pages = await readPages(doc, all, (n) => job.progress.set(n / all.length));
          const found = findInPages(pages, v.kind, v.query);
          // Sen repetir unha zona que xa está marcada.
          const fresh = found.filter((f) => !boxes.some((b) => b.page === f.page && Math.abs(b.x - f.x) < 0.3 && Math.abs(b.y - f.y) < 0.3 && Math.abs(b.w - f.w) < 0.3));
          for (const f of fresh) add(f.page, f.x, f.y, f.w, f.h);
          const pagesHit = new Set(fresh.map((f) => f.page)).size;
          if (!found.length) return tr("Non atopei nada que coincida. Se o PDF é unha dixitalización, primeiro hai que lelo en Recoñecer texto (OCR).");
          return tr("Zonas marcadas: {n}, en {pages} páxinas. Revísaas no visor antes de tapar.", { n: fresh.length, pages: pagesHit });
        } finally {
          await doc.destroy();
        }
      });
    } catch (e) {
      if (!isCancelled(e)) job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }

  async function run() {
    try {
      const out = await askOutput(file, tr("-tapado"), "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const doc = await openPdf(file.path);
        try {
          const pages = byPage.map(([n]) => n);
          const done: { page: number; jpeg: Uint8Array }[] = [];
          for (const n of pages) {
            if (await engine.jobCancelled()) throw new Error("Cancelei o traballo.");
            job.text.set(tr("Estou preparando a páxina {n}…", { n }));
            job.progress.set(done.length / pages.length);
            const page = await doc.getPage(n);
            const base = page.getViewport({ scale: 1 });
            const scale = Math.min(v.dpi / 72, 6000 / Math.max(base.width, base.height));
            const vp = page.getViewport({ scale });
            const canvas = document.createElement("canvas");
            canvas.width = Math.max(1, Math.round(vp.width));
            canvas.height = Math.max(1, Math.round(vp.height));
            const ctx = canvas.getContext("2d")!;
            ctx.fillStyle = "#fff";
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            await page.render({ canvasContext: ctx, viewport: vp }).promise;
            ctx.fillStyle = "#000";
            for (const b of boxes.filter((x) => x.page === n)) {
              ctx.fillRect(Math.floor((b.x / 100) * canvas.width), Math.floor((b.y / 100) * canvas.height), Math.ceil((b.w / 100) * canvas.width), Math.ceil((b.h / 100) * canvas.height));
            }
            const blob: Blob = await new Promise((ok, ko) => canvas.toBlob((x) => (x ? ok(x) : ko(new Error(tr("Non puiden debuxar a páxina")))), "image/jpeg", 0.92));
            canvas.width = canvas.height = 0;
            page.cleanup();
            done.push({ page: n, jpeg: new Uint8Array(await blob.arrayBuffer()) });
          }
          job.text.set(tr("Estou gardando o PDF…"));
          const r = await engine.pdfRedact(file.path, out, done, true);
          return `${listo()} ${fileNameOf(r.output_path)}\n${tr("Páxinas convertidas en imaxe: {n}. O que había debaixo das zonas xa non está no ficheiro.", { n: r.pages })}`;
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
    <h4 class="group-title">{$t("Zonas a tapar")}</h4>
    <GroupNote text={$t("Arrastra sobre as páxinas para debuxar cada zona. As páxinas tapadas pasan a ser imaxe: o texto de debaixo desaparece.")} />
    {#if byPage.length}
      <ul class="results">
        {#each byPage as [page, n]}
          <li class="zrow">
            <span class="rname">{$t("Páxina {n}", { n: page })}</span>
            <span class="rstate">
              {$t("Zonas: {n}", { n })}
              <button class="btn ghost icon" title={$t("Quitar as zonas da páxina")} aria-label={`${$t("Quitar as zonas da páxina")} ${page}`} on:click={() => (boxes = boxes.filter((b) => b.page !== page))}><Icon name="trash" size={14} /></button>
            </span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Buscar e marcar")}</h4>
    <SelectField label={$t("Buscar")} bind:value={$form.kind} options={KINDS.map((k) => ({ value: k.value, label: $t(k.label) }))} disabled={$busy} hint={$t("Marca todas as coincidencias no texto do PDF; revísaas antes de tapar.")} />
    {#if v.kind === "text"}
      <TextField label={$t("Texto")} bind:value={$form.query} placeholder={$t("ex: María Pérez")} />
    {/if}
    <button class="btn block" on:click={search} disabled={$busy || (v.kind === "text" && !v.query.trim())}><Icon name="search" size={14} /> {$t("Marcar coincidencias")}</button>
  </div>

  <div class="group">
    <Levels
      label={$t("Calidade das páxinas tapadas")}
      info={$t("Resolución á que se debuxan as páxinas tapadas. Máis ppp: máis nitidez e máis peso.")}
      levels={LEVELS.map((l) => ({ ...l, label: $t(l.label), detail: $t(l.detail) }))}
      values={{ dpi: v.dpi }}
      on:apply={(e) => form.update((f) => ({ ...f, ...e.detail }))}
    />
    <Slider label={$t("Resolución")} unit={$t("ppp")} min={100} max={400} step={10} bind:value={$form.dpi} def={200} />
  </div>

  <button class="btn primary block" on:click={run} disabled={$busy || !boxes.length}>
    {boxes.length ? $t("Tapar as zonas ({n})", { n: boxes.length }) : $t("Tapar as zonas")}
  </button>
  <JobStatus {job} />
</div>

<style>
  .zrow { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; }
  .zrow .rstate { display: inline-flex; align-items: center; gap: 0.3rem; }
</style>
