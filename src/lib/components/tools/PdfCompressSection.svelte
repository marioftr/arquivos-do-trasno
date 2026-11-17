<script lang="ts">
  import GroupNote from "../ui/GroupNote.svelte";
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
  import JobStatus from "../ui/JobStatus.svelte";
  import Segmented from "../ui/Segmented.svelte";

  export let file: FileEntry;

  /** Niveis de compresión (por iso en feminino: «compresión pouca, recomendada, extrema»). «Personalizado» deixa poñer os dous valores á man. */
  const LEVELS: Record<string, { quality: number; scale: number }> = {
    low: { quality: 85, scale: 100 },
    mid: { quality: 70, scale: 75 },
    high: { quality: 45, scale: 50 },
  };

  const form = createForm({ level: "mid", quality: 70, scale: 75 }, "pdf-comprimir", "pdfcompress|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  // Un nivel fixo enche calidade e tamaño; con «Personalizado» quedan os que poña o usuario.
  $: preset = LEVELS[$form.level];
  $: if (preset && ($form.quality !== preset.quality || $form.scale !== preset.scale)) form.update((f) => ({ ...f, ...preset }));

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
          ? tr("Listo! Deixei o PDF en {to} ({pct} % menos; antes eran {from}).", { to: formatBytes(after), pct, from: formatBytes(before) })
          : tr("Fixen o que puiden, pero este PDF xa estaba moi apretado: {to} (antes {from}).", { to: formatBytes(after), from: formatBytes(before) });
      });
    } catch (e) {
      job.say(false, `${fallo("comprimindo")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <Segmented
      label={$t("Nivel de compresión")}
      options={[
        { value: "low", label: $t("Pouca"), detail: $t("Calidade 85 %"), hint: $t("Mantén case todo; só limpa e comprime o que se pode sen perdas") },
        { value: "mid", label: $t("Recomendada"), detail: $t("Calidade 70 %"), hint: $t("Bo equilibrio entre peso e calidade") },
        { value: "high", label: $t("Extrema"), detail: $t("Calidade 45 %"), hint: $t("O máis pequeno posible; as imaxes perden nitidez") },
        { value: "custom", label: $t("Personalizado") },
      ]}
      bind:value={$form.level}
    />
    {#if $form.level === "custom"}
      <h4 class="group-title">{$t("Imaxes dentro do PDF")}</h4>
      <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={70} hint={$t("Máis alto: máis detalle e máis peso.")} />
      <Slider label={$t("Tamaño das imaxes")} unit="%" min={20} max={100} step={5} bind:value={$form.scale} def={75} hint={$t("100 % = sen reducir")} />
    {/if}
    <GroupNote text={$t("Tamén comprime o resto do contido e quita o que non se usa.")} />
  </div>
  <button class="btn primary block" on:click={run} disabled={$busy}>{$t("Comprimir PDF")}</button>
  <JobStatus {job} />
</div>
