<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import { pdfOverlay, viewUrl, type FileEntry, type PdfTextBox } from "../../files";
  import { resolveFont } from "../../fonts";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { askOutput, safePages, parsePageSpec } from "./shared";
  import Slider from "../ui/Slider.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import TextField from "../ui/TextField.svelte";
  import ColorField from "../ui/ColorField.svelte";
  import FontPicker from "../ui/FontPicker.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import Icon from "../Icon.svelte";

  export let file: FileEntry;

  const form = createForm({
    kind: "watermark" as "watermark" | "numbers" | "text" | "image",
    pageSpec: "",
    // marca de auga
    wmText: "BORRADOR",
    wmSize: 60,
    wmOpacity: 30,
    wmAngle: 45,
    wmColor: "#c0392b",
    wmFamily: "Helvetica",
    wmBold: true,
    wmItalic: false,
    // números
    numPosition: "bc",
    numFormat: "{n}",
    numStart: 1,
    numSize: 11,
    numMargin: 28,
    numColor: "#000000",
    numFamily: "Helvetica",
    numBold: false,
    numItalic: false,
    // textos: tantos como se queira, colocados cun clic sobre a páxina
    txItems: [] as PdfTextBox[],
    txSel: null as number | null,
    txEditing: null as number | null,
    txFamily: "Helvetica",
    txBold: false,
    txItalic: false,
    txSize: 16,
    txColor: "#000000",
    // imaxe / firma
    imX: 80,
    imY: 12,
    imWidth: 25,
    imOpacity: 100,
  });
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;
  let imagePath: string | null = null;
  let imageUrl = "";

  $: f = $form;
  $: pages = safePages(f.pageSpec, file.pages);

  // ---- textos sobre a páxina: o visor mostra, move, escala e xira; aquí quedan a lista e os axustes do escollido
  let nextId = 1;
  $: sel = f.txItems.find((i) => i.id === f.txSel) ?? null;
  interface Ui {
    text: string;
    family: string;
    bold: boolean;
    italic: boolean;
    size: number;
    color: string;
    angle: number;
  }
  let ui: Ui = { text: "", family: "Helvetica", bold: false, italic: false, size: 16, color: "#000000", angle: 0 };
  let lastSel: number | null = -1;
  // Ao escoller outro texto, o panel carga os seus axustes (se non hai ningún, os que levará o próximo).
  $: if (f.txSel !== lastSel) {
    lastSel = f.txSel;
    ui = sel
      ? { text: sel.text, family: sel.family, bold: sel.bold, italic: sel.italic, size: sel.size, color: sel.color, angle: sel.angle }
      : { text: "", family: f.txFamily, bold: f.txBold, italic: f.txItalic, size: f.txSize, color: f.txColor, angle: 0 };
  }
  // Cambiar un axuste cambia o texto escollido (ou os valores do próximo, se non hai ningún).
  $: pushUi(ui);
  function pushUi(u: Ui) {
    const cur = get(form);
    const it = cur.txItems.find((i) => i.id === cur.txSel);
    const defaults = { txFamily: u.family, txBold: u.bold, txItalic: u.italic, txSize: u.size, txColor: u.color };
    if (it) {
      if (it.text !== u.text || it.family !== u.family || it.bold !== u.bold || it.italic !== u.italic || it.size !== u.size || it.color !== u.color || it.angle !== u.angle) {
        form.update((v) => ({ ...v, ...defaults, txItems: v.txItems.map((i) => (i.id === v.txSel ? { ...i, text: u.text, family: u.family, bold: u.bold, italic: u.italic, size: u.size, color: u.color, angle: u.angle } : i)) }));
      }
    } else if (cur.txFamily !== u.family || cur.txBold !== u.bold || cur.txItalic !== u.italic || cur.txSize !== u.size || cur.txColor !== u.color) {
      form.update((v) => ({ ...v, ...defaults }));
    }
  }
  const patch = (id: number, p: Partial<PdfTextBox>) => form.update((v) => ({ ...v, txItems: v.txItems.map((i) => (i.id === id ? { ...i, ...p } : i)), txSel: id }));
  const items = {
    onSelect: (id: number | null) => form.update((v) => (v.txSel === id && v.txEditing === null ? v : { ...v, txSel: id, txEditing: null })),
    onPlace: (page: number, x: number, y: number) => {
      const id = nextId++;
      form.update((v) => ({
        ...v,
        txItems: [...v.txItems, { id, page, xPct: x, yPct: y, text: tr("Texto"), size: v.txSize, color: v.txColor, family: v.txFamily, bold: v.txBold, italic: v.txItalic, angle: 0 }],
        txSel: id,
        txEditing: id,
      }));
    },
    onMove: (id: number, x: number, y: number) => patch(id, { xPct: x, yPct: y }),
    onResize: (id: number, size: number) => {
      ui = { ...ui, size };
      patch(id, { size });
    },
    onRotate: (id: number, angle: number) => {
      ui = { ...ui, angle };
      patch(id, { angle });
    },
    onEdit: (id: number, text: string, style?: { bold: boolean; italic: boolean }) => {
      const cur = get(form);
      const it = cur.txItems.find((i) => i.id === id);
      if (!it) return;
      const bold = style?.bold ?? it.bold;
      const italic = style?.italic ?? it.italic;
      if (it.text === text && it.bold === bold && it.italic === italic && cur.txEditing === null) return;
      if (cur.txSel === id) ui = { ...ui, text, bold, italic };
      form.update((v) => ({ ...v, txItems: v.txItems.map((i) => (i.id === id ? { ...i, text, bold, italic } : i)), txEditing: null }));
    },
    onEditing: (id: number | null) => form.update((v) => ({ ...v, txEditing: id, txSel: id ?? v.txSel })),
  };
  function remove(id: number) {
    form.update((v) => ({ ...v, txItems: v.txItems.filter((i) => i.id !== id), txSel: v.txSel === id ? null : v.txSel, txEditing: null }));
  }
  /** Repite o texto escollido, na mesma posición, en todas as demais páxinas. */
  function repeatEverywhere() {
    const it = sel;
    const total = file.pages ?? 0;
    if (!it || !total) return;
    const copies: PdfTextBox[] = [];
    for (let n = 1; n <= total; n++) if (n !== it.page && !f.txItems.some((x) => x.page === n && x.text === it.text && x.xPct === it.xPct && x.yPct === it.yPct)) copies.push({ ...it, id: nextId++, page: n });
    form.update((v) => ({ ...v, txItems: [...v.txItems, ...copies] }));
  }
  const snippet = (s: string) => (s.replace(/\s+/g, " ").trim().slice(0, 34) || "…") + (s.length > 34 ? "…" : "");

  // Atallos co texto escollido: Supr borra, Ctrl+B negriña, Ctrl+I cursiva, Esc deixa de escoller.
  function key(e: KeyboardEvent) {
    if (f.kind !== "text" || f.txSel === null || f.txEditing !== null) return;
    if ((e.target as HTMLElement).closest("input, textarea, select, [contenteditable]")) return;
    const k = e.key.toLowerCase();
    if (e.key === "Delete" || e.key === "Backspace") {
      remove(f.txSel);
      e.preventDefault();
    } else if ((e.ctrlKey || e.metaKey) && k === "b") {
      ui = { ...ui, bold: !ui.bold };
      e.preventDefault();
    } else if ((e.ctrlKey || e.metaKey) && k === "i") {
      ui = { ...ui, italic: !ui.italic };
      e.preventDefault();
    } else if (e.key === "Escape") items.onSelect(null);
  }

  // Vista previa en directo sobre as páxinas.
  $: pdfOverlay.set(
    f.kind === "watermark"
      ? { text: { text: f.wmText, xPct: 50, yPct: 50, size: f.wmSize, color: f.wmColor, opacity: f.wmOpacity / 100, angle: f.wmAngle, centered: true, pages, font: { family: f.wmFamily, bold: f.wmBold, italic: f.wmItalic } } }
      : f.kind === "numbers"
        ? { numbers: { position: f.numPosition, format: f.numFormat, start: f.numStart, size: f.numSize, margin: f.numMargin, color: f.numColor, pages, font: { family: f.numFamily, bold: f.numBold, italic: f.numItalic } } }
        : f.kind === "text"
          ? { items: { list: f.txItems, selected: f.txSel, editing: f.txEditing, ...items } }
          : imageUrl
            ? { image: { url: imageUrl, xPct: f.imX, yPct: f.imY, widthPct: f.imWidth, opacity: f.imOpacity / 100, pages } }
            : null,
  );
  onDestroy(() => pdfOverlay.set(null));

  async function chooseImage() {
    const picked = await open({ multiple: false, filters: [{ name: tr("Imaxes"), extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"] }] });
    if (typeof picked !== "string") return;
    imagePath = picked;
    imageUrl = await viewUrl(picked);
  }

  async function apply() {
    let sel2: number[] | null = null;
    if (f.kind !== "text") {
      try {
        sel2 = parsePageSpec(f.pageSpec, file.pages);
      } catch (e) {
        job.say(false, errText(e));
        return;
      }
    }
    if (f.kind === "text" && !f.txItems.some((i) => i.text.trim())) {
      job.say(false, tr("Preme primeiro sobre a páxina onde queres o texto."));
      return;
    }
    if (f.kind === "image" && !imagePath) {
      job.say(false, tr("Escolle primeiro a imaxe (por exemplo, a túa firma)."));
      return;
    }
    const suffix = { watermark: "-marca", numbers: "-numerado", text: "-texto", image: "-sinatura" }[f.kind];
    try {
      const out = await askOutput(file, tr(suffix), "pdf");
      if (!out) return;
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        let done: string;
        if (f.kind === "watermark") {
          done = await engine.pdfWatermarkText(file.path, out, { text: f.wmText, size: f.wmSize, opacity: f.wmOpacity / 100, angle: f.wmAngle, color: f.wmColor, font: resolveFont({ family: f.wmFamily, bold: f.wmBold, italic: f.wmItalic }) }, sel2, true);
        } else if (f.kind === "numbers") {
          done = await engine.pdfPageNumbers(file.path, out, { position: f.numPosition, format: f.numFormat, start: f.numStart, size: f.numSize, margin: f.numMargin, color: f.numColor, font: resolveFont({ family: f.numFamily, bold: f.numBold, italic: f.numItalic }) }, sel2, true);
        } else if (f.kind === "text") {
          done = await engine.pdfAddTexts(
            file.path,
            out,
            f.txItems.map((i) => ({ page: i.page, text: i.text, xPct: i.xPct, yPct: i.yPct, size: i.size, color: i.color, font: resolveFont(i), angle: i.angle })),
            true,
          );
        } else {
          done = await engine.pdfStampImage(file.path, out, { image: imagePath!, xPct: f.imX, yPct: f.imY, widthPct: f.imWidth, opacity: f.imOpacity / 100 }, sel2, true);
        }
        return `${listo()} ${fileNameOf(done)}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<svelte:window on:keydown={key} />

<div class="stack">

  <Segmented
    options={[
      { value: "watermark", label: $t("Marca de auga") },
      { value: "numbers", label: $t("Números") },
      { value: "text", label: $t("Texto") },
      { value: "image", label: $t("Firma / imaxe") },
    ]}
    bind:value={$form.kind}
  />

  <div class="group">
    {#if f.kind === "watermark"}
      <TextField label={$t("Texto da marca de auga")} bind:value={$form.wmText} />
      <FontPicker label={$t("Letra")} bind:family={$form.wmFamily} bind:bold={$form.wmBold} bind:italic={$form.wmItalic} defFamily="Helvetica" />
      <Slider label={$t("Tamaño")} unit="pt" min={20} max={200} bind:value={$form.wmSize} def={60} />
      <Slider label={$t("Opacidade")} unit="%" min={5} max={100} bind:value={$form.wmOpacity} def={30} />
      <Slider label={$t("Xiro")} unit="°" min={-90} max={90} bind:value={$form.wmAngle} def={45} />
      <ColorField label={$t("Cor")} bind:value={$form.wmColor} />
    {:else if f.kind === "numbers"}
      <SelectField
        label={$t("Posición")}
        bind:value={$form.numPosition}
        def="bc"
        options={[
          { value: "bc", label: $t("Abaixo, centro") },
          { value: "br", label: $t("Abaixo, dereita") },
          { value: "bl", label: $t("Abaixo, esquerda") },
          { value: "tc", label: $t("Arriba, centro") },
          { value: "tr", label: $t("Arriba, dereita") },
          { value: "tl", label: $t("Arriba, esquerda") },
        ]}
      />
      <SelectField
        label={$t("Formato")}
        hint={$t("Como se escribe o número: só o número, ou «3 / 10» para saber tamén o total de páxinas.")}
        bind:value={$form.numFormat}
        def={"{n}"}
        options={[
          { value: "{n}", label: "1, 2, 3…" },
          { value: "{n} / {total}", label: "1 / 10, 2 / 10…" },
          { value: tr("Páxina") + " {n}", label: tr("Páxina") + " 1, " + tr("Páxina") + " 2…" },
          { value: "- {n} -", label: "- 1 -, - 2 -…" },
        ]}
      />
      <FontPicker label={$t("Letra")} bind:family={$form.numFamily} bind:bold={$form.numBold} bind:italic={$form.numItalic} defFamily="Helvetica" />
      <div class="pair">
        <NumberField label={$t("Comezar en")} min={0} bind:value={$form.numStart} def={1} />
        <NumberField label={$t("Tamaño")} unit="pt" min={6} max={48} bind:value={$form.numSize} def={11} />
      </div>
      <Slider label={$t("Distancia ao bordo")} unit="pt" min={8} max={120} bind:value={$form.numMargin} def={28} />
      <ColorField label={$t("Cor")} bind:value={$form.numColor} />
    {:else if f.kind === "text"}
      {#if f.txItems.length}
        <div class="list" role="listbox" aria-label={$t("Textos engadidos")}>
          {#each f.txItems as it (it.id)}
            <div class="li" class:on={it.id === f.txSel}>
              <button class="pick" role="option" aria-selected={it.id === f.txSel} on:click={() => items.onSelect(it.id)} title={it.text}>
                <span class="pg">{$t("Páx.")} {it.page}</span>
                <span class="sn">{snippet(it.text)}</span>
              </button>
              <button class="del" on:click={() => remove(it.id)} aria-label={$t("Quitar este texto")} title={$t("Quitar este texto")}><Icon name="close" size={13} /></button>
            </div>
          {/each}
        </div>
      {/if}
      {#if sel}
        <label class="ta">
          <span class="lab">{$t("Texto")}</span>
          <textarea bind:value={ui.text} rows="3"></textarea>
        </label>
      {/if}
      <FontPicker label={$t("Letra")} bind:family={ui.family} bind:bold={ui.bold} bind:italic={ui.italic} defFamily="Helvetica" />
      <Slider label={$t("Tamaño")} unit="pt" min={4} max={300} bind:value={ui.size} def={16} />
      {#if sel}<Slider label={$t("Xiro")} unit="°" min={-180} max={180} bind:value={ui.angle} def={0} />{/if}
      <ColorField label={$t("Cor")} bind:value={ui.color} />
      {#if sel}
        <button class="btn" on:click={repeatEverywhere} disabled={!file.pages || file.pages < 2}>{$t("Repetir en todas as páxinas")}</button>
      {/if}
    {:else}
      <button class="btn" on:click={chooseImage}><Icon name="image" size={14} /> {imagePath ? fileNameOf(imagePath) : $t("Escoller imaxe…")}</button>
      <p class="group-note">{$t("Vale un PNG con fondo transparente, por exemplo a túa firma.")}</p>
      <Slider label={$t("Posición horizontal")} unit="%" min={0} max={100} bind:value={$form.imX} def={80} />
      <Slider label={$t("Posición vertical")} unit="%" min={0} max={100} bind:value={$form.imY} def={12} hint={$t("0 % = abaixo; 100 % = arriba")} />
      <Slider label={$t("Ancho")} unit="%" min={3} max={90} bind:value={$form.imWidth} def={25} />
      <Slider label={$t("Opacidade")} unit="%" min={5} max={100} bind:value={$form.imOpacity} def={100} />
    {/if}
    {#if f.kind !== "text"}
      <TextField label={$t("Páxinas (baleiro = todas)")} placeholder={$t("ex: 1,3,5-7")} bind:value={$form.pageSpec} />
    {/if}
  </div>

  <button class="btn primary block" on:click={apply} disabled={$busy || (f.kind === "text" && !f.txItems.length)}>{$t("Gardar unha copia co cambio")}</button>
  <JobStatus {job} />
</div>

<style>
  .list { display: flex; flex-direction: column; gap: 2px; max-height: 11rem; overflow-y: auto; border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 3px; }
  .li { display: flex; align-items: center; border-radius: 5px; }
  .li.on { background: color-mix(in srgb, var(--accent) 18%, transparent); }
  .pick { flex: 1; min-width: 0; display: flex; gap: 0.6rem; align-items: baseline; text-align: left; background: none; border: none; color: var(--text); padding: 0.35rem 0.5rem; font-size: 0.84rem; border-radius: 5px; }
  .pick:hover { background: var(--bg-hover); }
  .pg { flex: 0 0 auto; font-size: 0.72rem; color: var(--text-faint); font-variant-numeric: tabular-nums; }
  .sn { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .del { display: grid; place-items: center; width: 26px; height: 26px; flex: 0 0 auto; background: none; border: none; border-radius: 5px; color: var(--text-faint); }
  .del:hover { background: var(--bg-hover); color: var(--accent-video); }
  .ta { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { font-size: 0.82rem; color: var(--text-muted); }
  textarea { width: 100%; resize: vertical; min-height: 4.2rem; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0.4rem 0.6rem; font: inherit; font-size: 0.9rem; }
  textarea:focus { outline: none; border-color: var(--accent); }
</style>
