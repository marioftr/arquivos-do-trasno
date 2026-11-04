<script lang="ts">
  import { actions, revertTo, revertCount } from "../../agent";
  import { t } from "../../i18n";
  import Icon from "../Icon.svelte";

  /** Lista compartida das accións do trasno (a máis antiga arriba). `limit` amosa só as últimas. */
  export let limit = Infinity;
  /** Ao abrir déixase á vista o final (o máis recente). */
  export let stick = false;

  let revertIdx: number | null = null;
  let copied = "";
  const hhmm = (n: number) => new Date(n).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  $: rows = $actions.map((a, i) => ({ a, i })).slice(0, limit).reverse();

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
</script>

{#if revertIdx !== null && $actions[revertIdx]}
  <div class="conf">
    <p><b>{$t("Desfacer este cambio?")}</b></p>
    <p class="cqt">{$actions[revertIdx].text}</p>
    <p class="muted">{revertCount($actions, revertIdx) === 1 ? $t("Desfarase só este cambio.") : $t("Desfaranse este cambio e os posteriores ({n} en total).", { n: revertCount($actions, revertIdx) })}</p>
    <div class="cacts">
      <button class="btn danger" on:click={() => { revertTo(revertIdx ?? 0); revertIdx = null; }}>{$t("Si, desfacer")}</button>
      <button class="btn ghost" on:click={() => (revertIdx = null)}>{$t("Cancelar")}</button>
    </div>
  </div>
{:else if rows.length}
  <ul use:toEnd>
    {#each rows as { a, i } (a.at + "|" + a.text + "|" + i)}
      <li>
        <button class="hrow" disabled={!a.undo} on:click={() => (revertIdx = i)} title={a.undo ? $t("Desfacer este cambio e os posteriores") : ""}>
          <time class="muted">{hhmm(a.at)}</time>
          <span class="ht">{a.text}</span>
          {#if a.undo}<span class="hu"><Icon name="undo" size={14} /></span>{/if}
        </button>
        {#if a.copy}<button class="hcopy" on:click={() => copyText(a.copy ?? "", `h${i}`)} aria-label={$t("Copiar")} title={$t("Copiar a mensaxe")}><Icon name={copied === `h${i}` ? "check" : "copy"} size={14} /></button>{/if}
      </li>
    {/each}
  </ul>
{:else}
  <p class="muted none">{$t("Aínda non hai accións.")}</p>
{/if}

<style>
  ul { margin: 0; padding: 0; list-style: none; }
  li { position: relative; padding: 0; }
  .hrow { width: 100%; display: grid; grid-template-columns: 3.4rem 1fr auto; align-items: center; gap: 0.6rem; padding: 0.55rem 0.9rem; padding-right: 2.6rem; background: none; border: none; border-radius: 0; color: var(--text); text-align: left; font-size: 0.8rem; }
  .hrow:disabled { cursor: default; }
  .hrow:hover:not(:disabled) { background: var(--bg-hover); }
  time { font-variant-numeric: tabular-nums; }
  .ht { min-width: 0; }
  .hu { color: var(--text-faint); display: inline-flex; }
  .hrow:hover .hu { color: var(--accent-video); }
  .hcopy { position: absolute; right: 0.5rem; top: 50%; transform: translateY(-50%); display: grid; place-items: center; width: 28px; height: 28px; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-muted); }
  .hcopy:hover { background: var(--bg-hover); color: var(--text); }
  .conf { padding: 0.3rem 0.9rem 0.5rem; display: flex; flex-direction: column; gap: 0.4rem; font-size: 0.82rem; }
  .conf p { margin: 0; }
  .cqt { padding: 0.35rem 0.6rem; background: var(--bg-input); border-radius: var(--radius-sm); }
  .cacts { display: flex; gap: 0.4rem; margin-top: 0.3rem; }
  .none { margin: 0; padding: 0.3rem 0.8rem; font-size: 0.8rem; }
</style>
