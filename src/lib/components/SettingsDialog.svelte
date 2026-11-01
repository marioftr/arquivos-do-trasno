<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { t, lang, LANGS, type Lang } from "../i18n";
  import { theme, outputMode, outputFolder, dialog, settingsTab, type Theme, type OutputMode } from "../settings";
  import { fileNameOf } from "../formats";
  import Modal from "./ui/Modal.svelte";
  import Segmented from "./ui/Segmented.svelte";
  import Icon from "./Icon.svelte";
  import ToolsPanel from "./ToolsPanel.svelte";
  import CreditsPanel from "./CreditsPanel.svelte";

  let tab: string = $settingsTab;
  $: settingsTab.set(tab as typeof $settingsTab);

  // Segmented usa texto; as tendas teñen tipos máis estritos.
  let langV: string = $lang;
  let themeV: string = $theme;
  let modeV: string = $outputMode;
  $: lang.set(langV as Lang);
  $: theme.set(themeV as Theme);
  $: outputMode.set(modeV as OutputMode);

  async function chooseFolder() {
    const c = await open({ directory: true });
    if (typeof c === "string") outputFolder.set(c);
  }

  // Cada atallo: teclas e descrición.
  const groups: { title: string; keys: { k: string; d: string }[] }[] = [
    {
      title: "Xeral",
      keys: [
        { k: "Ctrl + O", d: "Abrir ficheiros" },
        { k: "Supr", d: "Quitar da lista o ficheiro que se está vendo" },
        { k: "Ctrl + Z / Ctrl + Y", d: "Desfacer e refacer os axustes da ferramenta" },
      ],
    },
    {
      title: "Audio e vídeo",
      keys: [
        { k: "Espazo · K", d: "Reproducir e pausar" },
        { k: "← →", d: "Saltar 5 segundos (con Maiús, 1 segundo)" },
        { k: "J · L", d: "Saltar 10 segundos" },
        { k: "↑ ↓", d: "Subir e baixar o volume" },
        { k: "M", d: "Silenciar" },
        { k: "F", d: "Pantalla completa (vídeo)" },
        { k: "Inicio · Fin", d: "Ir ao principio ou ao final" },
        { k: ", · .", d: "Un fotograma cara atrás ou adiante (vídeo)" },
      ],
    },
    {
      title: "Imaxe",
      keys: [
        { k: "+ · −", d: "Ampliar e reducir" },
        { k: "0 · 1", d: "Axustar á xanela · tamaño real" },
      ],
    },
  ];
</script>

<Modal title={$t("Axustes")} onClose={() => dialog.set(null)}>
  <div class="split">
  <div class="side" role="tablist" aria-orientation="vertical">
    {#each [{ id: "general", label: $t("Xeral"), icon: "gear" }, { id: "shortcuts", label: $t("Atallos de teclado"), icon: "keyboard" }, { id: "tools", label: $t("Extensións"), icon: "puzzle" }, { id: "credits", label: $t("Créditos e licenzas"), icon: "info" }] as it}
      <button role="tab" aria-selected={tab === it.id} class:on={tab === it.id} on:click={() => (tab = it.id)}><Icon name={it.icon} size={16} /><span>{it.label}</span></button>
    {/each}
  </div>
  <div class="pane scrollbar-thin">
  {#if tab === "general"}
  <section>
    <h3>{$t("Aparencia")}</h3>
    <Segmented
      label={$t("Idioma")}
      options={LANGS.map((l) => ({ value: l.id, label: l.label }))}
      bind:value={langV}
    />
    <Segmented
      label={$t("Tema")}
      options={[
        { value: "system", label: $t("Sistema") },
        { value: "light", label: $t("Claro") },
        { value: "dark", label: $t("Escuro") },
      ]}
      bind:value={themeV}
    />
  </section>

  <section>
    <h3>{$t("Onde se gardan os resultados")}</h3>
    <Segmented
      options={[
        { value: "ask", label: $t("Preguntar cada vez") },
        { value: "beside", label: $t("Xunto ao orixinal") },
        { value: "folder", label: $t("Cartafol fixo") },
      ]}
      bind:value={modeV}
    />
    {#if modeV === "folder"}
      <button class="btn" on:click={chooseFolder}><Icon name="folder" size={14} /> {$outputFolder ? fileNameOf($outputFolder) : $t("Escoller cartafol…")}</button>
    {/if}
    <p class="muted note">
      {modeV === "ask"
        ? $t("Cada vez que exportes, o trasno pregunta onde gardar.")
        : $t("O trasno garda sen preguntar e nunca pisa un ficheiro que xa exista: engade un número ao nome. Os orixinais non se tocan.")}
    </p>
  </section>

  {:else if tab === "shortcuts"}
  <section>
    {#each groups as g}
      <div class="grp">
        <h4>{$t(g.title)}</h4>
        {#each g.keys as r}
          <div class="row"><kbd>{r.k}</kbd><span>{$t(r.d)}</span></div>
        {/each}
      </div>
    {/each}
  </section>
  {:else if tab === "tools"}
    <ToolsPanel />
  {:else}
    <CreditsPanel />
  {/if}
  </div>
  </div>
</Modal>

<style>
  .split { display: flex; gap: 1rem; min-height: 0; flex: 1; }
  .side { flex: 0 0 11.5rem; display: flex; flex-direction: column; gap: 0.15rem; }
  .side button { display: flex; align-items: center; gap: 0.6rem; text-align: left; background: none; border: none; color: var(--text-muted); padding: 0.55rem 0.7rem; border-radius: var(--radius-sm); font-size: 0.88rem; font-weight: 500; }
  .side button:hover { background: var(--bg-hover); color: var(--text); }
  .side button.on { background: color-mix(in srgb, var(--accent) 18%, transparent); color: var(--text); font-weight: 600; }
  .side button.on :global(svg) { color: var(--accent); }
  .pane { flex: 1; min-width: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; padding-right: 0.2rem; }
  @media (max-width: 640px) { .split { flex-direction: column; } .side { flex-direction: row; flex-wrap: wrap; flex-basis: auto; } }
  section { display: flex; flex-direction: column; gap: 0.6rem; }
  h3 { margin: 0; font-size: 0.86rem; font-weight: 600; }
  h4 { margin: 0.3rem 0 0.1rem; font-size: 0.78rem; font-weight: 600; color: var(--text-muted); text-transform: none; }
  .note { margin: 0; font-size: 0.8rem; }
  .grp { display: flex; flex-direction: column; }
  .row { display: flex; align-items: center; gap: 0.8rem; padding: 0.25rem 0; border-top: 1px solid var(--border); font-size: 0.85rem; }
  kbd { flex: 0 0 11rem; font-family: inherit; font-size: 0.78rem; background: var(--bg-input); border: 1px solid var(--border-strong); border-radius: 5px; padding: 1px 7px; width: fit-content; }
</style>
