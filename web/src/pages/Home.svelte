<script lang="ts">
  import { t } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import { href } from "../lib/route";
  import { heroSlot } from "../lib/hero";
  import type { Group } from "../lib/catalog";
  import type { Tool } from "../lib/tools";

  export let tools: Tool[];
  export let groups: Group[];
  export let h1: HTMLElement | undefined = undefined;

  $: count = (id: string) => tools.filter((x) => x.group === id).length;
</script>

<section class="hero">
  <div class="hero-tr" use:heroSlot></div>
  <div class="hero-tx">
    <h1 tabindex="-1" bind:this={h1}>Arquivos do Trasno</h1>
    <p class="lead">{$t("Son o trasno. Converto, comprimo e arranxo imaxes, PDF, audio, vídeo e subtítulos sen que os ficheiros saian do teu aparello.")}</p>
    <div class="cta">
      <a class="btn primary" href={href("ferramentas")}>{$t("Ver as ferramentas")}</a>
      <a class="btn" href={href("descarga")}><Icon name="download" size={20} />{$t("Descargar a app")}</a>
    </div>
  </div>
</section>

<ul class="areas" aria-label={$t("Que podes facer")}>
  {#each groups as g (g.id)}
    <li>
      <a class="area" href={href(`ferramentas/${g.id}`)} style={`--c:${g.color}`}>
        <span class="ic"><Icon name={g.icon} size={30} /></span>
        <span class="nm">{g.label}</span>
        <span class="ct">{count(g.id)} {$t("ferramentas")}</span>
        <span class="ds">{g.desc}</span>
      </a>
    </li>
  {/each}
</ul>

<section class="band" aria-labelledby="priv">
  <span class="bic"><Icon name="lock" size={28} /></span>
  <div>
    <h2 id="priv">{$t("Os teus ficheiros quedan contigo")}</h2>
    <p>{$t("Todo se procesa no teu navegador. Non hai contas, publicidade nin rastrexadores.")}</p>
  </div>
  <a class="btn" href={href("privacidade")}>{$t("Privacidade e datos")}</a>
</section>

<section class="band app" aria-labelledby="band-app">
  <span class="bic"><Icon name="download" size={28} /></span>
  <div>
    <h2 id="band-app">{$t("Na app para Windows")}</h2>
    <p>{$t("Ficheiros grandes e vídeo con FFmpeg completo, descargas por ligazón e torrents, editor de vídeo e visor 3D.")}</p>
  </div>
  <a class="btn primary" href={href("descarga")}>{$t("Descargar a app")}</a>
</section>

<style>
  .hero { display: flex; flex-direction: column; align-items: center; gap: var(--sp-5); padding: var(--sp-4) 0 var(--sp-6); text-align: center; }
  /* O oco do trasno: o trasno de verdade vive na cabeceira e para aquí cando a páxina está arriba. */
  .hero-tr { flex: none; width: 160px; height: 176px; }
  .hero-tx { min-width: 0; }
  h1 { margin: 0 0 var(--sp-3); font-size: var(--fs-display); line-height: 1.05; outline: none; letter-spacing: var(--tracking-display); font-weight: var(--fw-strong); }
  .lead { margin: 0 auto var(--sp-5); max-width: 32rem; color: var(--text-muted); font-size: var(--fs-lg); line-height: 1.5; }
  .cta { display: flex; flex-wrap: wrap; justify-content: center; gap: var(--sp-3); }
  .cta .btn { flex: 1 1 12rem; max-width: 16rem; }

  .areas { list-style: none; margin: 0 0 var(--sp-6); padding: 0; display: grid; gap: var(--sp-3); grid-template-columns: minmax(0, 1fr); }
  .area { display: grid; grid-template-columns: auto minmax(0, 1fr); grid-template-areas: "ic nm" "ic ct" "ic ds"; column-gap: var(--sp-4); row-gap: 2px; align-items: center; height: 100%; padding: var(--sp-4); background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); color: var(--text); text-decoration: none; transition: border-color var(--dur) var(--ease), background var(--dur) var(--ease); }
  .area:hover { background: var(--bg-hover); border-color: var(--c); }
  .area:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  .ic { grid-area: ic; display: grid; place-items: center; width: 3.25rem; height: 3.25rem; border-radius: var(--radius-md); background: color-mix(in srgb, var(--c) 16%, transparent); color: var(--c); }
  .nm { grid-area: nm; align-self: end; font-size: var(--fs-h3); font-weight: var(--fw-strong); line-height: 1.2; }
  .ct { grid-area: ct; align-self: start; color: var(--text-muted); font-size: var(--fs-sm); }
  .ds { grid-area: ds; align-self: start; color: var(--text-muted); font-size: var(--fs-sm); line-height: 1.45; }

  .band { display: flex; flex-direction: column; align-items: flex-start; gap: var(--sp-4); margin-bottom: var(--sp-4); padding: var(--sp-5); background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); }
  .band.app { border-color: var(--accent); }
  .bic { display: grid; place-items: center; width: 3.25rem; height: 3.25rem; border-radius: var(--radius-md); background: color-mix(in srgb, var(--accent) 16%, transparent); color: var(--accent); }
  h2 { margin: 0 0 var(--sp-1); font-size: var(--fs-h3); font-weight: var(--fw-strong); line-height: 1.25; }
  .band p { margin: 0; color: var(--text-muted); max-width: 40rem; }

  /* Dúas e catro columnas: icona enriba, para que os nomes longos non rebosen. */
  @media (min-width: 480px) {
    .areas { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--sp-4); }
    .area { grid-template-columns: minmax(0, 1fr); grid-template-areas: "ic" "nm" "ct" "ds"; align-items: start; }
    .nm { margin-top: var(--sp-3); }
    .ds { margin-top: var(--sp-3); }
  }
  @media (min-width: 720px) {
    .hero { flex-direction: row; text-align: left; gap: var(--sp-7); padding-top: var(--sp-6); }
    .lead { margin-left: 0; }
    .cta { justify-content: flex-start; }
    .areas { grid-template-columns: repeat(4, minmax(0, 1fr)); }
    .band { flex-direction: row; align-items: center; }
    .band > div { flex: 1; }
  }
</style>
