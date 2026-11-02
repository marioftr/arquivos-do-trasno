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
  <input type="checkbox" role="switch" bind:checked {disabled} />
  <span class="track"><span class="knob"></span></span>
</label>

<style>
  .toggle { position: relative; display: flex; align-items: center; justify-content: space-between; gap: 0.8rem; min-height: var(--control-h); cursor: pointer; }
  .toggle.disabled { opacity: 0.4; cursor: not-allowed; }
  .lab { display: inline-flex; align-items: center; font-size: 0.88rem; }
  input { position: absolute; inset: 0; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: pointer; z-index: 1; }
  .toggle.disabled input { cursor: not-allowed; }
  .track { position: relative; width: 34px; height: 20px; border-radius: 10px; background: var(--border-strong); transition: background 0.12s ease; flex-shrink: 0; }
  .knob { position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: #f2f4f8; transition: transform 0.12s ease; box-shadow: 0 1px 2px rgba(0, 0, 0, 0.5); }
  input:checked + .track { background: var(--accent); }
  input:checked + .track .knob { transform: translateX(14px); }
  input:focus-visible + .track { outline: 2px solid var(--accent); outline-offset: 2px; }
</style>
