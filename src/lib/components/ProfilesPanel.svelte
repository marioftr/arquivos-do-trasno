<script lang="ts">
  import { t } from "../i18n";
  import { savedProfiles, summaryOf, type SavedProfile } from "../settings";
  import Icon from "./Icon.svelte";

  let asking: string | null = null;

  function rename(p: SavedProfile, name: string) {
    const n = name.trim();
    if (!n) return;
    savedProfiles.update((l) => l.map((x) => (x.id === p.id ? { ...x, name: n } : x)));
  }
  function remove(id: string) {
    savedProfiles.update((l) => l.filter((x) => x.id !== id));
    asking = null;
  }
</script>

<section>
  <p class="muted note">
    {$t("Aquí están os perfís de saída que gardaches. Créanse na pestana Comprimir dun vídeo, con «Gardar como perfil…», e aparecen no selector de perfís.")}
  </p>

  {#if !$savedProfiles.length}
    <p class="muted empty">{$t("Aínda non gardaches ningún perfil. O trasno non ten nada que aprender de ti por agora.")}</p>
  {/if}

  {#each $savedProfiles as p (p.id)}
    <div class="card">
      <div class="top">
        <input class="name" value={p.name} on:change={(e) => rename(p, e.currentTarget.value)} aria-label={$t("Nome do perfil")} maxlength="40" />
        {#if asking === p.id}
          <span class="ask">{$t("Borrar?")}</span>
          <button class="btn danger" on:click={() => remove(p.id)}>{$t("Si, borrar")}</button>
          <button class="btn" on:click={() => (asking = null)}>{$t("Seguir")}</button>
        {:else}
          <button class="btn ghost icon" on:click={() => (asking = p.id)} aria-label={$t("Borrar perfil")} title={$t("Borrar perfil")}><Icon name="close" size={14} /></button>
        {/if}
      </div>
      <p class="muted sum">{summaryOf(p.v)}</p>
    </div>
  {/each}
</section>

<style>
  section { display: flex; flex-direction: column; gap: 0.7rem; }
  .note, .empty { margin: 0; font-size: 0.82rem; }
  .card { display: flex; flex-direction: column; gap: 0.25rem; padding: 0.6rem 0.7rem; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); }
  .top { display: flex; align-items: center; gap: 0.5rem; }
  .name { flex: 1; min-width: 0; height: var(--control-h); background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0 0.6rem; font: inherit; font-size: 0.88rem; font-weight: 550; }
  .name:focus { outline: none; border-color: var(--accent); }
  .sum { margin: 0; font-size: 0.78rem; }
  .ask { font-size: 0.82rem; }
  .top :global(.btn) { height: var(--control-h); min-height: var(--control-h); padding-block: 0; }
</style>
