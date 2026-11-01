<script lang="ts">
  import { onDestroy } from "svelte";
  import { viewUrl, mediaPreview, mediaSeek, mediaTime, trimRange, type FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { t } from "../../i18n";
  import Timeline from "../ui/Timeline.svelte";
  import Icon from "../Icon.svelte";

  export let file: FileEntry;

  let url = "";
  let el: HTMLMediaElement | undefined;
  let failed = false;

  $: load(file.path);
  // Vista previa en directo: velocidade e volume (o navegador non pasa de volume 1) e salto a un instante.
  $: if (el && $mediaPreview) el.playbackRate = Math.min(4, Math.max(0.25, $mediaPreview.speed));
  // Volume do reprodutor propio (0 a 1), combinado coa vista previa do multiplicador.
  let vol = 1;
  let muted = false;
  $: if (el) {
    el.volume = Math.min(1, Math.max(0, ($mediaPreview ? $mediaPreview.volume : 1) * vol));
    el.muted = muted;
  }
  let stage: HTMLDivElement | undefined;
  function fullscreen() {
    if (document.fullscreenElement) document.exitFullscreen();
    else stage?.requestFullscreen?.();
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
  // ---- teclado (só co visor aberto; non actúa se se está escribindo nun campo) ----
  function key(e: KeyboardEvent) {
    if (!el || e.ctrlKey || e.metaKey || e.altKey) return;
    if ((e.target as HTMLElement).closest("input, select, textarea, [contenteditable], [data-filelist]")) return;
    const step = e.shiftKey ? 1 : 5;
    switch (e.key) {
      case " ":
      case "k":
        e.preventDefault();
        toggle();
        break;
      case "ArrowLeft":
      case "j":
        e.preventDefault();
        seek(Math.max(0, now - (e.key === "j" ? 10 : step)));
        break;
      case "ArrowRight":
      case "l":
        e.preventDefault();
        seek(Math.min(duration || Infinity, now + (e.key === "l" ? 10 : step)));
        break;
      case "ArrowUp":
        e.preventDefault();
        vol = Math.min(1, Math.round((vol + 0.05) * 100) / 100);
        muted = false;
        break;
      case "ArrowDown":
        e.preventDefault();
        vol = Math.max(0, Math.round((vol - 0.05) * 100) / 100);
        break;
      case "m":
        muted = !muted;
        break;
      case "f":
        if (file.category === "video") fullscreen();
        break;
      case "Home":
        e.preventDefault();
        seek(0);
        break;
      case "End":
        e.preventDefault();
        if (duration) seek(duration);
        break;
      case ",":
      case ".":
        // un fotograma cara atrás ou adiante (con pausa)
        if (file.category === "video") {
          el.pause();
          seek(Math.max(0, now + (e.key === "." ? 1 : -1) / 30));
        }
        break;
    }
  }
  // O espazo non debe «premer» tamén o botón que teña o foco.
  function keyup(e: KeyboardEvent) {
    if (e.key === " " && !(e.target as HTMLElement).closest("input, select, textarea, [contenteditable]")) e.preventDefault();
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

  const tick_ = () => new Promise((r) => setTimeout(r, 120));
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
    const mine = file.path;
    try {
      // Primeiro só o comezo (sae en segundos); o resto prepárase detrás e cámbiase sen perder o punto.
      const first = await engine.makePreview(file.path, file.category === "video" ? 30 : undefined);
      url = await viewUrl(first);
      if (file.category === "video") {
        engine
          .makePreview(file.path)
          .then(async (full) => {
            if (full === first || file.path !== mine) return;
            const at = el?.currentTime ?? 0;
            const was = playing;
            url = await viewUrl(full);
            await tick_();
            if (el) {
              el.currentTime = at;
              if (was) el.play().catch(() => {});
            }
          })
          .catch(() => {});
      }
    } catch {
      failed = true;
    } finally {
      preparing = false;
    }
  }
</script>

<svelte:window on:keydown={key} on:keyup={keyup} />

<div class="wrap">
  {#if preparing}
    <p class="muted msg">{$t("O trasno está preparando unha vista previa de {ext}…", { ext: file.ext.toUpperCase() })}</p>
  {:else if failed}
    <p class="muted msg">{$t("O trasno non puido preparar a vista previa; quizais falta ffmpeg.")}</p>
  {:else if url && file.category === "video"}
    <div class="vid" bind:this={stage}>
      <!-- svelte-ignore a11y_media_has_caption, a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
      <video bind:this={el} src={url} on:click={toggle} on:dblclick={fullscreen} on:error={prepare} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={tick} on:play={() => (playing = true)} on:pause={() => (playing = false)} on:ended={() => (playing = false)}></video>
      <div class="under">
        <div class="ctl">
          <button class="btn icon" on:click={toggle} aria-label={playing ? $t("Pausa") : $t("Reproducir")} title={`${playing ? $t("Pausa") : $t("Reproducir")} (${$t("Espazo")})`}><Icon name={playing ? "pause" : "play"} size={15} /></button>
          <div class="tlw"><Timeline {duration} {now} height={34} on:seek={(e) => seek(e.detail)} /></div>
          <span class="muted time">{fmt(now)} / {fmt(duration)}</span>
          <button class="btn icon ghost" on:click={() => (muted = !muted)} aria-label={$t("Silenciar")} title={`${$t("Silenciar")} (M)`}><Icon name={muted || vol === 0 ? "mute" : "volume"} size={15} /></button>
          <input class="vol" type="range" min="0" max="1" step="0.01" bind:value={vol} on:input={() => (muted = false)} aria-label={$t("Volume")} title={`${$t("Volume")}: ${Math.round(vol * 100)} %`} />
          <button class="btn icon ghost" on:click={fullscreen} aria-label={$t("Pantalla completa")} title={`${$t("Pantalla completa")} (F)`}><Icon name="expand" size={15} /></button>
        </div>
      </div>
    </div>
  {:else if url}
    <div class="audio">
      <div class="nome">{file.name}</div>
      <Timeline {duration} {now} {peaks} height={150} on:seek={(e) => seek(e.detail)} />
      {#if !peaks.length}<span class="muted ph">{$t("O trasno está lendo as ondas do son…")}</span>{/if}
      <div class="ctl center">
        <button class="btn icon" on:click={toggle} aria-label={playing ? $t("Pausa") : $t("Reproducir")} title={`${playing ? $t("Pausa") : $t("Reproducir")} (${$t("Espazo")})`}><Icon name={playing ? "pause" : "play"} size={15} /></button>
        <span class="muted time">{fmt(now)} / {fmt(duration)}</span>
        <button class="btn icon ghost" on:click={() => (muted = !muted)} aria-label={$t("Silenciar")} title={`${$t("Silenciar")} (M)`}><Icon name={muted || vol === 0 ? "mute" : "volume"} size={15} /></button>
        <input class="vol" type="range" min="0" max="1" step="0.01" bind:value={vol} on:input={() => (muted = false)} aria-label={$t("Volume")} title={`${$t("Volume")}: ${Math.round(vol * 100)} %`} />
      </div>
      <audio bind:this={el} src={url} on:error={prepare} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={tick} on:play={() => (playing = true)} on:pause={() => (playing = false)} on:ended={() => (playing = false)}></audio>
    </div>
  {/if}
</div>

<style>
  .wrap { height: 100%; display: flex; align-items: center; justify-content: center; background: var(--viewer-bg); }
  .vid { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
  video { flex: 1; min-height: 0; width: 100%; object-fit: contain; cursor: pointer; background: transparent; }
  .under { padding: 0.7rem 1rem 0.6rem; background: var(--bg-rail); border-top: 1px solid var(--border); }
  .audio { display: flex; flex-direction: column; gap: 0.9rem; align-items: stretch; padding: 2rem; width: min(820px, 100%); }
  .nome { color: var(--text-muted); text-align: center; }
  .ctl { display: flex; align-items: center; gap: 0.7rem; }
  .ctl.center { justify-content: center; }
  .tlw { flex: 1; min-width: 0; }
  .vol { width: 88px; accent-color: var(--accent); }
  .vid { background: var(--viewer-bg); }
  .vid:fullscreen, .vid::backdrop { background: var(--viewer-bg); }
  .vid:fullscreen .under { background: var(--bg-rail); }
  .time { font-variant-numeric: tabular-nums; font-size: 0.85rem; }
  .ph { text-align: center; font-size: 0.82rem; }
  .msg { padding: 2rem; text-align: center; }
</style>
