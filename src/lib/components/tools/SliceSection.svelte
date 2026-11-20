<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { get } from "svelte/store";
  import { claim } from "../../claim";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { imageCrop, imageCropAspect, imageCropEnabled, imageSlice, imageNatural, type FileEntry } from "../../files";
  import { fileNameOf, formatBytes, TRASNO, fallo, composeFormat, composeOptions, composeShown, composeSame, animatedNotice, readOnlyLabel } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askDir } from "./shared";
  import Icon from "../Icon.svelte";
  import Levels from "../ui/Levels.svelte";
  import LockNote from "../ui/LockNote.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Slider from "../ui/Slider.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Cortar unha imaxe en partes iguais (columnas × filas) dentro dunha zona que se move e se escala no visor, como o recorte. */
  export let file: FileEntry;

  // Forma de cada parte (ancho / alto); a zona do visor ten a proporción que sae de multiplicala por columnas e filas.
  const SHAPES = [
    { id: 0, label: "Libre", ar: null as number | null, w: 1, h: 1 },
    { id: 1, label: "1:1", ar: 1, w: 1, h: 1 },
    { id: 2, label: "4:5", ar: 4 / 5, w: 4, h: 5 },
    { id: 3, label: "3:4", ar: 3 / 4, w: 3, h: 4 },
    { id: 4, label: "9:16", ar: 9 / 16, w: 9, h: 16 },
    { id: 5, label: "16:9", ar: 16 / 9, w: 16, h: 9 },
  ];
  // Tres plantillas de uso común (o nome é curto; debaixo vai o que fan) e «Personalizado».
  const PRESETS = [
    { id: "carrusel", label: "Carrusel", detail: "3 × 1 · 1:1", values: { cols: 3, rows: 1, shape: 1 }, glyph: { k: "grid", cols: 3, rows: 1, w: 1, h: 1 } },
    { id: "vertical", label: "Vertical", detail: "3 × 1 · 3:4", values: { cols: 3, rows: 1, shape: 3 }, glyph: { k: "grid", cols: 3, rows: 1, w: 3, h: 4 } },
    { id: "grella", label: "Cuadrícula", detail: "3 × 3 · 1:1", values: { cols: 3, rows: 3, shape: 1 }, glyph: { k: "grid", cols: 3, rows: 3, w: 1, h: 1 } },
  ] as const;

  const form = createForm({ cols: 3 as number | null, rows: 1 as number | null, shape: 1, order: 0, format: "same", quality: 92 }, "cortar", "cortar|" + file.path);
  onMount(() => useForm(form));
  $: v = $form;

  const job = createJob();
  const busy = job.busy;

  // Unha imaxe animada só deixaría un fotograma en cada parte.
  let animated = false;
  $: void engine.imageIsAnimated(file.path).then((a) => (animated = a)).catch(() => (animated = false));

  $: W = $imageNatural.w || file.width || 0;
  $: H = $imageNatural.h || file.height || 0;
  $: exts = [file.ext];
  $: outExt = composeFormat(v.format, exts);
  $: sameExt = composeSame(exts);
  $: cols = Math.max(1, Math.min(20, Math.round(v.cols || 1)));
  $: rows = Math.max(1, Math.min(20, Math.round(v.rows || 1)));
  $: shapeAr = SHAPES.find((s) => s.id === v.shape)?.ar ?? null;
  $: target = shapeAr ? (shapeAr * cols) / rows : null;

  // O visor debuxa a zona dividida e deixa movela e escalala; as esquinas manteñen a proporción cando a forma é fixa.
  const own = claim("imageCrop");
  onMount(() => {
    imageCropEnabled.set(true);
    return () => own.release(() => {
      imageCrop.set(null);
      imageCropAspect.set(null);
      imageCropEnabled.set(false);
      imageSlice.set(null);
    });
  });
  $: imageSlice.set({ cols, rows, byColumns: v.order === 1 });
  $: imageCropAspect.set(target);
  $: imageCropEnabled.set(!animated);
  $: if (animated) imageCrop.set(null);

  /** O rectángulo máis grande que cabe na imaxe con esa proporción (ou toda a imaxe se é libre), centrado. */
  function biggest(ar: number | null): { x: number; y: number; w: number; h: number } {
    if (!ar) return { x: 0, y: 0, w: W, h: H };
    let w = W;
    let h = W / ar;
    if (h > H) {
      h = H;
      w = H * ar;
    }
    w = Math.max(1, Math.min(W, Math.round(w)));
    h = Math.max(1, Math.min(H, Math.round(w / ar)));
    return { x: Math.round((W - w) / 2), y: Math.round((H - h) / 2), w, h };
  }
  /** Ao cambiar columnas, filas ou forma, a zona adáptase á nova proporción sen cambiar de sitio nin de ancho se cabe. */
  function reshape(ar: number | null, _w: number, _h: number, _anim: boolean) {
    if (!W || !H || animated) return;
    const c = get(imageCrop);
    if (!c) return void imageCrop.set(biggest(ar));
    if (!ar) return;
    let w = c.w;
    let h = w / ar;
    if (h > H) {
      h = H;
      w = h * ar;
    }
    if (w > W) {
      w = W;
      h = w / ar;
    }
    w = Math.max(1, Math.round(w));
    h = Math.max(1, Math.min(H, Math.round(w / ar)));
    imageCrop.set({ x: Math.max(0, Math.min(W - w, Math.round(c.x + (c.w - w) / 2))), y: Math.max(0, Math.min(H - h, Math.round(c.y + (c.h - h) / 2))), w, h });
  }
  $: reshape(target, W, H, animated);
  const fitAll = () => imageCrop.set(biggest(target));

  $: zone = $imageCrop;
  $: pw = zone ? Math.floor(zone.w / cols) : 0;
  $: ph = zone ? Math.floor(zone.h / rows) : 0;
  $: parts = cols * rows;
  $: tooMany = parts > 100;
  $: tooSmall = !!zone && (pw < 1 || ph < 1);
  $: problem = tooMany ? tr("O máximo son 100 partes: baixa as columnas ou as filas.") : tooSmall ? tr("Cada parte sairía de menos dun píxel: amplía a zona ou fai menos partes.") : "";

  $: levels = PRESETS.map((p) => ({ id: p.id, label: $t(p.label), detail: p.detail, glyph: p.glyph, values: { ...p.values } }));

  async function run() {
    const c = get(imageCrop);
    if (!c || problem) return;
    const dir = await askDir(file.path);
    if (!dir) return;
    const stem = file.name.replace(/\.[^.]+$/, "");
    try {
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.sliceImage(file.path, dir, stem, outExt, {
          rect: [c.x, c.y, c.w, c.h],
          cols,
          rows,
          by_columns: v.order === 1,
          quality: outExt === "jpg" ? v.quality : null,
          background: [255, 255, 255],
        });
        return tr("Listo! Deixei {n} partes de {w}×{h} px en {dir} ({size}).", { n: r.outputs.length, w: r.piece[0], h: r.piece[1], dir: fileNameOf(dir), size: formatBytes(r.bytes_written) });
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
  onDestroy(() => job.clear());
</script>

<div class="stack">
  {#if animated}<LockNote label={readOnlyLabel()} text={animatedNotice(file.ext, "cortar")} />{/if}
  <div class="locked-tools" class:on={animated} inert={animated}>
    <div class="group">
      <Levels
        label={$t("Plantilla")}
        info={$t("Arrastra a zona sobre a imaxe e tira das esquinas para escalala. Con forma fixa, as esquinas manteñen a proporción.")}
        {levels}
        values={{ cols: v.cols ?? 0, rows: v.rows ?? 0, shape: v.shape }}
        on:apply={async (e) => {
          form.update((f) => ({ ...f, ...e.detail }));
          await tick();
          fitAll(); // unha plantilla nova parte da zona máis grande posible
        }}
      />
      <div class="pair">
        <NumberField label={$t("Columnas")} min={1} max={20} bind:value={$form.cols} def={form.defaults.cols} />
        <NumberField label={$t("Filas")} min={1} max={20} bind:value={$form.rows} def={form.defaults.rows} />
      </div>
      <Segmented
        label={$t("Proporción")}
        info={$t("Proporción de cada parte (ancho:alto). Libre usa a que dea a zona.")}
        options={SHAPES.map((s) => ({ value: s.id, label: s.id === 0 ? $t(s.label) : s.label, glyph: s.ar ? ({ k: "ratio", w: s.w, h: s.h } as const) : ({ k: "free" } as const) }))}
        bind:value={$form.shape}
      />
      <button class="btn" on:click={fitAll}><Icon name="expand" size={14} /> {$t("Axustar á imaxe")}</button>
      <Segmented
        label={$t("Numerar")}
        options={[
          { value: 0, label: $t("Por filas"), glyph: { k: "order", by: "rows" } },
          { value: 1, label: $t("Por columnas"), glyph: { k: "order", by: "cols" } },
        ]}
        bind:value={$form.order}
      />
    </div>

    <div class="group">
      <SelectField
        label={$t("Formato")}
        value={composeShown(v.format, exts)}
        on:change={(e) => ($form.format = String(e.detail))}
        options={composeOptions($t, exts)}
        hint={sameExt ? $t("PNG, TIFF e BMP non perden calidade; JPG e WebP si, e deixan axustala.") : $t("Esta imaxe non se pode gardar no seu formato, así que non hai «Igual que o orixinal». PNG non perde calidade.")}
        def={sameExt ? "same" : "png"}
      />
      {#if outExt === "jpg"}
        <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={form.defaults.quality} />
      {/if}
    </div>

    {#if zone}
      <ul class="results">
        <li><span class="rname">{$t("Partes")}</span><span class="rstate">{cols} × {rows} = {parts}</span></li>
        <li><span class="rname">{$t("Cada parte")}</span><span class="rstate">{pw} × {ph} px</span></li>
        <li><span class="rname">{$t("Zona")}</span><span class="rstate">{zone.w} × {zone.h} px</span></li>
      </ul>
    {/if}
    {#if problem}<p class="warn selectable">{problem}</p>{/if}

    <button class="btn primary block" on:click={run} disabled={$busy || animated || !zone || !!problem}>
      {$t("Dividir en {n} partes", { n: parts })}
    </button>
  </div>
  <JobStatus {job} />
</div>

<style>
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); word-break: break-word; }
</style>
