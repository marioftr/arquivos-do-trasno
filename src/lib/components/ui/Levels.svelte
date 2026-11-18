<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Segmented from "./Segmented.svelte";
  import type { GlyphSpec } from "./Glyph.svelte";
  import { sliderDragging } from "./drag";
  import { t } from "../../i18n";

  type Vals = Record<string, number>;
  /** Selector de niveis con nome sobre uns valores que tamén se poden axustar á man (desprazadores sempre á vista).
   *  O nivel marcado sae dos valores: se coinciden cos dun nivel, ese; se non, «Personalizado». Mentres se move un
   *  desprazador a selección non cambia: só ao soltalo. Premer «Personalizado» déixao fixo ata escoller outro nivel. */
  export let label: string;
  export let info = "";
  export let levels: { id: string; label: string; detail?: string; hint?: string; glyph?: GlyphSpec; values: Vals }[];
  export let values: Vals;
  const dispatch = createEventDispatcher<{ apply: Vals }>();

  let pinned = false;
  let snap = values;
  $: if (!$sliderDragging) snap = values;
  $: auto = levels.find((l) => Object.keys(l.values).every((k) => snap[k] === l.values[k]))?.id ?? "custom";
  $: selected = pinned ? "custom" : auto;
  function pick(id: string) {
    pinned = id === "custom";
    const l = levels.find((x) => x.id === id);
    if (l) dispatch("apply", l.values);
  }
</script>

<Segmented
  {label}
  {info}
  options={[...levels.map((l) => ({ value: l.id, label: l.label, detail: l.detail, hint: l.hint, glyph: l.glyph })), { value: "custom", label: $t("Personalizado") }]}
  value={selected}
  on:pick={(e) => pick(String(e.detail))}
/>
