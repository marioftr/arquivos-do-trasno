import { writable } from "svelte/store";

/** Hai un desprazador en uso (arrastrando ou escribindo): os niveis con nome agardan a que se solte para cambiar de selección. */
export const sliderDragging = writable(false);
