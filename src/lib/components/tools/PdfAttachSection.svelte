<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, formatBytes, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Icon from "../Icon.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Anexos dun PDF: os que xa leva (pódense quitar) e os que se queren engadir. Viaxan dentro do ficheiro e saen co lector. */
  export let file: FileEntry;

  const job = createJob();
  const busy = job.busy;
  let current: engine.Attachment[] = [];
  let removed = new Set<string>();
  let added: { path: string; name: string; size: number }[] = [];
  let loaded = false;
  onDestroy(() => job.clear());

  onMount(async () => {
    try {
      current = await engine.pdfListAttachments(file.path);
    } catch {
      current = [];
    }
    loaded = true;
  });

  async function pick() {
    const picked = await open({ multiple: true });
    const list = Array.isArray(picked) ? picked : typeof picked === "string" ? [picked] : [];
    const fresh = await Promise.all(
      list
        .filter((p) => !added.some((a) => a.path === p))
        .map(async (p) => ({ path: p, name: fileNameOf(p), size: await engine.fileSize(p).catch(() => 0) })),
    );
    added = [...added, ...fresh];
  }
  const toggleRemoved = (name: string) => {
    const next = new Set(removed);
    next.has(name) ? next.delete(name) : next.add(name);
    removed = next;
  };

  $: changes = added.length + removed.size;

  async function save() {
    try {
      const out = await askOutput(file, tr("-con-anexos"), "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.pdfEditAttachments(file.path, out, added.map((a) => a.path), [...removed], true);
        current = await engine.pdfListAttachments(r.output_path).catch(() => current);
        added = [];
        removed = new Set();
        return `${listo()} ${fileNameOf(r.output_path)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <PanelInfo text={$t("Ficheiros que viaxan dentro do PDF e se poden sacar co lector. Cada un pode pesar ata 200 MB.")} />
    {#if current.length || added.length}
      <ul class="results">
        {#each current as a}
          <li class:erro={removed.has(a.name)}>
            <span class="rname" title={a.name}>{a.name}</span>
            <span class="rstate">
              {formatBytes(a.size)}
              <button class="btn ghost icon" on:click={() => toggleRemoved(a.name)} title={removed.has(a.name) ? $t("Non quitar") : $t("Quitar do PDF")} aria-label={`${removed.has(a.name) ? $t("Non quitar") : $t("Quitar do PDF")}: ${a.name}`}><Icon name={removed.has(a.name) ? "undo" : "trash"} size={14} /></button>
            </span>
          </li>
        {/each}
        {#each added as a}
          <li class="feito">
            <span class="rname" title={a.path}>{a.name}</span>
            <span class="rstate">
              {formatBytes(a.size)}
              <button class="btn ghost icon" on:click={() => (added = added.filter((x) => x !== a))} title={$t("Non engadir")} aria-label={`${$t("Non engadir")}: ${a.name}`}><Icon name="close" size={14} /></button>
            </span>
          </li>
        {/each}
      </ul>
    {:else if loaded}
      <p class="muted">{$t("Este PDF non ten anexos.")}</p>
    {/if}
    <button class="btn block" on:click={pick} disabled={$busy}><Icon name="plus" size={14} /> {$t("Engadir ficheiros")}</button>
  </div>

  <button class="btn primary block" on:click={save} disabled={$busy || !changes}>{$t("Gardar copia cos anexos")}</button>
  <JobStatus {job} />
</div>

<style>
  li.erro .rname { text-decoration: line-through; opacity: 0.6; }
  .rstate { display: inline-flex; align-items: center; gap: 0.3rem; }
</style>
