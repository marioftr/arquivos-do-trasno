try {
  var v = localStorage.getItem("arquivos-do-trasno.web-theme");
  document.documentElement.dataset.theme = v === "light" || v === "dark" ? v : matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
} catch (e) {}
