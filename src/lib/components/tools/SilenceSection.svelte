<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
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
  import Levels from "../ui/Levels.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  /** Que se considera silencio segundo o tipo de gravación; os desprazadores deixan poñer os tres valores á man. */
  const LEVELS = [
    { id: "suave", label: "Suave", detail: "Pausas longas", hint: "Máis de 1 segundo.", values: { thresholdDb: -40, minSilence: 1, padding: 0.15 } },
    { id: "normal", label: "Normal", detail: "Pausas habituais", hint: "Máis de medio segundo.", values: { thresholdDb: -35, minSilence: 0.5, padding: 0.1 } },
    { id: "forte", label: "Forte", detail: "Case todas", hint: "Máis de 0,3 segundos.", values: { thresholdDb: -30, minSilence: 0.3, padding: 0.05 } },
  ];

  const form = createForm({ thresholdDb: -35, minSilence: 0.5, padding: 0.1 }, "silencios", "silencios|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;


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
        return `${listo()} ${fileNameOf(r.output_path)}\n${tr("Cortáronse {n} {n:silencio|silencios}: de {a} a {b} (−{p} %).", { n: r.cuts, a: clock(r.before), b: clock(r.after), p: percent(r.before, r.after) })}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <PanelInfo text={$t("Quita os tramos sen son e xunta o que queda.")} />

    <Levels
      label={$t("Que pausas se cortan")}
      info={$t("Corta as pausas máis longas ca o tempo indicado. Sempre queda un chisco de silencio a cada lado.")}
      levels={LEVELS.map((l) => ({ ...l, label: $t(l.label), detail: $t(l.detail), hint: $t(l.hint) }))}
      values={{ thresholdDb: $form.thresholdDb, minSilence: $form.minSilence, padding: $form.padding }}
      on:apply={(e) => form.update((f) => ({ ...f, ...e.detail }))}
    />
      <Slider label={$t("Volume de corte")} unit="dB" min={-60} max={-15} step={1} bind:value={$form.thresholdDb} def={-35} hint={$t("O que soe por debaixo deste volume conta como silencio. Máis preto de 0, corta máis.")} />
      <Slider label={$t("Silencio mínimo")} unit="s" min={0.1} max={5} step={0.1} bind:value={$form.minSilence} def={0.5} hint={$t("Só se cortan os silencios que duran isto ou máis.")} />
      <Slider label={$t("Pausa que se deixa")} unit="s" min={0} max={1} step={0.05} bind:value={$form.padding} def={0.1} hint={$t("Silencio que queda a cada lado dun corte, para non cortar as palabras en seco.")} />
  </div>

  <div class="estimate">
    {#if analyzeErr}
      <span class="warn">{analyzeErr}</span>
    {:else if current}
      {#if current.cuts.length}
        <span class="est-main">{clock(current.result_duration)}</span>
        <span class="muted">{$t("orixinal {b}, −{p} % · {n} {n:silencio|silencios}", { b: clock(current.duration), p: percent(current.duration, current.result_duration), n: current.cuts.length })}</span>
      {:else}
        <span class="muted">{$t("Non hai silencios con estes valores.")}</span>
      {/if}
      <button class="btn ghost" on:click={analyze} disabled={$busy || analyzing}>{analyzing ? $t("Analizando…") : $t("Analizar de novo")}</button>
    {:else}
      <button class="btn" on:click={analyze} disabled={$busy || analyzing}>{analyzing ? $t("Analizando…") : $t("Analizar silencios")}</button>
    {/if}
  </div>

  <button class="btn primary block" on:click={cut} disabled={$busy || analyzing || (!!current && !current.cuts.length)}>{$t("Cortar silencios")}</button>
  <JobStatus {job} />
</div>

<style>
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); }
</style>
