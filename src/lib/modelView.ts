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
  /** É unha malla (non un simple grupo): só estas se agochan de verdade; un grupo é o conxunto dos seus fillos. */
  own: boolean;
}
/** Obxectos agochados (ids). Só afecta ao visor. */
export const modelHidden = writable<string[]>([]);
/** Malla e cuadrícula do visor: móvense desde a pestana Escena. */
export const modelOptions = writable({ wire: false, grid: true });
/** Grupo da pestana Escena que está aberto e grupos da lista de obxectos pregados (para non perdelos ao cambiar de pestana). */
export const modelPart = writable<"objects" | "materials" | "light">("objects");
export const modelCollapsed = writable<string[]>([]);

/** O obxecto e todo o que leva dentro. */
export function below(nodes: SceneNode[], id: string): SceneNode[] {
  const keep = new Set<string>([id]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const n of nodes) if (n.parent && keep.has(n.parent) && !keep.has(n.id)) (keep.add(n.id), (grew = true));
  }
  return nodes.filter((n) => keep.has(n.id));
}
/** Estado dun obxecto na lista: todo á vista, algo á vista ou nada. Un grupo vale polas mallas que leva dentro. */
export function nodeState(nodes: SceneNode[], hidden: ReadonlySet<string>, id: string): "all" | "some" | "none" {
  const own = below(nodes, id).filter((n) => n.own);
  const on = own.filter((n) => !hidden.has(n.id)).length;
  return on === 0 ? "none" : on === own.length ? "all" : "some";
}
/** Mallas á vista e totais de cada obxecto (cos seus fillos) nunha soa pasada: os pais veñen sempre antes cos seus fillos. */
export function nodeCounts(nodes: SceneNode[], hidden: ReadonlySet<string>): Map<string, { on: number; total: number }> {
  const out = new Map<string, { on: number; total: number }>();
  for (const n of nodes) out.set(n.id, { on: n.own && !hidden.has(n.id) ? 1 : 0, total: n.own ? 1 : 0 });
  for (let i = nodes.length - 1; i >= 0; i--) {
    const n = nodes[i];
    const p = n.parent ? out.get(n.parent) : undefined;
    const c = out.get(n.id)!;
    if (p) (p.on += c.on, (p.total += c.total));
  }
  return out;
}
/** Agocha ou amosa un obxecto con todo o seu interior: se hai algo á vista, agóchase todo; se non, amósase todo. */
export function toggleNode(nodes: SceneNode[], hidden: ReadonlySet<string>, id: string): string[] {
  const ids = below(nodes, id).filter((n) => n.own).map((n) => n.id);
  const next = new Set(hidden);
  if (nodeState(nodes, hidden, id) === "none") ids.forEach((i) => next.delete(i));
  else ids.forEach((i) => next.add(i));
  return [...next];
}
/** Deixa á vista só ese obxecto (e o seu interior): as demais mallas agóchanse. */
export function soloNode(nodes: SceneNode[], id: string): string[] {
  const keep = new Set(below(nodes, id).map((n) => n.id));
  return nodes.filter((n) => n.own && !keep.has(n.id)).map((n) => n.id);
}
/** Se xa se ve só ese obxecto. */
export function isSolo(nodes: SceneNode[], hidden: ReadonlySet<string>, id: string): boolean {
  const mine = soloNode(nodes, id);
  const all = nodes.filter((n) => n.own);
  return mine.length > 0 && mine.length === hidden.size && mine.every((i) => hidden.has(i)) && all.length > mine.length;
}
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
