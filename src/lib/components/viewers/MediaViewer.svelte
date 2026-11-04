<script lang="ts">
  import { onDestroy } from "svelte";
  import { viewUrl, mediaPreview, mediaSeek, mediaTime, trimRange, type FileEntry, videoDar } from "../../files";
  import * as engine from "../../engine";
  import { get } from "svelte/store";
  import { player } from "../../player";
  import { actionFor } from "../../keymap";
  import { peaksCache, previewCache, playPos } from "../../viewCache";
  import InfoTip from "../ui/InfoTip.svelte";
  import { errText } from "../../errors";
  import { tools, openSettings } from "../../settings";
  import { requestTool } from "../../toolinstall";
  import { t } from "../../i18n";
  import Timeline from "../ui/Timeline.svelte";
  import Icon from "../Icon.svelte";
  import SidePanel from "../ui/SidePanel.svelte";
  import PanelSection from "../ui/PanelSection.svelte";
  import OptionList from "../ui/OptionList.svelte";
  import { toast } from "../../toasts";
  import { tr } from "../../i18n";

  export let file: FileEntry;

  let url = "";
  let el: HTMLMediaElement | undefined;
  let failed = false;

  // Só cando cambia a ruta, non cada vez que chegan datos novos do ficheiro (tamaño, duración…): senón, a vista previa
  // que se está preparando reiniciábase ao chegar eses datos.
  $: filePath = file.path;
  $: load(filePath);
  // Vista previa en directo: velocidade e volume (o navegador non pasa de volume 1) e salto a un instante.
  $: if (el && $mediaPreview) el.playbackRate = Math.min(4, Math.max(0.25, $mediaPreview.speed));
  // Volume do reprodutor propio (0 a 1), combinado coa vista previa do multiplicador.
  let vol = get(player).vol;
  let muted = get(player).muted;
  $: player.set({ vol, muted });
  $: if (el) {
    el.volume = Math.min(1, Math.max(0, ($mediaPreview ? $mediaPreview.volume : 1) * vol));
    el.muted = muted;
  }
  let stage: HTMLDivElement | undefined;
  // Relación de aspecto elixida na conversión: o visor amósaa antes de converter (só se mostra, o ficheiro non cambia).
  let sw = 0;
  let sh = 0;
  function size(node: HTMLElement) {
    const ro = new ResizeObserver(() => ((sw = node.clientWidth), (sh = node.clientHeight)));
    ro.observe(node);
    return { destroy: () => ro.disconnect() };
  }
  $: darNum = (() => {
    const [a, b] = ($videoDar || "").split("/").map(Number);
    return a && b ? a / b : 0;
  })();
  $: vstyle = darNum && sw && sh ? `flex:none;margin:auto;object-fit:fill;width:${Math.min(sw, sh * darNum)}px;height:${Math.min(sw, sh * darNum) / darNum}px;` : "";
  function fullscreen() {
    if (document.fullscreenElement) document.exitFullscreen();
    else stage?.requestFullscreen?.();
  }
  // Pantalla completa: o vídeo ocupa toda a pantalla e os controis agóchanse se non se move o rato.
  let fs = false;
  let barH = 0;
  let idle = false;
  let idleTimer: ReturnType<typeof setTimeout>;
  function wake() {
    idle = false;
    clearTimeout(idleTimer);
    if (fs && playing && !panelOpen) idleTimer = setTimeout(() => (idle = true), 2000);
  }
  function fsChange() {
    fs = !!stage && document.fullscreenElement === stage;
    wake();
  }
  $: if (fs) (playing, panelOpen, wake());
  onDestroy(() => clearTimeout(idleTimer));
  // Posición lembrada: ao volver a un ficheiro retómase onde se deixou.
  let ready = false;
  let restoredFor = "";
  $: if (el && $mediaSeek) seek($mediaSeek.t);

  // Instante actual, para que as ferramentas (fotograma, corte) o poidan usar.
  let now = 0;
  let duration = 0;
  let playing = false;
  // Vídeos que o navegador non le: móstranse por tramos de SEG segundos recodificados ao momento. O tempo que ven
  // as ferramentas e a liña de tempo é o do ficheiro orixinal (tempo do tramo + inicio do tramo).
  // ---- pistas (vídeo, audio, subtítulos) do ficheiro e a elixida de cada tipo
  interface Stream {
    kind: "video" | "audio" | "subtitle";
    i: number; // posición dentro do seu tipo
    codec: string;
    lang: string;
    title: string;
    isDefault: boolean;
    w?: number;
    h?: number;
    ch?: number;
    text: boolean; // subtítulo de texto (os de imaxe non se poden amosar)
  }
  const TEXT_SUBS = ["subrip", "srt", "ass", "ssa", "webvtt", "mov_text", "text", "subviewer", "microdvd"];
  let streams: Stream[] = [];
  let selV: number | null = null; // null = a de por defecto
  let selA: number | null = null;
  let selS = -1; // -1 = sen subtítulos
  let panelOpen = false;
  $: videos = streams.filter((x) => x.kind === "video");
  $: audios = streams.filter((x) => x.kind === "audio");
  $: subs = streams.filter((x) => x.kind === "subtitle");
  $: hasTracks = file.category === "video" && (videos.length > 1 || audios.length > 1 || subs.length > 0);
  const defIndex = (l: Stream[]) => Math.max(0, l.findIndex((x) => x.isDefault));

  function langName(code: string): string {
    if (!code || code === "und") return tr("Sen idioma");
    try {
      return new Intl.DisplayNames([document.documentElement.lang || "gl"], { type: "language" }).of(code) ?? code;
    } catch {
      return code.toUpperCase();
    }
  }
  async function readStreams(path: string) {
    try {
      const info = await engine.probeMedia(path);
      if (file.path !== path) return;
      const count: Record<string, number> = { video: 0, audio: 0, subtitle: 0 };
      const out: Stream[] = [];
      for (const st of info.streams ?? []) {
        const kind = st.codec_type as Stream["kind"];
        if (!(kind in count)) continue;
        if (kind === "video" && (st as any).disposition?.attached_pic) continue; // carátula
        out.push({
          kind,
          i: count[kind]++,
          codec: String(st.codec_name ?? ""),
          lang: String((st as any).tags?.language ?? ""),
          title: String((st as any).tags?.title ?? ""),
          isDefault: !!(st as any).disposition?.default,
          w: st.width,
          h: st.height,
          ch: (st as any).channels,
          text: kind !== "subtitle" || TEXT_SUBS.includes(String(st.codec_name)),
        });
      }
      streams = out;
    } catch {
      streams = [];
    }
  }
  const describe = (x: Stream) => {
    const bits = [x.codec.toUpperCase()];
    if (x.kind === "video" && x.w) bits.push(`${x.w}×${x.h}`);
    if (x.kind === "audio" && x.ch) bits.push(x.ch === 1 ? tr("mono") : x.ch === 2 ? tr("estéreo") : `${x.ch} ${tr("canles")}`);
    if (x.isDefault) bits.push(tr("por defecto"));
    if (x.kind === "subtitle" && !x.text) bits.push(tr("de imaxe: non se pode amosar"));
    return bits.join(" · ");
  };
  const nameOf = (x: Stream, n: number) => (x.kind === "video" ? `${tr("Vídeo")} ${n + 1}` : x.title || langName(x.lang)) + (x.title && x.lang && x.kind !== "video" ? ` (${langName(x.lang)})` : "");

  // Subtítulos: léense a WebVTT e pintanse por riba co tempo do ficheiro orixinal.
  let cues: { s: number; e: number; t: string }[] = [];
  let subText = "";
  function parseVtt(txt: string) {
    const toSec = (v: string) => {
      const p = v.trim().replace(",", ".").split(":").map(Number);
      return p.length === 3 ? p[0] * 3600 + p[1] * 60 + p[2] : p[0] * 60 + p[1];
    };
    const out: typeof cues = [];
    for (const block of txt.replace(/\r/g, "").split(/\n\n+/)) {
      const lines = block.split("\n");
      const k = lines.findIndex((l) => l.includes("-->"));
      if (k < 0) continue;
      const [a, b] = lines[k].split("-->");
      const text = lines
        .slice(k + 1)
        .join("\n")
        .replace(/<[^>]+>/g, "")
        .replace(/&amp;/g, "&")
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/\{\\an?\d\}/g, "");
      const s0 = toSec(a);
      const e0 = toSec(b.trim().split(" ")[0]);
      if (Number.isFinite(s0) && Number.isFinite(e0) && text.trim()) out.push({ s: s0, e: e0, t: text.trim() });
    }
    return out.sort((x, y) => x.s - y.s);
  }
  function cueAt(t: number): string {
    let lo = 0;
    let hi = cues.length - 1;
    let found = "";
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (cues[mid].s <= t) {
        // varias liñas poden solaparse: mira tamén as anteriores próximas
        for (let k = mid; k >= 0 && k > mid - 4; k--) if (cues[k].s <= t && t < cues[k].e) found = found ? `${cues[k].t}\n${found}` : cues[k].t;
        lo = hi + 1;
      } else hi = mid - 1;
    }
    return found;
  }
  async function pickSubtitle(i: number) {
    selS = i;
    cues = [];
    subText = "";
    if (i < 0) return;
    try {
      const path = await engine.makeSubtitles(file.path, i);
      cues = parseVtt(await (await fetch(await viewUrl(path))).text());
      if (!cues.length) toast(tr("Estes subtítulos están en branco: non hai nada que amosar."), { kind: "warn" });
      tick();
    } catch {
      selS = -1;
      toast(tr("Estes subtítulos están nun formato que o trasno non entende. Non os puido ler."), { kind: "warn" });
    }
  }
  $: pickV = { video: selV, audio: selA };

  /** Cambia de pista de vídeo ou audio: prepara o ficheiro con ela e volve ao mesmo punto. */
  async function changeTrack(kind: "video" | "audio", v: number) {
    if (kind === "video") selV = v;
    else selA = v;
    await prepare(true, { t: now, play: playing });
  }

  const SEG = 60;
  let chunked = false;
  let segIndex = 0;
  let loadingSeg = false;
  let totalDur = 0;
  let afterLoad: { offset: number; play: boolean } | null = null;
  let segToken = 0;
  let segWatch: ReturnType<typeof setTimeout>;
  const prefetched = new Set<number>();
  $: segStart = segIndex * SEG;

  function tick() {
    if (!el) return;
    now = (chunked ? segStart : 0) + el.currentTime;
    if (ready && now > 0) playPos.set(file.path, now);
    duration = chunked ? totalDur : Number.isFinite(el.duration) && el.duration ? el.duration : (file.duration ?? 0);
    mediaTime.set(Math.round(now * 100) / 100);
    if (cues.length) subText = cueAt(now);
    // O tramo seguinte vai preparándose mentres se ve este.
    if (chunked && el.currentTime > SEG * 0.4 && segStart + SEG < totalDur && !prefetched.has(segIndex + 1)) {
      const n = segIndex + 1;
      prefetched.add(n);
      engine.makePreviewChunk(file.path, n * SEG, SEG, pickV).catch(() => prefetched.delete(n));
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
    // Se o tramo nunca chega a cargar, avísase en vez de quedar «preparando» para sempre.
    clearTimeout(segWatch);
    segWatch = setTimeout(() => {
      if (mine === segToken && loadingSeg) {
        loadingSeg = false;
        failed = true;
        failMsg = tr("O tramo tardou demasiado en prepararse.");
      }
    }, 60000);
    try {
      const path = await engine.makePreviewChunk(file.path, idx * SEG, SEG, pickV);
      if (mine !== segToken) return;
      const next = await viewUrl(path);
      if (mine !== segToken) return;
      if (next === url && el) {
        // É o mesmo tramo que xa está cargado: non haberá novo «loadedmetadata», así que se vai ao punto directamente.
        clearTimeout(segWatch);
        loadingSeg = false;
        afterLoad = null;
        el.currentTime = offset;
        if (play) el.play().catch(() => {});
        tick();
      } else url = next;
    } catch (e) {
      if (mine !== segToken) return;
      clearTimeout(segWatch);
      loadingSeg = false;
      failed = true;
      failMsg = errText(e);
    }
  }
  /** Cando o tramo novo está listo: vai ao punto pedido e segue reproducindo se se estaba. */
  function onMeta() {
    ready = true;
    if (restoredFor !== file.path && !afterLoad) {
      restoredFor = file.path;
      const p = playPos.get(file.path);
      const d = chunked ? totalDur : el && Number.isFinite(el.duration) ? el.duration : (file.duration ?? 0);
      if (p && p > 2 && (!d || p < d - 2)) afterLoad = { offset: p, play: false };
    }
    if (afterLoad && el) {
      el.currentTime = afterLoad.offset;
      if (afterLoad.play) el.play().catch(() => {});
      afterLoad = null;
    }
    loadingSeg = false;
    chunkRetries = 0;
    clearTimeout(segWatch);
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
  let preparingNow = false;

  async function load(path: string) {
    ready = false;
    failed = false;
    failMsg = "";
    converted = false;
    selV = null;
    selA = null;
    selS = -1;
    cues = [];
    subText = "";
    streams = [];
    panelOpen = false;
    readStreams(path);
    chunked = false;
    segIndex = 0;
    totalDur = 0;
    loadingSeg = false;
    afterLoad = null;
    segToken++;
    prefetched.clear();
    triedPreview = false;
    chunkRetries = 0;
    preparingNow = false;
    preparing = false;
    playing = false;
    url = await viewUrl(path);
  }

  // Formatos que o navegador non reproduce (MKV, AVI, AIFF...): o trasno prepara
  // unha copia lixeira temporal. O ficheiro orixinal non se modifica.
  // Un tramo recén feito pode fallar á primeira (o ficheiro aínda ocupado ou o visor adiantándose): reinténtase un par de veces antes de dar erro.
  let chunkRetries = 0;
  function onVideoError() {
    if (chunked && chunkRetries < 2 && url) {
      chunkRetries++;
      const base = url.replace(/([?&])r=\d+/, "").replace(/[?&]$/, "");
      const again = `${base}${base.includes("?") ? "&" : "?"}r=${Date.now()}`;
      setTimeout(() => {
        if (chunked) url = again;
      }, 700);
      return;
    }
    prepare();
  }
  /** Volve intentar a vista previa sen ter que cambiar de ficheiro. */
  function retry() {
    failed = false;
    failMsg = "";
    triedPreview = false;
    chunked = false;
    segToken++;
    prepare(true);
  }
  async function prepare(force = false, resume?: { t: number; play: boolean }) {
    // Un segundo erro do reprodutor mentres a vista previa aínda se prepara non vale como fallo.
    if (triedPreview && !force && preparingNow) return;
    if (triedPreview && !force) {
      failed = true;
      loadingSeg = false;
      clearTimeout(segWatch);
      if (!failMsg) failMsg = tr("O visor non puido reproducir este vídeo.");
      return;
    }
    triedPreview = true;
    preparingNow = true;
    // Ao cambiar de pista non se desmonta o reprodutor: só se avisa por riba.
    if (resume) loadingSeg = true;
    else preparing = true;
    const mine = file.path;
    const known = force ? undefined : previewCache.get(file.path);
    if (known) {
      converted = known.converted;
      url = await viewUrl(known.path);
      preparing = false;
      return;
    }
    try {
      const first = await engine.makePreview(file.path, pickV);
      segToken++;
      prefetched.clear();
      if (first === "chunks") {
        // Vídeo que o navegador non le: tramos de 1 minuto, coa duración e o tempo do ficheiro orixinal.
        converted = true;
        chunked = true;
        totalDur = file.duration ?? Number((await engine.probeMedia(file.path)).format?.duration) ?? 0;
        if (!Number.isFinite(totalDur)) totalDur = 0;
        const saved = playPos.get(file.path);
        const t = resume?.t ?? (!resume && saved && saved > 2 && (!totalDur || saved < totalDur - 2) ? saved : 0);
        if (!resume && t) restoredFor = file.path;
        const idx = Math.floor(t / SEG);
        await loadSeg(idx, t - idx * SEG, resume?.play ?? false);
      } else {
        chunked = false;
        converted = !first.includes(".remux.");
        if (resume) afterLoad = { offset: resume.t, play: resume.play };
        url = await viewUrl(first);
        if (!force) previewCache.set(mine, { path: first, converted });
      }
    } catch (e) {
      failed = true;
      failMsg = errText(e);
      loadingSeg = false;
    } finally {
      preparing = false;
      preparingNow = false;
    }
  }
</script>

<svelte:window on:keydown={key} on:keyup={keyup} />
<svelte:document on:fullscreenchange={fsChange} />

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
      {#if $tools && !$tools.ffmpeg}<button class="btn primary" on:click={() => requestTool("ffmpeg", tr("Para ver este vídeo ou audio o trasno precisa FFmpeg, que non está instalado."), retry)}>{$t("Instalar agora")}</button>
      {:else}<button class="btn" on:click={retry}><Icon name="reset" size={14} /> {$t("Tentar de novo")}</button>{/if}
    </div>
  {:else if url && file.category === "video"}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="vrow" class:fs class:idle={fs && idle} style={fs ? `--bar-h:${barH}px` : ""} bind:this={stage} on:mousemove={wake} on:pointerdown={wake}>
      <div class="vid">
        <div class="screen" use:size>
          <!-- svelte-ignore a11y_media_has_caption, a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
          <video bind:this={el} style={vstyle} src={url} on:click={toggle} on:dblclick={fullscreen} on:error={onVideoError} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={onMeta} on:play={() => (playing = true)} on:pause={() => (playing = false)} on:ended={onEnded}></video>
          {#if subText}<div class="subs" aria-live="off">{subText}</div>{/if}
          {#if loadingSeg}<div class="segload" role="status"><span class="indet" aria-hidden="true"><span></span></span>{$t("Preparando este tramo…")}</div>{/if}
        </div>
        <div class="under" bind:clientHeight={barH}>
          {#if converted}
            <p class="note" role="note">{$t("Vista previa recodificada só para velo aquí. O ficheiro orixinal non cambia.")}<InfoTip text={$t(WHY_PREVIEW)} /></p>
          {/if}
          <div class="ctl">
            <button class="btn icon" on:click={toggle} aria-label={playing ? $t("Pausa") : $t("Reproducir")} title={`${playing ? $t("Pausa") : $t("Reproducir")} (${$t("Espazo")})`}><Icon name={playing ? "pause" : "play"} size={15} /></button>
            <div class="tlw"><Timeline {duration} {now} height={34} on:seek={(e) => seek(e.detail)} /></div>
            <span class="muted time">{fmt(now)} / {fmt(duration)}</span>
            <button class="btn icon ghost" on:click={() => (muted = !muted)} aria-label={$t("Silenciar")} title={`${$t("Silenciar")} (M)`}><Icon name={muted || vol === 0 ? "mute" : "volume"} size={15} /></button>
            <input class="vol" type="range" min="0" max="1" step="0.01" bind:value={vol} on:input={() => (muted = false)} aria-label={$t("Volume")} title={`${$t("Volume")}: ${Math.round(vol * 100)} %`} />
            {#if hasTracks}
              <button class="btn ghost" class:on={panelOpen} on:click={() => (panelOpen = !panelOpen)} aria-pressed={panelOpen} title={$t("Escoller pista de vídeo, audio ou subtítulos")}><Icon name="sliders" size={15} /> {$t("Pistas")}</button>
            {/if}
            <button class="btn icon ghost" on:click={fullscreen} aria-label={$t("Pantalla completa")} title={`${$t("Pantalla completa")} (F)`}><Icon name="expand" size={15} /></button>
          </div>
        </div>
      </div>

      {#if panelOpen && hasTracks}
        <SidePanel id="pistas" title={$t("Pistas")} on:close={() => (panelOpen = false)}>
          {#if videos.length > 1}
            <PanelSection title={$t("Vídeo")} meta={String(videos.length)}>
              <OptionList
                label={$t("Vídeo")}
                items={videos.map((x, n) => ({ value: x.i, label: nameOf(x, n), detail: describe(x) }))}
                value={selV ?? defIndex(videos)}
                on:change={(e) => changeTrack("video", e.detail)}
              />
            </PanelSection>
          {/if}
          {#if audios.length}
            <PanelSection title={$t("Audio")} meta={String(audios.length)}>
              <OptionList
                label={$t("Audio")}
                items={audios.map((x, n) => ({ value: x.i, label: nameOf(x, n), detail: describe(x) }))}
                value={selA ?? defIndex(audios)}
                on:change={(e) => changeTrack("audio", e.detail)}
              />
            </PanelSection>
          {/if}
          {#if subs.length}
            <PanelSection title={$t("Subtítulos")} meta={String(subs.length)}>
              <OptionList
                label={$t("Subtítulos")}
                items={[{ value: -1, label: $t("Sen subtítulos") }, ...subs.map((x, n) => ({ value: x.i, label: nameOf(x, n), detail: describe(x), disabled: !x.text }))]}
                value={selS}
                on:change={(e) => pickSubtitle(e.detail)}
              />
            </PanelSection>
          {/if}
        </SidePanel>
      {/if}
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
      <audio bind:this={el} src={url} on:error={() => prepare()} on:timeupdate={tick} on:seeked={tick} on:loadedmetadata={onMeta} on:play={() => (playing = true)} on:pause={() => (playing = false)} on:ended={() => (playing = false)}></audio>
    </div>
  {/if}
</div>

<style>
  .wrap { height: 100%; display: flex; align-items: center; justify-content: center; background: var(--viewer-bg); }
  .vrow { display: flex; width: 100%; height: 100%; min-height: 0; }
  .vid { position: relative; display: flex; flex-direction: column; flex: 1; min-width: 0; height: 100%; min-height: 0; }
  .screen { position: relative; flex: 1; min-height: 0; display: flex; }
  .subs { position: absolute; left: 6%; right: 6%; bottom: 7%; text-align: center; white-space: pre-line; pointer-events: none; color: #fff; font-size: clamp(0.9rem, 2.2vw, 1.5rem); line-height: 1.3; text-shadow: 0 0 3px #000, 0 0 6px #000, 1px 1px 2px #000; }
  .btn.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
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
  /* Pantalla completa: o fondo do vídeo é negro en calquera tema; a barra e o panel seguen o tema da app. */
  .vrow:fullscreen, .vrow::backdrop { background: #000; }
  .vrow.fs { background: #000; position: relative; }
  .vrow.fs .vid, .vrow.fs .screen { background: #000; }
  .vrow.fs .note { display: none; }
  .vrow.fs .screen { position: absolute; inset: 0; }
  .vrow.fs video { width: 100%; height: 100%; }
  .vrow.fs .under { position: absolute; left: 0; right: 0; bottom: 0; z-index: 5; transition: opacity 0.2s ease; }
  /* O panel de pistas queda enriba da barra, nunca a tapa: o botón «Pistas» segue á vista para pechalo. */
  .vrow.fs :global(aside.sp) { position: absolute; right: 0; top: 0; bottom: var(--bar-h, 0px); z-index: 4; height: auto; transition: opacity 0.2s ease; }
  .vrow.fs.idle .under, .vrow.fs.idle :global(aside.sp) { opacity: 0; pointer-events: none; }
  .vrow.fs.idle video { cursor: none; }
  .time { font-variant-numeric: tabular-nums; font-size: 0.85rem; }
  .ph { text-align: center; font-size: 0.82rem; }
  .msg { padding: 2rem; text-align: center; }
</style>
