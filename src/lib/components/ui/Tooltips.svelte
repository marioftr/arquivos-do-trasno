<script lang="ts">
  import { onDestroy } from "svelte";

  /** Etiquetas ao pasar o rato: todo `title` sae nun globo propio (o do sistema non sempre aparece nas webviews) e a descrición de cada
   *  botón, mesmo os de só icona, vese igual en todas partes. */
  let tip: { text: string; x: number; y: number; below: boolean } | null = null;
  let el: HTMLElement | null = null;
  let saved = "";
  let addedLabel = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let lastShown = 0;

  function restore() {
    if (el) {
      if (saved) el.setAttribute("title", saved);
      if (addedLabel) el.removeAttribute("aria-label");
    }
    el = null;
    saved = "";
    addedLabel = false;
  }
  function hide() {
    clearTimeout(timer);
    tip = null;
    restore();
  }
  function show() {
    if (!el || !saved) return;
    const r = el.getBoundingClientRect();
    const below = r.bottom + 60 < window.innerHeight;
    tip = { text: saved, x: Math.min(Math.max(r.left + r.width / 2, 130), window.innerWidth - 130), y: below ? r.bottom + 6 : r.top - 6, below };
    lastShown = performance.now();
  }
  function over(e: PointerEvent) {
    const target = e.target as HTMLElement | null;
    if (el && target && el.contains(target)) return;
    const next = target?.closest?.("[title]") as HTMLElement | null;
    if (!next) return hide();
    const text = next.getAttribute("title") ?? "";
    hide();
    if (!text.trim()) return;
    el = next;
    saved = text;
    next.removeAttribute("title");
    // Sen o `title` un botón de só icona perdería o nome accesible mentres se mostra o globo.
    if (!next.hasAttribute("aria-label") && !next.textContent?.trim()) {
      next.setAttribute("aria-label", text);
      addedLabel = true;
    }
    // Se acaba de pecharse outro globo, este sae ao momento; se non, agarda un chisco.
    timer = setTimeout(show, performance.now() - lastShown < 600 && lastShown ? 0 : 450);
  }
  onDestroy(hide);
</script>

<svelte:window on:pointerover={over} on:pointerdown={hide} on:keydown={hide} on:blur={hide} on:scroll|capture={hide} />

{#if tip}
  <div class="tt" role="tooltip" style={`left:${tip.x}px;top:${tip.y}px;transform:translate(-50%, ${tip.below ? "0" : "-100%"})`}>{tip.text}</div>
{/if}

<style>
  .tt { position: fixed; z-index: 95; width: max-content; max-width: 260px; padding: 0.3rem 0.55rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); box-shadow: 0 6px 18px rgba(0, 0, 0, 0.35); font-size: var(--fs-sm); line-height: 1.35; color: var(--text); pointer-events: none; white-space: pre-line; }
</style>
