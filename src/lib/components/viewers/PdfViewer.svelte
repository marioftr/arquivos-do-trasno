<script lang="ts">
  import { onDestroy } from "svelte";
  import * as pdfjs from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
  import { viewUrl, pdfOverlay, type FileEntry, type PdfOverlay } from "../../files";
  import { t } from "../../i18n";
  import Slider from "../ui/Slider.svelte";
  import FindBar from "../ui/FindBar.svelte";
  import { tick } from "svelte";

  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

  export let file: FileEntry;

  interface PageInfo {
    n: number;
    wrap: HTMLDivElement;
    canvas: HTMLCanvasElement;
    w: number; // píxeles CSS
    h: number;
    view: number[]; // caixa da páxina en puntos [x0, y0, x1, y1]
    s: number; // píxeles CSS por punto
  }

  let container: HTMLDivElement;
  let doc: pdfjs.PDFDocumentProxy | null = null;
  let scale = 80;
  let error = "";
  let encrypted = false;
  let token = 0;
  let pages: PageInfo[] = [];
  let current = 1; // páxina que se está vendo

  // ---- buscar (Ctrl+F): o texto de cada páxina léese a petición e márcanse os cadros onde aparece
  interface Hit { page: number; rect: HTMLDivElement | null; item: any; at: number }
  let find = false;
  let query = "";
  let hits: Hit[] = [];
  let cur = 0; // 1..hits.length
  let searching = false;
  let searchToken = 0;
  const textCache = new Map<number, any[]>();
  let debounce: ReturnType<typeof setTimeout>;
  $: query, find, schedule();

  function schedule() {
    clearTimeout(debounce);
    if (!find || !query.trim()) {
      hits = [];
      cur = 0;
      clearMarks();
      return;
    }
    debounce = setTimeout(runSearch, 250);
  }
  function clearMarks() {
    container?.querySelectorAll(".hl").forEach((e) => e.remove());
  }
  async function runSearch() {
    const mine = ++searchToken;
    const d = doc;
    if (!d) return;
    searching = true;
    const q = query.trim().toLowerCase();
    const found: Hit[] = [];
    for (let n = 1; n <= d.numPages; n++) {
      if (mine !== searchToken) return;
      let items = textCache.get(n);
      if (!items) {
        items = (await (await d.getPage(n)).getTextContent()).items.filter((i: any) => typeof i.str === "string" && i.str);
        textCache.set(n, items);
      }
      for (const item of items) {
        const low = item.str.toLowerCase();
        for (let i = low.indexOf(q); i !== -1; i = low.indexOf(q, i + q.length)) found.push({ page: n, rect: null, item, at: i });
      }
    }
    if (mine !== searchToken) return;
    searching = false;
    hits = found;
    cur = found.length ? 1 : 0;
    await drawHits();
    goHit();
  }
  /** Debuxa un cadro amarelo sobre cada resultado das páxinas xa pintadas. */
  async function drawHits() {
    clearMarks();
    const d = doc;
    if (!d || !hits.length) return;
    const q = query.trim().length;
    for (const h of hits) {
      const p = pages[h.page - 1];
      if (!p) continue;
      const vp = (await d.getPage(h.page)).getViewport({ scale: p.s });
      const tx = pdfjs.Util.transform(vp.transform, h.item.transform);
      const len = Math.max(1, h.item.str.length);
      const w = h.item.width * p.s;
      const ht = (h.item.height || Math.abs(tx[3])) * p.s;
      const el = document.createElement("div");
      el.className = "hl";
      el.style.cssText = `left:${tx[4] + (w * h.at) / len}px;top:${tx[5] - ht}px;width:${(w * q) / len}px;height:${ht}px`;
      p.wrap.appendChild(el);
      h.rect = el;
    }
    markCurrent();
  }
  function markCurrent() {
    hits.forEach((h, i) => h.rect?.classList.toggle("cur", i + 1 === cur));
  }
  function goHit() {
    markCurrent();
    const h = hits[cur - 1];
    if (!h) return;
    (h.rect ?? pages[h.page - 1]?.wrap)?.scrollIntoView({ block: "center", behavior: "smooth" });
  }
  const step = (d: number) => {
    if (!hits.length) return;
    cur = ((cur - 1 + d + hits.length) % hits.length) + 1;
    goHit();
  };

  // ---- páxinas
  function onScroll() {
    if (!container || !pages.length) return;
    const mid = container.scrollTop + container.clientHeight / 3;
    let n = 1;
    for (const p of pages) if (p.wrap.offsetTop - container.offsetTop <= mid) n = p.n;
    current = n;
  }
  function goto(n: number) {
    if (!doc) return;
    const k = Math.max(1, Math.min(doc.numPages, Math.round(n)));
    pages[k - 1]?.wrap.scrollIntoView({ block: "start" });
    current = k;
  }

  function key(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      find = true;
      return;
    }
    if (e.ctrlKey || e.metaKey || e.altKey || t.closest("input, select, textarea, [contenteditable], [data-filelist]")) return;
    if (e.key === "PageDown" || e.key === "ArrowRight") goto(current + 1);
    else if (e.key === "PageUp" || e.key === "ArrowLeft") goto(current - 1);
    else if (e.key === "Home") goto(1);
    else if (e.key === "End" && doc) goto(doc.numPages);
    else if (e.key === "+" || e.key === "=") scale = Math.min(250, scale + 10);
    else if (e.key === "-") scale = Math.max(40, scale - 10);
    else if (e.key === "0") scale = 80;
    else return;
    e.preventDefault();
  }

  $: open(file.path);
  $: if (doc && container) render(scale);
  $: paint($pdfOverlay, pages);

  async function open(path: string) {
    const mine = ++token;
    error = "";
    encrypted = false;
    try {
      await doc?.destroy();
      doc = null;
      const res = await fetch(await viewUrl(path));
      const data = new Uint8Array(await res.arrayBuffer());
      const loaded = await pdfjs.getDocument({ data }).promise;
      if (mine !== token) {
        loaded.destroy();
        return;
      }
      doc = loaded;
    } catch (e: any) {
      if (e?.name === "PasswordException") encrypted = true;
      else error = String(e?.message ?? e);
    }
  }

  async function render(percent: number) {
    const mine = ++token;
    const current = doc;
    if (!current) return;
    container.replaceChildren();
    pages = [];
    const dpr = window.devicePixelRatio || 1;
    const built: PageInfo[] = [];
    for (let n = 1; n <= current.numPages; n++) {
      if (mine !== token) return;
      const page = await current.getPage(n);
      const s = (percent / 100) * 1.333;
      const viewport = page.getViewport({ scale: s });
      const wrap = document.createElement("div");
      wrap.className = "pg";
      wrap.style.width = `${viewport.width}px`;
      wrap.style.height = `${viewport.height}px`;
      const canvas = document.createElement("canvas");
      canvas.width = viewport.width * dpr;
      canvas.height = viewport.height * dpr;
      canvas.style.width = `${viewport.width}px`;
      canvas.style.height = `${viewport.height}px`;
      wrap.appendChild(canvas);
      container.appendChild(wrap);
      built.push({ n, wrap, canvas, w: viewport.width, h: viewport.height, view: page.view, s });
      await page.render({
        canvasContext: canvas.getContext("2d")!,
        viewport,
        transform: dpr !== 1 ? [dpr, 0, 0, dpr, 0, 0] : undefined,
      }).promise;
    }
    if (mine === token) {
      pages = built;
      onScroll();
      if (find && query.trim() && hits.length) {
        await tick();
        drawHits();
      }
    }
  }

  /** Debuxa os cambios pendentes por riba das páxinas (non toca o PDF). */
  function paint(ov: PdfOverlay | null, list: PageInfo[]) {
    for (const p of list) {
      p.wrap.querySelectorAll(".ov").forEach((e) => e.remove());
      p.wrap.classList.remove("sel", "dim");
      p.canvas.style.transform = "";
      p.canvas.style.position = "";
      p.wrap.style.width = `${p.w}px`;
      p.wrap.style.height = `${p.h}px`;
      if (!ov) continue;

      if (ov.select && ov.select.length) p.wrap.classList.add(ov.select.includes(p.n) ? "sel" : "dim");

      if (ov.rotate && (ov.rotate.page === null || ov.rotate.page === p.n) && ov.rotate.degrees % 360 !== 0) {
        const d = ov.rotate.degrees;
        const swap = d % 180 !== 0;
        p.wrap.style.width = `${swap ? p.h : p.w}px`;
        p.wrap.style.height = `${swap ? p.w : p.h}px`;
        p.canvas.style.position = "absolute";
        p.canvas.style.left = "50%";
        p.canvas.style.top = "50%";
        p.canvas.style.transform = `translate(-50%, -50%) rotate(${d}deg)`;
        continue;
      }

      const add = (el: HTMLElement) => {
        el.classList.add("ov");
        p.wrap.appendChild(el);
      };
      const txt = (text: string, size: number, color: string, left: number, top: number, extra = "") => {
        const e = document.createElement("div");
        e.textContent = text;
        e.style.cssText = `position:absolute;left:${left}px;top:${top}px;font:${size * p.s}px Helvetica,Arial,sans-serif;color:${color};white-space:pre;line-height:1;pointer-events:none;${extra}`;
        add(e);
      };
      const inPages = (l: number[] | null) => !l || l.includes(p.n);

      if (ov.crop) {
        const { l, b, r, t } = ov.crop;
        const mk = (css: string) => {
          const e = document.createElement("div");
          e.style.cssText = `position:absolute;background:rgba(10,10,14,.62);pointer-events:none;${css}`;
          add(e);
        };
        mk(`left:0;top:0;bottom:0;width:${l * p.s}px`);
        mk(`right:0;top:0;bottom:0;width:${r * p.s}px`);
        mk(`left:${l * p.s}px;right:${r * p.s}px;top:0;height:${t * p.s}px`);
        mk(`left:${l * p.s}px;right:${r * p.s}px;bottom:0;height:${b * p.s}px`);
      }
      if (ov.text && inPages(ov.text.pages)) {
        const o = ov.text;
        const cx = p.w * (o.xPct / 100);
        const cy = p.h * (1 - o.yPct / 100);
        if (o.centered) txt(o.text, o.size, o.color, cx, cy, `opacity:${o.opacity};transform:translate(-50%,-50%) rotate(${-o.angle}deg);`);
        else txt(o.text, o.size, o.color, cx, cy, `opacity:${o.opacity};transform:translateY(-100%);`);
      }
      if (ov.numbers) {
        const o = ov.numbers;
        const label = o.format.replace("{n}", String(o.start + p.n - 1)).replace("{total}", String(o.start + list.length - 1));
        const left = o.position[1] === "l" ? o.margin * p.s : o.position[1] === "r" ? p.w - o.margin * p.s : p.w / 2;
        const top = o.position[0] === "t" ? o.margin * p.s : p.h - o.margin * p.s;
        const tx = o.position[1] === "l" ? "0" : o.position[1] === "r" ? "-100%" : "-50%";
        txt(label, o.size, o.color, left, top, `transform:translate(${tx},${o.position[0] === "t" ? "0" : "-100%"});`);
      }
      if (ov.image && inPages(ov.image.pages)) {
        const o = ov.image;
        const img = document.createElement("img");
        img.src = o.url;
        img.style.cssText = `position:absolute;left:${p.w * (o.xPct / 100)}px;top:${p.h * (1 - o.yPct / 100)}px;width:${p.w * (o.widthPct / 100)}px;transform:translate(-50%,-50%);opacity:${o.opacity};pointer-events:none;`;
        add(img);
      }
    }
  }

  onDestroy(() => {
    token++;
    doc?.destroy();
  });
