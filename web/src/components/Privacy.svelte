<script lang="ts">
  import { t } from "@app/lib/i18n";
  import Icon from "@app/lib/components/Icon.svelte";

  $: secs = [
    {
      id: "files",
      icon: "lock",
      title: $t("Os teus ficheiros"),
      ps: [$t("Os ficheiros que elixes procésanse no teu aparello, dentro do navegador. Non se envían a ningún servidor."), $t("Non hai contas, publicidade nin rastrexadores.")],
    },
    {
      id: "engines",
      icon: "download",
      title: $t("Motores e modelos"),
      ps: [$t("Algunhas ferramentas descargan o seu motor ou modelo: o de audio e vídeo vén de jsDelivr e os de IA de Hugging Face. Só se descarga ese programa, nunca os teus ficheiros. Queda gardado no navegador e bórrase ao limpar os datos do sitio.")],
    },
    {
      id: "servers",
      icon: "eye",
      title: $t("Servizos externos"),
      ps: [$t("A páxina de descarga pregunta a GitHub cal é a última versión da app. Só se fai ao abrila, e GitHub pode ver o teu enderezo IP."), $t("A web está en Vercel, que pode rexistrar o enderezo IP de quen a visita.")],
    },
    {
      id: "browser",
      icon: "gear",
      title: $t("No teu navegador"),
      ps: [$t("O navegador garda no aparello o tema, o idioma e os últimos axustes de cada ferramenta.")],
    },
  ];
</script>

<div class="priv">
  {#each secs as s (s.id)}
    <section class="sec" aria-labelledby={`p-${s.id}`}>
      <header>
        <span class="ic"><Icon name={s.icon} size={26} /></span>
        <h2 id={`p-${s.id}`}>{s.title}</h2>
      </header>
      <div class="tx">
        {#each s.ps as p}<p>{p}</p>{/each}
      </div>
    </section>
  {/each}
</div>

<style>
  .priv { display: flex; flex-direction: column; }
  .sec { display: grid; gap: var(--sp-3); padding: var(--sp-5) 0; border-top: 1px solid var(--border); }
  .sec:first-child { border-top: 0; padding-top: 0; }
  header { display: flex; align-items: center; gap: var(--sp-3); }
  .ic { flex: none; display: grid; place-items: center; width: 3rem; height: 3rem; border-radius: var(--radius-md); background: color-mix(in srgb, var(--accent) 16%, transparent); color: var(--accent); }
  h2 { margin: 0; font-size: var(--fs-h3); line-height: 1.25; }
  .tx { display: flex; flex-direction: column; gap: var(--sp-3); max-width: 38rem; }
  .tx p { margin: 0; line-height: 1.6; }
  @media (min-width: 720px) {
    .sec { grid-template-columns: 15rem minmax(0, 1fr); gap: var(--sp-6); padding: var(--sp-6) 0; }
    .sec:first-child { padding-top: var(--sp-2); }
    header { align-self: start; }
  }
</style>
