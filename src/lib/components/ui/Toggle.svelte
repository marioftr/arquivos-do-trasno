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
  .track { position: relative; width: 38px; height: 22px; border-radius: 11px; background: var(--groove-bg); box-shadow: var(--groove); transition: background 0.14s ease, box-shadow 0.14s ease; flex-shrink: 0; }
  .knob { position: absolute; top: 2px; left: 2px; width: 18px; height: 18px; border-radius: 50%; background: linear-gradient(180deg, #ffffff, #cdd2da); border: 1px solid rgba(0, 0, 0, 0.4); box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.9), 0 2px 0 rgba(0, 0, 0, 0.4); transition: transform 0.14s cubic-bezier(0.3, 1.4, 0.5, 1); }
  .toggle:hover:not(.disabled) .knob { box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.9), 0 2px 0 rgba(0, 0, 0, 0.4), 0 0 0 4px color-mix(in srgb, var(--accent) 22%, transparent); }
  input:checked + .track { background: linear-gradient(180deg, var(--accent), color-mix(in srgb, var(--accent) 78%, black)); box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.35), 0 0 12px -2px color-mix(in srgb, var(--accent) 55%, transparent); }
  input:checked + .track .knob { transform: translateX(16px); }
  input:focus-visible + .track { outline: 2px solid var(--accent); outline-offset: 2px; }
</style>
