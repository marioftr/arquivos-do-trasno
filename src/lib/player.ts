// Volume e silencio do reprodutor: valen para todos os ficheiros e lémbranse entre sesións.

import { writable } from "svelte/store";

const KEY = "arquivos-do-trasno.volume";
function load(): { vol: number; muted: boolean } {
  try {
    const v = JSON.parse(localStorage.getItem(KEY) ?? "{}");
    return { vol: typeof v.vol === "number" ? Math.min(1, Math.max(0, v.vol)) : 1, muted: !!v.muted };
  } catch {
    return { vol: 1, muted: false };
  }
}
export const player = writable(load());
player.subscribe((v) => {
  try {
    localStorage.setItem(KEY, JSON.stringify(v));
  } catch {
    // sen almacenamento: vale mentres a app está aberta
  }
});
