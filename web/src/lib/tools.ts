/** As ferramentas da versión web, descritas con datos: a pantalla (Workbench) constrúese soa a partir de cada unha. */
import { tr } from "@app/lib/i18n";
import { EXT, encoders, render, sameMime, transform, type Mime } from "./image";
import { lang } from "@app/lib/i18n";
import { get } from "svelte/store";
import { formatBytes, stemOf, uniqueName, UserError, type Ctx, type Output } from "./files";

export type Values = Record<string, string>;
export interface Choice {
  value: string;
  label: string;
  /** Valor ou detalle pequeno baixo o nome («85», «1280 px»). */
  sub?: string;
  /** Modelo de IA que fai falta con esta opción (ver models.ts): se xa está descargado, amósase no seu sitio. */
  model?: string;
}
export type Opt =
  | { id: string; type: "seg"; label: string; info?: string; choices: () => Choice[]; when?: (v: Values) => boolean }
  | { id: string; type: "text"; label: string; info?: string; placeholder: string; when?: (v: Values) => boolean };

export interface Tool {
  id: string;
  group: "imaxe" | "pdf" | "audio" | "subs";
  icon: string;
  title: string;
  desc: string;
  /** A «i» do título: o que fai, en poucas palabras. */
  info: string;
  accept: string;
  multiple: boolean;
  /** A orde dos ficheiros importa (unir): hai botóns para subir e baixar. */
  ordered?: boolean;
  /** Cantos ficheiros fan falta polo menos (1 se non se di). */
  min?: number;
  options: Opt[];
  defaults: Values;
  /** Texto do botón verde segundo cantos ficheiros hai. */
  action: (n: number) => string;
  /** Un ficheiro, unha saída: os fallos sáltanse e dinse. */
  each?: (file: File, v: Values, taken: Set<string>, ctx: Ctx) => Promise<Output>;
  /** Todos os ficheiros xuntos. */
  all?: (files: File[], v: Values, ctx: Ctx) => Promise<Output[]>;
}

export const IMG = "image/*,.heic,.heif";
export const MEDIA = "audio/*,video/*,.mp3,.m4a,.wav,.ogg,.opus,.flac,.mp4,.mov,.mkv,.webm";
type Lang = "glg" | "spa" | "eng" | "cat";
const LANGS = (): Choice[] => [
  { value: "glg", label: "Galego" },
  { value: "spa", label: "Castelán" },
  { value: "eng", label: "Inglés" },
  { value: "cat", label: "Catalán" },
];
const DEFAULT_LANG = (): Lang => (({ gl: "glg", es: "spa", en: "eng" } as Record<string, Lang>)[get(lang)] ?? "glg");
export const MIME_OF: Record<string, string> = { mp3: "audio/mpeg", m4a: "audio/mp4", wav: "audio/wav", opus: "audio/ogg", gif: "image/gif" };

const QUALITY = (): Choice[] => [
  { value: "95", label: tr("Máxima"), sub: "95" },
  { value: "85", label: tr("Alta"), sub: "85" },
  { value: "70", label: tr("Media"), sub: "70" },
  { value: "50", label: tr("Baixa"), sub: "50" },
];
const mimeChoices = (withSame = false): Choice[] => [
  ...(withSame ? [{ value: "same", label: tr("Igual") }] : []),
  { value: "image/jpeg", label: "JPG" },
  { value: "image/png", label: "PNG" },
  ...(encoders.webp ? [{ value: "image/webp", label: "WebP" }] : []),
];

export const outName = (file: File, mime: Mime, taken: Set<string>, suffix = "") => uniqueName(`${stemOf(file.name)}${suffix}.${EXT[mime]}`, taken);

/** Axustes que cambian o tamaño en bytes: o que se di ao rematar cada imaxe. */
export const sizeNote = (file: File, blob: Blob, capped: boolean) => {
  const delta = blob.size - file.size;
  const pct = file.size ? Math.round((Math.abs(delta) / file.size) * 100) : 0;
  const base = `${formatBytes(file.size)} → ${formatBytes(blob.size)}${pct && Math.abs(delta) > 512 ? ` (${delta < 0 ? "−" : "+"}${pct} %)` : ""}`;
  return capped ? `${base} · ${tr("Reducida ao máximo que admite o navegador.")}` : base;
};

