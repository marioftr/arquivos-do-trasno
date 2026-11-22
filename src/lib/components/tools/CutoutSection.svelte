<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { onMount, onDestroy } from "svelte";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { viewUrl, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { tools } from "../../settings";
  import { missingOf, type Need } from "../../models";
  import { backdrop, cutoutView, cssOf, rgbOf, type Bg } from "../../transparency";
  import { askOutput } from "./shared";
  import BackgroundPicker from "../ui/BackgroundPicker.svelte";
  import LockNote from "../ui/LockNote.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Slider from "../ui/Slider.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import ExtensionGate from "../ui/ExtensionGate.svelte";

  export let file: FileEntry;

  const needs: Need[] = [{ kind: "model", id: "cutout" }];
  const form = createForm(
    { background: "transparent" as Bg | "transparent", color: "#ffffff", ext: "png", edge: 50, soft: 70, detail: 60 },
    undefined,
    "cutout|" + file.path,
  );
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  // Unha imaxe animada só deixaría un fotograma.
  let animated = false;
  $: void engine.imageIsAnimated(file.path).then((a) => (animated = a)).catch(() => (animated = false));

  $: missing = missingOf(needs, $tools).length > 0;
  $: solid = $form.background !== "transparent";
  // Co fondo transparente non vale JPG (non garda transparencia).
  $: formats = [{ value: "png", label: "PNG" }, { value: "webp", label: "WebP" }, ...(solid ? [{ value: "jpg", label: "JPG" }] : [])];
  $: if (!formats.some((f) => f.value === $form.ext)) $form.ext = "png";

  // ---- vista previa: só cando se pide. O modelo analiza a imaxe unha vez; despois os axustes só recompoñen a máscara en directo
  let previewing = false;
  let analyzing = false;
  let ready = false;
  let analysisFor = "";
  let rounds = 0;

  function stopPreview() {
    previewing = false;
    clearTimeout(timer);
    cutoutView.set(null);
    backdrop.set(null);
  }
  async function togglePreview() {
    if (previewing) {
      stopPreview();
      return;
    }
    if (ready && analysisFor === file.path) {
      previewing = true;
      void render();
      return;
    }
    const mine = ++rounds;
    analysisFor = file.path;
    ready = false;
    analyzing = true;
    try {
      await engine.cutoutAnalyze(file.path);
      if (mine !== rounds) return;
      ready = true;
      previewing = true;
      void render();
    } catch (e) {
      if (mine === rounds) {
        analysisFor = "";
        job.say(false, `${tr("Non puiden previsualizar o recorte.")}
${errText(e)}`);
      }
    } finally {
      if (mine === rounds) analyzing = false;
    }
  }
  // Se cambia a imaxe, a vista previa déixase.
  let shownFor = file.path;
  $: if (file.path !== shownFor) {
    shownFor = file.path;
    rounds++;
    analyzing = false;
    ready = false;
    analysisFor = "";
    stopPreview();
  }

  let seq = 0;
  let rendering = false;
  let again = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  /** Compón a vista previa cos axustes de agora (de un en un: se hai cambios mentres tanto, repítese ao acabar). */
  async function render() {
    if (!ready || !previewing) return;
    if (rendering) {
      again = true;
      return;
    }
    rendering = true;
    try {
      const f = $form;
      const p = await engine.cutoutPreview(file.path, ++seq, { edge: f.edge, soft: f.soft, detail: f.detail });
      if (previewing) cutoutView.set(await viewUrl(p));
    } catch (e) {
      stopPreview();
      job.say(false, `${tr("Non puiden previsualizar o recorte.")}
${errText(e)}`);
    } finally {
      rendering = false;
      if (again) {
        again = false;
        void render();
      }
    }
  }
  $: if (previewing) {
    void [$form.edge, $form.soft, $form.detail];
    clearTimeout(timer);
    timer = setTimeout(render, 80);
  }

  // O fondo que se elixe vese detrás do recorte, como quedará ao gardar.
  $: backdrop.set(previewing && $form.background !== "transparent" ? cssOf($form.background, $form.color) : null);
  onDestroy(() => {
    rounds++;
    previewing = false;
    ready = false;
    clearTimeout(timer);
    cutoutView.set(null);
    backdrop.set(null);
    void engine.cutoutForget().catch(() => {});
    void engine.forgetPreviews(file.path).catch(() => {});
  });

  async function run() {
    try {
      const f = $form;
      const out = await askOutput(file, "-sen-fondo", f.ext);
      if (!out) return;
      const bg = f.background === "transparent" ? null : rgbOf(f.background, f.color);
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.removeBackground(file.path, out, bg, { edge: f.edge, soft: f.soft, detail: f.detail }, true);
        return `${listo()} ${fileNameOf(r.output_path)}\n${tr("Quedou o {p} % da imaxe; o resto era fondo.", { p: Math.round(r.kept * 100) })}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <PanelInfo text={$t("Recoñece o obxecto principal da imaxe e quita o resto. O resultado vese no visor mentres se axusta.")} />
  {#if animated}<LockNote label={tr("Imaxe animada")} text={tr("Este {fmt} é animado: quitar o fondo só tratería un fotograma.", { fmt: file.ext.toUpperCase() })} />{/if}

  <div class="locked-tools" class:on={animated} inert={animated}>
    {#if missing}
      <ExtensionGate {needs} />
    {:else}
      <div class="group">
        <BackgroundPicker transparent bind:value={$form.background} bind:color={$form.color} />
      </div>
      <div class="group">
        <Slider label={$t("Recorte")} min={0} max={100} bind:value={$form.edge} def={50} hint={$t("Máis alto quita máis arredor do obxecto; máis baixo deixa máis.")} />
        <Slider label={$t("Suavidade")} min={0} max={100} bind:value={$form.soft} def={70} hint={$t("Máis baixo: beira máis seca; máis alto: máis difuminada.")} />
        <Slider label={$t("Detalle")} min={0} max={100} bind:value={$form.detail} def={60} hint={$t("Axusta a beira ás liñas da imaxe, como o pelo ou as follas.")} />
      </div>
      <div class="group">
        <SelectField label={$t("Gardar como")} bind:value={$form.ext} def="png" options={formats} />
        <div class="estimate">
          <button class="btn" class:on={previewing} aria-pressed={previewing} on:click={togglePreview} disabled={analyzing || animated}>{analyzing ? $t("Analizando…") : $t("Previsualizar")}</button>
        </div>
        <button class="btn primary block" on:click={run} disabled={$busy || animated || analyzing}>{$t("Quitar fondo")}</button>
      </div>
    {/if}
  </div>
  <JobStatus {job} />
</div>
