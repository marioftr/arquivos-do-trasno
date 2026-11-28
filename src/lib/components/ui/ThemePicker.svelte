<script lang="ts">
  import { t } from "../../i18n";

  /** Elixir o tema (Sistema, Claro ou Escuro) con miniaturas. Valor: «system», «light» ou «dark». */
  export let value: string;
  $: options = [{ id: "system", label: $t("Sistema") }, { id: "light", label: $t("Claro") }, { id: "dark", label: $t("Escuro") }];
</script>

<div class="themes" role="radiogroup" aria-label={$t("Tema")}>
  {#each options as th}
    <button type="button" class="theme" class:on={value === th.id} role="radio" aria-checked={value === th.id} on:click={() => (value = th.id)}>
      <span class={`mini ${th.id}`} aria-hidden="true">
        {#if th.id === "system"}
          <span class="half l"><i class="sb"></i><i class="hd"></i><i class="bd"></i><i class="ac"></i></span>
          <span class="half d"><i class="sb"></i><i class="hd"></i><i class="bd"></i><i class="ac"></i></span>
        {:else}
          <i class="sb"></i><i class="hd"></i><i class="bd"></i><i class="ac"></i>
        {/if}
      </span>
      <span class="tl">{th.label}</span>
    </button>
  {/each}
</div>

<style>
  .themes { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.6rem; }
  .theme { display: flex; flex-direction: column; gap: 0.45rem; padding: 0.4rem 0.4rem 0.5rem; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-md); color: var(--text-muted); font-size: var(--fs-sm); }
  .theme:hover { border-color: var(--text-faint); color: var(--text); }
  .theme.on { border-color: var(--accent); color: var(--text); box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 25%, transparent); }
  .mini { position: relative; display: block; height: 58px; border-radius: 6px; overflow: hidden; border: 1px solid rgba(128, 128, 128, 0.35); }
  .mini.light, .half.l { --m-bg: #f4f5f7; --m-side: #ffffff; --m-head: #e4e6ea; --m-body: #dcdfe4; }
  .mini.dark, .half.d { --m-bg: #14161a; --m-side: #1f2126; --m-head: #272a30; --m-body: #2f333a; }
  .mini.light, .mini.dark { background: var(--m-bg); }
  .mini.system { display: flex; }
  .half { position: relative; display: block; flex: 1; background: var(--m-bg); }
  .mini i { position: absolute; display: block; border-radius: 2px; }
  .mini .sb { left: 0; top: 0; bottom: 0; width: 22%; background: var(--m-side); border-radius: 0; }
  .mini .hd { left: 28%; right: 6%; top: 10%; height: 14%; background: var(--m-head); }
  .mini .bd { left: 28%; right: 6%; top: 32%; height: 34%; background: var(--m-body); }
  .mini .ac { left: 28%; width: 30%; bottom: 10%; height: 14%; background: #5ecf6f; }
  .half .sb { width: 28%; }
  .half .hd, .half .bd { left: 36%; right: 8%; }
  .half .ac { left: 36%; width: 40%; }
  .tl { text-align: center; }
</style>
