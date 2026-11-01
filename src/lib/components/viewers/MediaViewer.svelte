<script lang="ts">
  import { onDestroy } from "svelte";
  import { viewUrl, mediaPreview, mediaSeek, mediaTime, trimRange, type FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { actionFor } from "../../keymap";
  import { peaksCache, previewCache } from "../../viewCache";
  import InfoTip from "../ui/InfoTip.svelte";
  import { errText } from "../../errors";
  import { tools, openSettings } from "../../settings";
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
  $: if (el && $mediaSeek) seek($mediaSeek.t);

  // Instante actual, para que as ferramentas (fotograma, corte) o poidan usar.
  let now = 0;
  let duration = 0;
  let playing = false;
  // Vídeos que o navegador non le: móstranse por tramos de SEG segundos recodificados ao momento. O tempo que ven
  // as ferramentas e a liña de tempo é o do ficheiro orixinal (tempo do tramo + inicio do tramo).
  const SEG = 60;
  let chunked = false;
  let segIndex = 0;
  let loadingSeg = false;
  let totalDur = 0;
  let afterLoad: { offset: number; play: boolean } | null = null;
  let segToken = 0;
  const prefetched = new Set<number>();
  $: segStart = segIndex * SEG;

  function tick() {
    if (!el) return;
    now = (chunked ? segStart : 0) + el.currentTime;
    duration = chunked ? totalDur : Number.isFinite(el.duration) && el.duration ? el.duration : (file.duration ?? 0);
    mediaTime.set(Math.round(now * 100) / 100);
    // O tramo seguinte vai preparándose mentres se ve este.
    if (chunked && el.currentTime > SEG * 0.4 && segStart + SEG < totalDur && !prefetched.has(segIndex + 1)) {
      const n = segIndex + 1;
      prefetched.add(n);
      engine.makePreviewChunk(file.path, n * SEG, SEG).catch(() => prefetched.delete(n));
    }
    // Con un corte activo, a reprodución salta a parte que se perde.
    const r = $trimRange;
    if (r && playing) {
      if (now < r.start - 0.05) seek(r.start);
      else if (now >= r.end) {
        el.pause();
        seek(r.start);
      }
    }
  }
  onDestroy(() => mediaTime.set(0));

  async function loadSeg(idx: number, offset: number, play: boolean) {
    const mine = ++segToken;
    loadingSeg = true;
    segIndex = idx;
    afterLoad = { offset, play };
    try {
      const path = await engine.makePreviewChunk(file.path, idx * SEG, SEG);
      if (mine !== segToken) return;
      url = await viewUrl(path);
    } catch (e) {
      if (mine !== segToken) return;
      loadingSeg = false;
      failed = true;
      failMsg = errText(e);
    }
  }
  /** Cando o tramo novo está listo: vai ao punto pedido e segue reproducindo se se estaba. */
  function onMeta() {
    if (afterLoad && el) {
      el.currentTime = afterLoad.offset;
      if (afterLoad.play) el.play().catch(() => {});
      afterLoad = null;
    }
    loadingSeg = false;
    if (el && $mediaPreview) el.playbackRate = Math.min(4, Math.max(0.25, $mediaPreview.speed));
    tick();
  }
  function onEnded() {
    if (chunked && segStart + SEG < totalDur) loadSeg(segIndex + 1, 0, true);
    else playing = false;
  }

  function seek(t: number) {
    if (!el) return;
    if (!chunked) {
      el.currentTime = t;
      return;
    }
    t = Math.max(0, Math.min(totalDur || t, t));
    const idx = Math.floor(t / SEG);
    if (idx === segIndex && !loadingSeg) el.currentTime = t - segStart;
    else loadSeg(idx, t - idx * SEG, !el.paused || playing);
  }
  function toggle() {
    if (!el) return;
    if (el.paused) {
      const r = $trimRange;
      if (r && (now < r.start || now >= r.end)) seek(r.start);
      el.play();
    } else el.pause();
  }
  // ---- teclado (só co visor aberto; non actúa se se está escribindo nun campo) ----
  function key(e: KeyboardEvent) {
    if (!el) return;
    if ((e.target as HTMLElement).closest("input, select, textarea, [contenteditable], [data-filelist]")) return;
    const id = actionFor(e, "media");
    if (!id) return;
    const jump = (d: number) => seek(Math.min(duration || Infinity, Math.max(0, now + d)));
    switch (id) {
      case "m.play": toggle(); break;
      case "m.back5": jump(-5); break;
      case "m.fwd5": jump(5); break;
      case "m.back1": jump(-1); break;
      case "m.fwd1": jump(1); break;
      case "m.back10": jump(-10); break;
      case "m.fwd10": jump(10); break;
      case "m.volUp":
        vol = Math.min(1, Math.round((vol + 0.05) * 100) / 100);
        muted = false;
        break;
      case "m.volDown": vol = Math.max(0, Math.round((vol - 0.05) * 100) / 100); break;
      case "m.mute": muted = !muted; break;
      case "m.fullscreen": if (file.category === "video") fullscreen(); break;
      case "m.start": seek(0); break;
      case "m.end": if (duration) seek(duration); break;
      case "m.framePrev":
      case "m.frameNext":
        // un fotograma cara atrás ou adiante (con pausa)
        if (file.category === "video") {
          el.pause();
          seek(Math.max(0, now + (id === "m.frameNext" ? 1 : -1) / 30));
        }
        break;
    }
    e.preventDefault();
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
    const cached = peaksCache.get(file.path);
    if (cached) {
      peaks = cached;
      return;
    }
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
      peaksCache.set(file.path, out);
    } catch {
      peaks = [];
    }
  }

  const tick_ = () => new Promise((r) => setTimeout(r, 120));
  const WHY_PREVIEW =
    "O visor da app é o dun navegador, que só le uns poucos formatos (sobre todo MP4 e WebM). Por iso o trasno prepara unha copia temporal só para velo, coa mesma resolución. O orixinal non se toca e ao exportar parte sempre del.";
  let preparing = false;
  let converted = false; // a vista previa non é o ficheiro orixinal (recodificada)
  let failMsg = "";
  let triedPreview = false;

  async function load(path: string) {
    failed = false;
    failMsg = "";
    converted = false;
    chunked = false;
    segIndex = 0;
    totalDur = 0;
    loadingSeg = false;
    afterLoad = null;
    segToken++;
    prefetched.clear();
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
    const known = previewCache.get(file.path);
    if (known) {
      converted = known.converted;
      url = await viewUrl(known.path);
      preparing = false;
      return;
    }
    try {
      const first = await engine.makePreview(file.path);
      if (first === "chunks") {
        // Vídeo que o navegador non le: tramos de 1 minuto, coa duración e o tempo do ficheiro orixinal.
        converted = true;
        chunked = true;
        totalDur = file.duration ?? Number((await engine.probeMedia(file.path)).format?.duration) ?? 0;
        if (!Number.isFinite(totalDur)) totalDur = 0;
        await loadSeg(0, 0, false);
      } else {
        converted = !first.includes(".remux.");
        url = await viewUrl(first);
        previewCache.set(mine, { path: first, converted });
      }
    } catch (e) {
      failed = true;
      failMsg = errText(e);
    } finally {
      preparing = false;
    }
  }
