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
  import Levels from "../ui/Levels.svelte";

  export let file: FileEntry;

  /** Niveis de compresión (por iso en feminino: «compresión pouca, recomendada, extrema»); os desprazadores deixan poñer os dous valores á man. */
  const LEVELS = [
    { id: "low", label: "Pouca", detail: "Calidade 85 %", hint: "Imaxes JPG a calidade 85 e sen reducilas: case non se nota", values: { quality: 85, scale: 100 } },
    { id: "mid", label: "Recomendada", detail: "Calidade 70 %", hint: "Imaxes JPG a calidade 70 e ao 75 % do seu tamaño", values: { quality: 70, scale: 75 } },
    { id: "high", label: "Extrema", detail: "Calidade 45 %", hint: "Imaxes JPG a calidade 45 e á metade do seu tamaño: pesa moito menos, pero perden nitidez", values: { quality: 45, scale: 50 } },
  ];

  const form = createForm({ quality: 70, scale: 75 }, "pdf-comprimir", "pdfcompress|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

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
    <Levels
      label={$t("Nivel de compresión")}
      levels={LEVELS.map((l) => ({ ...l, label: $t(l.label), detail: $t(l.detail), hint: $t(l.hint) }))}
      values={{ quality: $form.quality, scale: $form.scale }}
      on:apply={(e) => form.update((f) => ({ ...f, ...e.detail }))}
    />
    <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={70} hint={$t("Calidade das imaxes JPG do PDF. Máis baixa: pesa menos e as imaxes quedan máis borrosas.")} />
    <Slider label={$t("Tamaño das imaxes")} unit="%" min={20} max={100} step={5} bind:value={$form.scale} def={75} hint={$t("Reduce o ancho e o alto das imaxes JPG a esta porcentaxe (50 % = a metade de cada lado, un cuarto dos píxeles). 100 % non as reduce.")} />
    <GroupNote text={$t("Só se reducen as imaxes JPG: texto e debuxos quedan igual. Un PDF sen imaxes case non muda.")} />
  </div>
  <button class="btn primary block" on:click={run} disabled={$busy}>{$t("Comprimir PDF")}</button>
  <JobStatus {job} />
</div>