export const mimeOf = (v: Values, file: File): Mime => (v.format && v.format !== "same" ? (v.format as Mime) : sameMime(file));
export const qualityOf = (v: Values) => Number(v.quality || 85) / 100;

/** Constrúese ao pedilas (non ao cargar o módulo): os textos van no idioma que haxa nese momento. */
export const buildTools = (): Tool[] => [
  {
    id: "converter",
    group: "imaxe",
    icon: "convert",
    title: tr("Converter imaxes"),
    desc: tr("A JPG, PNG ou WebP. Quita os metadatos."),
    info: tr("Cambia o formato. Quita os metadatos, como a localización. O JPG non admite transparencia: o que era transparente queda en branco."),
    accept: IMG,
    multiple: true,
    options: [
      { id: "format", type: "seg", label: tr("Formato"), choices: () => mimeChoices() },
      { id: "quality", type: "seg", label: tr("Calidade"), info: tr("Máis calidade pesa máis. A calidade 85 case non se distingue do orixinal."), choices: QUALITY, when: (v) => v.format !== "image/png" },
    ],
    defaults: { format: "image/jpeg", quality: "85" },
    action: (n) => (n === 1 ? tr("Converter 1 imaxe") : tr("Converter {n} imaxes", { n })),
    each: async (file, v, taken) => {
      const mime = mimeOf(v, file);
      const r = await render(file, { mime, quality: qualityOf(v) });
      return { name: outName(file, mime, taken), blob: r.blob, note: sizeNote(file, r.blob, r.capped) };
    },
  },
  {
    id: "redimensionar",
    group: "imaxe",
    icon: "resize",
    title: tr("Redimensionar imaxes"),
    desc: tr("Máis pequenas, por lado maior ou porcentaxe."),
    info: tr("Reduce o tamaño en píxeles sen cambiar a proporción. Non amplía imaxes pequenas. Mantense o formato."),
    accept: IMG,
    multiple: true,
    options: [
      {
        id: "mode",
        type: "seg",
        label: tr("Medida"),
        choices: () => [
          { value: "side", label: tr("Lado maior") },
          { value: "percent", label: tr("Porcentaxe") },
        ],
      },
      {
        id: "side",
        type: "seg",
        label: tr("Tamaño"),
        info: tr("O lado maior da imaxe, sexa ancho ou alto, queda neste valor."),
        when: (v) => v.mode === "side",
        choices: () => [
          { value: "640", label: tr("Pequeno"), sub: "640 px" },
          { value: "1280", label: tr("Mediano"), sub: "1280 px" },
          { value: "1920", label: tr("Grande"), sub: "1920 px" },
          { value: "2560", label: tr("Moi grande"), sub: "2560 px" },
        ],
      },
      {
        id: "percent",
        type: "seg",
        label: tr("Tamaño"),
        when: (v) => v.mode === "percent",
        choices: () => [
          { value: "75", label: "75 %" },
          { value: "50", label: "50 %" },
          { value: "25", label: "25 %" },
        ],
      },
    ],
    defaults: { mode: "side", side: "1280", percent: "50" },
    action: (n) => (n === 1 ? tr("Redimensionar 1 imaxe") : tr("Redimensionar {n} imaxes", { n })),
    each: async (file, v, taken) => {
      const mime = sameMime(file);
      const r = await render(file, { mime, quality: 0.9, ...(v.mode === "percent" ? { percent: Number(v.percent) } : { maxSide: Number(v.side) }) });
      return { name: outName(file, mime, taken, "-redimensionado"), blob: r.blob, note: `${r.width} × ${r.height} · ${formatBytes(r.blob.size)}${r.capped ? ` · ${tr("Reducida ao máximo que admite o navegador.")}` : ""}` };
    },
  },
  {
    id: "comprimir",
    group: "imaxe",
    icon: "compress",
    title: tr("Comprimir imaxes"),
    desc: tr("Pesan menos con pouca perda de calidade."),
    info: tr("Volve gardar a imaxe con menos calidade para que pese menos. Se xa estaba comprimida e non mellora, queda tal como estaba."),
    accept: IMG,
    multiple: true,
    options: [
      { id: "format", type: "seg", label: tr("Formato"), choices: () => mimeChoices(true).filter((c) => c.value !== "image/png") },
      { id: "quality", type: "seg", label: tr("Calidade"), choices: QUALITY },
    ],
    defaults: { format: "same", quality: "70" },
    action: (n) => (n === 1 ? tr("Comprimir 1 imaxe") : tr("Comprimir {n} imaxes", { n })),
    each: async (file, v, taken) => {
      let mime = mimeOf(v, file);
      if (mime === "image/png") mime = "image/jpeg"; // o PNG non perde calidade: comprimir é pasar a JPG
      const r = await render(file, { mime, quality: qualityOf(v) });
      if (r.blob.size >= file.size && file.type === mime) return { name: uniqueName(file.name, taken), blob: file, note: tr("Xa estaba comprimida: queda igual.") };
      return { name: outName(file, mime, taken, "-comprimido"), blob: r.blob, note: sizeNote(file, r.blob, r.capped) };
    },
  },
  {
    id: "restaurar",
    group: "imaxe",
    icon: "enhance",
    title: tr("Restaurar imaxes"),
    desc: tr("Luz, cor, ruído e nitidez. Tamén páxinas fotografadas."),
    info: tr("Corrixe o que a foto xa ten, sen inventar detalle. Automático equilibra luz e cor; Documento deixa o fondo branco e o texto negro."),
    accept: IMG,
    multiple: true,
    options: [
      {
        id: "mode",
        type: "seg",
        label: tr("Axuste"),
        choices: () => [
          { value: "auto", label: tr("Automático"), sub: tr("luz e cor") },
          { value: "light", label: tr("Máis luz"), sub: tr("levanta sombras") },
          { value: "doc", label: tr("Documento"), sub: tr("fondo branco") },
          { value: "bw", label: tr("Branco e negro") },
          { value: "denoise", label: tr("Menos ruído") },
          { value: "sharp", label: tr("Máis nitidez") },
        ],
      },
      {
        id: "amount",
        type: "seg",
        label: tr("Intensidade"),
        when: (v) => v.mode === "denoise" || v.mode === "sharp",
        choices: () => [
          { value: "soft", label: tr("Suave") },
          { value: "normal", label: tr("Normal") },
          { value: "strong", label: tr("Forte") },
        ],
      },
    ],
    defaults: { mode: "auto", amount: "normal" },
    action: (n) => (n === 1 ? tr("Restaurar imaxe") : tr("Restaurar {n} imaxes", { n })),
    each: async (file, v, taken) => {
      const { enhance, MAX_AREA } = await import("./enhance");
      const mime = v.mode === "doc" || v.mode === "bw" ? (sameMime(file) === "image/png" ? "image/png" : "image/jpeg") : sameMime(file);
      const r = await transform(file, mime, 0.92, MAX_AREA, (d) => enhance(d, v.mode as never, v.amount as never));
      return { name: outName(file, mime, taken, "-restaurado"), blob: r.blob, note: `${r.width} × ${r.height} · ${formatBytes(r.blob.size)}${r.capped ? ` · ${tr("Reducida ao máximo que admite o navegador.")}` : ""}` };
    },
  },
  {
    id: "xirar",
    group: "imaxe",
    icon: "rotate-cw",
    title: tr("Xirar imaxes"),
    desc: tr("Xira ou volta as imaxes."),
    info: tr("Xira ou volta as imaxes. O resultado garda a orientación xa corrixida, sen depender dos metadatos da cámara."),
    accept: IMG,
    multiple: true,
    options: [
      {
        id: "turn",
        type: "seg",
        label: tr("Xiro"),
        choices: () => [
          { value: "cw", label: tr("90° á dereita") },
          { value: "ccw", label: tr("90° á esquerda") },
          { value: "180", label: "180°" },
          { value: "flipH", label: tr("Voltear en horizontal") },
          { value: "flipV", label: tr("Voltear en vertical") },
        ],
      },
    ],
    defaults: { turn: "cw" },
    action: (n) => (n === 1 ? tr("Xirar 1 imaxe") : tr("Xirar {n} imaxes", { n })),
    each: async (file, v, taken) => {
      const mime = sameMime(file);
      const r = await render(file, {
        mime,
        quality: 0.92,
        rotate: v.turn === "cw" ? 90 : v.turn === "ccw" ? 270 : v.turn === "180" ? 180 : 0,
        flipH: v.turn === "flipH",
        flipV: v.turn === "flipV",
      });
      return { name: outName(file, mime, taken, "-xirado"), blob: r.blob, note: `${r.width} × ${r.height}` };
    },
  },
  {
    id: "ocr",
    group: "imaxe",
    icon: "scan-text",
    title: tr("Recoñecer texto"),
    desc: tr("Le o texto dunha foto ou captura."),
    info: tr("Le o texto das imaxes (OCR) e xera un TXT. Funciona mellor con texto recto e ben iluminado. Elixe o idioma do texto."),
    accept: IMG,
    multiple: true,
    options: [
      {
        id: "lang",
        type: "seg",
        label: tr("Idioma do texto"),
        choices: LANGS,
      },
    ],
    defaults: { lang: DEFAULT_LANG() },
    action: (n) => (n === 1 ? tr("Recoñecer texto") : tr("Recoñecer texto en {n} imaxes", { n })),
    each: async (file, v, taken, ctx) => {
      const { readText } = await import("./ocr");
      const r = await readText(file, v.lang, ctx, tr("Lendo o texto…"));
      if (!r.text) throw new UserError(tr("Non atopei texto nesta imaxe."));
      const words = r.text.split(/\s+/).filter(Boolean).length;
      return { name: uniqueName(`${stemOf(file.name)}.txt`, taken), blob: new Blob([r.text], { type: "text/plain;charset=utf-8" }), text: r.text, note: tr("{n} palabras", { n: words }) };
    },
  },
  {
    id: "quitar-fondo",
    group: "imaxe",
    icon: "cutout",
    title: tr("Quitar o fondo"),
    desc: tr("Deixa só o suxeito, con fondo transparente."),
    info: tr("Detecta o suxeito e quita o fondo. Con pelo ou bordos finos, o mellor modelo vai mellor."),
    accept: IMG,
    multiple: true,
    options: [
      {
        id: "quality",
        type: "seg",
        label: tr("Modelo"),
        choices: () => [
          { value: "fast", label: tr("Rápido"), sub: tr("uns {mb} MB", { mb: 5 }), model: "cut-fast" },
          { value: "best", label: tr("Mellor"), sub: tr("uns {mb} MB", { mb: 46 }), model: "cut-best" },
        ],
      },
      {
        id: "backdrop",
        type: "seg",
        label: tr("Fondo"),
        choices: () => [
          { value: "clear", label: tr("Transparente") },
          { value: "white", label: tr("Branco") },
          { value: "black", label: tr("Negro") },
        ],
      },
    ],
    defaults: { quality: "fast", backdrop: "clear" },
    action: (n) => (n === 1 ? tr("Quitar o fondo") : tr("Quitar o fondo a {n} imaxes", { n })),
    each: async (file, v, taken, ctx) => {
      const { removeBackground } = await import("./cutout");
      const blob = await removeBackground(file, v.quality as "fast" | "best", v.backdrop as "clear" | "white" | "black", ctx);
      return { name: uniqueName(`${stemOf(file.name)}-sen-fondo.png`, taken), blob, note: formatBytes(blob.size) };
    },
  },
  {
    id: "audio",
    group: "audio",
    icon: "wave",
    title: tr("Converter audio"),
    desc: tr("A MP3, M4A, WAV ou Opus. Tamén saca o son dun vídeo."),
    info: tr("Pasa audio a outro formato. Se o ficheiro é un vídeo, saca o son."),
    accept: MEDIA,
    multiple: true,
    options: [
      {
        id: "format",
        type: "seg",
        label: tr("Formato"),
        choices: () => [
          { value: "mp3", label: "MP3" },
          { value: "m4a", label: "M4A" },
          { value: "opus", label: "Opus" },
          { value: "wav", label: "WAV", sub: tr("sen perdas") },
        ],
      },
      {
        id: "kbps",
        type: "seg",
        label: tr("Calidade"),
        info: tr("Máis kbps pesa máis. 192 vale para case todo."),
        when: (v) => v.format !== "wav",
        choices: () => [
          { value: "320", label: tr("Máxima"), sub: "320 kbps" },
          { value: "192", label: tr("Alta"), sub: "192 kbps" },
          { value: "128", label: tr("Media"), sub: "128 kbps" },
          { value: "96", label: tr("Baixa"), sub: "96 kbps" },
        ],
      },
    ],
    defaults: { format: "mp3", kbps: "192" },
    action: (n) => (n === 1 ? tr("Converter 1 audio") : tr("Converter {n} audios", { n })),
    each: async (file, v, taken, ctx) => {
      const { run } = await import("./media");
      const codec = v.format === "wav" ? ["-c:a", "pcm_s16le"] : [`-c:a`, v.format === "m4a" ? "aac" : v.format === "opus" ? "libopus" : "libmp3lame", "-b:a", `${v.kbps}k`];
      const blob = await run(file, v.format, (i, o) => ["-i", i, "-vn", ...codec, o], ctx, tr("Convertendo…"));
      return { name: uniqueName(`${stemOf(file.name)}.${v.format}`, taken), blob: new Blob([blob], { type: MIME_OF[v.format] }), note: sizeNote(file, blob, false) };
    },
  },
  {
    id: "cortar",
    group: "audio",
    icon: "scissors",
    title: tr("Cortar audio ou vídeo"),
    desc: tr("Deixa só un tramo, sen perder calidade."),
    info: tr("Garda o tramo entre as dúas marcas sen volver codificar, así que é rápido. O corte vai ao fotograma clave máis próximo. Exemplos: 45, 1:30, 1:02:03"),
    accept: MEDIA,
    multiple: false,
    options: [
      { id: "from", type: "text", label: tr("Desde"), placeholder: "0:10" },
      { id: "to", type: "text", label: tr("Ata"), placeholder: "1:30" },
    ],
    defaults: { from: "", to: "" },
    action: () => tr("Cortar"),
    each: async (file, v, taken, ctx) => {
      const { run, parseTime } = await import("./media");
      const a = parseTime(v.from) ?? 0;
      const b = parseTime(v.to);
      if ((v.from.trim() && parseTime(v.from) === null) || (v.to.trim() && b === null)) throw new UserError(tr("Escribe o tempo así: 45, 1:30 ou 1:02:03."));
      if (b === null && !v.from.trim()) throw new UserError(tr("Escribe desde cando ata cando queres o tramo."));
      if (b !== null && b <= a) throw new UserError(tr("A marca final ten que ser posterior á inicial."));
      const ext = (/\.([a-z0-9]+)$/i.exec(file.name)?.[1] ?? "mp4").toLowerCase();
      const blob = await run(file, ext, (i, o) => ["-ss", String(a), "-i", i, ...(b !== null ? ["-t", String(b - a)] : []), "-c", "copy", "-avoid_negative_ts", "make_zero", o], ctx, tr("Cortando…"));
      return { name: uniqueName(`${stemOf(file.name)}-cortado.${ext}`, taken), blob: new Blob([blob], { type: file.type }), note: formatBytes(blob.size) };
    },
  },
  {
    id: "gif",
    group: "audio",
    icon: "film",
    title: tr("Vídeo a GIF"),
    desc: tr("Un GIF animado dun tramo curto."),
    info: tr("Fai un GIF desde o comezo do vídeo. Canto máis grande e longo, máis pesa."),
    accept: "video/*,.mp4,.mov,.mkv,.webm",
    multiple: false,
    options: [
      {
        id: "width",
        type: "seg",
        label: tr("Tamaño"),
        choices: () => [
          { value: "320", label: tr("Pequeno"), sub: "320 px" },
          { value: "480", label: tr("Mediano"), sub: "480 px" },
          { value: "640", label: tr("Grande"), sub: "640 px" },
        ],
      },
      {
        id: "secs",
        type: "seg",
        label: tr("Duración"),
        choices: () => [
          { value: "5", label: "5 s" },
          { value: "10", label: "10 s" },
          { value: "15", label: "15 s" },
        ],
      },
    ],
    defaults: { width: "480", secs: "5" },
    action: () => tr("Crear GIF"),
    each: async (file, v, taken, ctx) => {
      const { run } = await import("./media");
      const vf = `fps=12,scale=${v.width}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse`;
      const blob = await run(file, "gif", (i, o) => ["-t", v.secs, "-i", i, "-vf", vf, "-loop", "0", o], ctx, tr("Creando o GIF…"));
      return { name: uniqueName(`${stemOf(file.name)}.gif`, taken), blob: new Blob([blob], { type: "image/gif" }), note: formatBytes(blob.size) };
    },
  },
  {
    id: "transcribir",
    group: "audio",
    icon: "mic",
    title: tr("Transcribir"),
    desc: tr("Pasa a voz a subtítulos ou texto."),
    info: tr("Pasa a voz dun audio ou vídeo a texto con subtítulos, e opcionalmente tradúceo."),
    accept: MEDIA,
    multiple: false,
    options: [
      { id: "lang", type: "seg", label: tr("Idioma da voz"), choices: LANGS },
      {
        id: "model",
        type: "seg",
        label: tr("Modelo"),
        info: tr("O rápido recoñece peor, sobre todo con ruído ou acentos. O mellor vai máis lento e usa máis memoria."),
        choices: () => [
          { value: "tiny", label: tr("Rápido"), sub: tr("uns {mb} MB", { mb: 40 }), model: "asr-tiny" },
          { value: "base", label: tr("Mellor"), sub: tr("uns {mb} MB", { mb: 80 }), model: "asr-base" },
        ],
      },
      { id: "to", type: "seg", label: tr("Traducir a"), info: tr("Tradución automática: serve para entender, non para publicar sen revisar."), choices: () => [{ value: "none", label: tr("Non traducir") }, ...LANGS()] },
      {
        id: "format",
        type: "seg",
        label: tr("Formato"),
        choices: () => [
          { value: "srt", label: "SRT" },
          { value: "vtt", label: "VTT" },
          { value: "txt", label: "TXT" },
        ],
      },
    ],
    defaults: { lang: DEFAULT_LANG(), model: "tiny", to: "none", format: "srt" },
    action: () => tr("Transcribir"),
    all: async ([file], v, ctx) => {
      const [{ pcm16k }, ai, subs] = await Promise.all([import("./media"), import("./ai"), import("./subs")]);
      const pcm = await pcm16k(file, ctx, tr("Preparando o son…"));
      const cues = await ai.transcribe(pcm, v.lang as Lang, v.model as "tiny" | "base", ctx);
      if (!cues.length) throw new UserError(tr("Non atopei voz neste ficheiro."));
      const write = (list: typeof cues) => (v.format === "vtt" ? subs.toVtt(list) : v.format === "txt" ? subs.toTxt(list) : subs.toSrt(list));
      const out: Output[] = [{ name: `${stemOf(file.name)}.${v.format}`, blob: new Blob([write(cues)], { type: "text/plain;charset=utf-8" }), text: subs.toTxt(cues), note: tr("{n} subtítulos", { n: cues.length }) }];
      if (v.to !== "none" && v.to !== v.lang) {
        const lines = await ai.translate(cues.map((c) => c.text.replace(/\s+/g, " ")), v.lang as Lang, v.to as Lang, ctx);
        const done = cues.map((c, i) => ({ ...c, text: lines[i] ?? c.text }));
        out.push({ name: `${stemOf(file.name)}.${v.to}.${v.format}`, blob: new Blob([write(done)], { type: "text/plain;charset=utf-8" }), text: subs.toTxt(done), note: tr("Traducido automaticamente") });
      }
      return out;
    },
  },
  {
    id: "traducir",
    group: "subs",
    icon: "translate",
    title: tr("Traducir subtítulos"),
    desc: tr("Dun SRT ou VTT a galego, castelán, catalán ou inglés."),
    info: tr("Traduce un ficheiro de subtítulos mantendo os tempos. É tradución automática: serve para entender, non para publicar sen revisar."),
    accept: ".srt,.vtt,text/vtt",
    multiple: false,
    options: [
      { id: "from", type: "seg", label: tr("Está en"), choices: LANGS },
      { id: "to", type: "seg", label: tr("Traducir a"), choices: LANGS },
    ],
    defaults: { from: "eng", to: DEFAULT_LANG() },
    action: () => tr("Traducir"),
    all: async ([file], v, ctx) => {
      const [ai, subs] = await Promise.all([import("./ai"), import("./subs")]);
      const cues = subs.parseSubs(await file.text());
      if (!cues.length) throw new UserError(tr("Non atopei subtítulos neste ficheiro."));
      if (v.from === v.to) throw new UserError(tr("Elixe dous idiomas distintos."));
      const lines = await ai.translate(cues.map((c) => c.text.replace(/\s+/g, " ")), v.from as Lang, v.to as Lang, ctx);
      const done = cues.map((c, i) => ({ ...c, text: lines[i] ?? c.text }));
      const vtt = /\.vtt$/i.test(file.name);
      return [{ name: `${stemOf(file.name)}.${v.to}.${vtt ? "vtt" : "srt"}`, blob: new Blob([vtt ? subs.toVtt(done) : subs.toSrt(done)], { type: "text/plain;charset=utf-8" }), text: subs.toTxt(done), note: tr("{n} subtítulos", { n: done.length }) }];
    },
  },
  {
    id: "unir-pdf",
    group: "pdf",
    icon: "merge-doc",
    title: tr("Unir PDF"),
    desc: tr("Xunta varios PDF nun só."),
    info: tr("Xunta os PDF na orde da lista: usa as frechas para mover cada un."),
    accept: "application/pdf,.pdf",
    multiple: true,
    ordered: true,
    min: 2,
    options: [],
    defaults: {},
    action: (n) => tr("Unir {n} PDF", { n }),
    all: async (files, _v, ctx) => (await import("./pdf")).merge(files, ctx),
  },
  {
    id: "dividir-pdf",
    group: "pdf",
    icon: "split-doc",
    title: tr("Dividir PDF"),
    desc: tr("Extrae ou separa páxinas."),
    info: tr("Extrae páxinas nun PDF, separa por intervalos ou fai un PDF por páxina. Os intervalos escríbense así: 1-3, 5, 8-"),
    accept: "application/pdf,.pdf",
    multiple: false,
    options: [
      {
        id: "mode",
        type: "seg",
        label: tr("Como"),
        choices: () => [
          { value: "extract", label: tr("Extraer"), sub: tr("un PDF") },
          { value: "ranges", label: tr("Por intervalos"), sub: tr("un PDF cada un") },
          { value: "each", label: tr("Cada páxina"), sub: tr("un PDF cada unha") },
        ],
      },
      { id: "ranges", type: "text", label: tr("Páxinas"), placeholder: "1-3, 5, 8-", when: (v) => v.mode !== "each" },
    ],
    defaults: { mode: "extract", ranges: "" },
    action: () => tr("Dividir PDF"),
    all: async (files, v) => (await import("./pdf")).split(files[0], v.mode as "extract" | "ranges" | "each", v.ranges),
  },
  {
    id: "xirar-pdf",
    group: "pdf",
    icon: "rotate-ccw",
    title: tr("Xirar PDF"),
    desc: tr("Xira todas as páxinas."),
    info: tr("Xira todas as páxinas do PDF. O contido non se volve gardar como imaxe."),
    accept: "application/pdf,.pdf",
    multiple: true,
    options: [
      {
        id: "angle",
        type: "seg",
        label: tr("Xiro"),
        choices: () => [
          { value: "90", label: tr("90° á dereita") },
          { value: "270", label: tr("90° á esquerda") },
          { value: "180", label: "180°" },
        ],
      },
    ],
    defaults: { angle: "90" },
    action: (n) => (n === 1 ? tr("Xirar 1 PDF") : tr("Xirar {n} PDF", { n })),
    each: async (file, v) => (await import("./pdf")).rotate(file, Number(v.angle)),
  },
  {
    id: "imaxes-a-pdf",
    group: "pdf",
    icon: "image-doc",
    title: tr("Imaxes a PDF"),
    desc: tr("Un PDF coas imaxes, unha por páxina."),
    info: tr("Fai un PDF cunha imaxe por páxina, na orde da lista. As fotos quedan coa orientación correcta."),
    accept: IMG,
    multiple: true,
    ordered: true,
    options: [
      {
        id: "page",
        type: "seg",
        label: tr("Páxina"),
        choices: () => [
          { value: "image", label: tr("Tamaño da imaxe") },
          { value: "a4", label: "A4" },
        ],
      },
    ],
    defaults: { page: "a4" },
    action: (n) => (n === 1 ? tr("Crear PDF cunha imaxe") : tr("Crear PDF con {n} imaxes", { n })),
    all: async (files, v, ctx) => (await import("./pdf")).imagesToPdf(files, v.page as "image" | "a4", ctx),
  },
  {
    id: "pdf-imaxes",
    group: "pdf",
    icon: "image-pdf",
    title: tr("PDF a imaxes"),
    desc: tr("Unha imaxe por cada páxina."),
    info: tr("Garda cada páxina do PDF como imaxe. Máis resolución dá máis detalle e máis peso. Ata 150 páxinas."),
    accept: "application/pdf,.pdf",
    multiple: false,
    options: [
      { id: "format", type: "seg", label: tr("Formato"), choices: () => [{ value: "image/jpeg", label: "JPG" }, { value: "image/png", label: "PNG" }] },
      {
        id: "ppp",
        type: "seg",
        label: tr("Resolución"),
        choices: () => [
          { value: "100", label: tr("Baixa"), sub: "100 ppp" },
          { value: "150", label: tr("Media"), sub: "150 ppp" },
          { value: "220", label: tr("Alta"), sub: "220 ppp" },
        ],
      },
    ],
    defaults: { format: "image/jpeg", ppp: "150" },
    action: () => tr("Sacar as imaxes"),
    all: async ([file], v, ctx) => (await import("./pdfimg")).pdfToImages(file, v.format as "image/jpeg" | "image/png", Number(v.ppp), ctx),
  },
  {
    id: "pdf-texto",
    group: "pdf",
    icon: "text-doc",
    title: tr("PDF a texto"),
    desc: tr("Saca o texto dun PDF a un TXT."),
    info: tr("Copia o texto seleccionable do PDF. Se é unha dixitalización non hai texto: saca as imaxes e usa Recoñecer texto."),
    accept: "application/pdf,.pdf",
    multiple: false,
    options: [],
    defaults: {},
    action: () => tr("Sacar o texto"),
    all: async ([file], _v, ctx) => [await (await import("./pdfimg")).pdfToText(file, ctx)],
  },
  {
    id: "comprimir-pdf",
    group: "pdf",
    icon: "compress-doc",
    title: tr("Comprimir PDF"),
    desc: tr("Pesa moito menos, sobre todo se é unha dixitalización."),
    info: tr("Volve gardar cada páxina como imaxe JPG. Perde o texto seleccionable: serve para dixitalizacións e fotos, non para texto."),
    accept: "application/pdf,.pdf",
    multiple: true,
    options: [
      {
        id: "level",
        type: "seg",
        label: tr("Calidade"),
        choices: () => [
          { value: "high", label: tr("Alta"), sub: "150 ppp" },
          { value: "mid", label: tr("Media"), sub: "110 ppp" },
          { value: "low", label: tr("Baixa"), sub: "80 ppp" },
        ],
      },
    ],
    defaults: { level: "mid" },
    action: (n) => (n === 1 ? tr("Comprimir 1 PDF") : tr("Comprimir {n} PDF", { n })),
    each: async (file, v, taken, ctx) => {
      const lv = ({ high: [150, 0.8], mid: [110, 0.7], low: [80, 0.6] } as Record<string, [number, number]>)[v.level] ?? [110, 0.7];
      const r = await (await import("./pdfimg")).compressPdf(file, lv[0], lv[1], ctx);
      if (r.blob.size >= file.size) return { name: uniqueName(file.name, taken), blob: file, note: tr("Xa estaba comprimido: queda igual.") };
      return { ...r, note: sizeNote(file, r.blob, false) };
    },
  },
];
