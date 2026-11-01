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
    <line x1="3" y1="10" x2="3" y2="14" />
    <line x1="7.5" y1="6" x2="7.5" y2="18" />
    <line x1="12" y1="3" x2="12" y2="21" />
    <line x1="16.5" y1="7.5" x2="16.5" y2="16.5" />
    <line x1="21" y1="10" x2="21" y2="14" />
  {:else if name === "document"}
    <path d="M13.5 2.5H6.8a1.8 1.8 0 0 0-1.8 1.8v15.4a1.8 1.8 0 0 0 1.8 1.8h10.4a1.8 1.8 0 0 0 1.8-1.8V8z" />
    <path d="M13.5 2.5V8h5" />
    <line x1="8.3" y1="13" x2="15.7" y2="13" />
    <line x1="8.3" y1="16.6" x2="15.7" y2="16.6" />
  {:else if name === "video"}
    <rect x="2.5" y="4.5" width="19" height="15" rx="2.5" />
    <polygon points="10 9 15.5 12 10 15" />
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
  {:else if name === "cube"}
    <path d="M12 2.5l8.5 4.75v9.5L12 21.5l-8.5-4.75v-9.5z" />
    <polyline points="3.5 7.25 12 12 20.5 7.25" />
    <line x1="12" y1="12" x2="12" y2="21.5" />
  {:else if name === "up"}
    <polyline points="6 15 12 9 18 15" />
  {:else if name === "down"}
    <polyline points="6 9 12 15 18 9" />
  {:else if name === "undo"}
    <path d="M9 14L4 9l5-5" />
    <path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11" />
  {:else if name === "redo"}
    <path d="M15 14l5-5-5-5" />
    <path d="M20 9H9.5a5.5 5.5 0 0 0 0 11H13" />
  {:else if name === "reset"}
    <path d="M3 12a9 9 0 1 0 3-6.7" />
    <polyline points="3 4 3 9 8 9" />
  {:else if name === "sun"}
    <circle cx="12" cy="12" r="4" />
    <line x1="12" y1="2.5" x2="12" y2="5" /><line x1="12" y1="19" x2="12" y2="21.5" />
    <line x1="2.5" y1="12" x2="5" y2="12" /><line x1="19" y1="12" x2="21.5" y2="12" />
    <line x1="5.3" y1="5.3" x2="7" y2="7" /><line x1="17" y1="17" x2="18.7" y2="18.7" />
    <line x1="5.3" y1="18.7" x2="7" y2="17" /><line x1="17" y1="7" x2="18.7" y2="5.3" />
  {:else if name === "moon"}
    <path d="M20.5 14.2A8.5 8.5 0 1 1 9.8 3.5a6.8 6.8 0 0 0 10.7 10.7z" />
  {:else if name === "puzzle"}
    <path d="M10 3.5a2 2 0 0 1 4 0V6h3.5A1.5 1.5 0 0 1 19 7.5V11h-2.5a2 2 0 0 0 0 4H19v3.5a1.5 1.5 0 0 1-1.5 1.5H14v-2.5a2 2 0 0 0-4 0V20H6.5A1.5 1.5 0 0 1 5 18.5V15h2.5a2 2 0 0 0 0-4H5V7.5A1.5 1.5 0 0 1 6.5 6H10z" />
  {:else if name === "info"}
    <circle cx="12" cy="12" r="9" />
    <line x1="12" y1="11" x2="12" y2="16.5" />
    <line x1="12" y1="7.6" x2="12" y2="7.7" />
  {:else if name === "play"}
    <polygon points="7 4.5 19 12 7 19.5 7 4.5" />
  {:else if name === "pause"}
    <line x1="8" y1="5" x2="8" y2="19" />
    <line x1="16" y1="5" x2="16" y2="19" />
  {:else if name === "volume"}
    <polygon points="4 9.5 8 9.5 13 5.5 13 18.5 8 14.5 4 14.5 4 9.5" />
    <path d="M16.5 8.5a5 5 0 0 1 0 7" />
    <path d="M19 6a8.5 8.5 0 0 1 0 12" />
  {:else if name === "mute"}
    <polygon points="4 9.5 8 9.5 13 5.5 13 18.5 8 14.5 4 14.5 4 9.5" />
    <line x1="17" y1="9.5" x2="22" y2="14.5" />
    <line x1="22" y1="9.5" x2="17" y2="14.5" />
  {:else if name === "expand"}
    <polyline points="4 9 4 4 9 4" />
    <polyline points="20 9 20 4 15 4" />
    <polyline points="4 15 4 20 9 20" />
    <polyline points="20 15 20 20 15 20" />
  {:else if name === "copy"}
    <rect x="9" y="9" width="12" height="12" rx="2" />
    <path d="M5 15V5a2 2 0 0 1 2-2h10" />
  {/if}
</svg>

<style>
  .icon {
    display: block;
    flex-shrink: 0;
  }
</style>
