<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { Category } from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { samePath } from "./shared";
  import Icon from "../Icon.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  export let selection: FileEntry[];
  export let category: Category;

  const first = selection[0];

  // A orde de partida é a da lista; aquí pódese cambiar sen mexer na lista de ficheiros.
  let items: FileEntry[] = [...selection];
  function moveTo(from: number, to: number) {
    to = Math.max(0, Math.min(items.length - 1, to));
    if (from === to || from < 0) return;
    const next = [...items];
    const [x] = next.splice(from, 1);
    next.splice(to, 0, x);
    items = next;
  }
  const setPos = (i: number, ev: Event) => {
    const el = ev.currentTarget as HTMLInputElement;
    const v = Math.round(Number(el.value));
    if (Number.isFinite(v)) moveTo(i, v - 1);
    el.value = String(i + 1);
  };
  // Arrastrar con punteiro (o arrastre do navegador non vale: a ventá captura os ficheiros).
  let dragFrom = -1, dragOver = -1, dragging = false, sy = 0, oy = 0;
  function pdown(e: PointerEvent, i: number) {
    if (e.button !== 0) return;
    dragFrom = i; dragOver = i; sy = e.clientY; oy = 0;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function pmove(e: PointerEvent) {
    if (dragFrom < 0) return;
    if (!dragging && Math.abs(e.clientY - sy) > 5) dragging = true;
    if (!dragging) return;
    oy = e.clientY - sy;
    const el = document.elementFromPoint(e.clientX, e.clientY)?.closest("[data-i]") as HTMLElement | null;
    if (el) dragOver = Number(el.dataset.i);
  }
  function pup() {
    if (dragging && dragOver >= 0) moveTo(dragFrom, dragOver);
    dragFrom = -1; dragOver = -1; dragging = false; oy = 0;
  }
  const isPdf = category === "documento";
  const job = createJob();
  const busy = job.busy;

  $: title = isPdf ? $t("Unir PDF") : category === "imaxe" ? $t("Imaxes a PDF") : category === "video" ? $t("Xuntar vídeos") : $t("Xuntar audios");

  async function merge() {
    const asPdf = isPdf || category === "imaxe";
    const out = await save({
      defaultPath: await join(first.path.replace(/[\\/][^\\/]*$/, ""), `${tr("unido")}.${asPdf ? "pdf" : first.ext}`),
    });
    if (!out) return;
    const inputs = items.map((s) => s.path);
    if (inputs.some((p) => samePath(p, out))) {
      job.say(false, tr("Non vou sobrescribir un dos ficheiros de orixe. Escolle outro nome."));
      return;
    }
    await job.run(TRASNO.unindo, fallo("unindo"), async () => {
      const done = isPdf
        ? await engine.pdfMerge(inputs, out, true)
        : category === "imaxe"
          ? await engine.imagesToPdf(inputs, out, true)
          : await engine.concatMedia(inputs, out, true);
      return `${listo()} ${fileNameOf(done)}`;
    });
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{title}</h4>
    <p class="group-note">{$t("Vaise facer nesta orde (a da lista). Arrastra os ficheiros, usa as frechas ou escribe a posición para cambiala.")}</p>
    <ol class="order">
      {#each items as f, i (f.path)}
        <li
          class:lifted={dragging && dragFrom === i}
          class:target={dragging && dragOver === i && dragFrom !== i}
          data-i={i}
          style={dragging && dragFrom === i ? `transform: translateY(${oy}px)` : ""}
        >
          <span class="grip" on:pointerdown={(e) => pdown(e, i)} on:pointermove={pmove} on:pointerup={pup} on:pointercancel={pup} role="presentation" title={$t("Arrastra para mover")}>⠿</span>
          <input class="pos" type="number" min="1" max={items.length} value={i + 1} on:change={(e) => setPos(i, e)} aria-label={$t("Posición")} />
          <span class="nm" title={f.path}>{f.name}</span>
          <button class="mini" on:click={() => moveTo(i, i - 1)} disabled={i === 0} aria-label={$t("Subir")} title={$t("Subir")}><Icon name="up" size={13} /></button>
          <button class="mini" on:click={() => moveTo(i, i + 1)} disabled={i === items.length - 1} aria-label={$t("Baixar")} title={$t("Baixar")}><Icon name="down" size={13} /></button>
        </li>
      {/each}
    </ol>
  </div>
  <button class="btn primary block" on:click={merge} disabled={$busy}>
    <Icon name="layers" size={14} />
    {isPdf ? $t("Unir nun só PDF") : category === "imaxe" ? $t("Facer un PDF cunha imaxe por páxina") : $t("Xuntar nun só ficheiro")}
  </button>
  <JobStatus {job} />
</div>

<style>
  .order { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; font-size: 0.85rem; }
  .order li { display: flex; align-items: center; gap: 0.5rem; padding: 0.3rem 0.2rem; border-bottom: 1px solid var(--border); background: var(--bg); }
  .order li.lifted { position: relative; z-index: 5; pointer-events: none; opacity: 0.9; box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4); background: var(--bg-elevated); }
  .order li.target { background: color-mix(in srgb, var(--accent) 16%, var(--bg)); }
  .grip { cursor: grab; touch-action: none; user-select: none; color: var(--text-faint); padding: 0 0.2rem; font-size: 1rem; line-height: 1; }
  .pos { width: 3rem; height: 1.7rem; text-align: center; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: 4px; font: inherit; font-size: 0.8rem; }
  .pos:focus { outline: none; border-color: var(--accent); }
  .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; user-select: none; }
  .mini { display: flex; background: none; border: none; color: var(--text-muted); padding: 0.25rem; border-radius: 4px; }
  .mini:hover:not(:disabled) { color: var(--text); background: var(--bg-hover); }
  .mini:disabled { opacity: 0.25; cursor: default; }
</style>
