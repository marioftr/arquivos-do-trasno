<script lang="ts">
  import { onMount } from "svelte";
  import SelectField from "./SelectField.svelte";
  import { t } from "../../i18n";
  import { STANDARD_FONTS, sysFonts, loadFonts } from "../../fonts";

  /** Escolle a letra: as 12 estándar do PDF e as instaladas no sistema (que se incrustan no PDF). */
  export let label = "";
  export let value: string;
  export let def: string | undefined = undefined;

  onMount(loadFonts);
  $: groups = [
    { label: $t("Letras estándar do PDF"), options: STANDARD_FONTS.map((f) => ({ value: f.id, label: $t(f.label) })) },
    ...($sysFonts.length ? [{ label: $t("Letras do teu sistema"), options: $sysFonts.map((f) => ({ value: `file:${f.path}`, label: f.name })) }] : []),
  ];
</script>

<SelectField {label} bind:value options={[]} {groups} {def} />
