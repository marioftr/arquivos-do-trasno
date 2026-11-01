<script lang="ts">
  import { onMount, tick } from "svelte";
  import { t } from "../i18n";
  import { get } from "svelte/store";
  import { dialog, openSettings, toolActions, jump, theme, resolvedTheme } from "../settings";
  import { files } from "../files";
  import Icon from "./Icon.svelte";

  /** Buscador de accións (Ctrl+K): ferramentas do ficheiro actual e accións da aplicación. */
  export let onAdd: () => void;

  interface Cmd {
    id: string;
    label: string;
    hint?: string;
    icon: string;
    run: () => void;
  }

  let query = "";
  let sel = 0;
  let input: HTMLInputElement;
  let list: HTMLUListElement;
  onMount(() => input.focus());

  const close = () => dialog.set(null);
  $: cmds = [
    ...$toolActions.map((a): Cmd => ({ id: a.id, label: a.label, hint: $t("Ferramenta"), icon: "convert", run: () => (close(), jump.set({ tab: a.tab, mode: a.mode })) })),
    { id: "add", label: $t("Engadir ficheiros"), icon: "plus", run: () => (close(), onAdd()) },
    { id: "procs", label: $t("Procesos"), icon: "layers", run: () => dialog.set("procs") },
    { id: "settings", label: $t("Axustes"), icon: "gear", run: () => openSettings("general") },
    { id: "shortcuts", label: $t("Atallos de teclado"), icon: "keyboard", run: () => openSettings("shortcuts") },
    { id: "theme", label: $t("Cambiar o tema (claro/escuro)"), icon: "gear", run: () => (close(), theme.set(get(resolvedTheme) === "dark" ? "light" : "dark")) },
    ...($files.length ? [{ id: "clear", label: $t("Baleirar a lista de ficheiros"), icon: "close", run: () => (close(), files.set([])) } as Cmd] : []),
  ] as Cmd[];

  const norm = (s: string) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  $: shown = query.trim() ? cmds.filter((c) => norm(c.label).includes(norm(query.trim()))) : cmds;
  $: if (sel >= shown.length) sel = Math.max(0, shown.length - 1);

  async function key(e: KeyboardEvent) {
    if (e.key === "ArrowDown") sel = Math.min(shown.length - 1, sel + 1);
    else if (e.key === "ArrowUp") sel = Math.max(0, sel - 1);
    else if (e.key === "Enter") shown[sel]?.run();
    else if (e.key === "Escape") close();
    else return;
    e.preventDefault();
    await tick();
    list?.querySelector(".on")?.scrollIntoView({ block: "nearest" });
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="veil" on:click={close}>
  <div class="box" role="dialog" tabindex="-1" aria-modal="true" aria-label={$t("Buscar unha acción")} on:click|stopPropagation>
    <div class="q">
      <Icon name="search" size={16} />
      <input bind:this={input} bind:value={query} on:keydown={key} placeholder={$t("Que queres que faga o trasno?")} aria-label={$t("Buscar unha acción")} role="combobox" aria-expanded="true" aria-controls="cmdlist" />
    </div>
    <ul id="cmdlist" role="listbox" bind:this={list} class="scrollbar-thin">
      {#each shown as c, i (c.id)}
        <li role="option" aria-selected={i === sel} class:on={i === sel}>
          <button tabindex="-1" on:click={c.run} on:mousemove={() => (sel = i)}>
            <Icon name={c.icon} size={15} />
            <span class="l">{c.label}</span>
            {#if c.hint}<span class="muted h">{c.hint}</span>{/if}
          </button>
        </li>
      {/each}
      {#if !shown.length}<li class="muted none">{$t("O trasno non atopa nada así.")}</li>{/if}
    </ul>
    <p class="muted foot">↑↓ · Enter · Esc</p>
  </div>
</div>

<style>
  .veil { position: fixed; inset: 0; z-index: 55; background: rgba(0, 0, 0, 0.45); display: flex; justify-content: center; align-items: flex-start; padding-top: 14vh; }
  .box { width: min(520px, 92vw); background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); box-shadow: 0 18px 50px rgba(0, 0, 0, 0.5); overflow: hidden; }
  .q { display: flex; align-items: center; gap: 0.6rem; padding: 0.7rem 0.9rem; border-bottom: 1px solid var(--border); color: var(--text-muted); }
  input { flex: 1; background: none; border: none; outline: none; color: var(--text); font: inherit; font-size: 0.95rem; }
  ul { list-style: none; margin: 0; padding: 0.3rem; max-height: min(340px, 50vh); overflow-y: auto; }
  li button { width: 100%; display: flex; align-items: center; gap: 0.65rem; background: none; border: none; color: var(--text); text-align: left; padding: 0.5rem 0.65rem; border-radius: 6px; font-size: 0.88rem; }
  li.on button { background: color-mix(in srgb, var(--accent) 18%, transparent); }
  .l { flex: 1; }
  .h { font-size: 0.74rem; }
  .none { padding: 0.8rem; text-align: center; }
  .foot { margin: 0; padding: 0.4rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.72rem; }
</style>
