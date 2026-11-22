<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import InfoTip from "./InfoTip.svelte";
  import Glyph, { type GlyphSpec } from "./Glyph.svelte";
  import Icon from "../Icon.svelte";
  /** «pick»: o usuario premeu unha opción (tamén se xa era a elixida). */
  const dispatch = createEventDispatcher<{ pick: string | number }>();
  /** `detail`: segunda liña pequena dentro do botón (os valores que leva), para que cada opción diga o que fai sen outro texto aparte. */
  /** `glyph`: debuxo enriba do nome, para opcións que se entenden mellor vendo a forma (proporcións, orde, sentido…). */
  /** `icon`: nome dunha icona da aplicación, enriba do nome (para perfís e modos que se recoñecen mellor cunha imaxe). */
  export let options: { value: string | number; label: string; hint?: string; detail?: string; glyph?: GlyphSpec; icon?: string }[];
  export let value: string | number;
  /** Reparto en filas (p. ex. [5, 2]: cinco botóns arriba e dous abaixo, repartindo o ancho). Sen el, os botóns van nunha fila e pasan á seguinte se non caben. */
  export let rows: number[] | null = null;
  export let label = "";
  /** Explicación do selector, nunha «i» ao carón do título. Se hai `hint` nas opcións, engádense debaixo, unha por liña. */
  export let info = "";

  const gcd = (a: number, b: number): number => (b ? gcd(b, a % b) : a);
  $: cols = rows ? rows.reduce((a, n) => (a * n) / gcd(a, n), 1) : 0;
  /** Cuantas columnas ocupa o botón `i` na súa fila. */
  function span(i: number): string {
    if (!rows) return "";
    let left = i;
    for (const n of rows) {
      if (left < n) return `grid-column: span ${cols / n}`;
      left -= n;
    }
    return "";
  }
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
  <div class="seg" class:two={options.some((o) => o.detail || o.glyph || o.icon)} class:glyphs={options.some((o) => o.glyph || o.icon)} class:grid={!!rows} style={rows ? `grid-template-columns: repeat(${cols}, minmax(0, 1fr))` : ""} role="radiogroup" aria-label={label} on:keydown={key} bind:this={seg}>
    {#each options as o, i}
      <button role="radio" style={span(i)} aria-label={o.glyph || o.icon ? o.label : undefined} aria-checked={value === o.value} tabindex={value === o.value ? 0 : -1} class:on={value === o.value} on:click={() => ((value = o.value), dispatch("pick", o.value))}>{#if o.glyph}<Glyph spec={o.glyph} />{:else if o.icon}<Icon name={o.icon} size={20} />{/if}<span>{o.label}</span>{#if o.detail}<small>{o.detail}</small>{/if}</button>
    {/each}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
  /* Se as opcións non caben nunha fila, pasan á seguinte en vez de quedar cortadas. */
  .seg { display: flex; flex-wrap: wrap; gap: 2px; padding: 2px; border: 1px solid var(--border); border-radius: var(--radius-md); min-height: var(--control-h); background: var(--bg-input); }
  .seg.grid { display: grid; }
  .seg.two button { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.1rem; min-height: calc(var(--control-h) + 0.7rem); padding: 0.3em 0.5em; }
  .seg.glyphs button { gap: 0.15rem; padding-top: 0.35em; }
  .seg.glyphs button span { font-size: var(--fs-sm); }
  small { font-size: var(--fs-xs); font-weight: 400; color: var(--text-faint); font-variant-numeric: tabular-nums; }
  button.on small { color: var(--text-muted); }
  button { flex: 1 1 0; min-height: calc(var(--control-h) - 6px); background: transparent; border: none; border-radius: var(--radius-sm); color: var(--text-muted); font-size: var(--fs-md); padding: 0 0.5em; white-space: nowrap; }
  button:hover { color: var(--text); background: var(--bg-hover); }
  button.on { background: var(--bg-hover); color: var(--text); font-weight: 600; box-shadow: 0 0 0 1px var(--border-strong); }
</style>
