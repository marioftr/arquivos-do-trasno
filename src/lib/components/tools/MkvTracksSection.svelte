<script lang="ts">
  import GroupNote from "../ui/GroupNote.svelte";
  import { onMount } from "svelte";
  import * as engine from "../../engine";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { fallo, fileNameOf, listo, TRASNO } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createJob } from "../../job";
  import { askOutput } from "./shared";
  import Icon from "../Icon.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Pistas de un ou varios ficheiros: quitar, reordenar, renomear, cambiar idioma e escoller a que vai por defecto. */
  export let selection: FileEntry[];

  interface Row {
    key: string;
    input: number;
    id: number;
    type: string;
    codec: string;
    detail: string;
    keep: boolean;
    name: string;
    language: string;
    isDefault: boolean;
    orig: { name: string; language: string; isDefault: boolean };
  }

  let rows: Row[] = [];
  let loading = true;
  let loadError = "";
  const job = createJob();
  const busy = job.busy;
  const multi = selection.length > 1;
  // Cada ficheiro ten a súa cor (barra á esquerda de cada pista e lenda enriba).
  const COLORS = ["#4c9be8", "#e8a14c", "#6cc47a", "#c27be0", "#e8604c", "#4cc9c0", "#d6c04c", "#8f93a8"];
  const colorOf = (i: number) => COLORS[i % COLORS.length];

  onMount(async () => {
    try {
      const all: Row[] = [];
      for (let i = 0; i < selection.length; i++) {
        for (const x of await engine.mkvTracks(selection[i].path)) {
          const p = x.properties ?? {};
          const name = p.track_name ?? "";
          const language = p.language ?? "";
          all.push({
            key: `${i}:${x.id}`,
            input: i,
            id: x.id,
            type: x.type,
            codec: x.codec,
            detail: p.pixel_dimensions ?? (p.audio_channels ? `${p.audio_channels} ch` : ""),
            keep: true,
            name,
            language,
            isDefault: !!p.default_track,
            orig: { name, language, isDefault: false },
          });
        }
      }
      // Cando se xuntan varios ficheiros só unha pista de cada tipo pode ir por defecto.
      for (const ty of ["audio", "subtitles", "video"]) {
        let seen = false;
        all.forEach((r) => {
          if (r.type !== ty) return;
          if (r.isDefault && seen) r.isDefault = false;
          if (r.isDefault) seen = true;
        });
      }
      rows = all.map((r) => ({ ...r, orig: { ...r.orig, isDefault: r.isDefault } }));
    } catch (e) {
      loadError = errText(e);
    } finally {
      loading = false;
    }
  });

  $: kept = rows.filter((r) => r.keep);
  $: orderChanged = rows.some((r, i, a) => i > 0 && (a[i - 1].input > r.input || (a[i - 1].input === r.input && a[i - 1].id > r.id)));
  $: changed = multi || orderChanged || rows.some((r) => !r.keep || r.name !== r.orig.name || r.language !== r.orig.language || r.isDefault !== r.orig.isDefault);

  function setDefault(i: number) {
    const r = rows[i];
    rows = rows.map((x, j) => (x.type === r.type && x.keep ? { ...x, isDefault: j === i ? !r.isDefault : false } : x));
  }
  const update = (i: number, patch: Partial<Row>) => (rows = rows.map((x, j) => (j === i ? { ...x, ...patch } : x)));

  function moveTo(from: number, to: number) {
    to = Math.max(0, Math.min(rows.length - 1, to));
    if (from === to || from < 0) return;
    const next = [...rows];
    const [x] = next.splice(from, 1);
    next.splice(to, 0, x);
    rows = next;
  }
  // Arrastrar con punteiro: o arrastre do navegador non vale (a ventá captura os ficheiros).
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

  async function save() {
    try {
      const single = selection[0];
      const ext = !multi && (single.ext === "webm" || single.ext === "mka") ? single.ext : "mkv";
      const out = await askOutput(single, tr("-pistas"), ext);
      if (!out) return;
      const inputs: engine.MuxInput[] = selection.map((f, i) => ({
        path: f.path,
        keep: kept.filter((r) => r.input === i).map((r) => r.id),
        edits: kept
          .filter((r) => r.input === i)
          .map((r) => ({
            id: r.id,
            name: r.name !== r.orig.name ? r.name : undefined,
            language: r.language !== r.orig.language ? r.language : undefined,
            default: r.type === "video" ? undefined : r.isDefault,
          })),
      }));
      const order = kept.map((r) => ({ input: r.input, id: r.id }));
      await job.run(TRASNO.traballando, fallo("editando"), async () => `${listo()} ${fileNameOf(await engine.mkvMux(inputs, order, out, true))}`);
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <div class="group">
    <h4 class="group-title">{multi ? $t("Pistas dos ficheiros") : $t("Pistas do ficheiro")}</h4>
    <p class="group-note">
      {multi
        ? $t("Escolle as pistas de cada ficheiro e ordénaas: sairá un só MKV. Non se recodifica nada e os orixinais non se tocan.")
        : $t("Quita ou reordena as pistas e cambia o nome ou o idioma. Non se recodifica nada e o orixinal non se toca.")}
    </p>
    <div class="help">
      <button class="btn ghost" on:click={() => engine.openLink("iso639-2")} title={$t("Abre a lista oficial de códigos de idioma (ISO 639-2) no navegador")}><Icon name="info" size={14} /> {$t("Códigos de idioma")}</button>
      <button class="btn ghost" on:click={() => engine.openLink("mkvmerge-doc")} title={$t("Abre a documentación oficial de mkvmerge no navegador")}><Icon name="info" size={14} /> {$t("Guía de MKVToolNix")}</button>
    </div>
    {#if multi}
      <ul class="legend">
        {#each selection as f, i}<li><span class="dot" style={`background:${colorOf(i)}`}></span><span class="fn" title={f.path}>{f.name}</span></li>{/each}
      </ul>
    {/if}
    {#if loadError}
      <p class="group-note">{loadError}</p>
    {:else if loading}
      <GroupNote text={$t("O trasno está lendo as pistas…")} />
    {:else}
      <div class="list">
        {#each rows as r, i (r.key)}
          <div
            class="track"
            class:off={!r.keep}
            class:lifted={dragging && dragFrom === i}
            class:target={dragging && dragOver === i && dragFrom !== i}
            data-i={i}
            style={`${multi ? `border-left-color:${colorOf(r.input)};` : ""}${dragging && dragFrom === i ? `transform: translateY(${oy}px)` : ""}`}
          >
            <div class="l1">
              <span class="grip" on:pointerdown={(e) => pdown(e, i)} on:pointermove={pmove} on:pointerup={pup} on:pointercancel={pup} role="presentation" title={$t("Arrastra para mover")}>⠿</span>
              <input type="checkbox" checked={r.keep} on:change={(e) => update(i, { keep: e.currentTarget.checked })} aria-label={$t("Conservar esta pista")} title={$t("Conservar esta pista")} />
              <span class={`kind ${r.type}`}>{r.type === "video" ? $t("Vídeo") : r.type === "audio" ? $t("Audio") : r.type === "subtitles" ? $t("Subtítulos") : $t("Outra")}</span>
              <span class="muted detail" title={`${r.codec}${r.detail ? " · " + r.detail : ""}`}>{r.codec}{r.detail ? " · " + r.detail : ""}</span>
              {#if r.type !== "video"}
                <button class="mini def" class:on={r.isDefault} disabled={!r.keep} on:click={() => setDefault(i)} aria-pressed={r.isDefault} aria-label={$t("Pista que se reproduce por defecto")} title={$t("Pista que se reproduce por defecto")}><Icon name="check" size={13} /></button>
              {/if}
              <button class="mini" on:click={() => moveTo(i, i - 1)} disabled={i === 0} aria-label={$t("Subir")} title={$t("Subir")}><Icon name="up" size={13} /></button>
              <button class="mini" on:click={() => moveTo(i, i + 1)} disabled={i === rows.length - 1} aria-label={$t("Baixar")} title={$t("Baixar")}><Icon name="down" size={13} /></button>
            </div>
            <div class="l2">
              <input class="txt" value={r.name} placeholder={$t("Nome da pista")} disabled={!r.keep} on:input={(e) => update(i, { name: e.currentTarget.value })} aria-label={$t("Nome da pista")} />
              <input class="txt lang" value={r.language} placeholder="und" maxlength="8" disabled={!r.keep} on:input={(e) => update(i, { language: e.currentTarget.value })} aria-label={$t("Idioma (código de 3 letras)")} title={$t("Idioma (código de 3 letras)")} />
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
  <button class="btn primary block" on:click={save} disabled={$busy || loading || !!loadError || !changed || kept.length === 0}>{$t("Gardar unha copia")}</button>
  <JobStatus {job} />
</div>

<style>
  .help { display: flex; flex-wrap: wrap; gap: 0.3rem; }
  .legend { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.2rem; font-size: var(--fs-sm); }
  .legend li { display: flex; align-items: center; gap: 0.45rem; min-width: 0; }
  .dot { width: 9px; height: 9px; border-radius: 50%; flex: 0 0 auto; }
  .fn { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .list { display: flex; flex-direction: column; border: 1px solid var(--border); border-radius: var(--radius-sm); overflow: hidden; }
  .track { display: flex; flex-direction: column; gap: 0.25rem; padding: 0.35rem 0.4rem; background: var(--bg-input); border-bottom: 1px solid var(--border); border-left: 3px solid transparent; }
  .track:last-child { border-bottom-color: transparent; }
  .track.off { opacity: 0.5; }
  .track.lifted { position: relative; z-index: 5; pointer-events: none; opacity: 0.92; box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4); background: var(--bg-elevated); }
  .track.target { background: color-mix(in srgb, var(--accent) 16%, var(--bg-input)); }
  .l1, .l2 { display: flex; align-items: center; gap: 0.35rem; }
  .l2 { padding-left: 1.5rem; }
  .grip { cursor: grab; touch-action: none; user-select: none; color: var(--text-faint); padding: 0 0.15rem; line-height: 1; }
  .kind { flex: 0 0 auto; font-size: var(--fs-sm); font-weight: 650; }
  .kind.video { color: var(--accent-video); }
  .kind.audio { color: var(--accent-audio); }
  .kind.subtitles { color: var(--accent-pdf); }
  .detail { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: var(--fs-xs); }
  .txt { flex: 1; min-width: 0; height: 26px; background: var(--bg); color: var(--text); border: 1px solid var(--border-strong); border-radius: 4px; padding: 0 0.4em; font: inherit; font-size: var(--fs-sm); }
  .txt.lang { flex: 0 0 3.6rem; text-align: center; }
  .txt:focus { outline: none; border-color: var(--accent); }
  .mini { flex: 0 0 auto; display: flex; width: 22px; height: 22px; align-items: center; justify-content: center; background: none; border: none; color: var(--text-muted); border-radius: 4px; padding: 0; }
  .mini:hover:not(:disabled) { color: var(--text); background: var(--bg-hover); }
  .mini:disabled { opacity: 0.25; cursor: default; }
  .mini.def.on { background: var(--accent); color: var(--on-accent); }
</style>
