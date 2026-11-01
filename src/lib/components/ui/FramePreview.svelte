<script lang="ts">
  import { onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { viewUrl } from "../../files";
  import { t } from "../../i18n";

  /** Miniatura do fotograma que se vai exportar a un instante dado. */
  export let path: string;
  export let time: number;

  let video: HTMLVideoElement | undefined;
  let canvas: HTMLCanvasElement | undefined;
  let url = "";
  let failed = false;
  let triedPreview = false;
  let timer: ReturnType<typeof setTimeout>;

  $: init(path);
  async function init(p: string) {
    failed = false;
    triedPreview = false;
    url = await viewUrl(p);
  }
  async function onError() {
    if (triedPreview) {
      failed = true;
      return;
    }
    triedPreview = true;
    try {
      url = await viewUrl(await engine.makePreview(path));
    } catch {
      failed = true;
    }
  }

  // Varios cambios seguidos (escribir un número) só buscan o último.
  $: if (video && url) {
    clearTimeout(timer);
    const t0 = Math.max(0, time || 0);
    timer = setTimeout(() => {
      if (!video) return;
      const d = Number.isFinite(video.duration) ? video.duration : t0;
      video.currentTime = Math.min(t0, Math.max(0, d - 0.05));
    }, 120);
  }
  onDestroy(() => clearTimeout(timer));

  function draw() {
    if (!video || !canvas || !video.videoWidth) return;
    const w = 320;
    canvas.width = w;
    canvas.height = Math.round((video.videoHeight / video.videoWidth) * w);
    canvas.getContext("2d")!.drawImage(video, 0, 0, canvas.width, canvas.height);
  }
</script>

<div class="fp">
  <!-- svelte-ignore a11y_media_has_caption -->
  <video bind:this={video} src={url} muted preload="auto" on:loadeddata={() => (video!.currentTime = Math.max(0, time || 0))} on:seeked={draw} on:error={onError}></video>
  {#if failed}
    <p class="muted">{$t("Non hai vista previa deste fotograma.")}</p>
  {:else}
    <canvas bind:this={canvas}></canvas>
    <span class="tag">{(time || 0).toFixed(2)} s</span>
  {/if}
</div>

<style>
  .fp { position: relative; width: 100%; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); overflow: hidden; display: flex; justify-content: center; min-height: 60px; }
  video { display: none; }
  canvas { width: 100%; height: auto; display: block; }
  .tag { position: absolute; right: 6px; bottom: 6px; font-size: 0.72rem; background: rgba(0, 0, 0, 0.75); color: #fff; padding: 1px 6px; border-radius: 4px; }
  p { margin: 0.8rem; font-size: 0.82rem; }
</style>
