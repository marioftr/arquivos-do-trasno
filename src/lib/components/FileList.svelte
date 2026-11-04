<script lang="ts">
  import Icon from "./Icon.svelte";
  import { files, activePath, checked, toggleChecked, setChecked, removeFile, moveFile, moveFileTo, checkRange, removeChecked, undoRemove } from "../files";
  import { formatBytes } from "../formats";
  import { t } from "../i18n";
  import { revealInFolder, type Category } from "../engine";
  import { toast } from "../toasts";
  import { tr } from "../i18n";

  export let onAdd: () => void;

  const ICONS: Record<Category, string> = {
    imaxe: "image",
    audio: "audio",
    video: "video",
    documento: "document",
    modelo3d: "cube",
    descoñecido: "document",
  };

  $: allChecked = $files.length > 0 && $checked.size === $files.length;
  $: activeIndex = $files.findIndex((f) => f.path === $activePath);
  $: if (activeIndex >= 0 && box) reveal(activeIndex);

  // ---- lista longa: só se debuxan as filas que se ven (cada fila mide o mesmo)
  const ROW = 54;
  let box: HTMLUListElement;
  let scrollTop = 0;
  let viewH = 600;
  $: first = Math.max(0, Math.floor(scrollTop / ROW) - 8);
  $: last = Math.min($files.length, Math.ceil((scrollTop + viewH) / ROW) + 8);
  $: shown = $files.slice(first, last);
  function reveal(j: number) {
    if (!box) return;
    const top = j * ROW;
    if (top < box.scrollTop) box.scrollTop = top;
    else if (top + ROW > box.scrollTop + box.clientHeight) box.scrollTop = top + ROW - box.clientHeight;
  }

  // ---- clic: Ctrl alterna a marca, Maiús marca un intervalo; sen modificadores, só se ve ese ficheiro
  let anchor = -1;
  function click(e: MouseEvent, i: number) {
    const path = $files[i].path;
    if (e.shiftKey && anchor >= 0) {
      checkRange(anchor, i);
    } else if (e.ctrlKey || e.metaKey) {
      toggleChecked(path);
      anchor = i;
    } else {
      anchor = i;
    }
    activePath.set(path);
  }

  // ---- teclado (cando a lista ten o foco)
  function key(e: KeyboardEvent) {
    const n = $files.length;
    if (!n || (e.target as HTMLElement).closest("input, button.mini")) return;
    const go = (i: number) => {
      const j = Math.max(0, Math.min(n - 1, i));
      activePath.set($files[j].path);
      if (e.shiftKey && anchor >= 0) checkRange(anchor, j);
      else anchor = j;
      reveal(j);
      e.preventDefault();
    };
    if (e.key === "ArrowDown") go((activeIndex < 0 ? -1 : activeIndex) + 1);
    else if (e.key === "ArrowUp") go((activeIndex < 0 ? n : activeIndex) - 1);
    else if (e.key === "Home") go(0);
    else if (e.key === "End") go(n - 1);
    else if (e.key === " " && activeIndex >= 0) {
      toggleChecked($files[activeIndex].path);
      e.preventDefault();
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") {
      setChecked($files.map((f) => f.path));
      e.preventDefault();
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
      undoRemove();
      e.preventDefault();
      e.stopPropagation(); // non desfai tamén os axustes da ferramenta
    } else if (e.altKey && e.key === "ArrowUp" && activeIndex >= 0) {
      moveFile($files[activeIndex].path, -1);
      e.preventDefault();
    } else if (e.altKey && e.key === "ArrowDown" && activeIndex >= 0) {
      moveFile($files[activeIndex].path, 1);
      e.preventDefault();
    }
  }

  // ---- arrastrar para reordenar (con punteiro: o arrastre do navegador non vale, a ventá captura os ficheiros)
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
    // Fila baixo o punteiro por xeometría (cada fila mide o mesmo): por riba da primeira ou por baixo da última vale o extremo.
    const r = box.getBoundingClientRect();
    dragOver = Math.max(0, Math.min($files.length - 1, Math.floor((e.clientY - r.top + box.scrollTop) / ROW)));
  }
  function pup() {
    if (dragging && dragOver >= 0) moveFileTo(dragFrom, dragOver);
    dragFrom = -1; dragOver = -1; dragging = false; oy = 0;
  }

  // ---- menú de clic dereito
  let menu: { x: number; y: number; i: number } | null = null;
  function ctx(e: MouseEvent, i: number) {
    e.preventDefault();
    activePath.set($files[i].path);
    menu = { x: Math.min(e.clientX, window.innerWidth - 220), y: Math.min(e.clientY, window.innerHeight - 190), i };
  }
  const closeMenu = () => (menu = null);
  async function copyPath(p: string) {
    try {
      await navigator.clipboard.writeText(p);
      toast(tr("O trasno copiou a ruta: pégaa onde queiras."));
    } catch {
      toast(tr("O portapapeis púxose teimudo coa ruta. Tenta de novo."), { kind: "warn" });
    }
  }
