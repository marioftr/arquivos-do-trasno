<script lang="ts">
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import type { Job } from "../../job";

  export let job: Job;
  $: busy = job.busy;
  $: message = job.message;

  // Mentres traballa, o proceso vive na barra de abaixo: aquí só se ve como «entra» nela.
  let anchor: HTMLSpanElement;
  let was = false;
  $: if ($busy !== was) {
    was = $busy;
    if ($busy) fly();
  }
  function fly() {
    if (typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const to = document.querySelector<HTMLElement>(".status .trasno");
    if (!anchor || !to) return;
    const a = anchor.getBoundingClientRect();
    const b = to.getBoundingClientRect();
    const el = document.createElement("img");
    el.src = "/trasno/base.png";
    el.alt = "";
    Object.assign(el.style, { position: "fixed", left: `${a.left}px`, top: `${a.top - 16}px`, width: "32px", height: "32px", imageRendering: "pixelated", zIndex: "200", pointerEvents: "none" });
    document.body.appendChild(el);
    const dx = b.left + b.width / 2 - 16 - a.left;
    const dy = b.top + b.height / 2 - 16 - (a.top - 16);
    el
      .animate(
        [
          { transform: "translate(0,0) scale(1)", opacity: 1 },
          { transform: `translate(${dx * 0.45}px, ${Math.min(0, dy * 0.45) - 36}px) scale(1.5)`, opacity: 1, offset: 0.4 },
          { transform: `translate(${dx}px, ${dy}px) scale(1.9)`, opacity: 0.95 },
        ],
        { duration: 650, easing: "cubic-bezier(.45,.05,.3,1)" },
      )
      .finished.then(() => {
        el.remove();
        to.animate([{ transform: "scale(1)" }, { transform: "scale(1.18)" }, { transform: "scale(1)" }], { duration: 260 });
      })
      .catch(() => el.remove());
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

<span class="anchor" bind:this={anchor}></span>
{#if $message}
  <div class="msg-wrap" class:ok={$message.ok && !$message.neutral} class:err={!$message.ok} class:neutral={$message.neutral}>
    <p class="msg selectable">{$message.text}</p>
    {#if !$message.ok}
      <button class="btn ghost" on:click={copy}><Icon name="copy" size={13} /> {copied ? $t("Copiado") : $t("Copiar")}</button>
    {/if}
  </div>
{/if}

<style>
  .anchor { display: block; height: 0; }
  .msg-wrap { display: flex; flex-direction: column; align-items: flex-start; gap: 0.3rem; border-left: 2px solid var(--border-strong); padding: 0.1rem 0 0.1rem 0.75rem; }
  .msg-wrap.ok { border-left-color: var(--accent-imaxe); }
  .msg-wrap.err { border-left-color: var(--accent-video); }
  .msg { margin: 0; font-size: 0.86rem; white-space: pre-wrap; word-break: break-word; user-select: text; }
  .msg-wrap.neutral { border-left-color: var(--accent-pdf); }
  .neutral .msg { color: var(--accent-pdf); }
  .ok .msg { color: var(--accent-imaxe); }
  .err .msg { color: var(--accent-video); }
</style>
