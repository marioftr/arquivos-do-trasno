<script lang="ts">
  import { openLink, type LinkId } from "../engine";
  import { t } from "../i18n";

  const libs: { name: string; what: string; license: string }[] = [
    { name: "Tauri (e os seus complementos de diálogos, actualizacións e procesos)", what: "Marco da aplicación de escritorio", license: "MIT / Apache-2.0" },
    { name: "Svelte", what: "Interface", license: "MIT" },
    { name: "Vite", what: "Compilación da interface", license: "MIT" },
    { name: "Inter", what: "Tipo de letra da interface", license: "SIL Open Font License 1.1" },
    { name: "PDF.js", what: "Visor e miniaturas de PDF (Mozilla)", license: "Apache-2.0" },
    { name: "Tesseract.js", what: "Recoñecemento de texto en imaxes (OCR)", license: "Apache-2.0" },
    { name: "Tesseract OCR (paquetes de idioma «tessdata_fast»)", what: "Datos de idioma do OCR, que se descargan se os pides", license: "Apache-2.0" },
    { name: "three.js", what: "Visor 3D e conversión de modelos", license: "MIT" },
    { name: "lopdf", what: "Edición da estrutura dos PDF", license: "MIT" },
    { name: "image (image-rs)", what: "Lectura e escritura de imaxes (PNG, JPG, WebP, GIF, BMP, ICO, TIFF, PNM, TGA e QOI)", license: "MIT / Apache-2.0" },
    { name: "kamadak-exif", what: "Datos EXIF das fotos", license: "BSD-2-Clause" },
    { name: "ttf-parser", what: "Lectura de tipos de letra do sistema", license: "MIT / Apache-2.0" },
    { name: "ureq", what: "Descargas (actualizacións, programas e paquetes de idioma)", license: "MIT / Apache-2.0" },
    { name: "sevenz-rust e zip", what: "Descompresión dos programas descargados", license: "Apache-2.0 / MIT" },
    { name: "Rust e as súas bibliotecas (serde, uuid, sha2, which, tempfile…)", what: "Motor da aplicación", license: "MIT / Apache-2.0" },
  ];
  const ext: { name: string; what: string; license: string; link?: LinkId; licenseLink?: LinkId }[] = [
    { name: "FFmpeg", what: "Audio e vídeo, e tamén AVIF, JPEG 2000 e JPEG XL (non vai incluído: execútase se está instalado ou descargado)", license: "LGPL-2.1+ / GPL-2.0+ segundo a compilación", link: "ffmpeg", licenseLink: "ffmpeg-license" },
    { name: "MKVToolNix", what: "Edición de contedores MKV (non vai incluído: execútase se está instalado ou descargado)", license: "GPL-2.0", link: "mkvtoolnix", licenseLink: "mkvtoolnix-license" },
    { name: "LibreOffice", what: "Conversión de documentos de Office e OpenDocument (non vai incluído: execútase se está instalado)", license: "MPL-2.0", link: "libreoffice", licenseLink: "libreoffice-license" },
    { name: "Microsoft Office (Word, Excel, PowerPoint)", what: "Conversión de documentos de Office (non vai incluído nin se descarga: úsase o que o usuario teña instalado, só en Windows)", license: "Software privativo de Microsoft, co seu propio contrato de licenza" },
  ];
</script>

<div class="panel">
  <section>
    <h3>Arquivos do Trasno</h3>
    <p class="muted">
      {$t("O trasno é un duende da tradición galega (tamén chamado trasgu en Asturias e trasgo en Cantabria e León): un ser pequeno, bromista e de fogar que, se o tratas ben, fai as tarefas da casa mentres durmes. Aquí fai o traballo pesado cos teus ficheiros, sen tocar nunca os orixinais.")}
    </p>
  </section>

  <section>
    <h3>{$t("Programas externos")}</h3>
    <p class="muted">{$t("Non van no instalador da aplicación: podes instalalos ti ou deixar que a app os descargue, só se o aceptas, en Axustes → Extensións. A app só os executa como programas independentes.")}</p>
    {#each ext as e}
      <div class="row">
        <div>
          <strong>{e.name}</strong> · <span class="muted">{$t(e.what)}</span>
          <div class="muted small">{$t("Licenza:")} {e.license}</div>
        </div>
        <div class="links">
          {#if e.link}<button class="btn ghost" on:click={() => openLink(e.link!)}>{$t("Web")}</button>{/if}
          {#if e.licenseLink}<button class="btn ghost" on:click={() => openLink(e.licenseLink!)}>{$t("Licenza")}</button>{/if}
        </div>
      </div>
    {/each}
  </section>

  <section>
    <h3>{$t("Bibliotecas usadas")}</h3>
    {#each libs as l}
      <div class="row">
        <div><strong>{l.name}</strong> · <span class="muted">{$t(l.what)}</span></div>
        <span class="muted small">{l.license}</span>
      </div>
    {/each}
  </section>

  <section>
    <h3>{$t("Aviso")}</h3>
    <p class="muted small">
      {$t("Este software ofrécese «tal cal», sen garantía de ningún tipo. Os nomes FFmpeg, MKVToolNix, PDF.js, Tesseract, Tauri e os demais pertencen aos seus respectivos autores, que non avalan nin están asociados a esta aplicación. Cada biblioteca mantense baixo a súa propia licenza, que se conserva nos seus repositorios.")}
    </p>
    <p class="muted small">
      {$t("Microsoft, Office, Word, Excel, PowerPoint e LibreOffice son marcas dos seus propietarios, que non avalan nin están asociados a esta aplicación. A app non inclúe nin distribúe ningún deles: só usa os que haxa instalados no ordenador.")}
    </p>
  </section>
</div>

<style>
  .panel { display: flex; flex-direction: column; gap: 1rem; }
  section { display: flex; flex-direction: column; gap: 0.4rem; }
  h3 { margin: 0; font-size: var(--fs-md); font-weight: 600; }
  p { margin: 0; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.35rem 0; border-top: 1px solid var(--border); }
  .small { font-size: var(--fs-sm); }
  .links { display: flex; gap: 0.3rem; flex-shrink: 0; }
</style>
