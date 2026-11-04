<script lang="ts">
  import { onDestroy } from "svelte";
  import Icon from "../Icon.svelte";
  import { t } from "../../i18n";
  import { pauseJob } from "../../engine";
  import { pausedId } from "../../jobui";
  import type { Proc } from "../../procs";

  /** Un proceso en marcha como un rectángulo que se enche de esquerda a dereita, co texto por riba.
   *  A mesma peza vale para a barra de abaixo (pequena) e para Axustes → Procesos (grande, con máis información). */
  export let proc: Proc;
  export let pct = 0;
  export let large = false;
  /** Cantos procesos máis hai en marcha (só na pequena). */
  export let more = 0;
  export let onOpen: (() => void) | null = null;
  export let onCancel: (p: Proc) => void;

  let ask = false;
  /** Se foi o «cancelar» quen o puxo en pausa, «seguir» retómao. */
  let autoPaused = false;
  $: paused = $pausedId === proc.id;
  $: pausable = proc.source === "app";
  $: percent = Math.round(Math.max(0, Math.min(1, pct)) * 100);

  async function togglePause() {
    if (paused) {
      await pauseJob(false).catch(() => false);
      pausedId.set(null);
    } else if (await pauseJob(true).catch(() => false)) {
      pausedId.set(proc.id);
    }
  }
  // Premer ✕ para o proceso mentres se decide (se se pode); «seguir» retómao e o ✓ cancélao.
  async function askCancel() {
    ask = true;
    if (pausable && !paused && (await pauseJob(true).catch(() => false))) {
      pausedId.set(proc.id);
      autoPaused = true;
    }
  }
  async function keepGoing() {
    ask = false;
    if (autoPaused) {
      autoPaused = false;
      await pauseJob(false).catch(() => false);
      pausedId.set(null);
    }
  }
  function confirm() {
    ask = false;
    autoPaused = false;
    onCancel(proc);
  }

  // Tempo transcorrido e restante (só na versión grande): refrescan cada segundo.
  let now = Date.now();
  const timer = large ? setInterval(() => (now = Date.now()), 1000) : undefined;
  onDestroy(() => clearInterval(timer));
  const fmt = (ms: number) => {
    const s = Math.max(0, Math.round(ms / 1000));
    return s >= 3600 ? `${Math.floor(s / 3600)} h ${Math.floor((s % 3600) / 60)} min` : s >= 60 ? `${Math.floor(s / 60)} min ${s % 60} s` : `${s} s`;
  };
  $: elapsed = Math.max(0, now - proc.started);
  $: left = pct > 0.04 && pct < 1 ? (elapsed * (1 - pct)) / pct : null;
</script>

<div class="jb" class:large class:paused class:ask role="group" aria-label={proc.title}>
  <span class="jfill" style={`width:${Math.max(3, percent)}%`} aria-hidden="true"></span>
  {#if ask}
    <span class="jtext"><Icon name={paused ? "pause" : "warning"} size={large ? 18 : 14} /> <span class="q">{$t("Cancelar este proceso?")}</span><span class="pct">{percent} %</span></span>
    <button class="jx stop" on:click={confirm} title={$t("Si, cancelar")} aria-label={$t("Si, cancelar")}><Icon name="check" size={large ? 18 : 14} /></button>
    <button class="jx" on:click={keepGoing} title={$t("Seguir")} aria-label={$t("Seguir")}><Icon name="undo" size={large ? 18 : 14} /></button>
  {:else}
    {#if onOpen}
      <button class="jmain" on:click={onOpen} title={`${proc.title} — ${$t("Procesos")}`} aria-label={$t("Procesos")}>
        <span class="jt">{proc.title}</span>
        <span class="pct">{percent} %{more > 0 ? ` · +${more} ${$t("máis en marcha")}` : ""}</span>
      </button>
    {:else}
      <span class="jmain static">
        <span class="col">
          <span class="jt">{proc.title}</span>
          <span class="sub">{paused ? $t("En pausa") : proc.subtitle ?? ""} · {proc.source === "quick" ? $t("Clic dereito") : $t("Aplicación")} · {fmt(elapsed)}{left !== null && !paused ? ` · ${$t("quedan")} ${fmt(left)}` : ""}</span>
        </span>
        <span class="pct big">{percent} %</span>
      </span>
    {/if}
    {#if pausable}
      <button class="jx" on:click={togglePause} title={paused ? $t("Continuar") : $t("Pausar")} aria-label={paused ? $t("Continuar") : $t("Pausar")}><Icon name={paused ? "play" : "pause"} size={large ? 18 : 14} /></button>
    {/if}
    <button class="jx stop" on:click={askCancel} title={$t("Cancelar")} aria-label={$t("Cancelar")}><Icon name="close" size={large ? 18 : 14} /></button>
  {/if}
</div>

<style>
  .jb { --jc: var(--accent); position: relative; flex: 1 1 auto; min-width: 0; display: flex; align-items: stretch; height: 32px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); overflow: hidden; transition: border-color 0.25s; }
  .jb.large { height: 64px; border-radius: var(--radius-md); }
  .jb.paused, .jb.ask { --jc: var(--accent-pdf); border-color: color-mix(in srgb, var(--jc) 55%, var(--border)); }
  .jfill { position: absolute; left: 0; top: 0; bottom: 0; z-index: 0; background: color-mix(in srgb, var(--jc) 30%, transparent); border-right: 2px solid var(--jc); box-sizing: border-box; transition: width 0.3s, background 0.25s, border-color 0.25s; }
  .paused .jfill { animation: pausePulse 1.6s ease-in-out infinite; }
  @keyframes pausePulse { 50% { opacity: 0.55; } }
  .jmain, .jtext { position: relative; z-index: 1; flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.6rem; padding: 0 0.7rem; background: none; border: none; color: var(--text); font-size: 0.78rem; text-align: left; }
  .large .jmain, .large .jtext { padding: 0 1rem; font-size: 0.9rem; gap: 0.8rem; }
  button.jmain:hover { background: color-mix(in srgb, var(--text) 6%, transparent); }
  .jtext { gap: 0.5rem; white-space: nowrap; overflow: hidden; }
  .jtext .pct { margin-left: auto; }
  .q { overflow: hidden; text-overflow: ellipsis; }
  .col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 0.1rem; }
  .jt { flex: 1 1 auto; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 550; }
  .col .jt { flex: none; font-size: 0.95rem; }
  .sub { font-size: 0.76rem; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .pct { white-space: nowrap; font-variant-numeric: tabular-nums; color: var(--text-muted); }
  .pct.big { font-size: 1.25rem; font-weight: 600; color: var(--text); }
  .jx { position: relative; z-index: 1; flex: none; display: grid; place-items: center; width: 32px; padding: 0; background: none; border: none; border-left: 1px solid color-mix(in srgb, var(--text) 14%, transparent); border-radius: 0; color: var(--text-muted); }
  .large .jx { width: 52px; }
  .jx:hover { background: color-mix(in srgb, var(--text) 10%, transparent); color: var(--text); }
  .jx.stop:hover { background: color-mix(in srgb, var(--accent-video) 20%, transparent); color: var(--accent-video); }
</style>
