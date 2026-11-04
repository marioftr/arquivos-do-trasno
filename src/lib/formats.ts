// Espello, en TypeScript, do rexistro de formatos de trasno-engine
// (crates/trasno-engine/src/formats.rs). Mantense sinxelo e sincronizado a
// man por agora; se medra moito pasarase a xerar dende Rust.

import type { Category } from "./engine";

export const IMAGE_TARGETS = [
  "png", "jpg", "webp", "bmp", "gif", "ico", "tiff", "ppm", "pbm", "pgm", "tga", "qoi", "avif", "jp2", "jxl",
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
  "Feito! O trasno rematou sen romper nin un prato.",
  "Listo! Cousa de fogar: o trasno deixou todo no seu sitio (esta vez de verdade).",
  "Xa está! Aínda non cantou o galo e o trasno xa tiña isto feito.",
  "Rematado! O trasno tocou o gorro vermello e deu o traballo por bo.",
  "Todo correcto! Esta vez o trasno non tivo que facer ningunha trasnada.",
];

/** Frases de fallo por tipo de traballo (tamén ao azar). */
export const TRASNO_FALLO: Record<string, string[]> = {
  convertendo: [
    "Vaites! O trasno enredouse na conversión.",
    "Ouh! Trasnada de última hora: a conversión non saíu.",
    "Caramba! Ata o trasno perde o fío ás veces: a conversión fallou.",
  ],
  unindo: [
    "Vaites! Os ficheiros fixéronlle unha trasnada ao trasno e non se deixaron unir.",
    "Ouh! O trasno perdeu a agulla mentres cosía os ficheiros.",
  ],
  editando: [
    "Vaites! O trasno escorregou co gorro posto durante a edición.",
    "Ouh! A ferramenta escapóuselle ao trasno mentres editaba.",
    "Caramba! Algo se enredou na edición.",
  ],
  comprimindo: [
    "Vaites! A compresión foi máis teimuda ca o trasno.",
    "Ouh! O trasno apertou, apertou… e non houbo maneira.",
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
  traballando: "O trasno arremanga e ponse a traballar…",
  convertendo: "O trasno está convertendo… xa case!",
  unindo: "O trasno está cosendo os ficheiros…",
  abrindo: "O trasno está abrindo o modelo…",
  comprimindo: "O trasno está apertando os ficheiros… paciencia!",
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
