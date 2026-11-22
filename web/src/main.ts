import { mount } from "svelte";
import "@fontsource-variable/atkinson-hyperlegible-next";
import "@app/styles/tokens.css";
import "./web.css";
import { detectEncoders } from "./lib/image";
import App from "./App.svelte";

// Antes de pintar, saber que formatos escribe este navegador (Safari non escribe WebP).
await detectEncoders();
mount(App, { target: document.getElementById("app")! });

if ("serviceWorker" in navigator && import.meta.env.PROD) {
  addEventListener("load", () => void navigator.serviceWorker.register("/sw.js").catch(() => {}));
}