</script>

<svelte:window on:keydown={key} />

<div class="wrap">
  {#if find}
    <FindBar bind:query count={hits.length} current={cur} busy={searching} on:next={() => step(1)} on:prev={() => step(-1)} on:close={() => ((find = false), (query = ""))} />
  {/if}
  {#if encrypted}
    <p class="muted msg">{$t("Este PDF ten contrasinal. Desbloquéao na pestana Seguridade para velo.")}</p>
  {:else if error}
    <p class="muted msg">{$t("Non puiden abrir o PDF: {error}", { error })}</p>
  {:else}
    <div class="pages scrollbar-thin" bind:this={container} on:scroll={onScroll}></div>
  {/if}
  <div class="bar">
    <Slider compact label={$t("Zoom")} unit="%" min={40} max={250} bind:value={scale} />
    {#if doc}
      <span class="pag pgbox">
        <input type="number" min="1" max={doc.numPages} value={current} on:change={(e) => goto(Number(e.currentTarget.value))} aria-label={$t("Páxina")} />
        <span class="muted">/ {doc.numPages}</span>
      </span>
    {/if}
  </div>
</div>

<style>
  .wrap { position: relative; display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .pages {
    flex: 1; min-height: 0; overflow: auto;
    display: flex; flex-direction: column; align-items: flex-start; gap: 12px;
    padding: 16px; background: var(--viewer-bg);
  }
  .pages :global(.pg) { position: relative; flex-shrink: 0; margin-inline: auto; background: #fff; overflow: hidden; transition: opacity 0.12s ease; }
  .pages :global(.pg.sel) { outline: 3px solid var(--accent); outline-offset: 0; }
  .pages :global(.pg.dim) { opacity: 0.35; }
  .pages :global(canvas) { display: block; background: #fff; }
  .bar { display: flex; align-items: center; gap: 0.6rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .pag { margin-left: auto; }
  .pgbox { display: flex; align-items: center; gap: 0.35rem; }
  .pgbox input { width: 3.4rem; height: 26px; text-align: center; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: 5px; font: inherit; font-size: 0.82rem; }
  .pgbox input:focus { outline: none; border-color: var(--accent); }
  .pages :global(.hl) { position: absolute; background: color-mix(in srgb, #ffd54a 55%, transparent); mix-blend-mode: multiply; pointer-events: none; border-radius: 2px; }
  .pages :global(.hl.cur) { background: color-mix(in srgb, #ff9f1a 70%, transparent); outline: 2px solid #ff9f1a; }
  .msg { padding: 2rem; }
</style>
