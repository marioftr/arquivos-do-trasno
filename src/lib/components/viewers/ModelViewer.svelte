<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import type { FileEntry } from "../../files";
  import { t, tr, lang } from "../../i18n";
  import { TRASNO } from "../../formats";
  import {
    loadModelDetailed,
    fallbackMaterial,
    inspectMaterials,
    materialsOf,
    textureOk,
    TEXTURE_SLOTS,
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
  let raf = 0;
  let ro: ResizeObserver;
  let ready = false;

  let status = "";
  let stats = "";
  let notes: string[] = [];
  let missing: string[] = [];
  let report: MaterialReport[] = [];
  let channels: { id: string; label: string }[] = [];

  let wireframe = false;
  let grid = true;
  let channel = "full";
  let panelOpen = true;

  // Luz: dirección (azimut e altura), intensidade e luz ambiente.
  let azimuth = 45;
  let elevation = 50;
  let intensity = 1.6;
  let ambient = 1.1;
  let showMarker = true;

  const overlays: THREE.Material[] = [];
  const clay = fallbackMaterial();

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
    marker = new THREE.Mesh(new THREE.SphereGeometry(1, 16, 12), new THREE.MeshBasicMaterial({ color: 0xffd36a }));
    scene.add(marker);
    gridHelper = new THREE.GridHelper(10, 20, 0x3a3a42, 0x26262c);
    scene.add(gridHelper);

    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;

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
    ro?.disconnect();
    clear();
    controls?.dispose();
    renderer?.dispose();
  });

  $: if (ready) load(file);
  $: if (ready) gridHelper.visible = grid;
  $: if (ready) applyView(channel, wireframe);
  $: if (ready) updateLight(azimuth, elevation, intensity, ambient, showMarker, radius);

  function resize() {
    const w = host.clientWidth;
    const h = host.clientHeight;
    if (!w || !h) return;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }

  function updateLight(az: number, el: number, k: number, amb: number, mark: boolean, r: number) {
    const a = (az * Math.PI) / 180;
    const e = (el * Math.PI) / 180;
    const d = r * 2.5;
    sun.position.set(Math.cos(e) * Math.sin(a) * d, Math.sin(e) * d, Math.cos(e) * Math.cos(a) * d);
    sun.intensity = k;
    hemi.intensity = amb;
    marker.position.copy(sun.position);
    marker.scale.setScalar(r * 0.06);
    marker.visible = mark;
  }

  function clear() {
    overlays.forEach((m) => m.dispose());
    overlays.length = 0;
    if (!model) return;
    scene.remove(model);
    model.traverse((o: any) => {
      o.geometry?.dispose?.();
      const originals = o.userData?.orig ? (Array.isArray(o.userData.orig) ? o.userData.orig : [o.userData.orig]) : materialsOf(o);
      originals.forEach((m: any) => m.dispose?.());
    });
    model = null;
  }

  /** Aplica a canle de visualización (cor, normais, rugosidade...) e a malla. */
  function applyView(ch: string, wire: boolean) {
    if (!model) return;
    overlays.forEach((m) => m.dispose());
    overlays.length = 0;
    const cache = new Map<any, THREE.Material>();
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
      materialsOf(o).forEach((m: any) => (m.wireframe = wire));
    });
  }

  async function load(f: FileEntry) {
    status = tr(TRASNO.abrindo);
    stats = "";
    notes = [];
    missing = [];
    report = [];
    channels = [];
    channel = "full";
    clear();
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
      frame(obj);
      const sz = new THREE.Box3().setFromObject(obj).getSize(new THREE.Vector3());
      const dims = [sz.x, sz.y, sz.z].map((v) => +v.toPrecision(3)).join(" × ");
      stats = tr("{dims} u · mallas: {meshes} · triángulos: {tris}", { dims, meshes, tris: Math.round(tris).toLocaleString($lang) });
      notes = loaded.notes;
      missing = loaded.missing;
      report = inspectMaterials(obj);
      const present = TEXTURE_SLOTS.filter(({ slot }) => report.some((r) => r.slots.some((s) => s.slot === slot && s.ok)));
      channels = [
        { id: "full", label: "Completo" },
        { id: "clay", label: "Sen material (arxila)" },
        ...present.map((p) => ({ id: p.slot, label: p.label })),
      ];
      applyView("full", wireframe);
      status = "";
    } catch (e) {
      status = tr("Non se puido abrir o modelo: {error}", { error: String(e) });
    }
  }

  function frame(obj: THREE.Object3D) {
    obj.position.set(0, 0, 0);
    const box = new THREE.Box3().setFromObject(obj);
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());
    radius = Math.max(size.x, size.y, size.z) || 1;
    obj.position.set(-center.x, -box.min.y, -center.z);
    gridHelper.scale.setScalar(radius / 5);
    sun.target.position.set(0, size.y / 2, 0);
    camera.near = radius / 100;
    camera.far = radius * 100;
    camera.updateProjectionMatrix();
    camera.position.set(radius * 1.2, radius * 0.9, radius * 1.5);
    controls.target.set(0, size.y / 2, 0);
    controls.update();
  }
</script>

