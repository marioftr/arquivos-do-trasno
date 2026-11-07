<script lang="ts">
  import { t } from "../i18n";
  import { tools } from "../settings";
  import { openLink, cancelToolInstall } from "../engine";
  import { need, installing, installError, installToolNow, overall, phaseText, TOOL_INFO } from "../toolinstall";
  import Modal from "./ui/Modal.svelte";
  import Icon from "./Icon.svelte";

  $: n = $need;
  $: info = n ? TOOL_INFO[n.tool] : null;
  $: ready = !!n && !!$tools?.[n.tool];
  $: busy = $installing?.tool === n?.tool ? $installing : null;
  $: pct = busy ? Math.round(overall(busy) * 100) : 0;
  const close = () => need.set(null);

  async function accept() {
    if (!n) return;
    await installToolNow(n.tool);
  }
  function again() {
    const retry = n?.retry;
    close();
    retry?.();
  }
</script>

{#if n && info}
  <Modal title={$t("Falta un complemento")} onClose={() => !busy && close()}>
    <div class="body">
      <p class="reason">{n.reason}</p>
      <p class="muted"><strong>{info.name}</strong> · {$t(info.why)}</p>
      <p><span class="muted">{$t("Desbloquea:")}</span> {$t(info.unlocks)}</p>

      {#if busy}
        <div class="prog">
          <div class="line"><span>{phaseText(busy.phase)}</span><span class="muted pct">{pct} %</span></div>
          <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct}><div class="fill" style={`width:${pct}%`}></div></div>
        </div>
        <div class="acts"><button class="btn" on:click={() => cancelToolInstall()}>{$t("Cancelar descarga")}</button></div>
      {:else if ready}
        <p class="done"><Icon name="check" size={15} /> {$t("{name} xa está na casa. A traballar!", { name: info.name })}</p>
        <div class="acts">
          {#if n.retry}<button class="btn primary" on:click={again}>{$t("Repetir agora")}</button>{/if}
          <button class="btn" on:click={close}>{$t("Pechar")}</button>
        </div>
      {:else}
        <p class="muted small">
          {$t("O trasno descargará {name} (uns {size}) de {source}, comprobará que o ficheiro está íntegro e deixarao na súa carpeta de datos. Non fai falta ser administrador.", { name: info.name, size: info.size, source: info.source })}
        </p>
        {#if $installError?.tool === n.tool}<p class="fail selectable">{$installError.text}</p>{/if}
        <div class="acts">
          <button class="btn primary" on:click={accept}>{$t("Descargar e instalar")} <span class="muted">· {info.size}</span></button>
          <button class="btn" on:click={() => openLink(info.link)}><Icon name="open" size={14} /> {$t("Ir á páxina oficial")}</button>
          <button class="btn ghost" on:click={close}>{$t("Agora non")}</button>
        </div>
        <p class="muted small">{$t("Se o prefires, instálao ti e despois preme «Volver comprobar» en Axustes → Extensións.")}</p>
      {/if}
    </div>
  </Modal>
{/if}

<style>
  .body { display: flex; flex-direction: column; gap: 0.7rem; }
  .body p { margin: 0; }
  .reason { font-weight: 550; }
  .small { font-size: var(--fs-sm); }
  .fail { color: var(--accent-video); font-size: var(--fs-md); }
  .done { display: flex; align-items: center; gap: 0.4rem; color: var(--accent-imaxe); }
  .acts { display: flex; flex-wrap: wrap; align-items: center; gap: 0.6rem; }
  .prog { display: flex; flex-direction: column; gap: 0.4rem; }
  .line { display: flex; justify-content: space-between; font-size: var(--fs-md); }
  .pct { font-variant-numeric: tabular-nums; }
  .bar { height: 7px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); border-radius: 99px; transition: width 0.2s ease; }
</style>
