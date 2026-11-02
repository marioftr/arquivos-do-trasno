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
      <span>{tab.label}</span>
    </button>
  {/each}
</div>

<style>
  .strip { flex: 0 0 4.7rem; width: 4.7rem; display: flex; flex-direction: column; gap: 0.2rem; padding: 0.5rem 0.35rem; overflow-y: auto; overflow-x: hidden; background: var(--bg); border-right: 1px solid var(--border); }
  button { position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.25rem; min-height: 3.5rem; padding: 0.4rem 0.2rem; background: none; border: none; border-radius: var(--radius-md); color: var(--text-muted); }
  button span { font-size: 0.66rem; line-height: 1.15; text-align: center; font-weight: 550; overflow-wrap: anywhere; hyphens: auto; }
  button:hover { color: var(--text); background: var(--bg-hover); }
  button.on { color: var(--accent); background: color-mix(in srgb, var(--accent) 13%, transparent); }
  button.on::before { content: ""; position: absolute; left: -0.35rem; top: 0.7rem; bottom: 0.7rem; width: 3px; border-radius: 0 3px 3px 0; background: var(--accent); }
  /* Con poucos píxeles de alto/ancho a barra pasa a ser unha fila. */
  @media (max-width: 1050px) {
    .strip { flex: 0 0 auto; width: auto; flex-direction: row; overflow-x: auto; overflow-y: hidden; padding: 0.35rem 0.5rem; border-right: none; border-bottom: 1px solid var(--border); }
    button { flex: 0 0 auto; min-width: 4.4rem; min-height: 3rem; }
    button.on::before { left: 0.7rem; right: 0.7rem; top: auto; bottom: -0.35rem; width: auto; height: 3px; border-radius: 3px 3px 0 0; }
  }
</style>
