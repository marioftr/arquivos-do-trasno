<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import InfoTip from "./InfoTip.svelte";
  /** «pick»: o usuario premeu unha opción (tamén se xa era a elixida). */
  const dispatch = createEventDispatcher<{ pick: string | number }>();
  export let options: { value: string | number; label: string; hint?: string }[];
  export let value: string | number;
  export let label = "";
  export let info = "";

  let seg: HTMLDivElement;
  function key(e: KeyboardEvent) {
    const i = Math.max(0, options.findIndex((o) => o.value === value));
    const d = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const j = (i + d + options.length) % options.length;
    value = options[j].value;
    requestAnimationFrame(() => seg?.querySelectorAll<HTMLElement>("button")[j]?.focus());
  }
</script>

<div class="wrap">
  {#if label}<span class="lab">{label}{#if info}<InfoTip text={info} />{/if}</span>{/if}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="seg" role="radiogroup" aria-label={label} on:keydown={key} bind:this={seg}>
    {#each options as o}
      <button role="radio" aria-checked={value === o.value} tabindex={value === o.value ? 0 : -1} class:on={value === o.value} title={o.hint} on:click={() => ((value = o.value), dispatch("pick", o.value))}>{o.label}</button>
    {/each}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
  .seg { display: flex; gap: 2px; padding: 2px; border: 1px solid var(--border); border-radius: var(--radius-md); height: var(--control-h); background: var(--bg-input); }
  button { flex: 1; background: transparent; border: none; border-radius: var(--radius-sm); color: var(--text-muted); font-size: var(--fs-md); padding: 0 0.5em; white-space: nowrap; }
  button:hover { color: var(--text); background: var(--bg-hover); }
  button.on { background: var(--bg-hover); color: var(--text); font-weight: 600; box-shadow: 0 0 0 1px var(--border-strong); }
</style>
