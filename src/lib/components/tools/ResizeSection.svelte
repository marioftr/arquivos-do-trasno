<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { join } from "@tauri-apps/api/path";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, formatBytes, sameFormat, cannotSaveNotice, animatedNotice, readOnlyLabel, TRASNO, fallo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { samePath, askOutput, askDir } from "./shared";
  import Levels from "../ui/Levels.svelte";
  import LockNote from "../ui/LockNote.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import PhotoMeta from "../ui/PhotoMeta.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import Slider from "../ui/Slider.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Redimensionar imaxes: por porcentaxe ou en píxeles (ancho e alto, ou un tamaño con nome que fixa o lado maior). Unha ou varias de vez; sempre ficheiros novos e no mesmo formato. */
  export let selection: FileEntry[];

  const multi = selection.length > 1;
  const first = selection[0];

  // Tamaños con nome: fixan o lado maior. Calquera outra medida vai en «Personalizado».
  const SIDES = [
    { id: "peq", label: "Pequeno", px: 640 },
    { id: "medio", label: "Mediano", px: 1280 },
    { id: "grande", label: "Grande", px: 1920 },
  ];

  // `how`: 0 porcentaxe, 1 píxeles. `lock`: manter a proporción. `crop`: 0 recortar polo centro, 1 estirar.
  const form = createForm(
    { how: 1, percent: 50, width: 1920 as number | null, height: 1920 as number | null, lock: true, noUpscale: true, crop: 0, quality: 90, keepMeta: false, keepGps: false },
    "redimensionar",
    "redimensionar|" + selection.map((f) => f.path).join("|"),
  );
  onMount(() => {
    if ($form.how > 1) $form.how = 1; // o modo «lado maior» xa non existe: é o de píxeles
    return useForm(form);
  });
  $: v = $form;

  const job = createJob();
  const busy = job.busy;

  // Ficheiros que se poden redimensionar: o formato ten que saberse gardar, e as animacións só se levan en GIF e WebP.
  let animated = new Set<string>();
  $: void checkAnimated(selection);
  async function checkAnimated(list: FileEntry[]) {
    const found = new Set<string>();
    await Promise.all(
      list
        .filter((f) => ["webp", "png", "apng", "gif"].includes(f.ext.toLowerCase()))
        .map((f) => engine.imageIsAnimated(f.path).then((a) => a && found.add(f.path)).catch(() => {})),
    );
    if (list === selection) animated = found;
  }
  const blockedWhy = (f: FileEntry, anim: Set<string>): string | null => {
    if (!sameFormat(f.ext)) return cannotSaveNotice(f.ext, "redimensionar");
    if (anim.has(f.path) && !["gif", "webp"].includes(f.ext.toLowerCase())) return animatedNotice(f.ext, "redimensionar");
    return null;
  };
  $: blocked = selection.filter((f) => blockedWhy(f, animated));
  $: usable = selection.filter((f) => !blockedWhy(f, animated));
  $: allBlocked = usable.length === 0;
  $: blockedText = blocked.length === 1 ? (blockedWhy(blocked[0], animated) ?? "") : tr("{n} ficheiros non se redimensionan: ou só se poden ler, ou son animados e non se poden gardar así. Convérteos antes a PNG ou JPG, en Converter.", { n: blocked.length });

  const lossy = (f: FileEntry) => ["jpg", "jpeg", "webp", "avif", "jxl", "jp2"].includes(f.ext.toLowerCase());
  $: hasLossy = usable.some(lossy);
  $: hasMeta = usable.some((f) => engine.META_EXTS.includes(f.ext.toLowerCase()));

  // O axuste tal como o entende o motor.
  $: fit = (v.how === 0
    ? { mode: "percent", percent: v.percent }
    : v.lock
      ? { mode: "within", width: v.width || null, height: v.height || null, no_upscale: v.noUpscale }
      : { mode: "exact", width: v.width || 0, height: v.height || 0, crop: v.crop === 0 }) as engine.ResizeFit;

  // Cunha soa imaxe e a proporción mantida, o ancho e o alto van ligados: cambiar un cambia o outro no campo.
  let dims: [number, number] | null = null;
  $: if (!multi && usable[0]) void engine.resizePreview(usable[0].path, { mode: "percent", percent: 100 }).then(([from]) => (dims = from)).catch(() => (dims = null));
  $: aspect = dims ? dims[0] / dims[1] : 0;
  $: linked = !multi && v.how === 1 && v.lock && aspect > 0;
  /** O maior tamaño coa proporción da imaxe que cabe nun cadro. */
  const fitBox = (bw: number, bh: number): [number, number] => {
    const [w, h] = dims as [number, number];
    const k = Math.min(bw / w, bh / h);
    return [Math.max(1, Math.round(w * k)), Math.max(1, Math.round(h * k))];
  };
  function setSide(which: "width" | "height", raw: string) {
    const n = raw === "" ? null : Number(raw);
    form.update((f) => {
      const next = { ...f, [which]: n };
      if (linked && n && n > 0) {
        if (which === "width") next.height = Math.max(1, Math.round(n / aspect));
        else next.width = Math.max(1, Math.round(n * aspect));
      }
      return next;
    });
  }
  // Se ao activar a proporción as dúas medidas non a respectan, ténsase o cadro que forman: a imaxe cabe nel.
  $: if (linked && v.width && v.height && Math.abs(v.width - v.height * aspect) > 1 && Math.abs(v.height - v.width / aspect) > 1) {
    const [w, h] = fitBox(v.width, v.height);
    // Fóra desta pasada de reactividade, para que os campos se repinten co valor novo.
    queueMicrotask(() => form.update((f) => ({ ...f, width: w, height: h })));
  }

  // Tamaños con nome: cunha soa imaxe, o lado maior vale ese tamaño e o outro calcúlase; con varias, cada unha cabe nun cadro dese lado.
  $: sideLevels = SIDES.map((s) => {
    const [w, h] = dims && !multi ? fitBox(s.px, s.px) : [s.px, s.px];
    return { id: s.id, label: $t(s.label), detail: `${s.px} px`, values: { width: w, height: h } };
  });

  // Taboa co tamaño de cada imaxe e o que lle quedará.
  type Row = { file: FileEntry; from?: [number, number]; to?: [number, number]; error?: string; state: "agarda" | "feito" | "erro"; text?: string };
  let rows: Row[] = [];
  let token = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  $: schedule(JSON.stringify(fit), usable);
  function schedule(_k: string, list: FileEntry[]) {
    clearTimeout(timer);
    timer = setTimeout(() => void refresh(list), 200);
  }
  async function refresh(list: FileEntry[]) {
    const mine = ++token;
    const next: Row[] = await Promise.all(
      list.map(async (file): Promise<Row> => {
        try {
          const [from, to] = await engine.resizePreview(file.path, fit);
          return { file, from, to, state: "agarda" };
        } catch (e) {
          return { file, error: errText(e), state: "agarda" };
        }
      }),
    );
    if (mine === token) rows = next;
  }
  onDestroy(() => {
    clearTimeout(timer);
    token++;
  });
  const dim = (d?: [number, number]) => (d ? `${d[0]} × ${d[1]}` : "…");
  $: ready = rows.length > 0 && rows.length === usable.length && rows.every((r) => r.to);
  $: firstError = rows.find((r) => r.error)?.error ?? "";

  const opts = (f: FileEntry) => ({ quality: lossy(f) ? v.quality : undefined, metadata: engine.photoMetadata(v.keepMeta, v.keepGps) });

  async function runSingle() {
    const f = usable[0];
    const out = await askOutput(f, tr("-redimensionado"));
    if (!out) return;
    await job.run(TRASNO.traballando, fallo("editando"), async () => {
      const r = await engine.resizeImage(f.path, out, fit, opts(f), true);
      const before = (await engine.fileSize(f.path).catch(() => 0)) || (f.size ?? 0);
      return tr("Listo! Deixei a imaxe en {to} px, de {from} px ({from_size} a {to_size}).", { from: `${r.from[0]}×${r.from[1]}`, to: `${r.to[0]}×${r.to[1]}`, from_size: formatBytes(before), to_size: formatBytes(r.bytes_written) });
    });
  }

  async function runBatch() {
    const dir = await askDir(usable[0].path);
    if (!dir) return;
    rows = rows.map((r) => ({ ...r, state: "agarda", text: undefined }));
    job.busy.set(true);
    job.text.set(tr(TRASNO.traballando));
    job.clear();
    let cursor = 0;
    let before = 0;
    let after = 0;
    const list = rows.map((r) => r.file);
    const worker = async () => {
      while (cursor < list.length) {
        const i = cursor++;
        const f = list[i];
        try {
          const dot = f.name.lastIndexOf(".");
          const base = dot > 0 ? f.name.slice(0, dot) : f.name;
          const wanted = await join(dir, `${base}${tr("-redimensionado")}.${f.ext.toLowerCase()}`);
          if (samePath(wanted, f.path)) throw new Error(tr("Coincidiría co ficheiro orixinal."));
          const out = await engine.uniquePath(wanted);
          const r = await engine.resizeImage(f.path, out, fit, opts(f), false);
          before += f.size ?? 0;
          after += r.bytes_written;
          rows[i] = { ...rows[i], state: "feito", text: `${r.from[0]}×${r.from[1]} → ${r.to[0]}×${r.to[1]} · ${formatBytes(r.bytes_written)}` };
        } catch (e) {
          rows[i] = { ...rows[i], state: "erro", text: errText(e) };
        }
        rows = rows;
      }
    };
    await Promise.all(Array.from({ length: Math.min(3, list.length) }, worker));
    const bad = rows.filter((r) => r.state === "erro").length;
    job.say(
      !bad,
      bad
        ? tr("Vaites! {bad} de {total} imaxes escapáronseme.", { bad, total: list.length })
        : tr("Feito! Redimensionei {n} imaxes en {dir}: de {from} a {to}.", { n: list.length, dir: fileNameOf(dir), from: formatBytes(before), to: formatBytes(after) }),
    );
    job.busy.set(false);
  }
