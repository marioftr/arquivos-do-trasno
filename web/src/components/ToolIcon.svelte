<script lang="ts">
  import Icon from "@app/lib/components/Icon.svelte";

  /** As iconas das ferramentas da web: trazo único de 1,75, mesma cuadrícula de 24 e un debuxo distinto para cada unha. O resto (frechas, tema…) vén do conxunto do programa. */
  export let name: string;
  export let size = 24;
</script>

{#if ["resize", "compress", "rotate-cw", "rotate-ccw", "scan-text", "cutout", "wave", "scissors", "film", "mic", "translate", "merge-doc", "split-doc", "image-doc", "image-pdf", "compress-doc", "text-doc", "enhance", "convert"].includes(name)}
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
    {#if name === "convert"}
      <path d="M4 8h14" /><path d="m14 4 4 4-4 4" /><path d="M20 16H6" /><path d="m10 12-4 4 4 4" />
    {:else if name === "resize"}
      <path d="M14 4h6v6" /><path d="m20 4-7 7" /><path d="M10 20H4v-6" /><path d="m4 20 7-7" />
    {:else if name === "compress"}
      <path d="M14 10h6" /><path d="M14 10V4" /><path d="m20 4-6 6" /><path d="M10 14H4" /><path d="M10 14v6" /><path d="m4 20 6-6" />
    {:else if name === "rotate-cw"}
      <path d="M20 12a8 8 0 1 1-2.6-5.9" /><path d="M20 4v5h-5" />
    {:else if name === "rotate-ccw"}
      <path d="M4 12a8 8 0 1 0 2.6-5.9" /><path d="M4 4v5h5" />
    {:else if name === "scan-text"}
      <path d="M4 8V6a2 2 0 0 1 2-2h2" /><path d="M16 4h2a2 2 0 0 1 2 2v2" /><path d="M20 16v2a2 2 0 0 1-2 2h-2" /><path d="M8 20H6a2 2 0 0 1-2-2v-2" /><path d="M8 9h8" /><path d="M12 9v7" /><path d="M10 16h4" />
    {:else if name === "cutout"}
      <circle cx="12" cy="12" r="9" stroke-dasharray="2.6 3" /><circle cx="12" cy="10" r="2.6" /><path d="M7.5 17.5c.9-2.5 2.5-3.6 4.5-3.6s3.6 1.1 4.5 3.6" />
    {:else if name === "wave"}
      <path d="M3 10v4" /><path d="M7.5 6v12" /><path d="M12 3v18" /><path d="M16.5 7v10" /><path d="M21 10v4" />
    {:else if name === "scissors"}
      <circle cx="6" cy="6" r="2.8" /><circle cx="6" cy="18" r="2.8" /><path d="M20 4 8.4 15.6" /><path d="M14.5 14.5 20 20" /><path d="M8.4 8.4 12 12" />
    {:else if name === "film"}
      <rect x="3" y="4" width="18" height="16" rx="2.5" /><path d="M7.5 4v16" /><path d="M16.5 4v16" /><path d="M3 9h4.5" /><path d="M3 15h4.5" /><path d="M16.5 9H21" /><path d="M16.5 15H21" />
    {:else if name === "mic"}
      <rect x="9" y="3" width="6" height="11" rx="3" /><path d="M5.5 11a6.5 6.5 0 0 0 13 0" /><path d="M12 17.5V21" /><path d="M9 21h6" />
    {:else if name === "translate"}
      <path d="M4 6h9" /><path d="M8.5 4v2" /><path d="M6 6c.3 3 2.7 5.5 6 6.7" /><path d="M11.5 6c-.4 3.2-3 6-7 7.3" /><path d="m13 20 4-9 4 9" /><path d="M14.4 17h5.2" />
    {:else if name === "merge-doc"}
      <rect x="3.5" y="3.5" width="11" height="11" rx="2" /><rect x="9.5" y="9.5" width="11" height="11" rx="2" />
    {:else if name === "split-doc"}
      <path d="M6 3h8l5 5v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" /><path d="M14 3v5h5" /><path d="M4 14h16" stroke-dasharray="2.6 2.6" />
    {:else if name === "image-doc"}
      <path d="M6 3h8l5 5v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" /><path d="M14 3v5h5" /><circle cx="9.5" cy="12" r="1.2" /><path d="m7 19 3.5-3.5 2.5 2.5 1.5-1.5 2.5 2.5" />
    {:else if name === "image-pdf"}
      <rect x="3.5" y="4" width="17" height="16" rx="2.5" /><circle cx="9" cy="9.5" r="1.6" /><path d="m4 18 5-5 4 4 2.5-2.5L20 19" />
    {:else if name === "compress-doc"}
      <path d="M6 3h8l5 5v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" /><path d="M14 3v5h5" /><path d="m9 12.5 3 2.5 3-2.5" /><path d="m9 18 3-2.5 3 2.5" />
    {:else if name === "text-doc"}
      <path d="M6 3h8l5 5v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" /><path d="M14 3v5h5" /><path d="M8 12h8" /><path d="M8 15.5h8" /><path d="M8 19h5" />
    {:else if name === "enhance"}
      <path d="m12 3 1.7 4.6L18.3 9.3l-4.6 1.7L12 15.6l-1.7-4.6L5.7 9.3l4.6-1.7z" /><path d="m18.5 15 .8 2.2 2.2.8-2.2.8-.8 2.2-.8-2.2-2.2-.8 2.2-.8z" />
    {/if}
  </svg>
{:else}
  <Icon {name} {size} />
{/if}
