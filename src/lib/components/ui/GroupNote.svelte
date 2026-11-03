<script lang="ts">
  import InfoTip from "./InfoTip.svelte";

  /** Explicación dun grupo de axustes: en vez de ocupar un parágrafo, vive nunha «i» na esquina do grupo. */
  export let text: string;
</script>

<span class="gn"><InfoTip {text} /></span>

<style>
  .gn { align-self: flex-end; line-height: 1; margin: -0.2rem 0 -0.4rem; }
</style>
