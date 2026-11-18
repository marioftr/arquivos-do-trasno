<script lang="ts">
  import { onMount } from "svelte";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { viewUrl, type FileEntry } from "../../files";
  import { t } from "../../i18n";
  import TagsEditor from "./TagsEditor.svelte";

  /** Etiquetas dunha canción: le o ficheiro e, cando as ten, amosa o formulario. */
  export let file: FileEntry;

  let info: engine.TagInfo | null = null;
  let cover = "";
  let err = "";
  onMount(async () => {
    try {
      const r = await engine.tagsRead(file.path);
      if (r.cover) cover = await viewUrl(await engine.tagsCover(file.path));
      info = r;
    } catch (e) {
      err = errText(e);
    }
  });
</script>

{#if err}
  <div class="stack"><p class="warn" role="alert">{err}</p></div>
{:else if !info}
  <div class="stack"><p class="muted small">{$t("Lendo as etiquetas…")}</p></div>
{:else}
  <TagsEditor {file} {info} originalCover={cover} />
{/if}

<style>
  .small { margin: 0; font-size: var(--fs-sm); }
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); }
</style>
