<script lang="ts">
  import { t } from "../../i18n";
  import { hideBubble, holdBubble, releaseBubble, type Bubble } from "../../agent";
  import Icon from "../Icon.svelte";

  /** O globo do trasno: na barra de abaixo («foot») ou, se o trasno está na cabeceira dunha xanela, ao seu carón («head»). */
  export let b: Bubble;
  export let placement: "foot" | "head" = "foot";
  /** Se vai, todo o globo é un botón (avisos fixos da barra, sen copiar nin pechar). */
  export let press: (() => void) | undefined = undefined;

  let copied = false;
  async function copy() {
    try {
      await navigator.clipboard.writeText(b.copy ?? "");
      copied = true;
      setTimeout(() => (copied = false), 1400);
    } catch {
      // o texto tamén se pode seleccionar a man
    }
  }
  $: icon = b.kind === "tip" ? "bulb" : b.kind === "warn" ? "warning" : "sparkles";
</script>

{#if press}
  <button class="bubble warnb {placement}" on:click={press}><span class="bi"><Icon name="warning" size={17} /></span><span class="bt">{b.text}</span></button>
{:else}
  <div class="bubble {placement}" class:tipb={b.kind === "tip"} class:warnb={b.kind === "warn"} role="group" on:pointerenter={holdBubble} on:pointerleave={() => releaseBubble()}>
    <span class="bi"><Icon name={icon} size={17} /></span>
    <span class="bt">{#if b.kind === "tip"}<b>{$t("Sabías que…")}</b>{" "}{/if}{$t(b.text)}{#if b.action}{" "}<button class="bact" on:click={() => { b.action?.run(); hideBubble(); }}>{b.action.label}</button>{/if}</span>
    {#if b.copy}<button class="bclose" on:click={copy} aria-label={$t("Copiar")} title={$t("Copiar")}><Icon name={copied ? "check" : "copy"} size={14} /></button>{/if}
    <button class="bclose" on:click={hideBubble} aria-label={$t("Pechar")} title={$t("Pechar")}><Icon name="close" size={13} /></button>
  </div>
{/if}

<style>
  .bubble { --c: var(--accent-conversor); position: absolute; display: flex; align-items: flex-start; gap: 0.65rem; padding: 0.6rem 0.85rem 0.6rem 0.6rem; text-align: left; color: var(--text); background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); font-size: var(--fs-sm); line-height: 1.4; animation: rise 0.25s ease-out; }
  .bubble.foot { left: 116px; bottom: calc(100% + 0.5rem); max-width: min(30rem, calc(100vw - 7rem)); }
  /* Na cabeceira, o globo vai nunha segunda fila, ao carón do trasno, sen tapar nada. */
  .bubble.head { position: relative; flex: 0 0 auto; margin: 0.2rem 0 0.4rem 4.1rem; max-width: calc(100% - 4.1rem); }
  .bubble.tipb { --c: var(--accent); }
  .bubble.warnb { --c: var(--accent-pdf); }
  button.bubble:hover { border-color: var(--c); }
  .bact { background: none; border: none; padding: 0; font: inherit; font-weight: 600; color: var(--c); text-decoration: underline; text-underline-offset: 3px; }
  .bact:hover { color: var(--text); }
  .bclose { flex: none; align-self: flex-start; display: grid; place-items: center; width: 28px; height: 28px; padding: 0; margin: 0 -0.45rem 0 -0.15rem; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-faint); }
  .bclose:hover { background: var(--bg-hover); color: var(--text); }
  .bubble::before { content: ""; position: absolute; width: 10px; height: 10px; background: var(--bg-elevated); transform: rotate(45deg); }
  .foot::before { left: -6px; bottom: 0.75rem; border-left: 1px solid var(--border-strong); border-bottom: 1px solid var(--border-strong); }
  .head::before { left: -6px; top: 0.75rem; border-left: 1px solid var(--border-strong); border-bottom: 1px solid var(--border-strong); }
  @keyframes rise { from { opacity: 0; transform: translateY(6px); } }
  .bi { flex: none; display: grid; place-items: center; width: 28px; height: 28px; border-radius: 8px; background: color-mix(in srgb, var(--c) 16%, transparent); color: var(--c); }
  .bt { min-width: 0; align-self: center; }
  @media (prefers-reduced-motion: reduce) { .bubble { animation: none; } }
</style>
