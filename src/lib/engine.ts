// Capa fina sobre os comandos de Tauri definidos en src-tauri/src/commands.rs.
// Mantense todo tipado para que o resto da UI non teña que saber nada de
// invoke() nin de nomes de comandos en cru.

import { call } from "./errors";

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
  return call("detect_category", { path });
}

export async function suggestedTargets(path: string): Promise<string[]> {
  return call("suggested_targets", { path });
}

export async function ffmpegAvailable(): Promise<boolean> {
  return call("ffmpeg_available");
}

export async function fileSize(path: string): Promise<number> {
  return call("file_size", { path });
}

export async function imageDimensions(path: string): Promise<[number, number]> {
  return call("image_dimensions", { path });
}

export async function convertFile(
  input: string,
  output: string,
  options: ConvertOptions = {},
): Promise<ConvertResult> {
  return call("convert_file", {
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
  return call("trim_media", {
    input,
    output,
    overwrite,
    startSeconds,
    endSeconds,
  });
}

export async function concatMedia(inputs: string[], output: string, overwrite = false): Promise<string> {
  return call("concat_media", { inputs, output, overwrite });
}

export async function pdfMerge(inputs: string[], output: string, overwrite = false): Promise<string> {
  return call("pdf_merge", { inputs, output, overwrite });
}

export async function pdfExtractPages(
  input: string,
  pages: number[],
  output: string,
  overwrite = false,
): Promise<string> {
  return call("pdf_extract_pages", { input, pages, output, overwrite });
}

export async function pdfRotate(
  input: string,
  page: number | null,
  degrees: number,
  output: string,
  overwrite = false,
): Promise<string> {
  return call("pdf_rotate", { input, page, degrees, output, overwrite });
}

export async function pdfPageCount(input: string): Promise<number> {
  return call("pdf_page_count", { input });
}

export async function startupFiles(): Promise<string[]> {
  return call("startup_files");
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
  return call("edit_image", { input, output, overwrite, edits });
}

export async function imagesToPdf(inputs: string[], output: string, overwrite = false): Promise<string> {
  return call("images_to_pdf", { inputs, output, overwrite });
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
  return call("media_edit", { input, output, overwrite, op, value, hasVideo });
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
  return call("probe_media", { path });
}

export async function sha256(path: string): Promise<string> {
  return call("sha256", { path });
}

export async function revealInFolder(path: string): Promise<void> {
  return call("reveal_in_folder", { path });
}

/** Copia lixeira que o visor sabe amosar (PNG, MP4 pequeno ou MP3). */
export async function makePreview(path: string): Promise<string> {
  return call("make_preview", { path });
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
  return call("compress_image", {
    input,
    output,
    overwrite,
    quality: opts.quality ?? null,
    targetBytes: opts.targetBytes ?? null,
    maxSide: opts.maxSide ?? null,
  });
}

export async function estimateImage(input: string, quality: number, maxSide?: number): Promise<number> {
  return call("estimate_image", { input, quality, maxSide: maxSide ?? null });
}

export interface VideoCompressOpts {
  crf: number;
  targetBytes?: number;
  maxHeight?: number;
  audioKbps: number;
  videoKbps?: number;
}

export async function compressVideo(input: string, output: string, o: VideoCompressOpts, overwrite = false): Promise<string> {
  return call("compress_video", {
    input,
    output,
    overwrite,
    crf: o.crf,
    targetBytes: o.targetBytes ?? null,
    maxHeight: o.maxHeight ?? null,
    audioKbps: o.audioKbps,
    videoKbps: o.videoKbps ?? null,
  });
}

export async function estimateVideo(input: string, crf: number, maxHeight: number | undefined, audioKbps: number): Promise<number> {
  return call("estimate_video", { input, crf, maxHeight: maxHeight ?? null, audioKbps });
}

// ------------------------------------------------------------ ferramentas PDF

type Pages = number[] | null;

export async function pdfWatermarkText(input: string, output: string, o: { text: string; size: number; opacity: number; angle: number; color: string }, pages: Pages, overwrite = false): Promise<string> {
  return call("pdf_watermark_text", { input, output, overwrite, ...o, pages });
}

export async function pdfPageNumbers(input: string, output: string, o: { position: string; format: string; start: number; size: number; margin: number; color: string }, pages: Pages, overwrite = false): Promise<string> {
  return call("pdf_page_numbers", { input, output, overwrite, ...o, pages });
}

export async function pdfAddText(input: string, output: string, o: { text: string; xPct: number; yPct: number; size: number; color: string }, pages: Pages, overwrite = false): Promise<string> {
  return call("pdf_add_text", { input, output, overwrite, ...o, pages });
}

export async function pdfStampImage(input: string, output: string, o: { image: string; xPct: number; yPct: number; widthPct: number; opacity: number }, pages: Pages, overwrite = false): Promise<string> {
  return call("pdf_stamp_image", { input, output, overwrite, ...o, pages });
}

export async function pdfCrop(input: string, output: string, m: { left: number; bottom: number; right: number; top: number }, pages: Pages, overwrite = false): Promise<string> {
  return call("pdf_crop", { input, output, overwrite, ...m, pages });
}

export async function pdfCompress(input: string, output: string, quality: number, scalePercent: number, overwrite = false): Promise<string> {
  return call("pdf_compress", { input, output, overwrite, quality, scalePercent });
}

export async function pdfProtect(input: string, output: string, o: { userPassword: string; ownerPassword: string; allowPrint: boolean; allowCopy: boolean }, overwrite = false): Promise<string> {
  return call("pdf_protect", { input, output, overwrite, ...o });
}

export async function pdfUnprotect(input: string, output: string, password: string, overwrite = false): Promise<string> {
  return call("pdf_unprotect", { input, output, overwrite, password });
}

export async function pdfIsEncrypted(input: string): Promise<boolean> {
  return call("pdf_is_encrypted", { input });
}

export async function pdfArrange(input: string, output: string, order: [number, number][], overwrite = false): Promise<string> {
  return call("pdf_arrange", { input, output, overwrite, order });
}

export async function pdfRepair(input: string, output: string, overwrite = false): Promise<string> {
  return call("pdf_repair", { input, output, overwrite });
}
