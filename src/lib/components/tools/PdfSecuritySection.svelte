<script lang="ts">
  import { onMount } from "svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Toggle from "../ui/Toggle.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let file: FileEntry;

  let encrypted: boolean | null = null;
  let mode: "protect" | "unprotect" = "protect";
  let password = "";
  let owner = "";
  let allowPrint = true;
  let allowCopy = true;
  const job = createJob();
  const busy = job.busy;

  onMount(async () => {
    try {
      encrypted = await engine.pdfIsEncrypted(file.path);
      if (encrypted) mode = "unprotect";
    } catch {
      encrypted = null;
    }
  });

  async function run() {
    try {
      const suffix = { protect: "-protexido", unprotect: "-desbloqueado" }[mode];
      const out = await askOutput(file, tr(suffix), "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const done =
          mode === "protect"
            ? await engine.pdfProtect(file.path, out, { userPassword: password, ownerPassword: owner, allowPrint, allowCopy }, true)
            : await engine.pdfUnprotect(file.path, out, password, true);
        return `${listo()} ${fileNameOf(done)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <Segmented
    options={[
      { value: "protect", label: $t("Protexer") },
      { value: "unprotect", label: $t("Desbloquear") },
    ]}
    bind:value={mode}
  />

  <div class="group">
    {#if mode === "protect"}
      <label class="pw">
        <span class="lab">{$t("Contrasinal para abrilo")}</span>
        <input type="password" bind:value={password} autocomplete="off" />
      </label>
      <label class="pw">
        <span class="lab">{$t("Contrasinal de propietario (opcional)")}</span>
        <input type="password" bind:value={owner} autocomplete="off" />
      </label>
      <Toggle label={$t("Permitir imprimir")} bind:checked={allowPrint} />
      <Toggle label={$t("Permitir copiar texto")} bind:checked={allowCopy} />
      <p class="group-note">{$t("Gárdase unha copia cifrada; o PDF orixinal queda como estaba. Se perdes o contrasinal non hai maneira de recuperalo.")}</p>
    {:else}
      {#if encrypted === false}<p class="group-note">{$t("Este PDF non ten contrasinal.")}</p>{/if}
      <label class="pw">
        <span class="lab">{$t("Contrasinal actual")}</span>
        <input type="password" bind:value={password} autocomplete="off" />
      </label>
      <p class="group-note">{$t("Hai que saber o contrasinal: o trasno quita o bloqueo, non o rompe.")}</p>
    {/if}
  </div>

  <button class="btn primary block" on:click={run} disabled={$busy || (!password && !(mode === "unprotect" && encrypted === false))}>
    {mode === "protect" ? $t("Protexer o PDF") : $t("Desbloquear o PDF")}
  </button>
  <JobStatus {job} />
</div>

<style>
  .pw { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { display: inline-flex; align-items: center; font-size: 0.82rem; color: var(--text-muted); }
  input[type="password"] { height: var(--control-h); background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0 0.65em; font-family: inherit; font-size: 0.9rem; }
  input[type="password"]:focus { outline: none; border-color: var(--accent); }
</style>
