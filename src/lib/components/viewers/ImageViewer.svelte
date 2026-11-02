<script lang="ts">
  import { onMount, tick } from "svelte";
  import { viewUrl, imageEdit, imageCrop, imageCropAspect, imageCropEnabled, imageUrl, type FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { t } from "../../i18n";
  import Slider from "../ui/Slider.svelte";
  import AnnotLayer from "../ui/AnnotLayer.svelte";
  import { annoActive } from "../../annot";
  import { actionFor } from "../../keymap";

  export let file: FileEntry;

  let url = "";
  let failed = false;
  let zoom = 100;
  let fit = true;
  let stage: HTMLDivElement;
  let imgEl: HTMLImageElement | null = null;
  let box = { l: 0, t: 0, w: 0, h: 0 };
  let ro: ResizeObserver;
  let natural = { w: 0, h: 0 };

  $: load(file.path);
  $: imageUrl.set(url);
  $: e = $imageEdit;
  // Co recorte activo móstrase a imaxe sen xirar: o recorte aplícase antes do xiro.
  $: cropping = !!$imageCrop;
  // `clip-path` recorta o desenfoque aos límites da imaxe (sen el, o borrón sae cara a fóra).
  $: preview = e
    ? `${cropping ? "" : `transform: rotate(${e.rotate}deg) scale(${e.flipH ? -1 : 1}, ${e.flipV ? -1 : 1});`} filter: brightness(${1 + e.brightness / 100}) contrast(${1 + e.contrast / 100}) grayscale(${e.grayscale ? 1 : 0}) invert(${e.invert ? 1 : 0}) blur(${e.blur}px); clip-path: inset(0);`
    : "";

  let triedPreview = false;

  async function load(path: string) {
    failed = false;
    triedPreview = false;
    fit = true;
    zoom = 100;
    natural = { w: 0, h: 0 };
    url = await viewUrl(path);
  }

  // O navegador non le TIFF, TGA, QOI, PPM...: o trasno fai un PNG temporal para velo.
  async function onError() {
    if (triedPreview) {
      failed = true;
      return;
    }
    triedPreview = true;
    try {
      url = await viewUrl(await engine.makePreview(file.path));
    } catch {
      failed = true;
    }
  }

  const MIN = 8;
  const W = () => natural.w || file.width || imgEl?.naturalWidth || 1;
  const H = () => natural.h || file.height || imgEl?.naturalHeight || 1;
  $: sc = box.w && W() ? box.w / W() : 1;
  /** Zoom que se ve agora (en %), tamén co axuste á xanela. */
  $: shown = fit ? Math.round(sc * 100) : zoom;

  function measure() {
    if (!imgEl || !stage) return;
    const r = imgEl.getBoundingClientRect();
    const s = stage.getBoundingClientRect();
    box = { l: r.left - s.left + stage.scrollLeft, t: r.top - s.top + stage.scrollTop, w: r.width, h: r.height };
  }
  function loaded() {
    if (imgEl) natural = { w: imgEl.naturalWidth, h: imgEl.naturalHeight };
    measure();
  }
  $: if (imgEl && (fit || zoom || url || $imageCrop)) tick().then(measure);

  onMount(() => {
    ro = new ResizeObserver(measure);
    ro.observe(stage);
    return () => ro.disconnect();
  });
  $: if (ro && imgEl) ro.observe(imgEl);

  const clamp = (v: number, a: number, b: number) => Math.min(b, Math.max(a, v));
  const ZMIN = 5;
  const ZMAX = 800;

  // ---- zoom co rato (centrado no punteiro) e mover coa roda ou o botón central
  async function zoomAt(nz: number, px: number, py: number) {
    if (!imgEl) return;
    measure();
    const fx = box.w ? clamp((px + stage.scrollLeft - box.l) / box.w, 0, 1) : 0.5;
    const fy = box.h ? clamp((py + stage.scrollTop - box.t) / box.h, 0, 1) : 0.5;
    fit = false;
    zoom = clamp(Math.round(nz), ZMIN, ZMAX);
    await tick();
    measure();
    stage.scrollLeft = box.l + fx * box.w - px;
    stage.scrollTop = box.t + fy * box.h - py;
  }
  function wheel(ev: WheelEvent) {
    if (!imgEl || failed) return;
    ev.preventDefault();
    const r = stage.getBoundingClientRect();
    const cur = fit ? sc * 100 : zoom;
    zoomAt(cur * (ev.deltaY < 0 ? 1.15 : 1 / 1.15), ev.clientX - r.left, ev.clientY - r.top);
  }

  type Drag = { handle: string; sx: number; sy: number; c: { x: number; y: number; w: number; h: number } };
  let drag: Drag | null = null;
  let panning: { x: number; y: number; sl: number; st: number } | null = null;
  let creating: { x: number; y: number } | null = null;

  /** Punto do ratón en píxeles da imaxe. */
  const toImg = (ev: PointerEvent) => ({ x: clamp((ev.clientX - stage.getBoundingClientRect().left + stage.scrollLeft - box.l) / sc, 0, W()), y: clamp((ev.clientY - stage.getBoundingClientRect().top + stage.scrollTop - box.t) / sc, 0, H()) });

  function stageDown(ev: PointerEvent) {
    if (failed || !imgEl) return;
    const middle = ev.button === 1;
    if (!middle && ev.button !== 0) return;
    if (ev.button === 0 && $annoActive) return; // o debuxo ten a súa propia capa
    if (ev.button === 0 && $imageCropEnabled) {
      // arrastrar sobre a imaxe debuxa un recorte novo
      const p = toImg(ev);
      creating = p;
      imageCrop.set({ x: Math.round(p.x), y: Math.round(p.y), w: 1, h: 1 });
      stage.setPointerCapture(ev.pointerId);
      ev.preventDefault();
      return;
    }
    panning = { x: ev.clientX, y: ev.clientY, sl: stage.scrollLeft, st: stage.scrollTop };
    stage.setPointerCapture(ev.pointerId);
    ev.preventDefault();
  }
  function stageMove(ev: PointerEvent) {
    if (creating) {
      const p = toImg(ev);
      let l = Math.min(creating.x, p.x);
      let t = Math.min(creating.y, p.y);
      let w = Math.abs(p.x - creating.x);
      let h = Math.abs(p.y - creating.y);
      const ar = $imageCropAspect;
      if (ar) {
        h = w / ar;
        if (p.y < creating.y) t = creating.y - h;
        if (t < 0) {
          t = 0;
          h = creating.y;
          w = h * ar;
          if (p.x < creating.x) l = creating.x - w;
        }
        if (t + h > H()) {
          h = H() - t;
          w = h * ar;
          if (p.x < creating.x) l = creating.x - w;
        }
      }
      imageCrop.set({ x: Math.round(l), y: Math.round(t), w: Math.max(1, Math.round(w)), h: Math.max(1, Math.round(h)) });
      return;
    }
    if (panning) {
      stage.scrollLeft = panning.sl - (ev.clientX - panning.x);
      stage.scrollTop = panning.st - (ev.clientY - panning.y);
    }
  }
  function stageUp() {
    if (creating) {
      creating = null;
      // un simple clic non crea un recorte: se quedou diminuto, descártase
      const c = $imageCrop;
      if (c && (c.w < MIN || c.h < MIN)) imageCrop.set(null);
    }
    panning = null;
  }

  function down(ev: PointerEvent, handle: string) {
    if (ev.button !== 0 || !$imageCrop) return;
    drag = { handle, sx: ev.clientX, sy: ev.clientY, c: { ...$imageCrop } };
    (ev.currentTarget as HTMLElement).setPointerCapture(ev.pointerId);
    ev.stopPropagation();
  }

  function moveEv(ev: PointerEvent) {
    if (!drag) return;
    const dx = (ev.clientX - drag.sx) / sc;
    const dy = (ev.clientY - drag.sy) / sc;
    const { c, handle } = drag;
    const w = W();
    const h = H();
    if (handle === "move") {
      imageCrop.set({ ...c, x: Math.round(clamp(c.x + dx, 0, w - c.w)), y: Math.round(clamp(c.y + dy, 0, h - c.h)) });
      return;
    }
    let l = c.x;
    let t = c.y;
    let r = c.x + c.w;
    let b = c.y + c.h;
    if (handle.includes("l")) l = clamp(c.x + dx, 0, r - MIN);
    if (handle.includes("r")) r = clamp(c.x + c.w + dx, l + MIN, w);
    if (handle.includes("t")) t = clamp(c.y + dy, 0, b - MIN);
    if (handle.includes("b")) b = clamp(c.y + c.h + dy, t + MIN, h);
    const ar = $imageCropAspect;
    if (ar && handle.length === 2) {
      let nw = r - l;
      let nh = nw / ar;
      if (handle.includes("t")) t = b - nh;
      else b = t + nh;
      if (t < 0) {
        t = 0;
        nh = b - t;
        nw = nh * ar;
        if (handle.includes("l")) l = r - nw;
        else r = l + nw;
      }
      if (b > h) {
        b = h;
        nh = b - t;
        nw = nh * ar;
        if (handle.includes("l")) l = r - nw;
        else r = l + nw;
      }
    }
    imageCrop.set({ x: Math.round(l), y: Math.round(t), w: Math.max(MIN, Math.round(r - l)), h: Math.max(MIN, Math.round(b - t)) });
  }

  const up = () => (drag = null);

  // Teclado (configurable en Axustes → Atallos).
  function key(e: KeyboardEvent) {
    if ((e.target as HTMLElement).closest("input, select, textarea, [contenteditable]")) return;
    const id = actionFor(e, "image");
    const r = stage?.getBoundingClientRect();
    if (id === "i.zoomIn") zoomAt((fit ? sc * 100 : zoom) * 1.25, (r?.width ?? 0) / 2, (r?.height ?? 0) / 2);
    else if (id === "i.zoomOut") zoomAt((fit ? sc * 100 : zoom) / 1.25, (r?.width ?? 0) / 2, (r?.height ?? 0) / 2);
    else if (id === "i.fit") fit = true;
    else if (id === "i.actual") zoomAt(100, (r?.width ?? 0) / 2, (r?.height ?? 0) / 2);
    else return;
    e.preventDefault();
  }
  const toggleFit = () => {
    if (!imgEl || $imageCropEnabled || $annoActive) return;
    if (fit) {
      const r = stage.getBoundingClientRect();
      zoomAt(100, r.width / 2, r.height / 2);
    } else fit = true;
  };

  const HANDLES = ["tl", "t", "tr", "r", "br", "b", "bl", "l"];
  const pos = (h: string) => `left:${h.includes("l") ? 0 : h.includes("r") ? 100 : 50}%;top:${h.includes("t") ? 0 : h.includes("b") ? 100 : 50}%;cursor:${{ tl: "nwse", br: "nwse", tr: "nesw", bl: "nesw", t: "ns", b: "ns", l: "ew", r: "ew" }[h]}-resize`;
</script>

<svelte:window on:keydown={key} />

<div class="wrap">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="stage scrollbar-thin"
    class:fit
    class:pan={!$imageCropEnabled && !$annoActive && !!imgEl}
    class:panning={!!panning}
    class:aim={$imageCropEnabled}
    bind:this={stage}
    on:wheel={wheel}
    on:pointerdown={stageDown}
    on:pointermove={stageMove}
    on:pointerup={stageUp}
    on:pointercancel={stageUp}
    on:dblclick={toggleFit}
    on:auxclick|preventDefault
  >
    {#if failed}
      <div class="msg">
        <p class="muted">{$t("O trasno non puido preparar unha vista previa de {ext}.", { ext: file.ext.toUpperCase() })}</p>
        <button class="btn" on:click={() => load(file.path)}>{$t("Tentar de novo")}</button>
      </div>
    {:else if url}
      <div class="canvas">
        <img
          bind:this={imgEl}
          src={url}
          alt={file.name}
          style={(fit ? "" : `width:${(W() * zoom) / 100}px;`) + preview}
          on:error={onError}
          on:load={loaded}
          draggable="false"
        />
      </div>
      {#if $annoActive && box.w}
        <AnnotLayer {box} {sc} W={W()} H={H()} />
      {/if}
      {#if $imageCrop && box.w}
        <div class="layer" style={`left:${box.l}px;top:${box.t}px;width:${box.w}px;height:${box.h}px`}>
          <div
            class="crop"
            style={`left:${$imageCrop.x * sc}px;top:${$imageCrop.y * sc}px;width:${$imageCrop.w * sc}px;height:${$imageCrop.h * sc}px`}
            on:pointerdown={(ev) => down(ev, "move")}
            on:pointermove={moveEv}
            on:pointerup={up}
            role="presentation"
          >
            <span class="third v1"></span><span class="third v2"></span><span class="third h1"></span><span class="third h2"></span>
            {#each HANDLES as h}
              <span
                class="handle"
                style={pos(h)}
                on:pointerdown={(ev) => down(ev, h)}
                on:pointermove={moveEv}
                on:pointerup={up}
                role="presentation"
              ></span>
            {/each}
            <span class="size">{$imageCrop.w} × {$imageCrop.h}</span>
          </div>
        </div>
      {/if}
    {/if}
  </div>
  <div class="bar">
    <button class="btn" class:on={fit} on:click={() => (fit = true)}>{$t("Axustar")}</button>
    <button class="btn" class:on={!fit && zoom === 100} on:click={() => zoomAt(100, stage.clientWidth / 2, stage.clientHeight / 2)}>100%</button>
    <Slider compact label={$t("Zoom")} unit="%" min={ZMIN} max={ZMAX} value={shown} on:input={(ev) => zoomAt(Number((ev.target as HTMLInputElement).value) || shown, stage.clientWidth / 2, stage.clientHeight / 2)} />
    <span class="muted hint">{$t("Roda: zoom · Botón central: mover")}</span>
    {#if file.width}<span class="muted dims">{file.width} × {file.height} px</span>{/if}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .stage {
    position: relative;
    flex: 1; min-height: 0; overflow: auto;
    background: repeating-conic-gradient(var(--checker-a) 0% 25%, var(--checker-b) 0% 50%) 0 0 / 20px 20px;
    touch-action: none;
  }
  /* Marxe arredor da imaxe: os tiradores do recorte nunca quedan baixo as frechas dos paneis. */
  .canvas { box-sizing: border-box; display: flex; align-items: center; justify-content: center; padding: 26px; min-width: 100%; min-height: 100%; width: max-content; height: max-content; }
  .stage.fit .canvas { width: 100%; height: 100%; }
  .stage.fit img { max-width: 100%; max-height: 100%; object-fit: contain; }
  img { display: block; user-select: none; flex: none; }
  .stage.fit img { flex: 0 1 auto; }
  .stage.pan { cursor: grab; }
  .stage.panning { cursor: grabbing; }
  .stage.aim { cursor: crosshair; }
  .layer { position: absolute; overflow: hidden; pointer-events: none; }
  .crop { position: absolute; box-shadow: 0 0 0 9999px rgba(8, 8, 12, 0.62); outline: 1.5px solid #fff; cursor: move; pointer-events: auto; touch-action: none; }
  .third { position: absolute; background: rgba(255, 255, 255, 0.35); pointer-events: none; }
  .v1, .v2 { top: 0; bottom: 0; width: 1px; }
  .h1, .h2 { left: 0; right: 0; height: 1px; }
  .v1 { left: 33.33%; } .v2 { left: 66.66%; } .h1 { top: 33.33%; } .h2 { top: 66.66%; }
  .handle { position: absolute; width: 12px; height: 12px; margin: -6px 0 0 -6px; background: #fff; border: 1.5px solid #111; border-radius: 2px; touch-action: none; }
  .size { position: absolute; left: 50%; bottom: 100%; transform: translate(-50%, -6px); background: rgba(0, 0, 0, 0.75); color: #fff; font-size: 0.72rem; padding: 2px 6px; border-radius: 4px; white-space: nowrap; pointer-events: none; }
  .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.6rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .btn.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .hint { font-size: 0.76rem; }
  .dims { margin-left: auto; }
  .msg { margin: auto; text-align: center; padding: 2rem; }
</style>
