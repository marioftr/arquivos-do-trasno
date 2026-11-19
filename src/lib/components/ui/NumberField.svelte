<script lang="ts">
  import InfoTip from "./InfoTip.svelte";
  import { HINTS } from "../../hints";
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import { sliderDragging } from "./drag";

  export let label = "";
  export let value: number | null;
  export let min: number | undefined = undefined;
  export let max: number | undefined = undefined;
  export let step: number | string = 1;
  export let unit = "";
  export let placeholder = "";
  export let def: number | null | undefined = undefined;
  export let hint = "";
  export let disabled = false;
  /** Se se pasa, avísase en cada cambio (útil para campos ligados entre si). */
  export let onInput: ((raw: string) => void) | undefined = undefined;

  let el: HTMLInputElement;
  $: changed = def !== undefined && value !== def;

  function commit(raw: string) {
    if (onInput) onInput(raw);
    else value = raw === "" ? null : Number(raw);
  }
  function input(e: Event) {
    commit((e.currentTarget as HTMLInputElement).value);
  }
  const clamp = (n: number) => Math.min(max ?? Infinity, Math.max(min ?? -Infinity, n));
  const fmt = (n: number) => String(Math.round(n * 1e6) / 1e6);

  /** As frechas suben ou baixan un paso sen pasar dos límites (se non se admiten negativos, `min` xa os impide). */
  function nudge(dir: 1 | -1) {
    if (disabled) return;
    const s = Number(step) || 1;
    const cur = el.value === "" ? (min ?? 0) - (dir > 0 ? s : 0) : Number(el.value);
    const next = clamp((Number.isFinite(cur) ? cur : 0) + dir * s);
    el.value = fmt(next);
    commit(el.value);
  }
  /** Ao saír do campo, un valor fóra dos límites (por exemplo, un negativo) vólvese ao límite. */
  function leave() {
    sliderDragging.set(false);
    if (el.value === "") return;
    const n = Number(el.value);
    if (Number.isFinite(n) && clamp(n) !== n) {
      el.value = fmt(clamp(n));
      commit(el.value);
    }
  }

  // Manter premida a frecha repite o paso.
  let hold: ReturnType<typeof setTimeout> | undefined;
  let rep: ReturnType<typeof setInterval> | undefined;
  function press(dir: 1 | -1) {
    nudge(dir);
    hold = setTimeout(() => (rep = setInterval(() => nudge(dir), 70)), 420);
  }
  function release() {
    clearTimeout(hold);
    clearInterval(rep);
  }
  function wheelKey(e: KeyboardEvent) {
    if (e.key === "ArrowUp" || e.key === "ArrowDown") {
      e.preventDefault();
      nudge(e.key === "ArrowUp" ? 1 : -1);
    }
  }
  $: tip = hint || (label && HINTS[label] ? $t(HINTS[label]) : "");
</script>

<svelte:window on:pointerup={release} on:pointercancel={release} />

<div class="field">
  {#if label}
    <div class="head">
      <span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>
      {#if changed}
        <button class="reset" on:click={() => (value = def as number | null)} title={$t("Restaurar o valor por defecto ({v})", { v: def ?? "—" })} aria-label={$t("Restaurar o valor por defecto ({v})", { v: def ?? "—" })}>
          <Icon name="reset" size={13} />
        </button>
      {/if}
    </div>
  {/if}
  <label class="num" class:disabled>
    <input bind:this={el} type="number" {min} {max} {step} {placeholder} {disabled} value={value ?? ""} on:input={input} on:keydown={wheelKey} on:focus={() => sliderDragging.set(true)} on:blur={leave} aria-label={label} />
    {#if unit}<span class="unit">{unit}</span>{/if}
    <span class="steps">
      <button type="button" tabindex="-1" {disabled} aria-label={$t("Aumentar")} title={$t("Aumentar")} on:pointerdown|preventDefault={() => press(1)}><Icon name="up" size={10} /></button>
      <button type="button" tabindex="-1" {disabled} aria-label={$t("Reducir")} title={$t("Reducir")} on:pointerdown|preventDefault={() => press(-1)}><Icon name="down" size={10} /></button>
    </span>
  </label>
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .head { display: flex; align-items: center; justify-content: space-between; min-height: 18px; }
  .lab { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
  .reset { display: inline-flex; background: none; border: none; color: var(--text-faint); padding: 2px; border-radius: 4px; }
  .reset:hover { color: var(--accent); background: var(--bg-hover); }
  .num { display: flex; align-items: stretch; height: var(--control-h); background: var(--bg-input); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); box-shadow: var(--groove); overflow: hidden; }
  .num:hover { border-color: var(--text-faint); }
  .num:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent); }
  .num.disabled { opacity: 0.5; }
  .num input { flex: 1; width: 100%; min-width: 0; height: 100%; border: none; background: transparent; padding: 0 0.4em 0 0.65em; text-align: left; -moz-appearance: textfield; appearance: textfield; }
  .num input::-webkit-inner-spin-button, .num input::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
  .num input:focus { outline: none; border: none; box-shadow: none; }
  .unit { align-self: center; padding-right: 0.5em; font-size: var(--fs-sm); color: var(--text-faint); white-space: nowrap; }
  .steps { display: flex; flex-direction: column; width: 22px; flex: none; border-left: 1px solid var(--border); }
  .steps button { flex: 1; display: flex; align-items: center; justify-content: center; padding: 0; background: transparent; border: none; border-radius: 0; color: var(--text-faint); cursor: pointer; min-height: 0; }
  .steps button + button { border-top: 1px solid var(--border); }
  .steps button:hover:not(:disabled) { background: linear-gradient(180deg, var(--key-top), var(--key-bottom)); color: var(--accent); }
  .steps button:active:not(:disabled) { background: color-mix(in srgb, var(--accent) 22%, transparent); }
  .steps button:disabled { cursor: not-allowed; }
</style>
