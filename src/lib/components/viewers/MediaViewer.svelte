<script lang="ts">
  import { onDestroy } from "svelte";
  import { viewUrl, mediaPreview, mediaSeek, mediaTime, type FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { t } from "../../i18n";

  export let file: FileEntry;

  let url = "";
  let el: HTMLMediaElement | undefined;
  let failed = false;

  $: load(file.path);
  // Vista previa en directo: velocidade e volume (o navegador non pasa de volume 1) e salto a un instante.
  $: if (el && $mediaPreview) {
    el.playbackRate = Math.min(4, Math.max(0.25, $mediaPreview.speed));
    el.volume = Math.min(1, Math.max(0, $mediaPreview.volume));
  }
  $: if (el && $mediaSeek) el.currentTime = $mediaSeek.t;

  // Instante actual, para que as ferramentas (fotograma, corte) o poidan usar.
  let now = 0;
  let duration = 0;
  function tick() {
    if (!el) return;
    now = el.currentTime;
    duration = el.duration || 0;
    mediaTime.set(Math.round(now * 100) / 100);
  }
  onDestroy(() => mediaTime.set(0));

  // ---- onda de son (só audio): picos por columna, calculados no navegador ----
  let canvas: HTMLCanvasElement | undefined;
  let peaks: number[] = [];
  let waveFor = "";
  $: if (file.category === "audio" && url && url !== waveFor) analyse(url);
  async function analyse(u: string) {
    waveFor = u;
    peaks = [];
    try {
      const buf = await (await fetch(u)).arrayBuffer();
      if (buf.byteLength > 200_000_000) return;
      const ctx = new AudioContext();
      const audio = await ctx.decodeAudioData(buf);
      await ctx.close();
      const COLS = 900;
      const ch = audio.getChannelData(0);
      const step = Math.max(1, Math.floor(ch.length / COLS));
      const out: number[] = [];
      for (let i = 0; i < COLS; i++) {
        let max = 0;
        for (let j = i * step; j < Math.min(ch.length, (i + 1) * step); j += Math.max(1, step >> 6)) max = Math.max(max, Math.abs(ch[j]));
        out.push(max);
      }
      peaks = out;
    } catch {
      peaks = [];
    }
  }
  $: if (canvas) draw(peaks, now, duration);
  function draw(p: number[], at: number, dur: number) {
    if (!canvas) return;
    const w = (canvas.width = canvas.clientWidth * devicePixelRatio);
    const h = (canvas.height = canvas.clientHeight * devicePixelRatio);
    const g = canvas.getContext("2d")!;
    g.clearRect(0, 0, w, h);
    if (!p.length) return;
    const accent = getComputedStyle(canvas).getPropertyValue("--accent").trim() || "#7aa2f7";
    const played = dur ? at / dur : 0;
    const bw = w / p.length;
    for (let i = 0; i < p.length; i++) {
      const bh = Math.max(1, p[i] * h * 0.95);
      g.fillStyle = i / p.length <= played ? accent : "rgba(160,160,175,0.55)";
      g.fillRect(i * bw, (h - bh) / 2, Math.max(1, bw - 1), bh);
    }
  }
  function seekWave(ev: MouseEvent) {
    if (!el || !duration) return;
    const r = (ev.currentTarget as HTMLElement).getBoundingClientRect();
    el.currentTime = ((ev.clientX - r.left) / r.width) * duration;
  }

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
    <p class="muted msg">{$t("O trasno está preparando unha vista previa de {ext}…", { ext: file.ext.toUpperCase() })}</p>
  {:else if failed}
    <p class="muted msg">{$t("O trasno non puido preparar a vista previa; quizais falta ffmpeg.")}</p>
  {:else if url && file.category === "video"}
    <!-- svelte-ignore a11y_media_has_caption -->
    <video bind:this={el} src={url} controls on:error={prepare} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={tick}></video>
  {:else if url}
    <div class="audio">
      <div class="nome">{file.name}</div>
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="wave" on:click={seekWave}>
        <canvas bind:this={canvas}></canvas>
        {#if !peaks.length}<span class="muted ph">{$t("O trasno está lendo as ondas do son…")}</span>{/if}
      </div>
      <audio bind:this={el} src={url} controls on:error={prepare} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={tick}></audio>
    </div>
  {/if}
</div>

<style>
  .wrap { height: 100%; display: flex; align-items: center; justify-content: center; background: #0b0b0d; }
  video { max-width: 100%; max-height: 100%; }
  .audio { display: flex; flex-direction: column; gap: 1rem; align-items: center; padding: 2rem; width: min(760px, 100%); }
  audio { width: 100%; }
  .wave { position: relative; width: 100%; height: 140px; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); cursor: pointer; }
  .wave canvas { width: 100%; height: 100%; display: block; }
  .ph { position: absolute; inset: 0; display: grid; place-items: center; font-size: 0.85rem; }
  .nome { color: var(--text-muted); }
  .msg { padding: 2rem; text-align: center; }
</style>
