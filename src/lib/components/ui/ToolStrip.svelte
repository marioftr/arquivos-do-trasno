<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Icon from "../Icon.svelte";

  /** Barra de ferramentas do panel: unha icona (e o seu nome) por tarefa; a activa abre os seus axustes ao lado. */
  export let tabs: { id: string; label: string }[];
  export let value: string;
  const dispatch = createEventDispatcher<{ again: void }>();

  const ICON: Record<string, string> = {
    convert: "convert",
    compress: "compress",
    edit: "sliders",
    annotate: "pencil",
    info: "info",
    extract: "download",
    mkv: "tracks",
    merge: "merge",
    pages: "pages",
    shape: "crop",
    marks: "type",
    optimize: "wrench",
    security: "lock",
  };

  let bar: HTMLDivElement;
  function key(e: KeyboardEvent) {
    const i = tabs.findIndex((x) => x.id === value);
    const fwd = e.key === "ArrowDown" || e.key === "ArrowRight";
    const back = e.key === "ArrowUp" || e.key === "ArrowLeft";
    const to = fwd ? i + 1 : back ? i - 1 : e.key === "Home" ? 0 : e.key === "End" ? tabs.length - 1 : -1;
    if (to < 0) return;
    e.preventDefault();
    const j = (to + tabs.length) % tabs.length;
    value = tabs[j].id;
    requestAnimationFrame(() => bar?.querySelectorAll<HTMLElement>('[role="tab"]')[j]?.focus());
  }
</script>

<!-- Frechas para moverse entre ferramentas; só a activa entra co Tab (como nos controis nativos). -->
<!-- svelte-ignore a11y_interactive_supports_focus -->
<div class="strip scrollbar-thin" role="tablist" aria-orientation="vertical" on:keydown={key} bind:this={bar}>
  {#each tabs as tab}
    <button role="tab" title={tab.label} tabindex={value === tab.id ? 0 : -1} aria-selected={value === tab.id} class:on={value === tab.id} on:click={() => (value === tab.id ? dispatch("again") : (value = tab.id))}>
      <Icon name={ICON[tab.id] ?? "sliders"} size={20} />
      <span class="sr">{tab.label}</span>
    </button>
  {/each}
</div>

<style>
  .strip { flex: 0 0 3.4rem; width: 3.4rem; display: flex; flex-direction: column; padding: 0.3rem 0; overflow-y: auto; overflow-x: hidden; background: var(--bg-rail); border-right: 1px solid var(--border); }
  button { position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 3.4rem; padding: 0; background: none; border: none; border-left: 2px solid transparent; border-radius: 0; color: var(--text-muted); }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  button:hover { color: var(--text); background: var(--bg-hover); }
  button.on { color: var(--text); background: color-mix(in srgb, var(--accent) 10%, var(--bg-rail)); border-left-color: var(--accent-conversor); }
  /* Con poucos píxeles de alto/ancho a barra pasa a ser unha fila. */
  @media (max-width: 1050px) {
    .strip { flex: 0 0 auto; width: auto; flex-direction: row; overflow-x: auto; overflow-y: hidden; padding: 0 0.3rem; border-right: none; border-bottom: 1px solid var(--border); }
    button { flex: 0 0 auto; min-width: 2.8rem; min-height: 2.6rem; border-left: none; border-bottom: 2px solid transparent; }
    button.on { border-left-color: transparent; border-bottom-color: var(--accent-conversor); }
  }
</style>
