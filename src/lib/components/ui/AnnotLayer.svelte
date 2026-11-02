<script lang="ts">
  import { tick, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { annoTool, annoStyle, shapes, selected, commit, newId, boundsOf, centerOf, hit, moved, scaled, arrowHead, fontOf, lineH, undo, redo, type Shape } from "../../annot";

  /** Caixa da imaxe dentro do visor (px da pantalla), escala (pantalla / píxeles da imaxe) e tamaño da imaxe. */
  export let box: { l: number; t: number; w: number; h: number };
  export let sc: number;
  export let W: number;
  export let H: number;

  let svg: SVGSVGElement;
  let draft: Shape | null = null;
  let drag: { id: number; sx: number; sy: number; orig: Shape; before: Shape[] } | null = null;
  let erasing = false;
  let eraseKeep: Shape[] = [];
  let eraseBefore: Shape[] = [];
  let editBefore: Shape[] | null = null;
  let editing: Shape | null = null;
  // Escalar (esquina) e xirar (círculo de arriba) a figura escollida, como nos textos do PDF.
  let scaling: { id: number; orig: Shape; before: Shape[]; w0: number; ux: number; uy: number; sx: number; sy: number } | null = null;
  let rotating: { id: number; orig: Shape; before: Shape[]; cx: number; cy: number; a0: number } | null = null;
  const imgPt = (e: PointerEvent): [number, number] => {
    const r = svg.getBoundingClientRect();
    return [(e.clientX - r.left) / sc, (e.clientY - r.top) / sc];
  };
  function startScale(e: PointerEvent, s: Shape) {
    if (e.button !== 0) return;
    e.stopPropagation();
    e.preventDefault();
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    const b = boundsOf(s);
    const a = ((s.angle ?? 0) * Math.PI) / 180;
    scaling = { id: s.id, orig: s, before: get(shapes), w0: Math.max(4, b.r - b.l), ux: Math.cos(a), uy: Math.sin(a), sx: e.clientX, sy: e.clientY };
  }
  function startRotate(e: PointerEvent, s: Shape) {
    if (e.button !== 0) return;
    e.stopPropagation();
    e.preventDefault();
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    const [cx, cy] = centerOf(s);
    const [px, py] = imgPt(e);
    rotating = { id: s.id, orig: s, before: get(shapes), cx, cy, a0: Math.atan2(py - cy, px - cx) };
  }
  let editor: HTMLTextAreaElement | undefined;

  const pt = (e: PointerEvent): [number, number] => {
    const r = svg.getBoundingClientRect();
    return [Math.max(0, Math.min(W, (e.clientX - r.left) / sc)), Math.max(0, Math.min(H, (e.clientY - r.top) / sc))];
  };
  const tol = () => 6 / Math.max(sc, 0.05);

  function topHit(x: number, y: number, t = tol()): Shape | null {
    const list = get(shapes);
    for (let i = list.length - 1; i >= 0; i--) if (hit(list[i], x, y, t)) return list[i];
    return null;
  }

  function down(e: PointerEvent) {
    if (e.button !== 0) return; // o botón central deixa o paso ao visor para mover a imaxe
    if (editing) return finishText();
    const [x, y] = pt(e);
    const tool = get(annoTool);
    const st = get(annoStyle);
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    e.stopPropagation();
    e.preventDefault(); // sen isto o clic quitaba o foco ao cadro de texto recén aberto e o texto desaparecía
    if (tool === "select") {
      const s = topHit(x, y);
      selected.set(s?.id ?? null);
      if (s) drag = { id: s.id, sx: x, sy: y, orig: s, before: get(shapes) };
      return;
    }
    if (tool === "eraser") {
      erasing = true;
      eraseBefore = get(shapes);
      eraseKeep = eraseBefore;
      eraseAt(x, y);
      return;
    }
    selected.set(null);
    if (tool === "text") {
      const s: Shape = { id: newId(), type: "text", color: st.color, width: 1, x, y, text: "", size: st.size, family: st.family, bold: st.bold, italic: st.italic };
      startEdit(s);
      return;
    }
    if (tool === "pen" || tool === "highlight") {
      draft = { id: newId(), type: tool, color: st.color, width: st.width, pts: [[x, y]] };
    } else {
      draft = { id: newId(), type: tool, color: st.color, width: st.width, fill: st.fill && (tool === "rect" || tool === "ellipse"), x1: x, y1: y, x2: x, y2: y };
    }
  }

  function eraseAt(x: number, y: number) {
    const keep = eraseKeep.filter((s) => !hit(s, x, y, tol() * 1.5));
    if (keep.length !== eraseKeep.length) {
      eraseKeep = keep;
      shapes.set(keep);
    }
  }

  function move(e: PointerEvent) {
    if (scaling) {
      const d = ((e.clientX - scaling.sx) * scaling.ux + (e.clientY - scaling.sy) * scaling.uy) / sc;
      const k = Math.max(0.1, (scaling.w0 + d) / scaling.w0);
      const next = scaled(scaling.orig, k);
      shapes.update((l) => l.map((x) => (x.id === scaling!.id ? next : x)));
      return;
    }
    if (rotating) {
      const [px, py] = imgPt(e);
      let a = (rotating.orig.angle ?? 0) + ((Math.atan2(py - rotating.cy, px - rotating.cx) - rotating.a0) * 180) / Math.PI;
      if (e.shiftKey) a = Math.round(a / 15) * 15;
      a = Math.round((((a + 180) % 360) + 360) % 360 - 180);
      shapes.update((l) => l.map((x) => (x.id === rotating!.id ? { ...x, angle: a } : x)));
      return;
    }
    const [x, y] = pt(e);
    if (erasing) return eraseAt(x, y);
    if (drag) {
      const m = moved(drag.orig, x - drag.sx, y - drag.sy);
      shapes.update((l) => l.map((s) => (s.id === drag!.id ? m : s)));
      return;
    }
    if (!draft) return;
    if (draft.pts) {
      const last = draft.pts[draft.pts.length - 1];
      if (Math.hypot(x - last[0], y - last[1]) * sc > 1.5) draft = { ...draft, pts: [...draft.pts, [x, y]] };
    } else {
      let x2 = x;
      let y2 = y;
      if (e.shiftKey) {
        const dx = x - (draft.x1 ?? 0);
        const dy = y - (draft.y1 ?? 0);
        if (draft.type === "rect" || draft.type === "ellipse") {
          const m = Math.max(Math.abs(dx), Math.abs(dy));
          x2 = (draft.x1 ?? 0) + Math.sign(dx || 1) * m;
          y2 = (draft.y1 ?? 0) + Math.sign(dy || 1) * m;
        } else if (Math.abs(dx) > 0 || Math.abs(dy) > 0) {
          // liñas e frechas a múltiplos de 45°
          const a = Math.round(Math.atan2(dy, dx) / (Math.PI / 4)) * (Math.PI / 4);
          const len = Math.hypot(dx, dy);
          x2 = (draft.x1 ?? 0) + len * Math.cos(a);
          y2 = (draft.y1 ?? 0) + len * Math.sin(a);
        }
      }
      draft = { ...draft, x2, y2 };
    }
  }

  function up() {
    if (scaling || rotating) {
      const before = (scaling ?? rotating)!.before;
      const final = get(shapes);
      scaling = null;
      rotating = null;
      if (JSON.stringify(final) !== JSON.stringify(before)) {
        shapes.set(before);
        commit(final);
      }
      return;
    }
    if (erasing) {
      erasing = false;
      // o borrado fíxose sobre a marcha: queda no historial como un só paso
      const final = get(shapes);
      if (final.length !== eraseBefore.length) {
        shapes.set(eraseBefore);
        commit(final);
      }
      return;
    }
    if (drag) {
      const final = get(shapes);
      const { orig, before } = drag;
      drag = null;
      if (final.some((s) => s.id === orig.id && JSON.stringify(s) !== JSON.stringify(orig))) {
        shapes.set(before);
        commit(final);
      }
      return;
    }
    if (!draft) return;
    const d = draft;
    draft = null;
    const tiny = !d.pts && Math.hypot((d.x2 ?? 0) - (d.x1 ?? 0), (d.y2 ?? 0) - (d.y1 ?? 0)) * sc < 3;
    if (tiny) return;
    commit([...get(shapes), d]);
    selected.set(null);
  }

  // ---- texto
  async function startEdit(s: Shape) {
    editing = { ...s };
    selected.set(null);
    await tick();
    setTimeout(() => editor?.focus(), 0);
  }
  function finishText() {
    if (!editing) return;
    const s = editing;
    editing = null;
    if (editBefore) shapes.set(editBefore);
    editBefore = null;
    const list = get(shapes);
    const exists = list.some((x) => x.id === s.id);
    if (!(s.text ?? "").trim()) {
      if (exists) commit(list.filter((x) => x.id !== s.id));
      return;
    }
    commit(exists ? list.map((x) => (x.id === s.id ? s : x)) : [...list, s]);
    // Queda escollido: se agora se toca un axuste do panel (letra, tamaño, xiro...), aplícase a este texto.
    selected.set(s.id);
  }
  function dbl(e: MouseEvent) {
    if (get(annoTool) !== "select") return;
    const r = svg.getBoundingClientRect();
    const s = topHit((e.clientX - r.left) / sc, (e.clientY - r.top) / sc);
    if (s?.type === "text") {
      // sáca o texto do SVG mentres se edita
      editBefore = get(shapes);
      shapes.set(editBefore.filter((x) => x.id !== s.id));
      startEdit(s);
    }
  }
  function editKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === "Escape" || (e.key === "Enter" && (e.ctrlKey || e.metaKey))) {
      e.preventDefault();
      finishText();
    }
  }

  // ---- teclado: Supr borra o seleccionado; Ctrl+Z / Ctrl+Y desfán e refán
  function key(e: KeyboardEvent) {
    if ((e.target as HTMLElement).closest("input, textarea, select, [contenteditable]")) return;
    const sel = get(selected);
    if ((e.key === "Delete" || e.key === "Backspace") && sel !== null) {
      commit(get(shapes).filter((s) => s.id !== sel));
      selected.set(null);
      e.preventDefault();
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
      e.shiftKey ? redo() : undo();
      e.preventDefault();
      e.stopPropagation();
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "y") {
      redo();
      e.preventDefault();
    }
  }
  onDestroy(() => (editing = null));

  $: sel = $shapes.find((s) => s.id === $selected) ?? null;
  $: selBox = sel ? boundsOf(sel) : null;
  $: cursor = $annoTool === "select" ? "default" : $annoTool === "eraser" ? "cell" : $annoTool === "text" ? "text" : "crosshair";
  $: items = draft ? [...$shapes, draft] : $shapes;
  const pathOf = (pts: [number, number][]) => pts.map(([x, y], i) => `${i ? "L" : "M"}${x} ${y}`).join(" ");
