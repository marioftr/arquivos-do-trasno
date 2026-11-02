<script lang="ts">
  import type { OcrResult } from "../../ocr";
  import { toast } from "../../toasts";
  import { tr } from "../../i18n";

  /** Capa de texto sobre a imaxe: permite seleccionar e copiar o que se leu. A selección é propia (non a do navegador),
   *  así que sempre colle exactamente as palabras que se marcan, de liña en liña, como nun PDF. */
  export let box: { l: number; t: number; w: number; h: number };
  export let sc: number;
  export let result: OcrResult;

  let root: HTMLDivElement;
  // Palabras en orde de lectura, cada unha co índice da súa liña.
  $: flat = result.lines.flatMap((line, li) => line.map((w) => ({ ...w, li })));
  $: lineIdx = result.lines.map((_, li) => flat.map((w, i) => (w.li === li ? i : -1)).filter((i) => i >= 0));

  let a = -1;
  let b = -1;
  let hover = -1;
  let dragging = false;
  $: lo = Math.min(a, b);
  $: hi = Math.max(a, b);
  $: if (result) (a = -1), (b = -1);

  /** Palabra máis preto do punto (en px do visor): primeiro a liña máis próxima en vertical e dentro dela a máis próxima en horizontal. */
  function nearest(px: number, py: number, strict = false): number {
    let best = -1;
    let bd = Infinity;
    let bh = 0;
    let bx = 0;
    lineIdx.forEach((ids) => {
      if (!ids.length) return;
      const top = Math.min(...ids.map((i) => flat[i].y0)) * sc;
      const bot = Math.max(...ids.map((i) => flat[i].y1)) * sc;
      const dy = py < top ? top - py : py > bot ? py - bot : 0;
      if (dy >= bd) return;
      let wi = ids[0];
      let wd = Infinity;
      for (const i of ids) {
        const w = flat[i];
        const dx = px < w.x0 * sc ? w.x0 * sc - px : px > w.x1 * sc ? px - w.x1 * sc : 0;
        if (dx < wd) (wd = dx), (wi = i);
      }
      bd = dy;
      best = wi;
      bh = bot - top;
      bx = wd;
    });
    // En estrito (clic, cursor) só vale se o punto está sobre unha liña ou moi preto dela.
    if (strict && best >= 0 && (bd > bh * 0.5 || bx > bh * 2)) return -1;
    return best;
  }
  const local = (e: PointerEvent): [number, number] => {
    const r = root.getBoundingClientRect();
    return [e.clientX - r.left, e.clientY - r.top];
  };
  function down(e: PointerEvent) {
    if (e.button !== 0) return; // o botón central deixa o paso ao visor para mover a imaxe
    const n = nearest(...local(e), true);
    if (n < 0) {
      (a = -1), (b = -1); // un clic fóra do texto quita a selección e deixa o paso ao visor
      return;
    }
    e.stopPropagation();
    e.preventDefault();
    if (e.detail >= 3) {
      const ids = lineIdx[flat[n].li];
      (a = ids[0]), (b = ids[ids.length - 1]); // triple clic: a liña
      return;
    }
    a = b = n;
    if (e.detail === 2) return; // dobre clic: a palabra
    dragging = true;
    root.setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    hover = nearest(...local(e), !dragging);
    if (dragging) b = nearest(...local(e));
  }
  const up = () => (dragging = false);
  const hasSel = () => a >= 0 && flat.length > 0;
  /** O texto marcado: as palabras separadas por espazos e cada liña do orixinal nunha liña. */
  function selectedText(): string {
    const out: string[] = [];
    let cur = -1;
    for (let i = lo; i <= hi && i < flat.length; i++) {
      if (flat[i].li !== cur) {
        out.push("");
        cur = flat[i].li;
      }
      out[out.length - 1] += (out[out.length - 1] ? " " : "") + flat[i].text;
    }
    return out.join("\n");
  }
  async function copy() {
    try {
      await navigator.clipboard.writeText(selectedText());
      toast(tr("O trasno meteu o texto no portapapeis."));
    } catch {
      toast(tr("O portapapeis púxose teimudo co texto. Tenta de novo."), { kind: "warn" });
    }
  }
  function key(e: KeyboardEvent) {
    if ((e.target as HTMLElement)?.closest?.("input, textarea, select, [contenteditable]")) return;
    const mod = e.ctrlKey || e.metaKey;
    const k = e.key.toLowerCase();
    if (mod && k === "a" && flat.length) {
      e.preventDefault();
      (a = 0), (b = flat.length - 1);
    } else if (mod && k === "c" && hasSel() && !window.getSelection()?.toString()) {
      e.preventDefault();
      copy();
    } else if (k === "escape" && hasSel()) {
      (a = -1), (b = -1);
    }
  }
  /** Un clic fóra do texto quita a selección. */
  function outside(e: PointerEvent) {
    if (root && !root.contains(e.target as Node)) (a = -1), (b = -1);
  }
  const rect = (w: { x0: number; y0: number; x1: number; y1: number }, s: number) =>
    `left:${w.x0 * s}px;top:${w.y0 * s}px;width:${Math.max(2, (w.x1 - w.x0) * s)}px;height:${Math.max(4, (w.y1 - w.y0) * s)}px;`;
</script>

<svelte:window on:keydown={key} on:pointerdown|capture={outside} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="ocr"
  bind:this={root}
  style={`left:${box.l}px;top:${box.t}px;width:${box.w}px;height:${box.h}px`}
  on:dblclick|stopPropagation
  on:pointerdown={down}
  on:pointermove={move}
  on:pointerup={up}
  on:pointercancel={up}
  on:pointerleave={() => (hover = -1)}
>
  {#each flat as w, i}
    <span class:sel={a >= 0 && i >= lo && i <= hi} class:hov={i === hover && !dragging} style={rect(w, sc)}></span>
  {/each}
</div>

<style>
  .ocr { position: absolute; overflow: hidden; cursor: text; touch-action: none; user-select: none; }
  span { position: absolute; border-radius: 2px; background: rgba(255, 196, 0, 0.2); box-shadow: 0 0 0 1px rgba(255, 196, 0, 0.35); pointer-events: none; }
  span.hov { background: rgba(255, 196, 0, 0.4); box-shadow: 0 0 0 1px rgba(255, 196, 0, 0.75); }
  span.sel { background: rgba(255, 170, 0, 0.62); box-shadow: 0 0 0 1px rgba(255, 150, 0, 0.9); }
</style>
