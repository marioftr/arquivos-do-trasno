<script lang="ts">
  import { t, tr } from "../../i18n";
  import { tools } from "../../settings";
  import { cancelToolInstall } from "../../engine";
  import { formatBytes } from "../../formats";
  import { TOOL_INFO, installing, installError, installToolNow, overall, phaseText } from "../../toolinstall";
  import { WHISPER_MODELS, CUTOUT_MODEL, modelBusy, modelError, downloadModel, missingOf, type Need } from "../../models";

  /** Extensións que fan falta para a ferramenta: se falta algo, sae a descarga en vez da ferramenta. */
  export let needs: Need[];
  /** Aclaración opcional baixo a lista (por que fan falta varias cousas). */
  export let note = "";

  $: missing = missingOf(needs, $tools);
  const modelOf = (id: string) => (id === "cutout" ? CUTOUT_MODEL : WHISPER_MODELS.find((m) => m.id === id)!);
  const rows = (list: Need[]) =>
    list.map((n) =>
      n.kind === "tool"
        ? { name: TOOL_INFO[n.id].name, what: tr(TOOL_INFO[n.id].role), bytes: TOOL_INFO[n.id].bytes }
        : n.id === "cutout"
          ? { name: modelOf(n.id).name, what: tr("recoñece o obxecto"), bytes: modelOf(n.id).bytes }
          : { name: modelOf(n.id).name, what: tr("recoñece a voz, versión {name}", { name: tr(modelOf(n.id).label).toLowerCase() }), bytes: modelOf(n.id).bytes },
    );
  $: list = rows(missing);
  $: total = formatBytes(list.reduce((a, r) => a + r.bytes, 0));

  let stopped = false;
  async function download() {
    stopped = false;
    for (const n of missing) {
      const ok = n.kind === "tool" ? await installToolNow(n.id) : await downloadModel(n.id);
      if (!ok) {
        stopped = true;
        break;
      }
    }
  }
  $: busyTool = $installing;
  $: busyModel = $modelBusy;
  $: busy = !!busyTool || !!busyModel;
  $: pct = busyTool ? Math.round(overall(busyTool) * 100) : busyModel ? Math.round(busyModel.progress * 100) : 0;
  $: what = busyTool ? phaseText(busyTool.phase) : tr("Descargando…");
  $: err = $installError?.text ?? $modelError?.text ?? "";
</script>

{#if missing.length}
  <div class="gate" data-testid="extension-gate">
    <p class="lead">{$t("Falta instalar:")}</p>
    <ul>
      {#each list as r}
        <li><strong>{r.name}</strong> <span class="muted">· {r.what} · {formatBytes(r.bytes)}</span></li>
      {/each}
    </ul>
    {#if note && list.length > 1}<p class="muted small">{note}</p>{/if}
    <p class="muted small">{$t("Instálase unha soa vez e pódese quitar en Axustes → Extensións.")}</p>
    {#if busy}
      <div class="prog">
        <div class="line"><span>{what}</span><span class="muted pct">{pct} %</span></div>
        <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct}><div class="fill" style={`width:${pct}%`}></div></div>
      </div>
      <div class="estimate"><button class="btn" on:click={() => cancelToolInstall()}>{$t("Cancelar descarga")}</button></div>
    {:else}
      {#if err && stopped}<p class="fail selectable">{err}</p>{/if}
      <button class="btn primary block" on:click={download}>{$t("Descargar e instalar")} <span class="size">· {total}</span></button>
    {/if}
  </div>
{/if}

<style>
  .gate { display: flex; flex-direction: column; gap: 0.6rem; padding: 0.8rem 0.9rem; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); }
  .gate p, .gate ul { margin: 0; }
  ul { padding-left: 1.1rem; display: flex; flex-direction: column; gap: 0.2rem; }
  .lead { font-weight: var(--fw-medium); }
  .small { font-size: var(--fs-sm); }
  .size { font-weight: var(--fw-regular); }
  .fail { color: var(--accent-video); font-size: var(--fs-md); }
  .prog { display: flex; flex-direction: column; gap: 0.4rem; }
  .line { display: flex; justify-content: space-between; font-size: var(--fs-md); }
  .pct { font-variant-numeric: tabular-nums; }
  .bar { height: 7px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); border-radius: 99px; transition: width 0.2s ease; }
</style>
