<script lang="ts">
  import type { FileEntry } from "../files";
  import { t } from "../i18n";
  import ImageViewer from "./viewers/ImageViewer.svelte";
  import MediaViewer from "./viewers/MediaViewer.svelte";
  import PdfViewer from "./viewers/PdfViewer.svelte";
  import TextViewer from "./viewers/TextViewer.svelte";
  import ModelViewer from "./viewers/ModelViewer.svelte";

  export let file: FileEntry;
</script>

{#key file.path}
  {#if file.category === "imaxe"}
    <ImageViewer {file} />
  {:else if file.category === "audio" || file.category === "video"}
    <MediaViewer {file} />
  {:else if file.category === "documento" && file.ext === "pdf"}
    <PdfViewer {file} />
  {:else if file.category === "documento"}
    <TextViewer {file} />
  {:else if file.category === "modelo3d"}
    <ModelViewer {file} />
  {:else}
    <p class="muted msg">{$t("O trasno non coñece este tipo de ficheiro.")}</p>
  {/if}
{/key}

<style>
  .msg { padding: 2rem; }
</style>
