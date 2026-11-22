import { writable } from "svelte/store";

/** Tema: o do sistema ata que a persoa elixe un (o script theme-init.js xa o puxo antes de pintar). */
const KEY = "arquivos-do-trasno.web-theme";
const media = matchMedia("(prefers-color-scheme: light)");
const stored = () => {
  try {
    const v = localStorage.getItem(KEY);
    return v === "light" || v === "dark" ? v : null;
  } catch {
    return null;
  }
};
const system = () => (media.matches ? "light" : "dark");

export const theme = writable<"dark" | "light">(stored() ?? system());
theme.subscribe((v) => (document.documentElement.dataset.theme = v));
media.addEventListener("change", () => {
  if (!stored()) theme.set(system());
});

export function toggleTheme() {
  theme.update((v) => {
    const next = v === "dark" ? "light" : "dark";
    try {
      localStorage.setItem(KEY, next);
    } catch {
      // sen almacenamento: vale ata pechar
    }
    return next;
  });
}
