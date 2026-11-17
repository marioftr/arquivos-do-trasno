<script lang="ts">
  import { onMount } from "svelte";
  import * as pdfjs from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
  import { errText, isCancelled } from "../../errors";
  import * as engine from "../../engine";
  import { viewUrl, type FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr, lang as appLang } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { OCR_LANGS, OCR_SIZE, ocrInstalled, ocrDownload, ocrProgress, savedLang, rememberLang, refreshOcr, downloadLang, recognize, forgetOcr } from "../../ocr";
  import { askOutput, parsePageSpec } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import TextField from "../ui/TextField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

  export let file: FileEntry;

  /** Resolución con que se le cada páxina. «Personalizado» deixa poñer os ppp que se queiran. */
  const PROFILES: Record<string, number> = { rapida: 150, normal: 200, alta: 300 };

  const form = createForm({ lang: savedLang($appLang), profile: "normal", dpi: 200, skipText: true, pageSpec: "" }, "pdf-ocr", "pdfocr|" + file.path);
  onMount(() => {
    void refreshOcr();
    return useForm(form);
  });

  const job = createJob();
  const busy = job.busy;

  // Un perfil fixo enche os ppp; con «Personalizado» quedan os que poña o usuario.
  $: preset = PROFILES[$form.profile];
  $: if (preset && $form.dpi !== preset) form.update((f) => ({ ...f, dpi: preset }));
  $: if ($form.lang) rememberLang($form.lang);

  $: installed = $ocrInstalled.includes($form.lang);
  $: langName = OCR_LANGS.find((l) => l.id === $form.lang)?.label ?? $form.lang;

  let downloadErr = "";
  async function download() {
    downloadErr = "";
    try {
      await downloadLang($form.lang);
    } catch (e) {
      downloadErr = errText(e);
    }
  }

  /** Unha páxina xa ten texto de verdade (non basta con restos soltos): as letras e cifras abondan para que se poida buscar. */
  async function hasText(page: pdfjs.PDFPageProxy): Promise<boolean> {
    const c = await page.getTextContent();
    const text = c.items.map((i) => ("str" in i ? i.str : "")).join(" ");
    return (text.match(/[\p{L}\p{N}]/gu) ?? []).length >= 25;
  }

  const MAX_SIDE = 6000;

  async function run() {
    try {
      const out = await askOutput(file, tr("-ocr"), "pdf");
      if (!out) return;
      const f = $form;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        let doc: pdfjs.PDFDocumentProxy | null = null;
        const unsubProgress: (() => void)[] = [];
        try {
          doc = await pdfjs.getDocument({ data: new Uint8Array(await (await fetch(await viewUrl(file.path))).arrayBuffer()) }).promise;
          const wanted = parsePageSpec(f.pageSpec, doc.numPages) ?? Array.from({ length: doc.numPages }, (_, i) => i + 1);
          const layers: { page: number; words: engine.OcrTextWord[] }[] = [];
          let skipped = 0;
          let empty = 0;
          let step = 0;
          for (const n of wanted) {
            if (await engine.jobCancelled()) throw new Error("Cancelouse o traballo.");
            job.text.set(tr("O trasno está lendo a páxina {n} de {total}…", { n: step + 1, total: wanted.length }));
            job.progress.set(step / wanted.length);
            step++;
            const page = await doc.getPage(n);
            if (f.skipText && (await hasText(page))) {
              skipped++;
              continue;
            }
            // Debuxa a páxina á resolución pedida (sen pasar dun tamaño razoable) e lea o texto como calquera imaxe.
            const base = page.getViewport({ scale: 1 });
            const scale = Math.min(f.dpi / 72, MAX_SIDE / Math.max(base.width, base.height));
            const viewport = page.getViewport({ scale });
            const canvas = document.createElement("canvas");
            canvas.width = Math.max(1, Math.round(viewport.width));
            canvas.height = Math.max(1, Math.round(viewport.height));
            const ctx = canvas.getContext("2d")!;
            ctx.fillStyle = "#fff";
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            await page.render({ canvasContext: ctx, viewport }).promise;
            const blob: Blob = await new Promise((ok, ko) => canvas.toBlob((b) => (b ? ok(b) : ko(new Error(tr("O trasno non puido debuxar a páxina")))), "image/png"));
            // Das coordenadas da imaxe lida (píxeles do lenzo) ás do visor (as que entende `convertToPdfPoint`).
            const sx = viewport.width / canvas.width;
            const sy = viewport.height / canvas.height;
            canvas.width = canvas.height = 0;
            const url = URL.createObjectURL(blob);
            const key = `pdfocr:${file.path}:${n}:${Date.now()}`;
            const stop = ocrProgress.subscribe((p) => job.progress.set((step - 1 + p) / wanted.length));
            unsubProgress.push(stop);
            let res;
            try {
              res = await recognize(key, url, f.lang);
            } finally {
              stop();
              URL.revokeObjectURL(url);
              forgetOcr(key);
            }
            const words: engine.OcrTextWord[] = [];
            for (const w of res.lines.flat()) {
              // Esquina de abaixo á esquerda e fin da base, en píxeles da imaxe → coordenadas do PDF (a páxina pode estar xirada).
              const [x0, y0] = viewport.convertToPdfPoint(w.x0 * sx, w.y1 * sy);
              const [x1, y1] = viewport.convertToPdfPoint(w.x1 * sx, w.y1 * sy);
              const len = Math.hypot(x1 - x0, y1 - y0);
              const h = ((w.y1 - w.y0) * sy) / scale;
              if (!(len > 0) || !(h > 0)) continue;
              words.push({ text: w.text, x: x0, y: y0, w: len, h, angle: (Math.atan2(y1 - y0, x1 - x0) * 180) / Math.PI });
            }
            if (words.length) layers.push({ page: n, words });
            else empty++;
            page.cleanup();
          }
          if (!layers.length) {
            throw new Error(
              skipped === wanted.length
                ? tr("Todas as páxinas xa teñen texto: non hai nada que facer. Se queres lelas igual, desactiva «Saltar as páxinas que xa teñen texto».")
                : tr("O trasno non atopou texto en ningunha páxina. Proba con outro idioma ou con máis resolución."),
            );
          }
          job.text.set(tr("O trasno está gardando o PDF…"));
          job.progress.set(0.98);
          if (await engine.jobCancelled()) throw new Error("Cancelouse o traballo.");
          const r = await engine.pdfAddTextLayer(file.path, out, layers, true);
          const extra = [skipped ? tr("{n} xa tiñan texto.", { n: skipped }) : "", empty ? tr("{n} sen texto.", { n: empty }) : ""].filter(Boolean).join(" ");
          return `${listo()} ${fileNameOf(r.output_path)}\n${tr("Engadiuse texto a {pages} páxinas ({words} palabras): agora pódese buscar e copiar.", { pages: r.pages, words: r.words })} ${extra}`.trim();
        } finally {
          unsubProgress.forEach((s) => s());
          await doc?.destroy();
        }
      });
    } catch (e) {
      if (!isCancelled(e)) job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Texto dun PDF dixitalizado")}</h4>
    <GroupNote text={$t("Le as páxinas que son só imaxe (dixitalizacións) e engade o texto por debaixo, invisible: o PDF queda igual de aspecto, pero xa se pode buscar e copiar. Tarda uns segundos por páxina.")} />

    <SelectField
      label={$t("Idioma do texto")}
      bind:value={$form.lang}
      options={OCR_LANGS.map((l) => ({ value: l.id, label: l.label + ($ocrInstalled.includes(l.id) ? "" : " ⇩") }))}
      def={form.defaults.lang}
      disabled={$busy}
    />
    {#if !installed}
      <GroupNote text={$t("Para ler texto en {lang} o trasno ten que descargar o seu paquete de idioma (uns {size}, do proxecto Tesseract en GitHub).", { lang: langName, size: OCR_SIZE })} />
      <button class="btn block" on:click={download} disabled={$busy || !!$ocrDownload}>
        {$ocrDownload ? $t("Descargando… {p} %", { p: Math.round($ocrDownload.progress * 100) }) : $t("Descargar o paquete de idioma")}
      </button>
      {#if downloadErr}<p class="warn">{downloadErr}</p>{/if}
    {/if}

    <Segmented
      label={$t("Resolución de lectura")}
      options={[
        { value: "rapida", label: $t("Rápida"), detail: "150 ppp", hint: $t("150 ppp: borradores ou letra grande") },
        { value: "normal", label: $t("Normal"), detail: "200 ppp", hint: $t("200 ppp: a maioría das dixitalizacións") },
        { value: "alta", label: $t("Alta"), detail: "300 ppp", hint: $t("300 ppp: letra pequena ou dixitalizacións pobres") },
        { value: "custom", label: $t("Personalizado"), hint: $t("Os ppp que se queiran") },
      ]}
      bind:value={$form.profile}
    />
    {#if $form.profile === "custom"}
      <Slider label={$t("Resolución")} unit={$t("ppp")} min={100} max={400} step={10} bind:value={$form.dpi} def={200} hint={$t("Máis ppp = máis precisión nas letras pequenas e máis tempo por páxina")} />
    {/if}

    <Toggle label={$t("Saltar as páxinas que xa teñen texto")} bind:checked={$form.skipText} disabled={$busy} hint={$t("As páxinas que xa se poden buscar quedan tal cal. Desactívao para ler todas igual (por exemplo, se o texto que traen está mal).")} />
    <TextField label={$t("Páxinas (baleiro = todas)")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} hint={file.pages ? $t("O PDF ten {n} páxinas.", { n: file.pages }) : ""} />
  </div>

  <button class="btn primary block" on:click={run} disabled={$busy || !installed}>{$t("Engadir texto ao PDF")}</button>
  <JobStatus {job} />
</div>

<style>
  .warn { margin: 0; color: var(--accent-pdf); font-size: var(--fs-sm); }
</style>
