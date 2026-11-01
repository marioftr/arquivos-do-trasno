<script lang="ts">
  import { onMount } from "svelte";
  import { t, lang } from "../i18n";
  import { CHANGELOG, type Kind } from "../changelog";
  import { appVersion, updState, updFound, updError, updProgress, checkForUpdate, installUpdate, explorerInstalled } from "../updates";
  import Toggle from "./ui/Toggle.svelte";
  import Icon from "./Icon.svelte";

  export let part: "changelog" | "updates" = "changelog";

  const KIND: Record<Kind, string> = { new: "Novo", better: "Mellorado", fixed: "Arranxado" };
  let refreshMenu = true;
  let hasMenu = false;
  onMount(async () => {
    hasMenu = await explorerInstalled();
  });
  const pick = (c: { gl: string; es: string; en: string }) => c[$lang === "es" ? "es" : $lang === "en" ? "en" : "gl"];
  const dateOf = (d: string) => new Date(d + "T12:00:00").toLocaleDateString($lang === "gl" ? "gl-ES" : $lang);
</script>

{#if part === "changelog"}
  <section>
    {#each CHANGELOG as r (r.version)}
      <article class="rel">
        <header>
          <h3>{$t("Versión")} {r.version}<span class="muted"> · {pick(r.title)}</span></h3>
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

    {#if $updState === "found" && $updFound}
      <div class="box">
        <h3>{$t("Hai unha versión nova")}: {$updFound.version}</h3>
        {#if $updFound.notes}<p class="notes selectable">{$updFound.notes}</p>{/if}
        {#if hasMenu}
          <Toggle label={$t("Actualizar tamén as entradas do menú do clic dereito")} bind:checked={refreshMenu} hint={$t("As entradas do Explorador apuntan ao programa instalado. Se está marcado, o trasno volve crealas coa versión nova ao reiniciar, para que sigan funcionando e leven as novidades do menú.")} />
        {/if}
        <button class="btn primary" on:click={() => installUpdate(hasMenu && refreshMenu)}><Icon name="open" size={14} /> {$t("Instalar e reiniciar")}</button>
      </div>
    {:else if $updState === "downloading"}
      <div class="box" role="status">
        <p class="muted">{$t("O trasno está descargando a versión nova…")}</p>
        <div class="bar"><div class="fill" style={`width:${Math.round($updProgress * 100)}%`}></div></div>
        <span class="muted pct">{Math.round($updProgress * 100)} %</span>
      </div>
    {:else}
      <div class="row">
        <button class="btn" on:click={() => checkForUpdate(false)} disabled={$updState === "checking"}><Icon name="reset" size={14} /> {$updState === "checking" ? $t("Buscando…") : $t("Buscar actualizacións")}</button>
        {#if $updState === "none"}<span class="muted">{$t("Tes a última versión. O trasno non ten nada pendente.")}</span>{/if}
      </div>
      {#if $updState === "error"}
        <p class="warn selectable">{$t("O trasno non puido comprobar as actualizacións. Isto é normal na copia de desenvolvemento ou sen internet.")} <span class="muted">{$updError}</span></p>
      {/if}
    {/if}
    <p class="muted note">{$t("As versións publícanse en GitHub. O trasno só descarga ficheiros asinados coa clave do seu autor.")}</p>
  </section>
{/if}

<style>
  section { display: flex; flex-direction: column; gap: 1rem; }
  .rel { display: flex; flex-direction: column; gap: 0.5rem; }
  header { display: flex; align-items: baseline; justify-content: space-between; gap: 0.6rem; border-bottom: 1px solid var(--border); padding-bottom: 0.4rem; }
  h3 { margin: 0; font-size: 0.95rem; font-weight: 600; }
  .date { font-size: 0.78rem; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.45rem; }
  li { display: flex; gap: 0.6rem; align-items: baseline; font-size: 0.86rem; line-height: 1.4; }
  .tag { flex: 0 0 5.2rem; text-align: center; font-size: 0.68rem; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; padding: 0.1rem 0.3rem; border-radius: 4px; }
  .tag.new { background: color-mix(in srgb, var(--accent) 22%, transparent); color: var(--accent); }
  .tag.better { background: color-mix(in srgb, var(--accent-conversor) 22%, transparent); color: var(--accent-conversor); }
  .tag.fixed { background: color-mix(in srgb, var(--accent-pdf) 22%, transparent); color: var(--accent-pdf); }
  .tx { flex: 1; min-width: 0; }
  .cur { display: flex; align-items: baseline; gap: 0.6rem; font-size: 0.9rem; }
  .box { display: flex; flex-direction: column; gap: 0.7rem; padding: 0.8rem 0.9rem; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--bg-card); }
  .box p { margin: 0; }
  .notes { white-space: pre-wrap; font-size: 0.85rem; max-height: 10rem; overflow: auto; }
  .row { display: flex; align-items: center; gap: 0.7rem; flex-wrap: wrap; }
  .bar { height: 7px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); transition: width 0.3s ease; }
  .pct { font-size: 0.78rem; text-align: right; }
  .note { margin: 0; font-size: 0.78rem; }
  .warn { margin: 0; font-size: 0.82rem; color: var(--accent-pdf); }
</style>
