<script lang="ts">
  import type { Category } from "../engine";
  import type { FileEntry } from "../files";
  import { t } from "../i18n";
  import { formatBytes, CATEGORY_LABELS, PLAIN_TEXT } from "../formats";
  import ToolStrip from "./ui/ToolStrip.svelte";
  import FormBar from "./ui/FormBar.svelte";
  import { activeForm } from "../forms";
  import { canUndo, canRedo, shapes, undo, redo, clearAll } from "../annot";
  import { derived } from "svelte/store";
  const hasShapes = derived(shapes, (l) => l.length > 0);
  import ConvertSection from "./tools/ConvertSection.svelte";
  import CompressSection from "./tools/CompressSection.svelte";
  import ImageEditSection from "./tools/ImageEditSection.svelte";
  import MediaEditSection from "./tools/MediaEditSection.svelte";
  import ExtractSection from "./tools/ExtractSection.svelte";
  import AudioEffectsSection from "./tools/AudioEffectsSection.svelte";
  import TrimSection from "./tools/TrimSection.svelte";
  import PdfSplitSection from "./tools/PdfSplitSection.svelte";
  import PdfMarksSection from "./tools/PdfMarksSection.svelte";
  import PdfCropSection from "./tools/PdfCropSection.svelte";
  import PdfCompressSection from "./tools/PdfCompressSection.svelte";
  import PdfSecuritySection from "./tools/PdfSecuritySection.svelte";
  import PdfRepairSection from "./tools/PdfRepairSection.svelte";
  import Segmented from "./ui/Segmented.svelte";
  import Lazy from "./ui/Lazy.svelte";
  import MkvTracksSection from "./tools/MkvTracksSection.svelte";
  import Icon from "./Icon.svelte";
  import { tools, openSettings, toolActions, jump } from "../settings";
  import { requestTool } from "../toolinstall";
  import { ocrActive } from "../ocr";
  import MergeSection from "./tools/MergeSection.svelte";
  import InfoSection from "./tools/InfoSection.svelte";

  export let selection: FileEntry[] = [];
  let body: HTMLDivElement;

  $: multi = selection.length > 1;
  $: first = selection[0] as FileEntry | undefined;
  $: category = (selection.length && selection.every((f) => f.category === first!.category)
    ? first!.category
    : null) as Category | null;
  $: isPdf = category === "documento" && selection.every((f) => f.ext === "pdf");
  $: selKey = selection.map((f) => f.path).join("|");

  /** Cada tipo de ficheiro (ou lote) ten as súas pestañas: unha tarefa por pestana. */
  function tabsFor(cat: Category | null, many: boolean, pdf: boolean, ext: string, tt: (k: string) => string): { id: string; label: string }[] {
    const tab = (id: string, label: string) => ({ id, label: tt(label) });
    // Os contedores Matroska teñen a súa propia pestana de pistas (MKVToolNix).
    const mkvTab = () => (["mkv", "mka", "webm"].includes(ext) ? [tab("mkv", "Pistas")] : []);
    const multiMkv = () => [tab("mkv", "Pistas")];
    if (!cat) return [];
    if (many) {
      if (pdf) return [tab("merge", "Unir")];
      if (cat === "imaxe") return [tab("convert", "Converter"), tab("compress", "Comprimir"), tab("merge", "Unir")];
      if (cat === "video") return [tab("convert", "Converter"), tab("merge", "Unir"), ...multiMkv()];
      if (cat === "audio") return [tab("convert", "Converter"), tab("merge", "Unir"), ...multiMkv()];
      return [tab("convert", "Converter")];
    }
    switch (cat) {
      case "imaxe":
        return [tab("convert", "Converter"), tab("compress", "Comprimir"), tab("edit", "Editar"), tab("annotate", "Anotar"), tab("info", "Información")];
      case "video":
        return [tab("convert", "Converter"), tab("edit", "Editar"), ...mkvTab(), tab("extract", "Extraer"), tab("info", "Información")];
      case "audio":
        return [tab("convert", "Converter"), tab("edit", "Editar"), ...mkvTab(), tab("info", "Información")];
      case "documento":
        return pdf
          ? [
              tab("convert", "Converter"),
              tab("pages", "Páxinas"),
              tab("shape", "Xirar e recortar"),
              tab("marks", "Marcas"),
              tab("optimize", "Comprimir e reparar"),
              tab("security", "Seguridade"),
              tab("info", "Información"),
            ]
          : PLAIN_TEXT.includes(first?.ext ?? "")
            ? [tab("convert", "Converter"), tab("info", "Información")]
            : [tab("info", "Información")];
      default:
        return [tab("convert", "Converter"), tab("info", "Información")];
    }
  }

  $: tabs = tabsFor(category, multi, isPdf, first?.ext ?? "", $t);
  let current = "convert";
  $: if (tabs.length && !tabs.some((x) => x.id === current)) current = tabs[0].id;

  // Lembra a última pestana usada con cada tipo de ficheiro (e co lote).
  const TAB_KEY = "arquivos-do-trasno.tabs";
  let tabMem: Record<string, string> = {};
  try {
    tabMem = JSON.parse(localStorage.getItem(TAB_KEY) ?? "{}");
  } catch {
    tabMem = {};
  }
  $: memKey = category ? `${category}${multi ? "+" : ""}` : "";
  let lastKey = "";
  $: if (memKey !== lastKey) {
    lastKey = memKey;
    const saved = memKey ? tabMem[memKey] : "";
    if (saved && tabs.some((x) => x.id === saved)) current = saved;
  }
  $: if (memKey && memKey === lastKey && tabs.some((x) => x.id === current) && tabMem[memKey] !== current) {
    tabMem[memKey] = current;
    try {
      localStorage.setItem(TAB_KEY, JSON.stringify(tabMem));
    } catch {
      // ignorar
    }
  }

  /** Algunhas pestanas agrupan ferramentas parecidas: un selector enriba escolle cal. */
  function modesFor(tab: string, pdf: boolean, many: boolean, cat: Category | null, tt: (k: string) => string): { value: string; label: string }[] {
    const m = (value: string, label: string) => ({ value, label: tt(label) });
    // Un vídeo (ou varios) pódese converter a MP4 co perfil que máis lle conveña (comprimindo) ou a outro formato.
    if (tab === "convert" && cat === "video" && !pdf) return [m("mp4", "A MP4 con perfís"), m("format", "A outro formato")];
    if (many) return [];
    if (!pdf) {
      if (tab === "edit" && cat === "video") return [m("trim", "Cortar"), m("tune", "Velocidade e volume")];
      if (tab === "edit" && cat === "audio") return [m("trim", "Cortar"), m("tune", "Velocidade e volume"), m("effects", "Efectos")];
      return [];
    }
    switch (tab) {
      case "convert": return [m("images", "A imaxes"), m("format", "A outro formato")];
      case "pages": return [m("organize", "Ordenar"), m("extract", "Extraer"), m("split", "Dividir")];
      case "shape": return [m("rotate", "Xirar"), m("crop", "Recortar")];
      case "optimize": return [m("compress", "Comprimir"), m("repair", "Reparar")];
      default: return [];
    }
  }
  const sub: Record<string, string> = {};
  $: modes = modesFor(current, isPdf, multi, category, $t);
  $: if (modes.length && !modes.some((x) => x.value === sub[current])) sub[current] = modes[0].value;
  $: mode = modes.length ? sub[current] : "";

  // Ctrl+K: lista das ferramentas dispoñibles e salto directo a unha delas.
  $: toolActions.set(
    tabs.flatMap((tb) => {
      const ms = modesFor(tb.id, isPdf, multi, category, $t);
      return ms.length ? ms.map((m) => ({ id: `${tb.id}:${m.value}`, label: `${tb.label} · ${m.label}`, tab: tb.id, mode: m.value })) : [{ id: tb.id, label: tb.label, tab: tb.id }];
    }),
  );
  $: if ($jump && tabs.some((x) => x.id === $jump!.tab)) {
    current = $jump.tab;
    if ($jump.mode) sub[current] = $jump.mode;
    jump.set(null);
  }

  // Sen os programas externos, as ferramentas que dependen deles quedan bloqueadas (e explícase por que).
  $: locked = !!$tools && (current === "mkv" ? !$tools.mkvmerge : (category === "audio" || category === "video") && !$tools.ffmpeg);
