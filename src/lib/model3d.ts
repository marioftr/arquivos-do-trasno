// Carga e exportación de modelos 3D co lado web (three.js). O visor e o
// conversor comparten isto: o que se ve é exactamente o que se converte.

import * as THREE from "three";
import { convertFileSrc } from "@tauri-apps/api/core";
import { call } from "./errors";
import { writable, get } from "svelte/store";
import { viewUrl } from "./files";
import { tr } from "./i18n";
import { FBXLoader } from "three/examples/jsm/loaders/FBXLoader.js";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { OBJLoader } from "three/examples/jsm/loaders/OBJLoader.js";
import { MTLLoader } from "three/examples/jsm/loaders/MTLLoader.js";
import { STLLoader } from "three/examples/jsm/loaders/STLLoader.js";
import { PLYLoader } from "three/examples/jsm/loaders/PLYLoader.js";
import { STLExporter } from "three/examples/jsm/exporters/STLExporter.js";
import { PLYExporter } from "three/examples/jsm/exporters/PLYExporter.js";
import { GLTFExporter } from "three/examples/jsm/exporters/GLTFExporter.js";

export const fallbackMaterial = () =>
  new THREE.MeshStandardMaterial({ color: 0xb8bcc6, roughness: 0.6, metalness: 0.1, side: THREE.DoubleSide });

const ASSET_ORIGIN = /^https?:\/\/asset\.localhost\//i;

/** Cartafol e separador dun ficheiro, para resolver texturas relativas. */
function split(path: string): { dir: string; sep: string } {
  const sep = path.includes("\\") ? "\\" : "/";
  return { dir: path.slice(0, path.lastIndexOf(sep)), sep };
}

/**
 * Carga un modelo. As texturas e materiais que o modelo referencia (por
 * exemplo `texturas/pel.png` ou o `.mtl` dun OBJ) búscanse no mesmo cartafol,
 * para o que se autoriza ese cartafol no protocolo asset (só lectura).
 */
export async function loadModel(path: string, ext: string): Promise<THREE.Object3D> {
  return (await loadModelDetailed(path, ext)).obj;
}

export interface LoadedModel {
  obj: THREE.Object3D;
  /** Ficheiros (texturas) que o modelo pedía e non se puideron cargar. */
  missing: string[];
  /** Cambios feitos para que o modelo se vexa ben. */
  notes: string[];
}

export async function loadModelDetailed(path: string, ext: string): Promise<LoadedModel> {
  await call("allow_view_dir", { path });
  const url = convertFileSrc(path);
  const { dir, sep } = split(path);

  const manager = new THREE.LoadingManager();
  const missing: string[] = [];
  // As texturas cargan despois do modelo: agárdase a que remate todo (ou 8 s).
  const idle = new Promise<void>((resolve) => {
    manager.onLoad = () => resolve();
  });
  manager.onError = (u) => {
    missing.push(decodeURIComponent(u.replace(ASSET_ORIGIN, "")).split(/[\\/]/).pop() ?? u);
  };
  const finish = async (obj: THREE.Object3D): Promise<LoadedModel> => {
    await Promise.race([idle, new Promise((r) => setTimeout(r, 8000))]);
    const importNotes = await applyImports(obj, path, ext);
    const notes = [...importNotes, ...repairMaterials(obj)];
    return { obj, missing: [...new Set(missing)], notes };
  };
  manager.setURLModifier((u) => {
    if (u.startsWith("data:") || u.startsWith("blob:")) return u;
    // Os cargadores xuntan o nome relativo á base "http://asset.localhost/".
    if (ASSET_ORIGIN.test(u)) {
      const rest = decodeURIComponent(u.replace(ASSET_ORIGIN, ""));
      if (rest && !/^[A-Za-z]:|^[\\/]/.test(rest) && !rest.includes("%")) {
        const cleaned = rest.replace(/^\.?[\\/]/, "").split(/[\\/]/).filter((p) => p && p !== "..").join(sep);
        return convertFileSrc(`${dir}${sep}${cleaned}`);
      }
    }
    return u;
  });

  switch (ext) {
    case "fbx":
      return finish(await new FBXLoader(manager).loadAsync(url));
    case "glb":
    case "gltf":
      return finish((await new GLTFLoader(manager).loadAsync(url)).scene);
    case "obj": {
      const loader = new OBJLoader(manager);
      try {
        const mtlPath = path.replace(/\.obj$/i, ".mtl");
        const materials = await new MTLLoader(manager).loadAsync(convertFileSrc(mtlPath));
        materials.preload();
        loader.setMaterials(materials);
      } catch {
        // sen .mtl ao lado: material neutro
      }
      return finish(await loader.loadAsync(url));
    }
    case "stl":
      return finish(new THREE.Mesh(await new STLLoader(manager).loadAsync(url), fallbackMaterial()));
    case "ply": {
      const geo = await new PLYLoader(manager).loadAsync(url);
      geo.computeVertexNormals();
      return finish(new THREE.Mesh(geo, fallbackMaterial()));
    }
    default:
      throw new Error(tr("Non teño visor para o formato {ext}.", { ext }));
  }
}

