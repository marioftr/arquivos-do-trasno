<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { viewUrl, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import TextField from "../ui/TextField.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Formulario das etiquetas. Parte das do ficheiro (que son os valores por defecto) e lembra o que se cambiou ao saír da pestana. */
  export let file: FileEntry;
  export let info: engine.TagInfo;
  /** Portada do ficheiro (URL), se ten. */
  export let originalCover = "";

  const job = createJob();
  const busy = job.busy;

  const form = createForm(
    { ...info.tags, cover: "keep" as "keep" | "remove" | "replace", coverPath: "" },
    undefined,
    "etiquetas|" + file.path,
  );
  onMount(() => {
    // Só en desenvolvemento: as probas da app real non poden manexar o diálogo nativo de ficheiros.
    if (import.meta.env.DEV) (window as any).__trasnoCover = setCover;
    return useForm(form);
  });
  onDestroy(() => {
    if (import.meta.env.DEV) delete (window as any).__trasnoCover;
  });
  const dirty = form.dirty;

  let replaceUrl = "";
  $: if ($form.cover === "replace" && $form.coverPath) void viewUrl($form.coverPath).then((u) => (replaceUrl = u));
  $: coverUrl = $form.cover === "keep" ? originalCover : $form.cover === "replace" ? replaceUrl : "";

  async function chooseCover() {
    const picked = await open({ multiple: false, filters: [{ name: tr("Imaxes"), extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff", "gif"] }] });
    if (typeof picked === "string") setCover(picked);
  }
  function setCover(path: string) {
    form.update((f) => ({ ...f, cover: "replace", coverPath: path }));
  }
  function removeCover() {
    form.update((f) => ({ ...f, cover: "remove", coverPath: "" }));
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
    form.update((f) => ({ ...f, title: f.title || title, artist: f.artist || artist, track: f.track ?? n }));
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
      const { cover, coverPath, ...tags } = $form;
      const action: engine.CoverAction = cover === "replace" ? { kind: "replace", path: coverPath } : cover === "remove" ? { kind: "remove" } : { kind: "keep" };
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.tagsWrite(file.path, out, tags, action, true);
        const lost = r.not_saved.map((k) => tr(LABELS[k] ?? k).toLowerCase());
        const warn = lost.length ? `
${tr("Este formato non garda: {list}.", { list: lost.join(", ") })}` : "";
        return `${listo()} ${fileNameOf(r.output)}${warn}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}
${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Etiquetas")}</h4>
    <TextField label={$t("Título")} bind:value={$form.title} />
    <TextField label={$t("Artista")} bind:value={$form.artist} />
    <TextField label={$t("Álbum")} bind:value={$form.album} />
    <TextField label={$t("Artista do álbum")} bind:value={$form.album_artist} />
    <div class="pair">
      <NumberField label={$t("Pista")} bind:value={$form.track} min={1} max={9999} />
      <NumberField label={$t("Total de pistas")} bind:value={$form.track_total} min={1} max={9999} />
    </div>
    <div class="pair">
      <NumberField label={$t("Disco")} bind:value={$form.disc} min={1} max={9999} />
      <NumberField label={$t("Total de discos")} bind:value={$form.disc_total} min={1} max={9999} />
    </div>
    <div class="pair">
      <NumberField label={$t("Ano")} bind:value={$form.year} min={1} max={9999} />
      <TextField label={$t("Xénero")} bind:value={$form.genre} />
    </div>
    <TextField label={$t("Compositor")} bind:value={$form.composer} />
    <TextField label={$t("Comentario")} bind:value={$form.comment} />
    <label class="area">
      <span class="lab">{$t("Letra")}</span>
      <textarea bind:value={$form.lyrics} rows="5" aria-label={$t("Letra")}></textarea>
    </label>
    <button class="btn" on:click={fromName}>{$t("Sacar do nome do ficheiro")}</button>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Portada")}</h4>
    <GroupNote text={$t("Acepto JPEG e PNG. Se escolles outro formato, como WebP ou BMP, convértoo a PNG.")} />
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

  <button class="btn primary block" on:click={save} disabled={$busy || !$dirty}>{$t("Gardar etiquetas")}</button>
  <JobStatus {job} />
</div>

<style>
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 0.6rem; align-items: end; }
  .area { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { font-size: var(--fs-sm); color: var(--text-muted); }
  textarea { width: 100%; resize: vertical; min-height: 4.2rem; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0.4rem 0.6rem; font: inherit; font-size: var(--fs-md); }
  textarea:focus { outline: none; border-color: var(--accent); }
  .cover { display: flex; align-items: center; gap: 0.8rem; }
  .cover img { width: 96px; height: 96px; object-fit: cover; border-radius: var(--radius-sm); border: 1px solid var(--border); background: var(--bg-input); }
  .none { width: 96px; height: 96px; display: flex; align-items: center; justify-content: center; text-align: center; font-size: var(--fs-sm); border: 1px dashed var(--border-strong); border-radius: var(--radius-sm); }
  .cbtn { display: flex; flex-direction: column; gap: 0.4rem; align-items: flex-start; }
</style>
