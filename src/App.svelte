<script lang="ts">
  import { currentView } from "./lib/router";
  import Home from "./routes/Home.svelte";
  import Converter from "./routes/Converter.svelte";
  import ImageStudio from "./routes/ImageStudio.svelte";
  import AudioStudio from "./routes/AudioStudio.svelte";
  import PdfStudio from "./routes/PdfStudio.svelte";
  import VideoStudio from "./routes/VideoStudio.svelte";
</script>

<main class="shell">
  {#if $currentView === "inicio"}
    <Home />
  {:else if $currentView === "conversor"}
    <Converter />
  {:else if $currentView === "imaxe"}
    <ImageStudio />
  {:else if $currentView === "audio"}
    <AudioStudio />
  {:else if $currentView === "pdf"}
    <PdfStudio />
  {:else if $currentView === "video"}
    <VideoStudio />
  {/if}
</main>

<style>
  .shell {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>
