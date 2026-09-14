<script lang="ts">
  // Marca abstracta de Trasno: dous cadrados redondeados solapados,
  // suxerindo "dunha forma a outra" (a idea central do conversor) sen
  // ser unha icona literal nin copiar a lapela/lume doutras apps.
  export let size: number = 22;
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" aria-hidden="true" focusable="false">
  <rect x="3" y="7" width="14" height="14" rx="4" fill="var(--accent-conversor)" opacity="0.35" />
  <rect x="7" y="3" width="14" height="14" rx="4" fill="var(--accent-conversor)" />
</svg>
