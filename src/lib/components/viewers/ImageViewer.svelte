<script lang="ts">
  import { viewUrl, imageEdit, type FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { t } from "../../i18n";

  export let file: FileEntry;

  let url = "";
  let failed = false;
  let zoom = 100;
  let fit = true;

  $: load(file.path);
  $: e = $imageEdit;
  $: preview = e
    ? `transform: rotate(${e.rotate}deg) scale(${e.flipH ? -1 : 1}, ${e.flipV ? -1 : 1}); filter: brightness(${1 + e.brightness / 100}) contrast(${1 + e.contrast / 100}) grayscale(${e.grayscale ? 1 : 0}) invert(${e.invert ? 1 : 0}) blur(${e.blur}px);`
    : "";

  let triedPreview = false;

  async function load(path: string) {
    failed = false;
    triedPreview = false;
    fit = true;
    zoom = 100;
    url = await viewUrl(path);
  }

  // O navegador non le TIFF, TGA, QOI, PPM...: o trasno fai un PNG temporal para velo.
  async function onError() {
    if (triedPreview) {
      failed = true;
      return;
    }
    triedPreview = true;
    try {
      url = await viewUrl(await engine.makePreview(file.path));
    } catch {
      failed = true;
    }
  }
</script>

<div class="wrap">
  <div class="stage scrollbar-thin" class:fit>
    {#if failed}
      <p class="muted msg">{$t("O trasno non puido preparar unha vista previa de {ext}.", { ext: file.ext.toUpperCase() })}</p>
    {:else if url}
      <img
        src={url}
        alt={file.name}
        style={(fit ? "" : `width:${((file.width ?? 0) * zoom) / 100}px;`) + preview}
        on:error={onError}
      />
    {/if}
  </div>
  <div class="bar">
    <button class="btn" class:on={fit} on:click={() => (fit = true)}>{$t("Axustar")}</button>
    <button class="btn" class:on={!fit && zoom === 100} on:click={() => { fit = false; zoom = 100; }}>100%</button>
    <input type="range" min="10" max="400" bind:value={zoom} on:input={() => (fit = false)} />
    <input class="num-input" type="number" min="10" max="400" bind:value={zoom} on:input={() => (fit = false)} />
    <span class="muted">%</span>
    {#if file.width}<span class="muted dims">{file.width} × {file.height} px</span>{/if}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .stage {
    flex: 1; min-height: 0; overflow: auto;
    display: flex; align-items: center; justify-content: center;
    background: repeating-conic-gradient(#17171a 0% 25%, #1b1b1f 0% 50%) 0 0 / 20px 20px;
  }
  .stage.fit img { max-width: 100%; max-height: 100%; object-fit: contain; }
  .stage:not(.fit) { display: block; }
  img { display: block; margin: auto; }
  .bar { display: flex; align-items: center; gap: 0.6rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .bar input[type="range"] { width: 160px; }
  .btn.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .dims { margin-left: auto; }
  .msg { margin: auto; text-align: center; padding: 2rem; }
</style>
