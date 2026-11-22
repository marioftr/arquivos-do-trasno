<script lang="ts">
  import { t } from "@app/lib/i18n";
</script>

<article>
  <p>{$t("Os ficheiros que elixes procésanse no teu aparello, dentro do navegador. Non se envían a ningún servidor.")}</p>
  <p>{$t("Non hai contas, publicidade nin rastrexadores.")}</p>
  <p>{$t("Algunhas ferramentas descargan o seu motor ou modelo: o de audio e vídeo vén de jsDelivr e os de IA de Hugging Face. Só se descarga ese programa, nunca os teus ficheiros. Queda gardado no navegador e bórrase ao limpar os datos do sitio.")}</p>
  <p>{$t("O navegador garda no aparello o tema, o idioma e os últimos axustes de cada ferramenta.")}</p>
  <p>{$t("A web aloxase en Vercel, que, coma calquera servidor, pode rexistrar o enderezo IP das visitas.")}</p>
</article>

<style>
  article { max-width: 40rem; display: flex; flex-direction: column; gap: 0.9rem; }
  p { margin: 0; }
</style>
