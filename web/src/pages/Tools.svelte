<script lang="ts">
  import { t } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import ToolIcon from "../components/ToolIcon.svelte";
  import { href } from "../lib/route";
  import type { Group } from "../lib/catalog";
  import type { Tool } from "../lib/tools";

  export let tools: Tool[];
  export let groups: Group[];
  /** Grupo elixido na ruta («ferramentas/pdf»), ou baleiro para todos. */
  export let only = "";
  export let h1: HTMLElement | undefined = undefined;

  let query = "";
  const plain = (s: string) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  $: q = plain(query.trim());
  $: shown = tools.filter((x) => (!only || x.group === only) && (!q || plain(`${x.title} ${x.desc}`).includes(q)));
  $: visibleGroups = groups.filter((g) => shown.some((x) => x.group === g.id));
</script>

<h1 tabindex="-1" bind:this={h1}>{$t("Ferramentas")}</h1>

<div class="filters">
  <label class="search">
    <span class="sr-only">{$t("Buscar unha ferramenta")}</span>
    <Icon name="search" size={20} />
    <input type="search" bind:value={query} placeholder={$t("Buscar unha ferramenta")} autocomplete="off" autocapitalize="off" spellcheck="false" />
  </label>
  <nav class="chips" aria-label={$t("Tipo de ficheiro")}>
    <a class="chip" class:on={!only} aria-current={!only ? "page" : undefined} href={href("ferramentas")}>{$t("Todas")}<span>{tools.length}</span></a>
    {#each groups as g (g.id)}
      <a class="chip" class:on={only === g.id} aria-current={only === g.id ? "page" : undefined} href={href(`ferramentas/${g.id}`)} style={`--c:${g.color}`}>{g.label}<span>{tools.filter((x) => x.group === g.id).length}</span></a>
    {/each}
  </nav>
</div>

{#each visibleGroups as g (g.id)}
  <section class="group" aria-labelledby={`g-${g.id}`} style={`--c:${g.color}`}>
    <h2 id={`g-${g.id}`}><Icon name={g.icon} size={20} />{g.label}</h2>
    <ul class="cards">
      {#each shown.filter((x) => x.group === g.id) as x (x.id)}
        <li>
          <a class="card" href={href(x.id)}>
            <span class="ic"><ToolIcon name={x.icon} size={26} /></span>
            <span class="tx"><span class="ti">{x.title}</span><span class="de">{x.desc}</span></span>
            <Icon name="chevron-right" size={20} />
          </a>
        </li>
      {/each}
    </ul>
  </section>
{:else}
  <p class="none" role="status">{$t("Ningunha ferramenta coincide. Proba con outra palabra.")}</p>
{/each}

<style>
  h1 { margin: 0 0 var(--sp-4); font-size: var(--fs-h1); line-height: 1.15; outline: none; font-weight: var(--fw-heading); letter-spacing: var(--tracking-heading); }
  .filters { display: flex; flex-direction: column; gap: var(--sp-3); margin-bottom: var(--sp-6); }
  .search { display: flex; align-items: center; gap: 0.6rem; min-height: var(--control-h); padding: 0 0.8rem; background: var(--bg-input); border: 1px solid var(--border-strong); border-radius: var(--radius-md); color: var(--text-muted); }
  .search:focus-within { outline: 3px solid var(--accent); outline-offset: 1px; }
  .search input { flex: 1; min-width: 0; height: 100%; min-height: calc(var(--control-h) - 2px); padding: 0; background: none; border: 0; outline: none; color: var(--text); }
  .search input::placeholder { color: var(--text-muted); }
  .chips { display: flex; flex-wrap: wrap; gap: 0.4rem; }
  .chip { display: inline-flex; align-items: center; gap: 0.45rem; min-height: 44px; padding: 0 0.9rem; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: 999px; color: var(--text); font-weight: 600; text-decoration: none; }
  .chip span { font-size: var(--fs-xs); font-weight: 600; color: var(--text-muted); }
  .chip:hover { background: var(--bg-hover); }
  .chip.on { border-color: var(--accent); box-shadow: inset 0 0 0 1px var(--accent); background: color-mix(in srgb, var(--accent) 12%, var(--bg-card)); }
  .chip:focus-visible, .card:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  .group { margin-bottom: var(--sp-6); }
  h2 { display: flex; align-items: center; gap: var(--sp-2); margin: 0 0 var(--sp-3); padding-bottom: var(--sp-2); border-bottom: 1px solid var(--border); font-size: var(--fs-h3); color: var(--text); font-weight: var(--fw-heading); }
  h2 :global(svg) { color: var(--c); }
  .cards { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: minmax(0, 1fr); gap: var(--sp-3); }
  .cards li { display: flex; }
  .card { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.9rem; min-height: 5rem; padding: var(--sp-3) var(--sp-4); background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); color: var(--text); text-decoration: none; -webkit-tap-highlight-color: transparent; transition: transform var(--dur) var(--ease), box-shadow var(--dur) var(--ease), border-color var(--dur) var(--ease); }
  .card:hover { background: var(--bg-hover); border-color: var(--c); transform: translateY(-1px); box-shadow: var(--elev-2); }
  .ic { flex: none; display: grid; place-items: center; width: 3rem; height: 3rem; border-radius: var(--radius-md); background: color-mix(in srgb, var(--c) 16%, transparent); color: var(--c); }
  .tx { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .ti { font-weight: 700; font-size: var(--fs-md); }
  .de { font-size: var(--fs-sm); color: var(--text-muted); }
  .card :global(svg:last-child) { color: var(--text-faint); flex: none; }
  .none { color: var(--text-muted); }
  @media (prefers-reduced-motion: reduce) { .card:hover { transform: none; } }
  @media (min-width: 640px) { .cards { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
  @media (min-width: 960px) { .cards { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
</style>
