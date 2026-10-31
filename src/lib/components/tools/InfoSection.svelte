<script lang="ts">
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { formatBytes } from "../../formats";

  export let file: FileEntry;

  let rows: [string, string][] = [];
  let hash = "";
  let hashing = false;
  let note = "";

  $: load(file);

  function seconds(s?: string): string {
    const n = Number(s);
    if (!isFinite(n)) return "";
    const m = Math.floor(n / 60);
    return `${m}:${(n - m * 60).toFixed(1).padStart(4, "0")}`;
  }

  async function load(f: FileEntry) {
    hash = "";
    note = "";
    rows = [["Ruta", f.path]];
    if (f.size !== undefined) rows.push(["Tamaño", formatBytes(f.size)]);
    if (f.width) rows.push(["Dimensións", `${f.width} × ${f.height} px`]);
    if (f.pages) rows.push(["Páxinas", String(f.pages)]);
    if (f.category === "audio" || f.category === "video") {
      try {
        const info = await engine.probeMedia(f.path);
        const v = info.streams?.find((s) => s.codec_type === "video");
        const a = info.streams?.find((s) => s.codec_type === "audio");
        const extra: [string, string][] = [];
        if (info.format?.duration) extra.push(["Duración", seconds(info.format.duration)]);
        if (v) extra.push(["Vídeo", `${v.codec_name} · ${v.width}×${v.height}`]);
        if (a) extra.push(["Audio", `${a.codec_name}${a.sample_rate ? ` · ${a.sample_rate} Hz` : ""}`]);
        if (info.format?.bit_rate) extra.push(["Bitrate", `${Math.round(Number(info.format.bit_rate) / 1000)} kbps`]);
        if (f.path === file.path) rows = [...rows, ...extra];
      } catch {
        note = "Sen ffmpeg non se pode ler a información de audio e vídeo.";
      }
    }
  }

  async function computeHash() {
    hashing = true;
    try {
      hash = await engine.sha256(file.path);
    } catch (e) {
      hash = `erro: ${e}`;
    } finally {
      hashing = false;
    }
  }
</script>

<section>
  <h3>Información</h3>
  <dl class="selectable">
    {#each rows as [k, v]}
      <dt>{k}</dt>
      <dd title={v}>{v}</dd>
    {/each}
    {#if hash}
      <dt>SHA-256</dt>
      <dd class="hash" title={hash}>{hash}</dd>
    {/if}
  </dl>
  {#if note}<p class="muted tiny">{note}</p>{/if}
  <div class="acts">
    <button class="btn" on:click={computeHash} disabled={hashing}>{hashing ? "Calculando…" : "Calcular SHA-256"}</button>
    <button class="btn" on:click={() => engine.revealInFolder(file.path)}>Mostrar no cartafol</button>
  </div>
</section>

<style>
  section { padding: 1rem 0; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 0.6rem; }
  h3 { margin: 0; font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-muted); font-weight: 600; }
  dl { display: grid; grid-template-columns: auto 1fr; gap: 0.25rem 0.8rem; margin: 0; font-size: 0.8rem; }
  dt { color: var(--text-muted); }
  dd { margin: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .hash { font-family: Consolas, monospace; font-size: 0.7rem; }
  .acts { display: flex; gap: 0.4rem; flex-wrap: wrap; }
  .tiny { margin: 0; font-size: 0.78rem; }
</style>
