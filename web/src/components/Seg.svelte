<script lang="ts">
  import { t } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import Info from "./Info.svelte";
  import type { Choice } from "../lib/tools";

  /** Selector de varias opcións (radios de verdade: leitores de pantalla e teclado funcionan sen engadidos). */
  export let label: string;
  export let name: string;
  export let choices: Choice[];
  export let value: string;
  export let info = "";
  /** Modelos de IA xa descargados: as opcións que os usan amósano en vez do tamaño. */
  export let ready: Set<string> = new Set();
  // Poucas opcións e curtas: todas nunha fila; se non, repártense de dúas en dúas e a última estírase se queda soa.
  $: row = choices.length <= 3 && choices.every((c) => (c.label + (c.sub ?? "")).length <= 16);
  // Con detalle debaixo en algunha, todas teñen a mesma altura.
  $: tall = choices.some((c) => c.sub || c.model);
</script>

<fieldset>
  <legend>{label}{#if info}<Info text={info} />{/if}</legend>
  <div class="seg" class:row>
    {#each choices as c (c.value)}
      <label class:on={value === c.value} class:tall>
        <input type="radio" {name} value={c.value} bind:group={value} />
        <span class="t">{c.label}</span>
        {#if c.model && ready.has(c.model)}
          <span class="s done"><Icon name="check" size={14} />{$t("Descargado")}</span>
        {:else if c.sub}
          <span class="s">{c.sub}</span>
        {/if}
      </label>
    {/each}
  </div>
</fieldset>

<style>
  fieldset { border: 0; margin: 0; padding: 0; min-width: 0; }
  legend { padding: 0; margin-bottom: 0.45rem; font-size: var(--fs-sm); font-weight: 600; color: var(--text-muted); display: flex; align-items: center; gap: 0.4rem; }
  .seg { display: flex; flex-wrap: wrap; gap: 0.5rem; }
  label { position: relative; flex: 1 1 calc(50% - 0.25rem); min-width: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.1rem; min-height: var(--control-h); padding: 0.4rem 0.6rem; text-align: center; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-md); color: var(--text); font-weight: 600; line-height: 1.25; cursor: pointer; user-select: none; -webkit-tap-highlight-color: transparent; }
  .row label { flex: 1 1 0; }
  label.tall { min-height: 3.6rem; }
  label:hover { background: var(--bg-hover); }
  label.on { border-color: var(--accent); box-shadow: inset 0 0 0 1px var(--accent); background: color-mix(in srgb, var(--accent) 12%, var(--bg-card)); }
  input { position: absolute; opacity: 0; inset: 0; margin: 0; cursor: pointer; }
  label:has(input:focus-visible) { outline: 3px solid var(--accent); outline-offset: 2px; }
  .s { font-size: var(--fs-xs); font-weight: 400; color: var(--text-muted); }
  .s.done { display: inline-flex; align-items: center; gap: 0.25rem; font-weight: 600; color: var(--accent); }
</style>
