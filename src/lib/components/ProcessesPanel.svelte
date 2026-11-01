<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { t } from "../i18n";
  import { dialog } from "../settings";
  export let part: "running" | "history" = "running";
  import { procs, refreshProcs, requestCancel, clearFinished, type Proc } from "../procs";
  import { cancelJob, revealInFolder } from "../engine";
  import { fileNameOf } from "../formats";
  import { addPaths } from "../files";
  import Icon from "./Icon.svelte";

  let timer: ReturnType<typeof setInterval>;
  let asking: string | null = null;
  onMount(() => {
    refreshProcs();
    timer = setInterval(refreshProcs, 1000);
  });
  onDestroy(() => clearInterval(timer));

  $: running = $procs.filter((p) => p.state === "running");
  $: history = $procs.filter((p) => p.state !== "running");

  async function cancel(p: Proc) {
    asking = null;
    // Os da aplicación principal paranse aquí mesmo; os das xanelas rápidas pídeselles que se cancelen (borran só o incompleto).
    if (p.source === "app") await cancelJob().catch(() => false);
    else await requestCancel(p.id).catch(() => {});
    refreshProcs();
  }

  const clock = (ms: number) => new Date(ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  const length = (p: Proc) => {
    const s = Math.max(0, Math.round(((p.ended ?? Date.now()) - p.started) / 1000));
    return s >= 60 ? `${Math.floor(s / 60)} min ${s % 60} s` : `${s} s`;
  };
  const label = (s: Proc["state"]) =>
    s === "done" ? $t("Rematado") : s === "error" ? $t("Con erro") : s === "cancelled" ? $t("Cancelado") : s === "interrupted" ? $t("Interrompido") : $t("En marcha");
</script>

{#if part === "running"}
  <section>
    <h3>{$t("En marcha")} <span class="muted">{running.length}</span></h3>
    {#if !running.length}
      <p class="muted note">{$t("Non hai ningún proceso en marcha. O trasno descansa.")}</p>
    {/if}
    {#each running as p (p.id)}
      <div class="row">
        <div class="top">
          <div class="txt">
            <span class="n" title={p.title}>{p.title}</span>
            <span class="muted sub">{p.subtitle ?? ""} · {p.source === "quick" ? $t("Clic dereito") : $t("Aplicación")} · {length(p)}</span>
          </div>
          {#if asking === p.id}
            <span class="ask">{$t("Cancelar?")}</span>
            <button class="btn danger" on:click={() => cancel(p)}>{$t("Si, cancelar")}</button>
            <button class="btn" on:click={() => (asking = null)}>{$t("Seguir")}</button>
          {:else}
            <button class="btn" on:click={() => (asking = p.id)}>{$t("Cancelar")}</button>
          {/if}
        </div>
        <div class="bar"><div class="fill" style={`width:${Math.round(p.progress * 100)}%`}></div></div>
        <span class="muted pct">{Math.round(p.progress * 100)} %</span>
      </div>
    {/each}
  </section>

{:else}
  <section>
    <div class="hh">
      <h3>{$t("Historial")} <span class="muted">{history.length}</span></h3>
      {#if history.length}<button class="btn ghost" on:click={clearFinished}>{$t("Borrar historial")}</button>{/if}
    </div>
    {#if !history.length}<p class="muted note">{$t("Aínda non hai nada no historial.")}</p>{/if}
    {#each history as p (p.id)}
      <div class="hrow">
        <span class={`dotc ${p.state}`} aria-hidden="true"></span>
        <div class="txt">
          <span class="n" title={p.title}>{p.title}</span>
          <span class="muted sub">
            {p.subtitle ?? ""} · {label(p.state)} · {clock(p.started)} · {length(p)}{#if p.detail}&nbsp;·&nbsp;{p.detail}{/if}
          </span>
          {#if p.outputs?.length}<span class="muted sub">→ {p.outputs.map(fileNameOf).join(", ")}</span>{/if}
        </div>
        {#if p.outputs?.length}
          <button class="btn ghost icon" on:click={() => { addPaths(p.outputs ?? []); dialog.set(null); }} title={$t("Abrir co programa completo")} aria-label={$t("Abrir co programa completo")}><Icon name="open" size={15} /></button>
          <button class="btn ghost icon" on:click={() => revealInFolder(p.outputs![0])} title={$t("Amosar no cartafol")} aria-label={$t("Amosar no cartafol")}><Icon name="folder" size={15} /></button>
        {/if}
      </div>
    {/each}
  </section>
{/if}

<style>
  section { display: flex; flex-direction: column; gap: 0.5rem; }
  h3 { margin: 0; font-size: 0.86rem; font-weight: 600; }
  .hh { display: flex; align-items: center; justify-content: space-between; }
  .note { margin: 0; font-size: 0.82rem; }
  .row { display: flex; flex-direction: column; gap: 0.35rem; padding: 0.6rem 0.7rem; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); }
  .top { display: flex; align-items: center; gap: 0.5rem; }
  .txt { flex: 1; min-width: 0; display: flex; flex-direction: column; line-height: 1.3; }
  .n { font-weight: 550; font-size: 0.88rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sub { font-size: 0.76rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .bar { height: 7px; border-radius: 99px; background: var(--bg-hover); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); border-radius: 99px; transition: width 0.3s ease; }
  .pct { font-size: 0.74rem; text-align: right; font-variant-numeric: tabular-nums; }
  .ask { font-size: 0.8rem; }
  .row :global(.btn), .hrow :global(.btn) { height: var(--control-h); min-height: var(--control-h); padding-block: 0; box-sizing: border-box; }
  .hrow { display: flex; align-items: center; gap: 0.6rem; padding: 0.4rem 0.2rem; border-top: 1px solid var(--border); }
  .dotc { width: 9px; height: 9px; border-radius: 50%; flex: 0 0 auto; background: var(--text-faint); }
  .dotc.done { background: var(--accent); }
  .dotc.error { background: var(--accent-video); }
  .dotc.cancelled, .dotc.interrupted { background: var(--accent-pdf); }
</style>
