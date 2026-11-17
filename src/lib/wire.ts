import type * as THREE from "three";

/** Máis triángulos ca isto e a malla debuxa todos os triángulos (agrupalos tardaría demasiado). */
export const MAX_POLY_TRIS = 400_000;

/**
 * Arestas da malla coma polígonos: os formatos 3D gardan caras de catro ou máis lados, pero ao cargalos
 * o visor sempre as parte en triángulos. Aquí recompoñemos os polígonos: unha aresta compartida por dous
 * triángulos consecutivos e coplanares é a diagonal dun polígono e non se debuxa. Devolve os pares de
 * puntos (x, y, z) das arestas visibles, ou null se a malla é demasiado grande.
 */
export function polygonEdges(geo: THREE.BufferGeometry): Float32Array | null {
  const pos = geo.attributes.position;
  const idx = geo.index;
  const count = idx ? idx.count : pos.count;
  const tris = Math.floor(count / 3);
  if (!pos || tris === 0 || tris > MAX_POLY_TRIS) return null;

  geo.computeBoundingBox();
  const b = geo.boundingBox!;
  const span = Math.max(b.max.x - b.min.x, b.max.y - b.min.y, b.max.z - b.min.z) || 1;
  const q = 1 / (span * 1e-5);

  // Soldadura: vértices na mesma posición contan coma un só (os cargadores repiten vértices por cara).
  const weld = new Map<string, number>();
  const vid = (i: number) => {
    const key = `${Math.round(pos.getX(i) * q)},${Math.round(pos.getY(i) * q)},${Math.round(pos.getZ(i) * q)}`;
    let id = weld.get(key);
    if (id === undefined) weld.set(key, (id = weld.size));
    return id;
  };

  const normals = new Float32Array(tris * 3);
  const flat = new Uint8Array(tris); // 1 se o triángulo ten área
  const edgeAt = new Map<number, number>();
  const first: number[] = [];
  const second: number[] = [];
  const uses: number[] = [];
  const ends: number[] = [];

  const addEdge = (a: number, b2: number, ia: number, ib: number, t: number) => {
    const key = Math.min(a, b2) * 67108864 + Math.max(a, b2);
    const e = edgeAt.get(key);
    if (e === undefined) {
      edgeAt.set(key, first.length);
      first.push(t);
      second.push(-1);
      uses.push(1);
      ends.push(pos.getX(ia), pos.getY(ia), pos.getZ(ia), pos.getX(ib), pos.getY(ib), pos.getZ(ib));
    } else {
      uses[e]++;
      second[e] = t;
    }
  };

  for (let t = 0; t < tris; t++) {
    const i0 = idx ? idx.getX(t * 3) : t * 3;
    const i1 = idx ? idx.getX(t * 3 + 1) : t * 3 + 1;
    const i2 = idx ? idx.getX(t * 3 + 2) : t * 3 + 2;
    const a = vid(i0);
    const b2 = vid(i1);
    const c = vid(i2);
    const ux = pos.getX(i1) - pos.getX(i0), uy = pos.getY(i1) - pos.getY(i0), uz = pos.getZ(i1) - pos.getZ(i0);
    const vx = pos.getX(i2) - pos.getX(i0), vy = pos.getY(i2) - pos.getY(i0), vz = pos.getZ(i2) - pos.getZ(i0);
    const nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
    const len = Math.hypot(nx, ny, nz);
    if (len > 0) {
      normals[t * 3] = nx / len;
      normals[t * 3 + 1] = ny / len;
      normals[t * 3 + 2] = nz / len;
      flat[t] = 1;
    }
    if (a !== b2) addEdge(a, b2, i0, i1, t);
    if (b2 !== c) addEdge(b2, c, i1, i2, t);
    if (c !== a) addEdge(c, a, i2, i0, t);
  }

  const out: number[] = [];
  for (let e = 0; e < first.length; e++) {
    const t1 = first[e];
    const t2 = second[e];
    const diagonal =
      uses[e] === 2 &&
      Math.abs(t1 - t2) === 1 &&
      flat[t1] === 1 &&
      flat[t2] === 1 &&
      normals[t1 * 3] * normals[t2 * 3] + normals[t1 * 3 + 1] * normals[t2 * 3 + 1] + normals[t1 * 3 + 2] * normals[t2 * 3 + 2] > 0.99995;
    if (!diagonal) for (let k = 0; k < 6; k++) out.push(ends[e * 6 + k]);
  }
  return new Float32Array(out);
}
