<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import { FBXLoader } from "three/examples/jsm/loaders/FBXLoader.js";
  import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
  import { OBJLoader } from "three/examples/jsm/loaders/OBJLoader.js";
  import { STLLoader } from "three/examples/jsm/loaders/STLLoader.js";
  import { PLYLoader } from "three/examples/jsm/loaders/PLYLoader.js";
  import { viewUrl, type FileEntry } from "../../files";

  export let file: FileEntry;

  let host: HTMLDivElement;
  let renderer: THREE.WebGLRenderer;
  let scene: THREE.Scene;
  let camera: THREE.PerspectiveCamera;
  let controls: OrbitControls;
  let gridHelper: THREE.GridHelper;
  let model: THREE.Object3D | null = null;
  let raf = 0;
  let ro: ResizeObserver;
  let ready = false;
  let status = "";
  let wireframe = false;
  let grid = true;
  let stats = "";

  const fallback = new THREE.MeshStandardMaterial({ color: 0xb8bcc6, roughness: 0.6, metalness: 0.1, side: THREE.DoubleSide });

  onMount(() => {
    scene = new THREE.Scene();
    scene.background = new THREE.Color(0x141417);
    camera = new THREE.PerspectiveCamera(45, 1, 0.01, 10000);
    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(window.devicePixelRatio);
    host.appendChild(renderer.domElement);

    scene.add(new THREE.HemisphereLight(0xffffff, 0x404048, 1.1));
    const sun = new THREE.DirectionalLight(0xffffff, 1.6);
    sun.position.set(3, 6, 4);
    scene.add(sun);
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
  $: if (ready) applyWireframe(wireframe);

  function resize() {
    const w = host.clientWidth;
    const h = host.clientHeight;
    if (!w || !h) return;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }

  function materialsOf(o: any): THREE.Material[] {
    return Array.isArray(o.material) ? o.material : o.material ? [o.material] : [];
  }

  function clear() {
    if (!model) return;
    scene.remove(model);
    model.traverse((o: any) => {
      o.geometry?.dispose?.();
      materialsOf(o).forEach((m) => m.dispose());
    });
    model = null;
  }

  function applyWireframe(on: boolean) {
    model?.traverse((o: any) => materialsOf(o).forEach((m: any) => (m.wireframe = on)));
  }

  async function parse(url: string, ext: string): Promise<THREE.Object3D> {
    switch (ext) {
      case "fbx":
        return new FBXLoader().loadAsync(url);
      case "glb":
      case "gltf":
        return (await new GLTFLoader().loadAsync(url)).scene;
      case "obj":
        return new OBJLoader().loadAsync(url);
      case "stl":
        return new THREE.Mesh(await new STLLoader().loadAsync(url), fallback.clone());
      case "ply": {
        const geo = await new PLYLoader().loadAsync(url);
        geo.computeVertexNormals();
        return new THREE.Mesh(geo, fallback.clone());
      }
      default:
        throw new Error(`formato ${ext} sen visor`);
    }
  }

  async function load(f: FileEntry) {
    status = "O trasno está abrindo o modelo…";
    stats = "";
    clear();
    try {
      const obj = await parse(await viewUrl(f.path), f.ext);
      let meshes = 0;
      let tris = 0;
      obj.traverse((o: any) => {
        if (!o.isMesh) return;
        meshes++;
        const g = o.geometry;
        tris += (g.index ? g.index.count : g.attributes.position.count) / 3;
        if (!o.material) o.material = fallback.clone();
        materialsOf(o).forEach((m: any) => (m.side = THREE.DoubleSide));
      });
      model = obj;
      scene.add(obj);
      frame(obj);
      applyWireframe(wireframe);
      const sz = new THREE.Box3().setFromObject(obj).getSize(new THREE.Vector3());
      const dims = [sz.x, sz.y, sz.z].map((v) => +v.toPrecision(3)).join(" × ");
      stats = `${dims} u · ${meshes} malla${meshes === 1 ? "" : "s"} · ${Math.round(tris).toLocaleString("gl")} triángulos`;
      status = "";
    } catch (e) {
      status = `Non se puido abrir o modelo: ${e}`;
    }
  }

  function frame(obj: THREE.Object3D) {
    obj.position.set(0, 0, 0);
    const box = new THREE.Box3().setFromObject(obj);
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());
    const radius = Math.max(size.x, size.y, size.z) || 1;
    obj.position.set(-center.x, -box.min.y, -center.z);
    gridHelper.scale.setScalar(radius / 5);
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
  </div>
  <div class="bar">
    <button class="btn" on:click={() => model && frame(model)}>Recentrar</button>
    <label class="chk"><input type="checkbox" bind:checked={wireframe} /> Malla</label>
    <label class="chk"><input type="checkbox" bind:checked={grid} /> Cuadrícula</label>
    <span class="muted stats">{stats || "Arrastra para rotar · roda para zoom · botón dereito para mover"}</span>
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .canvas { position: relative; flex: 1; min-height: 0; overflow: hidden; }
  .canvas :global(canvas) { display: block; }
  .status { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; margin: 0; padding: 2rem; text-align: center; pointer-events: none; }
  .bar { display: flex; align-items: center; gap: 0.9rem; padding: 0.5rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.85rem; }
  .chk { display: inline-flex; align-items: center; gap: 0.35rem; color: var(--text-muted); }
  .stats { margin-left: auto; }
</style>
