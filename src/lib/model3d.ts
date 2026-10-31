// Carga e exportación de modelos 3D co lado web (three.js). O visor e o
// conversor comparten isto: o que se ve é exactamente o que se converte.

import * as THREE from "three";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { FBXLoader } from "three/examples/jsm/loaders/FBXLoader.js";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { OBJLoader } from "three/examples/jsm/loaders/OBJLoader.js";
import { MTLLoader } from "three/examples/jsm/loaders/MTLLoader.js";
import { STLLoader } from "three/examples/jsm/loaders/STLLoader.js";
import { PLYLoader } from "three/examples/jsm/loaders/PLYLoader.js";
import { OBJExporter } from "three/examples/jsm/exporters/OBJExporter.js";
import { STLExporter } from "three/examples/jsm/exporters/STLExporter.js";
import { PLYExporter } from "three/examples/jsm/exporters/PLYExporter.js";
import { GLTFExporter } from "three/examples/jsm/exporters/GLTFExporter.js";

export const MODEL_EXPORT_TARGETS = ["glb", "obj", "stl", "ply"];

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
  await invoke("allow_view_dir", { path });
  const url = convertFileSrc(path);
  const { dir, sep } = split(path);

  const manager = new THREE.LoadingManager();
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
      return new FBXLoader(manager).loadAsync(url);
    case "glb":
    case "gltf":
      return (await new GLTFLoader(manager).loadAsync(url)).scene;
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
      return loader.loadAsync(url);
    }
    case "stl":
      return new THREE.Mesh(await new STLLoader(manager).loadAsync(url), fallbackMaterial());
    case "ply": {
      const geo = await new PLYLoader(manager).loadAsync(url);
      geo.computeVertexNormals();
      return new THREE.Mesh(geo, fallbackMaterial());
    }
    default:
      throw new Error(`formato ${ext} sen visor`);
  }
}

async function exportBytes(obj: THREE.Object3D, ext: string): Promise<Uint8Array> {
  switch (ext) {
    case "obj":
      return new TextEncoder().encode(new OBJExporter().parse(obj));
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
        setTimeout(() => reject(new Error("a exportación PLY non respondeu")), 60_000);
      });
    case "glb": {
      const buf = (await new GLTFExporter().parseAsync(obj, { binary: true })) as ArrayBuffer;
      return new Uint8Array(buf);
    }
    default:
      throw new Error(`non se pode exportar a ${ext}`);
  }
}

function enc(s: string): string {
  return encodeURIComponent(s);
}

/** Converte un modelo a outro formato e gárdao de forma atómica (sen pisar nada). */
export async function convertModel(input: string, inputExt: string, output: string, outputExt: string, overwrite: boolean): Promise<number> {
  const obj = await loadModel(input, inputExt);
  const bytes = await exportBytes(obj, outputExt);
  obj.traverse((o: any) => {
    o.geometry?.dispose?.();
  });
  await invoke("write_bytes", bytes, {
    headers: { "x-path": enc(output), "x-source": enc(input), "x-overwrite": overwrite ? "1" : "0" },
  });
  return bytes.byteLength;
}
