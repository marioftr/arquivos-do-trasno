<script lang="ts">
  import { onMount, tick } from "svelte";
  import { downloadDir, join } from "@tauri-apps/api/path";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { t, tr, lang } from "../i18n";
  import { get } from "svelte/store";
  import { dialog, tools, refreshTools, pastedLink } from "../settings";
  import { errText, outputHook } from "../errors";
  import * as engine from "../engine";
  import { addPaths } from "../files";
  import { fileNameOf, formatBytes, formatClock, TRASNO, fallo, listo } from "../formats";
  import { createJob } from "../job";
  import { trasnoLeave } from "../flight";
  import { missingOf, type Need } from "../models";
  import { askDir } from "./tools/shared";
  import Modal from "./ui/Modal.svelte";
  import TextField from "./ui/TextField.svelte";
  import Segmented from "./ui/Segmented.svelte";
  import SelectField from "./ui/SelectField.svelte";
  import ExtensionGate from "./ui/ExtensionGate.svelte";

  /** Descargar vídeo ou audio por ligazón con yt-dlp. A descarga segue en segundo plano (procesos) e o resultado entra na lista de ficheiros. */
  const KEY = "arquivos-do-trasno.descarga";
  const saved = (() => {
    try {
      return JSON.parse(localStorage.getItem(KEY) ?? "{}") as { kind?: string; height?: number; container?: string; audio?: string; kbps?: number };
    } catch {
      return {};
    }
  })();

  let url = "";
  let kind: "video" | "audio" = saved.kind === "audio" ? "audio" : "video";
  let height = typeof saved.height === "number" ? saved.height : 0;
  let container: "mp4" | "mkv" = saved.container === "mkv" ? "mkv" : "mp4";
  let audio = typeof saved.audio === "string" ? saved.audio : "mp3";
  let kbps = typeof saved.kbps === "number" ? saved.kbps : 0;
  let urlInput: HTMLDivElement;

  const job = createJob();
  const busy = job.busy;

  const needs: Need[] = [{ kind: "tool", id: "ytdlp" }, { kind: "tool", id: "ffmpeg" }];
  $: missing = missingOf(needs, $tools).length > 0;

  onMount(async () => {
    // Só a ligazón que chega directa do botón de pegar se busca soa; a que se escribe ou pega no campo, co botón.
    const fromClipboard = get(pastedLink);
    if (fromClipboard) {
      pastedLink.set("");
      url = fromClipboard;
      autoSearch = true;
    }
    void refreshTools().catch(() => {});
    await tick();
    urlInput?.querySelector("input")?.focus();
  });

  $: try {
    localStorage.setItem(KEY, JSON.stringify({ kind, height, container, audio, kbps }));
  } catch {
    // sen almacenamento: vale mentres a app está aberta
  }

  // ---- comprobar a ligazón (co botón Buscar, ou soa se vén do botón de pegar do trasno)
  let info: engine.MediaInfo | null = null;
  let checking = false;
  let checkErr = "";
  let round = 0;
  let autoSearch = false;
  let thumb = "";
  let selected = new Set<string>();

  const looksLikeUrl = (v: string) => /^https?:\/\/[^\s/]+\.[^\s/]+/i.test(v);
  /** Ao cambiar a ligazón, o que se sabía da anterior xa non vale. */
  function forget() {
    round++;
    info = null;
    checkErr = "";
    checking = false;
    thumb = "";
  }
  $: url, forget();
  $: if (autoSearch && !missing && looksLikeUrl(url.trim())) {
    autoSearch = false;
    search();
  }
  $: canSearch = looksLikeUrl(url.trim()) && !missing && !checking && !$busy;
  function search() {
    const v = url.trim();
    if (looksLikeUrl(v) && !missing && !checking) void check(v);
  }

  async function check(v: string) {
    const mine = round;
    checking = true;
    try {
      const r = await engine.ytdlpInfo(v);
      if (mine !== round) return;
      info = r;
      selected = new Set(r.entries.map((e) => e.url));
      if (r.thumbnail) {
        // A miniatura chega despois: non fai agardar os datos.
        engine
          .ytdlpThumbnail(r.thumbnail)
          .then((p) => {
            if (mine === round) thumb = convertFileSrc(p);
          })
          .catch(() => {});
      }
    } catch (e) {
      if (mine === round) checkErr = errText(e);
    } finally {
      if (mine === round) checking = false;
    }
  }

  // ---- vídeo
  const HEIGHTS = [2160, 1440, 1080, 720, 480, 360];
  const fpsText = (f: number | null) => (f && f > 31 ? ` · ${Math.round(f)} fps` : "");
  $: videoOptions = info && !info.playlist && info.video.length
    ? [
        { value: 0, label: `${$t("Orixinal")} (${info.video[0].height}p)` },
        ...info.video.map((v) => ({ value: v.height, label: `${v.height}p${fpsText(v.fps)}${v.size ? ` · ${formatBytes(v.size)}` : ""}` })),
      ]
    : [{ value: 0, label: $t("Orixinal") }, ...HEIGHTS.map((h) => ({ value: h, label: `${h}p` }))];
  $: if (!videoOptions.some((o) => o.value === height)) height = 0;
  $: chosen = info && !info.playlist ? (height ? info.video.find((v) => v.height === height) : info.video[0]) : undefined;
  /** Como quedará o ficheiro de vídeo. */
  $: videoFormat = container === "mkv"
    ? [`MKV`, chosen ? `${chosen.height}p${fpsText(chosen.fps)}` : "", $t("sen recodificar")]
    : ["MP4", chosen ? `${chosen.codec} · ${chosen.height}p${fpsText(chosen.fps)}` : ""];
  $: videoSize = chosen?.size ? tr("uns {size}", { size: formatBytes(chosen.size) }) : "";

  // ---- audio
  $: audioFormats = [{ value: "mp3", label: "MP3" }, { value: "m4a", label: "M4A" }, { value: "opus", label: "Opus" }, { value: "flac", label: $t("FLAC (sen perdas)") }, { value: "wav", label: $t("WAV (sen perdas)") }];
  $: lossless = audio === "flac" || audio === "wav";
  $: kbpsOptions = [{ value: 0, label: info && !info.playlist && info.audio_kbps ? `${$t("Orixinal")} (${info.audio_kbps} kbps)` : $t("Orixinal") }, ...[320, 256, 192, 128, 96].map((k) => ({ value: k, label: `${k} kbps` }))];
  $: audioSource = info && !info.playlist && info.audio_kbps ? tr("{codec} a uns {k} kbps", { codec: info.audio_codec ?? "", k: info.audio_kbps }).trim() : "";

  // ---- lista
  $: entries = info?.playlist ? info.entries : [];
  /** Con lista, a xanela ábrese en dúas columnas: ligazón e axustes á esquerda, vídeos á dereita. */
  $: wide = entries.length > 0 && !missing;
  $: count = info?.playlist ? selected.size : 1;
  $: allOn = entries.length > 0 && selected.size === entries.length;
  function flip(u: string) {
    if (selected.has(u)) selected.delete(u);
    else selected.add(u);
    selected = new Set(selected);
  }
  const toggleAll = () => (selected = allOn ? new Set() : new Set(entries.map((e) => e.url)));
  const some = (node: HTMLInputElement, on: boolean) => {
    node.indeterminate = on;
    return { update: (v: boolean) => (node.indeterminate = v) };
  };

  // ---- datos que se amosan do vídeo
  const SITES: Record<string, string> = { Youtube: "YouTube", Twitter: "X (Twitter)", Tiktok: "TikTok", Soundcloud: "SoundCloud", Dailymotion: "Dailymotion", Vimeo: "Vimeo", Instagram: "Instagram", Facebook: "Facebook", Twitch: "Twitch", Reddit: "Reddit", Bandcamp: "Bandcamp" };
  const site = (k: string | null) => {
    if (!k) return "";
    const base = k.replace(/(Tab|Playlist|Channel|User|Album|Set|Series|Season|Clips?|Vod)$/, "") || k;
    return SITES[base] ?? SITES[base[0].toUpperCase() + base.slice(1).toLowerCase()] ?? base;
  };
  // Datas e miles escritos á man: o formato do sistema do webview non sempre coñece o galego e saía algo como «2026 M11 20».
  const MONTHS: Record<string, string[]> = {
    gl: ["xan", "feb", "mar", "abr", "mai", "xuñ", "xul", "ago", "set", "out", "nov", "dec"],
    es: ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"],
    en: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
  };
  const dateText = (d: string | null, l: string) => {
    const m = d && /^(\d{4})(\d{2})(\d{2})$/.exec(d);
    return m ? `${Number(m[3])} ${(MONTHS[l] ?? MONTHS.en)[Number(m[2]) - 1] ?? m[2]} ${m[1]}` : "";
  };
  const grouped = (n: number, l: string) => String(n).replace(/\B(?=(\d{3})+(?!\d))/g, l === "en" ? "," : ".");
  const clock = (s: number | null) => (s ? formatClock(s, 0) : "");
  const hostOf = (u: string) => {
    try {
      return new URL(u).hostname.replace(/^www\./, "");
    } catch {
      return u.slice(0, 40);
    }
  };
  $: facts = info
    ? [
        info.playlist ? (info.total > info.entries.length ? tr("{n} vídeos (amósanse {m})", { n: info.total, m: info.entries.length }) : tr("{n} vídeos", { n: info.entries.length })) : clock(info.duration),
        info.uploader ?? "",
        dateText(info.upload_date, $lang),
        info.views ? tr("{n} visualizacións", { n: grouped(info.views, $lang) }) : "",
      ].filter(Boolean)
    : [];

  // ---- descargar: pregunta onde gardar (como o resto da app), pecha a xanela e segue en segundo plano
  async function run() {
    const urls = info?.playlist ? entries.filter((e) => selected.has(e.url)).map((e) => e.url) : [url.trim()];
    if (!urls.length || !looksLikeUrl(url.trim()) || missing) return;
    const start = await downloadDir().catch(() => "");
    const dir = await askDir(start ? await join(start, "x") : url, start || undefined);
    if (!dir) return;
    const options: engine.DownloadOptions = { kind, maxHeight: kind === "video" && height ? height : null, container, audioFormat: audio, audioKbps: kind === "audio" && !lossless && kbps ? kbps : null };
    const title = info?.title || hostOf(url);
    const paths: string[] = [];
    trasnoLeave.set({ ticket: $t("Descargando…") });
    dialog.set(null);
    await job.run(
      TRASNO.traballando,
      fallo("editando"),
      async () => {
        const r = await engine.ytdlpDownload(urls, dir, options);
        for (const d of r.done) {
          paths.push(d.output_path);
          outputHook.fn?.(d.output_path);
        }
        const bytes = formatBytes(r.done.reduce((a, d) => a + d.bytes_written, 0));
        let text = r.done.length === 1 ? `${listo()} ${fileNameOf(r.done[0].output_path)}\n${bytes}` : `${listo()} ${tr("Descargados {n} vídeos", { n: r.done.length })}\n${bytes}`;
        if (r.failed.length) text += `\n${tr("Non se puideron descargar {n}: {detail}", { n: r.failed.length, detail: errText(r.failed[0].detail) })}`;
        return text;
      },
      title,
    );
    if (paths.length) await addPaths(paths);
  }
