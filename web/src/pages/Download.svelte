<script lang="ts">
  import { onMount } from "svelte";
  import { t, lang } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import GitHubMark from "../components/GitHubMark.svelte";
  import { heroSlot } from "../lib/hero";
  import { formatBytes } from "../lib/files";
  import { LATEST_URL, REPO_URL, RELEASES_URL, latestRelease, platform, type Release } from "../lib/release";
  import { href } from "../lib/route";

  export let h1: HTMLElement | undefined = undefined;

  let release: Release | null = null;
  let failed = false;
  const os = platform();

  onMount(() => {
    latestRelease().then((r) => (release = r), () => (failed = true));
  });

  $: exe = release?.assets.find((a) => a.kind === "exe");
  $: msi = release?.assets.find((a) => a.kind === "msi");
  $: date = release ? new Date(release.date).toLocaleDateString($lang === "gl" ? "gl-ES" : $lang === "es" ? "es-ES" : "en-GB", { day: "numeric", month: "long", year: "numeric" }) : "";
</script>

<div class="head">
  <div class="hero-tr" use:heroSlot></div>
  <div>
    <h1 tabindex="-1" bind:this={h1}>{$t("Descargar a app")}</h1>
    <p class="lead">{$t("Arquivos do Trasno para Windows: todas as ferramentas, máis as que non caben nun navegador.")}</p>
  </div>
</div>

{#if os === "mobile"}
  <p class="note" role="note">{$t("A app é para Windows. No móbil usa a web: instálase desde o navegador.")}</p>
{:else if os === "other"}
  <p class="note" role="note">{$t("De momento a app só existe para Windows. Na web tes moitas das ferramentas.")}</p>
{/if}

<section class="block" aria-labelledby="latest">
  <h2 id="latest">{$t("Última versión")}</h2>
  {#if release && exe}
    <p class="ver"><strong>{release.version}</strong><span>{$t("Publicada o {date}", { date })}</span></p>
    <div class="row">
      <a class="btn primary big" href={exe.url} rel="noopener noreferrer">
        <Icon name="download" size={22} />
        <span>{$t("Descargar para Windows")}<small>{$t("Instalador (.exe) · {size}", { size: formatBytes(exe.size) })}</small></span>
      </a>
      {#if msi}
        <a class="btn sm" href={msi.url} rel="noopener noreferrer"><Icon name="download" size={16} />{$t("Paquete MSI · {size}", { size: formatBytes(msi.size) })}</a>
      {/if}
    </div>
    <p class="small">{$t("Windows 10 e 11. A app actualízase soa; podes apagalo en Axustes.")}</p>
  {:else if failed}
    <p role="status">{$t("Non puiden consultar GitHub agora. Podes ver as versións na súa páxina.")}</p>
    <a class="btn primary" href={LATEST_URL} rel="noopener noreferrer"><GitHubMark size={20} />{$t("Ver a última versión en GitHub")}</a>
  {:else}
    <p role="status">{$t("Buscando a última versión…")}</p>
  {/if}
</section>

<section class="block" aria-labelledby="gh">
  <h2 id="gh">{$t("Código aberto")}</h2>
  <p>{$t("O código está en GitHub con licenza MIT. Alí tes todas as versións, as novidades de cada unha e onde avisar dun erro.")}</p>
  <div class="row">
    <a class="btn" href={REPO_URL} rel="noopener noreferrer"><GitHubMark size={20} />{$t("Código en GitHub")}</a>
    <a class="btn" href={RELEASES_URL} rel="noopener noreferrer"><Icon name="newspaper" size={20} />{$t("Todas as versións")}</a>
    <a class="btn" href={`${REPO_URL}/issues`} rel="noopener noreferrer"><Icon name="chat" size={20} />{$t("Avisar dun erro")}</a>
  </div>
</section>

<section class="block" aria-labelledby="web">
  <h2 id="web">{$t("Sen instalar nada")}</h2>
  <p>{$t("A web tamén se pode instalar coma unha app. No iPhone: Compartir → Engadir á pantalla de inicio. En Android: menú do navegador → Instalar aplicación.")}</p>
  <div class="row"><a class="btn" href={href("ferramentas")}>{$t("Ver as ferramentas")}</a></div>
</section>

<style>
  .hero-tr { flex: none; width: 96px; height: 106px; }
  .head { display: flex; align-items: center; gap: var(--sp-5); margin-bottom: var(--sp-6); }
  h1 { margin: 0 0 var(--sp-2); font-size: var(--fs-h1); line-height: 1.15; outline: none; font-weight: var(--fw-strong); letter-spacing: var(--tracking-heading); }
  .lead { margin: 0; color: var(--text-muted); max-width: 36rem; }
  .note { margin: 0 0 var(--sp-5); padding-left: var(--sp-4); border-left: 3px solid var(--accent-pdf); }
  .block { display: flex; flex-direction: column; align-items: flex-start; gap: var(--sp-3); margin-bottom: var(--sp-4); padding: var(--sp-5); background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); }
  .block:first-of-type { border-color: var(--accent); }
  h2 { margin: 0; font-size: var(--fs-h3); font-weight: var(--fw-strong); }
  .block p { margin: 0; max-width: 40rem; }
  .ver { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.2rem var(--sp-4); }
  .ver strong { font-size: var(--fs-h1); line-height: 1.1; font-variant-numeric: tabular-nums; }
  .ver span { color: var(--text-muted); font-size: var(--fs-sm); }
  .small { color: var(--text-muted); font-size: var(--fs-sm); }
  .row { display: flex; flex-wrap: wrap; gap: var(--sp-3); align-self: stretch; }
  /* Secundario discreto (o MSI é para administradores): letra pequena, sen caixa; o alvo de toque segue en 44 px. */
  .sm { min-height: 44px; padding: 0 var(--sp-3); background: none; border-color: transparent; color: var(--text-muted); font-size: var(--fs-sm); font-weight: var(--fw-medium); align-self: center; }
  .sm:hover { color: var(--text); }
  .big { min-height: 4rem; padding: 0.6rem 1.4rem; }
  .big span { display: flex; flex-direction: column; align-items: flex-start; text-align: left; line-height: 1.25; }
  .big small { font-size: var(--fs-sm); font-weight: var(--fw-medium); }
  a:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
</style>
