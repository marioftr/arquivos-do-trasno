<script lang="ts">
  import type { FileEntry } from "../files";
  import { t, tr } from "../i18n";
  import { removeFile, reimportFile, reloadTick } from "../files";
  import { revealInFolder } from "../engine";
  import { formatBytes } from "../formats";
  import Icon from "./Icon.svelte";

  export let file: FileEntry;

  // Cada visor carga só cando se necesita: three.js e PDF.js non pesan ao arrancar.
  const loaders: Record<string, () => Promise<{ default: any }>> = {
    imaxe: () => import("./viewers/ImageViewer.svelte"),
    media: () => import("./viewers/MediaViewer.svelte"),
    pdf: () => import("./viewers/PdfViewer.svelte"),
    html: () => import("./viewers/HtmlViewer.svelte"),
    text: () => import("./viewers/TextViewer.svelte"),
    model: () => import("./viewers/ModelViewer.svelte"),
  };

  function kindOf(f: FileEntry): string | null {
    if (f.category === "imaxe") return "imaxe";
    if (f.category === "audio" || f.category === "video") return "media";
    if (f.category === "documento" && f.ext === "pdf") return "pdf";
    if (f.category === "documento" && (f.ext === "html" || f.ext === "htm" || f.ext === "svg")) return "html";
    if (f.category === "documento") return "text";
    if (f.category === "modelo3d") return "model";
    return null;
  }

  const ICONS: Record<string, string> = { imaxe: "image", audio: "audio", video: "video", documento: "document", modelo3d: "cube" };
  $: meta = [
    file.size !== undefined ? formatBytes(file.size) : "",
    file.width ? `${file.width} × ${file.height}` : "",
    file.pages ? tr("{n} páx.", { n: file.pages }) : "",
  ]
    .filter(Boolean)
    .join(" · ");

  let Comp: any = null;
  let shownFor = "";
  $: kind = kindOf(file);
  $: if (kind) {
    const k = kind;
    loaders[k]().then((m) => {
      if (kindOf(file) === k) {
        Comp = m.default;
        shownFor = k;
      }
    });
  } else {
    Comp = null;
  }
</script>

<div class="doc">
  <div class="docbar">
    <span class="kind" aria-hidden="true"><Icon name={ICONS[file.category] ?? "document"} size={16} /></span>
    <span class="chip" title={file.path}>{file.ext.toUpperCase()}</span>
    <span class="meta muted">{meta}</span>
    <span class="grow"></span>
    <button class="ib" on:click={() => reimportFile(file.path)} title={$t("Actualizar: volver importar o ficheiro e deixalo todo como ao principio")} aria-label={$t("Actualizar o ficheiro")}><Icon name="reset" size={15} /></button>
    <button class="ib" on:click={() => revealInFolder(file.path).catch(() => {})} title={$t("Amosar no cartafol")} aria-label={$t("Amosar no cartafol")}><Icon name="folder" size={15} /></button>
    <button class="ib" on:click={() => removeFile(file.path)} title={$t("Quitar da lista")} aria-label={$t("Quitar da lista")}><Icon name="close" size={15} /></button>
  </div>
  <div class="vbody">
    {#key `${file.path}|${$reloadTick}`}
      {#if kind && Comp && shownFor === kind}
        <svelte:component this={Comp} {file} />
      {:else if !kind}
        <p class="muted msg">{$t("O trasno non coñece este tipo de ficheiro.")}</p>
      {/if}
    {/key}
  </div>
</div>

<style>
  .doc { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .vbody { flex: 1; min-height: 0; min-width: 0; display: flex; flex-direction: column; position: relative; }
  .vbody > :global(*) { flex: 1 1 auto; min-height: 0; }
  .docbar { flex: none; display: flex; align-items: center; gap: 0.55rem; min-height: 2.6rem; padding: 0 0.6rem 0 0.9rem; background: var(--bg-rail); border-bottom: 1px solid var(--border); font-size: 0.84rem; }
  .kind { display: inline-flex; color: var(--text-muted); }
  .chip { flex: none; padding: 0.05rem 0.45rem; border-radius: 99px; background: var(--bg-hover); color: var(--text-muted); font-size: 0.68rem; font-weight: 650; letter-spacing: 0.04em; }
  .meta { white-space: nowrap; font-size: 0.78rem; }
  .grow { flex: 1; }
  .ib { display: grid; place-items: center; width: 1.9rem; height: 1.9rem; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-muted); }
  .ib:hover { background: var(--bg-hover); color: var(--text); }
  .msg { padding: 2rem; }
</style>
