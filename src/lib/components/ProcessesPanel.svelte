<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { t } from "../i18n";
  import { actions, sessionStart } from "../agent";
  export let part: "running" | "history" = "running";
  import { procs, refreshProcs, requestCancel, clearFinished, type Proc } from "../procs";
  import { cancelJob } from "../engine";
  import Icon from "./Icon.svelte";
  import ActionList from "./ui/ActionList.svelte";
  import JobBar from "./ui/JobBar.svelte";
  import { pausedId } from "../jobui";

  let timer: ReturnType<typeof setInterval>;
  onMount(() => {
    refreshProcs();
    timer = setInterval(refreshProcs, 1000);
  });
  onDestroy(() => clearInterval(timer));

  $: running = $procs.filter((p) => p.state === "running");
  $: histCount = $actions.length + $procs.filter((p) => p.state !== "running" && !(p.source === "app" && p.started >= sessionStart)).length;
  /** Baleira a lista e o historial gardado. Os ficheiros que deixaron os procesos non se tocan. */
  function clearHistory() {
    actions.set([]);
    clearFinished();
  }

  async function cancel(p: Proc) {
    pausedId.set(null);
    // Os da aplicación principal paranse aquí mesmo; os das xanelas rápidas pídeselles que se cancelen (borran só o incompleto).
    if (p.source === "app") await cancelJob().catch(() => false);
    else await requestCancel(p.id).catch(() => {});
    refreshProcs();
  }

  const length = (p: Proc) => {
    const s = Math.max(0, Math.round(((p.ended ?? Date.now()) - p.started) / 1000));
    return s >= 60 ? `${Math.floor(s / 60)} min ${s % 60} s` : `${s} s`;
  };
</script>

{#if part === "running"}
  <section>
    <h3>{$t("En marcha")}<span class="muted cnt">&nbsp;·&nbsp;{running.length}</span></h3>
    {#if !running.length}
      <p class="muted note">{$t("Non hai ningún proceso en marcha. O trasno descansa.")}</p>
    {/if}
    {#each running as p (p.id)}
      <JobBar proc={p} pct={p.progress} large onCancel={cancel} />
    {/each}
  </section>

{:else}
  <section>
    <div class="hh">
      <h3>{$t("Historial")}<span class="muted cnt">&nbsp;·&nbsp;{histCount}</span></h3>
      {#if histCount}<button class="btn ghost" on:click={clearHistory}>{$t("Borrar historial")}</button>{/if}
    </div>
    <div class="alist"><ActionList newestFirst /></div>
  </section>
{/if}

<style>
  .alist { border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); padding: 0.3rem 0; }
  section { display: flex; flex-direction: column; gap: 0.5rem; }
  h3 { margin: 0; font-size: 0.86rem; font-weight: 600; }
  .cnt { font-weight: 500; }
  .hh { display: flex; align-items: center; justify-content: space-between; }
  .note { margin: 0; font-size: 0.82rem; }
</style>
