<script lang="ts">
  import { onMount } from "svelte";
  import SelectField from "./SelectField.svelte";
  import { t } from "../../i18n";
  import { STANDARD_FAMILIES, sysFonts, systemFamilies, loadFonts } from "../../fonts";

  /** Escolle a familia da letra: tres estándar e as instaladas no sistema. A negriña e a cursiva van á parte. */
  export let label = "";
  export let value: string;
  export let def: string | undefined = undefined;

  onMount(loadFonts);
  $: fams = systemFamilies($sysFonts);
  $: groups = [
    { label: $t("Letras estándar do PDF"), options: STANDARD_FAMILIES.map((f) => ({ value: f.id, label: $t(f.label) })) },
    ...(fams.length ? [{ label: $t("Letras do teu sistema"), options: fams.map((f) => ({ value: `sys:${f}`, label: f })) }] : []),
  ];
</script>

<SelectField {label} bind:value options={[]} {groups} {def} />
