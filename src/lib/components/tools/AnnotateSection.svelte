<script lang="ts">
  import GroupNote from "../ui/GroupNote.svelte";
  import { claim } from "../../claim";
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { imageUrl, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import { annoActive, annoTool, annoStyle, shapes, selected, canUndo, canRedo, undo, redo, clearAll, resetAnnotations, renderAnnotated, liveEdit, toolPrefs, type Tool } from "../../annot";
  import { activateAnno } from "../../annot";
  import Slider from "../ui/Slider.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import FontPicker from "../ui/FontPicker.svelte";
  import ColorField from "../ui/ColorField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import Icon from "../Icon.svelte";

  export let file: FileEntry;

  const job = createJob();
  const busy = job.busy;
  let ext = file.ext === "jpg" || file.ext === "webp" ? file.ext : "png";

  onMount(() => {
    activateAnno(file.path);
    annoActive.set(true);
  });
  const own = claim("annotate");
  onDestroy(() =>
    own.release(() => {
      annoActive.set(false);
      selected.set(null);
    }),
  );
  // Outra imaxe: gárdase o debuxado desta e recupérase o da nova.
  $: activateAnno(file.path);

  const TOOLS: { id: Tool; label: string; icon: string; hint: string }[] = [
    { id: "select", label: "Seleccionar", icon: "cursor", hint: "Move as figuras. Dobre clic nun texto para editalo; Supr borra a seleccionada." },
    { id: "pen", label: "Lapis", icon: "pencil", hint: "Debuxo a man alzada." },
    { id: "highlight", label: "Marcador", icon: "highlighter", hint: "Trazo grosa e translúcida, como un rotulador fluorescente." },
    { id: "line", label: "Liña", icon: "line", hint: "Liña recta. Con Maiús, a 0°, 45° ou 90°." },
    { id: "arrow", label: "Frecha", icon: "arrow-diag", hint: "Liña con punta de frecha." },
    { id: "rect", label: "Rectángulo", icon: "square", hint: "Con Maiús, un cadrado." },
    { id: "ellipse", label: "Elipse", icon: "circle", hint: "Con Maiús, un círculo." },
    { id: "text", label: "Texto", icon: "type", hint: "Preme na imaxe e escribe. Ctrl+Intro ou Esc remata." },
    { id: "eraser", label: "Goma", icon: "eraser", hint: "Borra as figuras polas que pases. Só toca o que debuxaches, nunca a imaxe." },
  ];

  $: tool = $annoTool;
  // Cada ferramenta lembra o seu groso e o seu recheo, e o xiro volve a 0: debuxar outra cousa non herda o da anterior.
  let prevTool: Tool = get(annoTool);
  $: switchTool(tool);
  function switchTool(t: Tool) {
    if (t === prevTool) return;
    const st = get(annoStyle);
    if (toolPrefs[prevTool]) toolPrefs[prevTool] = { width: st.width, fill: st.fill };
    prevTool = t;
    if (t !== "select") selected.set(null);
    const p = toolPrefs[t];
    annoStyle.update((x) => ({ ...x, width: p?.width ?? x.width, fill: p?.fill ?? x.fill, angle: 0 }));
  }
  $: selShape = $shapes.find((x) => x.id === $selected) ?? null;
  // Que axustes se ven: os da figura escollida ou, se non hai ningunha, os da ferramenta activa.
  $: kind = selShape?.type ?? (tool === "select" || tool === "eraser" ? null : tool);
  $: showWidth = !!kind && kind !== "text";
  $: showFill = kind === "rect" || kind === "ellipse";
  $: showText = kind === "text";

  // Xiro da figura escollida (o mesmo que o círculo do visor, con número editable).
  let uiAngle = 0;
  $: uiAngle = selShape ? (selShape.angle ?? 0) : $annoStyle.angle;
  function setAngle(v: number) {
    const id = get(selected);
    const cur = get(shapes).find((x) => x.id === id);
    if (!cur) {
      if (get(annoStyle).angle !== v) annoStyle.update((st) => ({ ...st, angle: v }));
      return;
    }
    if ( (cur.angle ?? 0) === v) return;
    liveEdit((l) => l.map((x) => (x.id === id ? { ...x, angle: v } : x)));
  }
  $: setAngle(uiAngle);

  type St = typeof $annoStyle;
  const sameStyle = (sh: (typeof $shapes)[number], st: St) =>
    sh.type === "text"
      ? sh.color === st.color && sh.size === st.size && sh.family === st.family && !!sh.bold === st.bold && !!sh.italic === st.italic
      : sh.color === st.color && sh.width === st.width && (sh.type === "rect" || sh.type === "ellipse" ? !!sh.fill === st.fill && (!st.fill || (sh.fillColor || sh.color) === (st.fillColor || st.color)) : true);
  // Ao activar o recheo, a súa cor empeza sendo a do trazo.
  $: if ($annoStyle.fill && !$annoStyle.fillColor) annoStyle.update((st) => ({ ...st, fillColor: st.color }));
  // Cambiar a cor, o groso ou a letra cambia tamén a figura escollida...
  $: restyle($annoStyle);
  function restyle(st: St) {
    const id = get(selected);
    const cur = get(shapes).find((x) => x.id === id);
    if (!cur || sameStyle(cur, st)) return;
    liveEdit((l) =>
      l.map((x) =>
        x.id !== id
          ? x
          : x.type === "text"
            ? { ...x, color: st.color, size: st.size, family: st.family, bold: st.bold, italic: st.italic }
            : { ...x, color: st.color, width: st.width, ...(x.type === "rect" || x.type === "ellipse" ? { fill: st.fill, fillColor: st.fill ? st.fillColor || st.color : x.fillColor } : {}) },
      ),
    );
  }
  // ...e ao revés: o panel amosa os axustes da figura escollida (tamén despois de escalala ou cambiala).
  $: if (selShape && !sameStyle(selShape, get(annoStyle))) {
    const sh = selShape;
    annoStyle.update((st) => (sh.type === "text" ? { ...st, color: sh.color, size: Math.round(sh.size ?? st.size), family: sh.family ?? st.family, bold: !!sh.bold, italic: !!sh.italic } : { ...st, color: sh.color, width: sh.width, fill: !!sh.fill, fillColor: sh.fill ? sh.fillColor || sh.color : st.fillColor }));
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
          <span class="ic" aria-hidden="true"><Icon name={x.icon} size={18} /></span>
          <span class="nm">{$t(x.label)}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Aspecto")}</h4>
    {#if kind}
        <Slider label={$t("Xiro")} unit="°" min={-180} max={180} bind:value={uiAngle} def={0} />
      <ColorField label={$t("Cor")} bind:value={$annoStyle.color} />
      {#if showWidth}
        <Slider label={$t("Groso")} unit="px" min={1} max={60} bind:value={$annoStyle.width} def={5} />
      {/if}
      {#if showFill}
        <Toggle label={$t("Encher a figura")} bind:checked={$annoStyle.fill} />
        {#if $annoStyle.fill}
          <ColorField label={$t("Cor do recheo")} bind:value={$annoStyle.fillColor} />
        {/if}
      {/if}
      {#if showText}
        <FontPicker label={$t("Letra")} bind:family={$annoStyle.family} bind:bold={$annoStyle.bold} bind:italic={$annoStyle.italic} defFamily="Helvetica" />
        <Slider label={$t("Tamaño do texto")} unit="px" min={8} max={300} bind:value={$annoStyle.size} def={40} />
      {/if}
    {:else if tool === "select"}
      <GroupNote text={$t("Preme unha figura para movela, cambiar o seu aspecto, escalala coa esquina ou xirala co círculo.")} />
    {/if}
  </div>

  <div class="group">
    <button class="btn danger block" on:click={clearAll} disabled={!$shapes.length}>{$t("Borrar todas as anotacións")}</button>
  </div>

  <div class="group">
    <SelectField label={$t("Gardar como")} bind:value={ext} options={["png", "jpg", "webp"].map((x) => ({ value: x, label: x.toUpperCase() }))} def="png" />
    <button class="btn primary block" on:click={save} disabled={!$shapes.length || $busy}>
      {$t("Gardar copia anotada")}
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
</style>
