<script lang="ts">
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import { trasnoAway } from "../../agent";
  import type { Job } from "../../job";

  export let job: Job;
  $: busy = job.busy;
  $: message = job.message;

  // Mentres traballa, o proceso vive na barra de abaixo. O trasno da barra baixa e desaparece, aparece
  // por riba xusto onde se premeu, para un instante e leva o traballo dun golpe á barra; despois volve.
  let anchor: HTMLSpanElement;
  let was = false;
  $: if ($busy !== was) {
    was = $busy;
    if ($busy) fly();
  }
  const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
  async function fly() {
    const to = document.querySelector<HTMLElement>(".status .trasno");
    const reduce = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduce || !to) return;
    const ae = document.activeElement as HTMLElement | null;
    const src = ae && ae.closest(".right") && ae.tagName === "BUTTON" ? ae : anchor;
    const a = src.getBoundingClientRect();
    const b = to.getBoundingClientRect();
    const x = a.left + a.width / 2 - 32;
    const top = a.top - 64 - 6;
    const el = document.createElement("img");
    el.src = "/trasno/base.png";
    el.alt = "";
    Object.assign(el.style, { position: "fixed", left: `${x}px`, top: "0px", width: "64px", height: "64px", imageRendering: "pixelated", zIndex: "300", pointerEvents: "none", transform: "translateY(-80px)", opacity: "0" });
    trasnoAway.set(true);
    await wait(210);
    document.body.appendChild(el);
    try {
      await el.animate([{ transform: "translateY(-80px)", opacity: 0 }, { transform: `translateY(${Math.max(0, top)}px)`, opacity: 1 }], { duration: 300, easing: "cubic-bezier(.2,.8,.3,1.1)", fill: "forwards" }).finished;
      await wait(170);
      const dy = b.top + b.height - 64 - Math.max(0, top);
      await el.animate([{ transform: `translateY(${Math.max(0, top)}px)` }, { transform: `translate(${b.left + b.width / 2 - 32 - x}px, ${Math.max(0, top) + dy}px)` }], { duration: 230, easing: "cubic-bezier(.6,0,.9,.6)", fill: "forwards" }).finished;
    } catch {
      // sen animación
    }
    el.remove();
    trasnoAway.set(false);
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
