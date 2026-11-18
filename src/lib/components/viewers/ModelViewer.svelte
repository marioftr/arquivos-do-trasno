<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { t, tr, lang } from "../../i18n";
  import { TRASNO } from "../../formats";
  import { get } from "svelte/store";
  import { modelLight, modelShow, modelInfo, modelHidden, modelOptions, type SceneNode } from "../../modelView";
  import { resolvedTheme } from "../../settings";
  import { polygonEdges } from "../../wire";
  import {
    loadModelDetailed,
    fallbackMaterial,
    inspectMaterials,
    materialsOf,
    textureOk,
    TEXTURE_SLOTS,
    textureImports,
    type MaterialReport,
  } from "../../model3d";

  export let file: FileEntry;

  let host: HTMLDivElement;
  let renderer: THREE.WebGLRenderer;
  let scene: THREE.Scene;
  let camera: THREE.PerspectiveCamera;
  let controls: OrbitControls;
  let gridHelper: THREE.GridHelper;
  let hemi: THREE.HemisphereLight;
  let sun: THREE.DirectionalLight;
  let marker: THREE.Mesh;
  let model: THREE.Object3D | null = null;
  let radius = 1;
  let center = new THREE.Vector3();
  let raf = 0;
  let ro: ResizeObserver;
  let ready = false;

  let status = "";
  let stats = "";
  let notes: string[] = [];
  let missing: string[] = [];
  let report: MaterialReport[] = [];
  let channels: { id: string; label: string }[] = [];
  let hasTextures = false;
  let nodes: SceneNode[] = [];

  let wireframe = false;
  let grid = true;
  // Malla e cuadrícula mándanse desde a pestana Escena.
  $: ({ wire: wireframe, grid } = $modelOptions);
  let azimuth = 45, elevation = 50, intensity = 1.6, ambient = 1.1, showMarker = true;
  let channel = "full";
  let isolated: number | null = null;
  // A pestana «Escena» das ferramentas manda sobre a luz, a canle e o material illado (índice ou null).
  $: ({ azimuth, elevation, intensity, ambient, showMarker } = $modelLight);
  $: ({ channel, isolated } = $modelShow);

  const overlays: THREE.Material[] = [];
  const clay = fallbackMaterial();
  const lineMat = new THREE.LineBasicMaterial({ color: 0xdcdce2 });

  // Ao engadir ou quitar unha textura desde a pestana, o modelo recárgase sen mover a cámara.
  $: importsKey = JSON.stringify($textureImports[file.path] ?? []);
  let loadedKey = "";
  let loadedFile = "";
  $: if (ready && importsKey !== loadedKey && loadedFile === file.path) void load(file, true);
  $: modelInfo.set({ nodes, status, report, channels, hasTextures, missing, notes });

  onMount(() => {
    scene = new THREE.Scene();
    scene.background = new THREE.Color(0x141417);
    camera = new THREE.PerspectiveCamera(45, 1, 0.01, 10000);
    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(window.devicePixelRatio);
    host.appendChild(renderer.domElement);

    hemi = new THREE.HemisphereLight(0xffffff, 0x404048, ambient);
    scene.add(hemi);
    sun = new THREE.DirectionalLight(0xffffff, intensity);
    scene.add(sun, sun.target);
    marker = new THREE.Mesh(new THREE.SphereGeometry(1, 20, 14), new THREE.MeshBasicMaterial({ color: 0xffd36a }));
    scene.add(marker);
    gridHelper = new THREE.GridHelper(10, 20, 0x3a3a42, 0x26262c);
    scene.add(gridHelper);

    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;

    const dom = renderer.domElement;
    dom.addEventListener("pointerdown", onDown);
    dom.addEventListener("pointermove", onMove);
    dom.addEventListener("pointerup", onUp);

    ro = new ResizeObserver(resize);
    ro.observe(host);
    resize();
    const tick = () => {
      raf = requestAnimationFrame(tick);
      controls.update();
      renderer.render(scene, camera);
    };
    tick();
    ready = true;
  });

  onDestroy(() => {
    cancelAnimationFrame(raf);
    modelHidden.set([]);
    modelInfo.set({ nodes: [], status: "", report: [], channels: [], hasTextures: false, missing: [], notes: [] });
    modelShow.set({ channel: "full", isolated: null });
    ro?.disconnect();
    clear();
    controls?.dispose();
    lineMat.dispose();
    renderer?.dispose();
  });

  $: if (ready) load(file, false);
  $: if (ready) gridHelper.visible = grid;
  $: if (ready) {
    const light = $resolvedTheme === "light";
    scene.background = new THREE.Color(light ? 0xe4e5ea : 0x141417);
    const mats = (Array.isArray(gridHelper.material) ? gridHelper.material : [gridHelper.material]) as THREE.LineBasicMaterial[];
    mats[0]?.color.set(light ? 0xa9abb6 : 0x3a3a42);
    mats[1]?.color.set(light ? 0xc6c8d1 : 0x26262c);
    lineMat.color.set(light ? 0x1b1c22 : 0xdcdce2);
  }
  $: if (ready) applyView(channel, wireframe, isolated, $modelHidden);
  $: if (ready) updateLight(azimuth, elevation, intensity, ambient, showMarker, radius, center);

  function resize() {
    const w = host.clientWidth;
    const h = host.clientHeight;
    if (!w || !h) return;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    // setSize borra o lenzo: volve debuxar na mesma quenda para que non se vexa un fotograma en branco ao redimensionar.
    controls?.update();
    renderer.render(scene, camera);
  }

  /** Dirección unitaria da luz a partir de azimut e altura. */
  function lightDir(az: number, el: number) {
    const a = (az * Math.PI) / 180;
    const e = (el * Math.PI) / 180;
    return new THREE.Vector3(Math.cos(e) * Math.sin(a), Math.sin(e), Math.cos(e) * Math.cos(a));
  }

  function updateLight(az: number, el: number, k: number, amb: number, mark: boolean, r: number, c: THREE.Vector3) {
    const dir = lightDir(az, el);
    sun.position.copy(c).addScaledVector(dir, r * 3);
    sun.target.position.copy(c);
    sun.intensity = k;
    hemi.intensity = amb;
    // O símbolo da luz vai preto do modelo, onde si se ve, e arrástrase co rato.
    marker.position.copy(c).addScaledVector(dir, r * 1.15);
    marker.scale.setScalar(Math.max(r * 0.05, 0.001));
    marker.visible = mark;
  }

  // ---- arrastrar a luz co rato ----
  const raycaster = new THREE.Raycaster();
  let dragging = false;

  function pointerRay(e: PointerEvent) {
    const rect = renderer.domElement.getBoundingClientRect();
    const ndc = new THREE.Vector2(((e.clientX - rect.left) / rect.width) * 2 - 1, -((e.clientY - rect.top) / rect.height) * 2 + 1);
    raycaster.setFromCamera(ndc, camera);
    return raycaster.ray;
  }

  const overMarker = () => showMarker && raycaster.intersectObject(marker).length > 0;

  function onDown(e: PointerEvent) {
    if (e.button !== 0) return;
    pointerRay(e);
    if (!overMarker()) return;
    dragging = true;
    controls.enabled = false;
    renderer.domElement.setPointerCapture(e.pointerId);
  }

  function onMove(e: PointerEvent) {
    const r = pointerRay(e);
    if (!dragging) {
      renderer.domElement.style.cursor = overMarker() ? "grab" : "";
      return;
    }
    renderer.domElement.style.cursor = "grabbing";
    const sphere = new THREE.Sphere(center, radius * 1.15);
    // Se o rato queda fóra da esfera, vale o punto máis próximo ao raio.
    const point = r.intersectSphere(sphere, new THREE.Vector3()) ?? r.closestPointToPoint(center, new THREE.Vector3());
    const d = point.clone().sub(center).normalize();
    const el = Math.round((Math.asin(Math.max(-1, Math.min(1, d.y))) * 180) / Math.PI);
    const az = Math.round(((Math.atan2(d.x, d.z) * 180) / Math.PI + 360) % 360);
    modelLight.update((l) => ({ ...l, elevation: el, azimuth: az }));
  }

  function onUp(e: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    controls.enabled = true;
    renderer.domElement.style.cursor = "";
    renderer.domElement.releasePointerCapture(e.pointerId);
  }

  function clear() {
    overlays.forEach((m) => m.dispose());
    overlays.length = 0;
    if (!model) return;
    scene.remove(model);
    model.traverse((o: any) => {
      o.geometry?.dispose?.();
      if (o.isLineSegments) return; // a malla de polígonos comparte o material de liñas
      const originals = o.userData?.orig ? (Array.isArray(o.userData.orig) ? o.userData.orig : [o.userData.orig]) : materialsOf(o);
      originals.forEach((m: any) => m.dispose?.());
    });
    model = null;
  }

  /** Aplica a canle (cor, normais...), a malla e o material illado. */
  function applyView(ch: string, wire: boolean, iso: number | null, hiddenIds: string[] = get(modelHidden)) {
    if (!model) return;
    const hidden = new Set(hiddenIds);
    overlays.forEach((m) => m.dispose());
    overlays.length = 0;
    const cache = new Map<any, THREE.Material>();
    const isoRef = iso !== null ? report[iso]?.ref : null;
    model.traverse((o: any) => {
      if (!o.isMesh) return;
      if (!o.userData.orig) o.userData.orig = o.material;
      const isArray = Array.isArray(o.userData.orig);
      const orig: any[] = isArray ? o.userData.orig : [o.userData.orig];
      const pick = (m: any): THREE.Material => {
        if (ch === "full") return m;
        if (ch === "clay") return clay;
        if (cache.has(m)) return cache.get(m)!;
        const tex = m[ch];
        const view = textureOk(tex)
          ? new THREE.MeshBasicMaterial({ map: tex, side: THREE.DoubleSide })
          : new THREE.MeshBasicMaterial({ color: 0x2a2a30, side: THREE.DoubleSide });
        overlays.push(view);
        cache.set(m, view);
        return view;
      };
      const next = orig.map(pick);
      o.material = isArray ? next : next[0];
      // A malla móstrase por polígonos (sen as diagonais dos triángulos); se o modelo é moi grande, triángulo a triángulo.
      if (wire && o.userData.poly === undefined) {
        const seg = polygonEdges(o.geometry);
        if (seg) {
          const g = new THREE.BufferGeometry();
          g.setAttribute("position", new THREE.BufferAttribute(seg, 3));
          o.userData.poly = new THREE.LineSegments(g, lineMat);
          o.add(o.userData.poly);
        } else o.userData.poly = null;
      }
      const poly: THREE.LineSegments | null = o.userData.poly ?? null;
      if (poly) poly.visible = wire;
      orig.forEach((m: any) => (m.visible = true));
      materialsOf(o).forEach((m: any) => {
        m.wireframe = wire && !poly;
        m.visible = !(wire && poly);
      });
      // Agochar unha malla non agocha as que leva dentro: por iso se fai coas capas e non con `visible`.
      const show = !hidden.has(o.uuid) && (!isoRef || orig.includes(isoRef));
      const layer = (x: THREE.Object3D) => (show ? x.layers.enable(0) : x.layers.disable(0));
      layer(o);
      if (poly) layer(poly);
    });
  }

  async function load(f: FileEntry, keepView: boolean) {
    status = tr(TRASNO.abrindo);
    stats = "";
    notes = [];
    missing = [];
    channels = [];
    loadedKey = JSON.stringify($textureImports[f.path] ?? []);
    loadedFile = f.path;
    if (!keepView) modelShow.set({ channel: "full", isolated: null });
    clear();
    report = [];
    nodes = [];
    modelHidden.set([]);
    try {
      const loaded = await loadModelDetailed(f.path, f.ext);
      const obj = loaded.obj;
      let meshes = 0;
      let tris = 0;
      obj.traverse((o: any) => {
        if (!o.isMesh) return;
        meshes++;
        const g = o.geometry;
        tris += (g.index ? g.index.count : g.attributes.position.count) / 3;
        if (!o.material) o.material = fallbackMaterial();
        materialsOf(o).forEach((m: any) => (m.side = THREE.DoubleSide));
      });
      model = obj;
      scene.add(obj);
      frame(obj, !keepView);
      const sz = new THREE.Box3().setFromObject(obj).getSize(new THREE.Vector3());
      const dims = [sz.x, sz.y, sz.z].map((v) => +v.toPrecision(3)).join(" × ");
      stats = tr("{dims} u · mallas: {meshes} · triángulos: {tris}", { dims, meshes, tris: Math.round(tris).toLocaleString($lang) });
      notes = loaded.notes;
      missing = loaded.missing;
      report = inspectMaterials(obj);
      nodes = sceneNodes(obj);
      const present = TEXTURE_SLOTS.filter(({ slot }) => report.some((r) => r.slots.some((s) => s.slot === slot && s.ok)));
      hasTextures = present.length > 0;
      channels = [
        { id: "full", label: "Completo" },
        { id: "clay", label: "Sen material (arxila)" },
        ...present.map((p) => ({ id: p.slot, label: p.label })),
      ];
      if (!channels.some((c) => c.id === channel)) modelShow.update((v) => ({ ...v, channel: "full" }));
      if (isolated !== null && isolated >= report.length) modelShow.update((v) => ({ ...v, isolated: null }));
      applyView(channel, wireframe, isolated);
      status = "";
      if (import.meta.env.DEV) (window as any).__trasnoWire = () => { let lines = 0, shown = 0; const on = (o: any) => { for (let p = o; p; p = p.parent) if (!p.visible) return false; return true; }; model?.traverse((o: any) => { if (o.isLineSegments && o.visible && o.layers.isEnabled(0)) lines += o.geometry.attributes.position.count / 2; if (o.isMesh && on(o) && o.layers.isEnabled(0)) shown++; }); return { lines, shown, tris: Math.round(tris) }; };
    } catch (e) {
      status = tr("Non puiden abrir o modelo: {error}", { error: errText(e) });
    }
  }

  /** Lista de obxectos do ficheiro (grupos e mallas) co seu fondo, para poder agochalos ou velos sós. */
  function sceneNodes(root: THREE.Object3D): SceneNode[] {
    const out: SceneNode[] = [];
    const count = (o: any): { meshes: number; tris: number } => {
      let meshes = 0, tris = 0;
      o.traverse((c: any) => {
        if (!c.isMesh) return;
        meshes++;
        tris += (c.geometry.index ? c.geometry.index.count : c.geometry.attributes.position.count) / 3;
      });
      return { meshes, tris: Math.round(tris) };
    };
    let n = 0;
    const walk = (o: any, depth: number, parent: string | null) => {
      for (const c of o.children) {
        if (c.isLineSegments || c.isLight || c.isCamera) continue;
        const { meshes, tris } = count(c);
        if (!meshes) continue;
        n++;
        out.push({ id: c.uuid, name: c.name || (c.isMesh ? tr("Malla {n}", { n }) : tr("Grupo {n}", { n })), depth, parent, meshes, tris, own: !!c.isMesh });
        walk(c, depth + 1, c.uuid);
      }
    };
    walk(root, 0, null);
    return out;
  }

  function frame(obj: THREE.Object3D, resetCamera: boolean) {
    obj.position.set(0, 0, 0);
    const box = new THREE.Box3().setFromObject(obj);
    const size = box.getSize(new THREE.Vector3());
    const c = box.getCenter(new THREE.Vector3());
    radius = Math.max(size.x, size.y, size.z) || 1;
    obj.position.set(-c.x, -box.min.y, -c.z);
    center = new THREE.Vector3(0, size.y / 2, 0);
    gridHelper.scale.setScalar(radius / 5);
    if (resetCamera) {
      camera.near = radius / 100;
      camera.far = radius * 100;
      camera.updateProjectionMatrix();
      camera.position.set(radius * 1.6, radius * 1.2, radius * 2);
      controls.target.copy(center);
      controls.update();
    }
  }

</script>

<div class="wrap">
  <div class="row">
    <div class="canvas" bind:this={host}>
      {#if status}<p class="status muted">{status}</p>{/if}
    </div>

  </div>
  <div class="bar">
    <button class="btn" title={$t("Axustar o modelo ao espazo dispoñible")} on:click={() => model && frame(model, true)}>{$t("Recentrar")}</button>
    <span class="muted stats">{stats || $t("Arrastra para rotar · roda para zoom · botón dereito para mover")}</span>
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .row { flex: 1; min-height: 0; display: flex; }
  .canvas { position: relative; flex: 1; min-width: 0; min-height: 0; overflow: hidden; }
  .canvas :global(canvas) { display: block; }
  .status { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; margin: 0; padding: 2rem; text-align: center; pointer-events: none; }
  .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.9rem; padding: 0.5rem 0.9rem 0.5rem max(0.9rem, var(--dock-gap, 0px)); border-top: 1px solid var(--border); font-size: var(--fs-md); }
  .stats { margin-left: auto; }
</style>
