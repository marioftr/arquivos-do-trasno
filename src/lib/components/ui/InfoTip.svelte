<script lang="ts">
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";

  /** Icono «i»: ao pasar o rato, enfocalo co teclado ou premelo, amosa a explicación do axuste. */
  export let text: string;

  let btn: HTMLButtonElement;
  let open = false;
  let pinned = false;
  let pos = { x: 0, y: 0, below: true };

  function show() {
    const r = btn.getBoundingClientRect();
    const below = r.bottom + 140 < window.innerHeight;
    pos = { x: Math.min(Math.max(r.left + r.width / 2, 150), window.innerWidth - 150), y: below ? r.bottom + 6 : r.top - 6, below };
    open = true;
  }
  const hide = () => {
    if (!pinned) open = false;
  };
</script>

<svelte:window
  on:pointerdown={(e) => {
    if (pinned && !btn?.contains(e.target as Node)) {
      pinned = false;
      open = false;
    }
  }}
  on:keydown={(e) => e.key === "Escape" && ((pinned = false), (open = false))}
/>

<button
  bind:this={btn}
  class="tip"
  type="button"
  aria-label={$t("Que fai isto?")}
  aria-expanded={open}
  on:mouseenter={show}
  on:mouseleave={hide}
  on:focus={show}
  on:blur={hide}
  on:click|stopPropagation|preventDefault={() => {
    pinned = !pinned;
    if (pinned) show();
    else open = false;
  }}
><Icon name="info" size={13} /></button>

{#if open}
  <div class="pop" role="tooltip" style={`left:${pos.x}px;top:${pos.y}px;transform:translate(-50%, ${pos.below ? "0" : "-100%"})`}>{text}</div>
{/if}

<style>
  .tip { display: inline-grid; place-items: center; width: 18px; height: 18px; padding: 0; margin-left: 0.25rem; background: none; border: none; border-radius: 50%; color: var(--text-faint); cursor: help; position: relative; z-index: 2; }
  .tip:hover, .tip[aria-expanded="true"] { color: var(--accent); }
  .pop { position: fixed; z-index: 90; width: max-content; max-width: 280px; padding: 0.5rem 0.65rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4); font-size: 0.8rem; line-height: 1.4; color: var(--text); pointer-events: none; font-weight: 400; text-transform: none; letter-spacing: 0; }
</style>