</script>

<svelte:window on:keydown={key} on:keyup={keyup} />

<div class="wrap">
  {#if preparing}
    <div class="msg loading" role="status">
      <p class="muted">{$t("O trasno está preparando a vista previa de {ext}…", { ext: file.ext.toUpperCase() })}<InfoTip text={$t(WHY_PREVIEW)} /></p>
      <div class="indet" aria-hidden="true"><span></span></div>
    </div>
  {:else if failed}
    <div class="msg">
      <p class="muted">
        {#if $tools && !$tools.ffmpeg}{$t("O trasno non pode preparar a vista previa porque falta ffmpeg.")}{:else}{$t("O trasno non puido preparar a vista previa.")}{/if}
      </p>
      {#if failMsg && !($tools && !$tools.ffmpeg)}<p class="muted tiny selectable">{failMsg}</p>{/if}
      {#if $tools && !$tools.ffmpeg}<button class="btn" on:click={() => openSettings("tools")}>{$t("Ver como instalalo")}</button>{/if}
    </div>
  {:else if url && file.category === "video"}
    <div class="vid" bind:this={stage}>
      <!-- svelte-ignore a11y_media_has_caption, a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
      <video bind:this={el} src={url} on:click={toggle} on:dblclick={fullscreen} on:error={prepare} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={onMeta} on:play={() => (playing = true)} on:pause={() => (playing = false)} on:ended={onEnded}></video>
      {#if loadingSeg}<div class="segload" role="status"><span class="indet" aria-hidden="true"><span></span></span>{$t("Preparando este tramo…")}</div>{/if}
      <div class="under">
        {#if converted}
          <p class="note" role="note">{$t("Vista previa recodificada só para velo aquí. O ficheiro orixinal non cambia.")}<InfoTip text={$t(WHY_PREVIEW)} /></p>
        {/if}
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
  .vid { position: relative; display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; }
  video { flex: 1; min-height: 0; width: 100%; object-fit: contain; cursor: pointer; background: transparent; }
  .segload { position: absolute; left: 50%; top: 40%; transform: translateX(-50%); display: flex; flex-direction: column; align-items: center; gap: 0.5rem; padding: 0.6rem 0.9rem; background: color-mix(in srgb, var(--bg-elevated) 92%, transparent); border: 1px solid var(--border-strong); border-radius: var(--radius-md); font-size: 0.82rem; pointer-events: none; }
  .loading { display: flex; flex-direction: column; align-items: center; gap: 0.7rem; }
  .loading p { margin: 0; }
  .tiny { font-size: 0.78rem; }
  .indet { width: min(280px, 70%); height: 5px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; position: relative; }
  .indet span { position: absolute; inset: 0 auto 0 0; width: 40%; border-radius: 99px; background: var(--accent); animation: vai 1.2s ease-in-out infinite alternate; }
  @keyframes vai { from { transform: translateX(0); } to { transform: translateX(150%); } }
  .note { display: flex; align-items: center; gap: 0.4rem; margin: 0 0 0.45rem; font-size: 0.76rem; color: var(--text-muted); }
  .under { padding: 0.7rem 1rem 0.6rem; background: var(--bg-rail); border-top: 1px solid var(--border); }
  .audio { display: flex; flex-direction: column; gap: 0.9rem; align-items: stretch; padding: 2rem; width: min(820px, 100%); }
  .nome { color: var(--text-muted); text-align: center; }
  .ctl { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 0.7rem; }
  .ctl.center { justify-content: center; }
  .tlw { flex: 1 1 9rem; min-width: 0; }
  .vol { width: 88px; accent-color: var(--accent); }
  .vid { background: var(--viewer-bg); }
  .vid:fullscreen, .vid::backdrop { background: var(--viewer-bg); }
  .vid:fullscreen .under { background: var(--bg-rail); }
  .time { font-variant-numeric: tabular-nums; font-size: 0.85rem; }
  .ph { text-align: center; font-size: 0.82rem; }
  .msg { padding: 2rem; text-align: center; }
</style>
