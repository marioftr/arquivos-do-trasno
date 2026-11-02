<script lang="ts">
  import { onDestroy } from "svelte";
  import * as pdfjs from "pdfjs-dist";
  import { getPdf, viewState } from "../../pdfCache";
  import * as engine from "../../engine";
  import { toast } from "../../toasts";
  import { tr } from "../../i18n";
  import Icon from "../Icon.svelte";
  import { actionFor } from "../../keymap";
  import { get } from "svelte/store";
  import { pdfOverlay, type FileEntry, type PdfOverlay } from "../../files";
  import { fontCss } from "../../fonts";
  import { t } from "../../i18n";
  import Slider from "../ui/Slider.svelte";
  import FindBar from "../ui/FindBar.svelte";
  import { tick, onMount } from "svelte";
  import "pdfjs-dist/web/pdf_viewer.css";


  export let file: FileEntry;

  interface PageInfo {
    n: number;
    wrap: HTMLDivElement;
    canvas: HTMLCanvasElement;
    w: number; // píxeles CSS
    h: number;
    view: number[]; // caixa da páxina en puntos [x0, y0, x1, y1]
    s: number; // píxeles CSS por punto
    done: boolean; // xa ten a súa imaxe debuxada
  }

  let container: HTMLDivElement;
  let doc: pdfjs.PDFDocumentProxy | null = null;
  let scale = 80;
  let error = "";
  let encrypted = false;
  let token = 0;
  let pages: PageInfo[] = [];
  let current = 1; // páxina que se está vendo

  // ---- axustar ao espazo dispoñible (ancho ou páxina enteira); segue o tamaño da xanela
  let fitMode: "none" | "width" | "page" = "width";
  let ro: ResizeObserver;
  async function applyFit() {
    if (!doc || !container || fitMode === "none") return;
    const pg = await doc.getPage(Math.max(1, Math.min(doc.numPages, current)));
    const [x0, y0, x1, y1] = pg.view;
    const w = (x1 - x0) * 1.333;
    const h = (y1 - y0) * 1.333;
    const availW = container.clientWidth - 32;
    const availH = container.clientHeight - 32;
    const pct = fitMode === "width" ? (availW / w) * 100 : Math.min(availW / w, availH / h) * 100;
    const next = Math.max(20, Math.min(250, Math.round(pct)));
    if (next !== scale) scale = next;
  }
  function setFit(m: "width" | "page") {
    fitMode = m;
    applyFit();
  }
  $: if (doc && container && fitMode !== "none") applyFit();
  onMount(() => {
    // Ao redimensionar, o axuste ao ancho agarda a que se solte (se non, as páxinas redebuxaríanse en cada píxel e parpadearían).
    let fitTimer: ReturnType<typeof setTimeout>;
    ro = new ResizeObserver(() => {
      clearTimeout(fitTimer);
      fitTimer = setTimeout(applyFit, 180);
    });
    return () => ro.disconnect();
  });
  $: if (ro && container) ro.observe(container);

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
    const id = actionFor(e, "pdf");
    if (id === "p.find") {
      e.preventDefault();
      find = true;
      return;
    }
    if (!id || t.closest("input, select, textarea, [contenteditable], [data-filelist]")) return;
    if (id === "p.next") goto(current + 1);
    else if (id === "p.prev") goto(current - 1);
    else if (id === "p.first") goto(1);
    else if (id === "p.last" && doc) goto(doc.numPages);
    else if (id === "p.zoomIn") ((fitMode = "none"), (scale = Math.min(250, scale + 10)));
    else if (id === "p.zoomOut") ((fitMode = "none"), (scale = Math.max(20, scale - 10)));
    else if (id === "p.fitWidth") setFit("width");
    else if (id === "p.fitPage") setFit("page");
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
    const st = viewState.get(path);
    current = st?.page ?? 1;
    if (st) {
      scale = st.scale;
      fitMode = st.fit;
    }
    try {
      doc = null;
      const loaded = await getPdf(path);
      if (mine !== token) return;
      doc = loaded;
    } catch (e: any) {
      if (e?.name === "PasswordException") encrypted = true;
      else error = String(e?.message ?? e);
    }
  }

  // ---- debuxo preguiceiro: primeiro as páxinas que o usuario ten diante, o resto cando se achegan
  const DPR = () => window.devicePixelRatio || 1;
  let io: IntersectionObserver | null = null;
  const pending: PageInfo[] = [];
  let painting = false;
  const proxies = new Map<number, pdfjs.PDFPageProxy>();

  function observePage(p: PageInfo) {
    (p.wrap as any).__p = p;
    io?.observe(p.wrap);
  }
  function enqueue(p: PageInfo) {
    if (p.done || pending.includes(p)) return;
    pending.push(p);
    pump();
  }
  async function pump() {
    if (painting) return;
    painting = true;
    const mine = token;
    while (pending.length && mine === token) {
      // a que está máis preto do que se está vendo, primeiro
      const mid = container.scrollTop + container.clientHeight / 2;
      pending.sort((x, y) => Math.abs(x.wrap.offsetTop - container.offsetTop + x.h / 2 - mid) - Math.abs(y.wrap.offsetTop - container.offsetTop + y.h / 2 - mid));
      const pg = pending.shift()!;
      if (pg.done) continue;
      await paintPage(pg, mine);
      await new Promise((r) => setTimeout(r, 0));
    }
    painting = false;
  }
  async function paintPage(pg: PageInfo, mine: number) {
    if (!doc) return;
    const page = proxies.get(pg.n) ?? (await doc.getPage(pg.n));
    proxies.set(pg.n, page);
    if (mine !== token || !pg.wrap.isConnected) return;
    const viewport = page.getViewport({ scale: pg.s });
    const dpr = DPR();
    pg.canvas.width = viewport.width * dpr;
    pg.canvas.height = viewport.height * dpr;
    pg.done = true;
    try {
      await page.render({ canvasContext: pg.canvas.getContext("2d")!, viewport, transform: dpr !== 1 ? [dpr, 0, 0, dpr, 0, 0] : undefined }).promise;
      // Capa de texto invisible por riba: permite seleccionar e copiar.
      pg.wrap.querySelector(".textLayer")?.remove();
      const layer = document.createElement("div");
      layer.className = "textLayer";
      layer.style.setProperty("--scale-factor", String(pg.s));
      pg.wrap.appendChild(layer);
      await new pdfjs.TextLayer({ textContentSource: page.streamTextContent(), container: layer, viewport }).render();
    } catch {
      // sen capa de texto ou debuxo cortado: o resto do PDF segue vendose
    }
    try {
      await drawLinks(pg, page, viewport);
    } catch {
      // sen ligazóns: non pasa nada
    }
    paint(get(pdfOverlay), [pg]);
  }

  /** Ligazóns do PDF: as internas levan á páxina; as web piden confirmación antes de abrirse. */
  async function drawLinks(pg: PageInfo, page: pdfjs.PDFPageProxy, viewport: pdfjs.PageViewport) {
    const anns = await page.getAnnotations({ intent: "display" });
    pg.wrap.querySelector(".linkLayer")?.remove();
    const layer = document.createElement("div");
    layer.className = "linkLayer";
    for (const a of anns) {
      if (a.subtype !== "Link" || !a.rect || !(a.dest || a.url || a.unsafeUrl)) continue;
      const [x1, y1, x2, y2] = viewport.convertToViewportRectangle(a.rect);
      const el = document.createElement("a");
      el.href = "#";
      el.style.cssText = `left:${Math.min(x1, x2)}px;top:${Math.min(y1, y2)}px;width:${Math.abs(x2 - x1)}px;height:${Math.abs(y2 - y1)}px`;
      el.title = a.url ?? a.unsafeUrl ?? tr("Ir á páxina");
      el.addEventListener("click", (ev) => {
        ev.preventDefault();
        followLink(a);
      });
      layer.appendChild(el);
    }
    if (layer.childElementCount) pg.wrap.appendChild(layer);
  }
  async function followLink(a: any) {
    if (!doc) return;
    try {
      if (a.dest) {
        const dest = typeof a.dest === "string" ? await doc.getDestination(a.dest) : a.dest;
        const ref = Array.isArray(dest) ? dest[0] : null;
        const idx = ref && typeof ref === "object" ? await doc.getPageIndex(ref) : Number(ref);
        if (Number.isFinite(idx)) goto(idx + 1);
        return;
      }
      const url = String(a.url ?? a.unsafeUrl ?? "");
      if (/^https?:\/\//i.test(url)) {
        toast(tr("Este PDF quere abrir: {url}", { url: url.length > 70 ? `${url.slice(0, 70)}…` : url }), {
          action: { label: tr("Abrir ligazón"), run: () => engine.openUrl(url).catch(() => toast(tr("O trasno non puido abrir a ligazón."), { kind: "warn" })) },
          ms: 9000,
        });
      }
    } catch {
      toast(tr("O trasno non atopou o destino desta ligazón."), { kind: "warn" });
    }
  }
  /** Páxinas que quedan lonxe da pantalla liberan a súa imaxe (un PDF longo non enche a memoria). */
  function release(pg: PageInfo) {
    if (!pg.done) return;
    pg.done = false;
    pg.canvas.width = 1;
    pg.canvas.height = 1;
    pg.wrap.querySelector(".textLayer")?.remove();
    pg.wrap.querySelector(".linkLayer")?.remove();
  }

  async function render(percent: number) {
    const mine = ++token;
    const d = doc;
    if (!d) return;
    const keep = current;
    io?.disconnect();
    pending.length = 0;
    pages = [];
    const s = (percent / 100) * 1.333;
    const first = proxies.get(1) ?? (await d.getPage(1));
    proxies.set(1, first);
    if (mine !== token) return;
    io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          const pg = (e.target as any).__p as PageInfo;
          if (e.isIntersecting) enqueue(pg);
          else release(pg);
        }
      },
      { root: container, rootMargin: "900px 0px" },
    );
    // Todos os ocos ao tamaño da primeira páxina (a pantalla non agarda); os tamaños reais corrixense axiña.
    const vp1 = first.getViewport({ scale: s });
    const built: PageInfo[] = [];
    const frag = document.createDocumentFragment();
    for (let n = 1; n <= d.numPages; n++) {
      const wrap = document.createElement("div");
      wrap.className = "pg";
      wrap.style.width = `${vp1.width}px`;
      wrap.style.height = `${vp1.height}px`;
      const canvas = document.createElement("canvas");
      canvas.style.width = `${vp1.width}px`;
      canvas.style.height = `${vp1.height}px`;
      canvas.width = 1;
      canvas.height = 1;
      wrap.appendChild(canvas);
      frag.appendChild(wrap);
      const info: PageInfo = { n, wrap, canvas, w: vp1.width, h: vp1.height, view: first.view, s, done: false };
      built.push(info);
    }
    // Cámbiase todo dunha vez: o contido vello vese ata o último momento.
    container.replaceChildren(frag);
    for (const b of built) observePage(b);
    pages = built;
    goto(keep);
    onScroll();
    // Tamaños reais das páxinas, en segundo plano (hai PDFs con páxinas de tamaños distintos).
    for (let n = 2; n <= d.numPages; n++) {
      if (mine !== token) return;
      const page = proxies.get(n) ?? (await d.getPage(n));
      proxies.set(n, page);
      const vp = page.getViewport({ scale: s });
      const pg = built[n - 1];
      if (Math.abs(vp.width - pg.w) > 0.5 || Math.abs(vp.height - pg.h) > 0.5) {
        pg.w = vp.width;
        pg.h = vp.height;
        pg.view = page.view;
        pg.wrap.style.width = `${vp.width}px`;
        pg.wrap.style.height = `${vp.height}px`;
        pg.canvas.style.width = `${vp.width}px`;
        pg.canvas.style.height = `${vp.height}px`;
      }
      if (n % 25 === 0) await new Promise((r) => setTimeout(r, 0));
    }
    if (mine === token) {
      pages = pages;
      if (find && query.trim() && hits.length) {
        await tick();
        drawHits();
      }
    }
  }

  /** Textos editables: clic para escoller, arrastrar para mover, esquina para escalar e dobre clic para escribir. */
  function paintItems(it: NonNullable<PdfOverlay["items"]>, p: PageInfo, txt: (text: string, size: number, color: string, left: number, top: number, extra?: string, font?: string, lh?: number) => HTMLElement) {
    for (const b of it.list.filter((x) => x.page === p.n)) {
      const sel = it.selected === b.id;
      const editing = it.editing === b.id;
      const el = txt(b.text || " ", b.size, b.color, p.w * (b.xPct / 100), p.h * (1 - b.yPct / 100), "z-index:3;", b.font, 1.2);
      el.dataset.tid = String(b.id);
      el.style.pointerEvents = "auto";
      el.style.cursor = editing ? "text" : "move";
      el.style.touchAction = "none";
      el.style.padding = "2px 3px";
      el.style.margin = "-2px 0 0 -3px";
      el.style.outline = sel ? "1.5px solid #4aa3ff" : "1px dashed rgba(74,163,255,.55)";
      if (editing) {
        el.contentEditable = "plaintext-only";
        el.spellcheck = false;
        queueMicrotask(() => {
          el.focus();
          const r = document.createRange();
          r.selectNodeContents(el);
          const g = window.getSelection();
          g?.removeAllRanges();
          g?.addRange(r);
        });
        el.addEventListener("blur", () => it.onEdit(b.id, el.innerText.replace(/\n$/, "")));
        el.addEventListener("keydown", (e) => {
          e.stopPropagation();
          if (e.key === "Escape" || (e.key === "Enter" && (e.ctrlKey || e.metaKey))) {
            e.preventDefault();
            el.blur();
          }
        });
        el.addEventListener("pointerdown", (e) => e.stopPropagation());
        continue;
      }
      if (sel) {
        const h = document.createElement("div");
        h.title = tr("Arrastra para cambiar o tamaño");
        h.style.cssText = "position:absolute;right:-9px;bottom:-9px;width:14px;height:14px;background:#fff;border:2px solid #4aa3ff;border-radius:3px;cursor:nwse-resize;z-index:4;touch-action:none;";
        h.addEventListener("pointerdown", (ev) => {
          if (ev.button !== 0) return;
          ev.stopPropagation();
          ev.preventDefault();
          const w0 = Math.max(10, el.offsetWidth);
          const sx = ev.clientX;
          let size = b.size;
          const mv = (e2: PointerEvent) => {
            size = Math.max(4, Math.min(500, Math.round(b.size * ((w0 + (e2.clientX - sx)) / w0) * 10) / 10));
            el.style.fontSize = `${size * p.s}px`;
          };
          const end = () => {
            window.removeEventListener("pointermove", mv);
            window.removeEventListener("pointerup", end);
            window.removeEventListener("pointercancel", end);
            it.onResize(b.id, size);
          };
          window.addEventListener("pointermove", mv);
          window.addEventListener("pointerup", end);
          window.addEventListener("pointercancel", end);
        });
        el.appendChild(h);
      }
      el.addEventListener("dblclick", (ev) => {
        ev.stopPropagation();
        it.onEditing(b.id);
      });
      el.addEventListener("pointerdown", (ev) => {
        if (ev.button !== 0 || ev.target !== el) return;
        ev.stopPropagation();
        const r = p.wrap.getBoundingClientRect();
        const sx = ev.clientX;
        const sy = ev.clientY;
        const l0 = parseFloat(el.style.left);
        const t0 = parseFloat(el.style.top);
        let nx = b.xPct;
        let ny = b.yPct;
        let moved = false;
        const mv = (e2: PointerEvent) => {
          const dx = e2.clientX - sx;
          const dy = e2.clientY - sy;
          if (!moved && Math.hypot(dx, dy) < 3) return;
          moved = true;
          el.style.left = `${l0 + dx}px`;
          el.style.top = `${t0 + dy}px`;
          nx = Math.max(0, Math.min(100, b.xPct + (dx / r.width) * 100));
          ny = Math.max(0, Math.min(100, b.yPct - (dy / r.height) * 100));
        };
        const end = () => {
          window.removeEventListener("pointermove", mv);
          window.removeEventListener("pointerup", end);
          window.removeEventListener("pointercancel", end);
          if (moved) it.onMove(b.id, Math.round(nx * 10) / 10, Math.round(ny * 10) / 10);
          if (it.selected !== b.id) it.onSelect(b.id);
        };
        window.addEventListener("pointermove", mv);
        window.addEventListener("pointerup", end);
        window.addEventListener("pointercancel", end);
      });
    }
  }

  /** Debuxa os cambios pendentes por riba das páxinas (non toca o PDF). */
  function paint(ov: PdfOverlay | null, list: PageInfo[]) {
    for (const p of list) {
      p.wrap.querySelectorAll(".ov").forEach((e) => e.remove());
      p.wrap.classList.remove("sel", "dim");
      const tl = p.wrap.querySelector<HTMLElement>(".textLayer");
      const ll = p.wrap.querySelector<HTMLElement>(".linkLayer");
      if (ll) ll.style.display = ov?.rotate && ov.rotate.degrees % 360 !== 0 && (ov.rotate.page === null || ov.rotate.page === p.n) ? "none" : "";
      if (tl) tl.style.display = ov?.rotate && ov.rotate.degrees % 360 !== 0 && (ov.rotate.page === null || ov.rotate.page === p.n) ? "none" : "";
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
      const txt = (text: string, size: number, color: string, left: number, top: number, extra = "", font?: string, lh = 1) => {
        const e = document.createElement("div");
        e.textContent = text;
        e.style.cssText = `position:absolute;left:${left}px;top:${top}px;font-size:${size * p.s}px;${fontCss(font)}color:${color};white-space:pre;line-height:${lh};pointer-events:none;${extra}`;
        add(e);
        return e;
      };
      p.wrap.style.cursor = ov.items ? "crosshair" : "";
      p.wrap.onclick = ov.items
        ? (ev: MouseEvent) => {
            // Un clic sobre unha zona baleira coloca un texto novo; sobre un texto xa o trata el.
            if ((ev.target as HTMLElement).closest(".ov")) return;
            if (ov.items!.editing !== null) return ov.items!.onEditing(null);
            const r = p.wrap.getBoundingClientRect();
            const x = ((ev.clientX - r.left) / r.width) * 100;
            const y = 100 - ((ev.clientY - r.top) / r.height) * 100;
            ov.items!.onPlace(p.n, Math.round(x * 10) / 10, Math.round(y * 10) / 10);
          }
        : null;
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
        txt(o.text, o.size, o.color, p.w * (o.xPct / 100), p.h * (1 - o.yPct / 100), `opacity:${o.opacity};transform:translate(-50%,-50%) rotate(${-o.angle}deg);`, o.font);
      }
      if (ov.items) paintItems(ov.items, p, txt);
      if (ov.numbers && inPages(ov.numbers.pages ?? null)) {
        const o = ov.numbers;
        const label = o.format.replace("{n}", String(o.start + p.n - 1)).replace("{total}", String(o.start + list.length - 1));
        const left = o.position[1] === "l" ? o.margin * p.s : o.position[1] === "r" ? p.w - o.margin * p.s : p.w / 2;
        const top = o.position[0] === "t" ? o.margin * p.s : p.h - o.margin * p.s;
        const tx = o.position[1] === "l" ? "0" : o.position[1] === "r" ? "-100%" : "-50%";
        txt(label, o.size, o.color, left, top, `transform:translate(${tx},${o.position[0] === "t" ? "0" : "-100%"});`, o.font);
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
    io?.disconnect();
    if (doc) viewState.set(file.path, { page: current, scale, fit: fitMode });
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
    <div class="msg">
      <p class="muted">{$t("Non puiden abrir o PDF: {error}", { error })}</p>
      <button class="btn" on:click={() => open(file.path)}>{$t("Tentar de novo")}</button>
    </div>
  {:else}
    <div class="pages scrollbar-thin" bind:this={container} on:scroll={onScroll}></div>
  {/if}
  <div class="bar">
    <button class="btn" class:on={fitMode === "width"} on:click={() => setFit("width")} title={$t("Axustar ao ancho (0)")}>{$t("Ancho")}</button>
    <button class="btn" class:on={fitMode === "page"} on:click={() => setFit("page")} title={$t("Páxina enteira (9)")}>{$t("Páxina enteira")}</button>
    <Slider compact label={$t("Zoom")} unit="%" min={20} max={250} bind:value={scale} on:input={() => (fitMode = "none")} />
    {#if doc}
      <span class="pag pgbox">
        <button class="btn ghost icon" on:click={() => goto(current - 1)} disabled={current <= 1} aria-label={$t("Páxina anterior")} title={$t("Páxina anterior")}><Icon name="back" size={14} /></button>
        <input type="number" min="1" max={doc.numPages} value={current} on:change={(e) => goto(Number(e.currentTarget.value))} aria-label={$t("Páxina")} />
        <span class="muted">/ {doc.numPages}</span>
        <button class="btn ghost icon" on:click={() => goto(current + 1)} disabled={current >= doc.numPages} aria-label={$t("Páxina seguinte")} title={$t("Páxina seguinte")}><Icon name="arrow-right" size={14} /></button>
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
  .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.6rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .pag { margin-left: auto; flex-shrink: 0; }
  .pgbox { display: flex; align-items: center; gap: 0.35rem; white-space: nowrap; }
  .pgbox input { width: 3.4rem; height: 26px; text-align: center; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: 5px; font: inherit; font-size: 0.82rem; }
  .pgbox input:focus { outline: none; border-color: var(--accent); }
  .pages :global(.hl) { position: absolute; background: color-mix(in srgb, #ffd54a 55%, transparent); mix-blend-mode: multiply; pointer-events: none; border-radius: 2px; }
  .pages :global(.hl.cur) { background: color-mix(in srgb, #ff9f1a 70%, transparent); outline: 2px solid #ff9f1a; }
  .msg { padding: 2rem; }
  .btn.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .pages :global(.textLayer) { user-select: text; z-index: 1; }
  .pages :global(.linkLayer a) { position: absolute; display: block; z-index: 2; cursor: pointer; border-radius: 2px; }
  .pages :global(.linkLayer a:hover) { background: color-mix(in srgb, var(--accent) 18%, transparent); outline: 1px solid color-mix(in srgb, var(--accent) 60%, transparent); }
  .pages :global(.textLayer *) { user-select: text; }
  .pages :global(.textLayer ::selection) { background: color-mix(in srgb, var(--accent) 35%, transparent); }
  .pages :global(.hl) { z-index: 0; }
</style>
