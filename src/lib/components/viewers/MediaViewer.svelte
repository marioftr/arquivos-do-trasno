<script lang="ts">
  import { viewUrl, type FileEntry } from "../../files";
  import * as engine from "../../engine";

  export let file: FileEntry;

  let url = "";
  let failed = false;

  $: load(file.path);

  let preparing = false;
  let triedPreview = false;

  async function load(path: string) {
    failed = false;
    triedPreview = false;
    preparing = false;
    url = await viewUrl(path);
  }

  // Formatos que o navegador non reproduce (MKV, AVI, AIFF...): o trasno prepara
  // unha copia lixeira temporal. O ficheiro orixinal non se modifica.
  async function prepare() {
    if (triedPreview) {
      failed = true;
      return;
    }
    triedPreview = true;
    preparing = true;
    try {
      url = await viewUrl(await engine.makePreview(file.path));
    } catch {
      failed = true;
    } finally {
      preparing = false;
    }
  }
</script>

<div class="wrap">
  {#if preparing}
    <p class="muted msg">O trasno está preparando unha vista previa de {file.ext.toUpperCase()}…</p>
  {:else if failed}
    <p class="muted msg">O trasno non puido preparar a vista previa (¿falta ffmpeg?).</p>
  {:else if url && file.category === "video"}
    <!-- svelte-ignore a11y_media_has_caption -->
    <video src={url} controls on:error={prepare}></video>
  {:else if url}
    <div class="audio">
      <div class="nome">{file.name}</div>
      <audio src={url} controls on:error={prepare}></audio>
    </div>
  {/if}
</div>

<style>
  .wrap { height: 100%; display: flex; align-items: center; justify-content: center; background: #0b0b0d; }
  video { max-width: 100%; max-height: 100%; }
  .audio { display: flex; flex-direction: column; gap: 1rem; align-items: center; padding: 2rem; width: min(560px, 100%); }
  audio { width: 100%; }
  .nome { color: var(--text-muted); }
  .msg { padding: 2rem; text-align: center; }
</style>