</script>

<Modal title={$t("Descargar por ligazón")} width={wide ? "960px" : "520px"} maxHeight="540px" mascot onClose={() => dialog.set(null)}>
  <div class="form" class:wide>
    <div class="top">
      <div class="urlrow">
        <div class="grow" bind:this={urlInput} role="presentation" on:keydown={(e) => e.key === "Enter" && search()}>
          <TextField label={$t("Ligazón")} bind:value={url} placeholder="https://…" disabled={$busy} />
        </div>
        <button class="btn" on:click={search} disabled={!canSearch} data-testid="download-search">{$t("Buscar")}</button>
      </div>

      {#if checking}
        <p class="status muted" role="status">{$t("Comprobando a ligazón…")}</p>
      {:else if checkErr}
        <p class="status warn selectable" role="alert">{checkErr}</p>
      {:else if info}
        <div class="card" data-testid="media-info">
          {#if thumb}<img class="thumb" src={thumb} alt="" data-testid="media-thumb" />{/if}
          <div class="body">
            <div class="chips">
              {#if site(info.platform)}<span class="site">{site(info.platform)}</span>{/if}
              {#if info.playlist}<span class="site alt">{$t("Lista")}</span>{/if}
            </div>
            <strong class="title">{info.title || hostOf(url)}</strong>
            {#if facts.length}<span class="facts muted">{facts.join(" · ")}</span>{/if}
          </div>
        </div>
      {/if}
    </div>

    {#if wide}
      <!-- Os vídeos da lista van nunha columna á dereita (nun móbil, debaixo dos detalles). -->
      <div class="side">
        <div class="list" data-testid="playlist">
          <label class="all"><input type="checkbox" checked={allOn} use:some={selected.size > 0 && !allOn} on:change={toggleAll} /> <span>{$t("{n} de {m} vídeos", { n: selected.size, m: entries.length })}</span></label>
          <ul class="scrollbar-thin">
            {#each entries as e, i (e.url)}
              <li><label><input type="checkbox" checked={selected.has(e.url)} on:change={() => flip(e.url)} /><span class="num">{i + 1}</span><span class="name" title={e.title}>{e.title || e.url}</span><span class="dur muted">{clock(e.duration)}</span></label></li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}

    <div class="opts">
      {#if missing}
        <ExtensionGate {needs} />
      {:else}
        <Segmented
          label={$t("Descargar")}
          options={[{ value: "video", label: $t("Vídeo") }, { value: "audio", label: $t("Audio") }]}
          bind:value={kind}
        />
        {#if kind === "video"}
          <Segmented label={$t("Formato")} options={[{ value: "mp4", label: "MP4" }, { value: "mkv", label: "MKV" }]} bind:value={container} info={$t("MP4 vai en case todos os reprodutores. MKV garda o vídeo tal como está en internet, sen recodificar, e pode levar códecs máis esixentes.")} />
          <SelectField label={$t("Calidade")} bind:value={height} options={videoOptions} def={0} hint={$t("Se non hai ese tamaño, descárgase o máis alto que haxa por debaixo.")} />
          <p class="format muted" data-testid="video-format">{$t("Sairá en")} <strong>{videoFormat.filter(Boolean).join(" · ")}</strong>{videoSize ? ` · ${videoSize}` : ""}</p>
        {:else}
          <SelectField label={$t("Formato")} bind:value={audio} def="mp3" options={audioFormats} />
          {#if !lossless}
            <SelectField label={$t("Calidade")} bind:value={kbps} options={kbpsOptions} def={0} hint={$t("Un bitrate máis alto pesa máis. Por riba do da fonte non mellora o son.")} />
          {/if}
          <p class="format muted" data-testid="audio-format">
            {$t("Sairá en")} <strong>{audio.toUpperCase()}{lossless ? "" : ` · ${kbps ? `${kbps} kbps` : $t("calidade orixinal")}`}</strong>{audioSource ? ` · ${tr("a fonte é {src}", { src: audioSource })}` : ""}
          </p>
        {/if}

        <button class="btn primary block" on:click={run} disabled={!looksLikeUrl(url.trim()) || checking || $busy || (!!info?.playlist && !count)} data-testid="download-go">
          {info?.playlist ? (count === 1 ? $t("Descargar 1 vídeo") : tr("Descargar {n} vídeos", { n: count })) : $t("Descargar")}
        </button>
      {/if}
    </div>
  </div>
</Modal>

<style>
  .form { display: grid; grid-template-columns: minmax(0, 1fr); grid-template-areas: "top" "side" "opts"; gap: 0.85rem; width: 100%; }
  .form.wide { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); grid-template-areas: "top side" "opts side"; grid-template-rows: auto 1fr; column-gap: 1.1rem; }
  .top, .opts { display: flex; flex-direction: column; gap: 0.85rem; min-width: 0; }
  .top { grid-area: top; }
  .opts { grid-area: opts; }
  .side { grid-area: side; position: relative; min-width: 0; min-height: 0; }
  .status { margin: 0; font-size: var(--fs-md); }
  .warn { color: var(--accent-pdf); }
  .urlrow { display: flex; align-items: flex-end; gap: 0.5rem; }
  .grow { flex: 1; min-width: 0; }
  .card { display: flex; gap: 0.75rem; align-items: flex-start; padding: 0.7rem 0.8rem; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); min-width: 0; }
  .thumb { flex: none; width: 7.5rem; aspect-ratio: 16 / 9; object-fit: cover; border-radius: var(--radius-sm); background: var(--bg-hover); }
  .body { display: flex; flex-direction: column; gap: 0.3rem; min-width: 0; flex: 1; }
  .chips { display: flex; gap: 0.4rem; }
  .site { align-self: flex-start; font-size: var(--fs-xs); font-weight: 600; padding: 0.1rem 0.5rem; border-radius: var(--radius-sm); background: color-mix(in srgb, var(--accent-video) 18%, transparent); color: var(--accent-video); }
  .site.alt { background: color-mix(in srgb, var(--accent) 18%, transparent); color: var(--accent); }
  .title { font-size: var(--fs-md); line-height: 1.35; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; overflow-wrap: anywhere; }
  .facts { font-size: var(--fs-sm); }
  .format { margin: -0.3rem 0 0; font-size: var(--fs-sm); }
  .format strong { color: var(--text); font-weight: 600; }
  .list { display: flex; flex-direction: column; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); min-height: 0; overflow: hidden; }
  /* En dúas columnas, a lista enche o alto da columna sen estirar a xanela; ao desbordar, desprázase por dentro. */
  .wide .side { min-height: 14rem; }
  .wide .list { position: absolute; inset: 0; }
  .wide ul { flex: 1; min-height: 0; max-height: none; }
  .all { display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 0.7rem; font-size: var(--fs-md); font-weight: 550; border-bottom: 1px solid var(--border); cursor: pointer; }
  ul { list-style: none; margin: 0; padding: 0.2rem 0; max-height: 9.5rem; overflow-y: auto; }
  li label { display: flex; align-items: center; gap: 0.55rem; padding: 0.3rem 0.7rem; cursor: pointer; font-size: var(--fs-md); }
  li label:hover { background: var(--bg-hover); }
  input[type="checkbox"] { flex: none; }
  .num { flex: none; min-width: 1.6rem; text-align: right; color: var(--text-faint); font-size: var(--fs-sm); font-variant-numeric: tabular-nums; }
  .name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dur { flex: none; font-size: var(--fs-sm); font-variant-numeric: tabular-nums; }
  @media (max-width: 820px) {
    .form.wide { grid-template-columns: minmax(0, 1fr); grid-template-areas: "top" "side" "opts"; grid-template-rows: none; }
    .wide .side { min-height: 0; }
    .wide .list { position: static; }
    .wide ul { flex: none; max-height: 10rem; }
  }
</style>
