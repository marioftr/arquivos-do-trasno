<script lang="ts">
  import { t } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";
  import Trasno from "../components/Trasno.svelte";
  import ToolIcon from "../components/ToolIcon.svelte";
  import { href } from "../lib/route";
  import type { Group } from "../lib/catalog";
  import type { Tool } from "../lib/tools";

  export let tools: Tool[];
  export let groups: Group[];
  export let h1: HTMLElement | undefined = undefined;

  $: byGroup = (id: string) => tools.filter((x) => x.group === id);
</script>

<section class="hero">
  <div class="hero-in">
    <Trasno size={160} />
    <div class="hero-tx">
      <h1 tabindex="-1" bind:this={h1}>Arquivos do Trasno</h1>
      <p class="lead">{$t("Son o trasno. Converto, comprimo e arranxo imaxes, PDF, audio, vídeo e subtítulos sen que os ficheiros saian do teu aparello.")}</p>
      <div class="cta">
        <a class="btn primary" href={href("ferramentas")}>{$t("Ver as ferramentas")}<Icon name="arrow-right" size={20} /></a>
        <a class="btn" href={href("descarga")}><Icon name="download" size={20} />{$t("Descargar a app")}</a>
      </div>
      <ul class="facts" aria-label={$t("En cifras")}>
        <li><strong>{tools.length}</strong><span>{$t("ferramentas")}</span></li>
        <li><strong>0</strong><span>{$t("ficheiros enviados")}</span></li>
        <li><strong>0</strong><span>{$t("contas")}</span></li>
      </ul>
    </div>
  </div>
</section>

