import { get, writable } from "svelte/store";
import { tick } from "svelte";
import { trasnoAway } from "./agent";
import { motionReduced } from "./settings";

/** Vóos do trasno entre o recanto da barra e a cabeceira dunha xanela. Mentres dura un, o trasno de ambos os sitios queda agochado
 *  para que nunca haxa dous (nin un oco baleiro). */
export const trasnoFlying = writable(false);
/** Como sae o trasno ao pechar a xanela: a casa (pechar sen máis) ou á barra co billete dun traballo que empeza. */
export const trasnoLeave = writable<{ ticket: string } | null>(null);

const dockEl = () => document.querySelector<HTMLElement>(".status .trasno");

function makeSprite(a: DOMRect): HTMLImageElement {
  const img = document.createElement("img");
  img.src = "/trasno/base.png";
  img.alt = "";
  Object.assign(img.style, { position: "fixed", left: `${a.left}px`, top: `${a.top}px`, width: `${a.width}px`, height: `${a.height}px`, imageRendering: "pixelated", filter: "drop-shadow(0 3px 0 rgba(0,0,0,.35))", transformOrigin: "0 0", willChange: "transform", zIndex: "300", pointerEvents: "none" });
  return img;
}

/** Arco suave de `a` a `b` (o tamaño cambia de un a outro). */
function glide(img: HTMLElement, a: DOMRect, b: DOMRect, ms: number, lift: number) {
  const dx = b.left - a.left;
  const dy = b.top - a.top;
  const s = b.width / a.width;
  return img.animate(
    [
      { transform: "translate(0px, 0px) scale(1)" },
      { transform: `translate(${dx / 2}px, ${dy / 2 - lift}px) scale(${(1 + s) / 2})`, offset: 0.5 },
      { transform: `translate(${dx}px, ${dy}px) scale(${s})` },
    ],
    { duration: ms, easing: "cubic-bezier(.45,0,.25,1)", fill: "forwards" },
  ).finished;
}

/** O trasno da barra voa ata a cabeceira da xanela que se abre. */
export async function flyIn(target: HTMLElement | undefined) {
  const dock = dockEl();
  if (get(motionReduced) || !dock || !target) return;
  trasnoFlying.set(true);
  const a = dock.getBoundingClientRect();
  const sprite = makeSprite(a);
  document.body.appendChild(sprite);
  try {
    await tick();
    await glide(sprite, a, target.getBoundingClientRect(), 380, 36);
  } catch {
    // sen animación
  } finally {
    sprite.remove();
    trasnoFlying.set(false);
  }
}

/** O da xanela que se pecha (`from`, medido antes de que desapareza) volve á barra; se hai billete, leva o traballo que empeza. */
export async function flyOut(from: DOMRect | undefined, ticketText: string | null) {
  const dock = dockEl();
  if (get(motionReduced) || !dock || !from || !from.width) {
    trasnoFlying.set(false);
    return;
  }
  const b = dock.getBoundingClientRect();
  const layer = document.createElement("div");
  Object.assign(layer.style, { position: "fixed", inset: "0", zIndex: "300", pointerEvents: "none", overflow: "hidden" });
  const sprite = makeSprite(from);
  layer.append(sprite);
  let ticket: HTMLDivElement | null = null;
  if (ticketText !== null) {
    ticket = document.createElement("div");
    ticket.textContent = ticketText.replace(/…$/, "").slice(0, 30);
    Object.assign(ticket.style, { position: "absolute", left: `${from.left + from.width * 0.7}px`, top: `${from.top + from.height * 0.25}px`, padding: "3px 9px", borderRadius: "6px", background: "var(--accent)", color: "var(--on-accent)", font: "600 11px/1.3 inherit", whiteSpace: "nowrap", opacity: "0", boxShadow: "0 4px 14px rgba(0,0,0,.35)", willChange: "transform, opacity" });
    layer.append(ticket);
  }
  const ring = document.createElement("div");
  Object.assign(ring.style, { position: "absolute", left: `${b.left + b.width / 2 - 28}px`, top: `${b.top + b.height - 40}px`, width: "56px", height: "56px", borderRadius: "50%", border: "2px solid var(--accent)", opacity: "0" });
  layer.append(ring);
  trasnoFlying.set(true);
  if (ticket) trasnoAway.set(true); // a barra de progreso nace ao chegar
  document.body.appendChild(layer);
  try {
    if (ticket) {
      const dx = b.left + b.width / 2 - (from.left + from.width * 0.7);
      const dy = b.top - from.top;
      await Promise.all([
        ticket.animate([{ transform: "translateY(10px) scale(.4)", opacity: 0 }, { transform: "translateY(0) scale(1)", opacity: 1 }], { duration: 180, easing: "cubic-bezier(.2,1.4,.4,1)", fill: "forwards" }).finished,
        sprite.animate([{ transform: "translateY(0px) scale(1)" }, { transform: "translateY(-10px) scale(1)" }, { transform: "translateY(0px) scale(1)" }], { duration: 180, easing: "ease-out" }).finished,
      ]);
      await Promise.all([
        glide(sprite, from, b, 360, 28),
        ticket.animate([{ transform: "translate(0, 0) scale(1)", opacity: 1 }, { transform: `translate(${dx}px, ${dy}px) scale(.4)`, opacity: 0 }], { duration: 360, easing: "cubic-bezier(.45,0,.25,1)", fill: "forwards" }).finished,
      ]);
    } else {
      await glide(sprite, from, b, 380, 36);
    }
    // impacto: o da barra ocupa o seu sitio ao instante e o voador retírase no mesmo fotograma
    dock.style.transition = "none";
    trasnoFlying.set(false);
    trasnoAway.set(false);
    await tick();
    sprite.remove();
    ticket?.remove();
    void dock.offsetHeight;
    dock.style.transition = "";
    if (ticket) await ring.animate([{ transform: "scale(.4)", opacity: 0.9 }, { transform: "scale(1.9)", opacity: 0 }], { duration: 420, easing: "ease-out" }).finished;
  } catch {
    // sen animación
  } finally {
    dock.style.transition = "";
    layer.remove();
    trasnoFlying.set(false);
    trasnoAway.set(false);
  }
}

