<script lang="ts">
  import { t } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";

  /** A «i»: abre un texto curto ao premela (tamén co teclado e no móbil, onde non hai «hover»). */
  export let text: string;
  let open = false;
  let root: HTMLElement;
  let button: HTMLButtonElement;
  let place = "";

  // O texto vai fixo e sempre dentro da pantalla, sexa onde sexa a «i».
  function toggle() {
    open = !open;
    if (!open) return;
    const r = button.getBoundingClientRect();
    const w = Math.min(336, innerWidth - 32);
    place = `top:${Math.round(r.bottom + 8)}px;left:${Math.round(Math.max(16, Math.min(r.left - 8, innerWidth - w - 16)))}px;width:${w}px`;
  }
</script>

<svelte:window
  on:pointerdown={(e) => open && root && !root.contains(e.target as Node) && (open = false)}
  on:keydown={(e) => e.key === "Escape" && (open = false)}
  on:scroll={() => (open = false)}
  on:resize={() => (open = false)}
/>

<span class="info" bind:this={root}>
  <button type="button" class="i" bind:this={button} aria-expanded={open} aria-label={$t("Que fai isto?")} on:click={toggle}><Icon name="info" size={18} /></button>
  {#if open}<span class="tip" role="note" style={place}>{text}</span>{/if}
</span>

<style>
  .info { display: inline-flex; vertical-align: middle; }
  .i { display: grid; place-items: center; width: 44px; height: 44px; margin: -12px -8px; padding: 0; background: none; border: 0; border-radius: 50%; color: var(--text-muted); }
  .i:hover { color: var(--text); }
  .i:focus-visible { outline: 3px solid var(--accent); outline-offset: -4px; }
  .tip { position: fixed; z-index: 30; padding: 0.7rem 0.85rem; background: var(--bg-elevated); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); font-size: var(--fs-sm); font-weight: var(--fw-regular); line-height: 1.45; text-align: left; }
</style>
