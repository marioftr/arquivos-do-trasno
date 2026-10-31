<script lang="ts">
  import { errText } from "../../errors";
  import { viewUrl, type FileEntry } from "../../files";
  import { t, tr } from "../../i18n";
  import Segmented from "../ui/Segmented.svelte";

  export let file: FileEntry;

  let source = "";
  let mode: "view" | "code" = "view";
  let error = "";

  $: load(file.path);

  async function load(path: string) {
    error = "";
    try {
      const res = await fetch(await viewUrl(path));
      source = (await res.text()).slice(0, 2_000_000);
    } catch (e) {
      error = tr("Non puiden ler o ficheiro: {error}", { error: errText(e) });
    }
  }

  // O HTML móstrase nun marco illado: sen scripts, sen formularios e sen saír do visor.
  $: doc = file.ext === "svg" ? `<!doctype html><body style="margin:0;display:flex;align-items:center;justify-content:center;min-height:100vh;background:#fff">${source}</body>` : source;
</script>

<div class="wrap">
  <div class="top">
    <Segmented options={[{ value: "view", label: $t("Vista") }, { value: "code", label: $t("Código") }]} bind:value={mode} />
    <p class="muted note">{$t("Móstrase sen scripts, por seguridade.")}</p>
  </div>
  {#if error}
    <p class="muted msg">{error}</p>
  {:else if mode === "view"}
    <iframe title={file.name} sandbox="" srcdoc={doc}></iframe>
  {:else}
    <pre class="selectable scrollbar-thin">{source}</pre>
  {/if}
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .top { display: flex; align-items: center; gap: 1rem; padding: 0.6rem 0.9rem; border-bottom: 1px solid var(--border); }
  .top :global(.seg) { width: 200px; }
  .note { margin: 0; font-size: 0.8rem; }
  iframe { flex: 1; min-height: 0; border: none; background: #fff; width: 100%; }
  pre { flex: 1; margin: 0; overflow: auto; padding: 1rem 1.25rem; font-size: 0.82rem; white-space: pre-wrap; word-break: break-word; font-family: Consolas, monospace; }
  .msg { padding: 2rem; }
</style>
