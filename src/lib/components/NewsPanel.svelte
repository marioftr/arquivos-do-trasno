<script lang="ts">
    import { t, lang } from "../i18n";
  import { CHANGELOG, type Kind } from "../changelog";
  import { appVersion, autoCheck, updState, updFound, updError, updProgress, notesFor } from "../updates";
  import Toggle from "./ui/Toggle.svelte";

  export let part: "changelog" | "updates" = "changelog";

  const KIND: Record<Kind, string> = { new: "Novo", better: "Mellorado", fixed: "Arranxado" };
  $: noRelease = /valid release JSON|404|not found/i.test($updError);
  $: notes = $updFound?.notes ? notesFor($updFound.notes, $lang) : [];
  const pick = (c: { gl: string; es: string; en: string }) => c[$lang === "es" ? "es" : $lang === "en" ? "en" : "gl"];
  const dateOf = (d: string) => new Date(d + "T12:00:00").toLocaleDateString($lang === "gl" ? "gl-ES" : $lang);
</script>

{#if part === "changelog"}
  <section>
    {#each CHANGELOG as r (r.version)}
      <article class="rel">
        <header>
          <h3>{$t("Versión")} {r.version}<span class="muted sep">{pick(r.title)}</span></h3>
          <span class="muted date">{dateOf(r.date)}</span>
        </header>
        <ul>
          {#each r.changes as c}
            <li><span class={`tag ${c.kind}`}>{$t(KIND[c.kind])}</span><span class="tx">{pick(c)}</span></li>
          {/each}
        </ul>
      </article>
    {/each}
  </section>
{:else}
  <section>
    <div class="cur">
      <span class="muted">{$t("Versión instalada")}</span>
      <strong>{$appVersion || "…"}</strong>
    </div>

    <Toggle label={$t("Actualizar automaticamente")} bind:checked={$autoCheck} hint={$t("Ao abrir a app busca versións novas e instálaas soas. Apagado, non busca nada.")} />

    {#if $updFound && ($updState === "found" || $updState === "downloading" || $updState === "ready")}
      <div class="box" role="status">
        <h3>{$t("Hai unha versión nova")}: {$updFound.version}</h3>
        {#if notes.length}
          <p class="muted what">{$t("Esta actualización inclúe:")}</p>
          <ul class="notes selectable">{#each notes as n}<li>{n}</li>{/each}</ul>
        {/if}
        {#if $updState === "ready"}
          <p class="muted">{$t("Descargada. Instálase en canto non haxa nada en marcha.")}</p>
        {:else}
          <p class="muted">{$t("Estou descargando a versión nova…")}</p>
          <div class="bar"><div class="fill" style={`width:${Math.round($updProgress * 100)}%`}></div></div>
          <span class="muted pct">{Math.round($updProgress * 100)} %</span>
        {/if}
      </div>
    {:else if $autoCheck}
      <p class="muted" role="status">{$updState === "checking" ? $t("Buscando…") : $updState === "none" ? $t("Tes a última versión.") : $updState === "error" ? (noRelease ? $t("Aínda non hai ningunha versión publicada en GitHub.") : $t("Non puiden comprobar as actualizacións. Revisa a conexión a internet e proba de novo.")) : ""}</p>
    {:else}
      <p class="muted">{$t("Co axuste apagado non busco versións novas. Podes baixalas desde GitHub.")}</p>
    {/if}
    <p class="muted note">{$t("As versións publícanse en GitHub. Só descargo ficheiros asinados coa clave do seu autor.")}</p>
  </section>
{/if}

<style>
  section { display: flex; flex-direction: column; gap: 1rem; }
  .rel { display: flex; flex-direction: column; gap: 0.5rem; }
  header { display: flex; align-items: baseline; justify-content: space-between; gap: 0.6rem; border-bottom: 1px solid var(--border); padding-bottom: 0.4rem; }
  h3 { margin: 0; font-size: var(--fs-lg); font-weight: 600; }
  .sep::before { content: "·"; margin: 0 0.55em; }
  .date { font-size: var(--fs-sm); }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.45rem; }
  li { display: flex; gap: 0.6rem; align-items: baseline; font-size: var(--fs-md); line-height: 1.4; }
  .tag { flex: 0 0 5.4rem; text-align: left; font-size: var(--fs-xs); font-weight: 500; letter-spacing: 0.03em; text-transform: uppercase; color: var(--text-faint); }
  .tag.fixed { color: var(--accent-pdf); }
  .tx { flex: 1; min-width: 0; }
  .cur { display: flex; align-items: baseline; gap: 0.6rem; font-size: var(--fs-md); }
  .box { display: flex; flex-direction: column; gap: 0.7rem; padding: 0.8rem 0.9rem; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--bg-card); }
  .box p { margin: 0; }
  .what { font-size: var(--fs-sm); }
  .notes { list-style: disc; padding-left: 1.2rem; display: block; font-size: var(--fs-md); max-height: 12rem; overflow-y: auto; overflow-x: hidden; }
  .notes li { display: list-item; margin-bottom: 0.3rem; }
  .bar { height: 7px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); transition: width 0.3s ease; }
  .pct { font-size: var(--fs-sm); text-align: right; }
  .note { margin: 0; font-size: var(--fs-sm); }
</style>
