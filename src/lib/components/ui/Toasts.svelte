<script lang="ts">
  import { toasts, dismiss } from "../../toasts";
  import { t } from "../../i18n";
  import TrasnoSprite from "./TrasnoSprite.svelte";
  import Icon from "../Icon.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each $toasts as n (n.id)}
    <div class="toast" class:warn={n.kind === "warn"} style={`--ms:${n.ms}ms`} role={n.kind === "warn" ? "alert" : "status"}>
      <span class="face"><TrasnoSprite state={n.kind === "warn" ? "warn" : "idle"} size={32} /></span>
      <div class="body">
        <span class="who">{n.kind === "warn" ? $t("O trasno tropezou") : $t("O trasno avisa")}</span>
        <span class="tx">{n.text}</span>
        {#if n.action}
          <button class="act" on:click={() => { n.action?.run(); dismiss(n.id); }}>{n.action.label}</button>
        {/if}
      </div>
      <button class="x" on:click={() => dismiss(n.id)} aria-label={$t("Pechar")} title={$t("Pechar")}><Icon name="close" size={13} /></button>
      <span class="time" aria-hidden="true"></span>
    </div>
  {/each}
</div>

<style>
  .toasts { position: fixed; right: 1rem; bottom: 1rem; z-index: 60; display: flex; flex-direction: column; gap: 0.5rem; align-items: flex-end; pointer-events: none; }
  .toast {
    position: relative; overflow: hidden; pointer-events: auto;
    display: flex; align-items: flex-start; gap: 0.7rem;
    width: min(380px, 92vw); padding: 0.65rem 0.7rem 0.75rem 0.7rem;
    background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.38);
    animation: entra 0.18s ease-out;
  }
  .face { flex-shrink: 0; margin-top: 1px; }
  .body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 0.12rem; }
  .who { font-size: 0.7rem; letter-spacing: 0.04em; text-transform: uppercase; font-weight: 600; color: var(--accent); }
  .toast.warn .who { color: var(--accent-pdf); }
  .tx { font-size: 0.86rem; line-height: 1.35; word-break: break-word; color: var(--text); }
  .act { align-self: flex-start; margin-top: 0.3rem; background: none; border: none; padding: 0; font: inherit; font-size: 0.84rem; font-weight: 600; color: var(--accent); text-decoration: underline; text-underline-offset: 3px; }
  .act:hover { color: var(--text); }
  .x { flex-shrink: 0; display: grid; place-items: center; width: 24px; height: 24px; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-faint); }
  .x:hover { background: var(--bg-hover); color: var(--text); }
  .time { position: absolute; left: 0; bottom: 0; height: 3px; width: 100%; background: var(--accent); opacity: 0.55; transform-origin: left; animation: tempo var(--ms) linear forwards; }
  .toast.warn .time { background: var(--accent-pdf); }
  .toast:hover .time { animation-play-state: paused; }
  @keyframes entra { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: none; } }
  @keyframes tempo { from { transform: scaleX(1); } to { transform: scaleX(0); } }
  @media (prefers-reduced-motion: reduce) { .toast { animation: none; } }
</style>
