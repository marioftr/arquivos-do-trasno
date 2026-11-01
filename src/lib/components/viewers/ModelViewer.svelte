<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import { errText } from "../../errors";
  import type { FileEntry } from "../../files";
  import { t, tr, lang } from "../../i18n";
  import { fileNameOf, TRASNO } from "../../formats";
  import Slider from "../ui/Slider.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Icon from "../Icon.svelte";
  import { resolvedTheme } from "../../settings";
  import {
    loadModelDetailed,
    fallbackMaterial,
    inspectMaterials,
    materialsOf,
    textureOk,
    TEXTURE_SLOTS,
    textureImports,
    addTextureImport,
    removeTextureImport,
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

  let wireframe = false;
  let grid = true;
  let channel = "full";
  let panelOpen = true;
  /** Índice do material illado (só se ve ese) ou null. */
  let isolated: number | null = null;

  // Luz: dirección (azimut e altura), intensidade e luz ambiente.
  let azimuth = 45;
  let elevation = 50;
  let intensity = 1.6;
  let ambient = 1.1;
  let showMarker = true;

  // Texturas que engade o usuario
  let importSlot = "map";
  let importMaterial: number | "all" = "all";

  const overlays: THREE.Material[] = [];
  const clay = fallbackMaterial();

  $: imports = $textureImports[file.path] ?? [];

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
    ro?.disconnect();
    clear();
    controls?.dispose();
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
  }
  $: if (ready) applyView(channel, wireframe, isolated);
  $: if (ready) updateLight(azimuth, elevation, intensity, ambient, showMarker, radius, center);

  function resize() {
    const w = host.clientWidth;
    const h = host.clientHeight;
    if (!w || !h) return;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
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
    elevation = Math.round((Math.asin(Math.max(-1, Math.min(1, d.y))) * 180) / Math.PI);
    azimuth = Math.round(((Math.atan2(d.x, d.z) * 180) / Math.PI + 360) % 360);
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
      const originals = o.userData?.orig ? (Array.isArray(o.userData.orig) ? o.userData.orig : [o.userData.orig]) : materialsOf(o);
      originals.forEach((m: any) => m.dispose?.());
    });
    model = null;
  }

  /** Aplica a canle (cor, normais...), a malla e o material illado. */
  function applyView(ch: string, wire: boolean, iso: number | null) {
    if (!model) return;
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
      materialsOf(o).forEach((m: any) => (m.wireframe = wire));
      o.visible = !isoRef || orig.includes(isoRef);
    });
  }

  async function load(f: FileEntry, keepView: boolean) {
    status = tr(TRASNO.abrindo);
    stats = "";
    notes = [];
    missing = [];
    channels = [];
    if (!keepView) {
      channel = "full";
      isolated = null;
    }
    clear();
    report = [];
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
      const present = TEXTURE_SLOTS.filter(({ slot }) => report.some((r) => r.slots.some((s) => s.slot === slot && s.ok)));
      hasTextures = present.length > 0;
      channels = [
        { id: "full", label: "Completo" },
        { id: "clay", label: "Sen material (arxila)" },
        ...present.map((p) => ({ id: p.slot, label: p.label })),
      ];
      if (!channels.some((c) => c.id === channel)) channel = "full";
      if (isolated !== null && isolated >= report.length) isolated = null;
      applyView(channel, wireframe, isolated);
      status = "";
    } catch (e) {
      status = tr("Non puiden abrir o modelo: {error}", { error: errText(e) });
    }
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
      camera.position.set(radius * 1.2, radius * 0.9, radius * 1.5);
      controls.target.copy(center);
      controls.update();
    }
  }

  async function importTexture() {
    const picked = await open({
      multiple: false,
      filters: [{ name: tr("Imaxes"), extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tga", "tif", "tiff", "gif"] }],
    });
    if (typeof picked !== "string") return;
    addTextureImport(file.path, { slot: importSlot, path: picked, material: importMaterial === "all" ? null : importMaterial });
    await load(file, true);
  }

  async function removeImport(i: number) {
    removeTextureImport(file.path, i);
    await load(file, true);
  }

  const slotLabel = (slot: string) => TEXTURE_SLOTS.find((s) => s.slot === slot)?.label ?? slot;
</script>

<div class="wrap">
  <div class="canvas" bind:this={host}>
    {#if status}<p class="status muted">{status}</p>{/if}

    <aside class="panel scrollbar-thin" class:closed={!panelOpen}>
      <button class="toggle" on:click={() => (panelOpen = !panelOpen)}>{panelOpen ? $t("Agochar panel") : $t("Panel")}</button>
      {#if panelOpen}
        <h4>{$t("Luz")}</h4>
        <Slider label={$t("Azimut")} unit="°" min={0} max={360} bind:value={azimuth} def={45} />
        <Slider label={$t("Altura")} unit="°" min={-90} max={90} bind:value={elevation} def={50} />
        <Slider label={$t("Forza")} min={0} max={5} step={0.1} bind:value={intensity} def={1.6} />
        <Slider label={$t("Ambiente")} min={0} max={4} step={0.1} bind:value={ambient} def={1.1} />
        <label class="chk"><input type="checkbox" bind:checked={showMarker} /> {$t("Amosar a luz (podes arrastrala)")}</label>

        <h4>{$t("Vista")}</h4>
        <div class="chan">
          {#each channels as c}
            <button class="ch" class:on={channel === c.id} on:click={() => (channel = c.id)}>{$t(c.label)}</button>
          {/each}
        </div>

        {#if report.length}
          <h4>{$t("Materiais")}</h4>
          <p class="muted tiny">{$t("Preme un material para velo só a el.")}</p>
          <ul class="mats">
            {#each report as m, i}
              <li>
                <button class="mat" class:on={isolated === i} on:click={() => (isolated = isolated === i ? null : i)} title={$t("Illar este material")}>
                  <div class="mname">
                    {#if m.color}<span class="sw" style={`background:${m.color}`}></span>{/if}
                    <span class="mn">{m.name}</span>
                    <span class="muted">{m.color ?? ""}</span>
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
                </button>
              </li>
            {/each}
          </ul>
          {#if isolated !== null}
            <button class="btn ghost" on:click={() => (isolated = null)}>{$t("Mostrar todos os materiais")}</button>
          {/if}
        {:else if !status}
          <p class="muted tiny">{$t("Este modelo non ten materiais: o trasno úsao en gris.")}</p>
        {/if}

        <h4>{$t("Texturas")}</h4>
        {#if !hasTextures && !imports.length && !status}
          <p class="muted tiny">{$t("Este modelo non ten texturas: importa unha imaxe para pintalo.")}</p>
        {/if}
        <SelectField
          label={$t("Tipo de textura")}
          bind:value={importSlot}
          options={TEXTURE_SLOTS.map((s) => ({ value: s.slot, label: $t(s.label) }))}
        />
        <SelectField
          label={$t("Aplicar a")}
          bind:value={importMaterial}
          options={[{ value: "all", label: $t("Todos os materiais") }, ...report.map((r, i) => ({ value: i, label: r.name }))]}
        />
        <button class="btn block" on:click={importTexture}><Icon name="plus" size={14} /> {$t("Importar textura…")}</button>
        {#if imports.length}
          <ul class="imps">
            {#each imports as imp, i}
              <li>
                <span class="in" title={imp.path}>{$t(slotLabel(imp.slot))}: {fileNameOf(imp.path)}</span>
                <button class="btn ghost icon" on:click={() => removeImport(i)} aria-label={$t("Quitar")} title={$t("Quitar")}><Icon name="close" size={13} /></button>
              </li>
            {/each}
          </ul>
          <p class="muted tiny">{$t("As texturas importadas tamén se levan ao converter o modelo.")}</p>
        {/if}

        {#if missing.length}
          <p class="warn selectable">{$t("Texturas que o modelo pide e non estaban no cartafol: {list}", { list: missing.join(", ") })}</p>
        {/if}
        {#each notes as n}<p class="muted tiny selectable">· {n}</p>{/each}
      {/if}
    </aside>
  </div>
  <div class="bar">
    <button class="btn" on:click={() => model && frame(model, true)}>{$t("Recentrar")}</button>
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
    position: absolute; top: 10px; right: 10px; width: 280px; max-height: calc(100% - 20px); overflow-y: auto;
    background: color-mix(in srgb, var(--bg-rail) 92%, transparent); border: 1px solid var(--border);
    border-radius: var(--radius-md); padding: 0.7rem 0.9rem 0.9rem; font-size: 0.84rem; backdrop-filter: blur(6px);
    display: flex; flex-direction: column; gap: 0.55rem;
  }
  .panel.closed { width: auto; }
  .toggle { align-self: flex-start; background: none; border: none; color: var(--text-muted); padding: 0; font: inherit; font-size: 0.8rem; }
  .toggle:hover { color: var(--text); }
  h4 { margin: 0.5rem 0 0; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-muted); font-weight: 600; border-top: 1px solid var(--border); padding-top: 0.7rem; }
  .chan { display: flex; flex-wrap: wrap; gap: 0.3rem; }
  .ch { background: transparent; border: 1px solid var(--border-strong); color: var(--text-muted); border-radius: var(--radius-sm); padding: 0.28em 0.6em; font-size: 0.78rem; }
  .ch:hover { color: var(--text); }
  .ch.on { border-color: var(--accent); color: var(--accent); }
  .mats { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.3rem; }
  .mat { width: 100%; text-align: left; background: transparent; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text); padding: 0.45rem 0.6rem; display: flex; flex-direction: column; gap: 0.15rem; }
  .mat:hover { background: var(--bg-hover); border-color: var(--border-strong); }
  .mat.on { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 12%, transparent); }
  .mname { display: flex; align-items: center; gap: 0.45rem; min-width: 0; }
  .mn { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sw { width: 11px; height: 11px; border-radius: 50%; border: 1px solid var(--border-strong); flex-shrink: 0; }
  .slots { display: flex; flex-direction: column; font-size: 0.74rem; color: var(--text-muted); padding-left: 1.05rem; }
  .slots .bad { color: var(--accent-video); }
  .imps { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; }
  .imps li { display: flex; align-items: center; justify-content: space-between; gap: 0.4rem; border-bottom: 1px solid var(--border); }
  .in { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.8rem; }
  .warn { color: var(--accent-pdf); font-size: 0.78rem; margin: 0; word-break: break-word; }
  .tiny { margin: 0; font-size: 0.78rem; }
  .bar { display: flex; align-items: center; gap: 0.9rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .chk { display: inline-flex; align-items: center; gap: 0.4rem; color: var(--text-muted); }
  .stats { margin-left: auto; }
</style>
