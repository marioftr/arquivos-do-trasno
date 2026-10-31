<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { Category } from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { samePath } from "./shared";
  import Icon from "../Icon.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let selection: FileEntry[];
  export let category: Category;

  const first = selection[0];
  const isPdf = category === "documento";
  const job = createJob();
  const busy = job.busy;

  $: title = isPdf ? $t("Unir PDF") : category === "imaxe" ? $t("Imaxes a PDF") : category === "video" ? $t("Xuntar vídeos") : $t("Xuntar audios");

  async function merge() {
    const asPdf = isPdf || category === "imaxe";
    const out = await save({
      defaultPath: await join(first.path.replace(/[\\/][^\\/]*$/, ""), `${tr("unido")}.${asPdf ? "pdf" : first.ext}`),
    });
    if (!out) return;
    const inputs = selection.map((s) => s.path);
    if (inputs.some((p) => samePath(p, out))) {
      job.say(false, tr("Non se sobrescribe un dos ficheiros de orixe. Escolle outro nome."));
      return;
    }
    await job.run(TRASNO.unindo, TRASNO.tropezouUnindo, async () => {
      const done = isPdf
        ? await engine.pdfMerge(inputs, out, true)
        : category === "imaxe"
          ? await engine.imagesToPdf(inputs, out, true)
          : await engine.concatMedia(inputs, out, true);
      return `${tr(TRASNO.listo)} ${fileNameOf(done)}`;
    });
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{title}</h4>
    <p class="group-note">{$t("Vaise facer na orde da lista; usa as frechas para reordenar.")}</p>
    <ol class="order selectable">
      {#each selection as f, i}
        <li><span class="n">{i + 1}</span><span class="nm" title={f.path}>{f.name}</span></li>
      {/each}
    </ol>
  </div>
  <button class="btn primary block" on:click={merge} disabled={$busy}>
    <Icon name="layers" size={14} />
    {isPdf ? $t("Unir nun só PDF") : category === "imaxe" ? $t("Facer un PDF cunha imaxe por páxina") : $t("Xuntar nun só ficheiro")}
  </button>
  <JobStatus {job} />
</div>

<style>
  .order { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; font-size: 0.85rem; }
  .order li { display: flex; align-items: center; gap: 0.7rem; padding: 0.4rem 0; border-bottom: 1px solid var(--border); }
  .n { color: var(--text-faint); font-variant-numeric: tabular-nums; min-width: 1.4em; text-align: right; }
  .nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
