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
  {:else if name === "open"}
    <path d="M14 3h7v7" />
    <path d="M21 3l-9 9" />
    <path d="M18 14v5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h5" />
  {:else if name === "load"}
    <path d="M14 4h4.5A1.5 1.5 0 0 1 20 5.5v13a1.5 1.5 0 0 1-1.5 1.5H14" />
    <polyline points="9.5 7.5 14 12 9.5 16.5" />
    <line x1="14" y1="12" x2="3.5" y2="12" />
  {:else if name === "download"}
    <path d="M3 15v3a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-3" />
    <path d="M12 3v12" />
    <polyline points="7.5 10.5 12 15 16.5 10.5" />
  {:else if name === "pipette"}
    <path d="m2 22 1-1h3l9-9" />
    <path d="M3 21v-3l9-9" />
    <path d="m15 6 3.4-3.4a2.1 2.1 0 1 1 3 3L18 9l.4.4a2.1 2.1 0 1 1-3 3l-3.8-3.8a2.1 2.1 0 1 1 3-3l.4.4Z" />
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
  {:else if name === "chat"}
    <path d="M4 5h16v11H9l-5 4z" />
    <line x1="8" y1="9.5" x2="16" y2="9.5" />
    <line x1="8" y1="12.5" x2="13" y2="12.5" />
  {:else if name === "trash"}
    <polyline points="4,7 20,7" />
    <path d="M9 7V4.5h6V7" />
    <path d="M6.5 7l1 13h9l1-13" />
    <line x1="10" y1="11" x2="10" y2="16" />
    <line x1="14" y1="11" x2="14" y2="16" />
  {:else if name === "checks"}
    <rect x="4" y="4" width="16" height="16" rx="2" />
    <polyline points="8,12 11,15 16,9" />
  {:else if name === "activity"}
    <polyline points="3,12 7,12 10,5 14,19 17,12 21,12" />
  {:else if name === "sparkles"}
    <path d="M10 3l1.8 5.2L17 10l-5.2 1.8L10 17l-1.8-5.2L3 10l5.2-1.8z" />
    <path d="M18 14l.8 2.2L21 17l-2.2.8L18 20l-.8-2.2L15 17l2.2-.8z" />
  {:else if name === "search"}
    <circle cx="11" cy="11" r="6.5" />
    <line x1="16" y1="16" x2="21" y2="21" />
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
  {:else if name === "reset" || name === "rotate-ccw"}
    <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
    <path d="M3 3v5h5" />
  {:else if name === "rotate-cw"}
    <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
    <path d="M21 3v5h-5" />
  {:else if name === "flip-h"}
    <line x1="3.5" y1="12" x2="20.5" y2="12" />
    <polyline points="7.5 8 3.5 12 7.5 16" />
    <polyline points="16.5 8 20.5 12 16.5 16" />
  {:else if name === "flip-v"}
    <line x1="12" y1="3.5" x2="12" y2="20.5" />
    <polyline points="8 7.5 12 3.5 16 7.5" />
    <polyline points="8 16.5 12 20.5 16 16.5" />
  {:else if name === "tri-down"}
    <polygon points="6 9.5 18 9.5 12 16.5" fill="currentColor" stroke="none" />
  {:else if name === "tri-right"}
    <polygon points="9.5 6 16.5 12 9.5 18" fill="currentColor" stroke="none" />
  {:else if name === "sliders"}
    <line x1="4" y1="7" x2="20" y2="7" />
    <line x1="4" y1="17" x2="20" y2="17" />
    <circle cx="9" cy="7" r="2.3" fill="var(--bg-elevated, transparent)" />
    <circle cx="15" cy="17" r="2.3" fill="var(--bg-elevated, transparent)" />
  {:else if name === "cursor"}
    <path d="M4.04 4.69a.5.5 0 0 1 .65-.65l16 6.5a.5.5 0 0 1-.06.95l-6.12 1.58a2 2 0 0 0-1.44 1.44l-1.58 6.12a.5.5 0 0 1-.95.06z" />
  {:else if name === "pencil"}
    <path d="M21.17 6.81a1 1 0 0 0-3.99-3.99L3.84 16.17a2 2 0 0 0-.5.83l-1.32 4.35a.5.5 0 0 0 .62.62l4.35-1.32a2 2 0 0 0 .83-.5z" />
    <path d="m15 5 4 4" />
  {:else if name === "highlighter"}
    <path d="m9 11-6 6v3h9l3-3" />
    <path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4" />
  {:else if name === "line"}
    <line x1="5" y1="19" x2="19" y2="5" />
  {:else if name === "arrow-diag"}
    <line x1="5" y1="19" x2="19" y2="5" />
    <polyline points="9 5 19 5 19 15" />
  {:else if name === "square"}
    <rect x="3.5" y="3.5" width="17" height="17" rx="2" />
  {:else if name === "circle"}
    <circle cx="12" cy="12" r="9" />
  {:else if name === "type"}
    <polyline points="4 7 4 4 20 4 20 7" />
    <line x1="9" y1="20" x2="15" y2="20" />
    <line x1="12" y1="4" x2="12" y2="20" />
  {:else if name === "eraser"}
    <path d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21" />
    <path d="M22 21H7" />
    <path d="m5 11 9 9" />
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
  {:else if name === "gear"}
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 14.5a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-1.8-.3 1.6 1.6 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.6 1.6 0 0 0-1-1.5 1.6 1.6 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.6 1.6 0 0 0 .3-1.8 1.6 1.6 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.6 1.6 0 0 0 1.5-1 1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.6 1.6 0 0 0 1.8.3h0a1.6 1.6 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.6 1.6 0 0 0 1 1.5h0a1.6 1.6 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8v0a1.6 1.6 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1z" />
  {:else if name === "keyboard"}
    <rect x="2.5" y="6" width="19" height="12" rx="2.2" />
    <line x1="6" y1="10" x2="6" y2="10.1" /><line x1="9.5" y1="10" x2="9.5" y2="10.1" /><line x1="13" y1="10" x2="13" y2="10.1" /><line x1="16.5" y1="10" x2="16.5" y2="10.1" />
    <line x1="7.5" y1="14.2" x2="16.5" y2="14.2" />
  {:else if name === "compress"}
    <polyline points="4 14 10 14 10 20" />
    <polyline points="20 10 14 10 14 4" />
    <line x1="14" y1="10" x2="21" y2="3" />
    <line x1="3" y1="21" x2="10" y2="14" />
  {:else if name === "lock"}
    <rect x="5" y="11" width="14" height="10" rx="2" />
    <path d="M8 11V8a4 4 0 0 1 8 0v3" />
  {:else if name === "merge"}
    <path d="M6 3v6a6 6 0 0 0 6 6" />
    <path d="M18 3v6a6 6 0 0 1-6 6" />
    <line x1="12" y1="15" x2="12" y2="21" />
  {:else if name === "pages"}
    <rect x="5" y="3" width="11" height="15" rx="1.5" />
    <path d="M9 21h8a2 2 0 0 0 2-2V7" />
  {:else if name === "crop"}
    <path d="M6 2v14a2 2 0 0 0 2 2h14" />
    <path d="M2 6h14a2 2 0 0 1 2 2v14" />
  {:else if name === "wrench"}
    <path d="M14.7 6.3a4 4 0 0 0-5.4 5.1L3.5 17.2a1.8 1.8 0 0 0 2.6 2.6l5.8-5.8a4 4 0 0 0 5.1-5.4l-2.4 2.4-2.1-.5-.5-2.1z" />
  {:else if name === "tracks"}
    <line x1="4" y1="7" x2="20" y2="7" />
    <line x1="4" y1="12" x2="20" y2="12" />
    <line x1="4" y1="17" x2="20" y2="17" />
    <circle cx="9" cy="7" r="1.6" />
    <circle cx="15" cy="12" r="1.6" />
    <circle cx="8" cy="17" r="1.6" />
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
