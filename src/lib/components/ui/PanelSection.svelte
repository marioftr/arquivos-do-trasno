<script lang="ts">
  import Icon from "../Icon.svelte";

  /** Sección plegable dun panel: título con frecha e o contido debaixo. */
  export let title: string;
  export let open = true;
  /** Texto pequeno á dereita do título (por exemplo, o número de elementos). */
  export let meta = "";
</script>

<section class:open>
  <button class="head" on:click={() => (open = !open)} aria-expanded={open}>
    <span class="chev"><Icon name={open ? "down" : "arrow-right"} size={12} /></span>
    <span class="tt">{title}</span>
    {#if meta}<span class="meta">{meta}</span>{/if}
  </button>
  {#if open}<div class="in"><slot /></div>{/if}
</section>

<style>
  section { border-bottom: 1px solid var(--border); }
  .head { width: 100%; display: flex; align-items: center; gap: 0.45rem; padding: 0.6rem 0.9rem; background: none; border: none; color: var(--text); text-align: left; }
  .head:hover { background: var(--bg-hover); }
  .chev { color: var(--text-faint); display: inline-flex; }
  .tt { flex: 1; font-size: 0.78rem; font-weight: 650; letter-spacing: 0.02em; }
  .meta { font-size: 0.74rem; color: var(--text-faint); }
  .in { display: flex; flex-direction: column; gap: 0.6rem; padding: 0.2rem 0.9rem 0.9rem; }
</style>
