<script lang="ts">
  import { onMount, onDestroy, tick as nextTick } from "svelte";
  import { t } from "../i18n";
  import { tutorialOpen, closeTutorial, uiScale, UI_SCALES, theme, motionReduced, type Theme } from "../settings";
  import Icon from "./Icon.svelte";
  import TrasnoSprite from "./ui/TrasnoSprite.svelte";
  import Segmented from "./ui/Segmented.svelte";
  import ThemePicker from "./ui/ThemePicker.svelte";
  import { DOCK, trasnoAway } from "../agent";

  /** Primeira execución: benvida, tamaño da letra, tema e, se a persoa quere, o titorial. Desde Axustes vai directo aos pasos.
   *  Titorial: unha benvida co trasno en grande; despois os pasos. Os pasos pintan unha copia pequena
   *  da xanela real (mesma estrutura, medidas e textos) e o trasno vai sinalando cada parte. Nada do que se ve toca ficheiros. */
  let phase: "welcome" | "font" | "theme" | "steps" = $tutorialOpen === "welcome" ? "welcome" : "steps";
  let themeV: string = $theme;
  $: theme.set(themeV as Theme);

  type Tab = "convert" | "compress" | "transform" | "restore" | "cutout" | "resize" | "slice" | "annotate" | "icons" | "ocr" | "info";
  interface Step {
    text: string;
    /** Onde para o trasno: xunto a un elemento da maqueta (selector) e por que lado, ou en coordenadas fixas (px do escenario de 1000 x 620, polos pes). */
    target?: string;
    side?: "left" | "right" | "above" | "below";
    at?: [number, number];
    state: "idle" | "working" | "notice";
    size?: number;
    /** Ficheiros na lista (0: a pantalla baleira de «Arrastra os ficheiros aquí»). */
    files: number;
    menu?: boolean;
    hl?: "list" | "rail" | "levels" | "bar";
    level?: 0 | 1 | 2 | 3;
    job?: "run";
    radial?: boolean;
    /** Xanela aberta por riba: descargas ou Axustes. */
    dlg?: "links" | "hub";
  }

  $: steps = [
    { text: $t("Arrastra ficheiros á xanela, preme Escoller ficheiros (Ctrl+O) ou pega con Ctrl+V. Valen imaxes, audio, vídeo, PDF e modelos 3D."), target: ".gbtn", side: "right", state: "notice", files: 0 },
    { text: $t("Preme un ficheiro da lista para velo no visor. Se marcas varios, a columna de ferramentas tamén cambia e ofrece opcións propias para traballar con eles xuntos, como Unir."), target: ".row.sel", side: "right", state: "idle", files: 4, hl: "list" },
    { text: $t("Fai clic co botón dereito nun ficheiro da lista para ver as súas accións: abrir o cartafol onde está, copiar a súa ruta, marcalo para traballar con varios ou quitalo da lista."), target: ".menu", side: "right", state: "idle", files: 4, menu: true },
    { text: $t("As ferramentas están na columna de iconos ao carón do visor. Cada tipo de ficheiro ten as súas propias opcións, así que a columna cambia segundo o que abras: unha imaxe ten Converter, Comprimir ou Redimensionar, e un vídeo, Editar, Son ou Subtítulos."), target: ".rail span:nth-child(2)", side: "left", state: "working", files: 4, hl: "rail" },
    { text: $t("Cada ferramenta ten o seu panel de axustes. Podes elixir un perfil con valores xa preparados ou usar Personalizado para axustalos ti mesmo."), target: ".seg", side: "left", state: "idle", files: 4, hl: "levels", level: 1 },
    { text: $t("O botón verde pon a ferramenta a traballar. A barra de abaixo mostra o progreso, e premendo nela ves a lista de procesos activos."), target: ".jobs", side: "above", state: "working", files: 4, hl: "bar", level: 1, job: "run" },
    { text: $t("Pasa o rato sobre min e sae un menú con funcións como descargar por ligazón ou abrir Axustes, e outras como engadir ficheiros, ver os procesos ou buscar."), at: [64, 606], state: "idle", size: 96, files: 4, radial: true, level: 1 },
    { text: $t("Descargar por ligazón baixa vídeo ou audio dunha ligazón co formato e a calidade que elixas, ou unha lista enteira. A pestana Torrents acepta ligazóns magnet e ficheiros .torrent, e deixa marcar os ficheiros que queres."), target: ".dlg", side: "right", state: "idle", files: 4, level: 1, dlg: "links" },
    { text: $t("En Axustes cambias o idioma e o tema, escolles onde se gardan os resultados, defines atallos e perfís de saída e instalas as extensións opcionais."), target: ".dlg", side: "right", state: "idle", files: 4, level: 1, dlg: "hub" },
  ] as Step[];

  const FILES = [
    { name: "foto.png", sub: "PNG · 2.4 MB", icon: "image", kind: "PNG", meta: "2.4 MB · 1600 × 1067", to: "JPG", rail: ["convert", "compress", "scaling", "crop", "cutout", "info"] },
    { name: "informe.pdf", sub: "PDF · 860 KB", icon: "document", kind: "PDF", meta: "860 KB", to: "DOCX", rail: ["convert", "compress", "pages", "merge", "lock", "scan-text"] },
    { name: "cancion.mp3", sub: "MP3 · 3.1 MB", icon: "audio", kind: "MP3", meta: "3.1 MB · 3:24", to: "OGG", rail: ["convert", "compress", "volume", "tracks", "info"] },
    { name: "video.mp4", sub: "MP4 · 18 MB", icon: "video", kind: "MP4", meta: "18 MB · 1920 × 1080", to: "WEBM", rail: ["convert", "compress", "slice", "volume", "tracks", "info"] },
  ];
  /** Con varios marcados a columna de ferramentas ofrece as de traballar con eles xuntos. */
  const MULTI_RAIL = ["merge", "convert", "compress"];
  const LEVELS = [
    { n: "WhatsApp", v: "55 %" },
    { n: "Web", v: "75 %" },
    { n: "Arquivo", v: "95 %" },
    { n: "Personalizado", v: "" },
  ];
  const HUB = [
    { g: "Procesos", icon: "layers", items: ["En marcha", "Historial"] },
    { g: "Axustes", icon: "gear", items: ["Xeral", "Atallos de teclado", "Perfís de saída", "Extensións"] },
    { g: "Novidades", icon: "info", items: ["Rexistro de cambios", "Actualizacións"] },
  ];
  /** Páxinas que vai percorrendo o paso de Axustes, co aspecto do seu contido (campos, lista, interruptores ou texto). */
  const HUB_TOUR: { item: string; group: string; body: "fields" | "list" | "toggles" | "text" }[] = [
    { item: "Xeral", group: "Axustes", body: "fields" },
    { item: "Atallos de teclado", group: "Axustes", body: "list" },
    { item: "Perfís de saída", group: "Axustes", body: "list" },
    { item: "Extensións", group: "Axustes", body: "toggles" },
    { item: "Rexistro de cambios", group: "Novidades", body: "text" },
    { item: "Actualizacións", group: "Novidades", body: "toggles" },
  ];
  // Os botóns do menú son os mesmos da app (DOCK): mesma lista, mesmas posicións.
  const RADIAL = DOCK.map((d) => ({ icon: d.icon, dx: Math.round(d.r * Math.cos((d.a * Math.PI) / 180)), dy: -Math.round(d.r * Math.sin((d.a * Math.PI) / 180)) }));

  let i = 0;
  $: step = steps[i];
  $: last = i === steps.length - 1;
  let moving = false;
  let timer: ReturnType<typeof setTimeout> | undefined;

  // Cada paso ten a súa pequena animación: un «latexo» que sobe cada pouco e a maqueta cambia con el (ficheiro elixido, ferramenta, nivel, páxina…).
  let beat = 0;
  let beatTimer: ReturnType<typeof setInterval> | undefined;
  /** Ficheiro aberto: nos pasos da lista e das ferramentas vai percorrendo os catro; a partir do latexo 4 do paso 2 hai varios marcados. */
  $: sel = step.hl === "list" ? (beat % 6 < 4 ? beat % 6 : 0) : step.hl === "rail" ? beat % 4 : 0;
  $: multi = step.hl === "list" && beat % 6 >= 4;
  $: file = FILES[sel];
  $: rail = multi ? MULTI_RAIL : file.rail;
  $: level = step.hl === "levels" ? beat % 4 : (step.level ?? 3);
  $: menuHot = step.menu ? beat % 4 : -1;
  // O cursor chega ata o trasno, sae o menú e vai pasando por cada botón.
  $: radialOn = !!step.radial && beat >= 1;
  $: hot = step.radial && beat >= 2 ? (beat - 2) % RADIAL.length : -1;
  $: cur = !step.radial ? null : beat < 1 ? { x: 420, y: 300 } : hot < 0 ? { x: 70, y: 556 } : { x: 64 + RADIAL[hot].dx + 6, y: H - 57 + RADIAL[hot].dy + 6 };
  $: lb = step.dlg === "links" ? beat % 6 : 0;
  $: tour = HUB_TOUR[step.dlg === "hub" ? beat % HUB_TOUR.length : 0];
  function go(to: number) {
    if (to < 0 || to >= steps.length || to === i) return;
    i = to;
    beat = 0;
    moving = true;
    clearTimeout(timer);
    timer = setTimeout(() => (moving = false), 950);
  }
  const next = () => (last ? closeTutorial() : go(i + 1));

  // O progreso da barra de abaixo sube e volve empezar mentres dura o paso.
  let pct = 8;
  let tick: ReturnType<typeof setInterval> | undefined;
  $: if (step.job === "run") {
    if (!tick) tick = setInterval(() => (pct = pct >= 100 ? 8 : Math.min(100, pct + 2)), 260);
  } else if (tick) {
    clearInterval(tick);
    tick = undefined;
    pct = 8;
  }

  // O trasno para xunto ao que explica o paso: mide o elemento na maqueta (xa escalada) e colocase ao seu lado.
  let appEl: HTMLElement;
  let pos = { x: 500, y: 300 };
  async function place() {
    await nextTick();
    const s = step;
    const size = s.size ?? 64;
    if (s.at) return void (pos = { x: s.at[0], y: s.at[1] });
    const el = s.target ? appEl?.querySelector(s.target) : null;
    if (!el) return;
    const a = appEl.getBoundingClientRect();
    const r = el.getBoundingClientRect();
    const L = (r.left - a.left) / k, T = (r.top - a.top) / k, R = (r.right - a.left) / k, B = (r.bottom - a.top) / k;
    const gap = 8;
    let x = (L + R) / 2;
    let y = (T + B) / 2 + size / 2;
    if (s.side === "left") x = L - gap - size / 2;
    else if (s.side === "right") x = R + gap + size / 2;
    else if (s.side === "below") y = B + gap + size;
    else y = T - gap;
    pos = { x: Math.max(size / 2, Math.min(W - size / 2, x)), y: Math.max(size, Math.min(H - 4, y)) };
  }
  $: if (appEl) {
    step;
    k;
    place();
  }

  // O escenario ten sempre as medidas da xanela real e escálase para caber.
  const W = 1000;
  const H = 620;
  let boxW = W;
  let winH = 800;
  $: k = Math.max(0.3, Math.min(1, boxW / W, (winH - 225) / H));

  let panel: HTMLElement;
  let before: HTMLElement | null = null;
  onMount(() => {
    before = document.activeElement as HTMLElement | null;
    trasnoAway.set(true); // o trasno real agacha: só pode haber un en pantalla, o do titorial
    panel.querySelector<HTMLElement>("button.primary")?.focus();
    beatTimer = setInterval(() => {
      if (phase === "steps" && !$motionReduced) beat++;
    }, 1400);
  });
  onDestroy(() => {
    clearTimeout(timer);
    clearInterval(tick);
    clearInterval(beatTimer);
    trasnoAway.set(false);
    before?.focus?.();
  });

  function to(next: typeof phase) {
    phase = next;
    beat = 0;
    requestAnimationFrame(() => panel?.querySelector<HTMLElement>("button.primary")?.focus());
  }

  function key(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      closeTutorial();
    } else if (phase === "steps" && e.key === "ArrowRight") {
      e.preventDefault();
      next();
    } else if (phase === "steps" && e.key === "ArrowLeft") {
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

<svelte:window on:keydown={key} bind:innerHeight={winH} />

<div class="veil" role="presentation">
  {#if phase === "welcome"}
    <div class="panel welcome" role="dialog" aria-modal="true" aria-labelledby="tut-title" bind:this={panel}>
      <div class="big" aria-hidden="true"><TrasnoSprite state="idle" size={192} /></div>
      <h2 id="tut-title">{$t("Arquivos do Trasno")}</h2>
      <p>{$t("Ola! Son o trasno e vivo nesta app. Convirto, comprimo e arranxo imaxes, audio, vídeo, PDF e modelos 3D: ti arrastras os ficheiros e eu póñome ao traballo.")}</p>
      <div class="wbtn">
        <button type="button" class="btn ghost" on:click={closeTutorial}>{$t("Agora non")}</button>
        <button type="button" class="btn primary" on:click={() => to("font")}>{$t("Comezar")}</button>
      </div>
    </div>
  {:else if phase === "font"}
    <div class="panel welcome" role="dialog" aria-modal="true" aria-labelledby="tut-title" bind:this={panel}>
      <div class="big sm" aria-hidden="true"><TrasnoSprite state="idle" size={96} /></div>
      <h2 id="tut-title">{$t("Tamaño da letra")}</h2>
      <p>{$t("Primeiro, o tamaño da letra: elixe o que che resulte máis cómodo de ler. Despois podes cambialo en Axustes.")}</p>
      <div class="opt"><Segmented label={$t("Tamaño da interface")} options={UI_SCALES.map((v) => ({ value: v, label: `${v} %` }))} bind:value={$uiScale} /></div>
      <p class="sample">{$t("Así se ve o texto da app.")}</p>
      <div class="wbtn">
        <button type="button" class="btn" on:click={() => to("welcome")}>{$t("Anterior")}</button>
        <button type="button" class="btn primary" on:click={() => to("theme")}>{$t("Seguinte")}</button>
      </div>
    </div>
  {:else if phase === "theme"}
    <div class="panel welcome" role="dialog" aria-modal="true" aria-labelledby="tut-title" bind:this={panel}>
      <div class="big sm" aria-hidden="true"><TrasnoSprite state="idle" size={96} /></div>
      <h2 id="tut-title">{$t("Tema")}</h2>
      <p>{$t("Agora o tema: claro, escuro ou o do sistema. Tamén o podes cambiar despois en Axustes.")}</p>
      <div class="opt"><ThemePicker bind:value={themeV} /></div>
      <div class="wbtn">
        <button type="button" class="btn ghost" on:click={closeTutorial}>{$t("Agora non")}</button>
        <button type="button" class="btn" on:click={() => to("font")}>{$t("Anterior")}</button>
        <button type="button" class="btn primary" on:click={() => to("steps")}>{$t("Ver o titorial")}</button>
      </div>
    </div>
  {:else}
    <div class="panel" role="dialog" aria-modal="true" aria-labelledby="tut-title" bind:this={panel}>
      <h2 id="tut-title" class="sr-only">{$t("Titorial")}</h2>

      <div class="wrap" bind:clientWidth={boxW} aria-hidden="true">
        <div class="stage" style={`width:${W * k}px;height:${H * k}px`}>
          <div class="app" bind:this={appEl} style={`transform:scale(${k})`}>
            <div class="main">
              {#if step.files === 0}
                <div class="empty">
                  <div class="zone">
                    <b>{$t("Arrastra os ficheiros aquí")}</b>
                    <span class="gbtn"><Icon name="folder" size={16} />{$t("Escoller ficheiros")}</span>
                    <small>PNG · JPG · WEBP · GIF · MP3 · WAV · FLAC · MP4 · MKV · WEBM · PDF · TXT · FBX · GLB · OBJ · STL · PLY</small>
                  </div>
                  <div class="drag"><Icon name="image" size={14} /> foto.png</div>
                </div>
              {:else}
                <div class="list" class:hl={step.hl === "list"}>
                  <div class="lh"><span>{multi ? $t("{n} {n:marcado|marcados}", { n: 2 }) : `${$t("Ficheiros")} · ${step.files}`}</span><span class="add"><Icon name="plus" size={16} />{$t("Engadir")}</span></div>
                  {#each FILES as f, n (f.name)}
                    <div class="row" class:sel={n === sel} style={`--d:${n * 110}ms`}>
                      <span class="cb" class:on={multi && n < 2}>{#if multi && n < 2}<Icon name="check" size={12} />{/if}</span>
                      <span class="ficon"><Icon name={f.icon} size={16} /></span>
                      <span class="fn"><b>{f.name}</b><small>{f.sub}</small></span>
                    </div>
                  {/each}
                </div>
                <div class="view">
                  <div class="vh"><Icon name={file.icon} size={16} /><b>{file.kind}</b><span>· {file.meta}</span></div>
                  <div class="vb">
                    {#key sel}
                      {#if sel === 0}
                        <div class="pic"><i class="sun"></i><i class="hill"></i></div>
                      {:else if sel === 1}
                        <div class="doc"><b></b><i></i><i></i><i class="s"></i><i></i><i class="s"></i></div>
                      {:else if sel === 2}
                        <div class="wave">{#each [30, 55, 80, 45, 95, 60, 35, 75, 50, 85, 40, 65, 30, 70, 45] as h}<i style={`height:${h}%`}></i>{/each}</div>
                      {:else}
                        <div class="pic vid"><i class="sun"></i><i class="hill"></i><span class="play"><Icon name="play" size={26} /></span></div>
                      {/if}
                    {/key}
                  </div>
                </div>
                <div class="rail" class:hl={step.hl === "rail"}>
                  {#each rail as ic, n (ic + sel + multi)}<span class:on={n === 0}><Icon name={ic} size={20} /></span>{/each}
                </div>
                <div class="side">
                  <div class="sh"><b>{multi ? $t("Unir") : $t("Converter")}</b></div>
                  <span class="lbl">{$t("Formato de saída")}</span>
                  <div class="dd">{file.to}<Icon name="chevron-down" size={16} /></div>
                  <span class="lbl">{$t("Nivel de calidade")}</span>
                  <div class="seg" class:hl={step.hl === "levels"}>
                    {#each LEVELS as l, n}<span class:on={n === level}><b>{l.n}</b>{#if l.v}<small>{l.v}</small>{/if}</span>{/each}
                  </div>
                  <span class="lbl">{$t("Calidade")}</span>
                  <div class="sl"><span class="track"><i style={`width:${level === 1 ? 75 : level === 0 ? 55 : level === 2 ? 95 : 85}%`}></i><u style={`left:${level === 1 ? 75 : level === 0 ? 55 : level === 2 ? 95 : 85}%`}></u></span><span class="num">{level === 1 ? 75 : level === 0 ? 55 : level === 2 ? 95 : 85} %</span></div>
                  <div class="gobtn" class:off={step.job === "run"}>{multi ? $t("Unir") : `${$t("Converter")} → ${file.to}`}</div>
                </div>
              {/if}
            </div>

            <div class="status">
              <span></span>
              {#if step.job === "run"}
                <div class="jobs" class:hl={step.hl === "bar"}>
                  <span class="jfill" style={`width:${pct}%`}></span>
                  <span class="jtext"><b>foto.png</b><em>{pct} %</em></span>
                  <span class="jx"><Icon name="close" size={14} /></span>
                </div>
              {:else}
                <span></span>
              {/if}
              <span></span>
            </div>

            {#if step.menu}
              <div class="menu" style="left:150px;top:96px">
                <span class:hot={menuHot === 0}><Icon name="folder" size={14} />{$t("Amosar no cartafol")}</span>
                <span class:hot={menuHot === 1}><Icon name="copy" size={14} />{$t("Copiar ruta")}</span>
                <span class:hot={menuHot === 2}><Icon name="check" size={14} />{$t("Marcar")}</span>
                <span class="sep" class:hot={menuHot === 3}><Icon name="close" size={14} />{$t("Quitar da lista")}</span>
              </div>
            {/if}
            {#if step.dlg}
              <div class="veil2"></div>
              {#if step.dlg === "links"}
                <div class="dlg">
                  <div class="dh"><b>{$t("Descargar por ligazón")}</b><Icon name="close" size={15} /></div>
                  <div class="db">
                    <div class="seg2"><span class:on={lb < 3}><Icon name="link" size={16} />{$t("Ligazóns")}</span><span class:on={lb >= 3}><Icon name="magnet" size={16} />{$t("Torrents")}</span></div>
                    <span class="dl">{$t("Ligazón")}</span>
                    <div class="urow"><span class="inp" class:on={lb % 3 >= 1}>{lb < 3 ? (lb >= 1 ? "https://…/video" : "https://…") : lb >= 4 ? "magnet:?xt=…" : "magnet:?"}</span><span class="bt" class:hot={lb === 2 || lb === 4}>{$t("Buscar")}</span></div>
                    {#if lb === 2}
                      <div class="found"><i class="th"></i><span class="ft"><b class="ln"></b><b class="ln s"></b></span></div>
                    {:else if lb >= 4}
                      <div class="found fl">
                        {#each [0, 1, 2] as n}<div class="fr"><span class="cb" class:on={n < 2}>{#if n < 2}<Icon name="check" size={12} />{/if}</span><b class="ln" style={`width:${[62, 48, 70][n]}%`}></b></div>{/each}
                      </div>
                    {/if}
                    <div class="gobtn" class:hot={lb === 5}>{$t("Descargar")}</div>
                  </div>
                </div>
              {:else}
                <div class="dlg hub">
                  <div class="dh"><b>Arquivos do Trasno</b><Icon name="close" size={15} /></div>
                  <div class="split">
                    <div class="nv">
                      {#each HUB as g}
                        <div class="gh"><Icon name={g.icon} size={14} />{$t(g.g)}</div>
                        {#each g.items as it}<span class:on={it === tour.item}>{$t(it)}</span>{/each}
                      {/each}
                    </div>
                    <div class="pn">
                      <b class="pt">{$t(tour.group)} · {$t(tour.item)}</b>
                      {#key tour.item}
                        <div class="pb">
                          {#if tour.body === "fields"}
                            {#each [0, 1, 2] as _}<b class="ln w"></b><i class="fc"></i>{/each}
                          {:else if tour.body === "list"}
                            {#each [0, 1, 2, 3, 4] as _}<div class="lr"><b class="ln"></b><u class="chip"></u></div>{/each}
                          {:else if tour.body === "toggles"}
                            {#each [0, 1, 2] as _, n}<div class="lr"><b class="ln"></b><u class="tg" class:on={n !== 1}></u></div>{/each}
                          {:else}
                            {#each [0, 1] as _}<b class="ln hd"></b><b class="ln"></b><b class="ln s"></b><b class="ln"></b>{/each}
                          {/if}
                        </div>
                      {/key}
                    </div>
                  </div>
                </div>
              {/if}
            {/if}
            {#if radialOn}
              <div class="rad" style={`left:64px;top:${H - 57}px`}>
                {#each RADIAL as r, n}<span class:hot={n === hot} style={`--dx:${r.dx}px;--dy:${r.dy}px;--n:${n}`}><Icon name={r.icon} size={22} /></span>{/each}
              </div>
            {/if}

            <div class="tr" class:moving style={`left:${pos.x}px;top:${pos.y}px;--s:${step.size ?? 64}px`}>
              <TrasnoSprite state={step.state} size={step.size ?? 64} />
            </div>
            {#if cur}
              <svg class="cur" viewBox="0 0 24 24" width="26" height="26" style={`left:${cur.x}px;top:${cur.y}px`}><path d="M4 2l15 9-6.5 1.6L9.4 19z" fill="#fff" stroke="#111" stroke-width="1.6" stroke-linejoin="round" /></svg>
            {/if}
          </div>
        </div>
      </div>

      <p class="say" aria-live="polite"><span class="n">{i + 1} / {steps.length}</span> {step.text}</p>

      <div class="nav">
        <button type="button" class="btn ghost" on:click={closeTutorial}>{last ? $t("Pechar") : $t("Saltar")}</button>
        <span class="dots" aria-hidden="true">{#each steps as _, n}<i class:on={n === i}></i>{/each}</span>
        <span class="go">
          <button type="button" class="btn" on:click={() => go(i - 1)} disabled={i === 0}>{$t("Anterior")}</button>
          <button type="button" class="btn primary" on:click={next}>{last ? $t("Rematar") : $t("Seguinte")}</button>
        </span>
      </div>
    </div>
  {/if}
</div>

<style>
  .veil { position: fixed; inset: 0; z-index: 70; display: flex; align-items: center; justify-content: center; padding: 1rem; background: rgba(0, 0, 0, 0.6); }
  .panel { display: flex; flex-direction: column; gap: 0.9rem; width: min(1060px, 100%); max-height: 100%; overflow: auto; padding: 1rem; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: calc(var(--radius-lg) + 2px); box-shadow: var(--shadow-pop); }

  /* Benvida */
  .welcome { width: min(460px, 100%); align-items: center; gap: 0.6rem; padding: 1.6rem 1.4rem 1.3rem; text-align: center; }
  .big { display: grid; place-items: center; width: 192px; height: 192px; margin-bottom: 0.2rem; }
  .welcome h2 { margin: 0; font-size: var(--fs-xl); }
  .welcome p { margin: 0 0 0.8rem; max-width: 24rem; color: var(--text-muted); font-size: var(--fs-lg); line-height: 1.5; }
  .wbtn { display: flex; flex-wrap: wrap; justify-content: center; gap: 0.5rem; }
  .big.sm { width: 96px; height: 96px; }
  .opt { width: 100%; margin: 0.2rem 0 0.4rem; text-align: left; }
  .welcome p.sample { margin: 0 0 0.6rem; color: var(--text); font-size: var(--fs-md); }

  /* Escenario: copia pequena da xanela real (1000 × 620). Cores, tamaños e textos son os da app. */
  .wrap { display: grid; justify-items: center; }
  .stage { position: relative; overflow: hidden; border: 1px solid var(--border); border-radius: var(--radius-md); }
  .app { position: absolute; left: 0; top: 0; width: 1000px; height: 620px; transform-origin: top left; display: flex; flex-direction: column; background: var(--bg); color: var(--text); }
  .main { position: relative; flex: 1; display: flex; min-height: 0; }
  .hl { outline: 2px solid var(--accent); outline-offset: -2px; z-index: 2; }

  .empty { position: absolute; inset: 14px; display: grid; place-items: center; border: 1px dashed var(--border-strong); border-radius: var(--radius-lg); background: radial-gradient(ellipse at 50% 45%, color-mix(in srgb, var(--accent) 7%, transparent), transparent 62%); }
  .zone { display: flex; flex-direction: column; align-items: center; gap: 12px; margin-top: 60px; }
  .zone b { font-size: var(--fs-xl); }
  .gbtn { display: inline-flex; align-items: center; gap: 8px; height: 38px; padding: 0 14px; border-radius: var(--radius-sm); background: var(--accent); color: var(--on-accent); font-weight: var(--fw-strong); font-size: var(--fs-md); }
  .zone small { max-width: 420px; text-align: center; color: var(--text-muted); font-size: var(--fs-xs); line-height: 1.5; }
  .drag { position: absolute; left: 130px; top: 130px; display: flex; align-items: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--accent); border-radius: var(--radius-sm); background: var(--bg-elevated); font-size: var(--fs-sm); animation: drop 3.2s ease-in-out infinite; }

  .list { flex: 0 0 230px; display: flex; flex-direction: column; background: var(--bg-rail); border-right: 1px solid var(--border); }
  .lh { text-transform: uppercase; display: flex; align-items: center; justify-content: space-between; height: 58px; padding: 0 10px 0 14px; border-bottom: 1px solid var(--border); font-size: var(--fs-xs); font-weight: var(--fw-strong); letter-spacing: 0.06em; color: var(--text-muted); }
  .add { display: inline-flex; align-items: center; gap: 6px; height: 36px; padding: 0 12px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); font-size: var(--fs-md); font-weight: var(--fw-medium); letter-spacing: 0; text-transform: none; color: var(--text); }
  .row { display: flex; align-items: center; gap: 10px; height: 68px; padding: 0 12px 0 16px; border-left: 3px solid transparent; animation: in 0.4s var(--d, 0ms) backwards; }
  .row.sel { background: color-mix(in srgb, var(--accent) 12%, var(--bg-rail)); border-left-color: var(--accent); }
  .cb { flex: none; display: grid; place-items: center; width: 15px; height: 15px; border: 1px solid var(--border-strong); border-radius: 3px; background: var(--bg-input); color: var(--on-accent); transition: background 0.2s, border-color 0.2s; }
  .cb.on { background: var(--accent); border-color: var(--accent); }
  .row { transition: background 0.3s, border-color 0.3s; }
  .doc { display: flex; flex-direction: column; gap: 9px; width: 44%; aspect-ratio: 3 / 4; padding: 7% 8%; border-radius: 3px; background: #f5f5f2; box-shadow: 0 2px 10px rgba(0, 0, 0, 0.35); animation: in 0.35s ease-out; }
  .doc b { display: block; width: 45%; height: 12px; border-radius: 2px; background: #8d8d88; }
  .doc i { display: block; height: 6px; border-radius: 2px; background: #c9c9c4; }
  .doc i.s { width: 70%; }
  .wave { display: flex; align-items: center; gap: 7px; width: 70%; height: 46%; padding: 0 5%; border-radius: var(--radius-md); background: var(--bg-rail); animation: in 0.35s ease-out; }
  .wave i { flex: 1; border-radius: 3px; background: var(--accent); opacity: 0.85; }
  .vid { filter: saturate(0.85); }
  .play { position: absolute; left: 50%; top: 50%; display: grid; place-items: center; width: 54px; height: 54px; margin: -27px 0 0 -27px; border-radius: 50%; background: rgba(0, 0, 0, 0.55); color: #fff; }
  .pic { animation: in 0.35s ease-out; }
  .ficon { flex: none; display: grid; place-items: center; width: 22px; color: var(--text-muted); }
  .fn { min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .fn b { font-weight: var(--fw-medium); font-size: var(--fs-md); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .fn small { color: var(--text-muted); font-size: var(--fs-xs); }

  .view { flex: 1; min-width: 0; display: flex; flex-direction: column; background: var(--viewer-bg); }
  .vh { display: flex; align-items: center; gap: 8px; height: 42px; padding: 0 14px; background: var(--bg-rail); border-bottom: 1px solid var(--border); font-size: var(--fs-sm); color: var(--text-muted); }
  .vh b { color: var(--text); }
  .vb { flex: 1; display: grid; place-items: center; background: repeating-conic-gradient(var(--checker-a) 0 25%, var(--checker-b) 0 50%) 0 0 / 24px 24px; }
  .pic { position: relative; width: 80%; aspect-ratio: 3 / 2; border-radius: 3px; background: linear-gradient(#7fb8e6, #cfe6f5); overflow: hidden; }
  .sun { position: absolute; right: 14%; top: 14%; width: 16%; aspect-ratio: 1; border-radius: 50%; background: #f7d046; }
  .hill { position: absolute; left: -10%; right: -10%; bottom: -28%; height: 60%; border-radius: 50%; background: #4aa05a; }

  .rail { flex: 0 0 54px; display: flex; flex-direction: column; overflow: hidden; background: var(--bg-rail); border-left: 1px solid var(--border); border-right: 1px solid var(--border); }
  .rail span { flex: none; display: grid; place-items: center; height: 54px; border-left: 2px solid transparent; color: var(--text-muted); }
  .rail span.on { color: var(--text); background: color-mix(in srgb, var(--accent) 10%, var(--bg-rail)); border-left-color: var(--accent); }

  .side { flex: 0 0 330px; display: flex; flex-direction: column; gap: 6px; padding: 14px 16px; background: var(--bg-rail); }
  .sh { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; font-size: var(--fs-lg); }
  .side .lbl { margin-top: 6px; color: var(--text-muted); font-size: var(--fs-sm); }
  .dd { display: flex; align-items: center; justify-content: space-between; height: 38px; padding: 0 12px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--bg-input); font-size: var(--fs-md); }
  .seg { display: grid; grid-template-columns: repeat(4, 1fr); border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-input); overflow: hidden; }
  .seg span { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 2px; height: 52px; color: var(--text-muted); font-size: var(--fs-sm); text-align: center; }
  .seg span b { font-weight: var(--fw-medium); }
  .seg span small { font-size: var(--fs-xs); opacity: 0.8; }
  .seg span.on { background: var(--bg-hover); color: var(--text); border-radius: var(--radius-md); }
  .seg span.on b { font-weight: var(--fw-strong); }
  .sl { display: flex; align-items: center; gap: 12px; }
  .track { position: relative; flex: 1; height: 5px; border-radius: 3px; background: var(--bg-hover); }
  .track i { position: absolute; left: 0; top: 0; bottom: 0; border-radius: 3px; background: var(--accent); }
  .track u { position: absolute; top: -8px; width: 8px; height: 21px; margin-left: -4px; border-radius: 2px; background: var(--text); }
  .num { display: grid; place-items: center; width: 62px; height: 38px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--bg-input); font-size: var(--fs-md); }
  .gobtn { display: grid; place-items: center; height: 38px; margin: 12px 0 8px; border-radius: var(--radius-sm); background: var(--accent); color: var(--on-accent); font-weight: var(--fw-strong); font-size: var(--fs-md); }
  .gobtn.off { opacity: 0.45; }

  .status { flex: 0 0 44px; display: grid; grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr); align-items: center; gap: 12px; padding: 0 14px 0 160px; background: var(--bg-rail); border-top: 1px solid var(--border); font-size: var(--fs-sm); }
  .jobs { position: relative; display: flex; align-items: stretch; width: 400px; height: 32px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); overflow: hidden; }
  .jfill { position: absolute; left: 0; top: 0; bottom: 0; background: color-mix(in srgb, var(--accent) 30%, transparent); border-right: 2px solid var(--accent); transition: width 0.26s linear; }
  .jtext { position: relative; flex: 1; display: flex; align-items: center; justify-content: space-between; padding: 0 12px; }
  .jtext em { font-style: normal; color: var(--text-muted); font-variant-numeric: tabular-nums; }
  .jx { position: relative; display: grid; place-items: center; width: 34px; border-left: 1px solid var(--border); color: var(--text-muted); }

  .menu { position: absolute; z-index: 5; width: 206px; padding: 6px; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-md); box-shadow: var(--shadow-pop); animation: pop 0.25s ease-out; }
  .menu span { display: flex; align-items: center; gap: 10px; height: 34px; padding: 0 10px; border-radius: var(--radius-sm); font-size: var(--fs-md); }
  .menu span.sep { margin-top: 4px; border-top: 1px solid var(--border); border-radius: 0 0 var(--radius-sm) var(--radius-sm); }
  .menu span:first-child { background: var(--bg-hover); }
  .veil2 { position: absolute; inset: 0; z-index: 4; background: rgba(8, 9, 12, 0.6); animation: pop 0.25s ease-out; }
  .dlg { position: absolute; z-index: 5; left: 50%; top: 46%; width: 480px; transform: translate(-50%, -50%); display: flex; flex-direction: column; background: var(--bg-elevated); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); box-shadow: var(--shadow-pop); animation: pop 0.25s ease-out; }
  .dlg.hub { width: 680px; }
  .dh { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px; border-bottom: 1px solid var(--border); font-size: var(--fs-lg); color: var(--text); }
  .dh :global(svg) { color: var(--text-muted); }
  .db { display: flex; flex-direction: column; gap: 10px; padding: 18px; }
  .seg2 { display: grid; grid-template-columns: 1fr 1fr; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-input); overflow: hidden; }
  .seg2 span { display: flex; align-items: center; justify-content: center; gap: 8px; height: 40px; color: var(--text-muted); font-size: var(--fs-md); }
  .seg2 span.on { background: var(--bg-hover); color: var(--text); font-weight: var(--fw-strong); border-radius: var(--radius-md); }
  .dl { color: var(--text-muted); font-size: var(--fs-sm); }
  .urow { display: flex; gap: 10px; }
  .inp { flex: 1; display: flex; align-items: center; height: 40px; padding: 0 12px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--bg-input); color: var(--text-muted); font-size: var(--fs-md); }
  .bt { display: grid; place-items: center; height: 40px; padding: 0 18px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); font-size: var(--fs-md); font-weight: var(--fw-medium); }
  .dlg .gobtn { margin: 6px 0 0; transition: box-shadow 0.25s; }
  .gobtn.hot, .bt.hot { box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 45%, transparent); }
  .bt.hot { border-color: var(--accent); }
  .inp.on { color: var(--text); }
  .seg2 span { transition: background 0.25s, color 0.25s; }
  .found { display: flex; align-items: center; gap: 12px; padding: 10px; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); animation: in 0.35s ease-out; }
  .found.fl { flex-direction: column; align-items: stretch; gap: 10px; }
  .fr { display: flex; align-items: center; gap: 10px; }
  .th { flex: none; width: 76px; height: 44px; border-radius: var(--radius-sm); background: linear-gradient(#7fb8e6, #cfe6f5); }
  .ft { flex: 1; display: flex; flex-direction: column; gap: 8px; }
  .ln { display: block; height: 8px; border-radius: 4px; background: var(--bg-hover); }
  .ln.s { width: 55%; }
  .ln.w { width: 40%; height: 9px; margin-top: 6px; background: var(--border-strong); }
  .ln.hd { width: 38%; height: 11px; margin-top: 6px; background: var(--border-strong); }
  .ft .ln { width: 80%; }
  .ft .ln.s { width: 50%; }
  .pb { display: flex; flex-direction: column; gap: 10px; animation: in 0.35s ease-out; }
  .lr { display: flex; align-items: center; justify-content: space-between; gap: 14px; height: 34px; padding: 0 12px; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); }
  .lr .ln { width: 52%; }
  .chip { display: block; width: 74px; height: 20px; border-radius: 6px; background: var(--bg-hover); border: 1px solid var(--border-strong); }
  .tg { display: block; width: 34px; height: 18px; border-radius: 9px; background: var(--bg-hover); border: 1px solid var(--border-strong); }
  .tg.on { background: var(--accent); border-color: var(--accent); }
  .nv span { transition: background 0.25s, color 0.25s; }
  .menu span.hot { background: var(--bg-hover); color: var(--text); }
  .split { display: flex; gap: 16px; padding: 16px; height: 350px; }
  .nv { flex: 0 0 190px; display: flex; flex-direction: column; gap: 2px; }
  .nv .gh { display: flex; align-items: center; gap: 8px; margin-top: 8px; padding: 2px 10px; color: var(--text-faint); font-size: var(--fs-xs); font-weight: var(--fw-strong); letter-spacing: 0.05em; text-transform: uppercase; }
  .nv .gh:first-child { margin-top: 0; }
  .nv span { padding: 6px 10px 6px 30px; border-radius: var(--radius-sm); color: var(--text-muted); font-size: var(--fs-md); font-weight: var(--fw-medium); }
  .nv span.on { background: color-mix(in srgb, var(--accent) 18%, transparent); color: var(--text); font-weight: var(--fw-strong); }
  .pn { flex: 1; display: flex; flex-direction: column; gap: 8px; }
  .pt { margin-bottom: 4px; font-size: var(--fs-lg); }
  .fc { display: block; height: 38px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--bg-input); }
  .cur { position: absolute; z-index: 7; margin: -3px 0 0 -3px; filter: drop-shadow(0 2px 2px rgba(0, 0, 0, 0.5)); transition: left 0.8s cubic-bezier(0.45, 0, 0.25, 1), top 0.8s cubic-bezier(0.45, 0, 0.25, 1); pointer-events: none; }
  .rad span.hot { color: var(--accent); border-color: var(--accent); transform: translate(var(--dx), var(--dy)) scale(1.1); }
  .rad span { transition: transform 0.2s, color 0.2s, border-color 0.2s; }
  .rad { position: absolute; z-index: 4; width: 0; height: 0; }
  .rad span { position: absolute; display: grid; place-items: center; width: 52px; height: 52px; margin: -26px; border-radius: 50%; color: var(--text-muted); background: radial-gradient(circle at 32% 24%, color-mix(in srgb, var(--text-muted) 14%, var(--bg-elevated)), var(--bg-elevated) 74%); border: 1px solid var(--border-strong); box-shadow: var(--shadow-pop); transform: translate(var(--dx), var(--dy)); animation: pop 0.3s calc(var(--n) * 50ms) ease-out backwards; }

  .tr { position: absolute; z-index: 6; width: var(--s); height: var(--s); margin: calc(var(--s) * -1) 0 0 calc(var(--s) / -2); transition: left 0.9s cubic-bezier(0.45, 0, 0.25, 1), top 0.9s cubic-bezier(0.45, 0, 0.25, 1); pointer-events: none; }
  .tr.moving :global(.sprite) { animation: walk 0.3s ease-in-out infinite alternate; }

  .say { margin: 0; padding: 0.8rem 1rem; min-height: 4.4rem; background: var(--bg-card); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); font-size: var(--fs-lg); line-height: 1.45; }
  .n { margin-right: 0.6rem; color: var(--text-muted); font-size: var(--fs-sm); font-weight: var(--fw-strong); font-variant-numeric: tabular-nums; }
  .nav { display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; gap: 0.5rem; }
  .nav .btn.ghost { justify-self: start; }
  .go { justify-self: end; display: flex; gap: 0.5rem; }
  .dots { display: flex; align-items: center; justify-content: center; gap: 6px; }
  .dots i { display: block; width: 8px; height: 8px; border-radius: 50%; background: var(--border-strong); transition: background 0.2s; }
  .dots i.on { background: var(--accent); }

  @keyframes walk { from { transform: translateY(0); } to { transform: translateY(-5px); } }
  @keyframes in { from { opacity: 0; } }
  @keyframes pop { from { opacity: 0; } }
  @keyframes drop { 0% { transform: translate(0, 0); opacity: 0; } 15% { opacity: 1; } 70%, 100% { transform: translate(300px, 150px); opacity: 0; } }
  @media (max-width: 640px) {
    .nav { grid-template-columns: 1fr 1fr; }
    .dots { grid-column: 1 / -1; grid-row: 1; margin-bottom: 0.3rem; }
  }
  @media (prefers-reduced-motion: reduce) { .drag, .row { animation: none; } }
</style>
