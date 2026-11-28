<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { t, lang, LANGS, type Lang } from "../i18n";
  import { theme, outputMode, outputFolder, openHub, openTutorial, motionReduced, motionPref, uiScale, UI_SCALES, type Theme, type OutputMode } from "../settings";
  import { installExplorerMenu } from "../explorerMenu";
  import { autoCheck } from "../updates";
  import { fileNameOf } from "../formats";
  import * as engine from "../engine";
  import { errText } from "../errors";
  import Segmented from "./ui/Segmented.svelte";
  import ThemePicker from "./ui/ThemePicker.svelte";
  import Toggle from "./ui/Toggle.svelte";
  import InfoTip from "./ui/InfoTip.svelte";
  import IconSelect from "./ui/IconSelect.svelte";
  import { tipsOn, trasnoClick, CLICK_ITEMS, type ClickAction } from "../agent";
  import Icon from "./Icon.svelte";
  let motionV = $motionReduced;
  $: if (motionV !== $motionReduced) motionPref.set(motionV ? "reduce" : "full");

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
      exMsg = $t("Feito. O menú do clic dereito xa ten as entradas de Arquivos do Trasno.");
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
    <h3>{$t("Trasno")}</h3>
    <Toggle label={$t("Consellos do trasno")} bind:checked={$tipsOn} hint={$t("Amosa de cando en vez consellos útiles sobre o ficheiro e a ferramenta abertos. Os avisos importantes sempre saen.")} />
    <div class="row">
      <span class="lab">{$t("Ao premer o trasno")}<InfoTip text={$t("Que ocorre ao premer directamente o trasno da barra inferior.")} /></span>
      <IconSelect label={$t("Ao premer o trasno")} options={CLICK_ITEMS.map((c) => ({ value: c.id, label: $t(c.action), icon: c.icon }))} bind:value={clickV} />
    </div>
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

  <section>
    <h3>{$t("Accesibilidade")}<InfoTip text={$t("Amplía o texto e os controis por igual, como o zoom do navegador. A xanela non baixa do tamaño no que nada se solapa.")} /></h3>
    <Toggle label={$t("Reducir animacións")} bind:checked={motionV} hint={$t("Sen saltos, voos nin transicións. Por defecto segue o axuste do sistema.")} />
    <Segmented label={$t("Tamaño da interface")} options={UI_SCALES.map((v) => ({ value: v, label: `${v} %` }))} bind:value={$uiScale} />
  </section>

  <section>
    <h3>{$t("Aparencia")}</h3>
    <Segmented
      label={$t("Idioma")}
      options={LANGS.map((l) => ({ value: l.id, label: l.label }))}
      bind:value={langV}
    />
    <div class="lab stk">{$t("Tema")}</div>
    <ThemePicker bind:value={themeV} />
  </section>

  {#if ex?.available}
  <section>
    <h3>{$t("Menú do clic dereito (Explorador)")}<InfoTip text={$t("Engade «Converter a…» e «Comprimir» ao clic dereito. Sen administrador. En Windows 11 sae en «Mostrar máis opcións». Ponse ao día soo.")} /></h3>
    <div class="exrow">
      {#if ex.installed}
        <button class="btn" on:click={exRemove} disabled={exBusy}>{$t("Quitar do menú")}</button>
      {:else}
        <button class="btn primary" on:click={exInstall} disabled={exBusy}>{$t("Instalar no menú")}</button>
      {/if}
      <span class="muted note">{ex.installed ? (ex.current ? $t("Instalado") : $t("Instalado, pero apunta a outra copia da app")) : $t("Non instalado")}</span>
    </div>
    {#if exMsg}<p class="note selectable">{exMsg}</p>{/if}
  </section>
  {/if}
  

<section>
  <h3>{$t("Titorial")}</h3>
  <div class="exrow"><button class="btn" on:click={() => openTutorial()}><Icon name="play" size={14} /> {$t("Ver o titorial")}</button></div>
</section>

<section>
  <h3>{$t("Actualizacións")}<InfoTip text={$t("As novidades de cada versión e o estado da actualización están en Novidades.")} /></h3>
  <Toggle label={$t("Actualizar automaticamente")} bind:checked={$autoCheck} hint={$t("Ao abrir a app busca versións novas e instálaas soas. Apagado, non busca nada.")} />
  <div class="exrow"><button class="btn" on:click={() => openHub("news.updates")}><Icon name="open" size={14} /> {$t("Ver o estado")}</button></div>
</section>

<style>
  .hint { margin: 0.3rem 0 0; font-size: var(--fs-sm); line-height: 1.4; }
  section { display: flex; flex-direction: column; gap: 0.6rem; }
  h3 { margin: 0; font-size: var(--fs-md); font-weight: var(--fw-strong); }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 0.8rem; }
  .lab { font-size: var(--fs-md); display: inline-flex; align-items: center; }
  /* Etiqueta dun control que ocupa a liña enteira (igual que a dos selectores segmentados): máis pequena e apagada. */
  .lab.stk { font-size: var(--fs-sm); color: var(--text-muted); margin-bottom: -0.4rem; }
  .note { margin: 0; font-size: var(--fs-sm); }
  .exrow { display: flex; flex-wrap: wrap; align-items: center; gap: 0.6rem; }
</style>
