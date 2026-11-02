<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { viewUrl, imageEdit, imageCrop, imageCropAspect, imageCropEnabled, imageUrl, type FileEntry } from "../../files";
  import * as engine from "../../engine";
  import { t } from "../../i18n";
  import AnnotLayer from "../ui/AnnotLayer.svelte";
  import OcrLayer from "../ui/OcrLayer.svelte";
  import Icon from "../Icon.svelte";
  import { lang as appLang } from "../../i18n";
  import { toast } from "../../toasts";
  import { tr } from "../../i18n";
  import { errText } from "../../errors";
  import { OCR_LANGS, OCR_SIZE, ocrInstalled, ocrDownload, ocrProgress, ocrActive, savedLang, rememberLang, refreshOcr, downloadLang, recognize, type OcrResult } from "../../ocr";
  import { annoActive } from "../../annot";
  import { actionFor } from "../../keymap";

  export let file: FileEntry;

  let url = "";
  let failed = false;
  let zoom = 100;
  let fit = true;
  let stage: HTMLDivElement;
  let viewEl: HTMLDivElement | undefined;
  let imgEl: HTMLImageElement | null = null;
  let box = { l: 0, t: 0, w: 0, h: 0 };
  let natural = { w: 0, h: 0 };
  let sw = 0;
  let sh = 0;

  $: load(file.path);
  $: imageUrl.set(url);
  $: e = $imageEdit;

  // ---- tamaño que se ve: a imaxe xa xirada ou volta (o recorte e os debuxos van sobre o que se ve)
  $: rot = (((e?.rotate ?? 0) % 360) + 360) % 360;
  $: swap = rot % 180 !== 0;
  $: nw = natural.w || file.width || 0;
  $: nh = natural.h || file.height || 0;
  $: TW = swap ? nh : nw;
  $: TH = swap ? nw : nh;
  const PAD = 26; // marxe arredor da imaxe: os tiradores do recorte non quedan baixo as frechas dos paneis
  $: fitScale = TW && TH && sw && sh ? Math.min(1, (sw - PAD * 2) / TW, (sh - PAD * 2) / TH) : 1;
  $: eff = fit ? fitScale : zoom / 100;
  $: vw = TW * eff;
  $: vh = TH * eff;
  $: sc = eff;
  $: shown = Math.round(eff * 100);
  // `clip-path` recorta o desenfoque aos límites da imaxe (sen el, o borrón sae cara a fóra).
  $: filt = e ? `filter: brightness(${1 + e.brightness / 100}) contrast(${1 + e.contrast / 100}) grayscale(${e.grayscale ? 1 : 0}) invert(${e.invert ? 1 : 0}) blur(${e.blur}px); clip-path: inset(0);` : "";
  $: imgStyle = `width:${nw * eff}px;height:${nh * eff}px;transform:translate(-50%,-50%) scale(${e?.flipH ? -1 : 1}, ${e?.flipV ? -1 : 1}) rotate(${rot}deg);${filt}`;

  let triedPreview = false;

  // ---- recoñecemento de texto (OCR): ler o texto da imaxe para seleccionalo e copialo
  let ocrOn = false;
  let ocrBusy = false;
  let ocrRes: OcrResult | null = null;
  let ocrErr = "";
  let ocrLang = savedLang($appLang);
  let askDownload = false;
  const ocrCache = new Map<string, OcrResult>();
  onMount(() => void refreshOcr().catch(() => {}));
  onDestroy(() => ocrActive.set(false));
  $: ocrActive.set(ocrOn);
  // Outra imaxe: o texto lido deixa de valer.
  $: if (file.path) {
    ocrOn = false;
    ocrRes = null;
    ocrErr = "";
    askDownload = false;
  }
  async function toggleOcr() {
    if (ocrOn) {
      ocrOn = false;
      return;
    }
    ocrOn = true;
    await runOcr();
  }
  async function runOcr() {
    ocrErr = "";
    if (!$ocrInstalled.includes(ocrLang)) {
      askDownload = true;
      return;
    }
    askDownload = false;
    const key = `${file.path}|${ocrLang}`;
    const cached = ocrCache.get(key);
    if (cached) {
      ocrRes = cached;
      return;
    }
    ocrBusy = true;
    try {
      const res = await recognize(url, ocrLang);
      ocrCache.set(key, res);
      if (ocrOn) ocrRes = res;
      if (!res.lines.length) toast(tr("O trasno non atopou texto nesta imaxe."));
    } catch (e) {
      ocrErr = errText(e);
    } finally {
      ocrBusy = false;
    }
  }
  async function acceptDownload() {
    try {
      await downloadLang(ocrLang);
      await runOcr();
    } catch (e) {
      ocrErr = errText(e);
    }
  }
  function changeLang() {
    rememberLang(ocrLang);
    ocrRes = null;
    if (ocrOn) runOcr();
  }
  async function copyAll() {
    if (!ocrRes) return;
    try {
      await navigator.clipboard.writeText(ocrRes.text);
      toast(tr("O trasno copiou o texto."));
    } catch {
      toast(tr("O trasno non puido copiar o texto."), { kind: "warn" });
    }
  }

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

  function measure() {
    if (!viewEl || !stage) return;
    const r = viewEl.getBoundingClientRect();
    const s = stage.getBoundingClientRect();
    box = { l: r.left - s.left + stage.scrollLeft, t: r.top - s.top + stage.scrollTop, w: r.width, h: r.height };
  }
  function loaded() {
    if (imgEl) natural = { w: imgEl.naturalWidth, h: imgEl.naturalHeight };
    tick().then(measure);
  }
  $: if (viewEl && (vw || vh || url || sw || sh)) tick().then(measure);

  /** Segue o tamaño do visor (para axustar a imaxe) sen depender do CSS. */
  function track(node: HTMLElement) {
    const ro = new ResizeObserver(() => {
      sw = node.clientWidth;
      sh = node.clientHeight;
      measure();
    });
    ro.observe(node);
    return { destroy: () => ro.disconnect() };
  }

  const clamp = (v: number, a: number, b: number) => Math.min(b, Math.max(a, v));
  const ZMIN = 5;
  const ZMAX = 800;

  // ---- zoom co rato (centrado no punteiro) e mover coa roda ou o botón central
  async function zoomAt(nz: number, px: number, py: number) {
    if (!viewEl) return;
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
    if (!viewEl || failed) return;
    ev.preventDefault();
    const r = stage.getBoundingClientRect();
    zoomAt(eff * 100 * (ev.deltaY < 0 ? 1.15 : 1 / 1.15), ev.clientX - r.left, ev.clientY - r.top);
  }

  type Drag = { handle: string; sx: number; sy: number; c: { x: number; y: number; w: number; h: number } };
  let drag: Drag | null = null;
  let panning: { x: number; y: number; sl: number; st: number } | null = null;
  let creating: { x: number; y: number } | null = null;

  /** Punto do ratón en píxeles da imaxe que se ve. */
  const toImg = (ev: PointerEvent) => {
    const s = stage.getBoundingClientRect();
    return { x: clamp((ev.clientX - s.left + stage.scrollLeft - box.l) / sc, 0, TW), y: clamp((ev.clientY - s.top + stage.scrollTop - box.t) / sc, 0, TH) };
  };

  function stageDown(ev: PointerEvent) {
    if (failed || !viewEl) return;
    if ((ev.target as HTMLElement).closest(".ocr span")) return; // sobre o texto lido, arrastrar selecciona
    const middle = ev.button === 1;
    if (!middle && ev.button !== 0) return;
    if (ev.button === 0 && $annoActive) return; // o debuxo ten a súa propia capa
    if (ev.button === 0 && $imageCropEnabled && !ocrOn) {
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
        if (t + h > TH) {
          h = TH - t;
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
    const w = TW;
    const h = TH;
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
      let nw2 = r - l;
      let nh2 = nw2 / ar;
      if (handle.includes("t")) t = b - nh2;
      else b = t + nh2;
      if (t < 0) {
        t = 0;
        nh2 = b - t;
        nw2 = nh2 * ar;
        if (handle.includes("l")) l = r - nw2;
        else r = l + nw2;
      }
      if (b > h) {
        b = h;
        nh2 = b - t;
        nw2 = nh2 * ar;
        if (handle.includes("l")) l = r - nw2;
        else r = l + nw2;
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
    const cx = (r?.width ?? 0) / 2;
    const cy = (r?.height ?? 0) / 2;
    if (id === "i.zoomIn") zoomAt(eff * 125, cx, cy);
    else if (id === "i.zoomOut") zoomAt(eff * 80, cx, cy);
    else if (id === "i.fit") fit = true;
    else if (id === "i.actual") zoomAt(100, cx, cy);
    else return;
    e.preventDefault();
  }
  const toggleFit = (ev?: MouseEvent) => {
    if ((ev?.target as HTMLElement | undefined)?.closest?.(".ocr span")) return;
    if (!viewEl || $imageCropEnabled || $annoActive) return;
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
  <div class="holder">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="stage scrollbar-thin"
      class:pan={!$imageCropEnabled && !$annoActive && !!viewEl}
      class:panning={!!panning}
      class:aim={$imageCropEnabled && !ocrOn}
      bind:this={stage}
      use:track
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
          <div class="view" bind:this={viewEl} style={nw ? `width:${vw}px;height:${vh}px` : "width:1px;height:1px"}>
            <img bind:this={imgEl} src={url} alt={file.name} style={nw ? imgStyle : "opacity:0"} on:error={onError} on:load={loaded} draggable="false" />
          </div>
        </div>
        {#if ocrOn && ocrRes && box.w && !$annoActive}
          <OcrLayer {box} {sc} result={ocrRes} />
        {/if}
        {#if $annoActive && box.w && !ocrOn}
          <AnnotLayer {box} {sc} W={TW} H={TH} />
        {/if}
        {#if $imageCrop && box.w && !ocrOn}
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
    {#if askDownload || $ocrDownload || ocrErr}
      <div class="ocrbar" role="status">
        {#if $ocrDownload}
          <span>{$t("Descargando o idioma…")} {Math.round($ocrDownload.progress * 100)} %</span>
          <div class="pbar"><div class="pfill" style={`width:${Math.round($ocrDownload.progress * 100)}%`}></div></div>
        {:else if askDownload}
          <span>{$t("Para ler texto en {lang} o trasno ten que descargar o seu paquete de idioma (uns {size}, do proxecto Tesseract en GitHub).", { lang: OCR_LANGS.find((l) => l.id === ocrLang)?.label ?? ocrLang, size: OCR_SIZE })}</span>
          <button class="btn primary" on:click={acceptDownload}>{$t("Aceptar e descargar")}</button>
          <button class="btn ghost" on:click={() => ((askDownload = false), (ocrOn = false))}>{$t("Agora non")}</button>
        {:else}
          <span class="err">{ocrErr}</span>
          <button class="btn" on:click={runOcr}>{$t("Tentar de novo")}</button>
        {/if}
      </div>
    {/if}
  </div>
  <div class="bar">
    <button class="btn" class:on={fit} on:click={() => (fit = true)}>{$t("Axustar")}</button>
    <button class="btn" class:on={!fit && zoom === 100} on:click={() => zoomAt(100, stage.clientWidth / 2, stage.clientHeight / 2)}>100%</button>
    <span class="muted pct">{shown} %</span>
    <span class="grow"></span>
    {#if ocrOn}
      <select class="lg" bind:value={ocrLang} on:change={changeLang} aria-label={$t("Idioma do texto")} disabled={ocrBusy}>
        {#each OCR_LANGS as l}<option value={l.id}>{l.label}{$ocrInstalled.includes(l.id) ? "" : " ⇩"}</option>{/each}
      </select>
      <button class="btn" on:click={copyAll} disabled={!ocrRes?.lines.length}><Icon name="copy" size={14} /> {$t("Copiar todo o texto")}</button>
    {/if}
    <button class="btn ocrbtn" class:on={ocrOn} on:click={toggleOcr} disabled={ocrBusy || !!$ocrDownload} title={$t("Le o texto da imaxe para poder seleccionalo e copialo")}><Icon name="type" size={15} /> {ocrBusy ? `${$t("Lendo o texto…")} ${Math.round($ocrProgress * 100)} %` : $t("Texto da imaxe")}</button>
    {#if file.width}<span class="muted dims">{file.width} × {file.height} px</span>{/if}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .holder { position: relative; flex: 1; min-height: 0; display: flex; }
  .stage {
    position: relative;
    flex: 1; min-height: 0; min-width: 0; overflow: auto;
    background: repeating-conic-gradient(var(--checker-a) 0% 25%, var(--checker-b) 0% 50%) 0 0 / 20px 20px;
    touch-action: none;
  }
  .canvas { box-sizing: border-box; display: flex; align-items: center; justify-content: center; padding: 26px; min-width: 100%; min-height: 100%; width: max-content; height: max-content; }
  .view { position: relative; flex: none; }
  img { position: absolute; left: 50%; top: 50%; max-width: none; display: block; user-select: none; transform-origin: center; }
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
  /* Barra de abaixo: unha soa fila fixa; nada cambia de sitio nin de tamaño ao usar o OCR. */
  .bar { display: flex; flex-wrap: nowrap; align-items: center; gap: 0.5rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; overflow: hidden; }
  .bar > * { flex: none; white-space: nowrap; }
  .btn.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .grow { flex: 1 1 auto; min-width: 0; }
  .pct { min-width: 3.2rem; font-variant-numeric: tabular-nums; }
  .ocrbtn { min-width: 11.5rem; justify-content: center; }
  .lg { height: var(--control-h); width: 7.4rem; }
  .dims { min-width: 6.5rem; text-align: right; }
  .ocrbar { position: absolute; left: 0; right: 0; bottom: 0; z-index: 6; display: flex; flex-wrap: wrap; align-items: center; gap: 0.6rem; padding: 0.55rem 0.9rem; border-top: 1px solid var(--border-strong); background: var(--bg-elevated); font-size: 0.82rem; }
  .ocrbar .err { color: var(--accent-video); }
  .pbar { flex: 1 1 8rem; height: 6px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .pfill { height: 100%; background: var(--accent); transition: width 0.2s ease; }
  .msg { margin: auto; text-align: center; padding: 2rem; }
</style>
