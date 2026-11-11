<script lang="ts">
  import { onMount } from "svelte";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  /** Obxectivos de volume segundo onde se vai escoitar o ficheiro. «Personalizado» deixa poñer os tres valores á man. */
  const PROFILES: Record<string, { lufs: number; peak: number; range: number }> = {
    streaming: { lufs: -14, peak: -1, range: 11 },
    podcast: { lufs: -16, peak: -1.5, range: 11 },
    emision: { lufs: -23, peak: -1, range: 15 },
  };

  const form = createForm({ fade: 2, echo: 300, bass: 6, treble: 6, profile: "podcast", lufs: -16, peak: -1.5, range: 11 }, "audio-efectos", "audiofx|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  // Un perfil fixo enche os tres valores; con «Personalizado» quedan os que poña o usuario.
  $: preset = PROFILES[$form.profile];
  $: if (preset && ($form.lufs !== preset.lufs || $form.peak !== preset.peak || $form.range !== preset.range)) form.update((f) => ({ ...f, ...preset }));

  const num = (v: number) => v.toFixed(1).replace("-", "−").replace(".", ",");

  async function normalize() {
    try {
      const out = await askOutput(file, tr("-normalizado"), file.ext);
      if (!out) return;
      const target = { lufs: $form.lufs, peak: $form.peak, range: $form.range };
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.normalizeAudio(file.path, out, target, false, true);
        const how = r.linear ? tr("Só cambiou o volume: a dinámica quedou igual.") : tr("Para chegar houbo que comprimir a dinámica.");
        return `${listo()} ${fileNameOf(r.output_path)}\n${tr("Volume: de {a} a {b} LUFS.", { a: num(r.before_lufs), b: num(target.lufs) })} ${how}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }

  async function apply(suffix: string, op: engine.MediaOp, value: number) {
    try {
      const out = await askOutput(file, tr(suffix), file.ext);
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => `${listo()} ${fileNameOf(await engine.mediaEdit(file.path, out, op, value, false, true))}`);
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">

  <div class="group">
    <h4 class="group-title">{$t("Son")}</h4>
    <div class="pair">
      <button class="btn" on:click={() => apply("-limpo", "denoise", 0)} disabled={$busy} title={$t("Reduce o ruído de fondo")}>{$t("Quitar ruído")}</button>
      <button class="btn" on:click={() => apply("-do-revés", "reverse", 0)} disabled={$busy}>{$t("Dar a volta ao son")}</button>
    </div>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Normalizar o volume")}</h4>
    <Segmented
      label={$t("Perfil")}
      info={$t("Iguala o volume percibido do ficheiro enteiro. Primeiro mídese e despois súbese ou báixase todo o mesmo, así o que era baixo segue sendo baixo. Streaming: −14 LUFS (Spotify, YouTube). Podcast: −16 LUFS. Emisión: −23 LUFS (radio e televisión). Personalizado: os valores que se queiran.")}
      options={[
        { value: "streaming", label: $t("Streaming"), hint: $t("−14 LUFS: Spotify, YouTube e semellantes") },
        { value: "podcast", label: $t("Podcast"), hint: $t("−16 LUFS: voz para escoitar con auriculares") },
        { value: "emision", label: $t("Emisión"), hint: $t("−23 LUFS: radio e televisión (EBU R128)") },
        { value: "custom", label: $t("Personalizado"), hint: $t("Os valores que se queiran") },
      ]}
      bind:value={$form.profile}
    />
    {#if $form.profile === "custom"}
      <Slider label={$t("Volume obxectivo")} unit="LUFS" min={-30} max={-8} step={0.5} bind:value={$form.lufs} def={-16} hint={$t("Volume percibido medio. Máis preto de 0 = máis alto.")} />
      <Slider label={$t("Pico máximo")} unit="dBTP" min={-6} max={0} step={0.5} bind:value={$form.peak} def={-1.5} hint={$t("O punto máis alto que pode alcanzar o son. Por baixo de 0 evítase que distorsione.")} />
      <Slider label={$t("Rango dinámico máximo")} unit="LU" min={1} max={30} step={1} bind:value={$form.range} def={11} hint={$t("Diferenza permitida entre o máis baixo e o máis alto. Se o ficheiro ten máis, comprímese para caber.")} />
    {/if}
    <button class="btn block" on:click={normalize} disabled={$busy}>{$t("Normalizar")}</button>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Esvaecer")}</h4>
    <Slider label={$t("Duración")} unit="s" min={0.1} max={30} step={0.1} bind:value={$form.fade} def={2} />
    <div class="pair">
      <button class="btn" on:click={() => apply("-entrada", "fadein", $form.fade)} disabled={$busy}>{$t("Ao principio")}</button>
      <button class="btn" on:click={() => apply("-saída", "fadeout", $form.fade)} disabled={$busy}>{$t("Ao final")}</button>
    </div>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Ecualizador")}</h4>
    <Slider label={$t("Graves")} unit="dB" min={-12} max={12} step={1} bind:value={$form.bass} def={6} />
    <button class="btn block" on:click={() => apply("-graves", "bass", $form.bass)} disabled={$busy}>{$t("Aplicar graves")}</button>
    <Slider label={$t("Agudos")} unit="dB" min={-12} max={12} step={1} bind:value={$form.treble} def={6} />
    <button class="btn block" on:click={() => apply("-agudos", "treble", $form.treble)} disabled={$busy}>{$t("Aplicar agudos")}</button>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Eco")}</h4>
    <Slider label={$t("Atraso")} unit="ms" min={20} max={2000} step={10} bind:value={$form.echo} def={300} />
    <button class="btn block" on:click={() => apply("-eco", "echo", $form.echo)} disabled={$busy}>{$t("Aplicar eco")}</button>
  </div>

  <JobStatus {job} />
</div>
