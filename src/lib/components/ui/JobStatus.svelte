<script lang="ts">
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import type { Job } from "../../job";
  import { cancelJob } from "../../engine";

  export let job: Job;
  $: busy = job.busy;
  $: text = job.text;
  $: progress = job.progress;
  $: message = job.message;

  // Cancelar pide confirmación; o traballo en marcha para e non queda ningún ficheiro a medias.
  let asking = false;
  $: if (!$busy) asking = false;
  async function doCancel() {
    asking = false;
    await cancelJob().catch(() => false);
  }

  let copied = false;
  async function copy() {
    try {
      await navigator.clipboard.writeText($message?.text ?? "");
      copied = true;
      setTimeout(() => (copied = false), 1400);
    } catch {
      // o texto tamén se pode seleccionar a man
    }
  }
</script>

{#if $busy}
  <div class="busyrow">
    <p class="busy"><span class="spin"></span>{$text}{#if $progress !== null}&nbsp;·&nbsp;{Math.round($progress * 100)} %{/if}</p>
    {#if asking}
      <span class="ask">{$t("Cancelar e non gardar nada?")}</span>
      <button class="btn danger" on:click={doCancel}>{$t("Si, cancelar")}</button>
      <button class="btn" on:click={() => (asking = false)}>{$t("Seguir")}</button>
    {:else}
      <button class="btn ghost" on:click={() => (asking = true)}>{$t("Cancelar")}</button>
    {/if}
  </div>
  {#if $progress !== null}
    <div class="pbar"><div class="pfill" style={`width:${Math.round($progress * 100)}%`}></div></div>
  {/if}
{/if}
{#if $message}
  <div class="msg-wrap" class:ok={$message.ok && !$message.neutral} class:err={!$message.ok} class:neutral={$message.neutral}>
    <p class="msg selectable">{$message.text}</p>
    {#if !$message.ok}
      <button class="btn ghost" on:click={copy}><Icon name="copy" size={13} /> {copied ? $t("Copiado") : $t("Copiar")}</button>
    {/if}
  </div>
{/if}

<style>
  .busyrow :global(.btn) { height: var(--control-h); min-height: var(--control-h); padding-block: 0; box-sizing: border-box; }
  .pbar { height: 6px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .pfill { height: 100%; background: var(--accent); border-radius: 99px; transition: width 0.3s ease; }
  .busyrow { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; }
  .ask { font-size: 0.82rem; }
  .busy { margin: 0; display: flex; align-items: center; gap: 0.6rem; font-size: 0.86rem; color: var(--text-muted); }
  .spin { width: 12px; height: 12px; border-radius: 50%; border: 2px solid var(--border-strong); border-top-color: var(--accent); animation: spin 0.8s linear infinite; flex-shrink: 0; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .msg-wrap { display: flex; flex-direction: column; align-items: flex-start; gap: 0.3rem; border-left: 2px solid var(--border-strong); padding: 0.1rem 0 0.1rem 0.75rem; }
  .msg-wrap.ok { border-left-color: var(--accent-imaxe); }
  .msg-wrap.err { border-left-color: var(--accent-video); }
  .msg { margin: 0; font-size: 0.86rem; white-space: pre-wrap; word-break: break-word; user-select: text; }
  .msg-wrap.neutral { border-left-color: var(--accent-pdf); }
  .neutral .msg { color: var(--accent-pdf); }
  .ok .msg { color: var(--accent-imaxe); }
  .err .msg { color: var(--accent-video); }
</style>