<div class="wrap">
  <div class="canvas" bind:this={host}>
    {#if status}<p class="status muted">{status}</p>{/if}

    <aside class="panel scrollbar-thin" class:closed={!panelOpen}>
      <button class="toggle" on:click={() => (panelOpen = !panelOpen)}>{panelOpen ? $t("Agochar panel") : $t("Panel")}</button>
      {#if panelOpen}
        <h4>{$t("Luz")}</h4>
        <div class="row"><span>{$t("Azimut")}</span><input type="range" min="0" max="360" bind:value={azimuth} /><input class="num-input" type="number" min="0" max="360" bind:value={azimuth} /></div>
        <div class="row"><span>{$t("Altura")}</span><input type="range" min="-90" max="90" bind:value={elevation} /><input class="num-input" type="number" min="-90" max="90" bind:value={elevation} /></div>
        <div class="row"><span>{$t("Forza")}</span><input type="range" min="0" max="5" step="0.1" bind:value={intensity} /><input class="num-input" type="number" min="0" max="5" step="0.1" bind:value={intensity} /></div>
        <div class="row"><span>{$t("Ambiente")}</span><input type="range" min="0" max="4" step="0.1" bind:value={ambient} /><input class="num-input" type="number" min="0" max="4" step="0.1" bind:value={ambient} /></div>
        <label class="chk"><input type="checkbox" bind:checked={showMarker} /> {$t("Amosar a luz")}</label>

        <h4>{$t("Vista")}</h4>
        <div class="chan">
          {#each channels as c}
            <button class="ch" class:on={channel === c.id} on:click={() => (channel = c.id)}>{$t(c.label)}</button>
          {/each}
        </div>

        {#if report.length}
          <h4>{$t("Materiais")}</h4>
          <ul class="mats selectable">
            {#each report as m}
              <li>
                <div class="mname">
                  {#if m.color}<span class="sw" style={`background:${m.color}`}></span>{/if}
                  {m.name} <span class="muted">{m.color ?? ""}</span>
                </div>
                {#if m.slots.length}
                  <div class="slots">
                    {#each m.slots as s}
                      <span class:bad={!s.ok}>{$t(s.label)}{s.ok ? "" : ` · ${$t("non cargou")}`}</span>
                    {/each}
                  </div>
                {:else}
                  <div class="slots muted">{$t("sen texturas")}</div>
                {/if}
              </li>
            {/each}
          </ul>
        {:else if !status}
          <p class="muted tiny">{$t("Este modelo non ten materiais: o trasno úsao en gris.")}</p>
        {/if}

        {#if missing.length}
          <p class="warn selectable">{$t("Texturas que o modelo pide e non estaban no cartafol: {list}", { list: missing.join(", ") })}</p>
        {/if}
        {#each notes as n}<p class="muted tiny selectable">· {n}</p>{/each}
      {/if}
    </aside>
  </div>
  <div class="bar">
    <button class="btn" on:click={() => model && frame(model)}>{$t("Recentrar")}</button>
    <label class="chk"><input type="checkbox" bind:checked={wireframe} /> {$t("Malla")}</label>
    <label class="chk"><input type="checkbox" bind:checked={grid} /> {$t("Cuadrícula")}</label>
    <span class="muted stats">{stats || $t("Arrastra para rotar · roda para zoom · botón dereito para mover")}</span>
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .canvas { position: relative; flex: 1; min-height: 0; overflow: hidden; }
  .canvas :global(canvas) { display: block; }
  .status { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; margin: 0; padding: 2rem; text-align: center; pointer-events: none; }
  .panel {
    position: absolute; top: 10px; right: 10px; width: 250px; max-height: calc(100% - 20px); overflow-y: auto;
    background: color-mix(in srgb, var(--bg-rail) 90%, transparent); border: 1px solid var(--border);
    border-radius: var(--radius-md); padding: 0.6rem 0.8rem; font-size: 0.82rem; backdrop-filter: blur(6px);
  }
  .panel.closed { width: auto; }
  .toggle { background: none; border: none; color: var(--text-muted); padding: 0; font: inherit; font-size: 0.78rem; }
  .toggle:hover { color: var(--text); }
  h4 { margin: 0.8rem 0 0.4rem; font-size: 0.68rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-muted); font-weight: 600; }
  .row { display: flex; align-items: center; gap: 0.4rem; margin-bottom: 0.25rem; }
  .row > span { min-width: 3.9rem; color: var(--text-muted); }
  .row input[type="range"] { flex: 1; min-width: 0; }
  .row .num-input { width: 3.8em; }
  .chan { display: flex; flex-wrap: wrap; gap: 0.3rem; }
  .ch { background: transparent; border: 1px solid var(--border-strong); color: var(--text-muted); border-radius: var(--radius-sm); padding: 0.25em 0.55em; font-size: 0.76rem; }
  .ch:hover { color: var(--text); }
  .ch.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .mats { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.45rem; }
  .mname { display: flex; align-items: center; gap: 0.4rem; }
  .sw { width: 10px; height: 10px; border-radius: 2px; border: 1px solid var(--border-strong); flex-shrink: 0; }
  .slots { display: flex; flex-direction: column; font-size: 0.74rem; color: var(--text-muted); padding-left: 1rem; }
  .slots .bad { color: var(--accent-video); }
  .warn { color: var(--accent-pdf); font-size: 0.76rem; margin: 0.6rem 0 0; word-break: break-word; }
  .tiny { margin: 0.2rem 0 0; font-size: 0.76rem; }
  .bar { display: flex; align-items: center; gap: 0.9rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .chk { display: inline-flex; align-items: center; gap: 0.35rem; color: var(--text-muted); }
  .stats { margin-left: auto; }
</style>
