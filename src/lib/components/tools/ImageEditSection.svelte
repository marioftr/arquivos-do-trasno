<script lang="ts">
  import * as engine from "../../engine";
  import { imageEdit, type FileEntry } from "../../files";
  import { fileNameOf, TRASNO, IMAGE_TARGETS } from "../../formats";
  import { askOutput } from "./shared";
  import { t, tr } from "../../i18n";
  import { onDestroy } from "svelte";

  export let file: FileEntry;

  let rotate = 0;
  let flipH = false;
  let flipV = false;
  let grayscale = false;
  let invert = false;
  let brightness = 0;
  let contrast = 0;
  let blur = 0;
  let cropOn = false;
  let cx = 0;
  let cy = 0;
  let cw = 0;
  let ch = 0;
  let ext = file.ext === "jpg" || file.ext === "png" || file.ext === "webp" ? file.ext : "png";

  let busy = false;
  let message: { ok: boolean; text: string } | null = null;

  $: changed = rotate || flipH || flipV || grayscale || invert || brightness || contrast || blur || cropOn;
  $: reset(file.path);
  $: imageEdit.set({ rotate, flipH, flipV, grayscale, invert, brightness, contrast, blur });
  onDestroy(() => imageEdit.set(null));

  function reset(_: string) {
    rotate = 0; flipH = flipV = grayscale = invert = false;
    brightness = contrast = blur = 0;
    cropOn = false;
    cx = cy = 0;
    cw = file.width ?? 0;
    ch = file.height ?? 0;
    message = null;
  }

  async function apply() {
    message = null;
    try {
      const out = await askOutput(file, `-${tr("editada")}`, ext);
      if (!out) return;
      busy = true;
      const done = await engine.editImage(
        file.path,
        out,
        {
          rotate,
          flipH,
          flipV,
          grayscale,
          invert,
          brightness,
          contrast,
          blur,
          crop: cropOn ? [cx, cy, cw, ch] : undefined,
          quality: 90,
        },
        true,
      );
      message = { ok: true, text: `${tr(TRASNO.listo)} ${fileNameOf(done)}` };
    } catch (e) {
      message = { ok: false, text: `${tr(TRASNO.tropezouEditando)}\n${e}` };
    } finally {
      busy = false;
    }
  }
</script>

<section>
  <h3>{$t("Editar")}</h3>
  <div class="row">
    <button class="btn" on:click={() => (rotate = (rotate + 270) % 360)} title={$t("Xirar 90° á esquerda")}>↺ 90°</button>
    <button class="btn" on:click={() => (rotate = (rotate + 90) % 360)} title={$t("Xirar 90° á dereita")}>↻ 90°</button>
    <button class="btn" class:on={flipH} on:click={() => (flipH = !flipH)}>{$t("Espello H")}</button>
    <button class="btn" class:on={flipV} on:click={() => (flipV = !flipV)}>{$t("Espello V")}</button>
  </div>
  <div class="row">
    <label class="check"><input type="checkbox" bind:checked={grayscale} /> {$t("Branco e negro")}</label>
    <label class="check"><input type="checkbox" bind:checked={invert} /> {$t("Inverter cores")}</label>
  </div>
  <div class="row"><span>{$t("Brillo")}</span><input type="range" min="-100" max="100" bind:value={brightness} /><input class="num-input" type="number" min="-100" max="100" bind:value={brightness} /></div>
  <div class="row"><span>{$t("Contraste")}</span><input type="range" min="-100" max="100" bind:value={contrast} /><input class="num-input" type="number" min="-100" max="100" bind:value={contrast} /></div>
  <div class="row"><span>{$t("Desenfoque")}</span><input type="range" min="0" max="20" step="0.5" bind:value={blur} /><input class="num-input" type="number" min="0" max="20" step="0.5" bind:value={blur} /></div>

  <label class="check"><input type="checkbox" bind:checked={cropOn} /> {$t("Recortar (en píxeles)")}</label>
  {#if cropOn}
    <div class="row crop">
      <label>x <input class="num-input" type="number" min="0" bind:value={cx} /></label>
      <label>y <input class="num-input" type="number" min="0" bind:value={cy} /></label>
      <label>{$t("ancho")} <input class="num-input" type="number" min="1" bind:value={cw} /></label>
      <label>{$t("alto")} <input class="num-input" type="number" min="1" bind:value={ch} /></label>
    </div>
  {/if}

  <div class="row">
    <span>{$t("Gardar como")}</span>
    <select bind:value={ext}>{#each IMAGE_TARGETS as t}<option value={t}>{t.toUpperCase()}</option>{/each}</select>
  </div>
  <div class="row">
    <button class="btn" on:click={() => reset(file.path)} disabled={!changed || busy}>{$t("Desfacer")}</button>
    <button class="btn primary grow" on:click={apply} disabled={!changed || busy}>{busy ? $t(TRASNO.traballando) : $t("Gardar copia editada")}</button>
  </div>
  {#if message}<p class="msg" class:ok={message.ok} class:erro={!message.ok}>{message.text}</p>{/if}
</section>

<style>
  section { padding: 1rem 0; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 0.6rem; }
  h3 { margin: 0; font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-muted); font-weight: 600; }
  .row { display: flex; align-items: center; gap: 0.5rem; font-size: 0.86rem; flex-wrap: wrap; }
  .row > span { min-width: 4.8rem; color: var(--text-muted); }
  .row input[type="range"] { flex: 1; min-width: 60px; }
  .row select { flex: 1; background: var(--bg-elevated); color: var(--text); border: 1px solid var(--border-strong); border-radius: var(--radius-sm); padding: 0.4em 0.6em; }
  .crop label { display: flex; align-items: center; gap: 0.3rem; color: var(--text-muted); }
  .check { display: flex; align-items: center; gap: 0.4rem; font-size: 0.85rem; color: var(--text-muted); }
  .btn.on { border-color: var(--accent-conversor); color: var(--accent-conversor); }
  .grow { flex: 1; justify-content: center; }
  .msg { margin: 0; font-size: 0.85rem; white-space: pre-wrap; word-break: break-word; }
  .msg.ok { color: var(--accent-imaxe); }
  .msg.erro { color: var(--accent-video); }
</style>
