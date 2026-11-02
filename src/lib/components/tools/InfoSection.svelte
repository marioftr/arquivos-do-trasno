<script lang="ts">
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { formatBytes } from "../../formats";
  import { t, tr } from "../../i18n";
  import Segmented from "../ui/Segmented.svelte";

  export let file: FileEntry;

  type Row = [string, string];
  interface Group {
    title: string;
    rows: Row[];
  }
  let rows: Row[] = [];
  let groups: Group[] = []; // resumo por pista (vídeo, audio, subtítulos)
  let advanced: Group[] = []; // todos os datos que dá ffprobe
  let hash = "";
  let hashing = false;
  let note = "";
  let level: "summary" | "advanced" = "summary";

  $: isMedia = file.category === "audio" || file.category === "video";
  $: load(file);

  const clock = (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = (s - h * 3600 - m * 60).toFixed(1).padStart(4, "0");
    return h ? `${h}:${String(m).padStart(2, "0")}:${sec}` : `${m}:${sec}`;
  };
  const kbps = (v: unknown) => (v !== undefined && v !== "" && Number.isFinite(Number(v)) && Number(v) > 0 ? `${Math.round(Number(v) / 1000)} kbps` : "");
  const fps = (r?: string) => {
    if (!r || r === "0/0") return "";
    const [a, b] = r.split("/").map(Number);
    return b ? `${(a / b).toFixed(a / b % 1 ? 3 : 0).replace(/\.?0+$/, "")} fps` : "";
  };
  const join = (...x: (string | undefined | false)[]) => x.filter(Boolean).join(" · ");
  const channels = (s: any) => s.channel_layout || (s.channels ? `${s.channels} ${tr("canles")}` : "");

  function summarise(info: any): Group[] {
    const out: Group[] = [];
    const f = info.format ?? {};
    const general: Row[] = [];
    if (f.format_long_name) general.push(["Contedor", f.format_long_name]);
    if (f.duration) general.push(["Duración", clock(Number(f.duration))]);
    const br = kbps(f.bit_rate);
    if (br) general.push(["Bitrate total", br]);
    if (f.tags?.creation_time) general.push(["Data de creación", String(f.tags.creation_time).replace("T", " ").replace(/\.\d+Z?$/, "")]);
    const enc = f.tags?.encoder ?? f.tags?.ENCODER ?? f.tags?.writing_application;
    if (enc) general.push(["Creado con", String(enc)]);
    if (f.tags?.title) general.push(["Título", String(f.tags.title)]);
    if (general.length) out.push({ title: tr("Xeral"), rows: general });
    const streams: any[] = info.streams ?? [];
    const count: Record<string, number> = {};
    for (const s of streams) {
      const kind = s.codec_type as string;
      count[kind] = (count[kind] ?? 0) + 1;
      const n = count[kind];
      const lang = s.tags?.language && s.tags.language !== "und" ? s.tags.language : "";
      const title = s.tags?.title ?? s.tags?.handler_name ?? "";
      const r: Row[] = [];
      if (kind === "video") {
        r.push(["Códec", join(s.codec_long_name ?? s.codec_name, s.profile && `(${s.profile})`)]);
        r.push(["Resolución", `${s.width} × ${s.height}`]);
        if (s.display_aspect_ratio && s.display_aspect_ratio !== "0:1") r.push(["Relación de aspecto", s.display_aspect_ratio]);
        const rate = fps(s.avg_frame_rate) || fps(s.r_frame_rate);
        if (rate) r.push(["Fotogramas", rate]);
        if (s.field_order && s.field_order !== "progressive" && s.field_order !== "unknown") r.push(["Escaneado", tr("Entrelazado ({o})", { o: s.field_order })]);
        r.push(["Formato de cor", join(s.pix_fmt, s.color_range, s.color_space)]);
        if (kbps(s.bit_rate)) r.push(["Bitrate", kbps(s.bit_rate)]);
        if (s.bits_per_raw_sample) r.push(["Profundidade", `${s.bits_per_raw_sample} bits`]);
      } else if (kind === "audio") {
        r.push(["Códec", join(s.codec_long_name ?? s.codec_name, s.profile && `(${s.profile})`)]);
        r.push(["Canles", channels(s)]);
        if (s.sample_rate) r.push(["Mostraxe", `${s.sample_rate} Hz${s.sample_fmt ? ` · ${s.sample_fmt}` : ""}`]);
        if (kbps(s.bit_rate)) r.push(["Bitrate", kbps(s.bit_rate)]);
      } else if (kind === "subtitle") {
        r.push(["Códec", s.codec_long_name ?? s.codec_name]);
      } else {
        r.push(["Códec", s.codec_long_name ?? s.codec_name ?? kind]);
      }
      if (lang) r.push(["Idioma", lang]);
      if (title) r.push(["Nome da pista", String(title)]);
      const label = kind === "video" ? tr("Vídeo") : kind === "audio" ? tr("Audio") : kind === "subtitle" ? tr("Subtítulos") : kind;
      out.push({ title: count[kind] > 1 || streams.filter((x) => x.codec_type === kind).length > 1 ? `${label} ${n}` : label, rows: r.filter(([, v]) => v) });
    }
    return out;
  }

  /** Todo o que dá ffprobe, sen filtrar: cada campo e cada etiqueta do ficheiro e das pistas. */
  function everything(info: any): Group[] {
    const flat = (o: Record<string, unknown>, prefix = ""): Row[] => {
      const r: Row[] = [];
      for (const [k, v] of Object.entries(o ?? {})) {
        if (v && typeof v === "object" && !Array.isArray(v)) r.push(...flat(v as Record<string, unknown>, `${prefix}${k}.`));
        else if (Array.isArray(v)) r.push([`${prefix}${k}`, JSON.stringify(v)]);
        else r.push([`${prefix}${k}`, String(v)]);
      }
      return r;
    };
    const out: Group[] = [{ title: tr("Ficheiro"), rows: flat(info.format ?? {}) }];
    (info.streams ?? []).forEach((s: any, i: number) => out.push({ title: `${tr("Pista")} ${i} · ${s.codec_type}`, rows: flat(s) }));
    (info.chapters ?? []).forEach((c: any, i: number) => out.push({ title: `${tr("Capítulo")} ${i + 1}`, rows: flat(c) }));
    return out;
  }

  async function load(f: FileEntry) {
    hash = "";
    note = "";
    groups = [];
    advanced = [];
    rows = [["Ruta", f.path]];
    if (f.size !== undefined) rows.push(["Tamaño", formatBytes(f.size)]);
    if (f.width) rows.push(["Dimensións", `${f.width} × ${f.height} px`]);
    if (f.pages) rows.push(["Páxinas", String(f.pages)]);
    if (f.category === "audio" || f.category === "video") {
      try {
        const info = await engine.probeMedia(f.path);
        if (f.path !== file.path) return;
        groups = summarise(info);
        advanced = everything(info);
      } catch {
        note = tr("Sen ffmpeg non se pode ler a información de audio e vídeo.");
      }
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
</script>

<div class="stack">
  {#if isMedia && (groups.length || advanced.length)}
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
    <h4 class="group-title">{$t("Información")}</h4>
    <dl class="selectable">
      {#each rows as [k, v]}
        <dt>{$t(k)}</dt>
        <dd title={v}>{v}</dd>
      {/each}
      {#if hash}
        <dt>SHA-256</dt>
        <dd class="hash" title={hash}>{hash}</dd>
      {/if}
    </dl>
    {#if note}<p class="group-note">{note}</p>{/if}
  </div>

  {#if isMedia}
    {#each level === "summary" ? groups : advanced as g (g.title + level)}
      <div class="group">
        <h4 class="group-title">{g.title}</h4>
        <dl class="selectable" class:raw={level === "advanced"}>
          {#each g.rows as [k, v]}
            <dt>{level === "advanced" ? k : $t(k)}</dt>
            <dd title={v}>{v}</dd>
          {/each}
        </dl>
      </div>
    {/each}
  {/if}

  <div class="group">
    <p class="group-note">{$t("Pegada dixital do ficheiro: se dous ficheiros teñen o mesmo SHA-256, son idénticos byte a byte")}</p>
    <button class="btn block" on:click={computeHash} disabled={hashing}>{hashing ? $t("Calculando…") : $t("Calcular SHA-256")}</button>
    <button class="btn block" on:click={() => engine.revealInFolder(file.path)}>{$t("Mostrar no cartafol")}</button>
  </div>
</div>

<style>
  dl { display: grid; grid-template-columns: auto 1fr; gap: 0.4rem 0.9rem; margin: 0; font-size: 0.84rem; }
  dt { color: var(--text-muted); }
  dd { margin: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  dl.raw { font-size: 0.78rem; }
  dl.raw dt { font-family: Consolas, monospace; font-size: 0.74rem; }
  .hash { font-family: Consolas, monospace; font-size: 0.72rem; }
</style>
