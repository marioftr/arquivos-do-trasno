<script lang="ts">
  import { onMount } from "svelte";
  import InfoTip from "./InfoTip.svelte";

  /** Explicación dun grupo de axustes. A «i» vai sempre ao carón do que explica: pegada ao título do grupo (a primeira nota do
   *  grupo) ou dentro do primeiro botón (á dereita). Se non hai nin unha cousa nin outra, a nota é un texto normal: nunca queda
   *  unha liña xenérica de «máis información» nin unha «i» solta. Con `label`, vai nunha liña co seu nome e a «i». */
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
      const first = group.querySelector(":scope > .gn") === el;
      const next = el.nextElementSibling as HTMLElement | null;
      if (title && first) {
        mode = "title";
        // Xusto despois do último carácter do título (non ao final da liña).
        const r = document.createRange();
        r.selectNodeContents(title);
        const end = r.getBoundingClientRect().right - group.getBoundingClientRect().left;
        style = `top:${title.offsetTop + (title.offsetHeight - 18) / 2}px;left:${Math.round(end + 2)}px`;
      } else if (!title && next && next.matches("button.btn")) {
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
  {:else}
    <span class="gl">{text}</span>
  {/if}
</span>

<style>
  .gn { line-height: 1; }
  .gn.title, .gn.inbtn { position: absolute; z-index: 3; }
  .gn.row { display: block; line-height: 1.4; }
  .gl { display: inline-flex; align-items: center; font-size: var(--fs-sm); color: var(--text-muted); }
</style>
