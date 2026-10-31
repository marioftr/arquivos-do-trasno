<script lang="ts">
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, AUDIO_TARGETS } from "../../formats";
  import { askOutput } from "./shared";
  import { t, tr } from "../../i18n";

  export let file: FileEntry;

  $: isVideo = file.category === "video";

  let frameAt = 0;
  let speed = 1;
  let volume = 1;
  let height = 720;
  let audioExt = "mp3";
  let audioKbps = 192;

  let busy = false;
  let message: { ok: boolean; text: string } | null = null;

  $: file.path, (message = null);

  async function run(suffix: string, ext: string, job: (out: string) => Promise<string>) {
    message = null;
    try {
      const out = await askOutput(file, tr(suffix), ext);
      if (!out) return;
      busy = true;
      message = { ok: true, text: `${tr(TRASNO.listo)} ${fileNameOf(await job(out))}` };
    } catch (e) {
      message = { ok: false, text: `${tr(TRASNO.tropezouEditando)}\n${e}` };
    } finally {
      busy = false;
    }
  }

  const frame = () => run("-fotograma", "png", (o) => engine.mediaEdit(file.path, o, "frame", frameAt, true, true));
  const faster = () => run("-velocidade", file.ext, (o) => engine.mediaEdit(file.path, o, "speed", speed, isVideo, true));
  const louder = () => run("-volume", file.ext, (o) => engine.mediaEdit(file.path, o, "volume", volume, isVideo, true));
  const mute = () => run("-mudo", file.ext, (o) => engine.mediaEdit(file.path, o, "mute", 0, true, true));
  const scale = () => run(`-${height}p`, file.ext, (o) => engine.mediaEdit(file.path, o, "scale", height, true, true));
  const audio = () =>
    run("", audioExt, async (o) => (await engine.convertFile(file.path, o, { overwrite: true, bitrateKbps: audioKbps })).output_path);
</script>

<section>
  <h3>{isVideo ? $t("Editar vídeo") : $t("Editar audio")}</h3>

  {#if isVideo}
    <div class="row">
      <span>{$t("Fotograma")}</span>
      <input class="num-input wide" type="number" min="0" step="0.1" bind:value={frameAt} />
      <span class="u">s</span>
      <button class="btn grow" on:click={frame} disabled={busy}>{$t("Gardar imaxe")}</button>
    </div>
  {/if}

  <div class="row">
    <span>{$t("Velocidade")}</span>
    <input type="range" min="0.25" max="4" step="0.05" bind:value={speed} />
    <input class="num-input" type="number" min="0.25" max="4" step="0.05" bind:value={speed} />
  </div>
  <button class="btn" on:click={faster} disabled={busy || speed === 1}>{$t("Aplicar velocidade ×{v}", { v: speed })}</button>

  <div class="row">
    <span>{$t("Volume")}</span>
    <input type="range" min="0" max="4" step="0.05" bind:value={volume} />
    <input class="num-input" type="number" min="0" max="4" step="0.05" bind:value={volume} />
  </div>
  <button class="btn" on:click={louder} disabled={busy || volume === 1}>{$t("Aplicar volume ×{v}", { v: volume })}</button>

  {#if isVideo}
    <div class="row">
      <span>{$t("Altura")}</span>
      <input class="num-input wide" type="number" min="16" step="2" bind:value={height} />
      <span class="u">px</span>
      <button class="btn grow" on:click={scale} disabled={busy}>{$t("Reducir vídeo")}</button>
    </div>
    <button class="btn" on:click={mute} disabled={busy}>{$t("Quitar o son")}</button>

    <div class="row">
      <span>{$t("Extraer audio")}</span>
      <select bind:value={audioExt}>{#each AUDIO_TARGETS as t}<option value={t}>{t.toUpperCase()}</option>{/each}</select>
      <input class="num-input wide" type="number" min="32" max="320" bind:value={audioKbps} title="kbps" />
      <button class="btn" on:click={audio} disabled={busy}>{$t("Extraer")}</button>
    </div>
  {/if}

  {#if busy}<p class="muted msg">{$t(TRASNO.traballando)}</p>{/if}
  {#if message}<p class="msg" class:ok={message.ok} class:erro={!message.ok}>{message.text}</p>{/if}
</section>

<style>
  section { padding: 1rem 0; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 0.6rem; }
  h3 { margin: 0; font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-muted); font-weight: 600; }
  .row { display: flex; align-items: center; gap: 0.5rem; font-size: 0.86rem; }
  .row > span:first-child { min-width: 5.2rem; color: var(--text-muted); }
  .row input[type="range"] { flex: 1; min-width: 0; }
  .row select { background: var(--bg-elevated); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0.4em 0.5em; }
  .num-input.wide { width: 5em; }
  .u { color: var(--text-muted); }
  .grow { flex: 1; justify-content: center; }
  .msg { margin: 0; font-size: 0.85rem; white-space: pre-wrap; word-break: break-word; }
  .msg.ok { color: var(--accent-imaxe); }
  .msg.erro { color: var(--accent-video); }
</style>