</script>

<svelte:window on:keydown={key} />

<div class="wrap" style={`left:${box.l}px;top:${box.t}px;width:${box.w}px;height:${box.h}px`}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <svg
    bind:this={svg}
    viewBox={`0 0 ${W} ${H}`}
    style={`cursor:${cursor}`}
    on:pointerdown={down}
    on:pointermove={move}
    on:pointerup={up}
    on:pointercancel={up}
    on:dblclick={dbl}
  >
    {#each items as s (s.id)}
      <g transform={s.angle ? `rotate(${s.angle} ${centerOf(s)[0]} ${centerOf(s)[1]})` : undefined}>
      {#if s.type === "pen" || s.type === "highlight"}
        {#if (s.pts ?? []).length === 1}
          <circle cx={s.pts?.[0][0]} cy={s.pts?.[0][1]} r={(s.type === "highlight" ? s.width * 3 : s.width) / 2} fill={s.color} opacity={s.type === "highlight" ? 0.38 : 1} />
        {:else}
          <path d={pathOf(s.pts ?? [])} fill="none" stroke={s.color} stroke-width={s.type === "highlight" ? s.width * 3 : s.width} stroke-linecap={s.type === "highlight" ? "butt" : "round"} stroke-linejoin="round" opacity={s.type === "highlight" ? 0.38 : 1} />
        {/if}
      {:else if s.type === "line" || s.type === "arrow"}
        <line x1={s.x1} y1={s.y1} x2={s.x2} y2={s.y2} stroke={s.color} stroke-width={s.width} stroke-linecap="round" />
        {#if s.type === "arrow"}
          {@const hd = arrowHead(s)}
          <polyline points={`${hd[0][0]},${hd[0][1]} ${s.x2},${s.y2} ${hd[1][0]},${hd[1][1]}`} fill="none" stroke={s.color} stroke-width={s.width} stroke-linecap="round" stroke-linejoin="round" />
        {/if}
      {:else if s.type === "rect"}
        <rect x={Math.min(s.x1 ?? 0, s.x2 ?? 0)} y={Math.min(s.y1 ?? 0, s.y2 ?? 0)} width={Math.abs((s.x2 ?? 0) - (s.x1 ?? 0))} height={Math.abs((s.y2 ?? 0) - (s.y1 ?? 0))} fill={s.fill ? s.color : "none"} stroke={s.fill ? "none" : s.color} stroke-width={s.width} stroke-linejoin="round" />
      {:else if s.type === "ellipse"}
        <ellipse cx={((s.x1 ?? 0) + (s.x2 ?? 0)) / 2} cy={((s.y1 ?? 0) + (s.y2 ?? 0)) / 2} rx={Math.max(1, Math.abs((s.x2 ?? 0) - (s.x1 ?? 0)) / 2)} ry={Math.max(1, Math.abs((s.y2 ?? 0) - (s.y1 ?? 0)) / 2)} fill={s.fill ? s.color : "none"} stroke={s.fill ? "none" : s.color} stroke-width={s.width} />
      {:else if s.type === "text"}
        <text x={s.x} y={s.y} fill={s.color} style={`font:${fontOf(s)}`} dominant-baseline="text-before-edge">
          {#each (s.text ?? "").split("\n") as line, i}<tspan x={s.x} dy={i ? lineH(s) : 0}>{line || " "}</tspan>{/each}
        </text>
      {/if}
      </g>
    {/each}
    {#if sel && selBox && $annoTool === "select"}
      <g transform={sel.angle ? `rotate(${sel.angle} ${(selBox.l + selBox.r) / 2} ${(selBox.t + selBox.b) / 2})` : undefined}>
        <rect class="sel" x={selBox.l} y={selBox.t} width={selBox.r - selBox.l} height={selBox.b - selBox.t} vector-effect="non-scaling-stroke" />
        <line class="stem" x1={(selBox.l + selBox.r) / 2} y1={selBox.t} x2={(selBox.l + selBox.r) / 2} y2={selBox.t - 26 / sc} vector-effect="non-scaling-stroke" />
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <circle class="rot" cx={(selBox.l + selBox.r) / 2} cy={selBox.t - 30 / sc} r={7 / sc} on:pointerdown={(e) => startRotate(e, sel)} />
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <rect class="grip" x={selBox.r - 7 / sc} y={selBox.b - 7 / sc} width={14 / sc} height={14 / sc} on:pointerdown={(e) => startScale(e, sel)} />
      </g>
    {:else if selBox && sel}
      <rect class="sel" x={selBox.l} y={selBox.t} width={selBox.r - selBox.l} height={selBox.b - selBox.t} vector-effect="non-scaling-stroke" transform={sel.angle ? `rotate(${sel.angle} ${(selBox.l + selBox.r) / 2} ${(selBox.t + selBox.b) / 2})` : undefined} />
    {/if}
  </svg>
  {#if editing}
    <textarea
      bind:this={editor}
      bind:value={editing.text}
      class="ed"
      style={`left:${(editing.x ?? 0) * sc}px;top:${(editing.y ?? 0) * sc}px;font:${fontOf({ ...editing, size: (editing.size ?? 32) * sc })};color:${editing.color};line-height:1.2`}
      rows="1"
      spellcheck="false"
      on:keydown={editKey}
      on:blur={finishText}
      on:input={(e) => ((e.currentTarget.style.height = "auto"), (e.currentTarget.style.height = `${e.currentTarget.scrollHeight}px`))}
    ></textarea>
  {/if}
</div>

<style>
  .wrap { position: absolute; pointer-events: none; overflow: visible; }
  svg { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: auto; touch-action: none; overflow: hidden; }
  .stem { stroke: #4aa3ff; stroke-width: 1.5px; pointer-events: none; }
  .rot { fill: #4aa3ff; stroke: #fff; stroke-width: 2px; vector-effect: non-scaling-stroke; cursor: grab; }
  .grip { fill: #fff; stroke: #4aa3ff; stroke-width: 2px; vector-effect: non-scaling-stroke; cursor: nwse-resize; }
  .sel { fill: none; stroke: #4aa3ff; stroke-width: 1.5px; stroke-dasharray: 5 3; pointer-events: none; }
  .ed { position: absolute; min-width: 4ch; max-width: 100%; padding: 0; margin: 0; background: rgba(255, 255, 255, 0.08); border: 1px dashed #4aa3ff; outline: none; resize: none; overflow: hidden; white-space: pre; pointer-events: auto; }
</style>
