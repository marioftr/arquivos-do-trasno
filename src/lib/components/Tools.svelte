<script lang="ts">
  import type { Category } from "../engine";
  import type { FileEntry } from "../files";
  import { t } from "../i18n";
  import { formatBytes, CATEGORY_LABELS, PLAIN_TEXT } from "../formats";
  import Tabs from "./ui/Tabs.svelte";
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
  import { tools, openSettings } from "../settings";
  import MergeSection from "./tools/MergeSection.svelte";
  import InfoSection from "./tools/InfoSection.svelte";

  export let selection: FileEntry[] = [];

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
      if (cat === "video") return [tab("convert", "Converter"), tab("compress", "Comprimir"), tab("merge", "Unir"), ...multiMkv()];
      if (cat === "audio") return [tab("convert", "Converter"), tab("merge", "Unir"), ...multiMkv()];
      return [tab("convert", "Converter")];
    }
    switch (cat) {
      case "imaxe":
        return [tab("convert", "Converter"), tab("compress", "Comprimir"), tab("edit", "Editar"), tab("info", "Información")];
      case "video":
        return [tab("convert", "Converter"), tab("compress", "Comprimir"), tab("edit", "Editar"), ...mkvTab(), tab("extract", "Extraer"), tab("info", "Información")];
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
    if (many) return [];
    const m = (value: string, label: string) => ({ value, label: tt(label) });
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
    <header>
      <h2 title={multi ? "" : first?.name}>{multi ? $t("{n} ficheiros", { n: selection.length }) : first?.name}</h2>
      <p class="muted sub">
        {$t(CATEGORY_LABELS[category])}{#if !multi && first?.size !== undefined} · {formatBytes(first.size)}{/if}{#if !multi && first?.pages} · {$t("{n} páx.", { n: first.pages })}{/if}
      </p>
    </header>

    <Tabs {tabs} bind:value={current} />

    <div class="body scrollbar-thin">
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
            <button class="btn primary" on:click={() => openSettings("tools")}>{$t("Ver como instalalo")}</button>
          </div>
        {:else if current === "mkv" && first}
          <MkvTracksSection {selection} />
        {:else if current === "convert" && mode === "images" && first}
          <Lazy loader={() => import("./tools/PdfImagesSection.svelte")} file={first} />
        {:else if current === "convert"}
          <ConvertSection {selection} {category} />
        {:else if current === "compress" && (category === "imaxe" || category === "video")}
          <CompressSection {selection} {category} />
        {:else if current === "edit" && mode === "trim" && first}
          <TrimSection file={first} />
        {:else if current === "edit" && mode === "effects" && first}
          <AudioEffectsSection file={first} />
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
  {/if}
</div>

<style>
  .tools { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  header { padding: 1rem 1.1rem 0.7rem; }
  header h2 { margin: 0; font-size: 1.02rem; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sub { margin: 0.15rem 0 0; font-size: 0.8rem; }
  .body { flex: 1; min-height: 0; overflow-y: auto; padding: 0.9rem 1.1rem 1.5rem; }
  .locked { display: flex; flex-direction: column; align-items: center; gap: 0.6rem; text-align: center; padding: 2rem 1rem; color: var(--accent-pdf); }
  .locked p { margin: 0; color: var(--text); }
  .locked .muted { color: var(--text-muted); }
  .modes { margin-bottom: 0.9rem; }
  .hint { padding: 1.2rem 1.1rem; font-size: 0.9rem; }
</style>
