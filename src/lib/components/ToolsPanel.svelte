<script lang="ts">
  import { onMount } from "svelte";
  import { openLink, cancelToolInstall, type ToolId } from "../engine";
  import { t } from "../i18n";
  import { tools, refreshTools } from "../settings";
  import { ocrInstalled, refreshOcr, OCR_LANGS } from "../ocr";
  import { installing, installError, installToolNow, removeToolNow, overall, phaseText, TOOL_INFO } from "../toolinstall";
  import Icon from "./Icon.svelte";
  import { toast } from "../toasts";
  import { tr } from "../i18n";

  let checking = false;
  async function recheck() {
    checking = true;
    const before = $tools;
    try {
      const s = await refreshTools();
      const found = [!before?.ffmpeg && s.ffmpeg ? "FFmpeg" : "", !before?.mkvmerge && s.mkvmerge ? "MKVToolNix" : ""].filter(Boolean);
      if (found.length) toast(tr("O trasno atopou {list}. Xa non fai falta descargalo.", { list: found.join(" e ") }));
      else if (s.ffmpeg && s.mkvmerge) toast(tr("O trasno revisou o almacén: ten todas as ferramentas."));
      else toast(tr("O que falta non aparece por ningunha parte. Se o acabas de instalar, pecha e abre a aplicación unha vez."), { kind: "warn", ms: 8000 });
    } catch {
      toast(tr("O trasno non puido revisar o almacén. Tenta de novo."), { kind: "warn" });
    } finally {
      checking = false;
    }
  }

  onMount(() => {
    refreshOcr().catch(() => {});
    refreshTools();
  });

  const keys: ToolId[] = ["ffmpeg", "mkvmerge"];
  // O que vai dentro da aplicación: non hai que instalar nada.
  const builtin: { name: string; what: string; license: string }[] = [
    { name: "Motor de imaxes", what: "Le e escribe PNG, JPG, WebP, GIF, BMP, ICO, TIFF, PNM, TGA e QOI.", license: "image-rs · MIT / Apache-2.0" },
    { name: "PDF.js", what: "Visor, miniaturas e busca de texto nos PDF.", license: "Mozilla · Apache-2.0" },
    { name: "lopdf", what: "Cortar, unir, ordenar, protexer e reparar PDF.", license: "MIT" },
    { name: "three.js", what: "Visor 3D e conversión de FBX e glTF.", license: "MIT" },
    { name: "Tesseract.js", what: "Recoñecemento de texto en imaxes (OCR). Cada idioma é un paquete que se descarga só se o pides.", license: "Apache-2.0" },
  ];
  $: ocrNames = $ocrInstalled.map((id) => OCR_LANGS.find((l) => l.id === id)?.label ?? id);
  // Antes de descargar pídese a aceptación expresa do usuario.
  let confirming: ToolId | null = null;
  async function accept(k: ToolId) {
    confirming = null;
    await installToolNow(k);
  }
</script>