</script>

<svelte:window on:pointerdown={(e) => { if (menu && !(e.target as HTMLElement).closest(".ctxmenu")) closeMenu(); }} on:keydown={(e) => e.key === "Escape" && closeMenu()} />

<div class="list" data-filelist>
  <div class="head">
    <span class="titulo">{$t("Ficheiros")}<span class="muted cnt">&nbsp;·&nbsp;{$files.length}</span></span>
    <button class="btn" on:click={onAdd}><Icon name="plus" size={14} /> {$t("Engadir")}</button>
  </div>
  {#if $checked.size}
    <div class="sel" role="toolbar" aria-label={$t("Ficheiros marcados")}>
      <span class="sc">{$t("{n} marcados", { n: $checked.size })}</span>
      <span class="hact">
        <button class="ib" on:click={() => setChecked($files.map((f) => f.path))} disabled={allChecked} title={$t("Marcar todo")} aria-label={$t("Marcar todo")}><Icon name="checks" size={16} /></button>
        <button class="ib" on:click={() => setChecked([])} title={$t("Desmarcar todo")} aria-label={$t("Desmarcar todo")}><Icon name="square" size={16} /></button>
        <button class="ib danger" on:click={removeChecked} title={$t("Quitar marcados da lista")} aria-label={$t("Quitar marcados da lista")}><Icon name="trash" size={16} /></button>
      </span>
    </div>
  {/if}

  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <ul class="scrollbar-thin" tabindex="0" role="listbox" aria-label={$t("Ficheiros")} on:keydown={key} bind:this={box} bind:clientHeight={viewH} on:scroll={() => (scrollTop = box.scrollTop)}>
    {#if first > 0}<li class="spacer" aria-hidden="true" style={`height:${first * ROW}px`}></li>{/if}
    {#each shown as f, k (f.path)}
      {@const i = first + k}
      <li
        class:active={$activePath === f.path}
        class:lifted={dragging && dragFrom === i}
        class:target={dragging && dragOver === i && dragFrom !== i}
        data-i={i}
        role="option"
        aria-selected={$activePath === f.path}
        style={dragging && dragFrom === i ? `transform: translateY(${oy}px)` : ""}
        on:contextmenu={(e) => ctx(e, i)}
      >
        <span class="grip" on:pointerdown={(e) => pdown(e, i)} on:pointermove={pmove} on:pointerup={pup} on:pointercancel={pup} role="presentation" title={$t("Arrastra para mover")}>⠿</span>
        <input
          type="checkbox"
          checked={$checked.has(f.path)}
          on:change={() => toggleChecked(f.path)}
          aria-label={$t("Marcar {name}", { name: f.name })}
          title={$t("Marcar para traballar con varios ficheiros á vez")}
        />
        <button class="row" tabindex="-1" on:click={(e) => click(e, i)} title={f.path}>
          <Icon name={ICONS[f.category]} size={16} />
          <span class="info">
            <span class="nome">{f.name}</span>
            <span class="muted meta">
              {f.ext.toUpperCase() || "?"}{#if f.size !== undefined}&nbsp;·&nbsp;{formatBytes(f.size)}{/if}{#if f.pages}&nbsp;·&nbsp;{$t("{n} páx.", { n: f.pages })}{/if}
            </span>
          </span>
        </button>
        <span class="acc">
          <button class="mini" tabindex="-1" on:click={() => moveFile(f.path, -1)} disabled={i === 0} aria-label={$t("Subir")} title={$t("Subir")}><Icon name="up" size={15} /></button>
          <button class="mini" tabindex="-1" on:click={() => moveFile(f.path, 1)} disabled={i === $files.length - 1} aria-label={$t("Baixar")} title={$t("Baixar")}><Icon name="down" size={15} /></button>
          <button class="mini" tabindex="-1" on:click={() => removeFile(f.path)} aria-label={$t("Quitar")} title={$t("Quitar da lista")}><Icon name="close" size={15} /></button>
        </span>
      </li>
    {/each}
    {#if last < $files.length}<li class="spacer" aria-hidden="true" style={`height:${($files.length - last) * ROW}px`}></li>{/if}
  </ul>

</div>

{#if menu}
  {@const f = $files[menu.i]}
  <div class="ctxmenu" style={`left:${menu.x}px;top:${menu.y}px`} role="menu">
    <button role="menuitem" on:click={() => { revealInFolder(f.path); closeMenu(); }}><Icon name="folder" size={14} /> {$t("Amosar no cartafol")}</button>
    <button role="menuitem" on:click={() => { copyPath(f.path); closeMenu(); }}><Icon name="copy" size={14} /> {$t("Copiar a ruta")}</button>
    <button role="menuitem" on:click={() => { toggleChecked(f.path); closeMenu(); }}><Icon name="check" size={14} /> {$checked.has(f.path) ? $t("Desmarcar") : $t("Marcar")}</button>
    <hr />
    <button role="menuitem" on:click={() => { removeFile(f.path); closeMenu(); }}><Icon name="close" size={14} /> {$t("Quitar da lista")}</button>
  </div>
{/if}

<style>
  .list { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .head { display: flex; align-items: center; justify-content: space-between; padding: 0.6rem 0.9rem; border-bottom: 1px solid var(--border); }
  .titulo { font-weight: 650; font-size: 0.72rem; letter-spacing: 0.07em; text-transform: uppercase; color: var(--text-muted); }
  .cnt { font-weight: 500; }
  ul { list-style: none; margin: 0; padding: 0.3rem 0; flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; overscroll-behavior-x: none; outline: none; }
  ul:focus-visible { box-shadow: inset 0 0 0 2px color-mix(in srgb, var(--accent) 45%, transparent); }
  li.spacer { padding: 0; border: none; background: none; pointer-events: none; }
  li:not(.spacer) { height: 54px; box-sizing: border-box; overflow: hidden; }
  li { --rowbg: var(--bg-rail); position: relative; display: flex; align-items: center; gap: 0.4rem; padding: 0 0.5rem 0 0.3rem; border-left: 2px solid transparent; background: var(--rowbg); }
  li:hover { --rowbg: var(--bg-hover); }
  li.active { --rowbg: color-mix(in srgb, var(--accent) 10%, var(--bg-rail)); border-left-color: var(--accent-conversor); }
  li.lifted { --rowbg: var(--bg-elevated); position: relative; z-index: 5; pointer-events: none; opacity: 0.92; box-shadow: var(--shadow-pop); }
  li.target { --rowbg: color-mix(in srgb, var(--accent) 18%, var(--bg-rail)); }
  .grip { cursor: grab; touch-action: none; user-select: none; color: var(--text-faint); padding: 0.4rem 0.15rem; line-height: 1; opacity: 0; }
  li:hover .grip, li.active .grip { opacity: 1; }
  .row { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.6rem; background: none; border: none; color: var(--text-muted); padding: 0.55rem 0; text-align: left; }
  li.active .row { color: var(--text); }
  .info { display: flex; flex-direction: column; min-width: 0; }
  .nome { color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.86rem; }
  .meta { font-size: 0.74rem; }
  /* As accións flotan sobre o final da fila: o texto non se reordena ao pasar o rato. */
  .acc { display: none; position: absolute; right: 0.4rem; top: 50%; transform: translateY(-50%); align-items: center; padding-left: 1.6rem; background: linear-gradient(90deg, transparent, var(--rowbg) 1.6rem); }
  li:hover .acc { display: flex; }
  /* O degradado das frechas non pode comer os clics: só os botóns reciben o punteiro */
  .acc { pointer-events: none; }
  .acc > :global(*) { pointer-events: auto; }
  .mini { background: none; border: none; color: var(--text-faint); width: 26px; height: 26px; align-items: center; justify-content: center; border-radius: var(--radius-sm); display: flex; }
  .mini:hover:not(:disabled) { color: var(--text); background: var(--bg-card); }
  .mini:disabled { opacity: 0.25; cursor: default; }
  .hact { display: flex; align-items: center; gap: 0.35rem; }
  .sel { display: flex; align-items: center; justify-content: space-between; padding: 0.25rem 0.9rem; background: color-mix(in srgb, var(--accent) 8%, var(--bg-rail)); border-bottom: 1px solid var(--border); font-size: 0.8rem; }
  .sc { font-weight: 600; }
  .ib { display: grid; place-items: center; width: 28px; height: 28px; padding: 0; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-muted); }
  .ib:hover:not(:disabled) { background: var(--bg-hover); color: var(--text); }
  .ib.danger:hover:not(:disabled) { color: var(--accent-video); }
  .ib:disabled { opacity: 0.35; cursor: default; }
  input[type="checkbox"] { flex-shrink: 0; }
  .ctxmenu { position: fixed; z-index: 70; min-width: 12.5rem; padding: 0.25rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); display: flex; flex-direction: column; }
  .ctxmenu button { display: flex; align-items: center; gap: 0.6rem; background: none; border: none; color: var(--text); text-align: left; padding: 0.45rem 0.6rem; border-radius: 5px; font-size: 0.86rem; }
  .ctxmenu button:hover { background: var(--bg-hover); }
  .ctxmenu hr { border: none; border-top: 1px solid var(--border); margin: 0.25rem 0; }
</style>
