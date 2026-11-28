<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { onMount } from "svelte";
  import { join } from "@tauri-apps/api/path";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, formatBytes, TRASNO, fallo, composeFormat, composeOptions, composeShown, composeSame } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { samePath, askPath } from "./shared";
  import ColorField from "../ui/ColorField.svelte";
  import NumberField from "../ui/NumberField.svelte";
  import OrderList from "../ui/OrderList.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Slider from "../ui/Slider.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";

  /** Xuntar nunha soa imaxe (unha debaixo doutra ou ao carón), quitando o que se repite nas de desprazamento. */
  export let selection: FileEntry[];

  const form = createForm(
    { direction: "vertical", align: "narrowest", removeOverlap: true, gap: 0, background: "#ffffff", format: "same", quality: 90 },
    "xuntar-capturas",
    "xuntar|" + selection.map((f) => f.path).join("|"),
  );
  onMount(() => useForm(form));
  $: v = $form;

  // A orde de partida é a da lista; aquí pódese cambiar sen mexer na lista de ficheiros.
  let items: FileEntry[] = [...selection];

  const job = createJob();
  const busy = job.busy;
  let joins: (number | null)[] = [];
  let joined: FileEntry[] = [];
  let joinedRemove = true;

  const rgb = (hex: string): [number, number, number] => {
    const n = parseInt(hex.replace("#", ""), 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  };
  $: horizontal = v.direction === "horizontal";
  $: exts = items.map((f) => f.ext);
  $: outExt = composeFormat(v.format, exts);
  $: sameExt = composeSame(exts);
  $: lossy = outExt === "jpg";

  async function run() {
    const dir = items[0].path.replace(/[\\/][^\\/]*$/, "");
    const out = await askPath(await join(dir, `${tr("capturas-xuntadas")}.${outExt}`));
    if (!out) return;
    const inputs = items.map((f) => f.path);
    if (inputs.some((p) => samePath(p, out))) {
      job.say(false, tr("Non vou sobrescribir un dos ficheiros de orixe. Escolle outro nome."));
      return;
    }
    joins = [];
    await job.run(TRASNO.unindo, fallo("unindo"), async () => {
      const r = await engine.stitchImages(
        inputs,
        out,
        {
          direction: v.direction as "vertical" | "horizontal",
          align: v.align as "narrowest" | "widest",
          remove_overlap: v.removeOverlap,
          gap: v.gap ?? 0,
          background: rgb(v.background),
          quality: lossy ? v.quality : null,
        },
        true,
      );
      joined = [...items];
      joinedRemove = v.removeOverlap;
      joins = r.joins;
      const lost = v.removeOverlap ? r.joins.filter((j) => j === null).length : 0;
      const base = tr("Listo! Xunteinas nunha imaxe de {w}×{h} px ({size}): {name}.", { w: r.size[0], h: r.size[1], size: formatBytes(r.bytes_written), name: fileNameOf(r.output_path) });
      return lost ? `${base} ${lost === 1 ? tr("Nunha unión non atopei onde coinciden e xunteinas tal cal.") : tr("En {n} {n:unión|unións} non atopei onde coinciden e xunteinas tal cal.", { n: lost })}` : base;
    });
  }
</script>

<div class="stack">
  <div class="group">
    <PanelInfo text={$t("Segue a orde da lista: cámbiase arrastrando, coas frechas ou escribindo a posición.")} />
    <OrderList bind:items />
  </div>

  <div class="group">
    <Segmented
      label={$t("Dirección")}
      options={[
        { value: "vertical", label: $t("Unha debaixo doutra"), glyph: { k: "stack", dir: "v" } },
        { value: "horizontal", label: $t("Unha ao carón doutra"), glyph: { k: "stack", dir: "h" } },
      ]}
      bind:value={$form.direction}
    />
    <Segmented
      label={horizontal ? $t("Igualar alto") : $t("Igualar ancho")}
      info={$t("Se as capturas miden distinto, todas pasan ao tamaño da máis pequena ou da máis grande, sen deformalas.")}
      options={[
        { value: "narrowest", label: horizontal ? $t("Á máis baixa") : $t("Á máis estreita"), glyph: { k: "match", to: "small", dir: horizontal ? "h" : "v" } },
        { value: "widest", label: horizontal ? $t("Á máis alta") : $t("Á máis ancha"), glyph: { k: "match", to: "large", dir: horizontal ? "h" : "v" } },
      ]}
      bind:value={$form.align}
    />
    <div class="pair">
      <NumberField label={$t("Espazo entre capturas")} unit="px" min={0} bind:value={$form.gap} def={form.defaults.gap} hint={$t("Non se aplica onde se quitan tramos repetidos.")} />
      <ColorField label={$t("Fondo")} bind:value={$form.background} />
    </div>
    <SelectField
      label={$t("Formato")}
      value={composeShown(v.format, exts)}
      on:change={(e) => ($form.format = String(e.detail))}
      options={composeOptions($t, exts)}
      hint={sameExt ? $t("PNG, TIFF e BMP non perden calidade; JPG e WebP si, e deixan axustala.") : $t("Con formatos distintos non hai «Igual que o orixinal»; por defecto sae PNG, que non perde calidade.")}
      def={sameExt ? "same" : "png"}
    />
    {#if lossy}
      <Slider label={$t("Calidade")} unit="%" min={10} max={100} bind:value={$form.quality} def={form.defaults.quality} />
    {/if}
    <Toggle label={$t("Quitar tramos repetidos")} bind:checked={$form.removeOverlap} hint={$t("Para capturas feitas ao baixar polo contido dunha páxina: únense sen repetir o tramo común, cabeceiras e pés fixos incluídos.")} />
  </div>

  <button class="btn primary block" on:click={run} disabled={$busy}>{$t("Xuntar {n} {n:imaxe|imaxes}", { n: items.length })}</button>

  {#if joins.length}
    <ul class="results">
      {#each joins as j, i}
        <li class={j === null && joinedRemove ? "erro" : "feito"}>
          <span class="rname" title={`${joined[i]?.name} → ${joined[i + 1]?.name}`}>{joined[i]?.name} → {joined[i + 1]?.name}</span>
          <span class="rstate">{j === null ? (joinedRemove ? $t("sen coincidencia") : "") : $t("{n} px repetidos quitados", { n: j })}</span>
        </li>
      {/each}
    </ul>
  {/if}
  <JobStatus {job} />
</div>
