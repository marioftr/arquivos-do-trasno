<script lang="ts">
  import GroupNote from "../ui/GroupNote.svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { Category } from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { samePath, askPath } from "./shared";
  import OrderList from "../ui/OrderList.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let selection: FileEntry[];
  export let category: Category;

  const first = selection[0];

  // A orde de partida é a da lista; aquí pódese cambiar sen mexer na lista de ficheiros.
  let items: FileEntry[] = [...selection];
  const isPdf = category === "documento";
  const job = createJob();
  const busy = job.busy;

  $: title = isPdf ? $t("Unir PDF") : category === "imaxe" ? $t("Imaxes a PDF") : category === "video" ? $t("Xuntar vídeos") : $t("Xuntar audios");

  async function merge() {
    const asPdf = isPdf || category === "imaxe";
    const out = await askPath(await join(first.path.replace(/[\\/][^\\/]*$/, ""), `${tr("unido")}.${asPdf ? "pdf" : first.ext}`));
    if (!out) return;
    const inputs = items.map((s) => s.path);
    if (inputs.some((p) => samePath(p, out))) {
      job.say(false, tr("Non vou sobrescribir un dos ficheiros de orixe. Escolle outro nome."));
      return;
    }
    await job.run(TRASNO.unindo, fallo("unindo"), async () => {
      const done = isPdf
        ? await engine.pdfMerge(inputs, out, true)
        : category === "imaxe"
          ? await engine.imagesToPdf(inputs, out, true)
          : await engine.concatMedia(inputs, out, true);
      return `${listo()} ${fileNameOf(done)}`;
    });
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{title}</h4>
    <GroupNote text={$t("Segue a orde da lista: cámbiase arrastrando, coas frechas ou escribindo a posición.")} />
    <OrderList bind:items />
  </div>
  <button class="btn primary block" on:click={merge} disabled={$busy}>
    {isPdf ? $t("Unir nun só PDF") : category === "imaxe" ? $t("Facer un PDF cunha imaxe por páxina") : $t("Xuntar nun só ficheiro")}
  </button>
  <JobStatus {job} />
</div>
