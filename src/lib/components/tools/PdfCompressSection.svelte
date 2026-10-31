<script lang="ts">
  import { onMount } from "svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, formatBytes, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import FormBar from "../ui/FormBar.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const LEVELS = [
    { value: "low", label: "Pouca", quality: 85, scale: 100, hint: "Mantén case todo; só limpa e comprime o que se pode sen perdas" },
    { value: "mid", label: "Recomendada", quality: 70, scale: 75, hint: "Bo equilibrio entre peso e calidade" },
    { value: "high", label: "Extrema", quality: 45, scale: 50, hint: "O máis pequeno posible; as imaxes perden nitidez" },
  ];

  const form = createForm({ level: "mid", quality: 70, scale: 75 });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  function pick(level: string) {
    const l = LEVELS.find((x) => x.value === level)!;
    form.update((f) => ({ ...f, level, quality: l.quality, scale: l.scale }));
  }
  $: level = $form.level;

  async function run() {
    try {
      const out = await askOutput(file, tr("-comprimido"), "pdf");
      if (!out) return;
      const f = $form;
      await job.run(TRASNO.comprimindo, fallo("comprimindo"), async () => {
        const done = await engine.pdfCompress(file.path, out, f.quality, f.scale, true);
        const after = await engine.fileSize(done);
        const before = file.size ?? 0;
        const pct = before ? Math.round((1 - after / before) * 100) : 0;
        return pct > 0
          ? tr("Listo! O trasno deixou o PDF en {to} ({pct} % menos; antes eran {from}).", { to: formatBytes(after), pct, from: formatBytes(before) })
          : tr("O trasno fixo o que puido, pero este PDF xa estaba moi apretado: {to} (antes {from}).", { to: formatBytes(after), from: formatBytes(before) });
      });
    } catch (e) {
      job.say(false, `${fallo("comprimindo")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <FormBar {form} />
  <div class="levels">
    {#each LEVELS as l}
      <button class="btn" class:primary={level === l.value} on:click={() => pick(l.value)} title={$t(l.hint)}>{$t(l.label)}</button>
    {/each}
  </div>
  <div class="group">
    <h4 class="group-title">{$t("Imaxes dentro do PDF")}</h4>
    <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={70} />
    <Slider label={$t("Tamaño das imaxes")} unit="%" min={20} max={100} step={5} bind:value={$form.scale} def={75} hint={$t("100 % = sen reducir")} />
    <p class="group-note">{$t("Tamén se comprime o resto do contido e se quita o que non se usa.")}</p>
  </div>
  <button class="btn primary block" on:click={run} disabled={$busy}>{$t("Comprimir PDF")}</button>
  <JobStatus {job} />
</div>

<style>
  .levels { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.4rem; }
</style>
