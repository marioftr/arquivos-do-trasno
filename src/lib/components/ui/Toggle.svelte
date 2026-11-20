<script lang="ts">
  import InfoTip from "./InfoTip.svelte";
  import { HINTS } from "../../hints";
  import { t } from "../../i18n";
  export let label: string;
  export let checked: boolean;
  export let hint = "";
  export let disabled = false;
  $: tip = hint || (label && HINTS[label] ? $t(HINTS[label]) : "");
</script>

<label class="toggle" class:disabled>
  <span class="lab">{label}{#if tip}<InfoTip text={tip} />{/if}</span>
  <input type="checkbox" role="switch" bind:checked {disabled} aria-label={label} />
  <span class="track"><span class="knob"></span></span>
</label>

<style>
  .toggle { position: relative; display: flex; align-items: center; justify-content: space-between; gap: 0.8rem; min-height: var(--control-h); cursor: pointer; }
  .toggle.disabled { opacity: 0.4; cursor: not-allowed; }
  .lab { display: inline-flex; align-items: center; font-size: var(--fs-md); }
  input { position: absolute; inset: 0; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: pointer; z-index: 1; }
  .toggle.disabled input { cursor: not-allowed; }
  .track { position: relative; width: 34px; height: 18px; border-radius: 3px; background: transparent; border: 1px solid var(--border-strong); transition: background 0.1s ease, border-color 0.1s ease; flex-shrink: 0; box-sizing: border-box; }
  .knob { position: absolute; top: 2px; left: 2px; width: 12px; height: 12px; border-radius: 1px; background: var(--text-muted); transition: transform 0.1s ease, background 0.1s ease; }
  .toggle:hover:not(.disabled) .track { border-color: var(--text-faint); }
  input:checked + .track { background: var(--accent); border-color: var(--accent); }
  input:checked + .track .knob { transform: translateX(16px); background: var(--on-accent); }
  input:focus-visible + .track { outline: 2px solid var(--accent); outline-offset: 2px; }
</style>