<section class="block" aria-labelledby="what">
  <h2 id="what">{$t("Que podes facer")}</h2>
  <ul class="areas">
    {#each groups as g (g.id)}
      <li class="area" style={`--c:${g.color}`}>
        <div class="area-head">
          <span class="ic"><Icon name={g.icon} size={26} /></span>
          <h3>{g.label}</h3>
        </div>
        <p>{g.desc}</p>
        <ul class="chips">
          {#each byGroup(g.id).slice(0, 6) as x (x.id)}
            <li><a href={href(x.id)}><ToolIcon name={x.icon} size={16} />{x.title}</a></li>
          {/each}
        </ul>
        <a class="more" href={href(`ferramentas/${g.id}`)}>{$t("Ver as {n}", { n: byGroup(g.id).length })}<Icon name="chevron-right" size={18} /></a>
      </li>
    {/each}
  </ul>
</section>

<section class="block" aria-labelledby="how">
  <h2 id="how">{$t("Como funciona")}</h2>
  <ol class="steps">
    <li><span class="n" aria-hidden="true">1</span><div><h3>{$t("Elixe os ficheiros")}</h3><p>{$t("Arrástraos, pégaos ou toca para elixilos.")}</p></div></li>
    <li><span class="n" aria-hidden="true">2</span><div><h3>{$t("Axusta o que queres")}</h3><p>{$t("Poucas opcións, con valores claros.")}</p></div></li>
    <li><span class="n" aria-hidden="true">3</span><div><h3>{$t("Garda o resultado")}</h3><p>{$t("Gárdao no aparello ou compárteo desde o móbil.")}</p></div></li>
  </ol>
</section>

<section class="block priv" aria-labelledby="priv">
  <span class="ic big"><Icon name="lock" size={30} /></span>
  <div>
    <h2 id="priv">{$t("Os teus ficheiros quedan contigo")}</h2>
    <p>{$t("Todo se procesa no teu navegador. Non hai contas, publicidade nin rastrexadores.")}</p>
    <a class="more" href={href("privacidade")}>{$t("Como tratamos os datos")}<Icon name="chevron-right" size={18} /></a>
  </div>
</section>

<section class="block" aria-labelledby="which">
  <h2 id="which">{$t("Na web ou na app")}</h2>
  <div class="two">
    <article class="col">
      <h3>{$t("Na web")}</h3>
      <ul>
        <li><Icon name="check" size={18} />{$t("Nada que instalar")}</li>
        <li><Icon name="check" size={18} />{$t("Vale no móbil e no ordenador")}</li>
        <li><Icon name="check" size={18} />{$t("Instálase coma unha app desde o navegador")}</li>
      </ul>
    </article>
    <article class="col accent">
      <h3>{$t("Na app para Windows")}</h3>
      <ul>
        <li><Icon name="check" size={18} />{$t("Ficheiros grandes e vídeo con FFmpeg completo")}</li>
        <li><Icon name="check" size={18} />{$t("Descargar vídeos por ligazón e torrents")}</li>
        <li><Icon name="check" size={18} />{$t("Editar vídeo, anotar imaxes, ver modelos 3D e máis")}</li>
        <li><Icon name="check" size={18} />{$t("Menú do botón dereito do Explorador")}</li>
      </ul>
      <a class="btn primary" href={href("descarga")}><Icon name="download" size={20} />{$t("Descargar a app")}</a>
    </article>
  </div>
</section>

<style>
  .hero { position: relative; padding: var(--sp-2) 0 0; }
  .hero-in { position: relative; overflow: hidden; display: flex; flex-direction: column; align-items: center; gap: var(--sp-5); text-align: center; padding: var(--sp-6) var(--sp-4) var(--sp-5); background: var(--glow), var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-xl); }
  .hero-tx { min-width: 0; }
  h1 { margin: 0 0 var(--sp-3); font-size: var(--fs-display); line-height: 1.05; outline: none; letter-spacing: var(--tracking-display); font-weight: var(--fw-heading); }
  .lead { margin: 0 auto var(--sp-5); max-width: 34rem; color: var(--text-muted); font-size: var(--fs-lg); line-height: 1.55; }
  .cta { display: flex; flex-wrap: wrap; justify-content: center; gap: var(--sp-3); }
  .cta .btn { flex: 1 1 13rem; max-width: 18rem; }
  .facts { list-style: none; margin: var(--sp-5) 0 0; padding: var(--sp-4) 0 0; display: flex; justify-content: center; gap: var(--sp-5) var(--sp-6); border-top: 1px solid var(--border); }
  .facts li { display: flex; flex-direction: column; align-items: center; min-width: 5rem; }
  .facts strong { font-size: var(--fs-h1); line-height: 1.1; color: var(--accent); font-variant-numeric: tabular-nums; font-weight: var(--fw-heading); }
  .facts span { font-size: var(--fs-sm); color: var(--text-muted); }

  .block { margin: var(--gap-section) 0; }
  h2 { margin: 0 0 var(--sp-5); font-size: var(--fs-xl); line-height: 1.2; letter-spacing: var(--tracking-heading); font-weight: var(--fw-heading); }
  h3 { margin: 0; font-size: var(--fs-h3); line-height: 1.25; font-weight: var(--fw-heading); }
  .ic { flex: none; display: grid; place-items: center; width: 3rem; height: 3rem; border-radius: var(--radius-md); background: color-mix(in srgb, var(--c, var(--accent)) 16%, transparent); color: var(--c, var(--accent)); }
  .ic.big { width: 4rem; height: 4rem; border-radius: var(--radius-lg); }

  .areas { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--sp-4); grid-template-columns: minmax(0, 1fr); }
  .area { display: flex; flex-direction: column; gap: var(--sp-3); padding: var(--sp-5); background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); box-shadow: var(--elev-1); transition: transform var(--dur) var(--ease), box-shadow var(--dur) var(--ease), border-color var(--dur) var(--ease); }
  .area:hover { transform: translateY(-2px); border-color: var(--c); box-shadow: var(--elev-2); }
  .area-head { display: flex; align-items: center; gap: var(--sp-3); }
  .area p { margin: 0; color: var(--text-muted); }
  .chips { list-style: none; margin: 0; padding: 0; display: flex; flex-wrap: wrap; gap: var(--sp-2); }
  .chips a { display: inline-flex; align-items: center; gap: 0.35rem; min-height: 44px; padding: 0 var(--sp-3); background: var(--bg); border: 1px solid var(--border); border-radius: 999px; color: var(--text); font-size: var(--fs-sm); text-decoration: none; transition: background var(--dur) var(--ease), border-color var(--dur) var(--ease); }
  .chips a:hover { background: var(--bg-hover); border-color: var(--c); }
  .chips a :global(svg) { color: var(--c); }
  .more { display: inline-flex; align-items: center; gap: 0.2rem; min-height: 44px; margin-top: auto; font-weight: 700; color: var(--text); text-decoration: underline; text-underline-offset: 3px; }
  a:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
  .chips a:focus-visible { outline-offset: 1px; }

  .steps { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--sp-4); grid-template-columns: minmax(0, 1fr); counter-reset: s; }
  .steps li { display: flex; gap: var(--sp-4); align-items: flex-start; padding: var(--sp-5); background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius-lg); }
  .steps p { margin: var(--sp-1) 0 0; color: var(--text-muted); }
  .n { flex: none; display: grid; place-items: center; width: 2.5rem; height: 2.5rem; border-radius: 50%; background: var(--accent); color: var(--on-accent); font-weight: 800; font-size: var(--fs-lg); }

  .priv { display: flex; gap: var(--sp-5); align-items: center; padding: var(--sp-5); background: linear-gradient(135deg, color-mix(in srgb, var(--accent) 10%, var(--bg-card)), var(--bg-card)); border: 1px solid var(--border-strong); border-radius: var(--radius-xl); }
  .priv h2 { margin-bottom: var(--sp-2); }
  .priv p { margin: 0 0 var(--sp-1); color: var(--text-muted); }

  .two { display: grid; gap: var(--sp-4); grid-template-columns: minmax(0, 1fr); }
  .col { display: flex; flex-direction: column; gap: var(--sp-3); padding: var(--sp-5); background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); }
  .col.accent { border-color: var(--accent); box-shadow: var(--elev-2); }
  .col ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--sp-2); }
  .col li { display: flex; gap: var(--sp-2); align-items: flex-start; }
  .col li :global(svg) { flex: none; margin-top: 0.2rem; color: var(--accent); }
  .col .btn { margin-top: auto; }

  @media (min-width: 640px) {
    .hero-in { flex-direction: row; text-align: left; padding: var(--sp-7) var(--sp-6) var(--sp-6); gap: var(--sp-6); align-items: center; }
    .lead { margin-left: 0; }
    .cta { justify-content: flex-start; }
    .facts { justify-content: flex-start; }
    .areas, .two { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .steps { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  }
  @media (prefers-reduced-motion: reduce) { .area:hover { transform: none; } }
</style>
