<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { claim } from "../../claim";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { mediaLook, mediaStill, mediaTime, viewUrl, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Restaurar vídeo: estabilizar, ruído, nitidez, desentrelazar, parpadeo e cor, todo nunha soa exportación. */
  export let file: FileEntry;

  const form = createForm(
    { stabilize: 0, denoise: 0, sharpen: 0, deinterlace: false, deflicker: false, brightness: 0, contrast: 1, saturation: 1 },
    undefined,
    "restaurar|" + file.path,
  );
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  const levels = () => [
    { value: 0, label: tr("Non") },
    { value: 1, label: tr("Suave") },
    { value: 2, label: tr("Normal") },
    { value: 3, label: tr("Forte") },
  ];

  const planOf = (f: typeof $form): engine.RestorePlan => ({ ...f });
  $: color = $form.brightness !== 0 || $form.contrast !== 1 || $form.saturation !== 1;
  $: frameEffects = $form.denoise > 0 || $form.sharpen > 0 || $form.deinterlace || $form.deflicker || color;
  $: changed = frameEffects || $form.stabilize > 0;

  // A cor vese en directo no visor (aproximada); o resto, nun fotograma que se prepara co motor.
  const ownLook = claim("mediaLook");
  const ownStill = claim("mediaStill");
  $: mediaLook.set(color ? { brightness: $form.brightness, contrast: $form.contrast, saturation: $form.saturation } : null);
  onDestroy(() => {
    ownLook.release(() => mediaLook.set(null));
    ownStill.release(() => mediaStill.set(null));
  });

  let showing = false;
  let stillAt = 0;
  let frameBusy = false;
  let frameErr = "";
  let token = 0;
  async function toggleFrame() {
    if (showing) {
      mediaStill.set(null);
      showing = false;
      return;
    }
    const mine = ++token;
    frameBusy = true;
    frameErr = "";
    try {
      const at = get(mediaTime);
      const url = await viewUrl(await engine.restoreFrame(file.path, at, planOf($form)));
      if (mine !== token) return;
      stillAt = at;
      mediaStill.set(url);
      showing = true;
    } catch (e) {
      frameErr = errText(e);
    } finally {
      frameBusy = false;
    }
  }
  // Se se move o vídeo, ou se cambia un axuste, o fotograma amosado xa non vale: volve o orixinal.
  $: if (showing && Math.abs($mediaTime - stillAt) > 0.05) clearStill();
  $: {
    JSON.stringify($form);
    clearStill();
  }
  function clearStill() {
    if (!showing) return;
    token++;
    mediaStill.set(null);
    showing = false;
  }

  async function exportAll() {
    try {
      const out = await askOutput(file, tr("-restaurado"), file.ext);
      if (!out) return;
      const p = planOf($form);
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.restoreApply(file.path, out, p, true);
        const note = r.stabilizer === "deshake" ? `\n${tr("Esta FFmpeg non ten vidstab: estabilicei cun método máis simple.")}` : "";
        return `${listo()} ${fileNameOf(r.output)}${note}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Imaxe")}</h4>
    <GroupNote text={$t("A imaxe recodifícase con calidade alta; o son non se toca.")} />
    <Segmented label={$t("Estabilizar")} info={$t("Compensa o tremor da cámara. Le o vídeo dúas veces e achega un pouco a imaxe para non deixar bordos.")} options={levels()} bind:value={$form.stabilize} />
    <Segmented label={$t("Quitar ruído")} info={$t("Limpa o gran dos vídeos con pouca luz. Canto máis forte, máis suaviza os detalles finos.")} options={levels()} bind:value={$form.denoise} />
    <Segmented label={$t("Nitidez")} info={$t("Realza os bordos dos vídeos borrosos. Moita nitidez tamén realza o ruído.")} options={levels()} bind:value={$form.sharpen} />
    <Toggle label={$t("Desentrelazar")} bind:checked={$form.deinterlace} hint={$t("Quita as liñas en pente dos vídeos de televisión antiga ou gravados en entrelazado.")} />
    <Toggle label={$t("Quitar parpadeo")} bind:checked={$form.deflicker} hint={$t("Iguala os saltos de luz entre fotogramas, como os das luces fluorescentes.")} />
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Cor")}</h4>
    <GroupNote text={$t("O visor amosa a cor en directo, de forma aproximada.")} />
    <Slider label={$t("Brillo")} min={-0.5} max={0.5} step={0.01} bind:value={$form.brightness} def={0} />
    <Slider label={$t("Contraste")} unit="×" min={0.5} max={2} step={0.05} bind:value={$form.contrast} def={1} />
    <Slider label={$t("Saturación")} unit="×" min={0} max={3} step={0.05} bind:value={$form.saturation} def={1} hint={$t("0 = branco e negro")} />
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Comparar")}</h4>
    <GroupNote text={$t("Ensina no visor como queda o fotograma onde estás. A estabilización só se ve ao exportar.")} />
    <button class="btn block" on:click={toggleFrame} disabled={$busy || frameBusy || !frameEffects}>{showing ? $t("Ver o orixinal") : frameBusy ? $t("Preparando o fotograma…") : $t("Ver fotograma restaurado")}</button>
    {#if frameErr}<p class="warn" role="alert">{frameErr}</p>{/if}
  </div>

  <button class="btn primary block" on:click={exportAll} disabled={$busy || !changed}>{$t("Restaurar")}</button>
  <JobStatus {job} />
</div>

<style>
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); }
</style>