export const TEXTURE_SLOTS: { slot: string; label: string }[] = [
  { slot: "map", label: "Cor (difuso)" },
  { slot: "normalMap", label: "Normais" },
  { slot: "bumpMap", label: "Relevo (bump)" },
  { slot: "displacementMap", label: "Desprazamento" },
  { slot: "roughnessMap", label: "Rugosidade" },
  { slot: "metalnessMap", label: "Metal" },
  { slot: "specularMap", label: "Especular" },
  { slot: "emissiveMap", label: "Emisión" },
  { slot: "aoMap", label: "Oclusión ambiental" },
  { slot: "lightMap", label: "Mapa de luz" },
  { slot: "alphaMap", label: "Opacidade" },
];

/** Unha textura serve se a súa imaxe cargou de verdade. */
export function textureOk(t: any): boolean {
  const i = t?.image;
  if (!i) return false;
  return (i.width || i.naturalWidth || i.videoWidth || 0) > 0;
}

export function materialsOf(o: any): any[] {
  return Array.isArray(o.material) ? o.material : o.material ? [o.material] : [];
}

export interface MaterialReport {
  /** O material en si, para poder isolalo no visor. */
  ref: any;
  name: string;
  color?: string;
  slots: { slot: string; label: string; ok: boolean }[];
}

/** Resumo dos materiais (únicos) do modelo e do estado das súas texturas. */
export function inspectMaterials(obj: THREE.Object3D): MaterialReport[] {
  const seen = new Set<any>();
  const out: MaterialReport[] = [];
  obj.traverse((o: any) => {
    for (const m of materialsOf(o)) {
      if (seen.has(m)) continue;
      seen.add(m);
      out.push({
        ref: m,
        name: m.name || tr("(sen nome)"),
        color: m.color ? `#${m.color.getHexString()}` : undefined,
        slots: TEXTURE_SLOTS.filter(({ slot }) => m[slot]).map(({ slot, label }) => ({ slot, label, ok: textureOk(m[slot]) })),
      });
    }
  });
  return out;
}

/** Quita texturas rotas (se non, a exportación falla e o visor pinta negro). */
export function stripBrokenTextures(obj: THREE.Object3D): number {
  let n = 0;
  obj.traverse((o: any) => {
    for (const m of materialsOf(o)) {
      for (const { slot } of TEXTURE_SLOTS) {
        if (m[slot] && !textureOk(m[slot])) {
          m[slot] = null;
          m.needsUpdate = true;
          n++;
        }
      }
    }
  });
  return n;
}

/**
 * Un material cunha textura que non cargou, ou cunha cor negra pura e nada
 * máis, pinta o modelo de negro. Arránxase cun gris claro neutro.
 */
export function repairMaterials(obj: THREE.Object3D): string[] {
  const notes: string[] = [];
  const broken = stripBrokenTextures(obj);
  if (broken) notes.push(tr("Texturas sen cargar quitadas: {n}", { n: broken }));
  let darkened = 0;
  const seen = new Set<any>();
  obj.traverse((o: any) => {
    for (const m of materialsOf(o)) {
      if (seen.has(m)) continue;
      seen.add(m);
      if (m.color && m.color.r + m.color.g + m.color.b === 0 && !m.map && !m.emissiveMap) {
        m.color.set(0xcccccc);
        m.needsUpdate = true;
        darkened++;
      }
    }
  });
  if (darkened) notes.push(tr("Materiais de cor negra postos en gris: {n}", { n: darkened }));
  return notes;
}

