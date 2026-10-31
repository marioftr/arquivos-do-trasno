<script lang="ts">
  import { viewUrl, type FileEntry } from "../../files";
  import { tr } from "../../i18n";

  export let file: FileEntry;

  let text = "";
  $: load(file.path);

  async function load(path: string) {
    try {
      const res = await fetch(await viewUrl(path));
      text = (await res.text()).slice(0, 200_000);
    } catch (e) {
      text = tr("Non se puido ler o ficheiro: {error}", { error: String(e) });
    }
  }
</script>

<pre class="selectable scrollbar-thin">{text}</pre>

<style>
  pre { margin: 0; height: 100%; overflow: auto; padding: 1rem 1.25rem; font-size: 0.85rem; white-space: pre-wrap; }
</style>
