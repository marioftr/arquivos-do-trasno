import { writable } from "svelte/store";

/** Ruta actual (o que vai tras «#/»): baleira é o inicio. Co hash, o botón de volver do móbil funciona sen servidor. */
const read = () => decodeURIComponent(location.hash.replace(/^#\/?/, ""));
export const route = writable(read());
window.addEventListener("hashchange", () => route.set(read()));
export const href = (id: string) => `#/${id}`;
