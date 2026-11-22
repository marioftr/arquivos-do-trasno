/** Núcleo da IA no navegador (transformers.js + ONNX Runtime en WebAssembly), compartido polo programa de escritorio e pola web:
 *  carga de modelos con progreso e tradución de textos con Opus-MT. Os modelos descárganse de Hugging Face a primeira vez que se usan
 *  (o navegador gárdaos) e o procesado é todo no aparello. */
import { tr } from "./i18n";

/** Un erro que se lle di tal cal á persoa (o texto xa vai traducido). */
export class UserError extends Error {}

/** O que unha ferramenta lle di á pantalla mentres traballa: fracción feita (null se non se sabe) e que está facendo. */
export interface Ctx {
  progress(frac: number | null, label?: string): void;
}

export type Lang = "glg" | "spa" | "eng" | "cat";

type TF = typeof import("@huggingface/transformers");
let tf: TF | null = null;

export async function lib(): Promise<TF> {
  if (tf) return tf;
  const m = await import("@huggingface/transformers");
  m.env.allowLocalModels = false;
  // O ONNX Runtime serve o a propia aplicación (nada de CDN): ver scripts/copy-web-assets.mjs e copy-ocr.mjs.
  m.env.backends.onnx.wasm!.wasmPaths = { mjs: "/ort/ort-wasm-simd-threaded.mjs", wasm: "/ort/ort-wasm-simd-threaded.wasm" } as never;
  tf = m;
  return m;
}

/** Dice canto vai da descarga de un modelo (suma de todos os seus ficheiros). */
export function downloads(ctx: Ctx, label: string) {
  const files = new Map<string, { loaded: number; total: number }>();
  // Se o modelo xa está gardado non hai descarga: «Cargando», e só se di «Descargando» cando chegan os primeiros datos.
  ctx.progress(null, tr("Cargando o modelo…"));
  return (e: { status: string; file?: string; loaded?: number; total?: number }) => {
    if (e.status === "progress" && e.file && e.total) {
      files.set(e.file, { loaded: e.loaded ?? 0, total: e.total });
      let l = 0;
      let t = 0;
      for (const f of files.values()) (l += f.loaded), (t += f.total);
      ctx.progress(t ? l / t : null, label);
    }
  };
}

const pipes = new Map<string, Promise<any>>();

/** Carga un modelo (cunha pipeline) dicindo canto vai da descarga. */
export function load(task: string, model: string, ctx: Ctx, label: string): Promise<any> {
  const key = `${task}|${model}`;
  const have = pipes.get(key);
  if (have) return have;
  const p = (async () => {
    const m = await lib();
    return await (m.pipeline as any)(task, model, { dtype: "q8", device: "wasm", progress_callback: downloads(ctx, label) });
  })().catch((e) => {
    pipes.delete(key);
    console.error(e);
    throw new UserError(tr("Non puiden descargar ou arrancar o modelo. Comproba a conexión e proba de novo."));
  });
  pipes.set(key, p);
  return p;
}

// ---------------------------------------------------------------- tradución

interface MT {
  tok: any;
  model: any;
}
const mts = new Map<string, Promise<MT>>();
function loadMT(name: string, ctx: Ctx, label: string): Promise<MT> {
  const have = mts.get(name);
  if (have) return have;
  const p = (async () => {
    const m = await lib();
    const progress_callback = downloads(ctx, label);
    const [tok, model] = await Promise.all([
      (m.AutoTokenizer as any).from_pretrained(name, { progress_callback }),
      (m.AutoModelForSeq2SeqLM as any).from_pretrained(name, { dtype: "q8", device: "wasm", progress_callback }),
    ]);
    return { tok, model };
  })().catch((e) => {
    mts.delete(name);
    console.error(e);
    throw new UserError(tr("Non puiden descargar ou arrancar o modelo. Comproba a conexión e proba de novo."));
  });
  mts.set(name, p);
  return p;
}

/** Traduce unha frase. A etiqueta de idioma de destino (`>>gl<<`) métese polo seu número: o tokenizador da web non a recoñece como palabra especial. */
async function translateOne({ tok, model }: MT, text: string, tag: string | null): Promise<string> {
  const m = await lib();
  const ids: number[] = [...(tag ? [tok.convert_tokens_to_ids([tag])[0]] : []), ...tok.encode(text)];
  const out = await model.generate({
    input_ids: new m.Tensor("int64", BigInt64Array.from(ids.map(BigInt)), [1, ids.length]),
    attention_mask: new m.Tensor("int64", BigInt64Array.from(ids.map(() => 1n)), [1, ids.length]),
    max_new_tokens: Math.min(256, ids.length * 3 + 12),
    num_beams: 1,
    do_sample: false,
  });
  return (tok.batch_decode(out, { skip_special_tokens: true })[0] as string).replace(/\s+/g, " ").trim();
}

// Os destinos do Opus-MT románico levan o código de dúas letras.
const TAG: Partial<Record<Lang, string>> = { glg: ">>gl<<", spa: ">>es<<", cat: ">>ca<<" };

/** Traduce textos: calquera lingua románica → inglés → galego, castelán ou catalán (Opus-MT; o inglés é a ponte). Unha frase de cada vez:
 *  en lotes con orixes de lonxitude distinta o modelo non remata a frase. */
export async function translate(texts: string[], from: Lang, to: Lang, ctx: Ctx): Promise<string[]> {
  if (from === to || !texts.length) return texts;
  const label = tr("Descargando o modelo de tradución (só a primeira vez)…");
  const steps: { name: string; tag: string | null }[] = [];
  if (from !== "eng") steps.push({ name: "Xenova/opus-mt-ROMANCE-en", tag: null });
  if (to !== "eng") steps.push({ name: "Xenova/opus-mt-en-ROMANCE", tag: TAG[to] ?? null });
  let cur = texts;
  for (const [k, st] of steps.entries()) {
    const mt = await loadMT(st.name, ctx, label);
    const out: string[] = [];
    for (const [i, t] of cur.entries()) {
      if (i % 4 === 0) {
        ctx.progress((k + i / cur.length) / steps.length, tr("Traducindo…"));
        await new Promise((r) => setTimeout(r)); // deixa pintar a barra
      }
      out.push(t.trim() ? await translateOne(mt, t, st.tag) : t);
    }
    cur = out;
  }
  return cur;
}

if (import.meta.env.DEV) (window as unknown as { __ai: unknown }).__ai = { load, lib };
