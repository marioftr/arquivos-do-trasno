<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { t, lang, LANGS, type Lang } from "../i18n";
  import { theme, outputMode, outputFolder, openHub, type Theme, type OutputMode } from "../settings";
  import { installExplorerMenu } from "../explorerMenu";
  import { fileNameOf } from "../formats";
  import * as engine from "../engine";
  import { errText } from "../errors";
  import Segmented from "./ui/Segmented.svelte";
  import Toggle from "./ui/Toggle.svelte";
  import InfoTip from "./ui/InfoTip.svelte";
  import { tipsOn, trasnoClick, type ClickAction } from "../agent";
  import Icon from "./Icon.svelte";

  // Segmented usa texto; as tendas teñen tipos máis estritos.
  let langV: string = $lang;
  let themeV: string = $theme;
  let modeV: string = $outputMode;
  $: lang.set(langV as Lang);
  $: theme.set(themeV as Theme);
  $: outputMode.set(modeV as OutputMode);
  let clickV: string = $trasnoClick;
  $: trasnoClick.set(clickV as ClickAction);

  // Menú de clic dereito do Explorador (só Windows).
  let ex: engine.ExplorerStatus | null = null;
  let exMsg = "";
  let exBusy = false;
  const refreshEx = () => engine.explorerStatus().then((s) => (ex = s)).catch(() => (ex = null));
  refreshEx();
  async function exInstall() {
    exBusy = true;
    exMsg = "";
    try {
      await installExplorerMenu();
      exMsg = $t("Feito. O menú do clic dereito xa ten as entradas do trasno.");
    } catch (e) {
      exMsg = errText(e);
    } finally {
      exBusy = false;
      refreshEx();
    }
  }
  async function exRemove() {
    exBusy = true;
    exMsg = "";
    try {
      await engine.explorerUninstall();
      exMsg = $t("Quitadas as entradas do menú do clic dereito.");
    } catch (e) {
      exMsg = errText(e);
    } finally {
      exBusy = false;
      refreshEx();
    }
  }

  async function chooseFolder() {
    const c = await open({ directory: true });
    if (typeof c === "string") outputFolder.set(c);
  }

</script>

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
    <h3>{$t("Trasno")}</h3>
    <Toggle label={$t("Consellos do trasno")} bind:checked={$tipsOn} hint={$t("O trasno comenta de cando en vez cousas útiles sobre o ficheiro e a ferramenta abertos. Desactivalo silencia eses comentarios, pero non os avisos importantes.")} />
    <Segmented
      label={$t("Ao premer o trasno")}
      info={$t("Que ocorre ao premer directamente o trasno da barra inferior.")}
      options={[
        { value: "procs", label: $t("Abrir os procesos") },
        { value: "palette", label: $t("Abrir o buscador") },
        { value: "say", label: $t("Soltar un comentario") },
      ]}
      bind:value={clickV}
    />
  </section>

  <section>
    <h3>{$t("Onde se gardan os resultados")}<InfoTip text={modeV === "ask" ? $t("Cada exportación pregunta onde gardar.") : $t("Gárdase sen preguntar e nunca se pisa un ficheiro existente: engádese un número ao nome. Os orixinais non se tocan.")} /></h3>
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
  </section>

  {#if ex?.available}
  <section>
    <h3>{$t("Menú do clic dereito (Explorador)")}<InfoTip text={$t("Engade «Converter a…» e «Comprimir» ao clic dereito sobre un ficheiro, sen abrir a aplicación completa. Non fai falta ser administrador e pódese quitar en calquera momento. En Windows 11 aparece en «Mostrar máis opcións»; se a aplicación cambia de cartafol, hai que instalalo de novo.")} /></h3>
    <div class="exrow">
      <button class="btn primary" on:click={exInstall} disabled={exBusy}>{ex.installed ? $t("Actualizar entradas") : $t("Instalar no menú")}</button>
      {#if ex.installed}<button class="btn" on:click={exRemove} disabled={exBusy}>{$t("Quitar do menú")}</button>{/if}
      <span class="muted note">{ex.installed ? (ex.current ? $t("Instalado") : $t("Instalado, pero apunta a outra copia da app")) : $t("Non instalado")}</span>
    </div>
    {#if exMsg}<p class="note selectable">{exMsg}</p>{/if}
  </section>
  {/if}
  

<section>
  <h3>{$t("Actualizacións")}<InfoTip text={$t("A busca de versións novas e o rexistro de cambios están en Novidades.")} /></h3>
  <div class="exrow"><button class="btn" on:click={() => openHub("news.updates")}><Icon name="open" size={14} /> {$t("Ir a Actualizacións")}</button></div>
</section>

<style>
  section { display: flex; flex-direction: column; gap: 0.6rem; }
  h3 { margin: 0; font-size: 0.86rem; font-weight: 600; }
  .note { margin: 0; font-size: 0.8rem; }
  .exrow { display: flex; flex-wrap: wrap; align-items: center; gap: 0.6rem; }
</style>
