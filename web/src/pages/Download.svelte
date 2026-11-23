<script lang="ts">
  import { onMount } from "svelte";
  import { t, lang } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import GitHubMark from "../components/GitHubMark.svelte";
  import Trasno from "../components/Trasno.svelte";
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
  <Trasno size={96} />
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

<section class="card main" aria-labelledby="latest">
  <h2 id="latest">{$t("Última versión")}</h2>
  {#if release && exe}
    <p class="ver"><strong>{release.version}</strong><span>{$t("Publicada o {date}", { date })}</span></p>
    <a class="btn primary big" href={exe.url} rel="noopener noreferrer">
      <Icon name="download" size={22} />
      <span>{$t("Descargar para Windows")}<small>{$t("Instalador (.exe) · {size}", { size: formatBytes(exe.size) })}</small></span>
    </a>
    {#if msi}
      <a class="alt" href={msi.url} rel="noopener noreferrer"><Icon name="download" size={18} />{$t("Paquete MSI · {size}", { size: formatBytes(msi.size) })}</a>
    {/if}
    <p class="small">{$t("Windows 10 e 11. A app avisa cando hai unha versión nova.")}</p>
  {:else if failed}
    <p role="status">{$t("Non puiden consultar GitHub agora. Podes ver as versións na súa páxina.")}</p>
    <a class="btn primary big" href={LATEST_URL} rel="noopener noreferrer"><GitHubMark size={22} /><span>{$t("Ver a última versión en GitHub")}</span></a>
  {:else}
    <p role="status">{$t("Buscando a última versión…")}</p>
  {/if}
</section>

<section class="card" aria-labelledby="gh">
  <h2 id="gh">{$t("Código aberto")}</h2>
  <p>{$t("O código está en GitHub con licenza MIT. Alí tes todas as versións, as novidades de cada unha e onde avisar dun erro.")}</p>
  <ul class="links">
    <li><a class="btn" href={REPO_URL} rel="noopener noreferrer"><GitHubMark size={20} />{$t("Código en GitHub")}</a></li>
    <li><a class="btn" href={RELEASES_URL} rel="noopener noreferrer"><Icon name="newspaper" size={20} />{$t("Todas as versións")}</a></li>
    <li><a class="btn" href={`${REPO_URL}/issues`} rel="noopener noreferrer"><Icon name="chat" size={20} />{$t("Avisar dun erro")}</a></li>
  </ul>
</section>

<section class="card" aria-labelledby="web">
  <h2 id="web">{$t("Sen instalar nada")}</h2>
  <p>{$t("A web tamén se pode instalar coma unha app. No iPhone: Compartir → Engadir á pantalla de inicio. En Android: menú do navegador → Instalar aplicación.")}</p>
  <a class="btn" href={href("ferramentas")}>{$t("Ver as ferramentas")}<Icon name="arrow-right" size={20} /></a>
</section>

<style>
  .head { display: flex; align-items: center; gap: 1.1rem; margin-bottom: 1.25rem; }
  h1 { margin: 0 0 0.3rem; font-size: var(--fs-xl); line-height: 1.2; outline: none; }
  .lead { margin: 0; color: var(--text-muted); max-width: 36rem; }
  h2 { margin: 0; font-size: var(--fs-lg); }
  .note { margin: 0 0 1rem; padding: 0.8rem 1rem; background: color-mix(in srgb, var(--accent-pdf) 12%, var(--bg-card)); border: 1px solid var(--accent-pdf); border-radius: var(--radius-md); }
  .card { display: flex; flex-direction: column; align-items: flex-start; gap: 0.8rem; margin-bottom: 1rem; padding: 1.1rem; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); }
  .card p { margin: 0; }
  .main { border-color: var(--accent); }
  .ver { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.2rem 0.8rem; }
  .ver strong { font-size: 1.6rem; line-height: 1.1; font-variant-numeric: tabular-nums; }
  .ver span { color: var(--text-muted); font-size: var(--fs-sm); }
  .big { align-self: stretch; min-height: 4rem; padding: 0.6rem 1.2rem; }
  .big span { display: flex; flex-direction: column; align-items: flex-start; text-align: left; line-height: 1.25; }
  .big small { font-size: var(--fs-sm); font-weight: 500; }
  .alt { display: inline-flex; align-items: center; gap: 0.4rem; min-height: 44px; color: var(--text); font-weight: 600; text-underline-offset: 3px; }
  .small { color: var(--text-muted); font-size: var(--fs-sm); }
  .links { list-style: none; margin: 0; padding: 0; display: flex; flex-wrap: wrap; gap: 0.5rem; align-self: stretch; }
  .links li { flex: 1 1 13rem; display: flex; }
  .links .btn { flex: 1; }
  a:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  @media (min-width: 640px) { .big { align-self: flex-start; min-width: 22rem; } }
</style>
