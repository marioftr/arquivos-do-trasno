<script lang="ts">
  import { tick } from "svelte";
  import { errText } from "../../errors";
  import { viewUrl, type FileEntry } from "../../files";
  import { tr } from "../../i18n";
  import FindBar from "../ui/FindBar.svelte";
  import { textCache } from "../../viewCache";
  import { actionFor } from "../../keymap";

  export let file: FileEntry;

  let text = "";
  $: filePath = file.path;
  $: load(filePath);

  async function load(path: string) {
    const hit = textCache.get(path);
    if (hit !== undefined) {
      text = hit;
      return;
    }
    try {
      const res = await fetch(await viewUrl(path));
      text = (await res.text()).slice(0, 200_000);
      textCache.set(path, text);
    } catch (e) {
      text = tr("Non puiden ler o ficheiro: {error}", { error: errText(e) });
    }
  }

  // ---- buscar (Ctrl+F)
  const MAX_HITS = 3000;
  let find = false;
  let query = "";
  let cur = 1; // 1..hits.length
  let box: HTMLPreElement;

  $: hits = (() => {
    const q = query.toLowerCase();
    if (!q) return [] as number[];
    const low = text.toLowerCase();
    const out: number[] = [];
    for (let i = low.indexOf(q); i !== -1 && out.length < MAX_HITS; i = low.indexOf(q, i + q.length)) out.push(i);
    return out;
  })();
  $: if (cur > hits.length) cur = hits.length ? 1 : 0;
  $: if (hits.length && cur === 0) cur = 1;
  $: parts = (() => {
    if (!hits.length) return [{ t: text, hit: -1 }];
    const out: { t: string; hit: number }[] = [];
    let at = 0;
    hits.forEach((h, k) => {
      if (h > at) out.push({ t: text.slice(at, h), hit: -1 });
      out.push({ t: text.slice(h, h + query.length), hit: k + 1 });
      at = h + query.length;
    });
    if (at < text.length) out.push({ t: text.slice(at), hit: -1 });
    return out;
  })();
  $: if (hits.length) tick().then(() => box?.querySelector("mark.cur")?.scrollIntoView({ block: "center" }));

  const step = (d: number) => {
    if (hits.length) cur = ((cur - 1 + d + hits.length) % hits.length) + 1;
  };

  function key(e: KeyboardEvent) {
    if (actionFor(e, "pdf") === "p.find") {
      e.preventDefault();
      find = true;
    }
  }
</script>

<svelte:window on:keydown={key} />

<div class="wrap">
  {#if find}
    <FindBar bind:query count={hits.length} current={cur} on:next={() => step(1)} on:prev={() => step(-1)} on:close={() => ((find = false), (query = ""))} />
  {/if}
  <pre class="selectable scrollbar-thin" bind:this={box}>{#each parts as p}{#if p.hit > 0}<mark class:cur={p.hit === cur}>{p.t}</mark>{:else}{p.t}{/if}{/each}</pre>
</div>

<style>
  .wrap { position: relative; height: 100%; }
  pre { margin: 0; height: 100%; overflow: auto; padding: 1rem 1.25rem; font-size: 0.85rem; white-space: pre-wrap; }
  mark { background: color-mix(in srgb, var(--accent-pdf) 45%, transparent); color: inherit; border-radius: 2px; }
  mark.cur { background: var(--accent-pdf); color: #fff; }
</style>
