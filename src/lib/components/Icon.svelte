<script lang="ts">
  export let name: string;
  export let size: number = 20;
  export let strokeWidth: number = 1.8;
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
  class="icon"
>
  {#if name === "image"}
    <rect x="3" y="3" width="18" height="18" rx="2.5" />
    <circle cx="8.5" cy="8.5" r="1.4" />
    <path d="M21 15.5l-5.5-5.5L5 20.5" />
  {:else if name === "audio"}
    <line x1="4" y1="21" x2="4" y2="14" />
    <line x1="4" y1="10" x2="4" y2="3" />
    <line x1="12" y1="21" x2="12" y2="12" />
    <line x1="12" y1="8" x2="12" y2="3" />
    <line x1="20" y1="21" x2="20" y2="16" />
    <line x1="20" y1="12" x2="20" y2="3" />
    <circle cx="4" cy="12" r="1.8" />
    <circle cx="12" cy="10" r="1.8" />
    <circle cx="20" cy="14" r="1.8" />
  {:else if name === "document"}
    <path d="M13.5 2.5H6.8a1.8 1.8 0 0 0-1.8 1.8v15.4a1.8 1.8 0 0 0 1.8 1.8h10.4a1.8 1.8 0 0 0 1.8-1.8V8z" />
    <path d="M13.5 2.5V8h5" />
    <line x1="8.3" y1="13" x2="15.7" y2="13" />
    <line x1="8.3" y1="16.6" x2="15.7" y2="16.6" />
  {:else if name === "video"}
    <rect x="2.5" y="6.5" width="14" height="11" rx="2" />
    <polygon points="21.5 8 16.5 12 21.5 16 21.5 8" />
  {:else if name === "convert"}
    <polyline points="17 1.5 21 5.5 17 9.5" />
    <path d="M3 12v-1.5a4.5 4.5 0 0 1 4.5-4.5H21" />
    <polyline points="7 22.5 3 18.5 7 14.5" />
    <path d="M21 12v1.5a4.5 4.5 0 0 1-4.5 4.5H3" />
  {:else if name === "back"}
    <polyline points="15 18 9 12 15 6" />
  {:else if name === "check"}
    <polyline points="20 6.5 9.5 17 4 11.5" />
  {:else if name === "close"}
    <line x1="18" y1="6" x2="6" y2="18" />
    <line x1="6" y1="6" x2="18" y2="18" />
  {:else if name === "warning"}
    <path d="M10.3 3.9L1.9 18.1a1.9 1.9 0 0 0 1.6 2.9h16.9a1.9 1.9 0 0 0 1.6-2.9L13.7 3.9a1.9 1.9 0 0 0-3.4 0z" />
    <line x1="12" y1="9.2" x2="12" y2="13.4" />
    <line x1="12" y1="16.8" x2="12" y2="16.9" />
  {:else if name === "chevron-down"}
    <polyline points="6 9 12 15 18 9" />
  {:else if name === "plus"}
    <line x1="12" y1="5" x2="12" y2="19" />
    <line x1="5" y1="12" x2="19" y2="12" />
  {:else if name === "folder"}
    <path d="M21.5 18.5a1.7 1.7 0 0 1-1.7 1.7H4.2a1.7 1.7 0 0 1-1.7-1.7V5.7A1.7 1.7 0 0 1 4.2 4h4.6l2 2.6h9a1.7 1.7 0 0 1 1.7 1.7z" />
  {:else if name === "arrow-right"}
    <line x1="4.5" y1="12" x2="18" y2="12" />
    <polyline points="12.5 6 18.5 12 12.5 18" />
  {:else if name === "layers"}
    <polygon points="12 2.5 22 8 12 13.5 2 8 12 2.5" />
    <polyline points="2 15 12 20.5 22 15" />
    <polyline points="2 11.5 12 17 22 11.5" />
  {/if}
</svg>

<style>
  .icon {
    display: block;
    flex-shrink: 0;
  }
</style>
