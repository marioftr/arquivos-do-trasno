<script lang="ts">
  import Toggle from "./Toggle.svelte";
  import GroupNote from "./GroupNote.svelte";
  import { t } from "../../i18n";

  /** Levar os metadatos da foto (cámara, data…) ao resultado. */
  export let keep: boolean;
  /** Levar tamén a ubicación (só ten efecto se `keep`). */
  export let gps: boolean;
  /** Se o formato de saída os pode levar (JPG, PNG e WebP). */
  export let supported = true;
  export let disabled = false;
</script>

{#if supported}
  <div class="meta">
    <Toggle label={$t("Manter metadatos")} bind:checked={keep} {disabled} hint={$t("Cámara, data e datos da toma, sen número de serie nin dono. O perfil de cor vai sempre.")} />
    <Toggle label={$t("Manter GPS")} bind:checked={gps} disabled={disabled || !keep} hint={$t("Ubicación onde se fixo a foto: quen a reciba poderá vela. Desactívao se vas compartila.")} />
  </div>
{:else}
  <GroupNote text={$t("Neste formato non se poden levar os metadatos da foto (só JPG, PNG e WebP). O perfil de cor tampouco.")} />
{/if}

<style>
  .meta { display: flex; flex-direction: column; gap: 0.15rem; }
</style>
