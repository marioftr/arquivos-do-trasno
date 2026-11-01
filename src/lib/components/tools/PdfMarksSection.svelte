<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import { pdfOverlay, viewUrl, type FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput, safePages, parsePageSpec } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import TextField from "../ui/TextField.svelte";
  import ColorField from "../ui/ColorField.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import Icon from "../Icon.svelte";

  export let file: FileEntry;

  const form = createForm({
    kind: "watermark" as "watermark" | "numbers" | "text" | "image",
    pageSpec: "",
    // marca de auga
    wmText: "BORRADOR",
    wmSize: 60,
    wmOpacity: 30,
    wmAngle: 45,
    wmColor: "#c0392b",
    // números
    numPosition: "bc",
    numFormat: "{n}",
    numStart: 1,
    numSize: 11,
    numMargin: 28,
    numColor: "#000000",
    // texto
    txText: "",
    txX: 10,
    txY: 90,
    txSize: 16,
    txColor: "#000000",
    // imaxe / firma
    imX: 80,
    imY: 12,
    imWidth: 25,
    imOpacity: 100,
  });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;
  let imagePath: string | null = null;
  let imageUrl = "";

  $: f = $form;
  $: pages = safePages(f.pageSpec, file.pages);

  // Vista previa en directo sobre as páxinas.
  $: pdfOverlay.set(
    f.kind === "watermark"
      ? { text: { text: f.wmText, xPct: 50, yPct: 50, size: f.wmSize, color: f.wmColor, opacity: f.wmOpacity / 100, angle: f.wmAngle, centered: true, pages } }
      : f.kind === "numbers"
        ? { numbers: { position: f.numPosition, format: f.numFormat, start: f.numStart, size: f.numSize, margin: f.numMargin, color: f.numColor } }
        : f.kind === "text"
          ? { text: { text: f.txText, xPct: f.txX, yPct: f.txY, size: f.txSize, color: f.txColor, opacity: 1, angle: 0, centered: false, pages } }
          : imageUrl
            ? { image: { url: imageUrl, xPct: f.imX, yPct: f.imY, widthPct: f.imWidth, opacity: f.imOpacity / 100, pages } }
            : null,
  );
  onDestroy(() => pdfOverlay.set(null));

  async function chooseImage() {
    const picked = await open({ multiple: false, filters: [{ name: tr("Imaxes"), extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"] }] });
    if (typeof picked !== "string") return;
    imagePath = picked;
    imageUrl = await viewUrl(picked);
  }

  const COLORS = (hex: string) => hex;

  async function apply() {
    let sel: number[] | null;
    try {
      sel = parsePageSpec(f.pageSpec, file.pages);
    } catch (e) {
      job.say(false, errText(e));
      return;
    }
    if (f.kind === "image" && !imagePath) {
      job.say(false, tr("Escolle primeiro a imaxe (por exemplo, a túa firma)."));
      return;
    }
    const suffix = { watermark: "-marca", numbers: "-numerado", text: "-texto", image: "-sinatura" }[f.kind];
    try {
      const out = await askOutput(file, tr(suffix), "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        let done: string;
        if (f.kind === "watermark") {
          done = await engine.pdfWatermarkText(file.path, out, { text: f.wmText, size: f.wmSize, opacity: f.wmOpacity / 100, angle: f.wmAngle, color: COLORS(f.wmColor) }, sel, true);
        } else if (f.kind === "numbers") {
          done = await engine.pdfPageNumbers(file.path, out, { position: f.numPosition, format: f.numFormat, start: f.numStart, size: f.numSize, margin: f.numMargin, color: f.numColor }, sel, true);
        } else if (f.kind === "text") {
          done = await engine.pdfAddText(file.path, out, { text: f.txText, xPct: f.txX, yPct: f.txY, size: f.txSize, color: f.txColor }, sel, true);
        } else {
          done = await engine.pdfStampImage(file.path, out, { image: imagePath!, xPct: f.imX, yPct: f.imY, widthPct: f.imWidth, opacity: f.imOpacity / 100 }, sel, true);
        }
        return `${listo()} ${fileNameOf(done)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">

  <Segmented
    options={[
      { value: "watermark", label: $t("Marca de auga") },
      { value: "numbers", label: $t("Números") },
      { value: "text", label: $t("Texto") },
      { value: "image", label: $t("Firma / imaxe") },
    ]}
    bind:value={$form.kind}
  />

  <div class="group">
    {#if f.kind === "watermark"}
      <TextField label={$t("Texto da marca de auga")} bind:value={$form.wmText} />
      <Slider label={$t("Tamaño")} unit="pt" min={20} max={200} bind:value={$form.wmSize} def={60} />
      <Slider label={$t("Opacidade")} unit="%" min={5} max={100} bind:value={$form.wmOpacity} def={30} />
      <Slider label={$t("Xiro")} unit="°" min={-90} max={90} bind:value={$form.wmAngle} def={45} />
      <ColorField label={$t("Cor")} bind:value={$form.wmColor} />
    {:else if f.kind === "numbers"}
      <SelectField
        label={$t("Posición")}
        bind:value={$form.numPosition}
        def="bc"
        options={[
          { value: "bc", label: $t("Abaixo, centro") },
          { value: "br", label: $t("Abaixo, dereita") },
          { value: "bl", label: $t("Abaixo, esquerda") },
          { value: "tc", label: $t("Arriba, centro") },
          { value: "tr", label: $t("Arriba, dereita") },
          { value: "tl", label: $t("Arriba, esquerda") },
        ]}
      />
      <SelectField
        label={$t("Formato")}
        hint={$t("Como se escribe o número: só o número, ou «3 / 10» para saber tamén o total de páxinas.")}
        bind:value={$form.numFormat}
        def={"{n}"}
        options={[
          { value: "{n}", label: "1, 2, 3…" },
          { value: "{n} / {total}", label: "1 / 10, 2 / 10…" },
          { value: tr("Páxina") + " {n}", label: tr("Páxina") + " 1, " + tr("Páxina") + " 2…" },
          { value: "- {n} -", label: "- 1 -, - 2 -…" },
        ]}
      />
      <div class="pair">
        <NumberField label={$t("Comezar en")} min={0} bind:value={$form.numStart} def={1} />
        <NumberField label={$t("Tamaño")} unit="pt" min={6} max={48} bind:value={$form.numSize} def={11} />
      </div>
      <Slider label={$t("Distancia ao bordo")} unit="pt" min={8} max={120} bind:value={$form.numMargin} def={28} />
      <ColorField label={$t("Cor")} bind:value={$form.numColor} />
    {:else if f.kind === "text"}
      <TextField label={$t("Texto")} bind:value={$form.txText} placeholder={$t("Escribe o que queres engadir")} />
      <Slider label={$t("Posición horizontal")} unit="%" min={0} max={100} bind:value={$form.txX} def={10} />
      <Slider label={$t("Posición vertical")} unit="%" min={0} max={100} bind:value={$form.txY} def={90} hint={$t("0 % = abaixo; 100 % = arriba")} />
      <Slider label={$t("Tamaño")} unit="pt" min={6} max={96} bind:value={$form.txSize} def={16} />
      <ColorField label={$t("Cor")} bind:value={$form.txColor} />
    {:else}
      <button class="btn" on:click={chooseImage}><Icon name="image" size={14} /> {imagePath ? fileNameOf(imagePath) : $t("Escoller imaxe…")}</button>
      <p class="group-note">{$t("Vale un PNG con fondo transparente, por exemplo a túa firma.")}</p>
      <Slider label={$t("Posición horizontal")} unit="%" min={0} max={100} bind:value={$form.imX} def={80} />
      <Slider label={$t("Posición vertical")} unit="%" min={0} max={100} bind:value={$form.imY} def={12} hint={$t("0 % = abaixo; 100 % = arriba")} />
      <Slider label={$t("Ancho")} unit="%" min={3} max={90} bind:value={$form.imWidth} def={25} />
      <Slider label={$t("Opacidade")} unit="%" min={5} max={100} bind:value={$form.imOpacity} def={100} />
    {/if}
    <TextField label={$t("Páxinas (baleiro = todas)")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} />
  </div>

  <button class="btn primary block" on:click={apply} disabled={$busy}>{$t("Gardar unha copia co cambio")}</button>
  <JobStatus {job} />
</div>
