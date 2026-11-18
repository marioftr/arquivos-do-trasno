<script lang="ts">
  import Icon from "../Icon.svelte";
  import InfoTip from "./InfoTip.svelte";

  /** Grupo dun panel con título que se pode pregar: cada categoría vai na súa propia xanela. */
  export let title: string;
  export let open = true;
  /** Dato curto á dereita do título (por exemplo, canto hai). */
  export let meta = "";
  /** Explicación do grupo: a «i» vai pegada ao título. */
  export let info = "";
</script>

<div class="group fold">
  <div class="hd">
    <button class="group-title head" type="button" aria-expanded={open} on:click={() => (open = !open)}>
      <Icon name={open ? "tri-down" : "tri-right"} size={12} />
      <span>{title}</span>
    </button>
    {#if info}<InfoTip text={info} />{/if}
    {#if meta}<span class="meta">{meta}</span>{/if}
  </div>
  {#if open}<slot />{/if}
</div>

<style>
  .hd { display: flex; align-items: center; gap: 0.1rem; }
  .head { display: flex; align-items: center; gap: 0.4rem; background: none; border: none; padding: 0; text-align: left; cursor: pointer; font-family: inherit; }
  .head:hover { color: var(--text); }
  .meta { margin-left: auto; font-size: var(--fs-xs); font-weight: 500; color: var(--text-faint); }
</style>
