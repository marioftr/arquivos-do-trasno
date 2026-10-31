<script lang="ts">
  import { viewUrl, type FileEntry } from "../../files";

  export let file: FileEntry;

  let url = "";
  let failed = false;

  $: load(file.path);

  async function load(path: string) {
    failed = false;
    url = await viewUrl(path);
  }
</script>

<div class="wrap">
  {#if failed}
    <p class="muted msg">O visor non reproduce o formato {file.ext.toUpperCase()}. Convértese a MP4 ou MP3 e xa se ve.</p>
  {:else if url && file.category === "video"}
    <!-- svelte-ignore a11y_media_has_caption -->
    <video src={url} controls on:error={() => (failed = true)}></video>
  {:else if url}
    <div class="audio">
      <div class="nome">{file.name}</div>
      <audio src={url} controls on:error={() => (failed = true)}></audio>
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
