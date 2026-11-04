<script lang="ts">
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import { trasnoAway } from "../../agent";
  import type { Job } from "../../job";

  export let job: Job;
  $: busy = job.busy;
  $: text = job.text;
  $: message = job.message;

  // Mentres traballa, o proceso vive na barra de abaixo. Ao empezar: o trasno da barra baixa, outro aparece por
  // riba xusto do botón premido cun billete do traballo, para un instante e baixan xuntos dun golpe á barra,
  // onde nace o recadro de progreso e o trasno volve ao seu sitio. Nunca hai dous á vez.
  let anchor: HTMLSpanElement;
  let was = false;
  $: if ($busy !== was) {
    was = $busy;
    if ($busy) fly();
  }
  const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
  async function fly() {
    const dock = document.querySelector<HTMLElement>(".status .trasno");
    const reduce = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduce || !dock || !anchor) return;
    const ae = document.activeElement as HTMLElement | null;
    const src = ae && ae.tagName === "BUTTON" && ae.closest(".right") ? ae : anchor;
    const a = src.getBoundingClientRect();
    const b = dock.getBoundingClientRect();
    const S = 72;
    const cx = a.left + a.width / 2;
    const standY = Math.max(4, a.top - S + 10); // de pé sobre o botón
    const fallTo = b.top;
    const layer = document.createElement("div");
    Object.assign(layer.style, { position: "fixed", inset: "0", zIndex: "300", pointerEvents: "none" });
    const sprite = document.createElement("img");
    sprite.src = "/trasno/base.png";
    sprite.alt = "";
    Object.assign(sprite.style, { position: "absolute", left: `${cx - S / 2}px`, top: "0", width: `${S}px`, height: `${S}px`, imageRendering: "pixelated", opacity: "0", filter: "drop-shadow(0 3px 0 rgba(0,0,0,.35))" });
    const label = ($text || "").replace(/…$/, "").slice(0, 30);
    const ticket = document.createElement("div");
    ticket.textContent = label;
    Object.assign(ticket.style, { position: "absolute", left: `${cx + 24}px`, top: `${standY + 22}px`, padding: "3px 9px", borderRadius: "6px", background: "var(--accent)", color: "var(--on-accent)", font: "600 11px/1.3 inherit", whiteSpace: "nowrap", opacity: "0", boxShadow: "0 4px 14px rgba(0,0,0,.35)" });
    const ring = document.createElement("div");
    Object.assign(ring.style, { position: "absolute", left: `${b.left + b.width / 2 - 28}px`, top: `${b.top + b.height - 40}px`, width: "56px", height: "56px", borderRadius: "50%", border: "2px solid var(--accent)", opacity: "0" });
    layer.append(ring, sprite, ticket);
    trasnoAway.set(true);
    try {
      await wait(210); // o trasno da barra acaba de baixar
      document.body.appendChild(layer);
      // 1) cae desde arriba ata ficar de pé sobre o botón; o billete sae do botón
      sprite.animate([{ transform: `translateY(${-S - 40}px) scale(1, 1.15)`, opacity: 0 }, { transform: `translateY(${standY + 6}px) scale(1.1, 0.88)`, opacity: 1, offset: 0.7 }, { transform: `translateY(${standY}px) scale(1)`, opacity: 1 }], { duration: 340, easing: "cubic-bezier(.3,.7,.4,1)", fill: "forwards" });
      ticket.animate([{ transform: "translateY(18px) scale(.4)", opacity: 0 }, { transform: "translateY(0) scale(1)", opacity: 1 }], { duration: 220, delay: 240, easing: "cubic-bezier(.2,1.4,.4,1)", fill: "forwards" });
      await wait(340 + 240);
      // 2) para un instante (un saltiño de alegría)
      await sprite.animate([{ transform: `translateY(${standY}px)` }, { transform: `translateY(${standY - 10}px)` }, { transform: `translateY(${standY}px)` }], { duration: 200, easing: "ease-out", fill: "forwards" }).finished;
      // 3) agáchase un pouco e baixa dun golpe á barra co billete
      const ticketX = cx + 24 - (b.left + b.width / 2);
      sprite.animate([{ transform: `translate(0, ${standY}px)` }, { transform: `translate(0, ${standY - 12}px)`, offset: 0.2 }, { transform: `translate(${b.left + b.width / 2 - cx}px, ${fallTo}px)` }], { duration: 300, easing: "cubic-bezier(.6,0,.9,.55)", fill: "forwards" });
      ticket.animate([{ transform: `translate(0, 0) scale(1)`, opacity: 1 }, { transform: `translate(${-ticketX - 0}px, ${fallTo - standY + 6}px) scale(.35)`, opacity: 0.2 }], { duration: 300, easing: "cubic-bezier(.6,0,.9,.55)", fill: "forwards" });
      await wait(300);
      // 4) impacto na barra: onda e volta o trasno ao seu sitio
      ring.animate([{ transform: "scale(.4)", opacity: 0.9 }, { transform: "scale(1.9)", opacity: 0 }], { duration: 420, easing: "ease-out" });
      sprite.style.opacity = "0";
    } catch {
      // sen animación
    }
    trasnoAway.set(false);
    setTimeout(() => layer.remove(), 450);
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
