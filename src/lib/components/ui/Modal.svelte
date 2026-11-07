<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { t } from "../../i18n";
  import Icon from "../Icon.svelte";

  export let title: string;
  export let onClose: () => void;
  /** Altura fixa: o diálogo non cambia de tamaño ao cambiar de pestana. */
  export let tall = false;

  let box: HTMLDivElement;
  let before: HTMLElement | null = null;
  onMount(() => {
    before = document.activeElement as HTMLElement | null;
    box?.querySelector<HTMLElement>("button.btn, input, select, [tabindex='0']")?.focus() ?? box?.focus();
  });
  onDestroy(() => before?.focus?.());

  // O foco queda dentro do diálogo: Tab desde o último elemento volve ao primeiro, e ao revés.
  function key(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
    if (e.key !== "Tab" || !box) return;
    const items = [...box.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea, [tabindex="0"]')].filter((x) => x.offsetParent !== null);
    if (!items.length) return;
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && (document.activeElement === first || !box.contains(document.activeElement))) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && (document.activeElement === last || !box.contains(document.activeElement))) {
      e.preventDefault();
      first.focus();
    }
  }
</script>

<svelte:window on:keydown={key} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="veil" on:click={onClose}>
  <div class="box" class:tall role="dialog" tabindex="-1" aria-modal="true" aria-label={title} bind:this={box} on:click|stopPropagation>
    <header>
      <h2>{title}</h2>
      <button class="btn ghost icon" on:click={onClose} aria-label={$t("Pechar")} title={$t("Pechar")}><Icon name="close" size={15} /></button>
    </header>
    <div class="content selectable scrollbar-thin" class:tall><slot /></div>
  </div>
</div>

<style>
  .veil { position: fixed; inset: 0; z-index: 50; background: rgba(8, 9, 12, 0.6); backdrop-filter: blur(3px); display: grid; grid-template: minmax(0, 1fr) / minmax(0, 1fr); place-items: center; padding: clamp(0.5rem, 2.5vw, 1.5rem); }
  .box { width: min(860px, 100%); max-height: 100%; min-height: 0; display: flex; flex-direction: column; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); box-shadow: var(--shadow-pop); }
  header { display: flex; align-items: center; justify-content: space-between; padding: 0.8rem 1.1rem 0.65rem; border-bottom: 1px solid var(--border); }
  h2 { margin: 0; font-size: var(--fs-lg); font-weight: 600; }
  .content { padding: 1rem 1.1rem 1.2rem; overflow-y: auto; overflow-x: hidden; overscroll-behavior-x: none; min-height: 0; flex: 1; display: flex; flex-direction: column; gap: 1rem; }
  .box.tall { height: min(660px, 100%); }
  .content.tall { overflow: hidden; }
</style>
