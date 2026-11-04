<script lang="ts">
  import { motionReduced } from "../../settings";
  import { get } from "svelte/store";
  import type { Job } from "../../job";

  export let job: Job;
  $: busy = job.busy;
  $: text = job.text;

  // Ao empezar: un billete co nome do traballo sae do botón premido, voa en arco ata a barra de abaixo e o trasno
  // dálle un saltiño ao collelo. Un só elemento, que se retira ao rematar: nada queda flotando.
  let anchor: HTMLSpanElement;
  let was = false;
  $: if ($busy !== was) {
    was = $busy;
    if ($busy) fly();
  }
  async function fly() {
    const dock = document.querySelector<HTMLElement>(".status .trasno");
    const target = document.querySelector<HTMLElement>(".status .mid") ?? dock;
    if (get(motionReduced) || !dock || !target || !anchor) return;
    const ae = document.activeElement as HTMLElement | null;
    const src = ae && ae.tagName === "BUTTON" && ae.closest(".right") ? ae : anchor;
    const a = src.getBoundingClientRect();
    const b = target.getBoundingClientRect();
    const sx = a.left + a.width / 2;
    const sy = a.top + a.height / 2;
    const ex = b.left + b.width / 2;
    const ey = b.top + b.height / 2;
    const ticket = document.createElement("div");
    ticket.textContent = ($text || "").replace(/…$/, "").slice(0, 30);
    Object.assign(ticket.style, { position: "fixed", left: "0", top: "0", zIndex: "300", pointerEvents: "none", padding: "4px 11px", borderRadius: "999px", background: "var(--accent)", color: "var(--on-accent)", font: "600 11px/1.3 inherit", whiteSpace: "nowrap", opacity: "0", boxShadow: "0 4px 14px rgba(0,0,0,.35)", willChange: "transform, opacity" });
    document.body.appendChild(ticket);
    const at = (x: number, y: number, k: number) => `translate(${x}px, ${y}px) translate(-50%, -50%) scale(${k})`;
    const lift = Math.min(140, Math.max(60, Math.abs(ey - sy) * 0.3));
    try {
      const fl = ticket.animate(
        [
          { transform: at(sx, sy, 0.5), opacity: 0 },
          { transform: at(sx, sy - 18, 1), opacity: 1, offset: 0.18 },
          { transform: at((sx + ex) / 2, Math.min(sy, ey) - lift, 1), opacity: 1, offset: 0.55 },
          { transform: at(ex, ey, 0.55), opacity: 0.9, offset: 0.92 },
          { transform: at(ex, ey, 0.3), opacity: 0 },
        ],
        { duration: 640, easing: "cubic-bezier(.4,.1,.3,1)", fill: "both" },
      );
      setTimeout(() => dock.animate([{ transform: "translateY(0)" }, { transform: "translateY(-9px)" }, { transform: "translateY(0)" }], { duration: 260, easing: "ease-out" }), 560);
      await fl.finished;
    } catch {
      // sen animación
    } finally {
      ticket.remove();
    }
  }
</script>

<span class="anchor" bind:this={anchor}></span>

<style>
  .anchor { display: block; height: 0; }
</style>
