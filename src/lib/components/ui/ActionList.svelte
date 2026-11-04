<script lang="ts">
  import { actions, revertTo, revertCount, sessionStart } from "../../agent";
  import { procs, type Proc } from "../../procs";
  import { t, tr } from "../../i18n";
  import { dialog } from "../../settings";
  import { addPaths } from "../../files";
  import { fileSize, revealInFolder } from "../../engine";
  import { fileNameOf } from "../../formats";
  import { toast } from "../../toasts";
  import Icon from "../Icon.svelte";

  /** Historial único: o que fixo o trasno nesta sesión e os procesos de antes (os das xanelas rápidas e doutras sesións),
   *  a máis antiga arriba. `limit` amosa só as últimas. */
  export let limit = Infinity;
  /** Ao abrir déixase á vista o final (o máis recente). */
  export let stick = false;
  /** Na lista grande de Axustes a máis recente vai arriba; no globo da barra, abaixo (máis perto do trasno). */
  export let newestFirst = false;
  /** Chámase despois de abrir un ficheiro desde a lista (para pechar o globo do historial). */
  export let onOpen: () => void = () => {};

  interface Row {
    key: string;
    at: number;
    text: string;
    /** Posición en `actions` (só as accións do trasno, que son as que se poden desfacer). */
    idx: number | null;
    undo: boolean;
    copy?: string;
    outputs?: string[];
  }

  let revertIdx: number | null = null;
  let copied = "";
  const hhmm = (n: number) => new Date(n).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  /** Día e mes, só se non é hoxe. */
  const dayOf = (n: number) => (new Date(n).toDateString() === new Date().toDateString() ? "" : new Date(n).toLocaleDateString([], { day: "2-digit", month: "2-digit" }));
  const stateLabel = (s: Proc["state"]) => (s === "done" ? tr("Rematado") : s === "error" ? tr("Con erro") : s === "cancelled" ? tr("Cancelado") : tr("Interrompido"));

  $: actionRows = $actions.map((a, i): Row => ({ key: `a${i}|${a.at}|${a.text}`, at: a.at, text: a.text, idx: i, undo: !!a.undo, copy: a.copy, outputs: a.outputs }));
  // Os traballos desta sesión da app xa están contados polas accións; só entran os demais (xanelas rápidas e sesións anteriores).
  $: procRows = $procs
    .filter((p) => p.state !== "running" && !(p.source === "app" && p.started >= sessionStart))
    .map((p): Row => ({
      key: `p${p.id}`,
      at: p.ended ?? p.started,
      text: `${stateLabel(p.state)}: ${p.title}${p.detail ? ` · ${p.detail}` : ""}`,
      idx: null,
      undo: false,
      outputs: p.outputs,
    }));
  $: rows = (() => {
    const l = [...actionRows, ...procRows].sort((x, y) => x.at - y.at).slice(-limit);
    return newestFirst ? l.reverse() : l;
  })();

  const toEnd = (n: HTMLElement) => {
    if (!stick) return;
    const box = n.closest(".scrolls") as HTMLElement | null;
    if (box) box.scrollTop = box.scrollHeight;
  };
  async function copyText(text: string, key: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = key;
      setTimeout(() => (copied = ""), 1400);
    } catch {
      // o texto tamén se pode seleccionar a man
    }
  }
  const gone = (name: string) => toast(tr("«{name}» xa non está onde o deixou o trasno: puido moverse ou borrarse.", { name }), { kind: "warn" });
  /** Abrir só le o ficheiro: nunca o modifica. Se xa non existe, avísase en lugar de intentalo. */
  async function openOut(outputs: string[]) {
    const here: string[] = [];
    for (const p of outputs) {
      try {
        await fileSize(p);
        here.push(p);
      } catch {
        gone(fileNameOf(p));
      }
    }
    if (!here.length) return;
    dialog.set(null);
    onOpen();
    await addPaths(here);
  }
  async function showOut(path: string) {
    try {
      await revealInFolder(path);
    } catch {
      gone(fileNameOf(path));
    }
  }
</script>

