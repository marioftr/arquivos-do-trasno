<script lang="ts">
  import { motionReduced } from "../../settings";
  import { get } from "svelte/store";
  import { tick } from "svelte";
  import { trasnoAway } from "../../agent";
  import type { Job } from "../../job";

  export let job: Job;
  $: busy = job.busy;
  $: text = job.text;

  // Ao empezar: o trasno da barra baixa e outro cae sobre o botón premido cun billete do traballo; dá un saltiño e baixa
  // dun golpe ata a barra co billete. Ao chegar, o da barra ocupa o seu sitio no mesmo instante (nunca hai dous á vez,
  // nin un oco baleiro) e nace o recadro de progreso.
  let anchor: HTMLSpanElement;
  let was = false;
  $: if ($busy !== was) {
    was = $busy;
    if ($busy) fly();
  }
  const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
  async function fly() {
    const dock = document.querySelector<HTMLElement>(".status .trasno");
    if (get(motionReduced) || !dock || !anchor) return;
    const ae = document.activeElement as HTMLElement | null;
    const src = ae && ae.tagName === "BUTTON" && ae.closest(".right") ? ae : anchor;
    const a = src.getBoundingClientRect();
    const b = dock.getBoundingClientRect();
    const S = b.width || 96;
    const cx = a.left + a.width / 2;
    const standY = Math.max(4, a.top - S + 10); // de pé sobre o botón
    const layer = document.createElement("div");
    Object.assign(layer.style, { position: "fixed", inset: "0", zIndex: "300", pointerEvents: "none", overflow: "hidden" });
    const sprite = document.createElement("img");
    sprite.src = "/trasno/base.png";
    sprite.alt = "";
    Object.assign(sprite.style, { position: "absolute", left: `${cx - S / 2}px`, top: "0", width: `${S}px`, height: `${S}px`, imageRendering: "pixelated", opacity: "0", filter: "drop-shadow(0 3px 0 rgba(0,0,0,.35))", willChange: "transform" });
    const ticket = document.createElement("div");
    ticket.textContent = ($text || "").replace(/…$/, "").slice(0, 30);
    Object.assign(ticket.style, { position: "absolute", left: `${cx + S / 4}px`, top: `${standY + S * 0.25}px`, padding: "3px 9px", borderRadius: "6px", background: "var(--accent)", color: "var(--on-accent)", font: "600 11px/1.3 inherit", whiteSpace: "nowrap", opacity: "0", boxShadow: "0 4px 14px rgba(0,0,0,.35)", willChange: "transform, opacity" });
    const ring = document.createElement("div");
    Object.assign(ring.style, { position: "absolute", left: `${b.left + b.width / 2 - 28}px`, top: `${b.top + b.height - 40}px`, width: "56px", height: "56px", borderRadius: "50%", border: "2px solid var(--accent)", opacity: "0" });
    layer.append(ring, sprite, ticket);
    const dx = b.left + b.width / 2 - cx; // desprazamento horizontal ata a barra
    trasnoAway.set(true);
    try {
      await wait(200); // o da barra acaba de baixar
      document.body.appendChild(layer);
      // 1) cae sobre o botón; o billete sae del
      await Promise.all([
        sprite.animate([{ transform: `translateY(${-S - 30}px) scale(1, 1.15)`, opacity: 0 }, { transform: `translateY(${standY + 6}px) scale(1.1, 0.88)`, opacity: 1, offset: 0.7 }, { transform: `translateY(${standY}px) scale(1)`, opacity: 1 }], { duration: 320, easing: "cubic-bezier(.3,.7,.4,1)", fill: "forwards" }).finished,
        ticket.animate([{ transform: "translateY(16px) scale(.4)", opacity: 0 }, { transform: "translateY(0) scale(1)", opacity: 1 }], { duration: 220, delay: 200, easing: "cubic-bezier(.2,1.4,.4,1)", fill: "forwards" }).finished,
      ]);
      // 2) saltiño de alegría
      await sprite.animate([{ transform: `translateY(${standY}px)` }, { transform: `translateY(${standY - 10}px)` }, { transform: `translateY(${standY}px)` }], { duration: 190, easing: "ease-out", fill: "forwards" }).finished;
      // 3) baixa dun golpe á barra co billete
      await Promise.all([
        sprite.animate([{ transform: `translate(0, ${standY}px)` }, { transform: `translate(0, ${standY - 12}px)`, offset: 0.2 }, { transform: `translate(${dx}px, ${b.top}px)` }], { duration: 290, easing: "cubic-bezier(.6,0,.9,.55)", fill: "forwards" }).finished,
        ticket.animate([{ transform: "translate(0, 0) scale(1)", opacity: 1 }, { transform: `translate(${dx - S / 4}px, ${b.top - standY - S * 0.1}px) scale(.4)`, opacity: 0 }], { duration: 290, easing: "cubic-bezier(.6,0,.9,.55)", fill: "forwards" }).finished,
      ]);
      // 4) impacto: o da barra ocupa o seu sitio ao instante e o voador retírase no mesmo fotograma
      dock.style.transition = "none";
      trasnoAway.set(false);
      await tick();
      sprite.remove();
      ticket.remove();
      void dock.offsetHeight;
      dock.style.transition = "";
      const w = ring.animate([{ transform: "scale(.4)", opacity: 0.9 }, { transform: "scale(1.9)", opacity: 0 }], { duration: 420, easing: "ease-out" });
      await w.finished;
    } catch {
      // sen animación
    } finally {
      dock.style.transition = "";
      layer.remove();
      trasnoAway.set(false);
    }
  }
</script>

<span class="anchor" bind:this={anchor}></span>

<style>
  .anchor { display: block; height: 0; }
</style>
