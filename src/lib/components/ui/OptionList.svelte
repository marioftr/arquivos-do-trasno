<script lang="ts">
  /** Lista de opcións con punto de selección (tipo radio): pistas, formatos… Funciona co teclado (↑ ↓). */
  export let items: { value: number; label: string; detail?: string; disabled?: boolean }[];
  export let value: number;
  export let label = "";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher<{ change: number }>();
  const pick = (v: number) => {
    value = v;
    dispatch("change", v);
  };

  let box: HTMLDivElement;
  function key(e: KeyboardEvent) {
    const ok = items.filter((i) => !i.disabled);
    const at = ok.findIndex((i) => i.value === value);
    const d = e.key === "ArrowDown" ? 1 : e.key === "ArrowUp" ? -1 : 0;
    if (!d || !ok.length) return;
    e.preventDefault();
    const next = ok[(Math.max(0, at) + d + ok.length) % ok.length];
    pick(next.value);
    requestAnimationFrame(() => box?.querySelector<HTMLElement>('[aria-checked="true"]')?.focus());
  }
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div class="list" role="radiogroup" aria-label={label} on:keydown={key} bind:this={box}>
  {#each items as it (it.value)}
    <button
      class="opt"
      class:on={value === it.value}
      role="radio"
      aria-checked={value === it.value}
      tabindex={value === it.value ? 0 : -1}
      disabled={it.disabled}
      on:click={() => pick(it.value)}
    >
      <span class="dot" aria-hidden="true"></span>
      <span class="txt">
        <span class="l">{it.label}</span>
        {#if it.detail}<span class="d">{it.detail}</span>{/if}
      </span>
    </button>
  {/each}
</div>

<style>
  .list { display: flex; flex-direction: column; gap: 0.15rem; }
  .opt { display: flex; align-items: flex-start; gap: 0.6rem; width: 100%; text-align: left; background: none; border: 1px solid transparent; border-radius: var(--radius-sm); padding: 0.4rem 0.5rem; color: var(--text-muted); }
  .opt:hover:not(:disabled) { background: var(--bg-hover); color: var(--text); }
  .opt:disabled { opacity: 0.45; cursor: not-allowed; }
  .opt.on { background: color-mix(in srgb, var(--accent) 14%, transparent); color: var(--text); }
  .dot { flex: 0 0 12px; height: 12px; margin-top: 3px; border-radius: 50%; border: 1.5px solid var(--text-faint); position: relative; }
  .opt.on .dot { border-color: var(--accent); }
  .opt.on .dot::after { content: ""; position: absolute; inset: 2px; border-radius: 50%; background: var(--accent); }
  .txt { display: flex; flex-direction: column; min-width: 0; }
  .l { font-size: var(--fs-md); overflow: hidden; text-overflow: ellipsis; }
  .d { font-size: var(--fs-xs); color: var(--text-faint); overflow: hidden; text-overflow: ellipsis; }
</style>
