<script lang="ts">
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import type { Job } from "../../job";

  export let job: Job;
  $: busy = job.busy;
  $: text = job.text;
  $: message = job.message;

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
  <p class="busy"><span class="spin"></span>{$text}</p>
{/if}
{#if $message}
  <div class="msg-wrap" class:ok={$message.ok} class:err={!$message.ok}>
    <p class="msg selectable">{$message.text}</p>
    {#if !$message.ok}
      <button class="btn ghost" on:click={copy}><Icon name="copy" size={13} /> {copied ? $t("Copiado") : $t("Copiar")}</button>
    {/if}
  </div>
{/if}

<style>
  .busy { margin: 0; display: flex; align-items: center; gap: 0.6rem; font-size: 0.86rem; color: var(--text-muted); }
  .spin { width: 12px; height: 12px; border-radius: 50%; border: 2px solid var(--border-strong); border-top-color: var(--accent); animation: spin 0.8s linear infinite; flex-shrink: 0; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .msg-wrap { display: flex; flex-direction: column; align-items: flex-start; gap: 0.3rem; border-left: 2px solid var(--border-strong); padding: 0.1rem 0 0.1rem 0.75rem; }
  .msg-wrap.ok { border-left-color: var(--accent-imaxe); }
  .msg-wrap.err { border-left-color: var(--accent-video); }
  .msg { margin: 0; font-size: 0.86rem; white-space: pre-wrap; word-break: break-word; user-select: text; }
  .ok .msg { color: var(--accent-imaxe); }
  .err .msg { color: var(--accent-video); }
</style>