<div class="panel">
  <p class="muted lead">
    {$t("Arquivos do Trasno traballa só coas imaxes, os PDF e o 3D. Para audio, vídeo e MKV axúdase de dous programas gratuítos que non van incluídos: o trasno pode descargalos por ti ou podes instalalos ti e a app detectaos soa.")}
  </p>

  <section class="card ok">
    <div class="head"><strong>{$t("Incluído coa aplicación")}</strong><span class="state ok"><Icon name="check" size={13} /> {$t("Sempre dispoñible")}</span></div>
    {#each builtin as b}
      <p><strong>{b.name}</strong> · <span class="muted">{$t(b.what)}</span> <span class="muted small">({b.license})</span></p>
    {/each}
    <p class="muted small">{$t("Idiomas de OCR descargados:")} {ocrNames.length ? ocrNames.join(", ") : $t("ningún")}</p>
  </section>

  {#each keys as key}
    {@const it = TOOL_INFO[key]}
    {@const ok = $tools?.[key]}
    {@const busy = $installing?.tool === key ? $installing : null}
    <section class="card" class:ok>
      <div class="head">
        <strong>{it.name}</strong>
        <span class="state" class:ok>
          <Icon name={ok ? "check" : "warning"} size={13} />
          {$tools === null ? $t("Comprobando…") : ok ? $t("Instalado") : $t("Non atopado")}
        </span>
      </div>
      <p class="muted">{$t(it.why)}</p>
      <p><span class="muted">{$t("Desbloquea:")}</span> {$t(it.unlocks)}</p>
      {#if busy}
        {@const pct = Math.round(overall(busy) * 100)}
        <div class="prog">
          <div class="line"><span>{phaseText(busy.phase)}</span><span class="muted pct">{pct} %</span></div>
          <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct}><div class="fill" style={`width:${pct}%`}></div></div>
          <div class="acts"><button class="btn" on:click={() => cancelToolInstall()}>{$t("Cancelar descarga")}</button></div>
        </div>
      {:else if confirming === key}
        <div class="ask">
          <p>{$t("O trasno descargará {name} (uns {size}) de {source}, comprobará que o ficheiro está íntegro e deixarao na súa carpeta de datos. Non fai falta ser administrador.", { name: it.name, size: it.size, source: it.source })}</p>
          <div class="acts">
            <button class="btn primary" on:click={() => accept(key)}>{$t("Aceptar e descargar")}</button>
            <button class="btn" on:click={() => (confirming = null)}>{$t("Agora non")}</button>
          </div>
        </div>
      {:else if !ok}
        <div class="acts">
          <button class="btn primary" on:click={() => (confirming = key)} disabled={!!$installing}>{$t("Descargar e instalar")} <span class="muted">· {it.size}</span></button>
          <button class="btn ghost" on:click={() => openLink(it.link)}>{$t("Abrir a páxina de descarga")}</button>
        </div>
        <p class="muted small">{$t("ou, en Windows, no terminal:")} <code class="selectable">{it.winget}</code></p>
      {:else if $tools?.[`${key}_own`]}
        <div class="acts">
          <span class="muted small">{$t("Descargado polo trasno.")}</span>
          <button class="btn ghost" on:click={() => removeToolNow(key)} disabled={!!$installing}>{$t("Quitar")}</button>
        </div>
      {/if}
      {#if $installError?.tool === key}<p class="fail selectable">{$installError.text}</p>{/if}
    </section>
  {/each}

  <div class="foot">
    <button class="btn" on:click={recheck} disabled={checking}><Icon name="reset" size={14} /> {checking ? $t("Comprobando…") : $t("Volver comprobar")}</button>
    <span class="muted">{$t("Tamén se aceptan ao carón do executable da app (ffmpeg.exe, ffprobe.exe, mkvmerge.exe).")}</span>
  </div>
</div>

<style>
  .panel { display: flex; flex-direction: column; gap: 1rem; }
  .lead { margin: 0; }
  .card { border: 1px solid var(--border); border-radius: var(--radius-md); padding: 0.85rem 1rem; display: flex; flex-direction: column; gap: 0.5rem; background: var(--bg-card); }
  .card p { margin: 0; }
  .head { display: flex; align-items: center; justify-content: space-between; }
  .state { display: inline-flex; align-items: center; gap: 0.3rem; font-size: 0.8rem; color: var(--accent-pdf); }
  .state.ok { color: var(--accent-imaxe); }
  .small { font-size: 0.8rem; }
  .fail { color: var(--accent-video); font-size: 0.84rem; }
  .prog { display: flex; flex-direction: column; gap: 0.4rem; }
  .line { display: flex; justify-content: space-between; font-size: 0.84rem; }
  .pct { font-variant-numeric: tabular-nums; }
  .bar { height: 7px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); border-radius: 99px; transition: width 0.2s ease; }
  .ask p { margin: 0 0 0.5rem; }
  .acts { display: flex; flex-wrap: wrap; align-items: center; gap: 0.7rem; }
  code { background: var(--bg-input); border: 1px solid var(--border); border-radius: 4px; padding: 1px 6px; font-size: 0.82rem; }
  .foot { display: flex; flex-wrap: wrap; align-items: center; gap: 0.8rem; font-size: 0.82rem; }
</style>
