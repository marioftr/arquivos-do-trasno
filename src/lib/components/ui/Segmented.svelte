<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import InfoTip from "./InfoTip.svelte";
  import Glyph, { type GlyphSpec } from "./Glyph.svelte";
  /** «pick»: o usuario premeu unha opción (tamén se xa era a elixida). */
  const dispatch = createEventDispatcher<{ pick: string | number }>();
  /** `detail`: segunda liña pequena dentro do botón (os valores que leva), para que cada opción diga o que fai sen outro texto aparte. */
  /** `glyph`: debuxo enriba do nome, para opcións que se entenden mellor vendo a forma (proporcións, orde, sentido…). */
  export let options: { value: string | number; label: string; hint?: string; detail?: string; glyph?: GlyphSpec }[];
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
  <div class="seg" class:two={options.some((o) => o.detail || o.glyph)} class:glyphs={options.some((o) => o.glyph)} role="radiogroup" aria-label={label} on:keydown={key} bind:this={seg}>
    {#each options as o}
      <button role="radio" aria-label={o.glyph ? o.label : undefined} aria-checked={value === o.value} tabindex={value === o.value ? 0 : -1} class:on={value === o.value} on:click={() => ((value = o.value), dispatch("pick", o.value))}>{#if o.glyph}<Glyph spec={o.glyph} />{/if}<span>{o.label}</span>{#if o.detail}<small>{o.detail}</small>{/if}</button>
    {/each}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; gap: 0.2rem; }
  .lab { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
  /* Se as opcións non caben nunha fila, pasan á seguinte en vez de quedar cortadas. */
  .seg { display: flex; flex-wrap: wrap; gap: 3px; padding: 3px; border: 1px solid var(--border); border-radius: var(--radius-md); min-height: var(--control-h); background: var(--groove-bg); box-shadow: var(--groove); }
  .seg.two button { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.1rem; min-height: calc(var(--control-h) + 0.7rem); padding: 0.3em 0.5em; }
  .seg.glyphs button { gap: 0.15rem; min-width: 4.2rem; padding-top: 0.35em; }
  .seg.glyphs button span { font-size: var(--fs-sm); }
  small { font-size: var(--fs-xs); font-weight: 400; color: var(--text-faint); font-variant-numeric: tabular-nums; }
  button.on small { color: var(--text-muted); }
  button { flex: 1 0 auto; min-height: calc(var(--control-h) - 6px); background: transparent; border: none; border-radius: var(--radius-sm); color: var(--text-muted); font-size: var(--fs-md); padding: 0 0.5em; white-space: nowrap; }
  button { transition: background 0.1s ease, color 0.1s ease, box-shadow 0.1s ease, transform 0.08s ease; }
  button:hover { color: var(--text); background: color-mix(in srgb, var(--bg-hover) 70%, transparent); }
  /* A opción elixida é unha tecla pulsada: corpo con brillo e unha franxa de acento por baixo. */
  button.on { background: linear-gradient(180deg, var(--key-top), var(--key-bottom)); color: var(--text); font-weight: 600; box-shadow: inset 0 1px 0 var(--key-hi), inset 0 -2px 0 var(--accent), 0 1px 0 var(--key-edge); }
  button:active:not(.on) { transform: translateY(1px); }
</style>
