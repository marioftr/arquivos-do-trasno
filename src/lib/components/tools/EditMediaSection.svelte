<script lang="ts">
  import { claim } from "../../claim";
  import { errText } from "../../errors";
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import * as engine from "../../engine";
  import { mediaAux, mediaPreview, mediaTime, trimRange, viewUrl, type FileEntry } from "../../files";
  import { fileNameOf, formatClock, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import Levels from "../ui/Levels.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;
  /** «edit»: cortar e velocidade. «sound»: volume, ruído e efectos. Cada parte ten o seu formulario e a súa exportación. */
  export let part: "edit" | "sound" = "edit";

  const isVideo = file.category === "video";
  // A duración chega un chisco despois de abrir o ficheiro (despois de ler os seus datos): ata entón, 0.
  let total = file.duration ?? 0;

  /** Obxectivos de volume segundo onde se vai escoitar o ficheiro; os desprazadores deixan poñer os tres valores á man. */
  const LEVELS = [
    { id: "streaming", label: "Streaming", detail: "−14 LUFS", hint: "Spotify, YouTube e semellantes", values: { lufs: -14, peak: -1, range: 11 } },
    { id: "podcast", label: "Podcast", detail: "−16 LUFS", hint: "Voz para escoitar con auriculares", values: { lufs: -16, peak: -1.5, range: 11 } },
    { id: "emision", label: "Emisión", detail: "−23 LUFS", hint: "Radio e televisión (EBU R128)", values: { lufs: -23, peak: -1, range: 15 } },
  ];

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
    (part === "edit" ? "editar|" : "son|") + file.path,
  );
  onMount(() => useForm(form));
  $: if ((file.duration ?? 0) !== total) {
    const before = total;
    total = file.duration ?? 0;
    // «Fin» aínda estaba no valor de «ata o final»: segue sendo o final co novo tamaño.
    if (!get(form).end || get(form).end === before) form.update((f) => ({ ...f, end: total }));
  }

  // O visor reproduce xa en directo coa velocidade, o volume, os graves, os agudos, o eco e o esvaecer elixidos e amosa o corte sobre a liña de tempo (as súas asas móvense e
  // os campos seguen sempre o que se ve).
  $: mediaPreview.set({
    speed: $form.speed,
    volume: $form.mute ? 0 : $form.normalize ? 1 : $form.volume,
    bass: $form.mute ? 0 : $form.bass,
    treble: $form.mute ? 0 : $form.treble,
    echoMs: !$form.mute && $form.echo ? $form.echoMs : 0,
    fadeIn: !$form.mute && $form.fadeIn ? $form.fade : 0,
    fadeOut: !$form.mute && $form.fadeOut ? $form.fade : 0,
  });
  const r1 = (v: number) => Math.round(v * 10) / 10;
  $: if (total && part === "edit") {
    const r = { start: $form.start, end: $form.end || total };
    const cur = get(trimRange);
    if (!cur || cur.start !== r.start || cur.end !== r.end) trimRange.set(r);
  }
  onMount(() =>
    trimRange.subscribe((r) => {
      if (!r || !total || part !== "edit") return;
      const start = r1(r.start);
      const end = r1(r.end) >= r1(total) ? total : r1(r.end);
      const f = get(form);
      if (f.start !== start || (f.end || total) !== end) form.update((v) => ({ ...v, start, end }));
    }),
  );
  const ownPreview = claim("mediaPreview");
  const ownTrim = claim("trimRange");
  const ownAux = claim("mediaAux");
  onDestroy(() => {
    clearTimeout(auxTimer);
    auxToken++;
    ownAux.release(() => mediaAux.set(null));
    ownPreview.release(() => mediaPreview.set(null));
    ownTrim.release(() => trimRange.set(null));
  });

  // Ruído e son ao revés non se poden facer no navegador: prepárase o son co motor (tarda un pouco) e o visor reprodúceo no seu lugar.
  let auxState: "idle" | "busy" | "error" = "idle";
  let auxMsg = "";
  let auxTimer: ReturnType<typeof setTimeout> | undefined;
  let auxToken = 0;
  $: auxKey = `${!$form.mute && $form.denoise}|${!$form.mute && $form.reverse}|${$form.start}|${$form.end}`;
  $: scheduleAux(auxKey);
  function scheduleAux(_key: string) {
    clearTimeout(auxTimer);
    const f = get(form);
    const denoise = !f.mute && f.denoise;
    const reverse = !f.mute && f.reverse;
    if (!denoise && !reverse) {
      auxToken++;
      mediaAux.set(null);
      auxState = "idle";
      return;
    }
    auxState = "busy";
    auxTimer = setTimeout(async () => {
      const mine = ++auxToken;
      try {
        const end = f.end && (!total || f.end < total - 0.05) ? f.end : null;
        const path = await engine.editAudioPreview(file.path, f.start || 0, end, denoise, reverse);
        if (mine !== auxToken) return;
        mediaAux.set(await viewUrl(path));
        auxState = "idle";
      } catch (e) {
        if (mine !== auxToken) return;
        mediaAux.set(null);
        auxMsg = errText(e);
        auxState = "error";
      }
    }, 700);
  }

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
  const signed = (v: number) => `${v > 0 ? "+" : "−"}${num(Math.abs(v))} dB`;
  /** Os axustes que cambian algo, ordenados como no formulario: nome e valor. */
  $: rows = ((): { k: string; v: string }[] => {
    const p = planOf($form);
    const r: { k: string; v: string }[] = [];
    if (p.start > 0 || p.end !== null) r.push({ k: tr("Tramo"), v: `${formatClock(p.start, 1)} – ${formatClock(p.end ?? total, 1)}` });
    if (p.speed !== 1) r.push({ k: tr("Velocidade"), v: `×${num(p.speed)}` });
    if (p.mute) r.push({ k: tr("Son"), v: tr("Quitado") });
    if (p.normalize) r.push({ k: tr("Volume"), v: `${num(p.normalize.lufs)} LUFS` });
    else if (p.volume !== 1) r.push({ k: tr("Volume"), v: `×${num(p.volume)}` });
    if (p.denoise) r.push({ k: tr("Ruído"), v: tr("Reducido") });
    if (p.reverse) r.push({ k: tr("Son"), v: tr("Ao revés") });
    if (p.fade_in || p.fade_out) r.push({ k: tr("Esvaecer"), v: [p.fade_in ? tr("principio") : "", p.fade_out ? tr("final") : ""].filter(Boolean).join(" + ") + ` · ${num(p.fade_in || p.fade_out)} s` });
    if (p.bass) r.push({ k: tr("Graves"), v: signed(p.bass) });
    if (p.treble) r.push({ k: tr("Agudos"), v: signed(p.treble) });
    if (p.echo_ms) r.push({ k: tr("Eco"), v: `${p.echo_ms} ms` });
    return r;
  })();
  $: changed = rows.length > 0;
  /** Duración do resultado se se coñece a do orixinal. */
  $: outLen = total ? Math.max(0, ($form.end || total) - $form.start) / $form.speed : null;

  async function exportAll() {
    try {
      const out = await askOutput(file, part === "edit" ? tr("-editado") : tr("-son"), file.ext);
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

</script>

<div class="stack">
  {#if part === "edit" && total}
    <div class="group">
      <h4 class="group-title">{$t("Cortar")}</h4>
      <GroupNote text={$t("Arrastra as asas da liña de tempo do visor. Só se garda o tramo elixido.")} />
      <Slider label={$t("Inicio")} unit="s" min={0} max={total} step={0.1} bind:value={$form.start} def={0} />
      <Slider label={$t("Fin")} unit="s" min={0} max={total} step={0.1} bind:value={$form.end} def={total} />
      <div class="pair">
        <button class="btn" on:click={() => form.update((v) => ({ ...v, start: Math.min($mediaTime, v.end || total) }))}>{$t("Inicio: onde está o visor")}</button>
        <button class="btn" on:click={() => form.update((v) => ({ ...v, end: Math.max($mediaTime, v.start) }))}>{$t("Fin: onde está o visor")}</button>
      </div>
    </div>
  {/if}

  {#if part === "edit"}
  <div class="group">
    <h4 class="group-title">{$t("Velocidade")}</h4>
    <Slider label={$t("Factor de velocidade")} unit="×" min={0.25} max={4} step={0.05} bind:value={$form.speed} def={1} hint={$t("2 = o dobre de rápido; 0,5 = a metade")} />
  </div>
  {:else}

  {#if isVideo}
    <div class="group">
      <h4 class="group-title">{$t("Pista de son")}</h4>
      <Toggle label={$t("Quitar o son")} bind:checked={$form.mute} />
    </div>
  {/if}

  <div class="group" class:off={$form.mute}>
    <h4 class="group-title">{$t("Volume")}</h4>
    <Toggle label={$t("Igualar o volume")} bind:checked={$form.normalize} disabled={$form.mute} hint={$t("Leva o volume percibido de todo o ficheiro ao valor elixido (EBU R128). Substitúe o multiplicador de volume.")} />
    {#if $form.normalize}
      <Levels
        label={$t("Perfil")}
        info={$t("LUFS mide o volume percibido: canto máis preto de 0, máis alto.")}
        levels={LEVELS.map((l) => ({ ...l, label: $t(l.label), detail: l.detail, hint: $t(l.hint) }))}
        values={{ lufs: $form.lufs, peak: $form.peak, range: $form.range }}
        on:apply={(e) => form.update((f) => ({ ...f, ...e.detail }))}
      />
        <Slider label={$t("Volume obxectivo")} unit="LUFS" min={-30} max={-8} step={0.5} bind:value={$form.lufs} def={-16} />
        <Slider label={$t("Pico máximo")} unit="dBTP" min={-6} max={0} step={0.5} bind:value={$form.peak} def={-1.5} hint={$t("O pico máis alto permitido. Por baixo de 0 non distorsiona.")} />
        <Slider label={$t("Rango dinámico máximo")} unit="LU" min={1} max={30} step={1} bind:value={$form.range} def={11} hint={$t("Diferenza máxima entre o máis baixo e o máis alto; se hai máis, comprímese.")} />
    {:else}
      <Slider label={$t("Multiplicador de volume")} unit="×" min={0} max={4} step={0.05} bind:value={$form.volume} def={1} disabled={$form.mute} hint={$t("1 = igual que o orixinal")} />
    {/if}
  </div>

  <div class="group" class:off={$form.mute}>
    <h4 class="group-title">{$t("Son")}</h4>
    <Toggle label={$t("Quitar ruído de fondo")} bind:checked={$form.denoise} disabled={$form.mute} hint={$t("Reduce o ruído constante (zumbido, ventilador). O visor tarda uns segundos en preparalo.")} />
    <Toggle label={$t("Dar a volta ao son")} bind:checked={$form.reverse} disabled={$form.mute} hint={$t("O visor tarda uns segundos en preparalo. Máximo 20 minutos de son.")} />
    {#if auxState === "busy"}
      <p class="aux"><span class="spin"></span>{$t("Preparando o son para o visor…")}</p>
    {:else if auxState === "error"}
      <p class="aux err">{auxMsg}</p>
    {/if}
    <Toggle label={$t("Esvaecer ao principio")} bind:checked={$form.fadeIn} disabled={$form.mute} />
    <Toggle label={$t("Esvaecer ao final")} bind:checked={$form.fadeOut} disabled={$form.mute} />
    {#if $form.fadeIn || $form.fadeOut}
      <Slider label={$t("Duración do esvaecer")} unit="s" min={0.1} max={30} step={0.1} bind:value={$form.fade} def={2} />
    {/if}
    <Slider label={$t("Graves")} unit="dB" min={-12} max={12} step={1} bind:value={$form.bass} def={0} disabled={$form.mute} />
    <Slider label={$t("Agudos")} unit="dB" min={-12} max={12} step={1} bind:value={$form.treble} def={0} disabled={$form.mute} />
    <Toggle label={$t("Eco")} bind:checked={$form.echo} disabled={$form.mute} />
    {#if $form.echo}
      <Slider label={$t("Atraso do eco")} unit="ms" min={20} max={2000} step={10} bind:value={$form.echoMs} def={300} />
    {/if}
  </div>
  {/if}

  {#if changed}
    <div class="group">
      <h4 class="group-title">{$t("Así quedará o ficheiro")}</h4>
      <dl class="rows">
        {#each rows as r}
          <dt>{r.k}</dt>
          <dd>{r.v}</dd>
        {/each}
        {#if outLen !== null}
          <dt class="total">{$t("Duración")}</dt>
          <dd class="total">{#if Math.abs(outLen - total) > 0.05}<span class="was">{formatClock(total, 1)}</span>&nbsp;→&nbsp;{/if}{formatClock(outLen, 1)}</dd>
        {/if}
      </dl>
    </div>
  {/if}

  <button class="btn primary block" on:click={exportAll} disabled={$busy || !changed}>{$t("Exportar")}</button>
  <JobStatus {job} />
</div>

<style>
  .off { opacity: 0.45; }
  .aux { display: flex; align-items: center; gap: 0.5rem; margin: 0; font-size: var(--fs-sm); color: var(--text-muted); }
  .aux.err { color: var(--accent-pdf); align-items: flex-start; }
  .spin { width: 12px; height: 12px; border-radius: 50%; border: 2px solid var(--border-strong); border-top-color: var(--accent); animation: giro 0.8s linear infinite; flex: none; }
  @keyframes giro { to { transform: rotate(360deg); } }
  .rows { display: grid; grid-template-columns: auto 1fr; column-gap: 0; row-gap: 0.35rem; margin: 0; font-size: var(--fs-md); }
  .rows dt { color: var(--text-muted); padding-right: 1rem; }
  .rows dd { margin: 0; text-align: right; font-variant-numeric: tabular-nums; user-select: text; }
  .rows .total { padding-top: 0.45rem; border-top: 1px solid var(--border); font-weight: 600; color: var(--text); }
  .was { color: var(--text-faint); font-weight: 400; }
</style>
