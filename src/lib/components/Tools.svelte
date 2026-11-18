<script lang="ts">
  import type { Category } from "../engine";
  import type { FileEntry } from "../files";
  import { t } from "../i18n";
  import { formatBytes, CATEGORY_LABELS, PLAIN_TEXT, OFFICE_EXTS, SUBTITLE_EXTS } from "../formats";
  import ToolStrip from "./ui/ToolStrip.svelte";
  import FormBar from "./ui/FormBar.svelte";
  import { activeForm } from "../forms";
  import { canUndo, canRedo, shapes, undo, redo, clearAll, resetAnnoStyle } from "../annot";
  import { derived } from "svelte/store";
  import { viewCtx, where } from "../agent";
  const hasShapes = derived(shapes, (l) => l.length > 0);
  // Restablecer en Anotar: fóra as anotacións e o aspecto volve ao por defecto.
  const resetAnno = () => {
    clearAll();
    resetAnnoStyle();
  };
  import ConvertSection from "./tools/ConvertSection.svelte";
  import CompressSection from "./tools/CompressSection.svelte";
  import ImageEditSection from "./tools/ImageEditSection.svelte";
  import ImageOcrSection from "./tools/ImageOcrSection.svelte";
  import EditMediaSection from "./tools/EditMediaSection.svelte";
  import ExtractSection from "./tools/ExtractSection.svelte";
  import RestoreSection from "./tools/RestoreSection.svelte";
  import ResizeSection from "./tools/ResizeSection.svelte";
  import StitchSection from "./tools/StitchSection.svelte";
  import SliceSection from "./tools/SliceSection.svelte";
  import TagsSection from "./tools/TagsSection.svelte";
  import SilenceSection from "./tools/SilenceSection.svelte";
  import ModelViewSection from "./tools/ModelViewSection.svelte";
  import VideoSubtitlesSection from "./tools/VideoSubtitlesSection.svelte";
  import SubtitleSection from "./tools/SubtitleSection.svelte";
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
      if (cat === "imaxe") return [tab("convert", "Converter"), tab("compress", "Comprimir"), tab("resize", "Redimensionar"), tab("merge", "Unir"), tab("stitch", "Xuntar capturas")];
      if (cat === "video") return [tab("convert", "Converter"), tab("merge", "Unir"), ...multiMkv()];
      if (cat === "audio") return [tab("convert", "Converter"), tab("merge", "Unir"), ...multiMkv()];
      return [tab("convert", "Converter")];
    }
    switch (cat) {
      case "imaxe":
        return [tab("convert", "Converter"), tab("compress", "Comprimir"), tab("edit", "Editar"), tab("resize", "Redimensionar"), tab("slice", "Cortar en partes"), tab("annotate", "Anotar"), tab("ocr", "Recoñecer texto (OCR)"), tab("info", "Información")];
      case "video":
        return [tab("convert", "Converter"), tab("edit", "Editar"), tab("sound", "Son"), ...(ext === "gif" ? [] : [tab("restore", "Restaurar")]), tab("subtitles", "Subtítulos"), ...mkvTab(), tab("extract", "Extraer"), tab("info", "Información")];
      case "audio":
        return [tab("convert", "Converter"), tab("edit", "Editar"), tab("sound", "Son"), tab("tags", "Etiquetas"), ...mkvTab(), tab("info", "Información")];
      case "documento":
        return pdf
          ? [
              tab("convert", "Converter"),
              tab("pages", "Páxinas"),
              tab("shape", "Xirar e recortar"),
              tab("marks", "Marcas"),
              tab("optimize", "Comprimir e reparar"),
              tab("ocr", "Recoñecer texto (OCR)"),
              tab("security", "Seguridade"),
              tab("info", "Información"),
            ]
          : SUBTITLE_EXTS.includes(first?.ext ?? "")
            ? [tab("convert", "Converter"), tab("edit", "Resincronizar"), tab("info", "Información")]
            : PLAIN_TEXT.includes(first?.ext ?? "") || OFFICE_EXTS.includes(first?.ext ?? "")
            ? [tab("convert", "Converter"), tab("info", "Información")]
            : [tab("info", "Información")];
      case "modelo3d":
        return [tab("convert", "Converter"), tab("view", "Escena"), tab("info", "Información")];
      default:
        return [tab("convert", "Converter"), tab("info", "Información")];
    }
  }

  $: tabs = tabsFor(category, multi, isPdf, first?.ext ?? "", $t);
  let current = "convert";
  let lastTab = "";
  $: if (current && current !== lastTab) {
    lastTab = current;
  }
  $: viewCtx.set({ category: category ?? undefined, tool: current, ext: first?.ext, count: selection.length });
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
      if (tab === "subtitles" && cat === "video") return [m("embed", "Incrustar"), m("burn", "Queimar"), m("extract", "Extraer")];
      // O son ten dúas tarefas: axustalo (volume, ruído, efectos) ou cortar os silencios.
      if (tab === "sound" && (cat === "video" || cat === "audio")) return [m("adjust", "Axustes"), m("silence", "Silencios")];
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
  // O historial conta cada cambio co ficheiro e o menú onde se fixo.
  $: where.set({
    file: multi ? $t("{n} ficheiros", { n: selection.length }) : (first?.name ?? ""),
    menu: [tabs.find((x) => x.id === current)?.label, modes.find((x) => x.value === mode)?.label].filter(Boolean).join(" · "),
  });

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
    <p class="muted hint">{$t("Abre un ficheiro e ensínoche as ferramentas que teño para el.")}</p>
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
        <FormBar custom={{ canUndo, canRedo, dirty: hasShapes, undo, redo, reset: resetAnno }} />
      {:else}
        <FormBar form={$activeForm} />
      {/if}
    </header>

    <div class="body scrollbar-thin" bind:this={body}>
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
        {:else if current === "resize" && category === "imaxe"}
          <ResizeSection {selection} />
        {:else if current === "stitch" && category === "imaxe"}
          <StitchSection {selection} />
        {:else if current === "slice" && category === "imaxe" && first}
          <SliceSection file={first} />
        {:else if current === "compress" && category === "imaxe"}
          <CompressSection {selection} {category} />
        {:else if current === "edit" && first && SUBTITLE_EXTS.includes(first.ext)}
          <SubtitleSection file={first} />
        {:else if current === "subtitles" && first && category === "video"}
          <VideoSubtitlesSection file={first} mode={mode as "embed" | "burn" | "extract"} />
        {:else if current === "view" && first && category === "modelo3d"}
          <ModelViewSection file={first} />
        {:else if current === "restore" && first && category === "video"}
          <RestoreSection file={first} />
        {:else if current === "tags" && first && category === "audio"}
          <TagsSection file={first} />
        {:else if current === "sound" && mode === "silence" && first}
          <SilenceSection file={first} />
        {:else if current === "sound" && first}
          <EditMediaSection file={first} part="sound" />
        {:else if current === "annotate" && first && category === "imaxe"}
          <Lazy loader={() => import("./tools/AnnotateSection.svelte")} file={first} />
        {:else if current === "edit" && first && category === "imaxe"}
          <ImageEditSection file={first} />
        {:else if current === "edit" && first}
          <EditMediaSection file={first} />
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
        {:else if current === "ocr" && first && category === "imaxe"}
          <ImageOcrSection file={first} />
        {:else if current === "ocr" && first}
          <Lazy loader={() => import("./tools/PdfOcrSection.svelte")} file={first} />
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
  header h2 { margin: 0; font-size: var(--fs-lg); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sub { margin: 0.1rem 0 0; font-size: var(--fs-xs); }
  .body { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; overscroll-behavior-x: none; padding: 0.9rem 1.1rem 1.5rem; }
  .locked { display: flex; flex-direction: column; align-items: center; gap: 0.6rem; text-align: center; padding: 2rem 1rem; color: var(--accent-pdf); }
  .locked p { margin: 0; color: var(--text); }
  .locked .muted { color: var(--text-muted); }
  .modes { margin-bottom: 0.9rem; }
  .hint { padding: 1.2rem 1.1rem; font-size: var(--fs-md); }
</style>
