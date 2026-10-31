<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import BrandMark from "./lib/components/BrandMark.svelte";
  import Icon from "./lib/components/Icon.svelte";
  import FileList from "./lib/components/FileList.svelte";
  import Viewer from "./lib/components/Viewer.svelte";
  import Tools from "./lib/components/Tools.svelte";
  import { files, activePath, checked, addPaths, removeFile } from "./lib/files";
  import * as engine from "./lib/engine";

  let dragging = false;
  let unlisten: (() => void) | null = null;
  let ffmpegOk: boolean | null = null;

  engine.ffmpegAvailable().then((v) => (ffmpegOk = v));

  async function escoller() {
    const selected = await open({ multiple: true });
    if (!selected) return;
    await addPaths(Array.isArray(selected) ? selected : [selected]);
  }

  onMount(async () => {
    engine.startupFiles().then((paths) => { if (paths.length) addPaths(paths); });
    unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === "drop") {
        dragging = false;
        addPaths(p.paths);
      } else if (p.type === "enter" || p.type === "over") {
        dragging = true;
      } else {
        dragging = false;
      }
    });
  });

  function onKey(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    if (t.closest("input, select, textarea")) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "o") {
      e.preventDefault();
      escoller();
    } else if (e.key === "Delete" && $activePath) {
      removeFile($activePath);
    }
  }

  onDestroy(() => unlisten?.());

  $: active = $files.find((f) => f.path === $activePath) ?? null;
  // Con dous ou máis ficheiros marcados, as ferramentas traballan co lote;
  // se non, co ficheiro que se está vendo.
  $: batch = $files.filter((f) => $checked.has(f.path));
  $: selection = batch.length > 1 ? batch : active ? [active] : [];
  $: needsFfmpeg = selection.some((f) => f.category === "audio" || f.category === "video");
</script>

<svelte:window on:keydown={onKey} />

<div class="app" class:dragging>
  <header class="top">
    <span class="brand"><BrandMark size={20} /> Arquivos do Trasno</span>
    <span class="grow"></span>
    {#if ffmpegOk === false && needsFfmpeg}
      <span class="aviso"><Icon name="warning" size={14} /> Falta ffmpeg (instálao con «winget install ffmpeg» ou pon ffmpeg.exe e ffprobe.exe ao carón da app).</span>
    {/if}
  </header>

  {#if $files.length === 0}
    <main class="home">
      <div class="drop">
        <BrandMark size={44} />
        <h1>O trasno agarda os teus ficheiros</h1>
        <p class="muted">
          Arrastra aquí o que sexa: imaxes, audio, vídeo, PDF, texto ou modelos 3D. Podes soltar varios á vez para unilos,
          comprimilos ou convertelos en lote.
        </p>
        <button class="btn primary" on:click={escoller}><Icon name="folder" size={16} /> Escoller ficheiros</button>
        <p class="muted formatos">
          PNG · JPG · WEBP · GIF · MP3 · WAV · FLAC · MP4 · MKV · WEBM · PDF · TXT · FBX · GLB · OBJ · STL · PLY
        </p>
      </div>
    </main>
  {:else}
    <main class="work">
      <aside class="left"><FileList onAdd={escoller} /></aside>
      <section class="center">
        {#if active}<Viewer file={active} />{/if}
      </section>
      <aside class="right"><Tools {selection} /></aside>
    </main>
  {/if}

  {#if dragging}
    <div class="veil"><span>Solta os ficheiros para que o trasno os colla</span></div>
  {/if}
</div>

<style>
  .app { position: relative; display: flex; flex-direction: column; width: 100%; height: 100vh; }
  .top { display: flex; align-items: center; gap: 0.8rem; padding: 0.55rem 1rem; border-bottom: 1px solid var(--border); background: var(--bg-rail); min-height: 46px; }
  .brand { display: inline-flex; align-items: center; gap: 0.55rem; font-weight: 650; font-size: 0.92rem; }
  .grow { flex: 1; }
  .aviso { display: inline-flex; align-items: center; gap: 0.4rem; color: var(--accent-pdf); font-size: 0.82rem; }

  .home { flex: 1; min-height: 0; display: flex; padding: 1.5rem; }
  .drop {
    flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 1rem;
    text-align: center; padding: 2rem; border: 1px dashed var(--border-strong); border-radius: var(--radius-md);
  }
  .drop h1 { margin: 0.4rem 0 0; font-size: 1.35rem; font-weight: 600; }
  .drop p { margin: 0; max-width: 54ch; }
  .formatos { font-size: 0.75rem; letter-spacing: 0.04em; margin-top: 0.8rem !important; }

  .work { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(210px, 270px) minmax(0, 1fr) minmax(270px, 330px); }
  .left, .right { min-height: 0; min-width: 0; background: var(--bg-rail); }
  .left { border-right: 1px solid var(--border); }
  .right { border-left: 1px solid var(--border); }
  .center { min-width: 0; min-height: 0; overflow: hidden; }

  @media (max-width: 1050px) {
    .work { grid-template-columns: minmax(190px, 230px) minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) minmax(0, 42%); }
    .right { grid-column: 1 / -1; border-left: none; border-top: 1px solid var(--border); }
  }

  .veil {
    position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
    background: color-mix(in srgb, var(--bg) 82%, transparent); border: 2px dashed var(--accent-conversor);
    color: var(--text); font-size: 1.05rem; pointer-events: none;
  }
</style>
