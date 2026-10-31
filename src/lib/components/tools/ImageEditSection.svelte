<script lang="ts">
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { imageEdit, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo, IMAGE_TARGETS } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import Icon from "../Icon.svelte";

  export let file: FileEntry;

  const form = createForm({
    rotate: 0,
    flipH: false,
    flipV: false,
    grayscale: false,
    invert: false,
    brightness: 0,
    contrast: 0,
    blur: 0,
    cropOn: false,
    cx: 0,
    cy: 0,
    cw: file.width ?? 0,
    ch: file.height ?? 0,
    ext: file.ext === "jpg" || file.ext === "png" || file.ext === "webp" ? file.ext : "png",
  });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;
  const dirty = form.dirty;

  // Vista previa en directo no visor (a copia gárdase só ao premer o botón).
  $: imageEdit.set({
    rotate: $form.rotate,
    flipH: $form.flipH,
    flipV: $form.flipV,
    grayscale: $form.grayscale,
    invert: $form.invert,
    brightness: $form.brightness,
    contrast: $form.contrast,
    blur: $form.blur,
  });
  onDestroy(() => imageEdit.set(null));

  const rot = (d: number) => form.update((f) => ({ ...f, rotate: (f.rotate + d + 360) % 360 }));

  async function apply() {
    const f = $form;
    try {
      const out = await askOutput(file, `-${tr("editada")}`, f.ext);
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const done = await engine.editImage(
          file.path,
          out,
          {
            rotate: f.rotate,
            flipH: f.flipH,
            flipV: f.flipV,
            grayscale: f.grayscale,
            invert: f.invert,
            brightness: f.brightness,
            contrast: f.contrast,
            blur: f.blur,
            crop: f.cropOn ? [f.cx, f.cy, f.cw, f.ch] : undefined,
            quality: 90,
          },
          true,
        );
        return `${listo()} ${fileNameOf(done)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <FormBar {form} />

  <div class="group">
    <h4 class="group-title">{$t("Transformar")}</h4>
    <div class="pair">
      <button class="btn" on:click={() => rot(270)} title={$t("Xirar 90° á esquerda")}>↺ {$t("Xirar 90° á esquerda")}</button>
      <button class="btn" on:click={() => rot(90)} title={$t("Xirar 90° á dereita")}>↻ {$t("Xirar 90° á dereita")}</button>
    </div>
    <Toggle label={$t("Espello horizontal")} bind:checked={$form.flipH} />
    <Toggle label={$t("Espello vertical")} bind:checked={$form.flipV} />
    <Toggle label={$t("Recortar (en píxeles)")} bind:checked={$form.cropOn} />
    {#if $form.cropOn}
      <div class="pair">
        <NumberField label="X" unit="px" min={0} bind:value={$form.cx} />
        <NumberField label="Y" unit="px" min={0} bind:value={$form.cy} />
        <NumberField label={$t("Ancho")} unit="px" min={1} bind:value={$form.cw} def={file.width ?? 0} />
        <NumberField label={$t("Alto")} unit="px" min={1} bind:value={$form.ch} def={file.height ?? 0} />
      </div>
    {/if}
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Cor e efectos")}</h4>
    <Slider label={$t("Brillo")} min={-100} max={100} bind:value={$form.brightness} def={0} />
    <Slider label={$t("Contraste")} min={-100} max={100} bind:value={$form.contrast} def={0} />
    <Slider label={$t("Desenfoque")} unit="px" min={0} max={20} step={0.5} bind:value={$form.blur} def={0} />
    <Toggle label={$t("Branco e negro")} bind:checked={$form.grayscale} />
    <Toggle label={$t("Inverter cores")} bind:checked={$form.invert} />
  </div>

  <div class="group">
    <SelectField label={$t("Gardar como")} bind:value={$form.ext} options={IMAGE_TARGETS.map((x) => ({ value: x, label: x.toUpperCase() }))} def={form.defaults.ext} />
    <button class="btn primary block" on:click={apply} disabled={!$dirty || $busy}>
      <Icon name="check" size={14} /> {$t("Gardar copia editada")}
    </button>
  </div>

  <JobStatus {job} />
</div>
