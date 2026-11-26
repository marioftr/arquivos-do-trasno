import { writable } from "svelte/store";

/** O sitio onde vai o trasno grande da páxina (inicio, descarga). A cabeceira ten un único trasno que viaxa entre ese sitio e o seu oco; se non hai sitio, está na cabeceira. */
export const heroEl = writable<HTMLElement | null>(null);

export function heroSlot(node: HTMLElement) {
  heroEl.set(node);
  return {
    destroy() {
      heroEl.update((c) => (c === node ? null : c));
    },
  };
}
