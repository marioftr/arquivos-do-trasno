<script lang="ts">
  import { t } from "../i18n";
  import { dialog, hubPage } from "../settings";
  import { procs } from "../procs";
  import { updateAvailable } from "../updates";
  import Modal from "./ui/Modal.svelte";
  import Icon from "./Icon.svelte";
  import ProcessesPanel from "./ProcessesPanel.svelte";
  import GeneralPanel from "./GeneralPanel.svelte";
  import ShortcutsPanel from "./ShortcutsPanel.svelte";
  import ProfilesPanel from "./ProfilesPanel.svelte";
  import ToolsPanel from "./ToolsPanel.svelte";
  import NewsPanel from "./NewsPanel.svelte";
  import CreditsPanel from "./CreditsPanel.svelte";

  /** Todo o que non son ferramentas, nunha soa xanela: a árbore da esquerda leva a calquera páxina. */
  $: running = $procs.filter((p) => p.state === "running").length;
  interface Item {
    id: string;
    label: string;
    badge?: number;
    dot?: boolean;
  }
  $: groups = ([
    { id: "procs", label: $t("Procesos"), icon: "layers", items: [{ id: "running", label: $t("En marcha"), badge: running || 0 }, { id: "history", label: $t("Historial") }] },
    {
      id: "settings",
      label: $t("Axustes"),
      icon: "gear",
      items: [
        { id: "general", label: $t("Xeral") },
        { id: "shortcuts", label: $t("Atallos de teclado") },
        { id: "profiles", label: $t("Perfís de saída") },
        { id: "tools", label: $t("Extensións") },
      ],
    },
    { id: "news", label: $t("Novidades"), icon: "info", items: [{ id: "changelog", label: $t("Rexistro de cambios") }, { id: "updates", label: $t("Actualizacións"), dot: $updateAvailable }] },
    { id: "about", label: $t("Sobre o trasno"), icon: "puzzle", items: [{ id: "credits", label: $t("Créditos e licenzas") }] },
  ] as { id: string; label: string; icon: string; items: Item[] }[]);
  $: [grp, sub] = $hubPage.split(".");
  $: title = groups.find((g) => g.id === grp)?.label ?? "";
</script>

<Modal title="Arquivos do Trasno" tall onClose={() => dialog.set(null)}>
  <div class="split">
    <nav class="side" aria-label={$t("Seccións")}>
      {#each groups as g}
        <div class="grp">
          <span class="gh" class:on={grp === g.id}><Icon name={g.icon} size={15} /> {g.label}</span>
          {#each g.items as it}
            <button class:on={grp === g.id && sub === it.id} aria-current={grp === g.id && sub === it.id ? "page" : undefined} on:click={() => hubPage.set(`${g.id}.${it.id}`)}>
              <span>{it.label}</span>
              {#if it.badge}<span class="badge">{it.badge}</span>{/if}
              {#if it.dot}<span class="dot" aria-label={$t("Hai unha versión nova")}></span>{/if}
            </button>
          {/each}
        </div>
      {/each}
    </nav>
    <div class="pane scrollbar-thin">
      <h2>{title}<span class="muted"> · {groups.find((g) => g.id === grp)?.items.find((i) => i.id === sub)?.label ?? ""}</span></h2>
      {#if $hubPage === "procs.running"}<ProcessesPanel part="running" />
      {:else if $hubPage === "procs.history"}<ProcessesPanel part="history" />
      {:else if $hubPage === "settings.general"}<GeneralPanel />
      {:else if $hubPage === "settings.shortcuts"}<ShortcutsPanel />
      {:else if $hubPage === "settings.profiles"}<ProfilesPanel />
      {:else if $hubPage === "settings.tools"}<ToolsPanel />
      {:else if $hubPage === "news.changelog"}<NewsPanel part="changelog" />
      {:else if $hubPage === "news.updates"}<NewsPanel part="updates" />
      {:else if $hubPage === "about.credits"}<CreditsPanel />
      {/if}
    </div>
  </div>
</Modal>

<style>
  .split { display: flex; gap: 1rem; min-height: 0; flex: 1; }
  .side { flex: 0 0 12.5rem; display: flex; flex-direction: column; gap: 0.7rem; overflow-y: auto; padding-right: 0.2rem; }
  .grp { display: flex; flex-direction: column; gap: 0.1rem; }
  .gh { display: flex; align-items: center; gap: 0.5rem; padding: 0.25rem 0.6rem; font-size: 0.74rem; font-weight: 700; letter-spacing: 0.05em; text-transform: uppercase; color: var(--text-faint); }
  .gh.on { color: var(--accent); }
  .side button { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; text-align: left; background: none; border: none; color: var(--text-muted); padding: 0.45rem 0.7rem 0.45rem 1.9rem; border-radius: var(--radius-sm); font-size: 0.87rem; font-weight: 500; }
  .side button:hover { background: var(--bg-hover); color: var(--text); }
  .side button.on { background: color-mix(in srgb, var(--accent) 18%, transparent); color: var(--text); font-weight: 600; }
  .badge { background: var(--accent); color: var(--on-accent); border-radius: 99px; font-size: 0.68rem; font-weight: 700; padding: 0 0.45em; min-width: 1.3em; text-align: center; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
  .pane { flex: 1; min-width: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; padding-right: 0.3rem; }
  h2 { margin: 0; font-size: 1rem; font-weight: 600; }
  @media (max-width: 700px) { .split { flex-direction: column; } .side { flex-basis: auto; flex-direction: row; flex-wrap: wrap; } }
</style>