{#if revertIdx !== null && $actions[revertIdx]}
  <div class="conf">
    <p><b>{$t("Desfacer este cambio?")}</b></p>
    <p class="cqt">{$actions[revertIdx].text}</p>
    <p class="muted">{revertCount($actions, revertIdx) === 1 ? $t("Desfarase só este cambio.") : $t("Desfaranse este cambio e os posteriores ({n} en total).", { n: revertCount($actions, revertIdx) })}</p>
    <p class="muted">{$t("Só se desfán cambios da lista e das anotacións: os ficheiros do disco non se tocan.")}</p>
    <div class="cacts">
      <button class="btn danger" on:click={() => { revertTo(revertIdx ?? 0); revertIdx = null; }}>{$t("Si, desfacer")}</button>
      <button class="btn ghost" on:click={() => (revertIdx = null)}>{$t("Cancelar")}</button>
    </div>
  </div>
{:else if rows.length}
  <ul use:toEnd>
    {#each rows as r (r.key)}
      <li>
        <button class="hrow" disabled={!r.undo} on:click={() => (revertIdx = r.idx)} title={r.undo ? $t("Desfacer este cambio e os posteriores") : ""}>
          <time class="muted">{hhmm(r.at)}{#if dayOf(r.at)}<small>{dayOf(r.at)}</small>{/if}</time>
          <span class="ht" title={r.text}>{r.text}</span>
          {#if r.undo}<span class="hu"><Icon name="undo" size={14} /></span>{/if}
        </button>
        {#if r.outputs?.length || r.copy}
          <span class="ract">
            {#if r.outputs?.length}
              <button class="ib" on:click={() => openOut(r.outputs ?? [])} title={$t("Abrir co programa completo")} aria-label={$t("Abrir co programa completo")}><Icon name="load" size={15} /></button>
              <button class="ib" on:click={() => showOut(r.outputs?.[0] ?? "")} title={$t("Amosar no cartafol")} aria-label={$t("Amosar no cartafol")}><Icon name="folder" size={15} /></button>
            {/if}
            {#if r.copy}<button class="ib" on:click={() => copyText(r.copy ?? "", r.key)} aria-label={$t("Copiar")} title={$t("Copiar a mensaxe")}><Icon name={copied === r.key ? "check" : "copy"} size={14} /></button>{/if}
          </span>
        {/if}
      </li>
    {/each}
  </ul>
{:else}
  <p class="muted none">{$t("Aínda non hai accións.")}</p>
{/if}

<style>
  ul { margin: 0; padding: 0; list-style: none; }
  li { display: flex; align-items: center; }
  .hrow { flex: 1; min-width: 0; display: grid; grid-template-columns: 3.4rem 1fr auto; align-items: center; gap: 0.6rem; padding: 0.55rem 0.5rem 0.55rem 0.9rem; background: none; border: none; border-radius: 0; color: var(--text); text-align: left; font-size: 0.8rem; }
  .hrow:disabled { cursor: default; }
  .hrow:hover:not(:disabled) { background: var(--bg-hover); }
  time { font-variant-numeric: tabular-nums; display: flex; flex-direction: column; line-height: 1.2; }
  time small { font-size: 0.68rem; color: var(--text-faint); }
  .ht { min-width: 0; overflow-wrap: anywhere; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
  .hu { color: var(--text-faint); display: inline-flex; }
  .hrow:hover .hu { color: var(--accent-video); }
  .ract { flex: none; display: flex; align-items: center; gap: 1px; padding-right: 0.5rem; }
  .ib { display: grid; place-items: center; width: 28px; height: 28px; padding: 0; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-muted); }
  .ib:hover { background: var(--bg-hover); color: var(--text); }
  .conf { padding: 0.3rem 0.9rem 0.5rem; display: flex; flex-direction: column; gap: 0.4rem; font-size: 0.82rem; }
  .conf p { margin: 0; }
  .cqt { padding: 0.35rem 0.6rem; background: var(--bg-input); border-radius: var(--radius-sm); }
  .cacts { display: flex; gap: 0.4rem; margin-top: 0.3rem; }
  .none { margin: 0; padding: 0.3rem 0.8rem; font-size: 0.8rem; }
</style>