</script>

<div class="tools">
  {#if !selection.length}
    <p class="muted hint">{$t("Abre un ficheiro para ver as ferramentas que o trasno ten para el.")}</p>
  {:else if !category}
    <p class="muted hint">
      {$t("Hai ficheiros de tipos distintos marcados. Marca só do mesmo tipo (por exemplo, varios PDF ou varias imaxes) para traballar con eles xuntos.")}
    </p>
  {:else}
    <div class="shell">
    <ToolStrip {tabs} bind:value={current} on:again={() => body?.scrollTo({ top: 0, behavior: "smooth" })} />
    <div class="main">
    <header>
      <div class="ttl">
        <h2>{tabs.find((x) => x.id === current)?.label ?? ""}</h2>
        {#if multi}<p class="muted sub">{$t("{n} ficheiros", { n: selection.length })}&nbsp;·&nbsp;{$t(CATEGORY_LABELS[category])}</p>{/if}
      </div>
      {#if current === "annotate" && category === "imaxe"}
        <FormBar custom={{ canUndo, canRedo, dirty: hasShapes, undo, redo, reset: clearAll, resetTitle: "Borrar todas as anotacións" }} />
      {:else}
        <FormBar form={$activeForm} />
      {/if}
    </header>

    {#if $ocrActive && category === "imaxe" && (current === "edit" || current === "annotate")}
      <p class="ocrnote">{$t("Desactiva «Texto da imaxe» para usar estas ferramentas.")}</p>
    {/if}
    <div class="body scrollbar-thin" class:off={$ocrActive && category === "imaxe" && (current === "edit" || current === "annotate")} inert={$ocrActive && category === "imaxe" && (current === "edit" || current === "annotate")} bind:this={body}>
      {#if modes.length}
        <div class="modes"><Segmented options={modes} bind:value={sub[current]} /></div>
      {/if}
      {#key `${selKey}|${current}|${mode}`}
        {#if locked}
          <div class="locked">
            <Icon name="warning" size={20} />
            <p><strong>{$t("Esta ferramenta está bloqueada")}</strong></p>
            <p class="muted">
              {current === "mkv"
                ? $t("Editar as pistas dun MKV precisa MKVToolNix, que non está instalado.")
                : $t("O audio e o vídeo precisan FFmpeg, que non está instalado.")}
            </p>
            <button class="btn primary" on:click={() => requestTool(current === "mkv" ? "mkvmerge" : "ffmpeg")}>{$t("Instalar agora")}</button>
          </div>
        {:else if current === "mkv" && first}
          <MkvTracksSection {selection} />
        {:else if current === "convert" && mode === "images" && first}
          <Lazy loader={() => import("./tools/PdfImagesSection.svelte")} file={first} />
        {:else if current === "convert" && category === "video" && mode === "mp4"}
          <CompressSection {selection} category="video" />
        {:else if current === "convert"}
          <ConvertSection {selection} {category} />
        {:else if current === "compress" && category === "imaxe"}
          <CompressSection {selection} {category} />
        {:else if current === "edit" && mode === "trim" && first}
          <TrimSection file={first} />
        {:else if current === "edit" && mode === "effects" && first}
          <AudioEffectsSection file={first} />
        {:else if current === "annotate" && first && category === "imaxe"}
          <Lazy loader={() => import("./tools/AnnotateSection.svelte")} file={first} />
        {:else if current === "edit" && first && category === "imaxe"}
          <ImageEditSection file={first} />
        {:else if current === "edit" && first}
          <MediaEditSection file={first} />
        {:else if current === "extract" && first}
          <ExtractSection file={first} />
        {:else if current === "pages" && mode === "extract" && first}
          <Lazy loader={() => import("./tools/PdfPagesSection.svelte")} file={first} part="extract" />
        {:else if current === "pages" && mode === "split" && first}
          <PdfSplitSection file={first} />
        {:else if current === "pages" && first}
          <Lazy loader={() => import("./tools/PdfOrganizeSection.svelte")} file={first} />
        {:else if current === "shape" && mode === "crop" && first}
          <PdfCropSection file={first} />
        {:else if current === "shape" && first}
          <Lazy loader={() => import("./tools/PdfPagesSection.svelte")} file={first} part="rotate" />
        {:else if current === "marks" && first}
          <PdfMarksSection file={first} />
        {:else if current === "optimize" && mode === "repair" && first}
          <PdfRepairSection file={first} />
        {:else if current === "optimize" && first}
          <PdfCompressSection file={first} />
        {:else if current === "security" && first}
          <PdfSecuritySection file={first} />
        {:else if current === "merge"}
          <MergeSection {selection} {category} />
        {:else if current === "info" && first}
          <InfoSection file={first} />
        {/if}
      {/key}
    </div>
    </div>
    </div>
  {/if}
</div>

<style>
  .tools { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .shell { flex: 1; min-height: 0; display: flex; }
  .main { flex: 1; min-width: 0; min-height: 0; display: flex; flex-direction: column; }
  @media (max-width: 1050px) { .shell { flex-direction: column; } }
  header { display: flex; align-items: center; gap: 0.5rem; padding: 0.55rem 0.8rem 0.45rem 1.1rem; min-height: 3rem; }
  .ttl { flex: 1; min-width: 0; }
  header h2 { margin: 0; font-size: 0.95rem; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sub { margin: 0.1rem 0 0; font-size: 0.76rem; }
  .ocrnote { margin: 0; padding: 0.5rem 1.1rem; font-size: 0.8rem; color: var(--accent-pdf); border-bottom: 1px solid var(--border); }
  .body.off { opacity: 0.45; }
  .body { flex: 1; min-height: 0; overflow-y: auto; padding: 0.9rem 1.1rem 1.5rem; }
  .locked { display: flex; flex-direction: column; align-items: center; gap: 0.6rem; text-align: center; padding: 2rem 1rem; color: var(--accent-pdf); }
  .locked p { margin: 0; color: var(--text); }
  .locked .muted { color: var(--text-muted); }
  .modes { margin-bottom: 0.9rem; }
  .hint { padding: 1.2rem 1.1rem; font-size: 0.9rem; }
</style>
