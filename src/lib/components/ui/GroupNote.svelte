<script lang="ts">
  import InfoTip from "./InfoTip.svelte";

  /** Explicación dun grupo de axustes: en vez de ocupar un parágrafo, vive nunha «i» na esquina do grupo. */
  export let text: string;
</script>

<span class="gn"><InfoTip {text} /></span>

<style>
  .gn { position: absolute; top: 0.95rem; right: 0; line-height: 1; }
</style>
