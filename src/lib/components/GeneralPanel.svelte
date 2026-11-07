<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { t, lang, LANGS, type Lang } from "../i18n";
  import { theme, outputMode, outputFolder, openHub, motionReduced, motionPref, uiScale, UI_SCALES, type Theme, type OutputMode } from "../settings";
  import { installExplorerMenu } from "../explorerMenu";
  import { fileNameOf } from "../formats";
  import * as engine from "../engine";
  import { errText } from "../errors";
  import Segmented from "./ui/Segmented.svelte";
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
    <h3>{$t("Trasno")}</h3>
    <Toggle label={$t("Consellos do trasno")} bind:checked={$tipsOn} hint={$t("O trasno comenta de cando en vez cousas útiles sobre o ficheiro e a ferramenta abertos. Desactivalo silencia eses comentarios, pero non os avisos importantes.")} />
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
    <h3>{$t("Accesibilidade")}<InfoTip text={$t("O tamaño da interface amplía o texto e os controis por igual, como o zoom do navegador. A xanela non pode ser máis pequena que o necesario para que nada se solape.")} /></h3>
    <Toggle label={$t("Reducir animacións")} bind:checked={motionV} hint={$t("O trasno e a interface quedan quietos: sen saltos, voos nin transicións. Por defecto segue o axuste de animacións do sistema.")} />
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
    <div class="themes" role="radiogroup" aria-label={$t("Tema")}>
      {#each [{ id: "system", label: $t("Sistema") }, { id: "light", label: $t("Claro") }, { id: "dark", label: $t("Escuro") }] as th}
        <button class="theme" class:on={themeV === th.id} role="radio" aria-checked={themeV === th.id} on:click={() => (themeV = th.id)}>
          <span class={`mini ${th.id}`} aria-hidden="true">
            {#if th.id === "system"}
              <span class="half l"><i class="sb"></i><i class="hd"></i><i class="bd"></i><i class="ac"></i></span>
              <span class="half d"><i class="sb"></i><i class="hd"></i><i class="bd"></i><i class="ac"></i></span>
            {:else}
              <i class="sb"></i><i class="hd"></i><i class="bd"></i><i class="ac"></i>
            {/if}
          </span>
          <span class="tl">{th.label}</span>
        </button>
      {/each}
    </div>
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
  .hint { margin: 0.3rem 0 0; font-size: var(--fs-sm); line-height: 1.4; }
  section { display: flex; flex-direction: column; gap: 0.6rem; }
  h3 { margin: 0; font-size: var(--fs-md); font-weight: 600; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 0.8rem; }
  .lab { font-size: var(--fs-md); display: inline-flex; align-items: center; }
  /* Etiqueta dun control que ocupa a liña enteira (igual que a dos selectores segmentados): máis pequena e apagada. */
  .lab.stk { font-size: var(--fs-sm); color: var(--text-muted); margin-bottom: -0.4rem; }
  .themes { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.6rem; }
  .theme { display: flex; flex-direction: column; gap: 0.45rem; padding: 0.4rem 0.4rem 0.5rem; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-md); color: var(--text-muted); font-size: var(--fs-sm); }
  .theme:hover { border-color: var(--text-faint); color: var(--text); }
  .theme.on { border-color: var(--accent); color: var(--text); box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 25%, transparent); }
  .mini { position: relative; display: block; height: 58px; border-radius: 6px; overflow: hidden; border: 1px solid rgba(128, 128, 128, 0.35); }
  .mini.light, .half.l { --m-bg: #f4f5f7; --m-side: #ffffff; --m-head: #e4e6ea; --m-body: #dcdfe4; }
  .mini.dark, .half.d { --m-bg: #14161a; --m-side: #1f2126; --m-head: #272a30; --m-body: #2f333a; }
  .mini.light, .mini.dark { background: var(--m-bg); }
  .mini.system { display: flex; }
  .half { position: relative; display: block; flex: 1; background: var(--m-bg); }
  .mini i { position: absolute; display: block; border-radius: 2px; }
  .mini .sb { left: 0; top: 0; bottom: 0; width: 22%; background: var(--m-side); border-radius: 0; }
  .mini .hd { left: 28%; right: 6%; top: 10%; height: 14%; background: var(--m-head); }
  .mini .bd { left: 28%; right: 6%; top: 32%; height: 34%; background: var(--m-body); }
  .mini .ac { left: 28%; width: 30%; bottom: 10%; height: 14%; background: #5ecf6f; }
  .half .sb { width: 28%; }
  .half .hd, .half .bd { left: 36%; right: 8%; }
  .half .ac { left: 36%; width: 40%; }
  .tl { text-align: center; }
  .note { margin: 0; font-size: var(--fs-sm); }
  .exrow { display: flex; flex-wrap: wrap; align-items: center; gap: 0.6rem; }
</style>
