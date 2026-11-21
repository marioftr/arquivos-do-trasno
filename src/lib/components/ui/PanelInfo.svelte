<script lang="ts">
  import { onDestroy } from "svelte";
  import { panelInfo } from "./panelInfo";

  /** Pon a explicación da ferramenta na «i» do título do panel mentres este compoñente exista. */
  export let text: string;
  $: panelInfo.set(text);
  onDestroy(() => panelInfo.update((t) => (t === text ? "" : t)));
</script>
