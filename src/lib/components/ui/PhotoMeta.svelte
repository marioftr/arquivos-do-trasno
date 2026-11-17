<script lang="ts">
  import Toggle from "./Toggle.svelte";
  import GroupNote from "./GroupNote.svelte";
  import { t } from "../../i18n";

  /** Levar os metadatos da foto (cámara, data…) ao resultado. */
  export let keep: boolean;
  /** Levar tamén a ubicación (só se `keep`). */
  export let gps: boolean;
  /** Se o formato de saída os pode levar (JPG, PNG e WebP). */
  export let supported = true;
  export let disabled = false;
</script>

{#if supported}
  <Toggle
    label={$t("Manter os metadatos da foto")}
    bind:checked={keep}
    {disabled}
    hint={$t("Cámara, data e datos da toma (sen número de serie nin dono). Por defecto non se levan ao resultado; o perfil de cor vai sempre.")}
  />
  {#if keep}
    <Toggle
      label={$t("Manter tamén a ubicación (GPS)")}
      bind:checked={gps}
      {disabled}
      hint={$t("Quen reciba a foto poderá ver onde se fixo. Mellor desactivado se vas compartila.")}
    />
  {/if}
{:else}
  <GroupNote text={$t("Neste formato non se poden levar os metadatos da foto (só JPG, PNG e WebP). O perfil de cor tampouco.")} />
{/if}
