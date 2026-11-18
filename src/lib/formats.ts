// Espello, en TypeScript, do rexistro de formatos de trasno-engine
// (crates/trasno-engine/src/formats.rs). Mantense sinxelo e sincronizado a
// man por agora; se medra moito pasarase a xerar dende Rust.

import type { Category } from "./engine";
import { tr } from "./i18n";

export const IMAGE_TARGETS = [
  "png", "jpg", "webp", "bmp", "gif", "ico", "tiff", "ppm", "pbm", "pgm", "tga", "qoi", "avif", "jp2", "jxl",
];

const IMAGE_ALIAS: Record<string, string> = { jpeg: "jpg", jpe: "jpg", jfif: "jpg", tif: "tiff", pnm: "ppm" };

/** A extensión na que se gardan as edicións dunha imaxe: a mesma que a do orixinal (o trasno só cambia de formato en Converter).
 *  Devolve null se o formato só se sabe ler (HEIC, DDS, EXR…): entón a edición queda bloqueada con aviso. */
export function sameFormat(ext: string): string | null {
  const e = ext.toLowerCase();
  return IMAGE_TARGETS.includes(IMAGE_ALIAS[e] ?? e) ? e : null;
}

/** Ferramentas que gardan no formato do orixinal e poden quedar bloqueadas para un ficheiro. */
export type LockTool = "editar" | "anotar" | "comprimir" | "redimensionar" | "cortar";
const VERB: Record<LockTool, string> = { editar: "editar", anotar: "anotar", comprimir: "comprimir", redimensionar: "redimensionar", cortar: "cortar" };
const VERB_LO: Record<LockTool, string> = { editar: "Editalo", anotar: "Anotalo", comprimir: "Comprimilo", redimensionar: "Redimensionalo", cortar: "Cortalo" };

/** Texto curto que se ve cando unha ferramenta está bloqueada; a explicación vai no «i». */
export const readOnlyLabel = () => tr("Só modo lectura");

/** Explicación de que un formato só se sabe ler (HEIC, DDS, EXR…) e non gardar: a ferramenta queda bloqueada. */
export function cannotSaveNotice(ext: string, tool: LockTool): string {
  const fmt = ext.toUpperCase();
  const verb = tr(VERB[tool]);
  return tr("Só se pode ler {fmt}, non gardalo: para {verb} hai que convertelo antes a PNG ou JPG, en Converter.", { fmt, verb });
}

/** Formatos que se poden gardar pero non teñen compresión propia (BMP, TIFF…): comprimir mantendo o formato non ten sentido. */
export function noCompressionNotice(ext: string): string {
  return tr("O {fmt} non ten compresión propia: convérteo antes a PNG ou JPG, en Converter.", { fmt: ext.toUpperCase() });
}

/** Imaxe animada que a ferramenta non sabe tratar sen perder os fotogramas (só se garda un). */
export function animatedNotice(ext: string, tool: LockTool): string {
  const fmt = ext.toUpperCase();
  const verb = tr(VERB[tool]);
  if (tool === "redimensionar") return tr("Este {fmt} é animado: só se poden redimensionar GIF e WebP animados sen perder a animación.", { fmt });
  const gifOnly = tool === "editar" || tool === "cortar" ? "" : " " + tr("Só se pode {verb} GIF sen perder a animación.", { verb });
  return tr("Este {fmt} é animado: {verbLo} só gardaría un fotograma.", { fmt, verb, verbLo: tr(VERB_LO[tool]).toLowerCase() }) + gifOnly;
}

/** Aviso de que unha imaxe animada pasa a un formato que só garda un fotograma. */
export function firstFrameNotice(from: string, to: string): string {
  return tr("Esta imaxe {from} é animada, pero {to} só garda unha imaxe fixa: gardarase só o primeiro fotograma. Para conservar a animación, escolle GIF ou WebP.", { from: from.toUpperCase(), to: to.toUpperCase() });
}

/** Documentos de Office que o trasno convirte con Microsoft Office ou LibreOffice (ver `office.rs` no motor). */
export const OFFICE_EXTS = ["docx", "doc", "docm", "odt", "rtf", "xlsx", "xls", "xlsm", "ods", "pptx", "ppt", "pptm", "ppsx", "pps", "odp"];
/** Saídas de documento que un PDF só dá con Word. */
export const PDF_TO_DOC = ["docx", "doc", "odt", "rtf"];

/** Formatos en que se gardan as imaxes que se compoñen a partir doutras (xuntar capturas, cortar en partes). */
export const COMPOSE_FORMATS = ["png", "jpg", "webp", "tiff", "bmp"];
/** O formato que sae: o elixido ou, con «same», o dos orixinais (se todos teñen o mesmo e se pode gardar; se non, PNG). */
export function composeFormat(choice: string, exts: string[]): string {
  if (choice !== "same") return choice;
  const norm = (e: string) => {
    const x = e.toLowerCase();
    return IMAGE_ALIAS[x] ?? x;
  };
  const kinds = new Set(exts.map(norm));
  const only = kinds.size === 1 ? [...kinds][0] : "";
  return COMPOSE_FORMATS.includes(only) ? only : "png";
}
/** Opcións do selector de formato de saída: «Igual que o orixinal» e os formatos nos que se pode gardar. */
export const composeOptions = (tt: (k: string) => string) => [
  { value: "same", label: tt("Igual que o orixinal") },
  { value: "png", label: "PNG" },
  { value: "jpg", label: "JPG" },
  { value: "webp", label: "WebP" },
  { value: "tiff", label: "TIFF" },
  { value: "bmp", label: "BMP" },
];

