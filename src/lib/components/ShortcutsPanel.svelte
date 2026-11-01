<script lang="ts">
  import { onDestroy } from "svelte";
  import { t, tr } from "../i18n";
  import { ACTIONS, GROUPS, custom, slotsOf, isChanged, comboOf, setSlot, resetAction, resetAll, pretty } from "../keymap";
  import { toast } from "../toasts";
  import Icon from "./Icon.svelte";

  /** Atallos fixos (non se poden cambiar): móvense con teclas de uso xeral doutros controis. */
  const FIXED: { title: string; keys: { k: string; d: string }[] }[] = [
    {
      title: "Lista de ficheiros",
      keys: [
        { k: "↑ ↓ · Inicio · Fin · Espazo · Ctrl + A", d: "Moverse pola lista, marcar con Espazo, marcar todo" },
        { k: "Maiús / Ctrl + clic", d: "Maiús ou Ctrl + clic marcan varios ficheiros" },
        { k: "Alt + ↑ ↓", d: "Mover o ficheiro na lista" },
        { k: "Ctrl + Z", d: "Desfacer ter quitado un ficheiro (coa lista en foco)" },
      ],
    },
    {
      title: "Pestanas e opcións",
      keys: [{ k: "← → · Inicio · Fin", d: "Moverse entre pestanas e opcións" }],
    },
  ];

  // Mentres se espera unha tecla, esa tecla non pode activar nada máis (nin pechar o diálogo).
  let capturing: { id: string; slot: 0 | 1 } | null = null;
  function onKey(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    if (e.key === "Escape" && !e.ctrlKey && !e.altKey && !e.shiftKey) {
      capturing = null;
      return;
    }
    const combo = comboOf(e);
    if (!combo) return; // só un modificador: agarda o resto
    const { id, slot } = capturing;
    capturing = null;
    const stolen = setSlot(id, slot, combo);
    if (stolen) {
      const other = ACTIONS.find((a) => a.id === stolen)!;
      toast(tr("«{key}» xa o usaba «{action}»; o trasno quitoullo.", { key: pretty(combo), action: tr(other.label) }));
    }
  }
  if (typeof window !== "undefined") window.addEventListener("keydown", onKey, true);
  onDestroy(() => window.removeEventListener("keydown", onKey, true));

  const changedCount = () => ACTIONS.filter((a) => isChanged(a.id, $custom)).length;
  let confirmAll = false;
</script>

<section>
  <div class="top">
    <p class="muted note">{$t("Preme un atallo para cambialo. Cada acción pode ter dous: valen os dous á vez.")}</p>
    {#if confirmAll}
      <span class="ask">{$t("Restablecer todos?")}</span>
      <button class="btn danger" on:click={() => { resetAll(); confirmAll = false; }}>{$t("Si, restablecer")}</button>
      <button class="btn" on:click={() => (confirmAll = false)}>{$t("Seguir")}</button>
    {:else}
      <button class="btn" on:click={() => (confirmAll = true)} disabled={!changedCount()}><Icon name="reset" size={14} /> {$t("Restablecer todos")}</button>
    {/if}
  </div>

  {#each GROUPS as g}
    <div class="grp">
      <h4>{$t(g.title)}</h4>
      {#each ACTIONS.filter((a) => a.group === g.id) as a (a.id)}
        {@const s = slotsOf(a.id, $custom)}
        <div class="row">
          <span class="d">{$t(a.label)}</span>
          {#each [0, 1] as i}
            {@const slot = i as 0 | 1}
            <span class="slot">
              <button
                class="key"
                class:wait={capturing?.id === a.id && capturing.slot === slot}
                class:empty={!s[slot]}
                on:click={() => (capturing = { id: a.id, slot })}
                title={s[slot] ? $t("Cambiar este atallo") : $t("Engadir un atallo")}
              >
                {#if capturing?.id === a.id && capturing.slot === slot}{$t("Preme unha tecla…")}{:else if s[slot]}{pretty(s[slot])}{:else}<span class="muted">+</span>{/if}
              </button>
              {#if s[slot]}
                <button class="x" on:click={() => setSlot(a.id, slot, null)} aria-label={$t("Quitar este atallo")} title={$t("Quitar este atallo")}><Icon name="close" size={11} /></button>
              {/if}
            </span>
          {/each}
          <button class="rs" class:vis={isChanged(a.id, $custom)} on:click={() => resetAction(a.id)} disabled={!isChanged(a.id, $custom)} aria-label={$t("Volver ao atallo por defecto")} title={$t("Volver ao atallo por defecto")}><Icon name="reset" size={13} /></button>
        </div>
      {/each}
    </div>
  {/each}

  {#each FIXED as g}
    <div class="grp">
      <h4>{$t(g.title)} <span class="muted fx">· {$t("fixos")}</span></h4>
      {#each g.keys as r}
        <div class="fixed"><kbd>{r.k}</kbd><span>{$t(r.d)}</span></div>
      {/each}
    </div>
  {/each}
</section>

<style>
  section { display: flex; flex-direction: column; gap: 1rem; }
  .top { display: flex; align-items: center; gap: 0.6rem; }
  .note { margin: 0; flex: 1; font-size: 0.82rem; }
  .ask { font-size: 0.82rem; }
  .top :global(.btn) { height: var(--control-h); min-height: var(--control-h); padding-block: 0; }
  .grp { display: flex; flex-direction: column; gap: 0.15rem; }
  h4 { margin: 0 0 0.3rem; font-size: 0.84rem; font-weight: 600; }
  .fx { font-weight: 400; font-size: 0.76rem; }
  .row { display: grid; grid-template-columns: minmax(0, 1fr) 8.2rem 8.2rem 1.8rem; align-items: center; gap: 0.4rem; padding: 0.15rem 0; border-top: 1px solid var(--border); }
  .d { font-size: 0.84rem; overflow: hidden; text-overflow: ellipsis; }
  .slot { position: relative; display: flex; }
  .key { flex: 1; min-height: 28px; padding: 0 1.5rem 0 0.5rem; background: var(--bg-input); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); font: inherit; font-size: 0.8rem; text-align: center; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .key.empty { padding: 0 0.5rem; border-style: dashed; background: transparent; }
  .key:hover { border-color: var(--text-faint); }
  .key.wait { border-color: var(--accent); box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 30%, transparent); color: var(--accent); }
  .x { position: absolute; right: 3px; top: 50%; transform: translateY(-50%); display: grid; place-items: center; width: 20px; height: 20px; background: none; border: none; border-radius: 50%; color: var(--text-faint); }
  .x:hover { background: var(--bg-hover); color: var(--text); }
  .rs { display: grid; place-items: center; width: 26px; height: 26px; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-faint); visibility: hidden; }
  .rs.vis { visibility: visible; color: var(--accent); }
  .rs:hover:not(:disabled) { background: var(--bg-hover); }
  .fixed { display: flex; align-items: baseline; gap: 0.8rem; padding: 0.2rem 0; border-top: 1px solid var(--border); font-size: 0.82rem; }
  kbd { flex: 0 0 13rem; font-family: inherit; font-size: 0.78rem; color: var(--text-muted); }
</style>
