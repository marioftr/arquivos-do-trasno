<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { viewUrl, imageEdit, imageCrop, imageCropAspect, type FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { t } from "../../i18n";
  import Slider from "../ui/Slider.svelte";

  export let file: FileEntry;

  let url = "";
  let failed = false;
  let zoom = 100;
  let fit = true;
  let stage: HTMLDivElement;
  let imgEl: HTMLImageElement | null = null;
  let box = { l: 0, t: 0, w: 0, h: 0 };
  let ro: ResizeObserver;

  $: load(file.path);
  $: e = $imageEdit;
  // Co recorte activo móstrase a imaxe sen xirar: o recorte aplícase antes do xiro.
  $: cropping = !!$imageCrop;
  $: preview = e
    ? `${cropping ? "" : `transform: rotate(${e.rotate}deg) scale(${e.flipH ? -1 : 1}, ${e.flipV ? -1 : 1});`} filter: brightness(${1 + e.brightness / 100}) contrast(${1 + e.contrast / 100}) grayscale(${e.grayscale ? 1 : 0}) invert(${e.invert ? 1 : 0}) blur(${e.blur}px);`
    : "";

  let triedPreview = false;

  async function load(path: string) {
    failed = false;
    triedPreview = false;
    fit = true;
    zoom = 100;
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

  // ---- recorte visual ----
  const MIN = 8;
  const W = () => file.width ?? imgEl?.naturalWidth ?? 1;
  const H = () => file.height ?? imgEl?.naturalHeight ?? 1;
  $: sc = box.w && W() ? box.w / W() : 1;

  function measure() {
    if (!imgEl || !stage) return;
    const r = imgEl.getBoundingClientRect();
    const s = stage.getBoundingClientRect();
    box = { l: r.left - s.left + stage.scrollLeft, t: r.top - s.top + stage.scrollTop, w: r.width, h: r.height };
  }
  $: if (imgEl && (fit || zoom || url || $imageCrop)) tick().then(measure);

  onMount(() => {
    ro = new ResizeObserver(measure);
    ro.observe(stage);
    return () => ro.disconnect();
  });
  $: if (ro && imgEl) ro.observe(imgEl);

  type Drag = { handle: string; sx: number; sy: number; c: { x: number; y: number; w: number; h: number } };
  let drag: Drag | null = null;
  const clamp = (v: number, a: number, b: number) => Math.min(b, Math.max(a, v));

  function down(ev: PointerEvent, handle: string) {
    if (!$imageCrop) return;
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

  const HANDLES = ["tl", "t", "tr", "r", "br", "b", "bl", "l"];
  const pos = (h: string) => `left:${h.includes("l") ? 0 : h.includes("r") ? 100 : 50}%;top:${h.includes("t") ? 0 : h.includes("b") ? 100 : 50}%;cursor:${{ tl: "nwse", br: "nwse", tr: "nesw", bl: "nesw", t: "ns", b: "ns", l: "ew", r: "ew" }[h]}-resize`;
</script>

<div class="wrap">
  <div class="stage scrollbar-thin" class:fit bind:this={stage}>
    {#if failed}
      <p class="muted msg">{$t("O trasno non puido preparar unha vista previa de {ext}.", { ext: file.ext.toUpperCase() })}</p>
    {:else if url}
      <img
        bind:this={imgEl}
        src={url}
        alt={file.name}
        style={(fit ? "" : `width:${((file.width ?? 0) * zoom) / 100}px;`) + preview}
        on:error={onError}
        on:load={measure}
        draggable="false"
      />
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
    <button class="btn" class:on={!fit && zoom === 100} on:click={() => { fit = false; zoom = 100; }}>100%</button>
    <Slider compact label={$t("Zoom")} unit="%" min={10} max={400} bind:value={zoom} on:input={() => (fit = false)} />
    {#if file.width}<span class="muted dims">{file.width} × {file.height} px</span>{/if}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .stage {
    position: relative;
    flex: 1; min-height: 0; overflow: auto;
    display: flex; align-items: center; justify-content: center;
    background: repeating-conic-gradient(#17171a 0% 25%, #1b1b1f 0% 50%) 0 0 / 20px 20px;
  }
  .stage.fit img { max-width: 100%; max-height: 100%; object-fit: contain; }
  .stage:not(.fit) { display: block; }
  img { display: block; margin: auto; user-select: none; }
  .layer { position: absolute; overflow: hidden; pointer-events: none; }
  .crop { position: absolute; box-shadow: 0 0 0 9999px rgba(8, 8, 12, 0.62); outline: 1.5px solid #fff; cursor: move; pointer-events: auto; touch-action: none; }
  .third { position: absolute; background: rgba(255, 255, 255, 0.35); pointer-events: none; }
  .v1, .v2 { top: 0; bottom: 0; width: 1px; }
  .h1, .h2 { left: 0; right: 0; height: 1px; }
  .v1 { left: 33.33%; } .v2 { left: 66.66%; } .h1 { top: 33.33%; } .h2 { top: 66.66%; }
  .handle { position: absolute; width: 12px; height: 12px; margin: -6px 0 0 -6px; background: #fff; border: 1.5px solid #111; border-radius: 2px; touch-action: none; }
  .size { position: absolute; left: 50%; bottom: 100%; transform: translate(-50%, -6px); background: rgba(0, 0, 0, 0.75); color: #fff; font-size: 0.72rem; padding: 2px 6px; border-radius: 4px; white-space: nowrap; pointer-events: none; }
  .bar { display: flex; align-items: center; gap: 0.6rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .btn.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .dims { margin-left: auto; }
  .msg { margin: auto; text-align: center; padding: 2rem; }
</style>
