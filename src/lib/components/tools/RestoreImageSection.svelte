<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { join } from "@tauri-apps/api/path";
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import { viewUrl, type FileEntry } from "../../files";
  import { fileNameOf, formatBytes, sameFormat, cannotSaveNotice, animatedNotice, readOnlyLabel, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { cutoutView } from "../../transparency";
  import { samePath, askOutput, askDir } from "./shared";
  import LockNote from "../ui/LockNote.svelte";
  import PhotoMeta from "../ui/PhotoMeta.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import Slider from "../ui/Slider.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Restaurar imaxes sen IA xerativa: luz, cor, ruído e nitidez. Unha ou varias de vez; sempre ficheiros novos e no mesmo formato. */
  export let selection: FileEntry[];

  const multi = selection.length > 1;
  const first = selection[0];

  type Values = engine.RestoreImageOptions;
  const NONE: Values = { kind: "photo", auto_levels: false, color_balance: false, shadows: 0, saturation: 0, denoise: 0, sharpen: 0, brightness: 0, contrast: 0, blur: 0, invert: false };
  /** Perfís: cada un fixa os axustes de abaixo (incluído o tipo: cor, documento ou branco e negro); tocar un axuste pasa a «Personalizado». */
  const PRESETS: Record<string, Values> = {
    auto: { ...NONE, auto_levels: true, color_balance: true, saturation: 12 },
    light: { ...NONE, shadows: 70 },
    document: { ...NONE, kind: "document" },
    bw: { ...NONE, kind: "bw" },
    denoise: { ...NONE, denoise: 2 },
    sharp: { ...NONE, sharpen: 2 },
  };
  const profiles = () => [
    { value: "light", label: tr("Máis luz"), icon: "sun" },
    { value: "document", label: tr("Documento"), icon: "document" },
    { value: "bw", label: tr("Branco e negro"), icon: "half" },
    { value: "denoise", label: tr("Menos ruído"), icon: "grain" },
    { value: "sharp", label: tr("Máis nitidez"), icon: "focus" },
    { value: "auto", label: tr("Automático"), icon: "sparkles" },
    { value: "custom", label: tr("Personalizado"), icon: "sliders" },
  ];
  const levels = () => [
    { value: 0, label: tr("Non") },
    { value: 1, label: tr("Suave") },
    { value: 2, label: tr("Normal") },
    { value: 3, label: tr("Forte") },
  ];

  const form = createForm({ profile: "auto", ...PRESETS.auto, keepMeta: true, keepGps: true }, undefined, "restaurar-imx|" + selection.map((f) => f.path).join("|"));
  onMount(() => useForm(form));

  const same = (a: Values, b: Values) => (Object.keys(a) as (keyof Values)[]).every((k) => a[k] === b[k]);
  const valuesOf = (f: Values): Values => ({ kind: f.kind, auto_levels: f.auto_levels, color_balance: f.color_balance, shadows: f.shadows, saturation: f.saturation, denoise: f.denoise, sharpen: f.sharpen, brightness: f.brightness, contrast: f.contrast, blur: f.blur, invert: f.invert });
  let lastProfile = $form.profile;
  /** O perfil e os axustes van xuntos: elixir un perfil pon os seus valores; mover un axuste leva a «Personalizado». Elixir «Personalizado» deixa os axustes como están. */
  function sync(f: typeof $form) {
    if (f.profile !== lastProfile) {
      lastProfile = f.profile;
      if (f.profile !== "custom" && PRESETS[f.profile]) form.update((v) => ({ ...v, ...PRESETS[f.profile] }));
      return;
    }
    if (f.profile !== "custom" && PRESETS[f.profile] && !same(PRESETS[f.profile], valuesOf(f))) {
      lastProfile = "custom";
      form.update((v) => ({ ...v, profile: "custom" }));
    }
  }
  $: sync($form);

  const opts = (f: typeof $form): engine.RestoreImageOptions => valuesOf(f);
  $: changed = !same(valuesOf($form), NONE);
  // Os controis de cor só valen coa imaxe en cor.
  $: colour = $form.kind === "photo";

  const job = createJob();
  const busy = job.busy;

  // Ficheiros que se poden restaurar: o formato ten que saberse gardar e non pode ser animado.
  let animated = new Set<string>();
  $: void checkAnimated(selection);
  async function checkAnimated(list: FileEntry[]) {
    const found = new Set<string>();
    await Promise.all(list.filter((f) => ["webp", "png", "apng", "gif"].includes(f.ext.toLowerCase())).map((f) => engine.imageIsAnimated(f.path).then((a) => a && found.add(f.path)).catch(() => {})));
    if (list === selection) animated = found;
  }
  const blockedWhy = (f: FileEntry, anim: Set<string>): string | null => (!sameFormat(f.ext) ? cannotSaveNotice(f.ext, "restaurar") : anim.has(f.path) ? animatedNotice(f.ext, "restaurar") : null);
  $: blocked = selection.filter((f) => blockedWhy(f, animated));
  $: usable = selection.filter((f) => !blockedWhy(f, animated));
  $: allBlocked = usable.length === 0;
  $: blockedText = blocked.length === 1 ? (blockedWhy(blocked[0], animated) ?? "") : tr("{n} ficheiros non se restauran: ou só se poden ler, ou son animados.", { n: blocked.length });
  const lossy = (f: FileEntry) => ["jpg", "jpeg", "webp", "avif", "jxl", "jp2"].includes(f.ext.toLowerCase());
  $: hasMeta = usable.some((f) => engine.META_EXTS.includes(f.ext.toLowerCase()));

  // ---- vista previa en directo no visor (só cunha imaxe): o motor prepara unha copia pequena cos axustes de agora.
  // A última copia lista queda cargada en memoria: «Ver orixinal» só cambia o que se amosa, sen volver pedir nada.
  let original = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;
  let token = 0;
  let ready: string | null = null;
  let keep: HTMLImageElement | null = null;
  $: schedule($form, allBlocked);
  function schedule(_f: unknown, off: boolean) {
    clearTimeout(timer);
    token++;
    if (multi || off || !changed) {
      ready = null;
      keep = null;
      return;
    }
    // Mentres se prepara a nova, séguese vendo a anterior (ou o orixinal se aínda non hai ningunha).
    timer = setTimeout(render, 300);
  }
  $: cutoutView.set(original ? null : ready);
  async function render() {
    const mine = ++token;
    try {
      const url = await viewUrl(await engine.restoreImagePreview(first.path, ++seq, opts($form)));
      // Descodificada de antemán: ao soltar «Ver orixinal» aparece á vez, sen agardar a que se cargue.
      const img = new Image();
      img.src = url;
      await img.decode().catch(() => {});
      if (mine === token) {
        keep = img;
        ready = url;
      }
    } catch (e) {
      if (mine === token) {
        ready = null;
        keep = null;
        job.say(false, `${tr("Non puiden previsualizar a imaxe.")}\n${errText(e)}`);
      }
    }
  }
  onDestroy(() => {
    clearTimeout(timer);
    token++;
    cutoutView.set(null);
    void engine.forgetPreviews(first.path).catch(() => {});
  });

  /** «Manter para ver o orixinal»: vale só mentres o botón segue premido. */
  function hold(e: PointerEvent) {
    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {
      // sen captura: vale igualmente ata soltar ou saír do botón
    }
    original = true;
  }

  const extra = (f: FileEntry) => ({ quality: lossy(f) ? 92 : undefined, metadata: engine.photoMetadata($form.keepMeta, $form.keepGps) });

  async function runSingle() {
    const f = usable[0];
    const out = await askOutput(f, tr("-restaurado"));
    if (!out) return;
    await job.run(TRASNO.traballando, fallo("editando"), async () => {
      const r = await engine.restoreImage(f.path, out, opts($form), extra(f), true);
      return `${listo()} ${fileNameOf(r.output_path)} (${formatBytes(r.bytes_written)})`;
    });
  }

  async function runBatch() {
    const dir = await askDir(usable[0].path);
    if (!dir) return;
    await job.run(TRASNO.traballando, fallo("editando"), async () => {
      const bad: string[] = [];
      let done = 0;
      for (const f of usable) {
        try {
          const dot = f.name.lastIndexOf(".");
          const base = dot > 0 ? f.name.slice(0, dot) : f.name;
          const wanted = await join(dir, `${base}${tr("-restaurado")}.${f.ext.toLowerCase()}`);
          if (samePath(wanted, f.path)) throw new Error(tr("Coincidiría co ficheiro orixinal."));
          await engine.restoreImage(f.path, await engine.uniquePath(wanted), opts($form), extra(f), false);
          done++;
        } catch (e) {
          bad.push(`${f.name}: ${errText(e)}`);
        }
      }
      if (bad.length) throw new Error(tr("Vaites! {bad} de {total} imaxes escapáronseme.", { bad: bad.length, total: usable.length }) + "\n" + bad.join("\n"));
      return tr("Feito! Restaurei {n} imaxes en {dir}.", { n: done, dir: fileNameOf(dir) });
    });
  }
</script>

<div class="stack">
  <PanelInfo text={$t("Corrixe a luz, a cor, o ruído e a nitidez sen inventar detalle, e aplica efectos. O resultado vese no visor mentres se axusta.")} />
  {#if blocked.length}<LockNote label={allBlocked ? readOnlyLabel() : tr("{n} sen restaurar", { n: blocked.length })} text={blockedText} />{/if}
  <div class="locked-tools" class:on={allBlocked} inert={allBlocked}>
    <div class="group">
      <Segmented
        label={$t("Perfil")}
        info={$t("Cada perfil fixa os axustes de abaixo; se os tocas, pasa a Personalizado. Documento deixa o papel branco e o texto negro.")}
        options={profiles()}
        rows={[5, 2]}
        bind:value={$form.profile}
      />
    </div>
    <div class="group">
      <h4 class="group-title">{$t("Luz e cor")}</h4>
      <Toggle label={$t("Niveis automáticos")} bind:checked={$form.auto_levels} disabled={!colour} hint={$t("Gaña contraste estirando a luz da foto. Non converte en día unha escena de noite nin ao revés.")} />
      <Toggle label={$t("Equilibrar a cor")} bind:checked={$form.color_balance} disabled={!colour} hint={$t("Quita as dominantes de cor, como o amarelo da luz artificial.")} />
      <Slider label={$t("Brillo")} min={-100} max={100} bind:value={$form.brightness} def={0} />
      <Slider label={$t("Contraste")} min={-100} max={100} bind:value={$form.contrast} def={0} />
      <Slider label={$t("Sombras")} unit="%" min={0} max={100} bind:value={$form.shadows} def={0} hint={$t("Levanta as zonas escuras sen queimar as claras.")} />
      <Slider label={$t("Saturación")} unit="%" min={-100} max={100} bind:value={$form.saturation} def={0} disabled={!colour} />
    </div>
    <div class="group">
      <h4 class="group-title">{$t("Ruído e nitidez")}</h4>
      <Segmented label={$t("Quitar ruído")} info={$t("Suaviza o gran das fotos con pouca luz sen borrar as beiras.")} options={levels()} bind:value={$form.denoise} />
      <Segmented label={$t("Nitidez")} info={$t("Realza as beiras das fotos borrosas. Moita nitidez tamén realza o ruído.")} options={levels()} bind:value={$form.sharpen} />
    </div>
    <div class="group">
      <h4 class="group-title">{$t("Efectos")}</h4>
      <Slider label={$t("Desenfoque")} unit="px" min={0} max={20} step={0.5} bind:value={$form.blur} def={0} />
      <Toggle label={$t("Inverter cores")} bind:checked={$form.invert} />
    </div>

    {#if !multi}
      <!-- Só vale mentres se mantén premido: ao soltar (ou saír do botón) volve o resultado. -->
      <div class="estimate">
        <button
          class="btn"
          class:on={original}
          disabled={!changed}
          on:pointerdown={hold}
          on:pointerup={() => (original = false)}
          on:pointercancel={() => (original = false)}
          on:lostpointercapture={() => (original = false)}
          on:keydown={(e) => (e.key === " " || e.key === "Enter") && ((original = true), e.preventDefault())}
          on:keyup={() => (original = false)}
          on:blur={() => (original = false)}
        >{$t("Ver orixinal")}</button>
      </div>
    {/if}
    <button class="btn primary block" on:click={multi ? runBatch : runSingle} disabled={$busy || allBlocked || !changed}>
      {multi ? $t("Restaurar {n} imaxes", { n: usable.length }) : $t("Restaurar imaxe")}
    </button>
  </div>
  {#if hasMeta}
    <PhotoMeta bind:keep={$form.keepMeta} bind:gps={$form.keepGps} disabled={$busy} />
  {/if}
  <JobStatus {job} />
</div>
