<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { tr, t } from "../../i18n";
  import { fileNameOf } from "../../formats";
  import { TEXTURE_SLOTS, textureImports, addTextureImport, removeTextureImport } from "../../model3d";
  import { modelLight, modelShow, modelInfo } from "../../modelView";
  import type { FileEntry } from "../../files";
  import Slider from "../ui/Slider.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import GroupNote from "../ui/GroupNote.svelte";
  import Icon from "../Icon.svelte";

  /** Vista do modelo 3D: luz, canles, materiais e texturas. Só cambia o que se ve no visor; non exporta nada. */
  export let file: FileEntry;

  let importSlot = "map";
  let importMaterial: number | "all" = "all";
  $: imports = $textureImports[file.path] ?? [];
  $: ({ report, channels, hasTextures, missing, notes, status } = $modelInfo);

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
  <div class="group">
    <h4 class="group-title">{$t("Luz")}</h4>
    <Slider label={$t("Azimut")} unit="°" min={0} max={360} bind:value={$modelLight.azimuth} def={45} />
    <Slider label={$t("Altura")} unit="°" min={-90} max={90} bind:value={$modelLight.elevation} def={50} />
    <Slider label={$t("Forza")} min={0} max={5} step={0.1} bind:value={$modelLight.intensity} def={1.6} />
    <Slider label={$t("Ambiente")} min={0} max={4} step={0.1} bind:value={$modelLight.ambient} def={1.1} />
    <label class="chk"><input type="checkbox" bind:checked={$modelLight.showMarker} /> {$t("Amosar a luz (podes arrastrala)")}</label>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Vista")}</h4>
    <div class="chan">
      {#each channels as c}
        <button class="ch" class:on={$modelShow.channel === c.id} on:click={() => ($modelShow.channel = c.id)}>{$t(c.label)}</button>
      {/each}
    </div>
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Materiais")}{report.length ? ` · ${report.length}` : ""}</h4>
    {#if report.length}
      <GroupNote text={$t("Preme un material para velo só a el.")} />
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
  </div>

  <div class="group">
    <h4 class="group-title">{$t("Texturas")}</h4>
    <GroupNote text={$t("As texturas importadas tamén se levan ao converter o modelo.")} />
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
  </div>
</div>

<style>
  .chk { display: inline-flex; align-items: center; gap: 0.4rem; color: var(--text-muted); }
  .chan { display: flex; flex-wrap: wrap; gap: 0.3rem; }
  .ch { background: transparent; border: 1px solid var(--border-strong); color: var(--text-muted); border-radius: var(--radius-sm); padding: 0.28em 0.6em; font-size: var(--fs-sm); }
  .ch:hover { color: var(--text); }
  .ch.on { border-color: var(--accent); color: var(--accent); }
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
