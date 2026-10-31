<script lang="ts">
  import Icon from "./Icon.svelte";
  import { files, activePath, checked, toggleChecked, setChecked, removeFile, moveFile, clearAll } from "../files";
  import { formatBytes } from "../formats";
  import { t } from "../i18n";
  import type { Category } from "../engine";

  export let onAdd: () => void;

  const ICONS: Record<Category, string> = {
    imaxe: "image",
    audio: "audio",
    video: "video",
    documento: "document",
    modelo3d: "cube",
    descoñecido: "document",
  };

  $: allChecked = $files.length > 0 && $checked.size === $files.length;
</script>

<div class="list">
  <div class="head">
    <span class="titulo">{$t("Ficheiros")} <span class="muted">{$files.length}</span></span>
    <button class="btn" on:click={onAdd}><Icon name="plus" size={14} /> {$t("Engadir")}</button>
  </div>

  <ul class="scrollbar-thin">
    {#each $files as f, i (f.path)}
      <li class:active={$activePath === f.path}>
        <input
          type="checkbox"
          checked={$checked.has(f.path)}
          on:change={() => toggleChecked(f.path)}
          aria-label={$t("Marcar {name}", { name: f.name })}
          title={$t("Marcar para traballar con varios ficheiros á vez")}
        />
        <button class="row" on:click={() => activePath.set(f.path)} title={f.path}>
          <Icon name={ICONS[f.category]} size={16} />
          <span class="info">
            <span class="nome">{f.name}</span>
            <span class="muted meta">
              {f.ext.toUpperCase() || "?"}{#if f.size !== undefined} · {formatBytes(f.size)}{/if}{#if f.pages} · {$t("{n} páx.", { n: f.pages })}{/if}
            </span>
          </span>
        </button>
        <span class="acc">
          <button class="mini" on:click={() => moveFile(f.path, -1)} disabled={i === 0} aria-label={$t("Subir")} title={$t("Subir")}><Icon name="up" size={13} /></button>
          <button class="mini" on:click={() => moveFile(f.path, 1)} disabled={i === $files.length - 1} aria-label={$t("Baixar")} title={$t("Baixar")}><Icon name="down" size={13} /></button>
          <button class="mini" on:click={() => removeFile(f.path)} aria-label={$t("Quitar")} title={$t("Quitar da lista")}><Icon name="close" size={13} /></button>
        </span>
      </li>
    {/each}
  </ul>

  <div class="foot">
    <button class="link" on:click={() => setChecked(allChecked ? [] : $files.map((f) => f.path))}>
      {allChecked ? $t("Desmarcar todo") : $t("Marcar todo")}
    </button>
    <button class="link" on:click={clearAll}>{$t("Baleirar")}</button>
  </div>
</div>

<style>
  .list { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .head { display: flex; align-items: center; justify-content: space-between; padding: 0.7rem 0.9rem; border-bottom: 1px solid var(--border); }
  .titulo { font-weight: 600; font-size: 0.88rem; }
  ul { list-style: none; margin: 0; padding: 0.3rem 0; flex: 1; min-height: 0; overflow-y: auto; }
  li { display: flex; align-items: center; gap: 0.4rem; padding: 0 0.5rem 0 0.8rem; border-left: 2px solid transparent; }
  li:hover { background: var(--bg-hover); }
  li.active { background: var(--bg-hover); border-left-color: var(--accent-conversor); }
  .row { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.6rem; background: none; border: none; color: var(--text-muted); padding: 0.55rem 0; text-align: left; }
  li.active .row { color: var(--text); }
  .info { display: flex; flex-direction: column; min-width: 0; }
  .nome { color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.86rem; }
  .meta { font-size: 0.74rem; }
  .acc { display: none; align-items: center; }
  li:hover .acc { display: flex; }
  .mini { background: none; border: none; color: var(--text-faint); padding: 0.2rem; border-radius: var(--radius-sm); display: flex; }
  .mini:hover:not(:disabled) { color: var(--text); background: var(--bg-card); }
  .mini:disabled { opacity: 0.25; cursor: default; }
  .foot { display: flex; justify-content: space-between; padding: 0.55rem 0.9rem; border-top: 1px solid var(--border); font-size: 0.8rem; }
  .link { background: none; border: none; color: var(--text-muted); padding: 0; font: inherit; }
  .link:hover { color: var(--text); }
  input[type="checkbox"] { flex-shrink: 0; }
</style>
