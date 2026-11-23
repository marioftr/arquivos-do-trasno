<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { t } from "../i18n";
  import { closeTutorial } from "../settings";
  import Icon from "./Icon.svelte";
  import TrasnoSprite from "./ui/TrasnoSprite.svelte";

  /** Volta rápida á app: o trasno anda por unha xanela de exemplo, con ficheiros de proba, e unha caixa de texto explica cada paso. Nada do que se ve é real: non toca ningún ficheiro. */
  type Focus = "none" | "list" | "view" | "tools" | "go" | "bar" | "radial" | "palette";
  interface Step {
    text: string;
    x: number;
    y: number;
    state: "idle" | "working" | "done" | "notice";
    focus: Focus;
    /** Cantos ficheiros de proba hai xa na lista. */
    files: number;
    /** O primeiro está elixido e sae no visor. */
    pick: boolean;
    /** Xa saíu o ficheiro novo. */
    result: boolean;
  }

  $: steps = [
    { text: $t("Son o trasno e traballo cos teus ficheiros sen que saian deste ordenador. Demos unha volta rápida."), x: 50, y: 74, state: "notice", focus: "none", files: 0, pick: false, result: false },
    { text: $t("Engade ficheiros arrastrándoos á xanela, co botón Abrir (Ctrl+O) ou pegándoos (Ctrl+V). Quedan na lista da esquerda."), x: 40, y: 56, state: "working", focus: "list", files: 4, pick: false, result: false },
    { text: $t("Elixe un ficheiro e velo no visor. Cada tipo ten as súas ferramentas na columna da dereita."), x: 52, y: 88, state: "idle", focus: "view", files: 4, pick: true, result: false },
    { text: $t("Axusta o que queres: hai poucas opcións e cada valor di o que fai. A «i» dá unha pista de cada unha."), x: 80, y: 62, state: "idle", focus: "tools", files: 4, pick: true, result: false },
    { text: $t("Preme o botón verde e sae un ficheiro novo na lista."), x: 80, y: 90, state: "done", focus: "go", files: 4, pick: true, result: true },
    { text: $t("Os traballos longos van en segundo plano: o progreso vese debaixo e o historial está en Procesos."), x: 40, y: 94, state: "working", focus: "bar", files: 4, pick: true, result: true },
    { text: $t("Se mantés o punteiro sobre min, saen os accesos rápidos: descargar vídeos por ligazón ou torrents, buscar e axustes."), x: 12, y: 94, state: "notice", focus: "radial", files: 4, pick: true, result: true },
    { text: $t("Se non sabes onde está algo, Ctrl+K abre o buscador de ferramentas e accións."), x: 78, y: 78, state: "idle", focus: "palette", files: 4, pick: true, result: true },
    { text: $t("Isto é todo. Podes repetir esta volta cando queiras en Axustes → Xeral. Boa sorte!"), x: 50, y: 74, state: "done", focus: "none", files: 4, pick: true, result: true },
  ] as Step[];

  const FILES = [
    { name: "foto.jpg", icon: "image" },
    { name: "informe.pdf", icon: "document" },
    { name: "cancion.mp3", icon: "audio" },
    { name: "video.mp4", icon: "video" },
  ];

  let i = 0;
  $: step = steps[i];
  $: last = i === steps.length - 1;
  let moving = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  function go(to: number) {
    if (to < 0 || to >= steps.length || to === i) return;
    i = to;
    moving = true;
    clearTimeout(timer);
    timer = setTimeout(() => (moving = false), 950);
  }
  const next = () => (last ? closeTutorial() : go(i + 1));

  let panel: HTMLElement;
  let before: HTMLElement | null = null;
  onMount(() => {
    before = document.activeElement as HTMLElement | null;
    panel.querySelector<HTMLElement>("button.primary")?.focus();
  });
  onDestroy(() => {
    clearTimeout(timer);
    before?.focus?.();
  });

  function key(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      closeTutorial();
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      next();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      go(i - 1);
    } else if (e.key === "Tab") {
      // O foco queda dentro da xanela.
      const items = [...panel.querySelectorAll<HTMLElement>("button:not(:disabled)")];
      if (!items.length) return;
      const at = items.indexOf(document.activeElement as HTMLElement);
      const to = e.shiftKey ? (at <= 0 ? items.length - 1 : at - 1) : at === items.length - 1 ? 0 : at + 1;
      e.preventDefault();
      items[to].focus();
    }
  }
</script>

<svelte:window on:keydown={key} />

