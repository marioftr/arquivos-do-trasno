// Coagotas propio: captura o que se ve na xanela (menús incluídos) e deixa tomar a cor de calquera píxel.
// O coagotas do navegador non vale aquí (a xanela de Windows non o amosa), así que a captura faise no sistema.

import { call } from "./errors";

const hex2 = (n: number) => n.toString(16).padStart(2, "0");

/** Devolve a cor elixida («#rrggbb») ou null se se cancela con Esc ou o clic dereito. */
export async function pickScreenColor(): Promise<string | null> {
  // Deixa un instante para que desapareza o que estea aberto (a paleta, por exemplo) antes de capturar.
  await new Promise((r) => setTimeout(r, 120));
  const buf = await call<ArrayBuffer>("capture_window");
  const dv = new DataView(buf);
  const w = dv.getUint32(0, true);
  const h = dv.getUint32(4, true);
  const data = new Uint8ClampedArray(buf, 8, w * h * 4);

  return new Promise((resolve) => {
    const root = document.createElement("div");
    root.style.cssText = "position:fixed;inset:0;z-index:100000;cursor:crosshair;user-select:none;";
    const cv = document.createElement("canvas");
    cv.width = w;
    cv.height = h;
    cv.style.cssText = "width:100%;height:100%;display:block;";
    cv.getContext("2d")!.putImageData(new ImageData(data, w, h), 0, 0);
    root.appendChild(cv);

    const N = 11; // píxeles da lupa por lado
    const Z = 11; // aumento
    const loupe = document.createElement("div");
    loupe.style.cssText = `position:fixed;width:${N * Z}px;border-radius:10px;overflow:hidden;box-shadow:0 6px 22px rgba(0,0,0,.55);border:2px solid #fff;background:#111;pointer-events:none;font:12px Consolas,monospace;color:#fff;`;
    const lc = document.createElement("canvas");
    lc.width = N * Z;
    lc.height = N * Z;
    lc.style.cssText = "display:block;image-rendering:pixelated;";
    const label = document.createElement("div");
    label.style.cssText = "padding:4px 6px;text-align:center;background:#111;";
    loupe.append(lc, label);
    root.appendChild(loupe);

    const at = (e: MouseEvent) => ({ x: Math.max(0, Math.min(w - 1, Math.floor((e.clientX * w) / window.innerWidth))), y: Math.max(0, Math.min(h - 1, Math.floor((e.clientY * h) / window.innerHeight))) });
    const colorAt = (x: number, y: number) => {
      const i = (y * w + x) * 4;
      return `#${hex2(data[i])}${hex2(data[i + 1])}${hex2(data[i + 2])}`;
    };
    const lctx = lc.getContext("2d")!;
    lctx.imageSmoothingEnabled = false;
    function move(e: MouseEvent) {
      const { x, y } = at(e);
      lctx.clearRect(0, 0, lc.width, lc.height);
      const half = (N - 1) / 2;
      for (let j = 0; j < N; j++) {
        for (let i = 0; i < N; i++) {
          const px = x - half + i;
          const py = y - half + j;
          lctx.fillStyle = px >= 0 && py >= 0 && px < w && py < h ? colorAt(px, py) : "#000";
          lctx.fillRect(i * Z, j * Z, Z, Z);
        }
      }
      lctx.strokeStyle = "#fff";
      lctx.lineWidth = 2;
      lctx.strokeRect(half * Z, half * Z, Z, Z);
      lctx.strokeStyle = "#000";
      lctx.lineWidth = 1;
      lctx.strokeRect(half * Z + 1, half * Z + 1, Z - 2, Z - 2);
      label.textContent = colorAt(x, y).toUpperCase();
      const lw = N * Z + 4;
      loupe.style.left = `${Math.min(window.innerWidth - lw - 8, e.clientX + 20)}px`;
      loupe.style.top = `${Math.min(window.innerHeight - lw - 40, e.clientY + 20)}px`;
    }
    function done(v: string | null) {
      window.removeEventListener("keydown", key, true);
      root.remove();
      resolve(v);
    }
    function key(e: KeyboardEvent) {
      e.stopPropagation();
      e.preventDefault();
      if (e.key === "Escape") done(null);
    }
    root.addEventListener("mousemove", move);
    root.addEventListener("mousedown", (e) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.button === 0) done(colorAt(at(e).x, at(e).y));
      else done(null); // clic dereito ou central: cancela
    });
    root.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      done(null);
    });
    window.addEventListener("keydown", key, true);
    document.body.appendChild(root);
  });
}
