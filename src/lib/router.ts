import { writable } from "svelte/store";

export type View = "inicio" | "imaxe" | "audio" | "pdf" | "video" | "conversor";

export const currentView = writable<View>("inicio");

/** Ficheiros á espera de entrar no Conversor (posto por "Abrir ficheiro"). */
export const pendingFiles = writable<string[]>([]);

export function goTo(view: View) {
  currentView.set(view);
}

export function openInConverter(paths: string[]) {
  pendingFiles.set(paths);
  currentView.set("conversor");
}
