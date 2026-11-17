<script lang="ts">
  import { onMount } from "svelte";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, formatClock, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;
  const isVideo = file.category === "video";

  /** Que se considera silencio segundo o tipo de gravación. «Personalizado» deixa poñer os tres valores á man. */
  const PROFILES: Record<string, { thresholdDb: number; minSilence: number; padding: number }> = {
    suave: { thresholdDb: -40, minSilence: 1, padding: 0.15 },
    normal: { thresholdDb: -35, minSilence: 0.5, padding: 0.1 },
    forte: { thresholdDb: -30, minSilence: 0.3, padding: 0.05 },
  };

  const form = createForm({ profile: "normal", thresholdDb: -35, minSilence: 0.5, padding: 0.1 }, "silencios", "silencios|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  // Un perfil fixo enche os tres valores; con «Personalizado» quedan os que poña o usuario.
  $: preset = PROFILES[$form.profile];
  $: if (preset && ($form.thresholdDb !== preset.thresholdDb || $form.minSilence !== preset.minSilence || $form.padding !== preset.padding)) form.update((f) => ({ ...f, ...preset }));

  const values = (): engine.SilenceValues => ({ thresholdDb: $form.thresholdDb, minSilence: $form.minSilence, padding: $form.padding });

  // O resultado da análise só vale para os valores con que se fixo.
  let analysis: { key: string; data: engine.SilenceAnalysis } | null = null;
  let analyzing = false;
  let analyzeErr = "";
  $: key = `${$form.thresholdDb}|${$form.minSilence}|${$form.padding}`;
  $: current = analysis && analysis.key === key ? analysis.data : null;

  const clock = (s: number) => formatClock(s, 1);
  const percent = (a: number, b: number) => (a > 0 ? Math.round((1 - b / a) * 100) : 0);

  async function analyze() {
    analyzeErr = "";
    analyzing = true;
    const k = key;
    try {
      analysis = { key: k, data: await engine.analyzeSilence(file.path, values()) };
    } catch (e) {
      analysis = null;
      analyzeErr = errText(e);
    } finally {
      analyzing = false;
    }
  }

  async function cut() {
    try {
      const out = await askOutput(file, tr("-sen-silencios"), file.ext);
      if (!out) return;
      const v = values();
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.cutSilence(file.path, out, v, true);
        return `${listo()} ${fileNameOf(r.output_path)}\n${tr("Cortáronse {n} silencios: de {a} a {b} (−{p} %).", { n: r.cuts, a: clock(r.before), b: clock(r.after), p: percent(r.before, r.after) })}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Cortar os silencios")}</h4>
    <GroupNote text={$t("Quita os tramos sen son (pausas, respiracións, silencios longos) e xunta o que queda.")} />

    <Segmented
      label={$t("Que se considera silencio")}
      options={[
        { value: "suave", label: $t("Suave"), detail: "−40 dB · 1 s", hint: $t("Só as pausas longas") },
        { value: "normal", label: $t("Normal"), detail: "−35 dB · 0,5 s", hint: $t("As pausas habituais") },
        { value: "forte", label: $t("Forte"), detail: "−30 dB · 0,3 s", hint: $t("Case todas as pausas") },
        { value: "custom", label: $t("Personalizado"), detail: $t("Valores propios"), hint: $t("Os valores que se queiran") },
      ]}
      bind:value={$form.profile}
    />
    {#if $form.profile === "custom"}
      <Slider label={$t("Volume de corte")} unit="dB" min={-60} max={-15} step={1} bind:value={$form.thresholdDb} def={-35} hint={$t("O que soe por debaixo deste volume conta como silencio. Máis preto de 0 = quita máis (tamén o son baixo).")} />
      <Slider label={$t("Silencio mínimo")} unit="s" min={0.1} max={5} step={0.1} bind:value={$form.minSilence} def={0.5} hint={$t("Só se cortan os silencios que duran isto ou máis.")} />
      <Slider label={$t("Pausa que se deixa")} unit="s" min={0} max={1} step={0.05} bind:value={$form.padding} def={0.1} hint={$t("Silencio que queda a cada lado de cada corte, para que as palabras non soen cortadas de golpe.")} />
    {/if}
  </div>

  <div class="group">
    <button class="btn block" on:click={analyze} disabled={$busy || analyzing}>{analyzing ? $t("Analizando…") : $t("Ver canto se cortaría")}</button>
    {#if analyzeErr}
      <p class="warn">{analyzeErr}</p>
    {:else if current}
      {#if current.cuts.length}
        <p class="result">
          {$t("{n} silencios: quitaríanse {a} de {b} (−{p} %). O resultado duraría {c}.", {
            n: current.cuts.length,
            a: clock(current.duration - current.result_duration),
            b: clock(current.duration),
            p: percent(current.duration, current.result_duration),
            c: clock(current.result_duration),
          })}
        </p>
      {:else}
        <p class="result">{$t("Con estes valores non hai silencios que cortar. Proba a subir o volume de corte ou a baixar o silencio mínimo.")}</p>
      {/if}
    {/if}
  </div>

  <button class="btn primary block" on:click={cut} disabled={$busy || analyzing || (!!current && !current.cuts.length)}>{$t("Cortar os silencios")}</button>
  <JobStatus {job} />
</div>

<style>
  .result { margin: 0; font-size: var(--fs-sm); color: var(--text-muted); }
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); }
</style>
