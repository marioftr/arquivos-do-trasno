<script lang="ts">
  // A mesma icona que ten a aplicación (src-tauri/icons).
  import icon from "../assets/icon.png";
  export let size: number = 22;
</script>

<img src={icon} width={size} height={size} alt="" draggable="false" style={`border-radius:${Math.round(size * 0.22)}px`} />

<style>
  img { display: block; flex-shrink: 0; -webkit-user-drag: none; user-select: none; }
</style>
