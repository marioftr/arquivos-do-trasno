// Estado compartido das barras de progreso: cal é o proceso que está en pausa (o ffmpeg que se parou).
import { writable } from "svelte/store";

export const pausedId = writable<string | null>(null);
