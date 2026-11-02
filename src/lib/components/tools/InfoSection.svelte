<script lang="ts">
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { t, tr, lang } from "../../i18n";
  import Segmented from "../ui/Segmented.svelte";

  export let file: FileEntry;

  type Row = [string, string];
  interface Group {
    title: string;
    rows: Row[];
  }
  let summary: Group[] = [];
  let advanced: Group[] = [];
  let raw: Group[] = []; // datos sen traballar, só na vista avanzada
  let hash = "";
  let hashing = false;
  let note = "";
  let loading = false;
  let level: "summary" | "advanced" = "summary";

  $: isMedia = file.category === "audio" || file.category === "video";
  $: load(file);

  // ---------- formato dos valores
  const locale = () => ($lang === "gl" ? "gl-ES" : $lang);
  const num = (n: number, d = 1) => n.toLocaleString(locale(), { maximumFractionDigits: d });
  /** Valores especiais: `@date:segundos` escríbese como data e hora locais. */
  function show(v: string): string {
    if (v.startsWith("@date:")) return new Date(Number(v.slice(6)) * 1000).toLocaleString(locale(), { dateStyle: "long", timeStyle: "short" });
    return $t(v);
  }
  function dur(s: number): string {
    if (!Number.isFinite(s)) return "";
    if (s < 60) return `${num(s)} s`;
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = Math.round(s % 60);
    return h ? `${h} h ${m} min ${sec} s` : `${m} min ${sec} s`;
  }
  const rate = (v: unknown) => {
    const n = Number(v);
    if (!Number.isFinite(n) || n <= 0) return "";
    return n >= 1_000_000 ? `${num(n / 1_000_000)} Mbps` : `${Math.round(n / 1000)} kbps`;
  };
  const size = (b: unknown) => {
    const n = Number(b);
    if (!Number.isFinite(n)) return "";
    const u = ["B", "KB", "MB", "GB", "TB"];
    let i = 0;
    let x = n;
    while (x >= 1024 && i < 4) (x /= 1024), i++;
    return i ? `${num(x)} ${u[i]} (${n.toLocaleString(locale())} bytes)` : `${n} B`;
  };
  const fpsOf = (r?: string) => {
    if (!r || r === "0/0") return 0;
    const [a, b] = r.split("/").map(Number);
    return b ? a / b : 0;
  };
  const ratio = (w: number, h: number) => {
    const r = w / h;
    for (const [a, b] of [[1, 1], [4, 3], [3, 2], [16, 9], [21, 9], [5, 4], [16, 10], [3, 4], [2, 3], [9, 16], [4, 5]]) if (Math.abs(a / b - r) < 0.015) return `${a}:${b}`;
    return `${num(r, 2)}:1`;
  };
  const CODECS: Record<string, string> = {
    h264: "H.264 (AVC, o máis habitual)", hevc: "H.265 (HEVC, moi comprimido)", av1: "AV1 (moderno, moi comprimido)", vp9: "VP9 (web)", vp8: "VP8 (web)", mpeg4: "MPEG-4 (antigo)", mpeg2video: "MPEG-2 (DVD, televisión)", mpeg1video: "MPEG-1",
    dvvideo: "DV (cinta Mini DV)", mjpeg: "Motion JPEG (cada imaxe é un JPEG)", prores: "Apple ProRes (edición profesional)", wmv3: "Windows Media Video", wmv2: "Windows Media Video", theora: "Theora", gif: "GIF animado", png: "PNG", rawvideo: "Sen comprimir",
    aac: "AAC (o máis habitual)", mp3: "MP3", ac3: "Dolby Digital (AC-3)", eac3: "Dolby Digital Plus", dts: "DTS", opus: "Opus", vorbis: "Vorbis (OGG)", flac: "FLAC (sen perdas)", alac: "Apple Lossless (sen perdas)",
    pcm_s16le: "PCM sen comprimir (16 bits)", pcm_s24le: "PCM sen comprimir (24 bits)", pcm_s32le: "PCM sen comprimir (32 bits)", pcm_u8: "PCM sen comprimir (8 bits)", wmav2: "Windows Media Audio", mp2: "MP2",
    subrip: "SubRip (texto, .srt)", ass: "ASS/SSA (texto con estilos)", webvtt: "WebVTT (texto)", mov_text: "Texto de MP4", hdmv_pgs_subtitle: "PGS (imaxe de Blu-ray)", dvd_subtitle: "DVD (imaxe)", dvb_subtitle: "DVB (imaxe)",
  };
  const codec = (s: any) => CODECS[s.codec_name] ?? s.codec_long_name ?? s.codec_name ?? "";
  const CONT: [RegExp, string][] = [
    [/matroska/i, "Matroska (MKV / WebM)"], [/mov|mp4/i, "MP4 / MOV"], [/avi/i, "AVI"], [/mpegts/i, "MPEG-TS (transmisión)"], [/flv/i, "Flash Video (FLV)"], [/asf/i, "Windows Media (WMV / WMA)"],
    [/ogg/i, "OGG"], [/wav/i, "WAV"], [/flac/i, "FLAC"], [/mp3/i, "MP3"], [/aiff/i, "AIFF"], [/mpeg/i, "MPEG"],
  ];
  const container = (f: any) => CONT.find(([re]) => re.test(f.format_name ?? ""))?.[1] ?? f.format_long_name ?? f.format_name ?? "";
  const quality = (w: number, h: number) => {
    const m = Math.max(w, h) >= Math.max(Math.min(w, h), 1) ? Math.min(w, h) : Math.max(w, h);
    return m >= 2000 ? "4K ou máis" : m >= 1400 ? "2K" : m >= 1000 ? "Full HD" : m >= 700 ? "HD" : "Definición estándar";
  };
  const PIX: Record<string, string> = { yuv420p: "Estándar (YUV 4:2:0, 8 bits)", yuvj420p: "Estándar (YUV 4:2:0, rango completo)", yuv422p: "Alta calidade (YUV 4:2:2)", yuv444p: "Máxima (YUV 4:4:4)", yuv420p10le: "10 bits (HDR ou cor ampla)", rgb24: "RGB 8 bits", gray: "Escala de grises" };
  const chan = (s: any) => (s.channels === 1 ? "Mono" : s.channels === 2 ? "Estéreo" : s.channels === 6 ? "5.1 (son envolvente)" : s.channels === 8 ? "7.1 (son envolvente)" : s.channels ? `${s.channels} canles` : s.channel_layout ?? "");
  const dateOf = (iso: string) => {
    const d = Date.parse(iso);
    return Number.isNaN(d) ? "" : `@date:${Math.floor(d / 1000)}`;
  };
  const rows = (...r: (Row | null | false | "" | 0 | undefined)[]): Row[] => r.filter((x): x is Row => !!x && !!x[1]);

  function mediaSummary(info: any): Group[] {
    const f = info.format ?? {};
    const out: Group[] = [];
    const tg = f.tags ?? {};
    out.push({
      title: tr("Xeral"),
      rows: rows(
        ["Tipo de ficheiro", container(f)],
        f.duration && ["Duración", dur(Number(f.duration))],
        ["Tamaño", size(f.size)],
        f.bit_rate && ["Datos por segundo (total)", rate(f.bit_rate)],
        tg.creation_time && ["Data de creación", dateOf(String(tg.creation_time))],
        (tg.encoder ?? tg.ENCODER ?? tg.writing_application) && ["Creado con", String(tg.encoder ?? tg.ENCODER ?? tg.writing_application)],
        tg.title && ["Título", String(tg.title)],
      ),
    });
    const per: Record<string, number> = {};
    const all: any[] = info.streams ?? [];
    for (const s of all) {
      const k = s.codec_type as string;
      per[k] = (per[k] ?? 0) + 1;
      const many = all.filter((x) => x.codec_type === k).length > 1;
      const lang = s.tags?.language && s.tags.language !== "und" ? s.tags.language : "";
      const name = s.tags?.title ?? "";
      if (k === "video") {
        if (s.disposition?.attached_pic) continue;
        const w = Number(s.width);
        const h = Number(s.height);
        const fr = fpsOf(s.avg_frame_rate) || fpsOf(s.r_frame_rate);
        const inter = s.field_order && !["progressive", "unknown"].includes(s.field_order);
        out.push({
          title: many ? `${tr("Vídeo")} ${per[k]}` : tr("Vídeo"),
          rows: rows(
            ["Compresión", codec(s)],
            w && h && ["Tamaño da imaxe", `${w} × ${h} px · ${tr(quality(w, h))}`],
            w && h && ["Forma", `${ratio(w, h)} · ${tr(w === h ? "Cadrada" : w > h ? "Horizontal" : "Vertical")}`],
            fr && ["Fluidez", `${num(fr, 2)} ${tr("imaxes por segundo")}`],
            ["Barrido", inter ? tr("Entrelazado (as liñas van en dous pasos; convén desentrelazar)") : tr("Progresivo (normal)")],
            s.pix_fmt && ["Cor", PIX[s.pix_fmt] ?? s.pix_fmt],
            s.bit_rate && ["Datos por segundo", rate(s.bit_rate)],
            lang && ["Idioma", lang],
            name && ["Nome da pista", String(name)],
          ),
        });
      } else if (k === "audio") {
        out.push({
          title: many ? `${tr("Audio")} ${per[k]}` : tr("Audio"),
          rows: rows(
            ["Compresión", codec(s)],
            ["Canles", chan(s)],
            s.sample_rate && ["Calidade do son", `${num(Number(s.sample_rate) / 1000, 1)} kHz`],
            s.bit_rate && ["Datos por segundo", rate(s.bit_rate)],
            lang && ["Idioma", lang],
            name && ["Nome da pista", String(name)],
          ),
        });
      } else if (k === "subtitle") {
        const text = !/pgs|dvd|dvb|xsub/.test(s.codec_name ?? "");
        out.push({
          title: many ? `${tr("Subtítulos")} ${per[k]}` : tr("Subtítulos"),
          rows: rows(["Formato", codec(s)], ["Tipo", text ? tr("Texto (pódese ler e editar)") : tr("Imaxe (non se pode editar)")], lang && ["Idioma", lang], name && ["Nome da pista", String(name)]),
        });
      }
    }
    return out;
  }

  // Datos técnicos coñecidos, con nome en palabras; o que non está nesta lista vai a «datos en bruto».
  const KNOWN: Record<string, [string, ((v: any) => string)?]> = {
    profile: ["Perfil"], level: ["Nivel"], pix_fmt: ["Formato de píxel"], color_space: ["Espazo de cor"], color_range: ["Rango de cor"], color_transfer: ["Curva de cor"], color_primaries: ["Cores primarias"],
    field_order: ["Orde das liñas"], sample_aspect_ratio: ["Forma do píxel"], display_aspect_ratio: ["Forma da imaxe"], has_b_frames: ["Fotogramas B"], refs: ["Fotogramas de referencia"],
    r_frame_rate: ["Velocidade nominal", (v) => `${num(fpsOf(v), 3)} ${tr("imaxes por segundo")}`], avg_frame_rate: ["Velocidade media", (v) => `${num(fpsOf(v), 3)} ${tr("imaxes por segundo")}`],
    time_base: ["Unidade de tempo"], nb_frames: ["Número de imaxes"], start_time: ["Comeza en", (v) => `${num(Number(v), 3)} s`], duration: ["Duración da pista", (v) => dur(Number(v))],
    bit_rate: ["Datos por segundo", rate], bits_per_raw_sample: ["Bits por mostra"], bits_per_sample: ["Bits por mostra"], coded_width: ["Ancho codificado"], coded_height: ["Alto codificado"],
    sample_fmt: ["Formato de mostra"], sample_rate: ["Mostras por segundo", (v) => `${num(Number(v), 0)} Hz`], channels: ["Canles"], channel_layout: ["Disposición das canles"],
    codec_name: ["Nome técnico do códec"], codec_long_name: ["Nome completo do códec"], format_name: ["Nome técnico do contedor"], nb_streams: ["Pistas no ficheiro"], probe_score: ["Seguranza ao detectar o formato (0–100)"],
  };
  const SKIP = new Set(["index", "codec_type", "codec_tag", "codec_tag_string", "width", "height", "filename", "size", "tags", "disposition", "id", "extradata_size", "format_long_name", "nb_programs", "nb_stream_groups"]);

  function flatten(o: Record<string, unknown>, known: boolean): { rows: Row[]; rest: Row[] } {
    const r: Row[] = [];
    const rest: Row[] = [];
    for (const [k, v] of Object.entries(o ?? {})) {
      if (SKIP.has(k) || v === undefined || v === null || v === "" || v === "N/A") continue;
      const hit = known ? KNOWN[k] : undefined;
      if (hit) r.push([hit[0], hit[1] ? hit[1](v) : String(v)]);
      else if (typeof v !== "object") rest.push([k, String(v)]);
    }
    return { rows: r.filter((x) => x[1]), rest };
  }
  function mediaAdvanced(info: any): { adv: Group[]; raw: Group[] } {
    const adv: Group[] = [];
    const f = info.format ?? {};
    const g = flatten(f, true);
    adv.push({ title: tr("Ficheiro"), rows: [["Tamaño", size(f.size)], ...g.rows].filter((x) => x[1]) as Row[] });
    const tags = Object.entries(f.tags ?? {}).map(([k, v]) => [k, String(v)] as Row);
    if (tags.length) adv.push({ title: tr("Etiquetas do ficheiro"), rows: tags });
    (info.streams ?? []).forEach((s: any, i: number) => {
      const x = flatten(s, true);
      const label = s.codec_type === "video" ? tr("Vídeo") : s.codec_type === "audio" ? tr("Audio") : s.codec_type === "subtitle" ? tr("Subtítulos") : tr("Pista");
      const d = Object.entries(s.disposition ?? {}).filter(([, v]) => v === 1).map(([k]) => k.replace(/_/g, " "));
      adv.push({ title: `${label} · ${tr("pista")} ${i + 1}`, rows: [...x.rows, ...(d.length ? ([["Marcas", d.join(", ")]] as Row[]) : []), ...Object.entries(s.tags ?? {}).map(([k, v]) => [k, String(v)] as Row)] });
    });
    (info.chapters ?? []).forEach((c: any, i: number) => adv.push({ title: `${tr("Capítulo")} ${i + 1}`, rows: [["Comeza", dur(Number(c.start_time))], ["Remata", dur(Number(c.end_time))], ...Object.entries(c.tags ?? {}).map(([k, v]) => [k, String(v)] as Row)] }));
    return { adv, raw: rawAll(info) };
  }

  /** Todo o que dá ffprobe, tal cal (nome técnico e valor), un grupo por ficheiro, pista e capítulo. */
  function rawAll(info: any): Group[] {
    const flat = (o: Record<string, unknown>, prefix = ""): Row[] => {
      const r: Row[] = [];
      for (const [k, v] of Object.entries(o ?? {})) {
        if (v && typeof v === "object" && !Array.isArray(v)) r.push(...flat(v as Record<string, unknown>, `${prefix}${k}.`));
        else r.push([`${prefix}${k}`, Array.isArray(v) ? JSON.stringify(v) : String(v)]);
      }
      return r;
    };
    const out: Group[] = [{ title: tr("Ficheiro"), rows: flat(info.format ?? {}) }];
    (info.streams ?? []).forEach((s: any, i: number) => {
      const label = s.codec_type === "video" ? tr("Vídeo") : s.codec_type === "audio" ? tr("Audio") : s.codec_type === "subtitle" ? tr("Subtítulos") : tr("Pista");
      out.push({ title: `${label} · ${tr("pista")} ${i + 1}`, rows: flat(s) });
    });
    (info.chapters ?? []).forEach((c: any, i: number) => out.push({ title: `${tr("Capítulo")} ${i + 1}`, rows: flat(c) }));
    return out;
  }

  async function load(f: FileEntry) {
    hash = "";
    note = "";
    summary = [];
    advanced = [];
    raw = [];
    loading = true;
    const path = f.path;
    try {
      if (f.category === "audio" || f.category === "video") {
        let info: any;
        try {
          info = await engine.probeMedia(path);
        } catch {
          note = tr("Sen ffmpeg non se pode ler a información de audio e vídeo.");
          const b = await engine.fileInfo(path);
          if (path === file.path) summary = b.summary.map((g) => ({ title: g.title, rows: g.rows }));
          return;
        }
        if (path !== file.path) return;
        summary = mediaSummary(info);
        const a = mediaAdvanced(info);
        advanced = a.adv;
        raw = a.raw;
      } else {
        const r = await engine.fileInfo(path);
        if (path !== file.path) return;
        summary = r.summary;
        advanced = r.advanced;
      }
    } catch (e) {
      note = errText(e);
    } finally {
      if (path === file.path) loading = false;
    }
  }

  async function computeHash() {
    hashing = true;
    try {
      hash = await engine.sha256(file.path);
    } catch (e) {
      hash = `${tr("erro")}: ${errText(e)}`;
    } finally {
      hashing = false;
    }
  }
  $: shown = level === "summary" || !advanced.length ? summary : advanced;
