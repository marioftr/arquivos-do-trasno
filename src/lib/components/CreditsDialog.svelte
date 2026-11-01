<script lang="ts">
  import { openLink, type LinkId } from "../engine";
  import { t } from "../i18n";
  import { dialog } from "../settings";
  import Modal from "./ui/Modal.svelte";

  const libs: { name: string; what: string; license: string }[] = [
    { name: "Tauri", what: "Marco da aplicación de escritorio", license: "MIT / Apache-2.0" },
    { name: "Svelte", what: "Interface", license: "MIT" },
    { name: "Vite", what: "Compilación da interface", license: "MIT" },
    { name: "Inter", what: "Tipo de letra da interface", license: "SIL Open Font License 1.1" },
    { name: "PDF.js", what: "Visor e miniaturas de PDF (Mozilla)", license: "Apache-2.0" },
    { name: "three.js", what: "Visor 3D e conversión de modelos", license: "MIT" },
    { name: "lopdf", what: "Edición da estrutura dos PDF", license: "MIT" },
    { name: "image (image-rs)", what: "Lectura e escritura de imaxes", license: "MIT / Apache-2.0" },
    { name: "Rust e as súas bibliotecas (serde, uuid, sha2, which…)", what: "Motor da aplicación", license: "MIT / Apache-2.0" },
  ];
  const ext: { name: string; what: string; license: string; link: LinkId; licenseLink: LinkId }[] = [
    { name: "FFmpeg", what: "Audio e vídeo (non vai incluído: execútase se está instalado)", license: "LGPL-2.1+ / GPL-2.0+ segundo a compilación", link: "ffmpeg", licenseLink: "ffmpeg-license" },
    { name: "MKVToolNix", what: "Edición de contedores MKV (non vai incluído: execútase se está instalado)", license: "GPL-2.0", link: "mkvtoolnix", licenseLink: "mkvtoolnix-license" },
  ];
</script>

<Modal title={$t("Créditos e licenzas")} onClose={() => dialog.set(null)}>
  <section>
    <h3>Arquivos do Trasno</h3>
    <p class="muted">
      {$t("O trasno é un duende da tradición galega (tamén chamado trasgu en Asturias e trasgo en Cantabria e León): un ser pequeno, bromista e de fogar que, se o tratas ben, fai as tarefas da casa mentres durmes. Aquí fai o traballo pesado cos teus ficheiros, sen tocar nunca os orixinais.")}
    </p>
  </section>

  <section>
    <h3>{$t("Programas externos")}</h3>
    <p class="muted">{$t("Non se distribúen coa aplicación: o usuario instálaos por separado e a app só os executa como programas independentes.")}</p>
    {#each ext as e}
      <div class="row">
        <div>
          <strong>{e.name}</strong> · <span class="muted">{$t(e.what)}</span>
          <div class="muted small">{$t("Licenza:")} {e.license}</div>
        </div>
        <div class="links">
          <button class="btn ghost" on:click={() => openLink(e.link)}>{$t("Web")}</button>
          <button class="btn ghost" on:click={() => openLink(e.licenseLink)}>{$t("Licenza")}</button>
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
      {$t("Este software ofrécese «tal cal», sen garantía de ningún tipo. Os nomes FFmpeg, MKVToolNix, PDF.js, Tauri e os demais pertencen aos seus respectivos autores, que non avalan nin están asociados a esta aplicación. Cada biblioteca mantense baixo a súa propia licenza, que se conserva nos seus repositorios.")}
    </p>
  </section>
</Modal>

<style>
  section { display: flex; flex-direction: column; gap: 0.4rem; }
  h3 { margin: 0; font-size: 0.86rem; font-weight: 600; }
  p { margin: 0; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.35rem 0; border-top: 1px solid var(--border); }
  .small { font-size: 0.78rem; }
  .links { display: flex; gap: 0.3rem; flex-shrink: 0; }
</style>
