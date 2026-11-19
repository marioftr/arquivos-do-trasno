<script lang="ts">
  import { onMount } from "svelte";
  import * as engine from "../../engine";
  import type { FileEntry } from "../../files";
  import { PDF_TO_DOC } from "../../formats";
  import { t } from "../../i18n";
  import { createForm, useForm } from "../../forms";
  import SelectField from "../ui/SelectField.svelte";
  import ConvertSection from "./ConvertSection.svelte";
  import PdfImagesSection from "./PdfImagesSection.svelte";
  import PdfMarkdownSection from "./PdfMarkdownSection.svelte";

  /** Converter un PDF: un só selector de formato de saída e, debaixo, as opcións propias de cada formato. */
  export let file: FileEntry;

  const form = createForm({ out: "png" }, "pdf-converter");
  onMount(() => useForm(form));

  // Os formatos de documento (Word, ODT…) só saen se hai Office ou LibreOffice instalado.
  let docs: string[] = [];
  onMount(async () => {
    try {
      const list = await engine.officeTargets("pdf");
      docs = list.filter((x) => PDF_TO_DOC.includes(x));
    } catch {
      docs = [];
    }
  });
  $: options = [
    { value: "png", label: "PNG" },
    { value: "jpg", label: "JPG" },
    { value: "txt", label: "TXT" },
    { value: "md", label: "Markdown" },
    ...docs.map((x) => ({ value: x, label: x.toUpperCase() })),
  ];
  $: out = options.some((o) => o.value === $form.out) ? $form.out : "png";
</script>

<div class="stack">
  <div class="group">
    <SelectField label={$t("Formato de saída")} bind:value={$form.out} {options} def="png" />
  </div>
  {#key out}
    {#if out === "png" || out === "jpg"}
      <PdfImagesSection {file} format={out} />
    {:else if out === "md"}
      <PdfMarkdownSection {file} />
    {:else}
      <ConvertSection selection={[file]} category="documento" forcedTarget={out} />
    {/if}
  {/key}
</div>