</script>

<div class="stack">
  {#if advanced.length}
    <Segmented
      label={$t("Nivel de detalle")}
      options={[
        { value: "summary", label: $t("Resumo") },
        { value: "advanced", label: $t("Avanzada") },
      ]}
      bind:value={level}
    />
  {/if}

  <div class="group">
    <h4 class="group-title">{$t("Ubicación")}</h4>
    <dl class="selectable">
      <dt>{$t("Ruta")}</dt>
      <dd class="wrap">{file.path}</dd>
      {#if hash}
        <dt>SHA-256</dt>
        <dd class="wrap hash">{hash}</dd>
      {/if}
    </dl>
    {#if note}<p class="group-note">{note}</p>{/if}
  </div>

  {#if loading && !shown.length}<p class="muted small">{$t("Lendo o ficheiro…")}</p>{/if}
  {#each shown as g (g.title + level)}
    {#if g.rows.length}
      <div class="group">
        <h4 class="group-title">{$t(g.title)}</h4>
        <dl class="selectable">
          {#each g.rows as [k, v]}
            <dt>{$t(k)}</dt>
            <dd class="wrap">{show(v)}</dd>
          {/each}
        </dl>
      </div>
    {/if}
  {/each}

  {#if level === "advanced" && raw.length}
    <details class="raw">
      <summary>{$t("Datos técnicos en bruto")}</summary>
      {#each raw as g (g.title)}
        <details class="sub">
          <summary>{g.title} <span class="muted">· {g.rows.length}</span></summary>
          <dl class="selectable mono">
            {#each g.rows as [k, v]}
              <dt>{k}</dt>
              <dd class="wrap">{v}</dd>
            {/each}
          </dl>
        </details>
      {/each}
    </details>
  {/if}

  <div class="group">
    <p class="group-note">{$t("Pegada dixital do ficheiro: se dous ficheiros teñen o mesmo SHA-256, son idénticos byte a byte")}</p>
    <button class="btn block" on:click={computeHash} disabled={hashing}>{hashing ? $t("Calculando…") : $t("Calcular SHA-256")}</button>
    <button class="btn block" on:click={() => engine.revealInFolder(file.path)}>{$t("Mostrar no cartafol")}</button>
  </div>
</div>

<style>
  dl { display: grid; grid-template-columns: minmax(6.5rem, 38%) 1fr; gap: 0.45rem 0.9rem; margin: 0; font-size: 0.84rem; }
  dt { color: var(--text-muted); }
  dd { margin: 0; min-width: 0; }
  dd.wrap { white-space: normal; overflow-wrap: anywhere; line-height: 1.35; }
  .hash { font-family: Consolas, monospace; font-size: 0.72rem; }
  .small { font-size: 0.82rem; margin: 0; }
  .raw { border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 0.5rem 0.7rem; }
  .raw summary { cursor: pointer; font-size: 0.84rem; color: var(--text-muted); }
  .sub { margin-top: 0.5rem; border-top: 1px solid var(--border); padding-top: 0.4rem; }
  .sub summary { cursor: pointer; font-size: 0.82rem; font-weight: 600; }
  .sub dl { margin-top: 0.5rem; }
  dl.mono { font-size: 0.74rem; }
  dl.mono dt { font-family: Consolas, monospace; }
</style>
