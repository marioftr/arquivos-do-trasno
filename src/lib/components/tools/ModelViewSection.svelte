<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { tr, t } from "../../i18n";
  import { fileNameOf } from "../../formats";
  import { TEXTURE_SLOTS, textureImports, addTextureImport, removeTextureImport } from "../../model3d";
  import { modelLight, modelShow, modelInfo, modelHidden, modelOptions, modelFolds, modelCollapsed, nodeCounts, toggleNode, soloNode, isSolo } from "../../modelView";
  import type { FileEntry } from "../../files";
  import Slider from "../ui/Slider.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import InfoTip from "../ui/InfoTip.svelte";
  import FoldGroup from "../ui/FoldGroup.svelte";
  import Icon from "../Icon.svelte";

  /** Escena do modelo 3D: obxectos, aspecto, materiais, texturas e luz. Só cambia o que se ve no visor; non exporta nada. */
  export let file: FileEntry;

  let importSlot = "map";
  let importMaterial: number | "all" = "all";
  let wide = false;
  $: imports = $textureImports[file.path] ?? [];
  $: ({ report, channels, hasTextures, missing, notes, status, nodes } = $modelInfo);

  // Cada obxecto da lista pódese agochar (un grupo vale polas mallas que leva dentro) ou ver só el.
  $: hidden = new Set($modelHidden);
  $: counts = nodeCounts(nodes, hidden);
  $: collapsed = new Set($modelCollapsed);
  $: parents = new Set(nodes.map((n) => n.parent).filter((p): p is string => !!p));
  $: byId = new Map(nodes.map((n) => [n.id, n]));
  $: rows = nodes.filter((n) => {
    for (let p = n.parent; p; p = byId.get(p)?.parent ?? null) if (collapsed.has(p)) return false;
    return true;
  });
  $: totalTris = nodes.filter((n) => !n.parent).reduce((a, n) => a + n.tris, 0);
  const stateOf = (c: { on: number; total: number } | undefined) => (!c || c.on === 0 ? "none" : c.on === c.total ? "all" : "some");
  const flip = (id: string) => modelHidden.set(toggleNode(nodes, hidden, id));
  const solo = (id: string) => modelHidden.set(isSolo(nodes, hidden, id) ? [] : soloNode(nodes, id));
  const fold = (id: string) => modelCollapsed.update((l) => (l.includes(id) ? l.filter((x) => x !== id) : [...l, id]));
  const indeterminate = (el: HTMLInputElement, v: boolean) => {
    el.indeterminate = v;
    return { update: (x: boolean) => (el.indeterminate = x) };
  };

  async function importTexture() {
    const picked = await open({
      multiple: false,
      filters: [{ name: tr("Imaxes"), extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tga", "tif", "tiff", "gif"] }],
    });
    if (typeof picked !== "string") return;
    addTextureImport(file.path, { slot: importSlot, path: picked, material: importMaterial === "all" ? null : importMaterial });
  }
  const slotLabel = (slot: string) => TEXTURE_SLOTS.find((s) => s.slot === slot)?.label ?? slot;
</script>

<div class="stack">
  {#if nodes.length > 1}
    <FoldGroup title={$t("Obxectos")} meta={`${nodes.length} · ${totalTris.toLocaleString()} ${$t("triángulos")}`} bind:open={$modelFolds.objects}>
      <div class="tools">
        <button class="btn ghost icon" on:click={() => modelHidden.set([])} title={$t("Amosar todos os obxectos")} aria-label={$t("Amosar todos os obxectos")}><Icon name="eye" size={15} /></button>
        <button class="btn ghost icon" on:click={() => modelHidden.set(nodes.filter((n) => n.own).map((n) => n.id))} title={$t("Agochar todos os obxectos")} aria-label={$t("Agochar todos os obxectos")}><Icon name="eye-off" size={15} /></button>
        <button class="btn ghost icon" on:click={() => modelCollapsed.set([])} title={$t("Despregar todos os grupos")} aria-label={$t("Despregar todos os grupos")}><Icon name="tri-down" size={15} /></button>
        <button class="btn ghost icon" on:click={() => modelCollapsed.set([...parents])} title={$t("Pregar todos os grupos")} aria-label={$t("Pregar todos os grupos")}><Icon name="tri-right" size={15} /></button>
        <button class="btn ghost icon" class:on={wide} aria-pressed={wide} on:click={() => (wide = !wide)} title={wide ? $t("Limitar a altura da lista") : $t("Ampliar a lista")} aria-label={wide ? $t("Limitar a altura da lista") : $t("Ampliar a lista")}><Icon name="expand" size={15} /></button>
      </div>
      <div class="cols muted"><span>{$t("Obxecto")}</span><span>{$t("Triángulos")}</span></div>
      <ul class="nodes" class:wide>
        {#each rows as n (n.id)}
          {@const st = stateOf(counts.get(n.id))}
          {@const only = isSolo(nodes, hidden, n.id)}
          <li style={`padding-left:${n.depth * 0.9}rem`} class:off={st === "none"}>
            {#if parents.has(n.id)}
              <button class="fold" on:click={() => fold(n.id)} aria-expanded={!collapsed.has(n.id)} aria-label={`${collapsed.has(n.id) ? $t("Despregar") : $t("Pregar")}: ${n.name}`} title={collapsed.has(n.id) ? $t("Despregar") : $t("Pregar")}><Icon name={collapsed.has(n.id) ? "tri-right" : "tri-down"} size={12} /></button>
            {:else}<span class="fold"></span>{/if}
            <label class="node">
              <input type="checkbox" checked={st !== "none"} use:indeterminate={st === "some"} on:change={() => flip(n.id)} aria-label={n.name} />
              <span class="nn" title={n.name}>{n.name}</span>
              <span class="muted nt">{n.tris.toLocaleString()}</span>
            </label>
            <button class="btn ghost icon solo" class:on={only} on:click={() => solo(n.id)} aria-pressed={only} aria-label={`${$t("Ver só este obxecto")}: ${n.name}`} title={$t("Ver só este obxecto")}><Icon name="focus" size={14} /></button>
          </li>
        {/each}
      </ul>
    </FoldGroup>
  {/if}

  <FoldGroup title={$t("Aspecto")} bind:open={$modelFolds.look}>
    <div class="chan">
      {#each channels as c}
        <button class="ch" class:on={$modelShow.channel === c.id} on:click={() => ($modelShow.channel = c.id)}>{$t(c.label)}</button>
      {/each}
    </div>
    <span class="chk-tip"><label class="chk"><input type="checkbox" bind:checked={$modelOptions.wire} /> {$t("Malla")}</label><InfoTip text={$t("Amosa o modelo en arames, só coas arestas das caras (sen as diagonais dos triángulos).")} /></span>
    <label class="chk"><input type="checkbox" bind:checked={$modelOptions.grid} /> {$t("Cuadrícula")}</label>
  </FoldGroup>

  <FoldGroup title={$t("Materiais")} meta={report.length ? String(report.length) : ""} info={report.length ? $t("Preme un material para velo só a el.") : ""} bind:open={$modelFolds.materials}>
    {#if report.length}
      <ul class="mats">
        {#each report as m, i}
          <li>
            <button class="mat" class:on={$modelShow.isolated === i} on:click={() => ($modelShow.isolated = $modelShow.isolated === i ? null : i)}>
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
      {#if $modelShow.isolated !== null}
        <button class="btn ghost" on:click={() => ($modelShow.isolated = null)}>{$t("Mostrar todos os materiais")}</button>
      {/if}
    {:else if !status}
      <p class="muted tiny">{$t("Este modelo non ten materiais: úsoo en gris.")}</p>
    {/if}
  </FoldGroup>

  <FoldGroup title={$t("Texturas")} meta={imports.length ? String(imports.length) : ""} info={$t("As texturas importadas tamén se levan ao converter o modelo.")} bind:open={$modelFolds.textures}>
    {#if !hasTextures && !imports.length && !status}
      <p class="muted tiny">{$t("Este modelo non ten texturas: importa unha imaxe para pintalo.")}</p>
    {/if}
    <SelectField label={$t("Tipo de textura")} bind:value={importSlot} options={TEXTURE_SLOTS.map((s) => ({ value: s.slot, label: $t(s.label) }))} />
    <SelectField label={$t("Aplicar a")} bind:value={importMaterial} options={[{ value: "all", label: $t("Todos os materiais") }, ...report.map((r, i) => ({ value: i, label: r.name }))]} />
    <button class="btn block" on:click={importTexture}><Icon name="plus" size={14} /> {$t("Importar textura…")}</button>
    {#if imports.length}
      <ul class="imps">
        {#each imports as imp, i}
          <li>
            <span class="in" title={imp.path}>{$t(slotLabel(imp.slot))}: {fileNameOf(imp.path)}</span>
            <button class="btn ghost icon" on:click={() => removeTextureImport(file.path, i)} aria-label={$t("Quitar")} title={$t("Quitar")}><Icon name="close" size={13} /></button>
          </li>
        {/each}
      </ul>
    {/if}
    {#if missing.length}
      <p class="warn selectable">{$t("Texturas que o modelo pide e non estaban no cartafol: {list}", { list: missing.join(", ") })}</p>
    {/if}
    {#each notes as n}<p class="muted tiny selectable">·&nbsp;{n}</p>{/each}
  </FoldGroup>

  <FoldGroup title={$t("Luz")} bind:open={$modelFolds.light}>
    <Slider label={$t("Azimut")} unit="°" min={0} max={360} bind:value={$modelLight.azimuth} def={45} />
    <Slider label={$t("Altura")} unit="°" min={-90} max={90} bind:value={$modelLight.elevation} def={50} />
    <Slider label={$t("Forza")} min={0} max={5} step={0.1} bind:value={$modelLight.intensity} def={1.6} />
    <Slider label={$t("Ambiente")} min={0} max={4} step={0.1} bind:value={$modelLight.ambient} def={1.1} />
    <label class="chk"><input type="checkbox" bind:checked={$modelLight.showMarker} /> {$t("Amosar a luz (podes arrastrala)")}</label>
  </FoldGroup>
</div>

<style>
  .chk { display: inline-flex; align-items: center; gap: 0.4rem; color: var(--text-muted); }
  .chk-tip { display: inline-flex; align-items: center; gap: 0.1rem; }
  .chan { display: flex; flex-wrap: wrap; gap: 0.3rem; }
  .ch { background: transparent; border: 1px solid var(--border-strong); color: var(--text-muted); border-radius: var(--radius-sm); padding: 0.28em 0.6em; font-size: var(--fs-sm); }
  .ch:hover { color: var(--text); }
  .ch.on { border-color: var(--accent); color: var(--accent); }
  .tools { display: flex; gap: 0.1rem; align-items: center; }
  .tools .on { color: var(--accent); }
  .cols { display: flex; justify-content: space-between; font-size: var(--fs-xs); padding: 0 2.2rem 0 1.2rem; border-bottom: 1px solid var(--border); margin-bottom: -0.4rem; }
  .nodes { list-style: none; margin: 0; padding: 0; max-height: min(24rem, 55vh); overflow-y: auto; display: flex; flex-direction: column; }
  .nodes.wide { max-height: none; overflow-y: visible; }
  .nodes li { display: flex; align-items: center; gap: 0.2rem; }
  .nodes li.off .nn { color: var(--text-faint); text-decoration: line-through; }
  .fold { width: 1rem; flex-shrink: 0; display: inline-grid; place-items: center; background: none; border: none; padding: 0; color: var(--text-faint); cursor: pointer; }
  button.fold:hover { color: var(--text); }
  .node { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.45rem; min-height: var(--control-h); cursor: pointer; }
  .nn { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .nt { font-size: var(--fs-xs); font-variant-numeric: tabular-nums; }
  .solo { opacity: 0.55; }
  li:hover .solo, .solo.on { opacity: 1; }
  .solo.on { color: var(--accent); }
  .mats { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.3rem; }
  .mat { width: 100%; text-align: left; background: transparent; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text); padding: 0.45rem 0.6rem; display: flex; flex-direction: column; gap: 0.15rem; }
  .mat:hover { background: var(--bg-hover); border-color: var(--border-strong); }
  .mat.on { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 12%, transparent); }
  .mname { display: flex; align-items: center; gap: 0.45rem; min-width: 0; }
  .mn { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sw { width: 11px; height: 11px; border-radius: 50%; border: 1px solid var(--border-strong); flex-shrink: 0; }
  .slots { display: flex; flex-direction: column; font-size: var(--fs-xs); color: var(--text-muted); padding-left: 1.05rem; }
  .slots .bad { color: var(--accent-video); }
  .imps { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; }
  .imps li { display: flex; align-items: center; justify-content: space-between; gap: 0.4rem; border-bottom: 1px solid var(--border); }
  .in { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: var(--fs-sm); }
  .warn { color: var(--accent-pdf); font-size: var(--fs-sm); margin: 0; word-break: break-word; }
  .tiny { margin: 0; font-size: var(--fs-sm); }
</style>