/**
 * OBJ compacto: vértices, UV e normais sen repetir (o exportador de three.js
 * escribe cada triángulo con todos os seus datos e a precisión completa, o
 * que infla o ficheiro moitas veces) e con 5 decimais.
 */
function objText(root: THREE.Object3D): string {
  root.updateMatrixWorld(true);
  const vs: string[] = [];
  const ts: string[] = [];
  const ns: string[] = [];
  const faces: string[] = ["# Arquivos do Trasno"];
  const vMap = new Map<string, number>();
  const tMap = new Map<string, number>();
  const nMap = new Map<string, number>();
  const fmt = (x: number) => String(+x.toFixed(5));
  const index = (map: Map<string, number>, list: string[], key: string, line: string) => {
    let i = map.get(key);
    if (i === undefined) {
      list.push(line);
      i = list.length;
      map.set(key, i);
    }
    return i;
  };
  const p = new THREE.Vector3();
  const n = new THREE.Vector3();
  const nm = new THREE.Matrix3();
  let counter = 0;

  root.traverse((o: any) => {
    if (!o.isMesh || !o.geometry?.attributes?.position) return;
    const g = o.geometry as THREE.BufferGeometry;
    const pos = g.attributes.position;
    const uv = g.attributes.uv;
    const nor = g.attributes.normal;
    nm.getNormalMatrix(o.matrixWorld);
    faces.push(`o ${(o.name || `malla${++counter}`).replace(/\s+/g, "_")}`);
    const ref = (k: number) => {
      const vi = g.index ? g.index.getX(k) : k;
      p.fromBufferAttribute(pos, vi).applyMatrix4(o.matrixWorld);
      const pl = `${fmt(p.x)} ${fmt(p.y)} ${fmt(p.z)}`;
      const a = index(vMap, vs, pl, `v ${pl}`);
      let t = "";
      let nn = "";
      if (uv) {
        const tl = `${fmt(uv.getX(vi))} ${fmt(uv.getY(vi))}`;
        t = String(index(tMap, ts, tl, `vt ${tl}`));
      }
      if (nor) {
        n.fromBufferAttribute(nor, vi).applyMatrix3(nm).normalize();
        const nl = `${fmt(n.x)} ${fmt(n.y)} ${fmt(n.z)}`;
        nn = String(index(nMap, ns, nl, `vn ${nl}`));
      }
      return nn ? `${a}/${t}/${nn}` : t ? `${a}/${t}` : `${a}`;
    };
    const count = g.index ? g.index.count : pos.count;
    for (let k = 0; k + 2 < count; k += 3) {
      faces.push(`f ${ref(k)} ${ref(k + 1)} ${ref(k + 2)}`);
    }
  });
  return [...vs, ...ts, ...ns, ...faces].join("\n") + "\n";
}

async function exportBytes(obj: THREE.Object3D, ext: string): Promise<Uint8Array> {
  switch (ext) {
    case "obj":
      return new TextEncoder().encode(objText(obj));
    case "stl": {
      const dv = new STLExporter().parse(obj, { binary: true }) as unknown as DataView;
      return new Uint8Array(dv.buffer, dv.byteOffset, dv.byteLength);
    }
    case "ply":
      return new Promise((resolve, reject) => {
        new PLYExporter().parse(
          obj,
          (res) => resolve(typeof res === "string" ? new TextEncoder().encode(res) : new Uint8Array(res as ArrayBuffer)),
          { binary: true },
        );
        setTimeout(() => reject(new Error(tr("A exportación a PLY non respondeu."))), 60_000);
      });
    case "glb": {
      const buf = (await new GLTFExporter().parseAsync(obj, { binary: true })) as ArrayBuffer;
      return new Uint8Array(buf);
    }
    default:
      throw new Error(tr("Non sei exportar a {ext}.", { ext }));
  }
}

function enc(s: string): string {
  return encodeURIComponent(s);
}

