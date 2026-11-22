<script lang="ts">
  import PanelInfo from "../ui/PanelInfo.svelte";
  import { onMount } from "svelte";
  import { errText } from "../../errors";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { fileNameOf, formatClock, TRASNO, fallo, listo } from "../../formats";
  import { t, tr } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import { createJob } from "../../job";
  import { tools } from "../../settings";
  import { WHISPER_MODELS, missingOf, type Need } from "../../models";
  import { askOutput } from "./shared";
  import Segmented from "../ui/Segmented.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import JobStatus from "../ui/JobStatus.svelte";
  import ExtensionGate from "../ui/ExtensionGate.svelte";

  export let file: FileEntry;

  /** Idiomas que se ofrecen (os nomes van na propia lingua). */
  const LANGS: { code: string; name: string }[] = [
    { code: "gl", name: "Galego" },
    { code: "es", name: "Español" },
    { code: "en", name: "English" },
    { code: "pt", name: "Português" },
    { code: "ca", name: "Català" },
    { code: "eu", name: "Euskara" },
    { code: "fr", name: "Français" },
    { code: "de", name: "Deutsch" },
    { code: "it", name: "Italiano" },
    { code: "nl", name: "Nederlands" },
    { code: "pl", name: "Polski" },
    { code: "ru", name: "Русский" },
    { code: "uk", name: "Українська" },
    { code: "tr", name: "Türkçe" },
    { code: "ar", name: "العربية" },
    { code: "hi", name: "हिन्दी" },
    { code: "ja", name: "日本語" },
    { code: "ko", name: "한국어" },
    { code: "zh", name: "中文" },
  ];

  /** Idiomas aos que se pode traducir. Whisper traduce a voz directamente, pero só ao inglés. */
  const TARGETS = [{ code: "en", name: "English" }];

  const form = createForm({ model: "whisper-small" as engine.ModelId, language: "auto", format: "srt", translate: false, target: "en", shortLines: true }, undefined, "transcribe|" + file.path);
  onMount(() => useForm(form));

  const job = createJob();
  const busy = job.busy;

  $: needs = [{ kind: "tool", id: "whisper" }, { kind: "model", id: $form.model }] as Need[];
  $: missing = missingOf(needs, $tools).length > 0;
  $: srtLike = $form.format !== "txt";

  const langName = (code: string | null) => (code ? (LANGS.find((l) => l.code === code)?.name ?? code) : "");

  async function run() {
    try {
      const f = $form;
      const out = await askOutput(file, "", f.format);
      if (!out) return;
      const options: engine.TranscribeOptions = { model: f.model, language: f.language, translate: f.translate, format: f.format as "srt" | "vtt" | "txt", shortLines: f.shortLines && f.format !== "txt" };
      await job.run(TRASNO.traballando, fallo("editando"), async () => {
        const r = await engine.transcribeMedia(file.path, out, options, true);
        const lang = f.language === "auto" && r.language ? ` ${tr("Idioma detectado: {lang}.", { lang: langName(r.language) })}` : "";
        return `${listo()} ${fileNameOf(r.output_path)}\n${tr("Transcritos {time} de audio en {n} frases.", { time: formatClock(r.duration, 0), n: r.segments })}${lang}`;
      });
    } catch (e) {
      job.say(false, `${fallo("editando")}\n${errText(e)}`);
    }
  }
</script>

<div class="stack">
  <PanelInfo text={$t("Pasa a voz a texto, en subtítulos ou en texto simple.")} />

  <div class="group">
    <Segmented
      label={$t("Calidade")}
      info={$t("Canto máis precisa, máis tarda. Cada modelo descárgase unha soa vez.")}
      options={WHISPER_MODELS.map((m) => ({ value: m.id, label: $t(m.label), detail: m.size, hint: $t(m.hint) }))}
      bind:value={$form.model}
    />
    <SelectField
      label={$t("Idioma da voz")}
      bind:value={$form.language}
      def="auto"
      options={[{ value: "auto", label: $t("Detectar") }, ...LANGS.map((l) => ({ value: l.code, label: l.name }))]}
      hint={$t("Indicar o idioma dá mellores resultados ca detectalo polo audio.")}
    />
    <SelectField
      label={$t("Resultado")}
      bind:value={$form.format}
      def="srt"
      options={[
        { value: "srt", label: $t("Subtítulos (SRT)") },
        { value: "vtt", label: $t("Subtítulos web (VTT)") },
        { value: "txt", label: $t("Texto (TXT)") },
      ]}
    />
    {#if srtLike}
      <Toggle label={$t("Liñas curtas")} bind:checked={$form.shortLines} hint={$t("Parte as frases en liñas de ata 42 caracteres, máis cómodas como subtítulos.")} />
    {/if}
    <Toggle label={$t("Traducir")} bind:checked={$form.translate} hint={$t("Traduce a voz directamente, sen pasar polo texto orixinal. Funciona mellor cos modelos Equilibrada e Precisa.")} />
    {#if $form.translate}
      <SelectField label={$t("Traducir a")} bind:value={$form.target} def="en" options={TARGETS.map((l) => ({ value: l.code, label: l.name }))} />
    {/if}
  </div>

  {#if missing}
    <ExtensionGate {needs} note={$t("O programa fai a transcrición e o modelo recoñece a voz: fan falta os dous.")} />
  {:else}
    <button class="btn primary block" on:click={run} disabled={$busy}>{$t("Transcribir")}</button>
  {/if}
  <JobStatus {job} />
</div>
