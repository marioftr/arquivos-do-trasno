<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import { imageNatural, type FileEntry } from "../../files";
  import { fallo, fileNameOf, formatBytes, listo, TRASNO, readOnlyLabel } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askDir } from "./shared";
  import LockNote from "../ui/LockNote.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Paquete de iconas: unha imaxe vira o cartafol de iconas da app (Windows, macOS), da web e PNG de varios tamaños. */
  export let file: FileEntry;

  const form = createForm({ windows: true, mac: true, web: true, png: true, fit: 0 }, "iconas", "iconas|" + file.path);
  onMount(() => useForm(form));
  $: v = $form;

  const job = createJob();
  const busy = job.busy;
  let report: engine.IconReport | null = null;
  onDestroy(() => job.clear());

  let animated = false;
  $: void engine.imageIsAnimated(file.path).then((a) => (animated = a)).catch(() => (animated = false));

  $: W = $imageNatural.w || file.width || 0;
  $: H = $imageNatural.h || file.height || 0;
  $: squareSrc = !W || !H || W === H;
  $: any = v.windows || v.mac || v.web || v.png;
  $: small = !!W && !!H && Math.min(W, H) < 1024;

  async function run() {
    const dir = await askDir(file.path);
    if (!dir) return;
    report = null;
    const stem = file.name.replace(/\.[^.]+$/, "");
    try {
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.makeIconPack(file.path, dir, stem, { windows: v.windows, mac: v.mac, web: v.web, png: v.png, fit: v.fit === 1 });
        report = r;
        return `${listo()} ${tr("Iconas en {dir}.", { dir: fileNameOf(r.folder) })}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  {#if animated}<LockNote label={readOnlyLabel()} text={tr("Este {fmt} é animado: unha icona só leva un fotograma, así que non se pode facer o paquete.", { fmt: file.ext.toUpperCase() })} />{/if}
  <div class="locked-tools" class:on={animated} inert={animated}>
    <div class="group">
      <h4 class="group-title">{$t("Que se crea")}</h4>
      <Toggle label={$t("Windows")} bind:checked={$form.windows} hint={$t("icon.ico con 7 tamaños, de 16 a 256 px.")} />
      <Toggle label={$t("macOS")} bind:checked={$form.mac} hint={$t("icon.icns con 7 tamaños, de 16 a 1024 px.")} />
      <Toggle label={$t("Web")} bind:checked={$form.web} hint={$t("favicon.ico e PNG, apple-touch-icon, iconas de Android e site.webmanifest.")} />
      <Toggle label={$t("PNG soltos")} bind:checked={$form.png} hint={$t("Un PNG por tamaño: 16, 32, 48, 64, 128, 256, 512 e 1024 px.")} />
    </div>
    <div class="locked-tools" class:on={squareSrc} inert={squareSrc}>
      <Segmented
        label={$t("Forma")}
        info={$t("As iconas son cadradas. Cortar polo centro enche o cadrado; Encaixar deixa a imaxe enteira con marxes transparentes.")}
        options={[
          { value: 0, label: $t("Cortar polo centro") },
          { value: 1, label: $t("Encaixar") },
        ]}
        bind:value={$form.fit}
      />
    </div>

    <button class="btn primary block" on:click={run} disabled={$busy || animated || !any}>{$t("Crear o paquete de iconas")}</button>
  </div>

  {#if small && !animated}<p class="muted">{$t("A imaxe mide menos de 1024 px: as iconas grandes sairán ampliadas.")}</p>{/if}
  {#if report}
    <ul class="results">
      <li><span class="rname">{$t("Cartafol")}</span><span class="rstate">{fileNameOf(report.folder)}</span></li>
      <li><span class="rname">{$t("Ficheiros")}</span><span class="rstate">{report.files.length}</span></li>
      <li><span class="rname">{$t("Peso")}</span><span class="rstate">{formatBytes(report.bytes_written)}</span></li>
    </ul>
  {/if}
  <JobStatus {job} />
</div>
