import { writable } from "svelte/store";
import type { MaterialReport } from "./model3d";

/** Estado da vista do modelo 3D: o visor debuxa e a pestana «Vista» das ferramentas manda. Só afecta ao visor; nada se exporta. */
export const modelLight = writable({ azimuth: 45, elevation: 50, intensity: 1.6, ambient: 1.1, showMarker: true });
/** Un obxecto da escena (un grupo ou unha malla do ficheiro) para a lista da pestana. */
export interface SceneNode {
  id: string;
  name: string;
  depth: number;
  parent: string | null;
  /** Mallas que leva dentro (el incluído) e triángulos que suman. */
  meshes: number;
  tris: number;
}
/** Obxectos agochados (ids). Só afecta ao visor. */
export const modelHidden = writable<string[]>([]);
export const modelShow = writable<{ channel: string; isolated: number | null }>({ channel: "full", isolated: null });

/** O que o visor sabe do modelo aberto (materiais, canles, avisos) para amosalo na pestana. */
export const modelInfo = writable<{ nodes: SceneNode[]; status: string; report: MaterialReport[]; channels: { id: string; label: string }[]; hasTextures: boolean; missing: string[]; notes: string[] }>({
  nodes: [],
  status: "",
  report: [],
  channels: [],
  hasTextures: false,
  missing: [],
  notes: [],
});
