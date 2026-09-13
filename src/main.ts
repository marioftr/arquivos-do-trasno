import { mount } from "svelte";
import "./styles/global.css";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error("non se atopou #app no index.html");
}

const app = mount(App, { target });

export default app;
