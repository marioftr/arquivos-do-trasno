<script lang="ts">
  import GroupNote from "../ui/GroupNote.svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  const job = createJob();
  const busy = job.busy;

  async function run() {
    try {
      const out = await askOutput(file, tr("-reparado"), "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => `${listo()} ${fileNameOf(await engine.pdfRepair(file.path, out, true))}`);
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <GroupNote text={$t("Gárdase de novo o PDF limpo, reconstruíndo as súas referencias e quitando o que sobra. Axuda cos PDF que se abren mal.")} />
  </div>
  <button class="btn primary block" on:click={run} disabled={$busy}>{$t("Reparar o PDF")}</button>
  <JobStatus {job} />
</div>
