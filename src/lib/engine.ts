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

export type MediaOp = "frame" | "speed" | "mute" | "volume" | "scale" | "normalize" | "reverse" | "fadein" | "fadeout" | "echo" | "bass" | "treble" | "denoise";

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
/** Pistas elixidas dentro de cada tipo (0 = a primeira). Sen elección usanse as de por defecto. */
export interface TrackPick {
  video?: number | null;
  audio?: number | null;
}

/** Vista previa que o navegador entende. Para vídeos que non le devolve «chunks»: hai que pedilos por tramos. */
export async function makePreview(path: string, pick: TrackPick = {}): Promise<string> {
  return call("make_preview", { path, video: pick.video ?? null, audio: pick.audio ?? null });
}

/** Tramo de vídeo recodificado só para velo (ver `make_preview_chunk`). */
export async function makePreviewChunk(path: string, start: number, len: number, pick: TrackPick = {}): Promise<string> {
  return call("make_preview_chunk", { path, start, len, video: pick.video ?? null, audio: pick.audio ?? null });
}

/** Subtítulos de texto dun vídeo en WebVTT (índice entre os subtítulos). */
/** Garda uns bytes (por exemplo, unha imaxe con debuxo) nun ficheiro novo: primeiro un temporal e despois o nome final. */
export const saveBytes = (path: string, bytes: Uint8Array): Promise<string> => call("save_bytes", bytes, { headers: { "x-path": encodeURIComponent(path) } });
export const makeFramePreview = (path: string, time: number): Promise<string> => call("make_frame_preview", { path, time });
export async function makeSubtitles(path: string, index: number): Promise<string> {
  return call("make_subtitles", { path, index });
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

/** Troca os cartafoles polos ficheiros que levan dentro. */
export async function expandPaths(paths: string[]): Promise<string[]> {
  return call("expand_paths", { paths });
}

/** Abre ficheiros na aplicación completa (na xa aberta, ou nunha nova). */
export async function openInApp(paths: string[]): Promise<void> {
  return call("open_in_app", { paths });
}

export interface VideoCompressOpts {
  crf: number;
  targetBytes?: number;
  maxHeight?: number;
  audioKbps: number;
  videoKbps?: number;
  deinterlace?: 0 | 1 | 2;
  compat?: boolean;
  dar?: string;
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
    deinterlace: o.deinterlace ?? null,
    compat: o.compat ?? null,
    dar: o.dar ?? null,
  });
}

export async function estimateVideo(
  input: string,
  crf: number,
  maxHeight: number | undefined,
  audioKbps: number,
  extra: { deinterlace?: number; compat?: boolean; dar?: string } = {},
): Promise<number> {
  return call("estimate_video", { input, crf, maxHeight: maxHeight ?? null, audioKbps, deinterlace: extra.deinterlace ?? null, compat: extra.compat ?? null, dar: extra.dar || null });
}

// ------------------------------------------------------------ ferramentas PDF

type Pages = number[] | null;

export async function pdfWatermarkText(input: string, output: string, o: { text: string; size: number; opacity: number; angle: number; color: string; font?: string }, pages: Pages, overwrite = false): Promise<string> {
  return call("pdf_watermark_text", { input, output, overwrite, ...o, pages });
}

export async function pdfPageNumbers(input: string, output: string, o: { position: string; format: string; start: number; size: number; margin: number; color: string; font?: string }, pages: Pages, overwrite = false): Promise<string> {
  return call("pdf_page_numbers", { input, output, overwrite, ...o, pages });
}

export async function pdfAddText(input: string, output: string, o: { text: string; xPct: number; yPct: number; size: number; color: string; font?: string }, pages: Pages, overwrite = false): Promise<string> {
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

export const beginJob = (): Promise<void> => call("begin_job");
export const cancelJob = (): Promise<boolean> => call("cancel_job");
export const discardOutput = (path: string): Promise<boolean> => call("discard_output", { path });
export const enableProgress = (): Promise<void> => call("enable_progress");
export const uniquePath = (path: string): Promise<string> => call("unique_path", { path });

export interface ToolsStatus {
  ffmpeg: boolean;
  ffprobe: boolean;
  mkvmerge: boolean;
  /** Descargados pola propia app (poden quitarse desde aquí). */
  ffmpeg_own?: boolean;
  mkvmerge_own?: boolean;
}
export type ToolId = "ffmpeg" | "mkvmerge";
export const installTool = (tool: ToolId): Promise<ToolsStatus> => call("install_tool", { tool });
export const cancelToolInstall = (): Promise<void> => call("cancel_tool_install");
export const removeTool = (tool: ToolId): Promise<ToolsStatus> => call("remove_tool", { tool });
export const toolsStatus = (): Promise<ToolsStatus> => call("tools_status");

export interface MkvTrack {
  id: number;
  type: "video" | "audio" | "subtitles" | string;
  codec: string;
  properties: { language?: string; track_name?: string; default_track?: boolean; pixel_dimensions?: string; audio_channels?: number };
}
export const mkvTracks = async (path: string): Promise<MkvTrack[]> => ((await call<any>("mkv_tracks", { path })).tracks ?? []) as MkvTrack[];

export interface TrackEdit {
  id: number;
  name?: string;
  language?: string;
  default?: boolean;
}
export interface MuxInput {
  path: string;
  keep: number[];
  edits: TrackEdit[];
}
export interface TrackRef {
  input: number;
  id: number;
}
export const mkvMux = (inputs: MuxInput[], order: TrackRef[], output: string, overwrite = false): Promise<string> =>
  call("mkv_mux", { inputs, order, output, overwrite });

export type LinkId = "ffmpeg" | "ffmpeg-license" | "mkvtoolnix" | "mkvtoolnix-license" | "iso639-2" | "mkvmerge-doc";
export const openUrl = (url: string): Promise<void> => call("open_url", { url });
export const openLink = (id: LinkId): Promise<void> => call("open_link", { id });

export interface ExplorerStatus {
  available: boolean;
  installed: boolean;
  current: boolean;
}
export const explorerStatus = (): Promise<ExplorerStatus> => call("explorer_status");
export const explorerInstall = (labels: Record<string, string>): Promise<void> => call("explorer_install", { labels });
export const explorerUninstall = (): Promise<void> => call("explorer_uninstall");

export interface StartupJob {
  action: "convert" | "compress" | "resize" | "rotate";
  target?: string;
  paths: string[];
}
export const startupJob = (): Promise<StartupJob | null> => call("startup_job");
