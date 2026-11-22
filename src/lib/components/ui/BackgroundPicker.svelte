<script lang="ts">
  import { t } from "../../i18n";
  import { rgbOf, type Bg } from "../../transparency";
  import Segmented from "./Segmented.svelte";
  import ColorField from "./ColorField.svelte";

  /** Fondo sobre o que se compón unha imaxe con transparencia. O mesmo selector en Converter, Editar e Quitar fondo. */
  export let value: Bg | "transparent";
  /** Cor de «Personalizado» (hexadecimal). */
  export let color = "#ffffff";
  /** Co fondo transparente como primeira opción (Quitar fondo). */
  export let transparent = false;
  export let label = $t("Fondo");
  export let info = "";

  const hex = (c: string) => {
    const [r, g, b] = rgbOf("color", c);
    return `#${[r, g, b].map((n) => n.toString(16).padStart(2, "0")).join("")}`;
  };
  $: options = [
    ...(transparent ? [{ value: "transparent", label: $t("Transparente"), glyph: { k: "checker" as const } }] : []),
    { value: "white", label: $t("Branco"), glyph: { k: "swatch" as const, color: "#ffffff" } },
    { value: "black", label: $t("Negro"), glyph: { k: "swatch" as const, color: "#000000" } },
    { value: "gray", label: $t("Gris"), glyph: { k: "swatch" as const, color: "#808080" } },
    { value: "color", label: $t("Personalizado"), glyph: value === "color" ? { k: "swatch" as const, color: hex(color) } : { k: "rainbow" as const } },
  ];
</script>

<div class="bgp">
  <Segmented {label} {info} {options} bind:value />
  {#if value === "color"}<ColorField label={$t("Cor do fondo")} bind:value={color} />{/if}
</div>

<style>
  .bgp { display: flex; flex-direction: column; gap: 0.5rem; }
</style>
