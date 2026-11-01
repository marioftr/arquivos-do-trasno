<script lang="ts">
  import type { FileEntry } from "../files";
  import { t } from "../i18n";

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

{#key file.path}
  {#if kind && Comp && shownFor === kind}
    <svelte:component this={Comp} {file} />
  {:else if !kind}
    <p class="muted msg">{$t("O trasno non coñece este tipo de ficheiro.")}</p>
  {/if}
{/key}

<style>
  .msg { padding: 2rem; }
</style>
