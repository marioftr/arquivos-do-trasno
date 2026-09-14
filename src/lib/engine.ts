// Capa fina sobre os comandos de Tauri definidos en src-tauri/src/commands.rs.
// Mantense todo tipado para que o resto da UI non teña que saber nada de
// invoke() nin de nomes de comandos en cru.

import { invoke } from "@tauri-apps/api/core";

export type Category = "imaxe" | "audio" | "video" | "documento" | "modelo3d" | "descoñecido";

export interface ConvertResult {
  output_path: string;
  bytes_written: number;
  category: Category;
}

export interface ConvertOptions {
  overwrite?: boolean;
  quality?: number;
  resize?: [number, number];
  bitrateKbps?: number;
}

export async function detectCategory(path: string): Promise<Category> {
  return invoke("detect_category", { path });
}

export async function suggestedTargets(path: string): Promise<string[]> {
  return invoke("suggested_targets", { path });
}

export async function ffmpegAvailable(): Promise<boolean> {
  return invoke("ffmpeg_available");
}

export async function fileSize(path: string): Promise<number> {
  return invoke("file_size", { path });
}

export async function imageDimensions(path: string): Promise<[number, number]> {
  return invoke("image_dimensions", { path });
}

export async function convertFile(
  input: string,
  output: string,
  options: ConvertOptions = {},
): Promise<ConvertResult> {
  return invoke("convert_file", {
    input,
    output,
    overwrite: options.overwrite ?? false,
    quality: options.quality ?? null,
    resize: options.resize ?? null,
    bitrateKbps: options.bitrateKbps ?? null,
  });
}

export async function trimMedia(
  input: string,
  output: string,
  startSeconds: number,
  endSeconds: number | null,
  overwrite = false,
): Promise<string> {
  return invoke("trim_media", {
    input,
    output,
    overwrite,
    startSeconds,
    endSeconds,
  });
}

export async function concatMedia(inputs: string[], output: string, overwrite = false): Promise<string> {
  return invoke("concat_media", { inputs, output, overwrite });
}

export async function pdfMerge(inputs: string[], output: string, overwrite = false): Promise<string> {
  return invoke("pdf_merge", { inputs, output, overwrite });
}

export async function pdfExtractPages(
  input: string,
  pages: number[],
  output: string,
  overwrite = false,
): Promise<string> {
  return invoke("pdf_extract_pages", { input, pages, output, overwrite });
}

export async function pdfRotate(
  input: string,
  page: number | null,
  degrees: number,
  output: string,
  overwrite = false,
): Promise<string> {
  return invoke("pdf_rotate", { input, page, degrees, output, overwrite });
}

export async function pdfPageCount(input: string): Promise<number> {
  return invoke("pdf_page_count", { input });
}
