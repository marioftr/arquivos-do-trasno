// Espello, en TypeScript, do rexistro de formatos de trasno-engine
// (crates/trasno-engine/src/formats.rs). Mantense sinxelo e sincronizado a
// man por agora; se medra moito pasarase a xerar dende Rust.

import type { Category } from "./engine";

export const IMAGE_TARGETS = [
  "png", "jpg", "webp", "bmp", "gif", "ico", "tiff", "ppm", "pbm", "pgm", "tga", "qoi",
];

export const AUDIO_TARGETS = ["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "aiff"];

export const VIDEO_TARGETS = ["mp4", "mkv", "mov", "webm", "avi", "gif"];

/** Documentos de texto plano: pódense maquetar nun PDF. */
export const PLAIN_TEXT = ["txt", "md", "csv", "json", "xml", "log"];

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
      return [...VIDEO_TARGETS, ...AUDIO_TARGETS];
    case "documento":
      // Un PDF dá o seu texto (e as súas páxinas como imaxes, na pestana Imaxes); un .txt maquétase nun PDF sinxelo.
      if (sourceExt === "pdf") return ["txt"];
      return PLAIN_TEXT.includes(sourceExt ?? "") ? ["pdf"] : [];
    case "modelo3d":
      // O motor Rust lé OBJ/STL/PLY; FBX e glTF léense e escríbense a través
      // do visor (three.js). Ver lib/model3d.ts e Tools.svelte.
      return ["glb", "obj", "stl", "ply"].filter((t) => t !== sourceExt);
    default:
      return [];
  }
}

import { tr } from "./i18n";

/** Frases de éxito: ao azar, para que non soen sempre igual. */
export const TRASNO_LISTO = [
  "Feito! O trasno rematou o seu traballo sen atopar ningún problema.",
  "Listo! O trasno deixou todo en orde e sen un só tropezo.",
  "Xa está! O trasno cumpriu o encargo e ata lle sobrou tempo para un café.",
  "Traballo rematado! O trasno bótalle unha ollada e dáo por bo.",
  "Todo correcto! O trasno recolleu as ferramentas e volveu ao seu recuncho.",
];

/** Frases de fallo por tipo de traballo (tamén ao azar). */
export const TRASNO_FALLO: Record<string, string[]> = {
  convertendo: [
    "Vaites! O trasno tropezou durante a conversión.",
    "Ouh! Algo se torceu mentres o trasno convertía.",
    "Caramba! A conversión escapóuselle das mans ao trasno.",
  ],
  unindo: ["Vaites! O trasno tropezou ao unir os ficheiros.", "Ouh! Os ficheiros non se deixaron coser polo trasno."],
  editando: [
    "Vaites! O trasno tropezou durante a edición.",
    "Ouh! A ferramenta do trasno escorregou mentres editaba.",
    "Caramba! Algo saíu mal na edición.",
  ],
  comprimindo: ["Vaites! O trasno tropezou durante a compresión.", "Ouh! A compresión resistiuse ao trasno."],
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
  traballando: "O trasno arremanga e ponse a traballar…",
  convertendo: "O trasno está convertendo… xa case!",
  unindo: "O trasno está cosendo os ficheiros…",
  abrindo: "O trasno está abrindo o modelo…",
  comprimindo: "O trasno está comprimindo… paciencia!",
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
