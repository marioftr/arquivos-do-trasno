<script lang="ts">
  import type { OcrResult } from "../../ocr";

  /** Capa de texto invisible sobre a imaxe: permite seleccionar e copiar o que se leu, como nun PDF. */
  export let box: { l: number; t: number; w: number; h: number };
  export let sc: number;
  export let result: OcrResult;

  let ctx: CanvasRenderingContext2D | null = null;
  const natural = (text: string, fs: number) => {
    ctx ??= document.createElement("canvas").getContext("2d");
    if (!ctx) return fs * text.length * 0.5;
    ctx.font = `${fs}px sans-serif`;
    return ctx.measureText(text).width;
  };
  function style(w: { text: string; x0: number; y0: number; x1: number; y1: number }, s: number): string {
    const h = Math.max(4, (w.y1 - w.y0) * s);
    const fs = h * 0.86;
    const width = Math.max(2, (w.x1 - w.x0) * s);
    const k = Math.max(0.3, Math.min(4, width / Math.max(1, natural(w.text, fs))));
    return `left:${w.x0 * s}px;top:${w.y0 * s}px;font-size:${fs}px;line-height:${h}px;height:${h}px;transform:scaleX(${k});`;
  }
</script>

<div class="ocr" style={`left:${box.l}px;top:${box.t}px;width:${box.w}px;height:${box.h}px`}>
  {#each result.lines as line}
    <div class="ln">{#each line as w}<span style={style(w, sc)}>{w.text}</span>{" "}{/each}</div>
  {/each}
</div>

<style>
  .ocr { position: absolute; pointer-events: none; overflow: hidden; }
  .ln { display: block; height: 0; }
  span { position: absolute; transform-origin: left top; white-space: pre; color: transparent; pointer-events: auto; cursor: text; user-select: text; border-radius: 2px; }
  .ocr:hover span { background: rgba(74, 163, 255, 0.14); }
  span::selection { background: rgba(74, 163, 255, 0.45); color: transparent; }
</style>
