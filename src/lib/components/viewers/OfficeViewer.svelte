<script lang="ts">
  import { onMount } from "svelte";
  import type { FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import { t } from "../../i18n";
  import { openHub } from "../../settings";
  import Icon from "../Icon.svelte";
  import InfoTip from "../ui/InfoTip.svelte";
  import PdfViewer from "./PdfViewer.svelte";

  export let file: FileEntry;

  // Un documento de Office vese como PDF: o trasno pídelle a Microsoft Office ou a LibreOffice unha copia en PDF (só para
  // velo aquí; o ficheiro orixinal non cambia) e amósaa co visor de PDF.
  let state: "loading" | "ready" | "missing" | "failed" = "loading";
  let pdf: FileEntry | null = null;
  let failMsg = "";

  async function load() {
    state = "loading";
    failMsg = "";
    try {
      const targets = await engine.officeTargets(file.ext);
      if (!targets.includes("pdf")) {
        state = "missing";
        return;
      }
      const path = await engine.makePreview(file.path);
      pdf = { ...file, path, ext: "pdf", category: "documento", pages: undefined, size: undefined };
      state = "ready";
    } catch (e) {
      failMsg = errText(e);
      state = "failed";
    }
  }
  onMount(load);
</script>

<div class="wrap">
  {#if state === "loading"}
    <div class="msg" role="status">
      <p class="muted">{$t("O trasno está preparando a vista previa de {ext}…", { ext: file.ext.toUpperCase() })}</p>
      <div class="indet" aria-hidden="true"><span></span></div>
    </div>
  {:else if state === "missing"}
    <div class="msg">
      <p>{$t("Para ver este documento fai falta Microsoft Office ou LibreOffice, e o trasno non atopa ningún dos dous neste equipo.")}</p>
      <button class="btn primary" on:click={() => openHub("settings.tools")}>{$t("Ver as extensións")}</button>
    </div>
  {:else if state === "failed"}
    <div class="msg">
      <p class="muted">{$t("O trasno non puido preparar a vista previa.")}</p>
      {#if failMsg}<p class="muted tiny selectable">{failMsg}</p>{/if}
      <button class="btn" on:click={load}><Icon name="reset" size={14} /> {$t("Tentar de novo")}</button>
    </div>
  {:else if pdf}
    <div class="pdf"><PdfViewer file={pdf} /></div>
    <p class="note" role="note">{$t("Vista previa convertida a PDF só para velo aquí. O ficheiro orixinal non cambia.")}<InfoTip text={$t("O trasno convirte o documento a PDF con Microsoft Office ou LibreOffice para poder amosalo. A maquetación pode variar un pouco da que se ve no programa orixinal.")} /></p>
  {/if}
</div>

<style>
  .wrap { height: 100%; display: flex; flex-direction: column; min-height: 0; background: var(--viewer-bg); }
  .pdf { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .pdf > :global(*) { flex: 1 1 auto; min-height: 0; }
  .msg { margin: auto; display: flex; flex-direction: column; align-items: center; gap: 0.7rem; max-width: 34rem; padding: 1.5rem; text-align: center; }
  .msg p { margin: 0; }
  .tiny { font-size: var(--fs-sm); }
  .note { display: flex; align-items: center; gap: 0.4rem; margin: 0; padding: 0.4rem 0.9rem 0.4rem max(0.9rem, var(--dock-gap, 0px)); font-size: var(--fs-xs); color: var(--text-muted); background: var(--bg-rail); border-top: 1px solid var(--border); user-select: text; }
  .indet { width: min(280px, 70%); height: 5px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; position: relative; }
  .indet span { position: absolute; inset: 0 auto 0 0; width: 40%; border-radius: 99px; background: var(--accent); animation: vai 1.2s ease-in-out infinite alternate; }
  @keyframes vai { from { transform: translateX(0); } to { transform: translateX(150%); } }
</style>
