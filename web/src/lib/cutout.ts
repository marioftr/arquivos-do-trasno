/** Quitar o fondo de unha imaxe no navegador: unha rede de segmentación (ONNX Runtime, WebAssembly) saca a máscara do suxeito.
 *  Os modelos son U²-Net pequeno e IS-Net (Apache-2.0), descárganse de Hugging Face e o navegador gárdaos. */
import { tr } from "@app/lib/i18n";
import { UserError, type Ctx } from "./files";
import { render } from "./image";

export type Quality = "fast" | "best";

const MODELS: Record<Quality, { url: string; size: number; mean: [number, number, number]; std: [number, number, number]; norm: "u2net" | "isnet" }> = {
  fast: { url: "https://huggingface.co/BritishWerewolf/U-2-Netp/resolve/main/onnx/model.onnx", size: 320, mean: [0.485, 0.456, 0.406], std: [0.229, 0.224, 0.225], norm: "u2net" },
  best: { url: "https://huggingface.co/Ko033/isnet-general-use-onnx/resolve/main/onnx/model_quantized.onnx", size: 1024, mean: [0.5, 0.5, 0.5], std: [1, 1, 1], norm: "isnet" },
};

type Ort = typeof import("onnxruntime-web");
type Session = import("onnxruntime-web").InferenceSession;
let ortLib: Ort | null = null;
const sessions = new Map<Quality, Promise<Session>>();

async function ort(): Promise<Ort> {
  if (ortLib) return ortLib;
  const m = (await import("onnxruntime-web/wasm")) as unknown as Ort;
  m.env.wasm.wasmPaths = { wasm: "/ort/ort-wasm-simd-threaded.wasm" } as never;
  m.env.wasm.numThreads = 1;
  ortLib = m;
  return m;
}

/** O modelo, da caché do navegador ou descargado dicindo canto vai. */
async function modelBytes(url: string, ctx: Ctx, label: string): Promise<ArrayBuffer> {
  const cache = await caches.open("trasno-models").catch(() => null);
  const hit = await cache?.match(url);
  if (hit) {
    ctx.progress(null, tr("Cargando o modelo…"));
    return hit.arrayBuffer();
  }
  ctx.progress(0, label);
  const res = await fetch(url);
  if (!res.ok || !res.body) throw new UserError(tr("Non se puido descargar ou arrancar o modelo. Comproba a conexión e proba de novo."));
  const total = Number(res.headers.get("content-length")) || 0;
  const reader = res.body.getReader();
  const chunks: Uint8Array[] = [];
  let got = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    got += value.length;
    if (total) ctx.progress(got / total, label);
  }
  const blob = new Blob(chunks as BlobPart[]);
  await cache?.put(url, new Response(blob)).catch(() => {});
  return blob.arrayBuffer();
}

function session(q: Quality, ctx: Ctx): Promise<Session> {
  const have = sessions.get(q);
  if (have) return have;
  const p = (async () => {
    const lib = await ort();
    const bytes = await modelBytes(MODELS[q].url, ctx, tr("Descargando o modelo…"));
    return lib.InferenceSession.create(bytes, { executionProviders: ["wasm"] });
  })().catch((e) => {
    sessions.delete(q);
    console.error(e);
    throw e instanceof UserError ? e : new UserError(tr("Non se puido descargar ou arrancar o modelo. Comproba a conexión e proba de novo."));
  });
  sessions.set(q, p);
  return p;
}

export type Backdrop = "clear" | "white" | "black";

/** Quita o fondo: devolve un PNG (transparente, ou co fondo liso que se pida). */
export async function removeBackground(file: File, q: Quality, backdrop: Backdrop, ctx: Ctx): Promise<Blob> {
  const m = MODELS[q];
  const sess = await session(q, ctx);
  ctx.progress(null, tr("Quitando o fondo…"));
  await new Promise((r) => setTimeout(r));
  // A imaxe orixinal, coa orientación da foto e nun tamaño que o móbil aguante.
  const base = await render(file, { mime: "image/png", maxSide: 2400 });
  const bmp = await createImageBitmap(base.blob);
  const W = bmp.width;
  const H = bmp.height;

  // Entrada: a imaxe esticada ao cadrado da rede e normalizada.
  const S = m.size;
  const small = document.createElement("canvas");
  small.width = small.height = S;
  const sg = small.getContext("2d", { willReadFrequently: true })!;
  sg.drawImage(bmp, 0, 0, S, S);
  const px = sg.getImageData(0, 0, S, S).data;
  const input = new Float32Array(3 * S * S);
  for (let i = 0; i < S * S; i++) for (let c = 0; c < 3; c++) input[c * S * S + i] = (px[i * 4 + c] / 255 - m.mean[c]) / m.std[c];
  const lib = await ort();
  const out = await sess.run({ [sess.inputNames[0]]: new lib.Tensor("float32", input, [1, 3, S, S]) });
  const raw = out[sess.outputNames[0]].data as ArrayLike<number>;

  // Máscara 0–255: o modelo rápido xa a dá en bytes; o outro en probabilidades (escálase ao rango real).
  const n = S * S;
  const alpha = new Uint8ClampedArray(n);
  let lo = Infinity;
  let hi = -Infinity;
  for (let i = 0; i < n; i++) {
    const v = raw[i];
    if (v < lo) lo = v;
    if (v > hi) hi = v;
  }
  const span = hi - lo || 1;
  for (let i = 0; i < n; i++) alpha[i] = m.norm === "u2net" && hi > 1.5 ? raw[i] : ((raw[i] - lo) / span) * 255;

  const maskCanvas = document.createElement("canvas");
  maskCanvas.width = maskCanvas.height = S;
  const mg = maskCanvas.getContext("2d")!;
  const img = mg.createImageData(S, S);
  for (let i = 0; i < n; i++) img.data[i * 4 + 3] = alpha[i];
  mg.putImageData(img, 0, 0);

  // Composición: orixinal con alfa = máscara (escalada con suavizado) e, se se pide, sobre un fondo liso.
  const outCanvas = document.createElement("canvas");
  outCanvas.width = W;
  outCanvas.height = H;
  const g = outCanvas.getContext("2d")!;
  g.imageSmoothingQuality = "high";
  g.drawImage(bmp, 0, 0);
  g.globalCompositeOperation = "destination-in";
  g.drawImage(maskCanvas, 0, 0, W, H);
  bmp.close();
  let final = outCanvas;
  if (backdrop !== "clear") {
    final = document.createElement("canvas");
    final.width = W;
    final.height = H;
    const f = final.getContext("2d")!;
    f.fillStyle = backdrop === "white" ? "#fff" : "#000";
    f.fillRect(0, 0, W, H);
    f.drawImage(outCanvas, 0, 0);
  }
  const blob = await new Promise<Blob | null>((r) => final.toBlob(r, "image/png"));
  if (!blob) throw new UserError(tr("Algo fallou ao procesar. Proba con outro ficheiro."));
  return blob;
}