</script>

<div class="stack">
  {#if blocked.length}<LockNote label={allBlocked ? readOnlyLabel() : tr("{n} sen redimensionar", { n: blocked.length })} text={blockedText} />{/if}
  <div class="locked-tools" class:on={allBlocked} inert={allBlocked}>
    <Segmented
      label={$t("Medida")}
      info={$t("Porcentaxe escala todas por igual; Píxeles fixa o ancho e o alto.")}
      options={[
        { value: 0, label: $t("Porcentaxe") },
        { value: 1, label: $t("Píxeles") },
      ]}
      bind:value={$form.how}
    />

    <div class="group">
      {#if v.how === 0}
        <Slider label={$t("Porcentaxe")} unit="%" min={1} max={400} bind:value={$form.percent} def={form.defaults.percent} />
      {:else}
        <Toggle label={$t("Manter proporción")} bind:checked={$form.lock} hint={multi ? $t("Cada imaxe cabe no cadro de ancho e alto sen deformarse. Sen isto, todas toman o tamaño exacto escrito.") : $t("Ao cambiar o ancho ou o alto, o outro axústase pola proporción da imaxe. Sen isto, cada un vale o que se escriba.")} />
        <Levels label={$t("Tamaño")} info={$t("Pequeno, Mediano e Grande fixan o lado maior en 640, 1280 e 1920 px, tamén nas imaxes verticais; a outra medida calcúlase.")} levels={sideLevels} values={{ width: v.width ?? 0, height: v.height ?? 0 }} on:apply={(e) => form.update((f) => ({ ...f, ...e.detail }))} />
        <div class="pair">
          <NumberField label={$t("Ancho")} unit="px" min={1} placeholder={v.lock ? $t("libre") : ""} value={v.width} onInput={(r) => setSide("width", r)} />
          <NumberField label={$t("Alto")} unit="px" min={1} placeholder={v.lock ? $t("libre") : ""} value={v.height} onInput={(r) => setSide("height", r)} />
        </div>
        <Toggle label={$t("Non ampliar as imaxes pequenas")} bind:checked={$form.noUpscale} disabled={!v.lock} hint={$t("As imaxes que xa caben no tamaño elixido quedan como están. Só con proporción mantida.")} />
        <div class="locked-tools" class:on={v.lock} inert={v.lock}>
          <Segmented
            label={$t("Se a proporción cambia")}
            info={$t("Recortar polo centro enche o tamaño e corta o que sobra; Estirar deforma a imaxe. Só sen proporción mantida.")}
            options={[
              { value: 0, label: $t("Recortar polo centro") },
              { value: 1, label: $t("Estirar") },
            ]}
            bind:value={$form.crop}
          />
        </div>
      {/if}
      {#if hasLossy}
        <Slider label={$t("Calidade (JPG, WebP…)")} unit="%" min={10} max={100} bind:value={$form.quality} def={form.defaults.quality} hint={$t("As imaxes con perdas vólvense gardar a esta calidade. O PNG non perde nada.")} />
      {/if}
      {#if hasMeta}
        <PhotoMeta bind:keep={$form.keepMeta} bind:gps={$form.keepGps} disabled={$busy} />
      {/if}
    </div>

    {#if rows.length}
      <ul class="results">
        {#each rows as r}
          <li class={r.state}>
            <span class="rname" title={r.file.path}>{r.file.name}</span>
            <span class="rstate">{r.state === "feito" ? r.text : r.state === "erro" ? $t("erro") : r.error ? "" : `${dim(r.from)} → ${dim(r.to)}`}</span>
          </li>
        {/each}
      </ul>
      {#if firstError}<p class="warn selectable">{firstError}</p>{/if}
      {#each rows.filter((r) => r.state === "erro") as r}<p class="warn selectable">{r.file.name}: {r.text}</p>{/each}
    {/if}

    <button class="btn primary block" on:click={multi ? runBatch : runSingle} disabled={$busy || allBlocked || !ready}>
      {multi ? $t("Redimensionar {n} imaxes", { n: usable.length }) : $t("Redimensionar imaxe")}
    </button>
  </div>
  <JobStatus {job} />
</div>

<style>
  .warn { margin: 0; font-size: var(--fs-sm); color: var(--accent-pdf); word-break: break-word; }
</style>
