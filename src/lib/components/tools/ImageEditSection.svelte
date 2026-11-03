<script lang="ts">
  import GroupNote from "../ui/GroupNote.svelte";
  import { claim } from "../../claim";
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { get } from "svelte/store";
  import { imageEdit, imageCrop, imageCropAspect, imageCropEnabled, imageNatural, type FileEntry } from "../../files";
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
  import { HINTS } from "../../hints";

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
  }, undefined, "imgedit|" + file.path);
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
  const own = claim("imageEdit");
  onDestroy(() =>
    own.release(() => {
      imageEdit.set(null);
      imageCrop.set(null);
      imageCropAspect.set(null);
      imageCropEnabled.set(false);
    }),
  );

  // ---- recorte directo no visor: arrastrar sobre a imaxe debuxa o rectángulo; os campos seguen o que se ve
  // O recorte vai sobre a imaxe tal como se ve: despois de xirala ou voltala (ancho e alto trócanse cos xiros de 90°).
  $: W0 = $imageNatural.w || file.width || 0;
  $: H0 = $imageNatural.h || file.height || 0;
  $: swapped = $form.rotate % 180 !== 0;
  $: W = swapped ? H0 : W0;
  $: H = swapped ? W0 : H0;
  onMount(() => {
    imageCropEnabled.set(true);
    // Ao abrir a edición xa se ve o rectángulo sobre toda a imaxe, listo para axustar.
    if (W && H && !get(imageCrop)) imageCrop.set({ x: 0, y: 0, w: W, h: H });
    return imageCrop.subscribe((c) => {
      const f = get(form);
      if (!c) {
        if (f.cropOn) form.update((v) => ({ ...v, cropOn: false }));
        return;
      }
      if (!f.cropOn || f.cx !== c.x || f.cy !== c.y || f.cw !== c.w || f.ch !== c.h) form.update((v) => ({ ...v, cropOn: true, cx: c.x, cy: c.y, cw: c.w, ch: c.h }));
    });
  });
  /** Os campos numéricos moven o rectángulo do visor. */
  function typedCrop() {
    const f = get(form);
    if (!f.cropOn) return;
    const w = Math.max(1, Math.min(W, Math.round(f.cw)));
    const h = Math.max(1, Math.min(H, Math.round(f.ch)));
    imageCrop.set({ x: Math.max(0, Math.min(W - w, Math.round(f.cx))), y: Math.max(0, Math.min(H - h, Math.round(f.cy))), w, h });
  }
  $: ($form.cx, $form.cy, $form.cw, $form.ch, typedCrop());

  let aspect = "free";
  let lastAspect = "free";
  $: if (aspect !== lastAspect) setAspect(aspect);
  function setAspect(a: string) {
    lastAspect = a;
    const ar = a === "free" ? null : Number(a);
    imageCropAspect.set(ar);
    const c = get(imageCrop);
    if (!ar || !c) return;
    // O recorte actual encolle, centrado, ata ter a proporción pedida: nunca se estira nin se deforma.
    let w = c.w;
    let h = w / ar;
    if (h > c.h) {
      h = c.h;
      w = h * ar;
    }
    w = Math.max(1, Math.min(W, Math.round(w)));
    h = Math.max(1, Math.min(H, Math.round(w / ar)));
    const x = Math.max(0, Math.min(W - w, Math.round(c.x + (c.w - w) / 2)));
    const y = Math.max(0, Math.min(H - h, Math.round(c.y + (c.h - h) / 2)));
    imageCrop.set({ x, y, w, h });
  }
  function fullCrop() {
    imageCrop.set({ x: 0, y: 0, w: W, h: H });
  }
  const clearCrop = () => imageCrop.set(null);

  /** Xira a imaxe e leva o recorte co xiro (se hai un), para que siga sobre o mesmo anaco. */
  function rot(d: number) {
    const before = get(form);
    const c = get(imageCrop);
    const bw = before.rotate % 180 !== 0 ? H0 : W0;
    const bh = before.rotate % 180 !== 0 ? W0 : H0;
    form.update((f) => ({ ...f, rotate: (f.rotate + d + 360) % 360 }));
    if (!c) return;
    if (d === 90) imageCrop.set({ x: bh - c.y - c.h, y: c.x, w: c.h, h: c.w });
    else if (d === 270) imageCrop.set({ x: c.y, y: bw - c.x - c.w, w: c.h, h: c.w });
    else if (d === 180) imageCrop.set({ x: bw - c.x - c.w, y: bh - c.y - c.h, w: c.w, h: c.h });
  }
  function flip(axis: "H" | "V") {
    const c = get(imageCrop);
    form.update((f) => (axis === "H" ? { ...f, flipH: !f.flipH } : { ...f, flipV: !f.flipV }));
    if (c) imageCrop.set(axis === "H" ? { ...c, x: W - c.x - c.w } : { ...c, y: H - c.y - c.h });
  }

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
      <button class="btn" on:click={() => rot(270)} title={$t("Xirar 90° á esquerda")}><Icon name="rotate-ccw" size={15} /> {$t("Xirar 90° á esquerda")}</button>
      <button class="btn" on:click={() => rot(90)} title={$t("Xirar 90° á dereita")}><Icon name="rotate-cw" size={15} /> {$t("Xirar 90° á dereita")}</button>
    </div>
    <div class="pair">
      <button class="btn tog" class:on={$form.flipH} aria-pressed={$form.flipH} on:click={() => flip("H")} title={$t(HINTS["Espello horizontal"])}><Icon name="flip-h" size={15} /> {$t("Espello horizontal")}</button>
      <button class="btn tog" class:on={$form.flipV} aria-pressed={$form.flipV} on:click={() => flip("V")} title={$t(HINTS["Espello vertical"])}><Icon name="flip-v" size={15} /> {$t("Espello vertical")}</button>
    </div>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Recortar")}</h4>
    <GroupNote text={$t("Arrastrar sobre a imaxe delimita o que queda; despois pódese mover o rectángulo ou tirar das esquinas.")} />
    <Segmented
      label={$t("Proporción")}
      options={[
        { value: "free", label: $t("Libre") },
        { value: "1", label: "1:1" },
        { value: "1.3333", label: "4:3" },
        { value: "1.7778", label: "16:9" },
        { value: "0.75", label: "3:4" },
        { value: "0.5625", label: "9:16" },
      ]}
      bind:value={aspect}
    />
    <div class="pair">
      <button class="btn" on:click={fullCrop}>{$t("Toda a imaxe")}</button>
      <button class="btn" on:click={clearCrop} disabled={!$form.cropOn}>{$t("Quitar o recorte")}</button>
    </div>
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
    <SelectField label={$t("Gardar como")} bind:value={$form.ext} options={[...IMAGE_TARGETS, "pdf"].map((x) => ({ value: x, label: x.toUpperCase() }))} def={form.defaults.ext} />
    <button class="btn primary block" on:click={apply} disabled={!$dirty || $busy}>
      {$t("Gardar copia editada")}
    </button>
  </div>

  <JobStatus {job} />
</div>
