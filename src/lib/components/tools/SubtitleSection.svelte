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
  import Segmented from "../ui/Segmented.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import TextField from "../ui/TextField.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  /** Cambios de cadencia habituais (lexenda feita para → vídeo de). «Personalizado» deixa poñer os dous valores. */
  const FPS_PAIRS: Record<string, [number, number]> = {
    a: [23.976, 25],
    b: [25, 23.976],
    c: [24, 25],
    d: [25, 24],
  };

  const form = createForm({ how: "shift", seconds: 0, fpsPair: "a", fromFps: 23.976, toFps: 25, firstAt: "", lastAt: "" }, "lexendas", "lexendas|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  // Un cambio de cadencia fixo enche os dous valores; con «Personalizado» quedan os que poña o usuario.
  $: pair = FPS_PAIRS[$form.fpsPair];
  $: if (pair && ($form.fromFps !== pair[0] || $form.toFps !== pair[1])) form.update((f) => ({ ...f, fromFps: pair[0], toFps: pair[1] }));

  let info: engine.SubtitleInfo | null = null;
  let infoErr = "";
  onMount(() => {
    engine
      .subtitleInfo(file.path)
      .then((i) => (info = i))
      .catch((e) => (infoErr = errText(e)));
  });

  /** «1:02:03,5», «02:03.250» ou «123,5» (segundos) → milisegundos; `null` se non se entende. */
  function parseTime(raw: string): number | null {
    const s = raw.trim().replace(",", ".");
    if (!s) return null;
    const parts = s.split(":");
    if (parts.length > 3 || parts.some((p) => !/^\d+(\.\d+)?$/.test(p))) return null;
    const nums = parts.map(Number);
    const sec = nums.reduce((acc, n) => acc * 60 + n, 0);
    return Math.round(sec * 1000);
  }
  const clock = (ms: number) => {
    const v = Math.max(0, Math.round(ms));
    const pad = (n: number, l = 2) => String(n).padStart(l, "0");
    return `${pad(Math.floor(v / 3_600_000))}:${pad(Math.floor(v / 60_000) % 60)}:${pad(Math.floor(v / 1000) % 60)},${pad(v % 1000, 3)}`;
  };

  async function compute(): Promise<engine.Retime> {
    const f = $form;
    if (f.how === "shift") return { shift_ms: Math.round((f.seconds ?? 0) * 1000), factor: 1 };
    if (f.how === "fps") return engine.subtitleFpsChange(f.fromFps, f.toFps);
    const a = parseTime(f.firstAt);
    const b = parseTime(f.lastAt);
    if (a === null || b === null) throw new Error(tr("Escribe os dous tempos coma «0:01:02,5» (horas:minutos:segundos)."));
    if (!info) throw new Error(tr("Aínda non se leu a lexenda."));
    return engine.subtitleTwoPoints(info.first_start, info.last_start, a, b);
  }

  async function run(identity = false) {
    try {
      const r = identity ? { shift_ms: 0, factor: 1 } : await compute();
      const out = await askOutput(file, tr(identity ? "-utf8" : "-resincronizada"), file.ext);
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const done = await engine.retimeSubtitle(file.path, out, r, true);
        const dropped = done.dropped ? " " + tr("{n} falas quedaron antes do principio e descartáronse.", { n: done.dropped }) : "";
        return `${listo()} ${fileNameOf(done.output_path)}\n${tr("{n} falas.", { n: done.cues })}${dropped}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Resincronizar a lexenda")}</h4>
    <GroupNote text={$t("Se a lexenda sae adiantada, atrasada ou vai perdéndose pouco a pouco, aquí axústase.")} />
    {#if info}
      <p class="muted info">
        {$t("{n} falas · a primeira empeza en {a} e a última en {b}.", { n: info.cues, a: clock(info.first_start), b: clock(info.last_start) })}
      </p>
      {#if info.encoding !== "UTF-8"}
        <p class="warn">{$t("Esta lexenda está en {enc}, non en UTF-8: nalgúns reprodutores os acentos e os «ñ» saen mal. Ao gardala aquí quedará en UTF-8.", { enc: info.encoding })}</p>
        <button class="btn block" on:click={() => run(true)} disabled={$busy}>{$t("Gardar en UTF-8 sen cambiar nada máis")}</button>
      {/if}
    {:else if infoErr}
      <p class="warn">{infoErr}</p>
    {/if}

    <Segmented
      label={$t("Que lle pasa")}
      info={$t("Desprazar: vai toda igual de adiantada ou atrasada. Cambiar a cadencia: a lexenda foi feita para un vídeo con outra cadencia (23,976, 24, 25…) e vai perdendo o paso. Dous puntos: se vai perdendo o paso de calquera xeito, indicas cando debe aparecer a primeira e a última fala e o trasno axusta todo.")}
      options={[
        { value: "shift", label: $t("Desprazar"), hint: $t("Toda igual de adiantada ou atrasada") },
        { value: "fps", label: $t("Cadencia"), hint: $t("Vai perdendo o paso aos poucos") },
        { value: "points", label: $t("Dous puntos"), hint: $t("Indicas onde caen a primeira e a última fala") },
      ]}
      bind:value={$form.how}
    />

    {#if $form.how === "shift"}
      <NumberField label={$t("Desprazamento")} unit="s" step={0.1} min={-3600} max={3600} bind:value={$form.seconds} def={0} hint={$t("Positivo = as falas saen máis tarde; negativo = máis cedo.")} />
    {:else if $form.how === "fps"}
      <Segmented
        label={$t("Cambio de cadencia")}
        info={$t("Primeiro, a cadencia para a que se fixo a lexenda; despois, a do vídeo no que a vas usar. Personalizado: os valores que se queiran.")}
        options={[
          { value: "a", label: "23,976 → 25" },
          { value: "b", label: "25 → 23,976" },
          { value: "c", label: "24 → 25" },
          { value: "d", label: "25 → 24" },
          { value: "custom", label: $t("Personalizado") },
        ]}
        bind:value={$form.fpsPair}
      />
      {#if $form.fpsPair === "custom"}
        <div class="pair">
          <NumberField label={$t("Lexenda feita para")} unit="fps" step={0.001} min={1} max={240} bind:value={$form.fromFps} def={23.976} />
          <NumberField label={$t("Vídeo de")} unit="fps" step={0.001} min={1} max={240} bind:value={$form.toFps} def={25} />
        </div>
      {/if}
    {:else}
      <TextField label={$t("A primeira fala debe saír en")} placeholder="0:00:02,500" bind:value={$form.firstAt} hint={info ? $t("Agora sae en {a}.", { a: clock(info.first_start) }) : ""} />
      <TextField label={$t("A última fala debe saír en")} placeholder="1:32:10,000" bind:value={$form.lastAt} hint={info ? $t("Agora sae en {a}.", { a: clock(info.last_start) }) : ""} />
    {/if}
  </div>

  <button class="btn primary block" on:click={() => run(false)} disabled={$busy || !info}>{$t("Resincronizar")}</button>
  <JobStatus {job} />
</div>

<style>
  .info { margin: 0; font-size: var(--fs-sm); }
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); }
</style>
