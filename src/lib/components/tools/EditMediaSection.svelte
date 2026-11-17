<script lang="ts">
  import { claim } from "../../claim";
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import * as engine from "../../engine";
  import { mediaPreview, mediaTime, trimRange, viewUrl, type FileEntry } from "../../files";
  import { fileNameOf, formatClock, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const isVideo = file.category === "video";
  const total = file.duration ?? 0;
  /** Segundos que dura a vista previa. */
  const PREVIEW_S = 8;

  /** Obxectivos de volume segundo onde se vai escoitar o ficheiro. «Personalizado» deixa poñer os tres valores á man. */
  const PROFILES: Record<string, { lufs: number; peak: number; range: number }> = {
    streaming: { lufs: -14, peak: -1, range: 11 },
    podcast: { lufs: -16, peak: -1.5, range: 11 },
    emision: { lufs: -23, peak: -1, range: 15 },
  };

  const form = createForm(
    {
      start: 0,
      end: total,
      speed: 1,
      volume: 1,
      mute: false,
      denoise: false,
      reverse: false,
      normalize: false,
      profile: "podcast",
      lufs: -16,
      peak: -1.5,
      range: 11,
      fadeIn: false,
      fadeOut: false,
      fade: 2,
      bass: 0,
      treble: 0,
      echo: false,
      echoMs: 300,
    },
    undefined,
    "editar|" + file.path,
  );
  onMount(() => useForm(form));

  // Un perfil fixo enche os tres valores; con «Personalizado» quedan os que poña o usuario.
  $: preset = PROFILES[$form.profile];
  $: if (preset && ($form.lufs !== preset.lufs || $form.peak !== preset.peak || $form.range !== preset.range)) form.update((f) => ({ ...f, ...preset }));

  // O visor reproduce xa coa velocidade e o volume elixidos e amosa o corte sobre a liña de tempo (as súas asas móvense e
  // os campos seguen sempre o que se ve).
  $: mediaPreview.set({ speed: $form.speed, volume: $form.mute ? 0 : $form.normalize ? 1 : $form.volume });
  const r1 = (v: number) => Math.round(v * 10) / 10;
  $: if (total) {
    const r = { start: $form.start, end: $form.end || total };
    const cur = get(trimRange);
    if (!cur || cur.start !== r.start || cur.end !== r.end) trimRange.set(r);
  }
  onMount(() =>
    trimRange.subscribe((r) => {
      if (!r || !total) return;
      const start = r1(r.start);
      const end = r1(r.end) >= r1(total) ? total : r1(r.end);
      const f = get(form);
      if (f.start !== start || (f.end || total) !== end) form.update((v) => ({ ...v, start, end }));
    }),
  );
  const ownPreview = claim("mediaPreview");
  const ownTrim = claim("trimRange");
  onDestroy(() => {
    ownPreview.release(() => mediaPreview.set(null));
    ownTrim.release(() => trimRange.set(null));
    stopPreview();
  });

  const job = createJob();
  const busy = job.busy;

  const planOf = (f: typeof $form): engine.EditPlan => {
    const sound = !f.mute;
    return {
      start: f.start || 0,
      end: f.end && (!total || f.end < total - 0.05) ? f.end : null,
      speed: f.speed,
      volume: sound && !f.normalize ? f.volume : 1,
      mute: f.mute,
      denoise: sound && f.denoise,
      reverse: sound && f.reverse,
      fade_in: sound && f.fadeIn ? f.fade : 0,
      fade_out: sound && f.fadeOut ? f.fade : 0,
      bass: sound ? f.bass : 0,
      treble: sound ? f.treble : 0,
      echo_ms: sound && f.echo ? f.echoMs : 0,
      normalize: sound && f.normalize ? { lufs: f.lufs, peak: f.peak, range: f.range } : null,
    };
  };

  const num = (v: number) => String(Math.round(v * 100) / 100).replace(".", ",");
  /** Resumo do que se vai facer, en palabras. */
  $: changes = ((): string[] => {
    const p = planOf($form);
    const c: string[] = [];
    if (p.start > 0 || p.end !== null) c.push(tr("cortar de {a} a {b}", { a: formatClock(p.start, 1), b: formatClock(p.end ?? total, 1) }));
    if (p.speed !== 1) c.push(tr("velocidade ×{v}", { v: num(p.speed) }));
    if (p.mute) c.push(tr("sen son"));
    if (p.normalize) c.push(tr("volume igualado a {v} LUFS", { v: num(p.normalize.lufs) }));
    else if (p.volume !== 1) c.push(tr("volume ×{v}", { v: num(p.volume) }));
    if (p.denoise) c.push(tr("sen ruído"));
    if (p.reverse) c.push(tr("son ao revés"));
    if (p.fade_in) c.push(tr("esvaecer ao principio"));
    if (p.fade_out) c.push(tr("esvaecer ao final"));
    if (p.bass) c.push(tr("graves {v} dB", { v: num(p.bass) }));
    if (p.treble) c.push(tr("agudos {v} dB", { v: num(p.treble) }));
    if (p.echo_ms) c.push(tr("eco"));
    return c;
  })();
  /** Duración do resultado se se coñece a do orixinal. */
  $: outLen = total ? Math.max(0, ($form.end || total) - $form.start) / $form.speed : null;
  /** A imaxe só se cópia se non se corta nin se cambia a velocidade. */
  $: imageCopied = onlyTrim || !(planOf($form).start > 0 || planOf($form).end !== null || $form.speed !== 1);
  /** Só se corta: gárdase sen recodificar. */
  $: onlyTrim = changes.length === 1 && (planOf($form).start > 0 || planOf($form).end !== null);

  async function exportAll() {
    try {
      const out = await askOutput(file, tr("-editado"), file.ext);
      if (!out) return;
      const p = planOf($form);
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.editApply(file.path, out, p, true);
        const how = isVideo ? (r.video_copied ? tr("A imaxe copiouse sen recodificar.") : tr("A imaxe recodificouse con calidade alta.")) : "";
        const len = r.duration ? tr("Dura {t}.", { t: formatClock(r.duration, 1) }) : "";
        return `${listo()} ${fileNameOf(r.output)}\n${[len, how].filter(Boolean).join(" ")}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }

  // ---- vista previa: un anaco curto co plan aplicado
  let previewing = false;
  let previewUrl = "";
  let previewErr = "";
  function stopPreview() {
    previewUrl = "";
  }
  async function preview() {
    previewErr = "";
    previewing = true;
    previewUrl = "";
    try {
      const path = await engine.editPreview(file.path, planOf($form), get(mediaTime), PREVIEW_S, isVideo);
      previewUrl = await viewUrl(path);
    } catch (e) {
      previewErr = errText(e);
    } finally {
      previewing = false;
    }
  }
</script>

<div class="stack">
  {#if total}
    <div class="group">
      <h4 class="group-title">{$t("Cortar")}</h4>
      <GroupNote text={$t("Arrastra as asas da liña de tempo do visor ou usa os desprazadores. Só se garda o tramo elixido.")} />
      <Slider label={$t("Inicio")} unit="s" min={0} max={total} step={0.1} bind:value={$form.start} def={0} />
      <Slider label={$t("Fin")} unit="s" min={0} max={total} step={0.1} bind:value={$form.end} def={total} />
      <div class="pair">
        <button class="btn" on:click={() => form.update((v) => ({ ...v, start: Math.min($mediaTime, v.end || total) }))}>{$t("Inicio: onde está o visor")}</button>
        <button class="btn" on:click={() => form.update((v) => ({ ...v, end: Math.max($mediaTime, v.start) }))}>{$t("Fin: onde está o visor")}</button>
      </div>
    </div>
  {/if}

  <div class="group">
    <h4 class="group-title">{$t("Velocidade")}</h4>
    <Slider label={$t("Factor de velocidade")} unit="×" min={0.25} max={4} step={0.05} bind:value={$form.speed} def={1} hint={$t("2 = o dobre de rápido; 0,5 = a metade")} />
  </div>

  {#if isVideo}
    <div class="group">
      <h4 class="group-title">{$t("Pista de son")}</h4>
      <Toggle label={$t("Quitar o son")} bind:checked={$form.mute} hint={$t("O vídeo queda sen pista de son e os axustes de son desaparecen.")} />
    </div>
  {/if}

  <div class="group" class:off={$form.mute}>
    <h4 class="group-title">{$t("Volume")}</h4>
    <Toggle label={$t("Igualar o volume")} bind:checked={$form.normalize} disabled={$form.mute} hint={$t("Iguala o volume percibido do ficheiro enteiro (EBU R128): mídese e despois súbese ou báixase todo o mesmo, así o que era baixo segue sendo baixo. Substitúe o multiplicador de volume.")} />
    {#if $form.normalize}
      <Segmented
        label={$t("Perfil")}
        info={$t("Streaming: −14 LUFS (Spotify, YouTube). Podcast: −16 LUFS. Emisión: −23 LUFS (radio e televisión). Personalizado: os valores que se queiran.")}
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
    {:else}
      <Slider label={$t("Multiplicador de volume")} unit="×" min={0} max={4} step={0.05} bind:value={$form.volume} def={1} disabled={$form.mute} hint={$t("1 = igual que o orixinal")} />
    {/if}
  </div>

  <div class="group" class:off={$form.mute}>
    <h4 class="group-title">{$t("Son")}</h4>
    <Toggle label={$t("Quitar ruído de fondo")} bind:checked={$form.denoise} disabled={$form.mute} hint={$t("Reduce o ruído constante (zumbido, ventilador). Non fai milagres coa voz doutras persoas.")} />
    <Toggle label={$t("Dar a volta ao son")} bind:checked={$form.reverse} disabled={$form.mute} />
    <Toggle label={$t("Esvaecer ao principio")} bind:checked={$form.fadeIn} disabled={$form.mute} />
    <Toggle label={$t("Esvaecer ao final")} bind:checked={$form.fadeOut} disabled={$form.mute} />
    {#if $form.fadeIn || $form.fadeOut}
      <Slider label={$t("Duración do esvaecer")} unit="s" min={0.1} max={30} step={0.1} bind:value={$form.fade} def={2} />
    {/if}
    <Slider label={$t("Graves")} unit="dB" min={-12} max={12} step={1} bind:value={$form.bass} def={0} disabled={$form.mute} hint={$t("0 = sen cambios. Positivo = máis graves; negativo = menos.")} />
    <Slider label={$t("Agudos")} unit="dB" min={-12} max={12} step={1} bind:value={$form.treble} def={0} disabled={$form.mute} hint={$t("0 = sen cambios. Positivo = máis agudos; negativo = menos.")} />
    <Toggle label={$t("Eco")} bind:checked={$form.echo} disabled={$form.mute} />
    {#if $form.echo}
      <Slider label={$t("Atraso do eco")} unit="ms" min={20} max={2000} step={10} bind:value={$form.echoMs} def={300} />
    {/if}
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Antes de exportar")}</h4>
    {#if changes.length}
      <p class="sum">{$t("Vaise aplicar: {lista}.", { lista: changes.join(" · ") })}{#if outLen !== null && !onlyTrim}&nbsp;{$t("Resultado: uns {t}.", { t: formatClock(outLen, 1) })}{/if}</p>
      {#if isVideo}
        <p class="sum muted">{imageCopied ? $t("A imaxe cópiase sen recodificar.") : $t("A imaxe recodifícase con moita calidade.")}</p>
      {/if}
    {:else}
      <p class="sum muted">{$t("Aínda non cambiaches nada: o resultado sería igual que o orixinal.")}</p>
    {/if}
    <button class="btn" on:click={preview} disabled={$busy || previewing || !changes.length || onlyTrim} title={$t("Aplica os axustes a uns segundos desde onde está o visor, para oír como quedan")}>
      {previewing ? $t("Preparando a vista previa…") : $t("Probar uns segundos ({s} s)", { s: PREVIEW_S })}
    </button>
    {#if previewUrl}
      {#if isVideo}
        <!-- svelte-ignore a11y_media_has_caption -->
        <video class="pv" src={previewUrl} controls autoplay></video>
      {:else}
        <audio class="pv" src={previewUrl} controls autoplay></audio>
      {/if}
      {#if $form.normalize}<p class="sum muted">{$t("O volume igualado só se nota ao exportar: mide o ficheiro enteiro.")}</p>{/if}
    {/if}
    {#if previewErr}<p class="sum err">{previewErr}</p>{/if}
  </div>

  <button class="btn primary block" on:click={exportAll} disabled={$busy || !changes.length}>{$t("Exportar")}</button>
  <JobStatus {job} />
</div>

<style>
  .off { opacity: 0.45; }
  .sum { margin: 0; font-size: var(--fs-sm); line-height: 1.45; user-select: text; }
  .err { color: var(--accent-video); }
  .pv { width: 100%; max-height: 220px; border-radius: var(--radius-sm); background: #000; }
</style>
