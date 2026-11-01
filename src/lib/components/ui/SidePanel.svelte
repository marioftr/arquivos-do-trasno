<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { t } from "../../i18n";
  import Icon from "../Icon.svelte";

  /** Panel acoplado ao lado dun visor: empurra o contido en vez de taparo. Leva as súas seccións dentro. */
  export let title: string;
  export let width = 272;
  const dispatch = createEventDispatcher<{ close: void }>();
</script>

<aside class="sp" style={`--sp-w:${width}px`} aria-label={title}>
  <header>
    <h3>{title}</h3>
    <button class="x" on:click={() => dispatch("close")} aria-label={$t("Agochar panel")} title={$t("Agochar panel")}><Icon name="close" size={14} /></button>
  </header>
  <div class="body scrollbar-thin"><slot /></div>
</aside>

<style>
  .sp { flex: 0 0 var(--sp-w); width: var(--sp-w); max-width: 46%; min-width: 0; display: flex; flex-direction: column; background: var(--bg-rail); border-left: 1px solid var(--border); font-size: 0.84rem; }
  header { display: flex; align-items: center; justify-content: space-between; padding: 0.55rem 0.7rem 0.55rem 0.9rem; border-bottom: 1px solid var(--border); }
  h3 { margin: 0; font-size: 0.86rem; font-weight: 600; }
  .x { display: grid; place-items: center; width: 26px; height: 26px; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-faint); }
  .x:hover { background: var(--bg-hover); color: var(--text); }
  .body { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; }
</style>
