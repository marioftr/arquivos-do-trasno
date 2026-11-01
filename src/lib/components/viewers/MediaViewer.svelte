<script lang="ts">
  import { onDestroy } from "svelte";
  import { viewUrl, mediaPreview, mediaSeek, mediaTime, trimRange, type FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { t } from "../../i18n";
  import Timeline from "../ui/Timeline.svelte";

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
  let playing = false;
  function tick() {
    if (!el) return;
    now = el.currentTime;
    duration = Number.isFinite(el.duration) && el.duration ? el.duration : (file.duration ?? 0);
    mediaTime.set(Math.round(now * 100) / 100);
    // Con un corte activo, a reprodución salta a parte que se perde.
    const r = $trimRange;
    if (r && playing) {
      if (now < r.start - 0.05) el.currentTime = r.start;
      else if (now >= r.end) {
        el.pause();
        el.currentTime = r.start;
      }
    }
  }
  onDestroy(() => mediaTime.set(0));

  function seek(t: number) {
    if (el) el.currentTime = t;
  }
  function toggle() {
    if (!el) return;
    if (el.paused) {
      const r = $trimRange;
      if (r && (el.currentTime < r.start || el.currentTime >= r.end)) el.currentTime = r.start;
      el.play();
    } else el.pause();
  }
  const fmt = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

  // ---- onda de son (só audio): picos por columna, calculados no navegador ----
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

  let preparing = false;
  let triedPreview = false;

  async function load(path: string) {
    failed = false;
    triedPreview = false;
    preparing = false;
    playing = false;
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
    <div class="vid">
      <!-- svelte-ignore a11y_media_has_caption -->
      <video bind:this={el} src={url} controls on:error={prepare} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={tick} on:play={() => (playing = true)} on:pause={() => (playing = false)}></video>
      {#if $trimRange}
        <div class="under"><Timeline {duration} {now} height={34} on:seek={(e) => seek(e.detail)} /></div>
      {/if}
    </div>
  {:else if url}
    <div class="audio">
      <div class="nome">{file.name}</div>
      <Timeline {duration} {now} {peaks} height={150} on:seek={(e) => seek(e.detail)} />
      {#if !peaks.length}<span class="muted ph">{$t("O trasno está lendo as ondas do son…")}</span>{/if}
      <div class="ctl">
        <button class="btn" on:click={toggle}>{playing ? $t("Pausa") : $t("Reproducir")}</button>
        <span class="muted time">{fmt(now)} / {fmt(duration)}</span>
      </div>
      <audio bind:this={el} src={url} on:error={prepare} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={tick} on:play={() => (playing = true)} on:pause={() => (playing = false)} on:ended={() => (playing = false)}></audio>
    </div>
  {/if}
</div>

<style>
  .wrap { height: 100%; display: flex; align-items: center; justify-content: center; background: #0b0b0d; }
  .vid { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
  video { flex: 1; min-height: 0; width: 100%; object-fit: contain; }
  .under { padding: 0.9rem 1rem 0.7rem; background: var(--bg-rail); border-top: 1px solid var(--border); }
  .audio { display: flex; flex-direction: column; gap: 0.9rem; align-items: stretch; padding: 2rem; width: min(820px, 100%); }
  .nome { color: var(--text-muted); text-align: center; }
  .ctl { display: flex; align-items: center; gap: 0.8rem; justify-content: center; }
  .time { font-variant-numeric: tabular-nums; font-size: 0.85rem; }
  .ph { text-align: center; font-size: 0.82rem; }
  .msg { padding: 2rem; text-align: center; }
</style>
