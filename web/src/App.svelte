<script lang="ts">
  import { tick } from "svelte";
  import { t, lang, LANGS, type Lang } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import Info from "./components/Info.svelte";
  import Workbench from "./components/Workbench.svelte";
  import { route, href } from "./lib/route";
  import { theme, toggleTheme } from "./lib/theme";
  import { buildGroups, buildTools } from "./lib/tools";

  // Os textos das ferramentas fanse no idioma actual: cámbianse ao cambiar de idioma.
  const build = (_l: Lang) => ({ tools: buildTools(), groups: buildGroups() });
  $: ({ tools, groups } = build($lang));
  $: tool = tools.find((x) => x.id === $route);
  $: if ($route && !tool) location.hash = "";

  const ACCENT: Record<string, string> = { imaxe: "var(--accent-imaxe)", pdf: "var(--accent-pdf)", audio: "var(--accent-audio)" };

  let h1: HTMLElement | undefined;
  // Ao cambiar de pantalla o foco vai ao título (lectores de pantalla) e a páxina volve arriba.
  $: $route, void tick().then(() => (scrollTo(0, 0), h1?.focus({ preventScroll: true })));
  $: document.title = tool ? `${tool.title} · Arquivos do Trasno` : "Arquivos do Trasno";
</script>

<a class="skip" href="#main" on:click|preventDefault={() => h1?.focus()}>{$t("Ir ao contido")}</a>

<header class="top">
  <div class="top-in">
    {#if tool}
      <a class="back" href={href("")} aria-label={$t("Volver ao inicio")}><Icon name="back" size={24} /></a>
    {:else}
      <img class="logo" src="/trasno/base.png" alt="" width="32" height="32" />
    {/if}
    <span class="brand">Arquivos do Trasno</span>
    <span class="grow"></span>
    <label class="lang">
      <span class="sr-only">{$t("Idioma")}</span>
      <select bind:value={$lang} aria-label={$t("Idioma")}>
        {#each LANGS as l (l.id)}<option value={l.id as Lang} title={l.label}>{l.id.toUpperCase()}</option>{/each}
      </select>
    </label>
    <button type="button" class="round" on:click={toggleTheme} aria-label={$theme === "dark" ? $t("Tema claro") : $t("Tema escuro")} title={$theme === "dark" ? $t("Tema claro") : $t("Tema escuro")}>
      <Icon name={$theme === "dark" ? "sun" : "moon"} size={22} />
    </button>
  </div>
</header>

<main id="main">
  {#if tool}
    {#key tool.id}
      <h1 tabindex="-1" bind:this={h1}>{tool.title}<Info text={tool.info} /></h1>
      <Workbench {tool} />
    {/key}
  {:else}
    <section class="hero">
      <img src="/trasno/base.png" alt="" width="96" height="96" />
      <div>
        <h1 tabindex="-1" bind:this={h1}>Arquivos do Trasno</h1>
        <p>{$t("Imaxe e PDF directamente no teu aparello. Os ficheiros non se envían a ningún servidor.")}</p>
      </div>
    </section>
    {#each groups as g (g.id)}
      <section class="group" aria-labelledby={`g-${g.id}`}>
        <h2 id={`g-${g.id}`}>{g.label}</h2>
        <ul class="cards">
          {#each tools.filter((x) => x.group === g.id) as x (x.id)}
            <li>
              <a class="card" href={href(x.id)} style={`--c:${ACCENT[x.group]}`}>
                <span class="ic"><Icon name={x.icon} size={26} /></span>
                <span class="tx"><span class="ti">{x.title}</span><span class="de">{x.desc}</span></span>
                <Icon name="chevron-right" size={20} />
              </a>
            </li>
          {/each}
        </ul>
      </section>
    {/each}
  {/if}
</main>

<style>
  .skip { position: absolute; left: 0.5rem; top: -4rem; z-index: 100; padding: 0.6rem 1rem; background: var(--accent); color: var(--on-accent); border-radius: var(--radius-md); font-weight: 600; text-decoration: none; }
  .skip:focus { top: 0.5rem; }
  .top { position: sticky; top: 0; z-index: 20; background: var(--bg-rail); border-bottom: 1px solid var(--border); padding-top: env(safe-area-inset-top); }
  .top-in { max-width: 60rem; margin: 0 auto; min-height: 3.5rem; display: flex; align-items: center; gap: 0.6rem; padding: 0 max(1rem, env(safe-area-inset-right)) 0 max(1rem, env(safe-area-inset-left)); }
  .brand { font-weight: 700; font-size: var(--fs-lg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .grow { flex: 1; }
  .logo { image-rendering: pixelated; flex: none; }
  .back, .round { flex: none; display: grid; place-items: center; width: 44px; height: 44px; margin-left: -0.5rem; padding: 0; background: none; border: 0; border-radius: var(--radius-md); color: var(--text); }
  .round { margin-left: 0; }
  .back:hover, .round:hover { background: var(--bg-hover); }
  .back:focus-visible, .round:focus-visible, select:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  select { min-height: 44px; padding: 0 0.5rem; background: var(--bg-card); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-md); font-size: var(--fs-md); }
  main { max-width: 60rem; margin: 0 auto; padding: 1.25rem max(1rem, env(safe-area-inset-right)) 1.5rem max(1rem, env(safe-area-inset-left)); }
  h1 { margin: 0 0 1.1rem; font-size: var(--fs-xl); line-height: 1.2; display: flex; align-items: center; gap: 0.6rem; outline: none; }
  .hero { display: flex; align-items: center; gap: 1rem; margin-bottom: 1.5rem; }
  .hero img { image-rendering: pixelated; flex: none; }
  .hero h1 { margin-bottom: 0.3rem; }
  .hero p { margin: 0; color: var(--text-muted); max-width: 34rem; }
  .group { margin-bottom: 1.5rem; }
  h2 { margin: 0 0 0.6rem; font-size: var(--fs-md); text-transform: none; color: var(--text-muted); font-weight: 700; }
  .cards { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: minmax(0, 1fr); gap: 0.6rem; }
  .card { display: flex; align-items: center; gap: 0.9rem; min-height: 4.5rem; padding: 0.75rem 0.9rem; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); color: var(--text); text-decoration: none; -webkit-tap-highlight-color: transparent; }
  .card:hover { background: var(--bg-hover); }
  .card:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  .ic { flex: none; display: grid; place-items: center; width: 2.9rem; height: 2.9rem; border-radius: var(--radius-md); background: color-mix(in srgb, var(--c) 16%, transparent); color: var(--c); }
  .tx { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .ti { font-weight: 700; }
  .de { font-size: var(--fs-sm); color: var(--text-muted); }
  .card :global(svg:last-child) { color: var(--text-faint); flex: none; }
  @media (min-width: 640px) { .cards { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
  @media (min-width: 960px) { .cards { grid-template-columns: repeat(3, minmax(0, 1fr)); } main { padding-top: 2rem; } }
</style>