<div class="veil" role="presentation">
  <div class="panel" role="dialog" aria-modal="true" aria-labelledby="tut-title" bind:this={panel}>
    <h2 id="tut-title" class="sr-only">{$t("Volta rápida")}</h2>

    <div class="stage" aria-hidden="true">
      <div class="win">
        <div class="list" class:hl={step.focus === "list"}>
          {#each FILES as f, k (f.name)}
            <div class="row" class:in={k < step.files} class:sel={step.pick && k === 0} style={`--d:${k * 120}ms`}>
              <Icon name={f.icon} size={14} /><span>{f.name}</span>
            </div>
          {/each}
          <div class="row new" class:in={step.result}><Icon name="image" size={14} /><span>foto.png</span><b>{$t("Novo")}</b></div>
        </div>
        <div class="view" class:hl={step.focus === "view"}>
          <div class="pic" class:on={step.pick}><i class="sun"></i><i class="hill"></i></div>
        </div>
        <div class="tools" class:hl={step.focus === "tools" || step.focus === "go"}>
          <div class="tabs"><span class="on">{$t("Converter")}</span><span>{$t("Comprimir")}</span><span>{$t("Transformar")}</span><span>{$t("Restaurar")}</span></div>
          <div class="fmt"><span>PNG</span><span class="on">JPG</span><span>WebP</span></div>
          <div class="sld"><i></i><em class="inf"><Icon name="info" size={12} /></em></div>
          <div class="sld s2"><i></i></div>
          <div class="go" class:press={step.focus === "go"}>{$t("Converter")}</div>
        </div>
      </div>
      <div class="bar" class:hl={step.focus === "bar"} class:on={step.focus === "bar"}><span class="lab">{$t("Convertendo…")}</span><span class="track"><span class="fill"></span></span></div>

      {#if step.focus === "palette"}
        <div class="pal"><Icon name="search" size={14} /><span>{$t("Buscar…")}</span><kbd>Ctrl+K</kbd></div>
      {/if}
      {#if step.focus === "radial"}
        <div class="rad">
          <span style="--a:-70deg"><Icon name="link" size={14} /></span>
          <span style="--a:-35deg"><Icon name="magnet" size={14} /></span>
          <span style="--a:0deg"><Icon name="search" size={14} /></span>
          <span style="--a:35deg"><Icon name="gear" size={14} /></span>
        </div>
      {/if}

      <div class="tr" class:moving class:hop={step.state === "done"} style={`left:${step.x}%;top:${step.y}%`}>
        <TrasnoSprite state={step.state} size={64} />
      </div>
    </div>

    <p class="say" aria-live="polite"><span class="n">{i + 1} / {steps.length}</span> {step.text}</p>

    <div class="nav">
      <button type="button" class="btn ghost" on:click={closeTutorial}>{last ? $t("Pechar") : $t("Saltar")}</button>
      <span class="dots" aria-hidden="true">{#each steps as _, k}<i class:on={k === i}></i>{/each}</span>
      <button type="button" class="btn" on:click={() => go(i - 1)} disabled={i === 0}>{$t("Anterior")}</button>
      <button type="button" class="btn primary" on:click={next}>{last ? $t("Empezar") : $t("Seguinte")}</button>
    </div>
  </div>
</div>

<style>
  .veil { position: fixed; inset: 0; z-index: 70; display: flex; align-items: center; justify-content: center; padding: 1rem; background: rgba(0, 0, 0, 0.6); }
  .panel { display: flex; flex-direction: column; gap: 0.9rem; width: min(760px, 100%); max-height: 100%; overflow: auto; padding: 1rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: calc(var(--radius-lg) + 2px); box-shadow: var(--shadow-pop); }

  /* Escenario: unha xanela de mentira con ficheiros de proba; o trasno anda por riba. */
  .stage { position: relative; aspect-ratio: 16 / 9; min-height: 220px; background: var(--bg); border: 1px solid var(--border); border-radius: var(--radius-lg); overflow: hidden; }
  .win { position: absolute; inset: 8px 8px 30px; display: grid; grid-template-columns: 26% minmax(0, 1fr) 29%; grid-template-rows: minmax(0, 1fr); gap: 8px; }
  .list, .view, .tools { position: relative; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-md); transition: box-shadow 0.3s, border-color 0.3s; }
  .hl { border-color: var(--accent); box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 30%, transparent); }
  .list { display: flex; flex-direction: column; gap: 4px; padding: 6px; font-size: var(--fs-xs); }
  .row { display: flex; align-items: center; gap: 6px; padding: 4px 6px; border-radius: 5px; background: var(--bg-hover); color: var(--text); opacity: 0; transform: translateX(-24px); transition: opacity 0.4s var(--d, 0ms), transform 0.4s var(--d, 0ms); white-space: nowrap; overflow: hidden; }
  .row.in { opacity: 1; transform: none; }
  .row span { overflow: hidden; text-overflow: ellipsis; }
  .row.sel { outline: 2px solid var(--accent); }
  .row.new { background: color-mix(in srgb, var(--accent) 22%, var(--bg-hover)); }
  .row b { margin-left: auto; font-size: 0.65rem; color: var(--accent); text-transform: uppercase; }
  .view { display: grid; place-items: center; padding: 8px; }
  .pic { position: relative; width: 100%; height: 100%; border-radius: 6px; background: var(--bg-hover); overflow: hidden; transition: background 0.4s; }
  .pic.on { background: linear-gradient(#7fb8e6, #cfe6f5); }
  .sun, .hill { position: absolute; display: block; opacity: 0; transition: opacity 0.4s; }
  .pic.on .sun, .pic.on .hill { opacity: 1; }
  .sun { width: 18%; aspect-ratio: 1; right: 16%; top: 14%; border-radius: 50%; background: #f7d046; }
  .hill { left: -10%; right: -10%; bottom: -25%; height: 60%; border-radius: 50%; background: #4aa05a; }
  .tools { display: flex; flex-direction: column; gap: 6px; padding: 6px; font-size: var(--fs-xs); }
  .tabs, .fmt { display: flex; flex-wrap: wrap; gap: 3px; }
  .tabs span, .fmt span { padding: 2px 6px; border-radius: 4px; background: var(--bg-hover); color: var(--text-muted); font-size: 0.68rem; }
  .tabs span.on, .fmt span.on { background: color-mix(in srgb, var(--accent) 25%, var(--bg-hover)); color: var(--text); }
  .sld { position: relative; height: 6px; margin: 4px 0; border-radius: 3px; background: var(--bg-hover); }
  .sld i { position: absolute; left: 0; top: 0; bottom: 0; width: 62%; border-radius: 3px; background: var(--accent); }
  .sld.s2 i { width: 35%; }
  .inf { position: absolute; right: 0; top: -14px; display: grid; color: var(--text-muted); }
  .go { margin-top: auto; padding: 5px; text-align: center; border-radius: 6px; background: var(--accent); color: var(--on-accent); font-weight: 700; font-size: 0.72rem; transition: transform 0.2s; }
  .go.press { animation: press 1.6s ease-in-out infinite; }
  .bar { position: absolute; left: 8px; right: 8px; bottom: 8px; height: 16px; display: flex; align-items: center; gap: 8px; padding: 0 8px; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: 7px; font-size: 0.62rem; color: var(--text-muted); }
  .bar .track { flex: 1; height: 4px; border-radius: 2px; background: var(--bg-hover); overflow: hidden; }
  .bar .fill { display: block; height: 100%; width: 0; background: var(--accent); }
  .bar.on .fill { animation: fill 2.6s ease-in-out infinite; }

  .pal { position: absolute; left: 50%; top: 14%; transform: translateX(-50%); display: flex; align-items: center; gap: 8px; width: 56%; padding: 8px 10px; background: var(--bg-elevated); border: 1px solid var(--accent); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); font-size: var(--fs-sm); color: var(--text-muted); animation: pop 0.35s ease-out; }
  .pal kbd { margin-left: auto; padding: 1px 6px; border: 1px solid var(--border-strong); border-radius: 4px; font: inherit; font-size: 0.68rem; color: var(--text); }
  .rad { position: absolute; left: 12%; bottom: 20%; width: 0; height: 0; }
  .rad span { position: absolute; left: 0; bottom: 0; display: grid; place-items: center; width: 30px; height: 30px; margin: -15px; border-radius: 50%; background: var(--bg-elevated); border: 1px solid var(--accent); color: var(--text); transform: rotate(var(--a)) translateY(-62px) rotate(calc(var(--a) * -1)); animation: pop 0.35s ease-out backwards; }
  .rad span:nth-child(2) { animation-delay: 0.06s; }
  .rad span:nth-child(3) { animation-delay: 0.12s; }
  .rad span:nth-child(4) { animation-delay: 0.18s; }

  .tr { position: absolute; width: 64px; height: 64px; margin: -64px 0 0 -32px; transition: left 0.9s cubic-bezier(0.45, 0, 0.25, 1), top 0.9s cubic-bezier(0.45, 0, 0.25, 1); pointer-events: none; filter: drop-shadow(0 3px 2px rgba(0, 0, 0, 0.35)); }
  .tr.moving :global(.sprite) { animation: walk 0.3s ease-in-out infinite alternate; }
  .tr.hop :global(.sprite) { animation: hop 1s ease-in-out infinite; }

  .say { margin: 0; padding: 0.8rem 1rem; min-height: 4.4rem; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); font-size: var(--fs-lg); line-height: 1.45; }
  .n { display: inline-block; margin-right: 0.5rem; padding: 0 0.45rem; border-radius: 999px; background: var(--bg-hover); color: var(--text-muted); font-size: var(--fs-xs); font-weight: 600; }
  .nav { display: flex; align-items: center; gap: 0.5rem; }
  .dots { display: flex; gap: 5px; margin: 0 auto; }
  .dots i { width: 7px; height: 7px; border-radius: 50%; background: var(--border-strong); transition: background 0.2s, transform 0.2s; }
  .dots i.on { background: var(--accent); transform: scale(1.3); }

  @keyframes walk { from { transform: translateY(0); } to { transform: translateY(-5px); } }
  @keyframes hop { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-8px); } }
  @keyframes press { 0%, 55%, 100% { transform: scale(1); } 65% { transform: scale(0.93); } 80% { transform: scale(1); } }
  @keyframes fill { from { width: 0; } to { width: 100%; } }
  @keyframes pop { from { opacity: 0; } }
  @media (max-width: 560px) {
    .nav { flex-wrap: wrap; }
    .dots { order: -1; flex: 1 0 100%; justify-content: center; margin-bottom: 0.2rem; }
  }
</style>
