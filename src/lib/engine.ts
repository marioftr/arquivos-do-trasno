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

export async function startupFiles(): Promise<string[]> {
  return invoke("startup_files");
}

export interface ImageEdits {
  rotate?: number;
  flipH?: boolean;
  flipV?: boolean;
  grayscale?: boolean;
  invert?: boolean;
  brightness?: number;
  contrast?: number;
  blur?: number;
  crop?: [number, number, number, number];
  quality?: number;
  resize?: [number, number];
}

export async function editImage(input: string, output: string, edits: ImageEdits, overwrite = false): Promise<string> {
  return invoke("edit_image", { input, output, overwrite, edits });
}

export async function imagesToPdf(inputs: string[], output: string, overwrite = false): Promise<string> {
  return invoke("images_to_pdf", { inputs, output, overwrite });
}

export type MediaOp = "frame" | "speed" | "mute" | "volume" | "scale";

export async function mediaEdit(
  input: string,
  output: string,
  op: MediaOp,
  value: number,
  hasVideo: boolean,
  overwrite = false,
): Promise<string> {
  return invoke("media_edit", { input, output, overwrite, op, value, hasVideo });
}

export interface MediaInfo {
  format?: { duration?: string; bit_rate?: string; format_long_name?: string };
  streams?: {
    codec_type: string;
    codec_name?: string;
    width?: number;
    height?: number;
    r_frame_rate?: string;
    sample_rate?: string;
    channels?: number;
  }[];
}

export async function probeMedia(path: string): Promise<MediaInfo> {
  return invoke("probe_media", { path });
}

export async function sha256(path: string): Promise<string> {
  return invoke("sha256", { path });
}

export async function revealInFolder(path: string): Promise<void> {
  return invoke("reveal_in_folder", { path });
}

/** Copia lixeira que o visor sabe amosar (PNG, MP4 pequeno ou MP3). */
export async function makePreview(path: string): Promise<string> {
  return invoke("make_preview", { path });
}

export interface ImageCompressReport {
  output_path: string;
  bytes_written: number;
  quality: number;
  scale_percent: number;
}

export async function compressImage(
  input: string,
  output: string,
  opts: { quality?: number; targetBytes?: number; maxSide?: number },
  overwrite = false,
): Promise<ImageCompressReport> {
  return invoke("compress_image", {
    input,
    output,
    overwrite,
    quality: opts.quality ?? null,
    targetBytes: opts.targetBytes ?? null,
    maxSide: opts.maxSide ?? null,
  });
}

export async function estimateImage(input: string, quality: number, maxSide?: number): Promise<number> {
  return invoke("estimate_image", { input, quality, maxSide: maxSide ?? null });
}

export interface VideoCompressOpts {
  crf: number;
  targetBytes?: number;
  maxHeight?: number;
  audioKbps: number;
}

export async function compressVideo(input: string, output: string, o: VideoCompressOpts, overwrite = false): Promise<string> {
  return invoke("compress_video", {
    input,
    output,
    overwrite,
    crf: o.crf,
    targetBytes: o.targetBytes ?? null,
    maxHeight: o.maxHeight ?? null,
    audioKbps: o.audioKbps,
  });
}

export async function estimateVideo(input: string, crf: number, maxHeight: number | undefined, audioKbps: number): Promise<number> {
  return invoke("estimate_video", { input, crf, maxHeight: maxHeight ?? null, audioKbps });
}
