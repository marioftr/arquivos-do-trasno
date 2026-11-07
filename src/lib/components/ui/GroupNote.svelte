<script lang="ts">
  import { onMount } from "svelte";
  import InfoTip from "./InfoTip.svelte";
  import { t } from "../../i18n";

  /** Explicación dun grupo de axustes: en vez de ocupar un parágrafo, vive nunha «i». Vai na liña do título do
   *  grupo ou dentro do primeiro botón (á dereita). Se non hai nin unha cousa nin outra, nunca queda unha «i» solta: o texto
   *  curto amósase tal cal e o longo vai nunha liña co seu nome (`label`) e a «i». */
  export let text: string;
  export let label = "";

  let el: HTMLSpanElement;
  let mode: "row" | "title" | "inbtn" = "row";
  let style = "";

  onMount(() => {
    const group = el.closest<HTMLElement>(".group");
    if (!group) return;
    const place = () => {
      const title = group.querySelector<HTMLElement>(":scope > .group-title");
      const next = el.nextElementSibling as HTMLElement | null;
      if (title) {
        mode = "title";
        title.style.paddingRight = "1.8rem";
        style = `top:${title.offsetTop + (title.offsetHeight - 18) / 2}px;right:0`;
      } else if (next && next.matches("button.btn")) {
        mode = "inbtn";
        style = `top:${next.offsetTop + (next.offsetHeight - 18) / 2}px;right:0.7rem`;
      } else {
        mode = "row";
        style = "";
      }
    };
    place();
    const ro = new ResizeObserver(place);
    ro.observe(group);
    return () => ro.disconnect();
  });
</script>

<span class={`gn ${mode}`} {style} bind:this={el}>
  {#if mode !== "row"}
    <InfoTip {text} />
  {:else if label}
    <span class="gl">{label}<InfoTip {text} /></span>
  {:else if text.length <= 72}
    <span class="gl">{text}</span>
  {:else}
    <span class="gl">{$t("Máis información")}<InfoTip {text} /></span>
  {/if}
</span>

<style>
  .gn { line-height: 1; }
  .gn.title, .gn.inbtn { position: absolute; z-index: 3; }
  .gn.row { display: block; line-height: 1.4; }
  .gl { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
</style>
