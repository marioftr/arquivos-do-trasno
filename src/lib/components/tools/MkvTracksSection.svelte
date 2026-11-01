<script lang="ts">
  import { onMount } from "svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  interface Row {
    id: number;
    type: string;
    codec: string;
    detail: string;
    keep: boolean;
    name: string;
    language: string;
    isDefault: boolean;
    orig: { name: string; language: string; isDefault: boolean };
  }

  let rows: Row[] = [];
  let loading = true;
  let loadError = "";
  const job = createJob();
  const busy = job.busy;

  const KIND: Record<string, string> = { video: "Vídeo", audio: "Audio", subtitles: "Subtítulos" };

  onMount(async () => {
    try {
      const list = await engine.mkvTracks(file.path);
      rows = list.map((x) => {
        const p = x.properties ?? {};
        const row = {
          id: x.id,
          type: x.type,
          codec: x.codec,
          detail: p.pixel_dimensions ?? (p.audio_channels ? `${p.audio_channels} ch` : ""),
          keep: true,
          name: p.track_name ?? "",
          language: p.language ?? "",
          isDefault: !!p.default_track,
        };
        return { ...row, orig: { name: row.name, language: row.language, isDefault: row.isDefault } };
      });
    } catch (e) {
      loadError = errText(e);
    } finally {
      loading = false;
    }
  });

  $: kept = rows.filter((r) => r.keep);
  $: changed = rows.some((r) => !r.keep || r.name !== r.orig.name || r.language !== r.orig.language || r.isDefault !== r.orig.isDefault);

  // Só pode haber unha pista por defecto de cada tipo.
  function setDefault(r: Row) {
    rows = rows.map((x) => (x.type === r.type ? { ...x, isDefault: x.id === r.id ? !r.isDefault : false } : x));
  }

  async function save() {
    try {
      const out = await askOutput(file, tr("-pistas"), file.ext === "webm" || file.ext === "mka" ? file.ext : "mkv");
      if (!out) return;
      const edits = rows
        .filter((r) => r.keep)
        .map((r) => ({
          id: r.id,
          name: r.name !== r.orig.name ? r.name : undefined,
          language: r.language !== r.orig.language ? r.language : undefined,
          default: r.isDefault !== r.orig.isDefault ? r.isDefault : undefined,
        }));
      await job.run(TRASNO.traballando, fallo("editando"), async () => `${listo()} ${fileNameOf(await engine.mkvRemux(file.path, out, kept.map((r) => r.id), edits, true))}`);
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Pistas do ficheiro")}</h4>
    <p class="group-note">{$t("Quita as pistas que non queiras e cambia o nome ou o idioma. Non se recodifica nada: non hai perda de calidade e o orixinal non se toca.")}</p>
    {#if loadError}
      <p class="group-note">{loadError}</p>
    {:else if loading}
      <p class="group-note">{$t("O trasno está lendo as pistas…")}</p>
    {:else}
      {#each rows as r (r.id)}
        <div class="track" class:off={!r.keep}>
          <div class="top">
            <Toggle label={`${$t(KIND[r.type] ?? "Outra")} · ${r.codec}${r.detail ? " · " + r.detail : ""}`} bind:checked={r.keep} />
          </div>
          {#if r.keep}
            <div class="fields">
              <input class="txt" placeholder={$t("Nome da pista")} bind:value={r.name} aria-label={$t("Nome da pista")} />
              <input class="txt lang" placeholder="glg" maxlength="3" bind:value={r.language} aria-label={$t("Idioma (código de 3 letras)")} title={$t("Idioma (código de 3 letras)")} />
              {#if r.type !== "video"}
                <button class="btn" class:primary={r.isDefault} on:click={() => setDefault(r)} title={$t("Pista que se reproduce por defecto")}>{$t("Por defecto")}</button>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
  <button class="btn primary block" on:click={save} disabled={$busy || loading || !!loadError || !changed || kept.length === 0}>{$t("Gardar unha copia")}</button>
  <JobStatus {job} />
</div>

<style>
  .track { border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 0.55rem 0.7rem; background: var(--bg-input); display: flex; flex-direction: column; gap: 0.5rem; }
  .track.off { opacity: 0.55; }
  .fields { display: flex; gap: 0.4rem; }
  .txt { flex: 1; min-width: 0; height: var(--control-h); background: var(--bg); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0 0.6em; font: inherit; font-size: 0.88rem; }
  .txt.lang { flex: 0 0 4.2rem; text-align: center; }
  .txt:focus { outline: none; border-color: var(--accent); }
</style>
