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
  import { heroInView } from "./lib/hero";

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
  // Inicio e descarga levan o trasno en grande: a cabeceira só o ten cando ese se vai.
  $: logoOff = (view === "home" || view === "download") && $heroInView;
</script>

<a class="skip" href="#main" on:click|preventDefault={() => h1?.focus()}>{$t("Ir ao contido")}</a>

<header class="top">
  <div class="top-in">
    {#if tool}
      <a class="back" href={href("ferramentas")} aria-label={$t("Volver ás ferramentas")}><Icon name="back" size={24} /></a>
    {:else}
      <a class="logo" class:off={logoOff} href={href("")} aria-label={$t("Inicio")} tabindex={logoOff ? -1 : undefined}><img src="/trasno/base.png" alt="" width="32" height="32" /></a>
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
    <div class="foot-brand">
      <b>{NAME}</b>
      <p>{$t("Código aberto, licenza MIT.")}</p>
    </div>
    <nav class="foot-nav" aria-label={$t("Máis")}>
      <a href={href("ferramentas")}>{$t("Ferramentas")}</a>
      <a href={href("descarga")}>{$t("Descarga")}</a>
      <a href={href("privacidade")}>{$t("Privacidade e datos")}</a>
      <a href={REPO_URL} rel="noopener noreferrer">GitHub</a>
      <a href={RELEASES_URL} rel="noopener noreferrer">{$t("Versións")}</a>
    </nav>
  </div>
</footer>

<style>
  :global(#app) { min-height: 100vh; min-height: 100dvh; display: flex; flex-direction: column; }
  /* main é fillo dunha columna flex: sen width, as marxes automáticas fano encoller ao contido (p. ex. unha busca sen resultados). */
  .skip { position: absolute; left: 0.5rem; top: -4rem; z-index: 100; padding: 0.6rem 1rem; background: var(--accent); color: var(--on-accent); border-radius: var(--radius-md); font-weight: var(--fw-strong); text-decoration: none; }
  .skip:focus { top: 0.5rem; }
  .top { position: sticky; top: 0; z-index: 20; background: var(--bg-rail); border-bottom: 1px solid var(--border); padding-top: env(safe-area-inset-top); }
  .top-in { max-width: var(--page-w); margin: 0 auto; min-height: 3.5rem; display: flex; flex-wrap: wrap; align-items: center; gap: 0.4rem 0.6rem; padding: 0 max(var(--gutter), env(safe-area-inset-right)) 0 max(var(--gutter), env(safe-area-inset-left)); }
  .brand { flex: 1 1 0; color: var(--text); font-weight: var(--fw-strong); font-size: var(--fs-lg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; text-decoration: none; }
  .grow { flex: 1; }
  .logo { flex: none; display: grid; place-items: center; width: 44px; height: 44px; margin-left: -6px; }
  /* O oco do trasno é sempre o mesmo: o texto da cabeceira non se move cando aparece ou desaparece. */
  .logo img { image-rendering: pixelated; transition: opacity 0.25s var(--ease), transform 0.35s var(--ease); }
  /* Con o trasno grande á vista, o da cabeceira agáchase un chisco máis abaixo; ao saír aquel, sobe ao seu sitio. */
  .logo.off { visibility: hidden; transition: visibility 0s 0.35s; }
  .logo.off img { opacity: 0; transform: translateY(14px); }
  /* Os controis da cabeceira (volver, GitHub, idioma, tema): mesma altura, mesmo marco, mesmo fondo. */
  .back, .round, select { flex: none; height: 44px; background: var(--bg-card); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-md); }
  .back, .round { display: grid; place-items: center; width: 44px; padding: 0; text-decoration: none; }
  select { padding: 0 0.6rem; font-size: var(--fs-md); font-weight: var(--fw-strong); }
  .back:hover, .round:hover, select:hover { background: var(--bg-hover); }
  .back:focus-visible, .round:focus-visible, select:focus-visible, .logo:focus-visible, .brand:focus-visible, .nav a:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  /* Móbil: a navegación vai nunha segunda fila, repartida por igual. */
  .nav { order: 9; flex: 1 0 100%; display: flex; gap: 0.25rem; padding-bottom: 0.4rem; }
  .nav a { flex: 1; display: grid; place-items: center; min-height: 44px; padding: 0 0.6rem; border-radius: var(--radius-md); color: var(--text-muted); font-weight: var(--fw-medium); text-decoration: none; }
  .nav a:hover { color: var(--text); text-decoration: underline; text-decoration-color: var(--accent); text-decoration-thickness: 2px; text-underline-offset: 5px; }
  .nav a[aria-current="page"] { color: var(--text); font-weight: var(--fw-strong); }
  main { width: 100%; max-width: var(--page-w); margin: 0 auto; padding: clamp(1.25rem, 0.6rem + 2.4vw, 2.5rem) max(var(--gutter), env(safe-area-inset-right)) var(--sp-7) max(var(--gutter), env(safe-area-inset-left)); flex: 1 0 auto; }
  .tool-h { margin: 0 0 var(--sp-4); font-size: var(--fs-h1); font-weight: var(--fw-strong); letter-spacing: var(--tracking-heading); line-height: 1.2; display: flex; align-items: center; gap: 0.6rem; outline: none; }
  .foot { flex: none; margin-top: var(--sp-5); border-top: 1px solid var(--border); background: var(--bg-rail); }
  .foot-in { max-width: var(--page-w); margin: 0 auto; display: flex; flex-direction: column; gap: var(--sp-4); padding: var(--sp-5) max(var(--gutter), env(safe-area-inset-right)) calc(var(--sp-5) + env(safe-area-inset-bottom)) max(var(--gutter), env(safe-area-inset-left)); }
  .foot-brand b { display: block; line-height: 1.2; }
  .foot p { margin: 0; color: var(--text-muted); font-size: var(--fs-sm); }
  /* Móbil: ligazóns en dúas columnas, cada unha cun obxectivo de toque de 48 px. */
  .foot-nav { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); border-top: 1px solid var(--border); padding-top: var(--sp-2); }
  .foot-nav a { display: flex; align-items: center; min-height: 48px; color: var(--text); font-size: var(--fs-md); text-decoration: none; }
  .foot-nav a:hover { text-decoration: underline; text-decoration-color: var(--accent); text-decoration-thickness: 2px; text-underline-offset: 5px; }
  .foot-nav a:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; border-radius: var(--radius-sm); }
  .grow { display: none; }
  @media (min-width: 720px) {
    .brand { flex: none; }
    .grow { display: block; }
    .nav { order: 0; flex: 0 1 auto; padding: 0 0 0 0.75rem; }
    .nav a { flex: none; padding: 0 0.9rem; }
  }
  @media (min-width: 720px) {
    .foot-in { flex-direction: row; align-items: center; justify-content: space-between; gap: var(--sp-6); }
    .foot-nav { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 0 var(--sp-5); border-top: 0; padding-top: 0; }
    .foot-nav a { min-height: 44px; font-size: var(--fs-sm); }
  }
</style>
