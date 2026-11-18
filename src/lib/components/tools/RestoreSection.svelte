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

  // O visor amosa o resultado en directo: a cor aplícase ao vídeo (aproximada) e o resto nun fotograma que se prepara co motor para o
  // instante en que está parado. «Ver o orixinal» devolve o vídeo sen axustes para comparar.
  const ownLook = claim("mediaLook");
  const ownStill = claim("mediaStill");
  let original = false;
  $: mediaLook.set(color && !original ? { brightness: $form.brightness, contrast: $form.contrast, saturation: $form.saturation } : null);
  onDestroy(() => {
    clearTimeout(timer);
    token++;
    ownLook.release(() => mediaLook.set(null));
    ownStill.release(() => mediaStill.set(null));
  });

  let stillAt = -1;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let token = 0;
  $: schedule($form, $mediaTime, original, frameEffects);
  function schedule(_f: unknown, now: number, orig: boolean, effects: boolean) {
    clearTimeout(timer);
    token++; // calquera fotograma que se estea preparando xa non vale
    // Se se move o vídeo ou cambia un axuste, o fotograma amosado xa non vale; prepárase outro cando quede quieto.
    if (get(mediaStill) && (orig || Math.abs(now - stillAt) > 0.005 || !effects)) {
      mediaStill.set(null);
    }
    if (orig || !effects) return;
    timer = setTimeout(render, 350);
  }
  async function render() {
    const mine = ++token;
    const at = get(mediaTime);
    try {
      const url = await viewUrl(await engine.restoreFrame(file.path, at, planOf(get(form))));
      if (mine !== token) return;
      stillAt = at;
      mediaStill.set(url);
    } catch {
      // sen fotograma: o visor segue co vídeo (e a cor aproximada)
    }
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
  <Toggle label={$t("Ver o orixinal")} bind:checked={original} />

  <div class="group">
    <h4 class="group-title">{$t("Imaxe")}</h4>
    <Segmented label={$t("Estabilizar")} info={$t("Compensa o tremor da cámara e achega un pouco a imaxe para non deixar bordos. Só se ve ao exportar.")} options={levels()} bind:value={$form.stabilize} />
    <Segmented label={$t("Quitar ruído")} info={$t("Limpa o gran dos vídeos con pouca luz. Canto máis forte, máis suaviza os detalles finos.")} options={levels()} bind:value={$form.denoise} />
    <Segmented label={$t("Nitidez")} info={$t("Realza os bordos dos vídeos borrosos. Moita nitidez tamén realza o ruído.")} options={levels()} bind:value={$form.sharpen} />
    <Toggle label={$t("Desentrelazar")} bind:checked={$form.deinterlace} hint={$t("Quita as liñas en pente dos vídeos de televisión antiga ou gravados en entrelazado.")} />
    <Toggle label={$t("Quitar parpadeo")} bind:checked={$form.deflicker} hint={$t("Iguala os saltos de luz entre fotogramas, como os das luces fluorescentes.")} />
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Cor")}</h4>
    <Slider label={$t("Brillo")} min={-0.5} max={0.5} step={0.01} bind:value={$form.brightness} def={0} />
    <Slider label={$t("Contraste")} unit="×" min={0.5} max={2} step={0.05} bind:value={$form.contrast} def={1} />
    <Slider label={$t("Saturación")} unit="×" min={0} max={3} step={0.05} bind:value={$form.saturation} def={1} hint={$t("0 = branco e negro")} />
  </div>

  <button class="btn primary block" on:click={exportAll} disabled={$busy || !changed}>{$t("Restaurar")}</button>
  <JobStatus {job} />
</div>

<style>
</style>
