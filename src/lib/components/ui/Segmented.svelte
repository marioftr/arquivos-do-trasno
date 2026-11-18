<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import InfoTip from "./InfoTip.svelte";
  /** «pick»: o usuario premeu unha opción (tamén se xa era a elixida). */
  const dispatch = createEventDispatcher<{ pick: string | number }>();
  /** `detail`: segunda liña pequena dentro do botón (os valores que leva), para que cada opción diga o que fai sen outro texto aparte. */
  export let options: { value: string | number; label: string; hint?: string; detail?: string }[];
  export let value: string | number;
  export let label = "";
  /** Explicación do selector, nunha «i» ao carón do título. Se hai `hint` nas opcións, engádense debaixo, unha por liña. */
  export let info = "";

  $: tip = [info, ...options.filter((o) => o.hint).map((o) => `${o.label}: ${o.hint}`)].filter(Boolean).join(String.fromCharCode(10));
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
  {#if label}<span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>{/if}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="seg" class:two={options.some((o) => o.detail)} role="radiogroup" aria-label={label} on:keydown={key} bind:this={seg}>
    {#each options as o}
      <button role="radio" aria-checked={value === o.value} tabindex={value === o.value ? 0 : -1} class:on={value === o.value} on:click={() => ((value = o.value), dispatch("pick", o.value))}><span>{o.label}</span>{#if o.detail}<small>{o.detail}</small>{/if}</button>
    {/each}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
  /* Se as opcións non caben nunha fila, pasan á seguinte en vez de quedar cortadas. */
  .seg { display: flex; flex-wrap: wrap; gap: 2px; padding: 2px; border: 1px solid var(--border); border-radius: var(--radius-md); min-height: var(--control-h); background: var(--bg-input); }
  .seg.two button { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.1rem; min-height: calc(var(--control-h) + 0.7rem); padding: 0.3em 0.5em; }
  small { font-size: var(--fs-xs); font-weight: 400; color: var(--text-faint); font-variant-numeric: tabular-nums; }
  button.on small { color: var(--text-muted); }
  button { flex: 1 0 auto; min-height: calc(var(--control-h) - 6px); background: transparent; border: none; border-radius: var(--radius-sm); color: var(--text-muted); font-size: var(--fs-md); padding: 0 0.5em; white-space: nowrap; }
  button:hover { color: var(--text); background: var(--bg-hover); }
  button.on { background: var(--bg-hover); color: var(--text); font-weight: 600; box-shadow: 0 0 0 1px var(--border-strong); }
</style>
