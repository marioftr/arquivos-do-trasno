// Espello, en TypeScript, do rexistro de formatos de trasno-engine
// (crates/trasno-engine/src/formats.rs). Mantense sinxelo e sincronizado a
// man por agora; se medra moito pasaráse a xerar dende Rust.

import type { Category } from "./engine";

export const IMAGE_TARGETS = [
  "png", "jpg", "webp", "bmp", "gif", "ico", "tiff", "ppm", "pbm", "pgm", "tga", "qoi",
];

export const AUDIO_TARGETS = ["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "aiff"];

export const VIDEO_TARGETS = ["mp4", "mkv", "mov", "webm", "avi", "gif"];

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
      return IMAGE_TARGETS;
    case "audio":
      return AUDIO_TARGETS;
    case "video":
      return VIDEO_TARGETS;
    case "documento":
      // Un PDF extrae o seu texto; un .txt maquétase nun PDF sinxelo.
      return sourceExt === "pdf" ? ["txt"] : ["pdf"];
    case "modelo3d":
      // De momento só mallas simples; gltf/glb/fbx recoñécense pero aínda
      // non se poden converter (ver crates/trasno-engine/src/model3d.rs).
      return ["obj", "stl", "ply"];
    default:
      return [];
  }
}

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
