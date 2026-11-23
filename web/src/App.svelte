<script lang="ts">
  import { tick } from "svelte";
  import { t, lang, LANGS, type Lang } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import Info from "./components/Info.svelte";
  import Workbench from "./components/Workbench.svelte";
  import Privacy from "./components/Privacy.svelte";
  import GitHubMark from "./components/GitHubMark.svelte";
  import Home from "./pages/Home.svelte";
  import Tools from "./pages/Tools.svelte";
  import Download from "./pages/Download.svelte";
  import { route, href } from "./lib/route";
  import { theme, toggleTheme } from "./lib/theme";
  import { buildGroups, buildTools } from "./lib/catalog";
  import { REPO_URL, RELEASES_URL } from "./lib/release";

  // Os textos das ferramentas fanse no idioma actual: cámbianse ao cambiar de idioma.
  const build = (_l: Lang) => ({ tools: buildTools(), groups: buildGroups() });
  $: ({ tools, groups } = build($lang));
  $: tool = tools.find((x) => x.id === $route);
  $: [page, sub = ""] = $route.split("/") as [string, string?];
  $: view = tool ? "tool" : page === "ferramentas" ? "tools" : page === "descarga" ? "download" : page === "privacidade" ? "privacy" : $route === "" ? "home" : "none";
  $: if (view === "none") location.hash = "";
  $: only = groups.some((g) => g.id === sub) ? sub : "";

  let h1: HTMLElement | undefined;
  // Ao cambiar de pantalla o foco vai ao título (lectores de pantalla) e a páxina volve arriba.
  $: $route, void tick().then(() => (scrollTo(0, 0), h1?.focus({ preventScroll: true })));
  const NAME = "Arquivos do Trasno";
  $: document.title = tool ? `${tool.title} · ${NAME}` : view === "tools" ? `${$t("Ferramentas")} · ${NAME}` : view === "download" ? `${$t("Descargar a app")} · ${NAME}` : view === "privacy" ? `${$t("Privacidade e datos")} · ${NAME}` : NAME;

  $: nav = [
    { id: "home", to: "", label: $t("Inicio") },
    { id: "tools", to: "ferramentas", label: $t("Ferramentas") },
    { id: "download", to: "descarga", label: $t("Descarga") },
  ];
  $: current = view === "tool" ? "tools" : view;
</script>

<a class="skip" href="#main" on:click|preventDefault={() => h1?.focus()}>{$t("Ir ao contido")}</a>

