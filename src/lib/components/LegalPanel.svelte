<script lang="ts">
  import { t, lang } from "../i18n";
  import { DOCS, LINKS, type DocId } from "../legal";
  import { openUrl } from "../engine";
  import { toast } from "../toasts";
  import { tr } from "../i18n";
  import Icon from "./Icon.svelte";

  export let doc: DocId;

  $: d = DOCS[doc];
  $: k = ($lang === "es" ? "es" : $lang === "en" ? "en" : "gl") as "gl" | "es" | "en";
  const go = (url: string) => openUrl(url).catch(() => toast(tr("A ligazón non se deixou abrir. O trasno ensaiou, pero nada."), { kind: "warn" }));
  const dateOf = (s: string) => new Date(s + "T12:00:00").toLocaleDateString($lang === "gl" ? "gl-ES" : $lang);
</script>

<article class="doc selectable">
  <header>
    <h3>{d.title[k]}</h3>
    <span class="muted when">{$t("Última actualización")}: {dateOf(d.updated)}</span>
  </header>
  {#each d.sections as s}
    <section>
      <h4>{s.h[k]}</h4>
      {#each s.p as para}<p class:mono={doc === "license"}>{para[k]}</p>{/each}
    </section>
  {/each}
  <footer>
    <button class="btn" on:click={() => go(LINKS.issues)}><Icon name="open" size={14} /> {$t("Contacto: incidencias no repositorio")}</button>
    {#if doc === "license"}<button class="btn" on:click={() => go(LINKS.license)}><Icon name="open" size={14} /> {$t("Ver LICENSE no repositorio")}</button>{/if}
  </footer>
</article>

<style>
  /* O contedor é unha columna con scroll: sen «flex: none» o texto encollería e o último botón quedaría cortado. */
  .doc { display: flex; flex-direction: column; gap: 1rem; flex: none; padding-bottom: 0.6rem; }
  header { display: flex; flex-direction: column; gap: 0.15rem; border-bottom: 1px solid var(--border); padding-bottom: 0.6rem; }
  h3 { margin: 0; font-size: var(--fs-lg); font-weight: 600; }
  .when { font-size: var(--fs-sm); }
  section { display: flex; flex-direction: column; gap: 0.45rem; }
  h4 { margin: 0; font-size: var(--fs-md); font-weight: 650; }
  p { margin: 0; font-size: var(--fs-md); line-height: 1.5; color: var(--text); }
  p.mono { font-size: var(--fs-sm); color: var(--text-muted); }
  footer { display: flex; flex-wrap: wrap; gap: 0.5rem; padding-top: 0.4rem; }
</style>
