<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Segmented from "../ui/Segmented.svelte";
  import TextField from "../ui/TextField.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Subtítulos e vídeo: incrustalos como pista, queimalos na imaxe ou sacar unha pista a ficheiro. */
  export let file: FileEntry;
  export let mode: "embed" | "burn" | "extract" = "embed";

  const job = createJob();
  const busy = job.busy;

  // ---- subtítulos escollidos (incrustar e queimar)
  let subsPath = "";
  let subsInfo: engine.SubtitleInfo | null = null;
  let subsErr = "";
  let language = "";
  async function chooseSubs() {
    const picked = await open({ multiple: false, filters: [{ name: tr("Subtítulos"), extensions: ["srt", "vtt", "ass", "ssa"] }] });
    if (typeof picked === "string") await setSubs(picked);
  }
  async function setSubs(picked: string) {
    subsPath = picked;
    subsInfo = null;
    subsErr = "";
    try {
      subsInfo = await engine.subtitleInfo(picked);
    } catch (e) {
      subsErr = errText(e);
    }
  }

  // ---- pistas do vídeo (extraer)
  let tracks: engine.SubtitleTrack[] | null = null;
  let position = 0;
  let format: "srt" | "vtt" | "ass" = "srt";
  onMount(() => {
    // Só en desenvolvemento: as probas da app real non poden manexar o diálogo nativo de ficheiros.
    if (import.meta.env.DEV) (window as any).__trasnoSubs = setSubs;
    engine
      .subtitleTracks(file.path)
      .then((l) => {
        tracks = l;
        position = l.find((x) => x.text_based)?.position ?? 0;
      })
      .catch(() => (tracks = []));
  });
  onDestroy(() => {
    if (import.meta.env.DEV) delete (window as any).__trasnoSubs;
  });
  const trackName = (x: engine.SubtitleTrack) => [x.language ? x.language.toUpperCase() : tr("Sen idioma"), x.title, x.codec].filter(Boolean).join(" · ");

  const KEEPS = ["mp4", "m4v", "mov", "mkv", "webm"];
  const ext = file.ext.toLowerCase();

  async function run() {
    try {
      if (mode === "extract") {
        const out = await askOutput(file, tr("-subtitulos"), format);
        if (!out) return;
        await job.run(TRASNO.traballando, fallo("editando"), async () => {
          const r = await engine.subtitleExtract(file.path, position, out, true);
          return `${listo()} ${fileNameOf(r.output)}\n${tr("{n} {n:fala|falas}.", { n: r.cues })}`;
        });
        return;
      }
      const embed = mode === "embed";
      const outExt = KEEPS.includes(ext) ? ext : embed ? "mkv" : "mp4";
      const out = await askOutput(file, embed ? tr("-subtitulado") : tr("-queimado"), outExt);
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = embed ? await engine.subtitleEmbed(file.path, subsPath, out, language.trim().toLowerCase(), true) : await engine.subtitleBurn(file.path, subsPath, out, true);
        return `${listo()} ${fileNameOf(r.output)}\n${tr("{n} {n:fala|falas}.", { n: r.cues })}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
  $: ready = mode === "extract" ? !!tracks?.[position]?.text_based : !!subsInfo;
</script>

<div class="stack">
  {#if mode === "extract"}
    <div class="group">
      {#if tracks === null}
        <p class="muted small">{$t("Lendo as pistas…")}</p>
      {:else if !tracks.length}
        <p class="muted small">{$t("Este vídeo non ten pistas de subtítulos.")}</p>
      {:else}
        <PanelInfo text={$t("Sácase unha pista de subtítulos a ficheiro. As pistas de imaxe (PGS, DVD) non se poden pasar a texto.")} />
        <ul class="tracks" role="radiogroup" aria-label={$t("Pista")}>
          {#each tracks as x}
            <li>
              <button role="radio" aria-checked={position === x.position} class:on={position === x.position} disabled={!x.text_based} on:click={() => (position = x.position)}>
                <span>{trackName(x)}</span>
                {#if !x.text_based}<small>{$t("Imaxe")}</small>{/if}
              </button>
            </li>
          {/each}
        </ul>
        <Segmented
          label={$t("Formato")}
          options={[
            { value: "srt", label: "SRT" },
            { value: "vtt", label: "WebVTT" },
            { value: "ass", label: "ASS" },
          ]}
          bind:value={format}
        />
      {/if}
    </div>
    <button class="btn primary block" on:click={run} disabled={$busy || !ready}>{$t("Sacar a ficheiro")}</button>
  {:else}
    <div class="group">
      <PanelInfo
        text={mode === "embed"
          ? $t("Engade uns subtítulos como pista que se acende e apaga no reprodutor.")
          : $t("Pasan a formar parte da imaxe: saen en calquera reprodutor, pero non se poden apagar. Recodifica o vídeo.")}
      />
      <button class="btn block" on:click={chooseSubs} disabled={$busy}>{subsPath ? fileNameOf(subsPath) : $t("Escoller subtítulos…")}</button>
      {#if subsInfo}
        <p class="muted small">{$t("{n} {n:fala|falas} · {enc}", { n: subsInfo.cues, enc: subsInfo.encoding })}</p>
      {:else if subsErr}
        <p class="warn">{subsErr}</p>
      {/if}
      {#if mode === "embed"}
        <TextField label={$t("Idioma")} placeholder="glg" bind:value={language} hint={$t("Código de tres letras (glg, spa, eng). Opcional.")} />
      {/if}
    </div>
    <button class="btn primary block" on:click={run} disabled={$busy || !ready}>{mode === "embed" ? $t("Incrustar") : $t("Queimar")}</button>
  {/if}
  <JobStatus {job} />
</div>

<style>
  .small { margin: 0; font-size: var(--fs-sm); }
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); }
  .tracks { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.3rem; }
  .tracks button { width: 100%; display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; text-align: left; background: transparent; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text); padding: 0.45rem 0.6rem; }
  .tracks button:hover:not(:disabled) { background: var(--bg-hover); border-color: var(--border-strong); }
  .tracks button.on { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 12%, transparent); }
  .tracks button:disabled { opacity: 0.5; }
  .tracks small { color: var(--text-faint); font-size: var(--fs-xs); }
</style>
