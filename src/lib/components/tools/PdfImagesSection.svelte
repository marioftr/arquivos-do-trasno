<script lang="ts">
  import { askPath, askDir } from "./shared";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as pdfjs from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
  import { errText } from "../../errors";
  import { viewUrl, type FileEntry } from "../../files";
  import { writeBytes } from "../../fsio";
  import { fallo, fileNameOf, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import Icon from "../Icon.svelte";
  import Slider from "../ui/Slider.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import TextField from "../ui/TextField.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

  export let file: FileEntry;

  const form = createForm({ format: "png", dpi: 150, quality: 90, pageSpec: "" }, "pdf-imaxes");
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  function parsePages(spec: string, total: number): number[] {
    if (!spec.trim()) return Array.from({ length: total }, (_, i) => i + 1);
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
    const bad = pages.find((n) => n < 1 || n > total);
    if (bad !== undefined) throw new Error(tr("A páxina {page} non existe: o PDF só ten {total}.", { page: bad, total }));
    return pages;
  }

  async function run() {
    const dir = await askDir(file.path);
    if (!dir) return;
    const f = $form;
    job.busy.set(true);
    job.clear();
    let doc: pdfjs.PDFDocumentProxy | null = null;
    try {
      const res = await fetch(await viewUrl(file.path));
      doc = await pdfjs.getDocument({ data: new Uint8Array(await res.arrayBuffer()) }).promise;
      const pages = parsePages(f.pageSpec, doc.numPages);
      const dot = file.name.lastIndexOf(".");
      const base = dot > 0 ? file.name.slice(0, dot) : file.name;
      const ext = f.format === "jpg" ? "jpg" : "png";
      const digits = String(doc.numPages).length;
      for (let i = 0; i < pages.length; i++) {
        job.text.set(tr("O trasno está debuxando a páxina {n} de {total}…", { n: i + 1, total: pages.length }));
        const page = await doc.getPage(pages[i]);
        const viewport = page.getViewport({ scale: f.dpi / 72 });
        const canvas = document.createElement("canvas");
        canvas.width = Math.round(viewport.width);
        canvas.height = Math.round(viewport.height);
        const ctx = canvas.getContext("2d")!;
        ctx.fillStyle = "#fff";
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        await page.render({ canvasContext: ctx, viewport }).promise;
        const blob: Blob = await new Promise((resolve, reject) =>
          canvas.toBlob((b) => (b ? resolve(b) : reject(new Error(tr("O trasno non puido debuxar a páxina")))), ext === "jpg" ? "image/jpeg" : "image/png", f.quality / 100),
        );
        const name = `${base}-${tr("páxina")}-${String(pages[i]).padStart(digits, "0")}.${ext}`;
        await writeBytes(await join(dir, name), file.path, new Uint8Array(await blob.arrayBuffer()), false);
      }
      job.say(true, `${listo()} ${tr("O trasno gardou {n} imaxes en {dir} sen perder nin unha páxina.", { n: pages.length, dir: fileNameOf(dir) })}`);
    } catch (e) {
      job.say(false, `${fallo("convertendo")}\n${errText(e)}`);
    } finally {
      await doc?.destroy();
      job.busy.set(false);
    }
  }
</script>

<div class="stack">

  <div class="group">
    <h4 class="group-title">{$t("Páxinas a imaxes")}</h4>
    <SelectField
      label={$t("Formato das imaxes")}
      bind:value={$form.format}
      options={[{ value: "png", label: "PNG" }, { value: "jpg", label: "JPG" }]}
      def="png"
    />
    <Slider label={$t("Resolución")} unit={$t("ppp")} min={72} max={300} step={6} bind:value={$form.dpi} def={150} hint={$t("Máis ppp = imaxes máis nítidas e máis pesadas")} />
    {#if $form.format === "jpg"}
      <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={90} />
    {/if}
    <TextField label={$t("Páxinas (baleiro = todas)")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} hint={file.pages ? $t("O PDF ten {n} páxinas.", { n: file.pages }) : ""} />
  </div>

  <button class="btn primary block" on:click={run} disabled={$busy}>{$t("Converter a imaxes")}</button>
  <JobStatus {job} />
</div>
