/** Audio e vídeo no navegador con FFmpeg en WebAssembly (ffmpeg.wasm). O núcleo (uns 30 MB, GPL) non vai na web: descárgase a petición desde jsDelivr e o navegador gárdao. */
import { tr } from "@app/lib/i18n";
import { UserError, type Ctx } from "./files";

const CORE = "https://cdn.jsdelivr.net/npm/@ffmpeg/core@0.12.10/dist/esm";
/** O navegador de un móbil non aguanta máis (o ficheiro entra dúas veces na memoria: lido e procesado). */
const MAX_BYTES = 250 * 1024 * 1024;

type FFmpegT = import("@ffmpeg/ffmpeg").FFmpeg;
let ff: FFmpegT | null = null;
let loading: Promise<FFmpegT> | null = null;
let current: Ctx | null = null;

/** Descarga un ficheiro dicindo canto vai (o núcleo pesa moito) e devolve unha URL local para o motor. */
async function fetchBlobURL(url: string, type: string, onFrac: (f: number) => void): Promise<string> {
  const res = await fetch(url);
  if (!res.ok || !res.body) throw new UserError(tr("Non puiden descargar o motor de audio e vídeo. Comproba a conexión."));
  const total = Number(res.headers.get("content-length")) || 0;
  const chunks: Uint8Array[] = [];
  let got = 0;
  const reader = res.body.getReader();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    got += value.length;
    if (total) onFrac(got / total);
  }
  return URL.createObjectURL(new Blob(chunks as BlobPart[], { type }));
}

async function engine(ctx: Ctx): Promise<FFmpegT> {
  current = ctx;
  if (ff) return ff;
  loading ??= (async () => {
    const { FFmpeg } = await import("@ffmpeg/ffmpeg");
    const f = new FFmpeg();
    f.on("progress", ({ progress }) => current?.progress(Math.max(0, Math.min(1, progress))));
    const label = tr("Descargando o motor de audio e vídeo (só a primeira vez)…");
    ctx.progress(0, label);
    const coreURL = await fetchBlobURL(`${CORE}/ffmpeg-core.js`, "text/javascript", () => {});
    const wasmURL = await fetchBlobURL(`${CORE}/ffmpeg-core.wasm`, "application/wasm", (p) => current?.progress(p, label));
    await f.load({ coreURL, wasmURL });
    ff = f;
    return f;
  })().catch((e) => {
    loading = null;
    console.error(e);
    throw e instanceof UserError ? e : new UserError(tr("Non puiden arrancar o motor de audio e vídeo neste navegador."));
  });
  return loading;
}

/** Pasa un ficheiro polo motor: escribe a entrada, executa `args(in, out)` e devolve o que saíu. */
export async function run(file: File, outExt: string, args: (input: string, output: string) => string[], ctx: Ctx, label: string): Promise<Blob> {
  if (file.size > MAX_BYTES) throw new UserError(tr("«{name}» pesa demasiado para o navegador (máximo 250 MB).", { name: file.name }));
  const f = await engine(ctx);
  ctx.progress(0, label);
  const input = `in-${Date.now()}.${(/\.([a-z0-9]+)$/i.exec(file.name)?.[1] ?? "bin").toLowerCase()}`;
  const output = `out-${Date.now()}.${outExt}`;
  await f.writeFile(input, new Uint8Array(await file.arrayBuffer()));
  try {
    const code = await f.exec(args(input, output));
    if (code !== 0) throw new UserError(tr("Non puiden procesar «{name}». O ficheiro está danado ou ese formato non se admite.", { name: file.name }));
    const data = (await f.readFile(output)) as Uint8Array;
    return new Blob([data.slice().buffer as ArrayBuffer]);
  } finally {
    await f.deleteFile(input).catch(() => {});
    await f.deleteFile(output).catch(() => {});
  }
}

/** «1:30», «01:02:03», «45» ou «12,5» → segundos (null se non se entende). */
export function parseTime(text: string): number | null {
  const t = text.trim().replace(",", ".");
  if (!t) return null;
  const parts = t.split(":");
  if (parts.length > 3 || parts.some((p) => !/^\d+(\.\d+)?$/.test(p))) return null;
  return parts.reduce((a, p) => a * 60 + Number(p), 0);
}

/** O son de un ficheiro (audio ou vídeo) como mono de 16 kHz en Float32: o que lle fai falta a Whisper. */
export async function pcm16k(file: File, ctx: Ctx, label: string): Promise<Float32Array> {
  const blob = await run(file, "raw", (i, o) => ["-i", i, "-vn", "-ac", "1", "-ar", "16000", "-f", "f32le", o], ctx, label);
  return new Float32Array(await blob.arrayBuffer());
}
