<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { errText } from "../../errors";
  import { imageUrl, type FileEntry } from "../../files";
  import { t, tr, lang as appLang } from "../../i18n";
  import { noteAction, trasnoSay } from "../../agent";
  import { toast } from "../../toasts";
  import { OCR_LANGS, OCR_SIZE, ocrVerdict, ocrInstalled, ocrDownload, ocrProgress, ocrActive, ocrShown, savedLang, rememberLang, refreshOcr, downloadLang, recognize, ocrPeek, detectLang, type OcrResult } from "../../ocr";
  import SelectField from "../ui/SelectField.svelte";
  import GroupNote from "../ui/GroupNote.svelte";

  export let file: FileEntry;

  let lang = savedLang($appLang);
  let busy = false;
  let res: OcrResult | null = null;
  let err = "";
  let downloadErr = "";

  $: installed = $ocrInstalled.includes(lang);
  $: langName = OCR_LANGS.find((l) => l.id === lang)?.label ?? lang;
  $: words = res?.lines.length ? res.text.split(/\s+/).filter(Boolean).length : 0;

  onMount(() => {
    ocrActive.set(true);
    void refreshOcr().catch(() => {});
    // Se esta imaxe xa se leu (ou se está lendo), o texto volve sen empezar de novo.
    const cached = ocrPeek(file.path, lang);
    if (cached && cached !== "busy") show(cached);
    else if (cached === "busy") void read();
  });
  onDestroy(() => {
    ocrActive.set(false);
    ocrShown.set(null);
  });

  function show(r: OcrResult | null) {
    res = r;
    ocrShown.set(r);
  }

  async function read() {
    err = "";
    const path = file.path;
    const l = lang;
    busy = true;
    try {
      const r = await recognize(path, get(imageUrl), l);
      if (file.path === path && lang === l) {
        show(r);
        // O trasno conta o que atopou (ou que non atopou nada) e queda no historial.
        const v = ocrVerdict(r, file.name);
        noteAction(v.note);
        trasnoSay(v.say, "info", 9000, undefined, { copy: v.found ? r.text : undefined, notify: !v.found });
        offerLang(path, l, r);
      }
    } catch (e) {
      err = errText(e);
    } finally {
      busy = false;
    }
  }

  // Se o texto lido parece doutro idioma, o trasno ofrécese a lelo de novo nel (descargando o paquete se fai falta).
  const offered = new Set<string>();
  function offerLang(path: string, cur: string, r: OcrResult) {
    const g = r.lines.length ? detectLang(r.text) : null;
    if (!g || g === cur || offered.has(`${path}|${g}`)) return;
    offered.add(`${path}|${g}`);
    const name = OCR_LANGS.find((x) => x.id === g)?.label.toLowerCase();
    const curName = OCR_LANGS.find((x) => x.id === cur)?.label.toLowerCase() ?? cur;
    if (!name) return;
    const have = get(ocrInstalled).includes(g);
    trasnoSay(
      have
        ? tr("Este texto parece estar en {lang}, pero o lin como {cur}. Queres que o lea de novo en {lang}?", { lang: name, cur: curName })
        : tr("Este texto parece estar en {lang}. Queres que descargue o paquete de idioma (uns {size}) e o lea de novo?", { lang: name, size: OCR_SIZE }),
      "info",
      30000,
      {
        label: have ? tr("Ler en {lang}", { lang: name }) : tr("Descargar e ler"),
        run: async () => {
          if (file.path !== path) return;
          lang = g;
          rememberLang(g);
          show(null);
          if (!have) await download();
          if (get(ocrInstalled).includes(g)) await read();
        },
      },
    );
  }

  async function download() {
    downloadErr = "";
    try {
      await downloadLang(lang);
    } catch (e) {
      downloadErr = errText(e);
    }
  }

  function changeLang() {
    rememberLang(lang);
    // Outro idioma: o texto de antes xa non vale. Se esa lectura xa estaba feita, volve ao momento.
    const cached = ocrPeek(file.path, lang);
    show(cached && cached !== "busy" ? cached : null);
    err = "";
  }

  async function copyAll() {
    if (!res) return;
    try {
      await navigator.clipboard.writeText(res.text);
      toast(tr("Metín o texto no portapapeis."));
    } catch {
      toast(tr("O portapapeis púxose teimudo co texto. Tenta de novo."), { kind: "warn" });
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Texto da imaxe")}</h4>
    <GroupNote text={$t("Le o texto da imaxe para seleccionalo e copialo sobre o visor.")} />

    <SelectField
      label={$t("Idioma do texto")}
      bind:value={lang}
      options={OCR_LANGS.map((l) => ({ value: l.id, label: l.label + ($ocrInstalled.includes(l.id) ? "" : " ⇩") }))}
      disabled={busy}
      on:change={changeLang}
    />
    {#if !installed}
      <GroupNote text={$t("Para ler {lang} hai que descargar o seu paquete de idioma (uns {size}, de Tesseract en GitHub).", { lang: langName, size: OCR_SIZE })} />
      <button class="btn block" on:click={download} disabled={busy || !!$ocrDownload}>
        {$ocrDownload ? $t("Descargando… {p} %", { p: Math.round($ocrDownload.progress * 100) }) : $t("Descargar o paquete de idioma")}
      </button>
      {#if downloadErr}<p class="warn">{downloadErr}</p>{/if}
    {/if}
  </div>

  <button class="btn primary block" on:click={read} disabled={busy || !installed || !!$ocrDownload}>
    {busy ? `${$t("Lendo o texto…")} ${Math.round($ocrProgress * 100)} %` : res ? $t("Ler de novo") : $t("Ler o texto da imaxe")}
  </button>

  {#if err}
    <p class="warn">{err}</p>
  {:else if res}
    <div class="group">
      <h4 class="group-title">{$t("Resultado")}</h4>
      {#if words}
        <p class="note">{$t("{n} palabras. Selecciona o texto co rato sobre a imaxe ou cópiao todo.", { n: words })}</p>
        <button class="btn block" on:click={copyAll}>{$t("Copiar todo o texto")}</button>
      {:else}
        <p class="note">{$t("Non atopei texto. Proba con outro idioma ou cunha imaxe máis nítida.")}</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .note { margin: 0; font-size: var(--fs-sm); color: var(--text-muted); user-select: text; }
  .warn { margin: 0; color: var(--accent-pdf); font-size: var(--fs-sm); user-select: text; }
</style>