export const AUDIO_TARGETS = ["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "aiff"];

export const VIDEO_TARGETS = ["mp4", "mkv", "mov", "webm", "avi", "gif"];
/** Imaxes animadas que se poden facer desde un vídeo. */
export const CLIP_TARGETS = ["gif", "webp"];

/** Documentos de texto plano: pódense maquetar nun PDF. */
export const PLAIN_TEXT = ["txt", "md", "csv", "json", "xml", "log"];

/** Subtítulos: vense como texto, convértense entre eles (e a UTF-8) e resincronízanse. */
export const SUBTITLE_EXTS = ["srt", "vtt", "ass", "ssa"];
export const SUBTITLE_TARGETS = ["srt", "vtt", "ass"];

export const CATEGORY_LABELS: Record<Category, string> = {
  imaxe: "Imaxe",
  audio: "Audio",
  video: "Vídeo",
  documento: "Documento",
  modelo3d: "Modelo 3D",
  descoñecido: "Descoñecido",
};

export function targetsForCategory(category: Category, sourceExt?: string): string[] {
  switch (category) {
    case "imaxe":
      // Unha imaxe tamén pode ser un PDF dunha páxina.
      return [...IMAGE_TARGETS, "pdf"];
    case "audio":
      return AUDIO_TARGETS;
    case "video":
      // Un vídeo tamén se pode converter só ao seu audio.
      return [...VIDEO_TARGETS, "webp", ...AUDIO_TARGETS];
    case "documento":
      // Un PDF dá o seu texto (e as súas páxinas como imaxes, na pestana Imaxes); un .txt maquétase nun PDF sinxelo.
      if (sourceExt === "pdf") return ["txt"];
      if (SUBTITLE_EXTS.includes(sourceExt ?? "")) return SUBTITLE_TARGETS;
      return PLAIN_TEXT.includes(sourceExt ?? "") ? ["pdf"] : [];
    case "modelo3d":
      // O motor Rust lé OBJ/STL/PLY; FBX e glTF léense e escríbense a través
      // do visor (three.js). Ver lib/model3d.ts e Tools.svelte.
      return ["glb", "obj", "stl", "ply"].filter((t) => t !== sourceExt);
    default:
      return [];
  }
}

/** Frases de éxito: ao azar, para que non soen sempre igual. */
export const TRASNO_LISTO = [
  "Feito! Rematei sen romper nin un prato.",
  "Listo! Cousa de fogar: deixei todo no seu sitio (esta vez de verdade).",
  "Xa está! Aínda non cantou o galo e eu xa tiña isto feito.",
  "Rematado! Toquei o gorro vermello e dei o traballo por bo.",
  "Todo correcto! Esta vez non tiven que facer ningunha trasnada.",
];

/** Frases de fallo por tipo de traballo (tamén ao azar). */
export const TRASNO_FALLO: Record<string, string[]> = {
  convertendo: [
    "Vaites! Enredeime na conversión.",
    "Ouh! Trasnada de última hora: a conversión non saíu.",
    "Caramba! Ata un trasno perde o fío ás veces: a conversión fallou.",
  ],
  unindo: [
    "Vaites! Os ficheiros fixéronme unha trasnada e non se deixaron unir.",
    "Ouh! Perdín a agulla mentres cosía os ficheiros.",
  ],
  editando: [
    "Vaites! Escorregei co gorro posto durante a edición.",
    "Ouh! A ferramenta escapóuseme mentres editaba.",
    "Caramba! Algo se enredou na edición.",
  ],
  comprimindo: [
    "Vaites! A compresión foi máis teimuda ca min.",
    "Ouh! Apertei, apertei… e non houbo maneira.",
  ],
};

export function fallo(kind: keyof typeof TRASNO_FALLO | string): string {
  const list = TRASNO_FALLO[kind] ?? TRASNO_FALLO.editando;
  return tr(list[Math.floor(Math.random() * list.length)]);
}

export function listo(): string {
  return tr(TRASNO_LISTO[Math.floor(Math.random() * TRASNO_LISTO.length)]);
}

/** Frases do trasno, o duende que fai o traballo de fondo. O texto en galego
 * é a chave de tradución: pásase por `tr()` / `$t()` ao amosalo. */
export const TRASNO = {
  traballando: "Arremango e póñome a traballar…",
  convertendo: "Estou convertendo… xa case!",
  unindo: "Estou cosendo os ficheiros…",
  abrindo: "Estou abrindo o modelo…",
  comprimindo: "Estou apertando os ficheiros… paciencia!",
};

export function extensionOf(path: string): string {
  const dot = path.lastIndexOf(".");
  if (dot === -1 || dot === path.length - 1) return "";
  return path.slice(dot + 1).toLowerCase();
}

export function withExtension(path: string, newExt: string): string {
  const dot = path.lastIndexOf(".");
  const base = dot === -1 ? path : path.slice(0, dot);
  return `${base}.${newExt}`;
}

export function fileNameOf(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[i]}`;
}

/** Tempo como reloxo: «5:09» ou, con máis dunha hora, «1:05:09». `decimals` engade décimas («1:05:09,5»). */
export function formatClock(seconds: number, decimals = 0): string {
  const total = Math.max(0, Number.isFinite(seconds) ? seconds : 0);
  const k = 10 ** decimals;
  const whole = Math.floor(total);
  const frac = decimals ? Math.min(k - 1, Math.floor((total - whole) * k)) : 0;
  const h = Math.floor(whole / 3600);
  const m = Math.floor((whole % 3600) / 60);
  const sec = String(whole % 60).padStart(2, "0");
  const tail = decimals ? "," + String(frac).padStart(decimals, "0") : "";
  return `${h ? `${h}:${String(m).padStart(2, "0")}` : m}:${sec}${tail}`;
}