<header class="top">
  <div class="top-in">
    {#if tool}
      <a class="back" href={href("ferramentas")} aria-label={$t("Volver ás ferramentas")}><Icon name="back" size={24} /></a>
    {:else}
      <a class="logo" href={href("")} aria-label={$t("Inicio")}><img src="/trasno/base.png" alt="" width="32" height="32" /></a>
    {/if}
    <a class="brand" href={href("")}>{NAME}</a>
    <nav class="nav" aria-label={$t("Principal")}>
      {#each nav as n (n.id)}
        <a href={href(n.to)} aria-current={current === n.id ? "page" : undefined}>{n.label}</a>
      {/each}
    </nav>
    <span class="grow"></span>
    <a class="round" href={REPO_URL} rel="noopener noreferrer" aria-label={$t("Código en GitHub")} title={$t("Código en GitHub")}><GitHubMark size={22} /></a>
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
      <h1 class="tool-h" tabindex="-1" bind:this={h1}>{tool.title}<Info text={tool.info} /></h1>
      <Workbench {tool} />
    {/key}
  {:else if view === "privacy"}
    <h1 class="tool-h" tabindex="-1" bind:this={h1}>{$t("Privacidade e datos")}</h1>
    <Privacy />
  {:else if view === "tools"}
    <Tools {tools} {groups} {only} bind:h1 />
  {:else if view === "download"}
    <Download bind:h1 />
  {:else}
    <Home {tools} {groups} bind:h1 />
  {/if}
</main>

<footer class="foot">
  <div class="foot-in">
    <nav class="foot-nav" aria-label={$t("Máis")}>
      <a href={href("ferramentas")}>{$t("Ferramentas")}</a>
      <a href={href("descarga")}>{$t("Descarga")}</a>
      <a href={href("privacidade")}>{$t("Privacidade e datos")}</a>
      <a href={REPO_URL} rel="noopener noreferrer">GitHub</a>
      <a href={RELEASES_URL} rel="noopener noreferrer">{$t("Versións")}</a>
    </nav>
    <p>{$t("Código aberto, licenza MIT.")}</p>
  </div>
</footer>

<style>
  .skip { position: absolute; left: 0.5rem; top: -4rem; z-index: 100; padding: 0.6rem 1rem; background: var(--accent); color: var(--on-accent); border-radius: var(--radius-md); font-weight: 600; text-decoration: none; }
  .skip:focus { top: 0.5rem; }
  .top { position: sticky; top: 0; z-index: 20; background: var(--bg-rail); border-bottom: 1px solid var(--border); padding-top: env(safe-area-inset-top); }
  .top-in { max-width: var(--page-w); margin: 0 auto; min-height: 3.5rem; display: flex; flex-wrap: wrap; align-items: center; gap: 0.4rem 0.6rem; padding: 0 max(1rem, env(safe-area-inset-right)) 0 max(1rem, env(safe-area-inset-left)); }
  .brand { flex: 1 1 0; color: var(--text); font-weight: 700; font-size: var(--fs-lg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; text-decoration: none; }
  .grow { flex: 1; }
  .logo { flex: none; display: grid; place-items: center; width: 44px; height: 44px; margin-left: -6px; }
  .logo img { image-rendering: pixelated; }
  /* Os controis da cabeceira (volver, GitHub, idioma, tema): mesma altura, mesmo marco, mesmo fondo. */
  .back, .round, select { flex: none; height: 44px; background: var(--bg-card); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-md); }
  .back, .round { display: grid; place-items: center; width: 44px; padding: 0; text-decoration: none; }
  select { padding: 0 0.6rem; font-size: var(--fs-md); font-weight: 600; }
  .back:hover, .round:hover, select:hover { background: var(--bg-hover); }
  .back:focus-visible, .round:focus-visible, select:focus-visible, .logo:focus-visible, .brand:focus-visible, .nav a:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  /* Móbil: a navegación vai nunha segunda fila, repartida por igual. */
  .nav { order: 9; flex: 1 0 100%; display: flex; gap: 0.25rem; padding-bottom: 0.4rem; }
  .nav a { flex: 1; display: grid; place-items: center; min-height: 44px; padding: 0 0.6rem; border-radius: var(--radius-md); color: var(--text-muted); font-weight: 600; text-decoration: none; border-bottom: 3px solid transparent; }
  .nav a:hover { background: var(--bg-hover); color: var(--text); }
  .nav a[aria-current="page"] { color: var(--text); border-bottom-color: var(--accent); }
  main { max-width: var(--page-w); margin: 0 auto; padding: 1.25rem max(1rem, env(safe-area-inset-right)) 1.5rem max(1rem, env(safe-area-inset-left)); min-height: 60vh; }
  .tool-h { margin: 0 0 var(--sp-4); font-size: var(--fs-h1); font-weight: var(--fw-heading); letter-spacing: var(--tracking-heading); line-height: 1.2; display: flex; align-items: center; gap: 0.6rem; outline: none; }
  .foot { margin-top: 1rem; border-top: 1px solid var(--border); background: var(--bg-rail); }
  .foot-in { max-width: var(--page-w); margin: 0 auto; padding: 0.75rem max(1rem, env(safe-area-inset-right)) calc(1.25rem + env(safe-area-inset-bottom)) max(1rem, env(safe-area-inset-left)); }
  .foot-nav { display: flex; flex-wrap: wrap; gap: 0 1.1rem; }
  .foot-nav a { display: inline-flex; align-items: center; min-height: 44px; color: var(--text); font-size: var(--fs-sm); text-underline-offset: 3px; }
  .foot-nav a:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  .foot p { margin: 0; color: var(--text-muted); font-size: var(--fs-sm); }
  .grow { display: none; }
  @media (min-width: 720px) {
    .brand { flex: none; }
    .grow { display: block; }
    .nav { order: 0; flex: 0 1 auto; padding: 0 0 0 0.75rem; }
    .nav a { flex: none; padding: 0 0.9rem; }
  }
  @media (min-width: 960px) { main { padding-top: 2rem; } }
</style>
