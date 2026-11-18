<script lang="ts">
  import InfoTip from "./InfoTip.svelte";

  /** Grupo dun panel con título (e a «i» pegada) e, á dereita, un dato curto (por exemplo, canto hai). */
  export let title: string;
  export let meta = "";
  export let info = "";
</script>

<div class="group">
  <div class="hd">
    <span class="group-title">{title}</span>
    {#if info}<InfoTip text={info} />{/if}
    {#if meta}<span class="meta">{meta}</span>{/if}
  </div>
  <slot />
</div>

<style>
  .hd { display: flex; align-items: center; gap: 0.1rem; }
  .hd .group-title { padding-right: 0; }
  .meta { margin-left: auto; font-size: var(--fs-xs); font-weight: 500; color: var(--text-faint); }
</style>
