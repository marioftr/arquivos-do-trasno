import { writable } from "svelte/store";

/** Hai un trasno en grande á vista (inicio e descarga)? Mentres si, a cabeceira non leva o seu; ao saír da pantalla ou cambiar de páxina, sobe e ocupa o seu sitio. */
export const heroInView = writable(true);

export function watchHero(node: HTMLElement) {
  heroInView.set(true);
  const io = new IntersectionObserver(([e]) => heroInView.set(e.isIntersecting), { rootMargin: "-80px 0px 0px 0px" });
  io.observe(node);
  return { destroy: () => io.disconnect() };
}