/** Converte un modelo a outro formato e gárdao de forma atómica (sen pisar nada). */
export async function convertModel(input: string, inputExt: string, output: string, outputExt: string, overwrite: boolean): Promise<number> {
  const obj = await loadModel(input, inputExt);
  stripBrokenTextures(obj);
  const bytes = await exportBytes(obj, outputExt);
  obj.traverse((o: any) => {
    o.geometry?.dispose?.();
  });
  await call("write_bytes", bytes, {
    headers: { "x-path": enc(output), "x-source": enc(input), "x-overwrite": overwrite ? "1" : "0" },
  });
  return bytes.byteLength;
}

// ---------------------------------------------------------------- texturas importadas

/** Textura que o usuario engadiu a un modelo (non vai dentro do ficheiro do modelo). */
export interface TextureImport {
  slot: string;
  path: string;
  /** Índice do material (na lista de `inspectMaterials`) ou null para todos. */
  material: number | null;
}

export const textureImports = writable<Record<string, TextureImport[]>>({});

export function addTextureImport(model: string, t: TextureImport) {
  textureImports.update((all) => ({ ...all, [model]: [...(all[model] ?? []), t] }));
}

export function removeTextureImport(model: string, index: number) {
  textureImports.update((all) => ({ ...all, [model]: (all[model] ?? []).filter((_, i) => i !== index) }));
}

const NEEDS_STANDARD = new Set(["roughnessMap", "metalnessMap"]);

function uniqueMaterials(obj: THREE.Object3D): any[] {
  const seen = new Set<any>();
  const list: any[] = [];
  obj.traverse((o: any) => {
    for (const m of materialsOf(o)) {
      if (!seen.has(m)) {
        seen.add(m);
        list.push(m);
      }
    }
  });
  return list;
}

/** Cambia un material por outro (en todas as mallas que o usan). */
function swapMaterial(obj: THREE.Object3D, from: any, to: any) {
  obj.traverse((o: any) => {
    if (Array.isArray(o.material)) o.material = o.material.map((m: any) => (m === from ? to : m));
    else if (o.material === from) o.material = to;
  });
}

/** Aplica as texturas que o usuario importou para este modelo. */
export async function applyImports(obj: THREE.Object3D, modelPath: string, ext: string): Promise<string[]> {
  const imports = get(textureImports)[modelPath] ?? [];
  const notes: string[] = [];
  if (!imports.length) return notes;

  let hasUv = true;
  obj.traverse((o: any) => {
    if (o.isMesh && !o.geometry?.attributes?.uv) hasUv = false;
  });
  if (!hasUv) notes.push(tr("Algunhas mallas non teñen coordenadas UV: a textura non se verá nelas"));

  for (const imp of imports) {
    try {
      const tex = await new THREE.TextureLoader().loadAsync(await viewUrl(imp.path));
      tex.colorSpace = imp.slot === "map" || imp.slot === "emissiveMap" ? THREE.SRGBColorSpace : THREE.NoColorSpace;
      // glTF usa as UV sen voltear; o resto (FBX, OBJ...) voltéaas.
      tex.flipY = !(ext === "glb" || ext === "gltf");
      tex.wrapS = tex.wrapT = THREE.RepeatWrapping;
      const mats = uniqueMaterials(obj);
      const targets = imp.material === null ? mats : mats[imp.material] ? [mats[imp.material]] : [];
      for (let m of targets) {
        if (NEEDS_STANDARD.has(imp.slot) && !m.isMeshStandardMaterial) {
          const up = new THREE.MeshStandardMaterial({
            color: m.color ?? 0xffffff,
            map: m.map ?? null,
            normalMap: m.normalMap ?? null,
            side: THREE.DoubleSide,
            name: m.name,
          });
          swapMaterial(obj, m, up);
          m = up;
        }
        m[imp.slot] = tex;
        if (imp.slot === "map" && m.color) m.color.set(0xffffff);
        if (imp.slot === "alphaMap") m.transparent = true;
        m.needsUpdate = true;
      }
    } catch (e) {
      notes.push(tr("Non se puido cargar a textura {name}", { name: imp.path.split(/[\\/]/).pop() ?? imp.path }));
    }
  }
  return notes;
}
