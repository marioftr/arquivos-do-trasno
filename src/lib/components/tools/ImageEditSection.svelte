<script lang="ts">
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { get } from "svelte/store";
  import { imageEdit, imageCrop, imageCropAspect, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo, IMAGE_TARGETS } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import Segmented from "../ui/Segmented.svelte";
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
  onDestroy(() => {
    imageEdit.set(null);
    imageCrop.set(null);
    imageCropAspect.set(null);
  });

  // ---- recorte visual: o visor e os campos están sempre sincronizados ----
  const W = file.width ?? 0;
  const H = file.height ?? 0;
  let wasOn = false;
  $: {
    const f = $form;
    if (f.cropOn) {
      if (!wasOn && f.cw === W && f.ch === H && W && H) {
        // Ao acender o recorte, empeza cun rectángulo centrado ao 80 %.
        const w = Math.round(W * 0.8);
        const h = Math.round(H * 0.8);
        form.update((v) => ({ ...v, cx: Math.round((W - w) / 2), cy: Math.round((H - h) / 2), cw: w, ch: h }));
      } else {
        const cur = get(imageCrop);
        if (!cur || cur.x !== f.cx || cur.y !== f.cy || cur.w !== f.cw || cur.h !== f.ch) imageCrop.set({ x: f.cx, y: f.cy, w: f.cw, h: f.ch });
      }
    } else if (get(imageCrop)) {
      imageCrop.set(null);
    }
    wasOn = f.cropOn;
  }
  onMount(() =>
    imageCrop.subscribe((c) => {
      if (!c) return;
      const f = get(form);
      if (f.cx !== c.x || f.cy !== c.y || f.cw !== c.w || f.ch !== c.h) form.update((v) => ({ ...v, cx: c.x, cy: c.y, cw: c.w, ch: c.h }));
    }),
  );

  let aspect = "free";
  let lastAspect = "free";
  $: if (aspect !== lastAspect) setAspect(aspect);
  function setAspect(a: string) {
    lastAspect = a;
    const ar = a === "free" ? null : Number(a);
    imageCropAspect.set(ar);
    const c = get(imageCrop);
    if (ar && c) {
      const h = Math.max(8, Math.min(H - c.y, Math.round(c.w / ar)));
      imageCrop.set({ ...c, h });
    }
  }
  function fullCrop() {
    form.update((v) => ({ ...v, cx: 0, cy: 0, cw: W, ch: H }));
  }

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

  <div class="group">
    <h4 class="group-title">{$t("Transformar")}</h4>
    <div class="pair">
      <button class="btn" on:click={() => rot(270)} title={$t("Xirar 90° á esquerda")}>↺ {$t("Xirar 90° á esquerda")}</button>
      <button class="btn" on:click={() => rot(90)} title={$t("Xirar 90° á dereita")}>↻ {$t("Xirar 90° á dereita")}</button>
    </div>
    <Toggle label={$t("Espello horizontal")} bind:checked={$form.flipH} />
    <Toggle label={$t("Espello vertical")} bind:checked={$form.flipV} />
    <Toggle label={$t("Recortar")} bind:checked={$form.cropOn} />
    {#if $form.cropOn}
      <p class="group-note">{$t("Arrastra o rectángulo sobre a imaxe, ou escribe as medidas.")}</p>
      <Segmented
        label={$t("Proporción")}
        options={[
          { value: "free", label: $t("Libre") },
          { value: "1", label: "1:1" },
          { value: "1.3333", label: "4:3" },
          { value: "1.7778", label: "16:9" },
          { value: "0.75", label: "3:4" },
        ]}
        bind:value={aspect}
      />
      <button class="btn" on:click={fullCrop}>{$t("Toda a imaxe")}</button>
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
    <SelectField label={$t("Gardar como")} bind:value={$form.ext} options={[...IMAGE_TARGETS, "pdf"].map((x) => ({ value: x, label: x.toUpperCase() }))} def={form.defaults.ext} />
    <button class="btn primary block" on:click={apply} disabled={!$dirty || $busy}>
      <Icon name="check" size={14} /> {$t("Gardar copia editada")}
    </button>
  </div>

  <JobStatus {job} />
</div>
