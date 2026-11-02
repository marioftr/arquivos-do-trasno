<script lang="ts">
  import { onDestroy } from "svelte";
  import * as engine from "../../engine";
  import { viewUrl } from "../../files";
  import { t } from "../../i18n";

  /** Miniatura do fotograma que se vai exportar a un instante dado: sácaa ffmpeg, así que vale para calquera formato. */
  export let path: string;
  export let time: number;

  let url = "";
  let failed = false;
  let loading = false;
  let timer: ReturnType<typeof setTimeout>;
  let token = 0;

  // Varios cambios seguidos (escribir un número, mover o vídeo) só buscan o último.
  $: schedule(path, time);
  function schedule(p: string, t0: number) {
    clearTimeout(timer);
    timer = setTimeout(() => load(p, Math.max(0, Math.round((t0 || 0) * 20) / 20)), 200);
  }
  async function load(p: string, t0: number) {
    const mine = ++token;
    loading = true;
    try {
      const out = await viewUrl(await engine.makeFramePreview(p, t0));
      if (mine !== token) return;
      url = out;
      failed = false;
    } catch {
      if (mine === token) failed = true;
    } finally {
      if (mine === token) loading = false;
    }
  }
  onDestroy(() => {
    clearTimeout(timer);
    token++;
  });
</script>

<div class="fp" class:loading>
  {#if failed && !url}
    <p class="muted">{$t("Non hai vista previa deste fotograma.")}</p>
  {:else if url}
    <img src={url} alt="" draggable="false" />
    <span class="tag">{(time || 0).toFixed(2)} s</span>
  {:else}
    <p class="muted">{$t("Preparando o fotograma…")}</p>
  {/if}
</div>

<style>
  .fp { position: relative; width: 100%; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); overflow: hidden; display: flex; justify-content: center; min-height: 60px; transition: opacity 0.12s ease; }
  .fp.loading { opacity: 0.7; }
  img { width: 100%; height: auto; display: block; }
  .tag { position: absolute; right: 6px; bottom: 6px; font-size: 0.72rem; background: rgba(0, 0, 0, 0.75); color: #fff; padding: 1px 6px; border-radius: 4px; }
  p { margin: 0.8rem; font-size: 0.82rem; }
</style>
