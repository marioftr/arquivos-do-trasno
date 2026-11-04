<script lang="ts">
  import Icon from "../Icon.svelte";

  /** Desplegable con icona en cada opción. Teclado: frechas, Intro/Espazo e Esc, como un listbox nativo. */
  export let options: { value: string; label: string; icon: string }[];
  export let value: string;
  export let label = "";

  let btn: HTMLButtonElement;
  let open = false;
  let active = 0;
  let pos = { left: 0, top: 0, width: 0, up: false };
  $: current = options.find((o) => o.value === value) ?? options[0];

  function show() {
    const r = btn.getBoundingClientRect();
    const h = options.length * 34 + 10;
    const up = r.bottom + h > window.innerHeight - 8 && r.top > h;
    pos = { left: r.right, top: up ? r.top - 4 : r.bottom + 4, width: Math.max(r.width, 210), up };
    active = Math.max(0, options.findIndex((o) => o.value === value));
    open = true;
  }
  function choose(v: string) {
    value = v;
    open = false;
    btn.focus();
  }
  function key(e: KeyboardEvent) {
    if (!open) {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        show();
      }
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      open = false;
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      active = (active + 1) % options.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      active = (active - 1 + options.length) % options.length;
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      choose(options[active].value);
    } else if (e.key === "Tab") open = false;
  }
</script>

<svelte:window on:pointerdown={(e) => { if (open && !btn?.contains(e.target as Node) && !(e.target as HTMLElement | null)?.closest?.(".isl")) open = false; }} on:resize={() => (open = false)} />

<button bind:this={btn} class="trig" type="button" aria-haspopup="listbox" aria-expanded={open} aria-label={label} on:click={() => (open ? (open = false) : show())} on:keydown={key}>
  <Icon name={current.icon} size={15} />
  <span class="cur">{current.label}</span>
  <Icon name="chevron-down" size={14} />
</button>

{#if open}
  <ul class="isl" role="listbox" aria-label={label} style={`left:${pos.left}px;top:${pos.top}px;min-width:${pos.width}px;transform:translate(-100%, ${pos.up ? "-100%" : "0"})`}>
    {#each options as o, i}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <li role="option" aria-selected={o.value === value} class:act={i === active} class:sel={o.value === value} on:pointerenter={() => (active = i)} on:click={() => choose(o.value)}>
        <Icon name={o.icon} size={15} />
        <span>{o.label}</span>
        {#if o.value === value}<span class="ck"><Icon name="check" size={14} /></span>{/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
  .trig { display: inline-flex; align-items: center; gap: 0.5rem; height: var(--control-h); padding: 0 0.6rem; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); font-size: 0.86rem; }
  .trig:hover { border-color: var(--accent); }
  .cur { white-space: nowrap; }
  .isl { position: fixed; z-index: 120; margin: 0; padding: 0.25rem; list-style: none; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); box-shadow: var(--shadow-pop); }
  li { display: flex; align-items: center; gap: 0.55rem; height: 32px; padding: 0 0.6rem; border-radius: 3px; color: var(--text-muted); font-size: 0.86rem; cursor: pointer; }
  li.act { background: var(--bg-hover); color: var(--text); }
  li.sel { color: var(--text); }
  .ck { margin-left: auto; color: var(--accent); display: inline-flex; }
</style>
