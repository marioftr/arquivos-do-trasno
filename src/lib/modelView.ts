import { writable } from "svelte/store";
import type { MaterialReport } from "./model3d";

/** Estado da vista do modelo 3D: o visor debuxa e a pestana «Vista» das ferramentas manda. Só afecta ao visor; nada se exporta. */
export const modelLight = writable({ azimuth: 45, elevation: 50, intensity: 1.6, ambient: 1.1, showMarker: true });
export const modelShow = writable<{ channel: string; isolated: number | null }>({ channel: "full", isolated: null });

/** O que o visor sabe do modelo aberto (materiais, canles, avisos) para amosalo na pestana. */
export const modelInfo = writable<{ status: string; report: MaterialReport[]; channels: { id: string; label: string }[]; hasTextures: boolean; missing: string[]; notes: string[] }>({
  status: "",
  report: [],
  channels: [],
  hasTextures: false,
  missing: [],
  notes: [],
});
