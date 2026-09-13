import { writable } from "svelte/store";

export type View = "inicio" | "imaxe" | "audio" | "pdf" | "video" | "conversor";

export const currentView = writable<View>("inicio");

export function goTo(view: View) {
  currentView.set(view);
}
