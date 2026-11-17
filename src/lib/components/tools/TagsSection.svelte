<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { viewUrl, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import TextField from "../ui/TextField.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Etiquetas dunha canción: título, artista, álbum, portada… Gárdase unha copia; o orixinal non se toca. */
  export let file: FileEntry;

  const job = createJob();
  const busy = job.busy;

  const EMPTY: engine.MusicTags = { title: "", artist: "", album: "", album_artist: "", composer: "", genre: "", comment: "", lyrics: "", year: null, track: null, track_total: null, disc: null, disc_total: null };
  let tags: engine.MusicTags = { ...EMPTY };
  let original = JSON.stringify(EMPTY);
  let loaded = false;
  let loadErr = "";
  let format = "";

  // Portada: a do ficheiro, outra que se elixa, ou ningunha.
  let cover: engine.CoverAction = { kind: "keep" };
  let hadCover = false;
  let coverUrl = "";
  let coverPath = "";

  onMount(async () => {
    // Só en desenvolvemento: as probas da app real non poden manexar o diálogo nativo de ficheiros.
    if (import.meta.env.DEV) (window as any).__trasnoCover = setCover;
    try {
      const info = await engine.tagsRead(file.path);
      tags = { ...EMPTY, ...info.tags };
      original = JSON.stringify(tags);
      format = info.format;
      hadCover = !!info.cover;
      if (info.cover) coverUrl = await viewUrl(await engine.tagsCover(file.path));
      loaded = true;
    } catch (e) {
      loadErr = errText(e);
    }
  });
  onDestroy(() => {
    if (import.meta.env.DEV) delete (window as any).__trasnoCover;
  });

  async function chooseCover() {
    const picked = await open({ multiple: false, filters: [{ name: tr("Imaxes"), extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff", "gif"] }] });
    if (typeof picked === "string") await setCover(picked);
  }
  async function setCover(path: string) {
    coverPath = path;
    coverUrl = await viewUrl(path);
    cover = { kind: "replace", path };
  }
  function removeCover() {
    cover = { kind: "remove" };
    coverUrl = "";
    coverPath = "";
  }
  async function resetCover() {
    cover = { kind: "keep" };
    coverPath = "";
    coverUrl = hadCover ? await viewUrl(await engine.tagsCover(file.path)) : "";
  }

  /** «01 - Artista - Título», «Artista - Título» ou «03. Título»: enche só os campos que están baleiros. */
  function fromName() {
    let stem = file.name.replace(/\.[^.]+$/, "").replace(/_/g, " ").trim();
    let n: number | null = null;
    const m = /^(\d{1,3})\s*[-.)]\s*(.+)$/.exec(stem);
    if (m) {
      n = Number(m[1]);
      stem = m[2];
    }
    const parts = stem.split(/\s+-\s+/).map((x) => x.trim()).filter(Boolean);
    const [artist, title] = parts.length >= 2 ? [parts[0], parts.slice(1).join(" - ")] : ["", parts[0] ?? ""];
    tags = { ...tags, title: tags.title || title, artist: tags.artist || artist, track: tags.track ?? n };
  }

  $: changed = loaded && (JSON.stringify(tags) !== original || cover.kind !== "keep");
  function undo() {
    tags = JSON.parse(original);
    void resetCover();
  }

  const LABELS: Record<string, string> = {
    title: "Título",
    artist: "Artista",
    album: "Álbum",
    album_artist: "Artista do álbum",
    composer: "Compositor",
    genre: "Xénero",
    comment: "Comentario",
    lyrics: "Letra",
    year: "Ano",
    track: "Pista",
    track_total: "Total de pistas",
    disc: "Disco",
    disc_total: "Total de discos",
  };

  async function save() {
    try {
      const out = await askOutput(file, tr("-etiquetado"), file.ext);
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.tagsWrite(file.path, out, tags, cover, true);
        const lost = r.not_saved.map((k) => tr(LABELS[k] ?? k).toLowerCase());
        const warn = lost.length ? `\n${tr("Este formato non garda: {list}.", { list: lost.join(", ") })}` : "";
        return `${listo()} ${fileNameOf(r.output)}${warn}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  {#if loadErr}
    <p class="warn" role="alert">{loadErr}</p>
  {:else if !loaded}
    <p class="muted small">{$t("Lendo as etiquetas…")}</p>
  {:else}
    <div class="group">
      <h4 class="group-title">{$t("Etiquetas")}</h4>
      <GroupNote text={$t("O son non se recodifica: só cambian as etiquetas.")} />
      <TextField label={$t("Título")} bind:value={tags.title} />
      <TextField label={$t("Artista")} bind:value={tags.artist} />
      <TextField label={$t("Álbum")} bind:value={tags.album} />
      <TextField label={$t("Artista do álbum")} bind:value={tags.album_artist} />
      <div class="pair">
        <NumberField label={$t("Pista")} bind:value={tags.track} min={1} max={9999} placeholder="—" />
        <NumberField label={$t("Total de pistas")} bind:value={tags.track_total} min={1} max={9999} placeholder="—" />
      </div>
      <div class="pair">
        <NumberField label={$t("Disco")} bind:value={tags.disc} min={1} max={9999} placeholder="—" />
        <NumberField label={$t("Total de discos")} bind:value={tags.disc_total} min={1} max={9999} placeholder="—" />
      </div>
      <div class="pair">
        <NumberField label={$t("Ano")} bind:value={tags.year} min={1} max={9999} placeholder="—" />
        <TextField label={$t("Xénero")} bind:value={tags.genre} />
      </div>
      <TextField label={$t("Compositor")} bind:value={tags.composer} />
      <TextField label={$t("Comentario")} bind:value={tags.comment} />
      <label class="area">
        <span class="lab">{$t("Letra")}</span>
        <textarea bind:value={tags.lyrics} rows="5" aria-label={$t("Letra")}></textarea>
      </label>
      <button class="btn" on:click={fromName}>{$t("Sacar do nome do ficheiro")}</button>
    </div>

    <div class="group">
      <h4 class="group-title">{$t("Portada")}</h4>
      <GroupNote text={$t("JPEG e PNG métense tal cal; calquera outra imaxe páso a PNG.")} />
      <div class="cover">
        {#if coverUrl}
          <img src={coverUrl} alt={$t("Portada")} draggable="false" />
        {:else}
          <span class="none muted">{$t("Sen portada")}</span>
        {/if}
        <div class="cbtn">
          <button class="btn" on:click={chooseCover} disabled={$busy}>{coverUrl ? $t("Cambiar…") : $t("Escoller imaxe…")}</button>
          {#if coverUrl}<button class="btn" on:click={removeCover} disabled={$busy}>{$t("Quitar")}</button>{/if}
        </div>
      </div>
    </div>

    <div class="row">
      <button class="btn primary block" on:click={save} disabled={$busy || !changed}>{$t("Gardar etiquetas")}</button>
      <button class="btn" on:click={undo} disabled={$busy || !changed}>{$t("Desfacer")}</button>
    </div>
    <JobStatus {job} />
  {/if}
</div>

<style>
  .small { margin: 0; font-size: var(--fs-sm); }
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); }
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 0.6rem; align-items: end; }
  .row { display: flex; gap: 0.5rem; }
  .row .block { flex: 1; }
  .area { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { font-size: var(--fs-sm); color: var(--text-muted); }
  textarea { width: 100%; resize: vertical; min-height: 4.2rem; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0.4rem 0.6rem; font: inherit; font-size: var(--fs-md); }
  textarea:focus { outline: none; border-color: var(--accent); }
  .cover { display: flex; align-items: center; gap: 0.8rem; }
  .cover img { width: 96px; height: 96px; object-fit: cover; border-radius: var(--radius-sm); border: 1px solid var(--border); background: var(--bg-input); }
  .none { width: 96px; height: 96px; display: flex; align-items: center; justify-content: center; text-align: center; font-size: var(--fs-sm); border: 1px dashed var(--border-strong); border-radius: var(--radius-sm); }
  .cbtn { display: flex; flex-direction: column; gap: 0.4rem; align-items: flex-start; }
</style>
