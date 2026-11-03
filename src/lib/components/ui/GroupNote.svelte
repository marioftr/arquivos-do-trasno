<script lang="ts">
  import { onMount } from "svelte";
  import InfoTip from "./InfoTip.svelte";

  /** Explicación dun grupo de axustes: en vez de ocupar un parágrafo, vive nunha «i». Vai na liña do título do
   *  grupo, dentro do primeiro botón (á dereita) ou, se non hai nin unha cousa nin outra, nunha liña propia. */
  export let text: string;

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

<span class={`gn ${mode}`} {style} bind:this={el}><InfoTip {text} /></span>

<style>
  .gn { line-height: 1; }
  .gn.title, .gn.inbtn { position: absolute; z-index: 3; }
  .gn.row { align-self: flex-end; margin: -0.2rem 0 -0.4rem; }
</style>
