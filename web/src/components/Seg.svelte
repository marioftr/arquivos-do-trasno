<script lang="ts">
  import Info from "./Info.svelte";
  import type { Choice } from "../lib/tools";

  /** Selector de varias opcións (radios de verdade: leitores de pantalla e teclado funcionan sen engadidos). */
  export let label: string;
  export let name: string;
  export let choices: Choice[];
  export let value: string;
  export let info = "";
  // Poucas opcións e curtas: todas nunha fila; se non, repártense en liñas segundo caiban.
  $: row = choices.length <= 3 && choices.every((c) => (c.label + (c.sub ?? "")).length <= 16);
</script>

<fieldset>
  <legend>{label}{#if info}<Info text={info} />{/if}</legend>
  <div class="seg" class:row style={`--n:${choices.length}`}>
    {#each choices as c (c.value)}
      <label class:on={value === c.value}>
        <input type="radio" {name} value={c.value} bind:group={value} />
        <span class="t">{c.label}</span>
        {#if c.sub}<span class="s">{c.sub}</span>{/if}
      </label>
    {/each}
  </div>
</fieldset>

<style>
  fieldset { border: 0; margin: 0; padding: 0; min-width: 0; }
  legend { padding: 0; margin-bottom: 0.4rem; font-size: var(--fs-sm); font-weight: 600; color: var(--text-muted); display: flex; align-items: center; gap: 0.4rem; }
  .seg { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 7.5rem), 1fr)); gap: 0.5rem; }
  .seg.row { grid-template-columns: repeat(var(--n), minmax(0, 1fr)); }
  label { position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.1rem; min-height: var(--control-h); padding: 0.45rem 0.6rem; text-align: center; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-md); color: var(--text); font-weight: 600; cursor: pointer; user-select: none; -webkit-tap-highlight-color: transparent; }
  label:hover { background: var(--bg-hover); }
  label.on { border-color: var(--accent); box-shadow: inset 0 0 0 1px var(--accent); background: color-mix(in srgb, var(--accent) 12%, var(--bg-card)); }
  input { position: absolute; opacity: 0; inset: 0; margin: 0; cursor: pointer; }
  label:has(input:focus-visible) { outline: 3px solid var(--accent); outline-offset: 2px; }
  .s { font-size: var(--fs-xs); font-weight: 400; color: var(--text-muted); }
</style>
