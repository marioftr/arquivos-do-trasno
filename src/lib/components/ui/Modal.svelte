<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { t } from "../../i18n";
  import { bubble, trasnoCovered } from "../../agent";
  import { flyIn, flyOut, trasnoFlying, trasnoLeave } from "../../flight";
  import Icon from "../Icon.svelte";
  import TrasnoSprite from "./TrasnoSprite.svelte";
  import Bubble from "./Bubble.svelte";

  export let title: string;
  export let onClose: () => void;
  /** Altura fixa: o diálogo non cambia de tamaño ao cambiar de pestana. */
  export let tall = false;
  /** Ancho máximo do diálogo. */
  export let width = "860px";
  /** Altura máxima do diálogo (o que sobra desprázase dentro). */
  export let maxHeight = "100%";
  /** O trasno acompaña no cabezallo (o do recanto da aplicación queda agochado mentres o diálogo está aberto). */
  export let mascot = false;

  let box: HTMLDivElement;
  let mas: HTMLSpanElement | undefined;
  let before: HTMLElement | null = null;
  onMount(() => {
    before = document.activeElement as HTMLElement | null;
    box?.querySelector<HTMLElement>("button.btn, input, select, [tabindex='0']")?.focus() ?? box?.focus();
    if (mascot) {
      trasnoCovered.set(true);
      void flyIn(mas);
    }
  });
  // Ao pecharse, o trasno da cabeceira volve á barra (ou vai co billete dun traballo que empeza). Mídese antes de que a xanela desapareza.
  function leave(_node: Element) {
    if (mascot) {
      const leaving = get(trasnoLeave);
      trasnoLeave.set(null);
      void flyOut(mas?.getBoundingClientRect(), leaving ? leaving.ticket : null);
    }
    return { duration: 160, css: (t: number) => `opacity:${t}` };
  }
  onDestroy(() => {
    before?.focus?.();
    if (mascot) trasnoCovered.set(false);
  });

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
<div class="veil" on:click={onClose} out:leave|global>
  <div class="box" class:tall style={`--w:${width};--mh:${maxHeight}`} role="dialog" tabindex="-1" aria-modal="true" aria-label={title} bind:this={box} on:click|stopPropagation>
    <header class:mascot>
      {#if mascot}<span class="mas" class:flying={$trasnoFlying} bind:this={mas} aria-hidden="true"><TrasnoSprite state="idle" size={64} /></span>{/if}
      <h2>{title}</h2>
      {#if mascot && $bubble && $trasnoCovered}<Bubble b={$bubble} placement="head" />{/if}
      <button class="btn ghost icon" on:click={onClose} aria-label={$t("Pechar")} title={$t("Pechar")}><Icon name="close" size={15} /></button>
    </header>
    <div class="content selectable scrollbar-thin" class:tall><slot /></div>
  </div>
</div>

<style>
  .veil { position: fixed; inset: 0; z-index: 50; background: rgba(8, 9, 12, 0.6); backdrop-filter: blur(3px); display: grid; grid-template: minmax(0, 1fr) / minmax(0, 1fr); place-items: center; padding: clamp(0.5rem, 2.5vw, 1.5rem); }
  .box { transition: width 0.22s ease; width: min(var(--w, 860px), 100%); max-height: min(var(--mh, 100%), 100%); min-height: 0; display: flex; flex-direction: column; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); box-shadow: var(--shadow-pop); }
  header { position: relative; flex-wrap: wrap; display: flex; align-items: center; justify-content: space-between; padding: 0.8rem 1.1rem 0.65rem; border-bottom: 1px solid var(--border); }
  header.mascot { padding: 0.35rem 1.1rem 0.35rem 0.6rem; }
  .mas { display: block; line-height: 0; margin-right: 0.5rem; }
  .mas.flying { visibility: hidden; }
  header.mascot h2 { margin-right: auto; }
  h2 { margin: 0; font-size: var(--fs-lg); font-weight: 600; }
  .content { padding: 1rem 1.1rem 1.2rem; overflow-y: auto; overflow-x: hidden; overscroll-behavior-x: none; min-height: 0; flex: 1; display: flex; flex-direction: column; gap: 1rem; }
  .box.tall { height: min(660px, 100%); }
  .content.tall { overflow: hidden; }
</style>
