<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { imageUrl, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import { annoActive, annoTool, annoStyle, shapes, selected, canUndo, canRedo, undo, redo, clearAll, resetAnnotations, renderAnnotated, FONTS, type Tool } from "../../annot";
  import { setAnnoPath, annoPath } from "../../annot";
  import Slider from "../ui/Slider.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import ColorField from "../ui/ColorField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import Icon from "../Icon.svelte";

  export let file: FileEntry;

  const job = createJob();
  const busy = job.busy;
  let ext = file.ext === "jpg" || file.ext === "webp" ? file.ext : "png";

  onMount(() => {
    if (get(annoPath) !== file.path) {
      resetAnnotations();
      setAnnoPath(file.path);
    }
    annoActive.set(true);
  });
  onDestroy(() => {
    annoActive.set(false);
    selected.set(null);
  });
  // Outra imaxe: o debuxado empeza de cero.
  $: if (get(annoPath) !== file.path) {
    resetAnnotations();
    setAnnoPath(file.path);
  }

  const TOOLS: { id: Tool; label: string; icon: string; hint: string }[] = [
    { id: "select", label: "Seleccionar", icon: "↖", hint: "Move as figuras. Dobre clic nun texto para editalo; Supr borra a seleccionada." },
    { id: "pen", label: "Lapis", icon: "✎", hint: "Debuxo a man alzada." },
    { id: "highlight", label: "Marcador", icon: "▮", hint: "Trazo grosa e translúcida, como un rotulador fluorescente." },
    { id: "line", label: "Liña", icon: "╱", hint: "Liña recta. Con Maiús, a 0°, 45° ou 90°." },
    { id: "arrow", label: "Frecha", icon: "→", hint: "Liña con punta de frecha." },
    { id: "rect", label: "Rectángulo", icon: "▭", hint: "Con Maiús, un cadrado." },
    { id: "ellipse", label: "Elipse", icon: "◯", hint: "Con Maiús, un círculo." },
    { id: "text", label: "Texto", icon: "T", hint: "Preme na imaxe e escribe. Ctrl+Intro ou Esc remata." },
    { id: "eraser", label: "Goma", icon: "⌫", hint: "Borra as figuras polas que pases. Só toca o que debuxaches, nunca a imaxe." },
  ];

  $: tool = $annoTool;
  $: textTool = tool === "text" || ($selected !== null && $shapes.find((s) => s.id === $selected)?.type === "text");
  $: shapeTool = tool === "rect" || tool === "ellipse" || ($selected !== null && ["rect", "ellipse"].includes($shapes.find((s) => s.id === $selected)?.type ?? ""));
  $: noStroke = textTool && tool !== "select" ? true : false;

  // Cambiar a cor, o groso ou a letra cambia tamén a figura seleccionada.
  $: restyle($annoStyle);
  function restyle(st: typeof $annoStyle) {
    const id = get(selected);
    if (id === null) return;
    shapes.update((l) =>
      l.map((s) =>
        s.id !== id
          ? s
          : s.type === "text"
            ? { ...s, color: st.color, size: st.size, font: st.font, bold: st.bold, italic: st.italic }
            : { ...s, color: st.color, width: st.width, fill: s.type === "rect" || s.type === "ellipse" ? st.fill : s.fill },
      ),
    );
  }
  // Ao seleccionar unha figura, o panel amosa os seus axustes.
  $: if ($selected !== null) {
    const s = get(shapes).find((x) => x.id === $selected);
    if (s) annoStyle.update((st) => (s.type === "text" ? { ...st, color: s.color, size: s.size ?? st.size, font: s.font ?? st.font, bold: !!s.bold, italic: !!s.italic } : { ...st, color: s.color, width: s.width, fill: !!s.fill }));
  }

  async function save() {
    try {
      const out = await askOutput(file, `-${tr("anotada")}`, ext);
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const mime = ext === "jpg" ? "image/jpeg" : ext === "webp" ? "image/webp" : "image/png";
        const src = get(imageUrl) || "";
        const blob = await renderAnnotated(src, get(shapes), mime, 0.92);
        const done = await engine.saveBytes(out, new Uint8Array(await blob.arrayBuffer()));
        return `${listo()} ${fileNameOf(done)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{$t("Ferramentas")}</h4>
    <div class="tools" role="radiogroup" aria-label={$t("Ferramentas")}>
      {#each TOOLS as x}
        <button class="tl" class:on={tool === x.id} role="radio" aria-checked={tool === x.id} title={`${$t(x.label)} — ${$t(x.hint)}`} on:click={() => annoTool.set(x.id)}>
          <span class="ic" aria-hidden="true">{x.icon}</span>
          <span class="nm">{$t(x.label)}</span>
        </button>
      {/each}
    </div>
    <div class="pair">
      <button class="btn" on:click={undo} disabled={!$canUndo}><Icon name="undo" size={14} /> {$t("Desfacer")}</button>
      <button class="btn" on:click={redo} disabled={!$canRedo}><Icon name="redo" size={14} /> {$t("Refacer")}</button>
    </div>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Aspecto")}</h4>
    <ColorField label={$t("Cor")} bind:value={$annoStyle.color} />
    {#if !textTool || tool === "select"}
      <Slider label={$t("Groso")} unit="px" min={1} max={60} bind:value={$annoStyle.width} def={5} />
    {/if}
    {#if shapeTool}
      <Toggle label={$t("Encher a figura")} bind:checked={$annoStyle.fill} />
    {/if}
    {#if textTool}
      <SelectField label={$t("Letra")} bind:value={$annoStyle.font} options={FONTS.map((f) => ({ value: f, label: f }))} def="Segoe UI" />
      <Slider label={$t("Tamaño do texto")} unit="px" min={8} max={300} bind:value={$annoStyle.size} def={40} />
      <div class="pair">
        <button class="btn tog" class:on={$annoStyle.bold} aria-pressed={$annoStyle.bold} on:click={() => ($annoStyle.bold = !$annoStyle.bold)}><b>N</b> {$t("Negriña")}</button>
        <button class="btn tog" class:on={$annoStyle.italic} aria-pressed={$annoStyle.italic} on:click={() => ($annoStyle.italic = !$annoStyle.italic)}><i>C</i> {$t("Cursiva")}</button>
      </div>
    {/if}
  </div>

  <div class="group">
    <button class="btn danger block" on:click={clearAll} disabled={!$shapes.length}>{$t("Borrar todo o debuxado")}</button>
    <p class="group-note">{$t("Só se borra o que debuxaches; a imaxe orixinal non se toca.")}</p>
  </div>

  <div class="group">
    <SelectField label={$t("Gardar como")} bind:value={ext} options={["png", "jpg", "webp"].map((x) => ({ value: x, label: x.toUpperCase() }))} def="png" />
    <button class="btn primary block" on:click={save} disabled={!$shapes.length || $busy}>
      <Icon name="check" size={14} /> {$t("Gardar copia anotada")}
    </button>
  </div>

  <JobStatus {job} />
</div>

<style>
  .tools { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.35rem; }
  .tl { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.15rem; min-height: 3.1rem; padding: 0.3rem 0.2rem; background: var(--bg-input); color: var(--text-muted); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); font-size: 0.78rem; }
  .tl:hover { color: var(--text); background: var(--bg-hover); }
  .tl.on { color: var(--text); border-color: var(--accent); background: color-mix(in srgb, var(--accent) 18%, transparent); }
  .ic { font-size: 1.15rem; line-height: 1; }
  .nm { white-space: nowrap; }
  .tog.on { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 18%, transparent); }
</style>
