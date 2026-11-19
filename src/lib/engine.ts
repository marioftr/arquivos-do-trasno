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
  lossless?: boolean;
  /** Fondo [r, g, b] para a transparencia ao pasar a un formato que non a admite. */
  background?: [number, number, number];
  /** Vídeo → GIF ou WebP animado: tope de fotogramas por segundo (por defecto 15). */
  fps?: number;
  /** Vídeo → GIF ou WebP animado: ancho máximo en píxeles (por defecto 640). */
  maxWidth?: number;
  /** Imaxe → imaxe: metadatos do orixinal que se levan (por defecto ningún). Só JPG, PNG e WebP sen perdas. */
  metadata?: PhotoMetadata;
}

/** Que facer cos metadatos dunha foto: `strip` ningún, `no_location` cámara e data sen ubicación, `all` todo. */
export type PhotoMetadata = "strip" | "no_location" | "all";

/** Formatos de imaxe que poden levar metadatos e perfil de cor. */
export const META_EXTS = ["jpg", "jpeg", "png", "webp"];

/** Dos dous interruptores da interface ao valor que entende o motor. */
export const photoMetadata = (keep: boolean, gps: boolean): PhotoMetadata => (!keep ? "strip" : gps ? "all" : "no_location");

export async function detectCategory(path: string): Promise<Category> {
  return call("detect_category", { path });
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
    lossless: options.lossless ?? null,
    background: options.background ?? null,
    fps: options.fps ?? null,
    maxWidth: options.maxWidth ?? null,
    metadata: options.metadata ?? null,
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
  background?: [number, number, number];
}

export async function editImage(input: string, output: string, edits: ImageEdits, overwrite = false): Promise<string> {
  return call("edit_image", { input, output, overwrite, edits });
}

export async function imagesToPdf(inputs: string[], output: string, overwrite = false): Promise<string> {
  return call("images_to_pdf", { inputs, output, overwrite });
}

export type MediaOp = "frame" | "speed" | "mute" | "volume" | "scale" | "normalize" | "reverse" | "fadein" | "fadeout" | "echo" | "bass" | "treble" | "denoise";

export interface NormalizeResult {
  output_path: string;
  before_lufs: number;
  before_peak: number;
  before_range: number;
  /** `false` se houbo que comprimir a dinámica para chegar ao obxectivo. */
  linear: boolean;
}

/** Iguala o volume percibido en dúas pasadas (EBU R128). `lufs`: obxectivo; `peak`: pico máximo (dBTP); `range`: rango dinámico máximo (LU). */
export interface SubtitleInfo {
  format: "srt" | "vtt" | "ass";
  cues: number;
  /** Inicio da primeira fala, inicio da última e fin da última (ms). */
  first_start: number;
  last_start: number;
  last_end: number;
  encoding: string;
}
export interface Retime {
  shift_ms: number;
  factor: number;
}
export interface RetimeReport {
  output_path: string;
  cues: number;
  dropped: number;
}
export interface SubtitleTrack {
  position: number;
  codec: string;
  language: string | null;
  title: string | null;
  text_based: boolean;
}
export interface SubtitleVideoReport {
  output: string;
  cues: number;
}
export const subtitleTracks = (path: string): Promise<SubtitleTrack[]> => call("subtitle_tracks", { path });
export const subtitleEmbed = (video: string, subs: string, output: string, language: string, overwrite = false): Promise<SubtitleVideoReport> =>
  call("subtitle_embed", { video, subs, output, language, overwrite });
export const subtitleBurn = (video: string, subs: string, output: string, overwrite = false): Promise<SubtitleVideoReport> => call("subtitle_burn", { video, subs, output, overwrite });
export const subtitleExtract = (video: string, position: number, output: string, overwrite = false): Promise<SubtitleVideoReport> =>
  call("subtitle_extract", { video, position, output, overwrite });
export interface RestorePlan {
  stabilize: number;
  denoise: number;
  sharpen: number;
  deinterlace: boolean;
  deflicker: boolean;
  brightness: number;
  contrast: number;
  saturation: number;
}
export const restoreApply = (input: string, output: string, plan: RestorePlan, overwrite = false): Promise<{ output: string; stabilizer: string | null }> =>
  call("restore_apply", { input, output, overwrite, plan });
export const restoreFrame = (input: string, time: number, plan: RestorePlan): Promise<string> => call("restore_frame", { input, time, plan });
export interface MusicTags {
  title: string;
  artist: string;
  album: string;
  album_artist: string;
  composer: string;
  genre: string;
  comment: string;
  lyrics: string;
  year: number | null;
  track: number | null;
  track_total: number | null;
  disc: number | null;
  disc_total: number | null;
}
export interface TagInfo {
  tags: MusicTags;
  cover: { mime: string; bytes: number } | null;
  format: string;
  duration: number;
}
export type CoverAction = { kind: "keep" } | { kind: "remove" } | { kind: "replace"; path: string };
export const tagsRead = (path: string): Promise<TagInfo> => call("tags_read", { path });
export const tagsCover = (path: string): Promise<string> => call("tags_cover", { path });
export const tagsWrite = (input: string, output: string, tags: MusicTags, cover: CoverAction, overwrite = false): Promise<{ output: string; not_saved: string[] }> =>
  call("tags_write", { input, output, overwrite, tags, cover });
export const subtitleInfo = (path: string): Promise<SubtitleInfo> => call("subtitle_info", { path });
export const retimeSubtitle = (input: string, output: string, r: Retime, overwrite = false): Promise<RetimeReport> =>
  call("retime_subtitle", { input, output, overwrite, shiftMs: r.shift_ms, factor: r.factor });
export const subtitleTwoPoints = (first: number, last: number, wantFirst: number, wantLast: number): Promise<Retime> =>
  call("subtitle_two_points", { first, last, wantFirst, wantLast });
export const subtitleFpsChange = (fromFps: number, toFps: number): Promise<Retime> => call("subtitle_fps_change", { fromFps, toFps });

export interface SilenceValues {
  /** Volume por baixo do cal hai silencio (dB). */
  thresholdDb: number;
  /** Silencio mínimo para cortalo (s). */
  minSilence: number;
  /** Silencio que se deixa a cada lado de cada corte (s). */
  padding: number;
}
export interface SilenceAnalysis {
  duration: number;
  /** Tramos que se cortarían: [inicio, fin] en segundos. */
  cuts: [number, number][];
  result_duration: number;
}
export interface SilenceReport {
  output_path: string;
  before: number;
  after: number;
  cuts: number;
}
/** Busca os silencios (non escribe nada): cantos hai e canto duraría o resultado. */
export const analyzeSilence = (input: string, v: SilenceValues): Promise<SilenceAnalysis> => call("analyze_silence", { input, thresholdDb: v.thresholdDb, minSilence: v.minSilence, padding: v.padding });
/** Corta os silencios nun ficheiro novo (audio ou vídeo). */
export const cutSilence = (input: string, output: string, v: SilenceValues, overwrite = false): Promise<SilenceReport> =>
  call("cut_silence", { input, output, overwrite, thresholdDb: v.thresholdDb, minSilence: v.minSilence, padding: v.padding });

export const normalizeAudio = (input: string, output: string, target: { lufs: number; peak: number; range: number }, hasVideo: boolean, overwrite = false): Promise<NormalizeResult> =>
  call("normalize_audio", { input, output, overwrite, lufs: target.lufs, peak: target.peak, range: target.range, hasVideo });

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

/** Todo o que se lle pode facer a un audio ou vídeo nunha soa exportación (os valores por defecto non cambian nada). */
export interface EditPlan {
  start: number;
  end: number | null;
  speed: number;
  volume: number;
  mute: boolean;
  denoise: boolean;
  reverse: boolean;
  fade_in: number;
  fade_out: number;
  bass: number;
  treble: number;
  echo_ms: number;
  normalize: { lufs: number; peak: number; range: number } | null;
}
export interface EditReport {
  output: string;
  duration: number | null;
  video_copied: boolean;
}
/** Aplica o plan enteiro nun ficheiro novo. */
export const editApply = (input: string, output: string, plan: EditPlan, overwrite = false): Promise<EditReport> => call("edit_apply", { input, output, overwrite, plan });
/** O son do tramo con ruído quitado e/ou ao revés, no seu sitio da liña de tempo (para a vista previa en directo). Devolve a ruta temporal. */
export const editAudioPreview = (input: string, start: number, end: number | null, denoise: boolean, reverse: boolean): Promise<string> => call("edit_audio_preview", { input, start, end, denoise, reverse });

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
export interface OfficeStatus { word: boolean; excel: boolean; powerpoint: boolean; libreoffice: boolean }
/** Que hai instalado para converter documentos de Office. */
export const officeStatus = (): Promise<OfficeStatus> => call("office_status");
/** Formatos de saída para un documento de Office (ou PDF), segundo o que haxa instalado. */
export const officeTargets = (ext: string): Promise<string[]> => call("office_targets", { ext });
export const convertOffice = (input: string, output: string, overwrite: boolean): Promise<string> => call("convert_office", { input, output, overwrite });

/** Garda unha imaxe (en PNG) no formato da extensión de `path`, sexa cal sexa. */
export const saveImageBytes = (path: string, png: Uint8Array): Promise<string> => call("save_image_bytes", png, { headers: { "x-path": encodeURIComponent(path) } });
/** Ten a imaxe máis dun fotograma (GIF, WebP ou PNG animados)? */
export const imageIsAnimated = (path: string): Promise<boolean> => call("image_is_animated", { path });
/** Anota un GIF animado: `overlay` é unha capa PNG transparente co debuxado que se superpón en todos os fotogramas. */
export const annotateAnimated = (input: string, output: string, overlay: Uint8Array): Promise<string> =>
  call("annotate_animated", overlay, { headers: { "x-input": encodeURIComponent(input), "x-path": encodeURIComponent(output) } });
export interface InfoGroup {
  title: string;
  rows: [string, string][];
}
export const fileInfo = (path: string): Promise<{ summary: InfoGroup[]; advanced: InfoGroup[] }> => call("file_info", { path });
export interface PdfTextItem {
  page: number;
  text: string;
  xPct: number;
  yPct: number;
  size: number;
  color: string;
  font?: string;
  angle?: number;
}
export const pdfAddTexts = (input: string, output: string, items: PdfTextItem[], overwrite = false): Promise<string> => call("pdf_add_texts", { input, output, overwrite, items });
export interface SysFont {
  name: string;
  family: string;
  weight: number;
  italic: boolean;
  path: string;
}
export const listFonts = (): Promise<SysFont[]> => call("list_fonts");
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
  opts: { quality?: number; targetBytes?: number; maxSide?: number; background?: [number, number, number]; metadata?: PhotoMetadata },
  overwrite = false,
): Promise<ImageCompressReport> {
  return call("compress_image", {
    input,
    output,
    overwrite,
    quality: opts.quality ?? null,
    targetBytes: opts.targetBytes ?? null,
    maxSide: opts.maxSide ?? null,
    background: opts.background ?? null,
    metadata: opts.metadata ?? null,
  });
}

/** Extensión en que se comprime unha imaxe: a mesma se o motor sabe (JPG, PNG, WebP, AVIF, JPEG XL, JPEG 2000), se non JPG. */
export const COMPRESS_KEEP = ["jpg", "jpeg", "png", "webp", "avif", "jxl", "jp2", "gif"];
export const compressExt = (ext: string) => (COMPRESS_KEEP.includes(ext.toLowerCase()) ? ext.toLowerCase() : "jpg");

export type ResizeFit =
  | { mode: "within"; width: number | null; height: number | null; no_upscale: boolean }
  | { mode: "percent"; percent: number }
  | { mode: "exact"; width: number; height: number; crop: boolean; no_upscale: boolean };

export interface ResizeReport {
  output_path: string;
  from: [number, number];
  to: [number, number];
  bytes_written: number;
}

export async function resizeImage(
  input: string,
  output: string,
  fit: ResizeFit,
  opts: { quality?: number; background?: [number, number, number]; metadata?: PhotoMetadata } = {},
  overwrite = false,
): Promise<ResizeReport> {
  return call("resize_image", { input, output, overwrite, fit, quality: opts.quality ?? null, background: opts.background ?? null, metadata: opts.metadata ?? null });
}

/** Tamaño actual e tamaño que sairía con ese axuste (sen tocar nada). */
export async function resizePreview(input: string, fit: ResizeFit): Promise<[[number, number], [number, number]]> {
  return call("resize_preview", { input, fit });
}

export interface StitchOptions {
  direction: "vertical" | "horizontal";
  align: "narrowest" | "widest";
  remove_overlap: boolean;
  gap: number;
  background: [number, number, number];
  quality: number | null;
}

export interface StitchReport {
  output_path: string;
  size: [number, number];
  /** Por cada unión: píxeles repetidos que se quitaron, ou null se non se atopou onde se solapan. */
  joins: (number | null)[];
  bytes_written: number;
}

export async function stitchImages(inputs: string[], output: string, options: StitchOptions, overwrite = false): Promise<StitchReport> {
  return call("stitch_images", { inputs, output, overwrite, options });
}

export interface SliceOptions {
  /** Zona que se reparte (x, y, ancho, alto), en píxeles da imaxe tal como se ve. */
  rect: [number, number, number, number];
  cols: number;
  rows: number;
  by_columns: boolean;
  quality: number | null;
  background: [number, number, number];
}
export interface SliceReport {
  outputs: string[];
  piece: [number, number];
  bytes_written: number;
}

/** Corta unha imaxe en partes iguais e garda cada unha como `stem-N.ext` en `dir` (nunca pisa un ficheiro). */
export async function sliceImage(input: string, dir: string, stem: string, ext: string, options: SliceOptions): Promise<SliceReport> {
  return call("slice_image", { input, dir, stem, ext, options });
}

export async function estimateImage(input: string, ext: string, quality: number, maxSide?: number, background?: [number, number, number]): Promise<number> {
  return call("estimate_image", { input, ext, quality, maxSide: maxSide ?? null, background: background ?? null });
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
export const pauseJob = (paused: boolean): Promise<boolean> => call("pause_job", { paused });
export const jobCancelled = (): Promise<boolean> => call("job_cancelled");
export interface OcrTextWord {
  text: string;
  /** Esquina de abaixo á esquerda da palabra tal como se ve, en puntos do PDF (orixe abaixo á esquerda). */
  x: number;
  y: number;
  /** Longo sobre a liña e alto, en puntos. */
  w: number;
  h: number;
  /** Graos (sentido contrario ao reloxo) da liña respecto do eixe X: 0 nunha páxina normal. */
  angle: number;
}
export interface TextLayerReport {
  output_path: string;
  pages: number;
  words: number;
}
export const pdfAddTextLayer = (input: string, output: string, pages: { page: number; words: OcrTextWord[] }[], overwrite = false): Promise<TextLayerReport> =>
  call("pdf_add_text_layer", { input, output, overwrite, pages });

// ---- PDF: limpar, índice, tapar e formularios
export interface SanitizeOptions { metadata: boolean; scripts: boolean; attachments: boolean; links: boolean; comments: boolean }
export interface SanitizeReport { output_path: string; metadata: number; scripts: number; attachments: number; links: number; comments: number }
export const pdfSanitize = (input: string, output: string, options: SanitizeOptions, overwrite = false): Promise<SanitizeReport> => call("pdf_sanitize", { input, output, overwrite, options });
export interface OutlineEntry { title: string; page: number; level: number }
export const pdfSetOutline = (input: string, output: string, entries: OutlineEntry[], overwrite = false): Promise<string> => call("pdf_set_outline", { input, output, overwrite, entries });
export interface RedactReport { output_path: string; pages: number; fields_removed: number }
/** Cada páxina chega xa debuxada (JPEG, co negro por riba): na saída pasa a ser unha soa imaxe. */
export function pdfRedact(input: string, output: string, pages: { page: number; jpeg: Uint8Array }[], overwrite = false): Promise<RedactReport> {
  const body = new Uint8Array(pages.reduce((n, p) => n + p.jpeg.length, 0));
  let at = 0;
  for (const p of pages) {
    body.set(p.jpeg, at);
    at += p.jpeg.length;
  }
  const meta = { input, output, overwrite, pages: pages.map((p) => [p.page, p.jpeg.length]) };
  return call("pdf_redact", body, { headers: { "x-meta": encodeURIComponent(JSON.stringify(meta)) } });
}
export interface IconOptions { windows: boolean; mac: boolean; web: boolean; png: boolean; fit: boolean }
export interface IconReport { folder: string; files: string[]; bytes_written: number; source_side: number }
export const makeIconPack = (input: string, dir: string, stem: string, options: IconOptions): Promise<IconReport> => call("make_icon_pack", { input, dir, stem, options });
export interface Attachment { name: string; size: number }
export const pdfListAttachments = (input: string): Promise<Attachment[]> => call("pdf_list_attachments", { input });
export interface AttachReport { output_path: string; added: number; removed: number }
export const pdfEditAttachments = (input: string, output: string, add: string[], remove: string[], overwrite = false): Promise<AttachReport> => call("pdf_edit_attachments", { input, output, overwrite, add, remove });
export interface FormField {
  name: string;
  kind: "text" | "check" | "radio" | "choice";
  value: string;
  options: { value: string; label: string }[];
  page: number;
  rect: [number, number, number, number];
  read_only: boolean;
  multiline: boolean;
}
export const pdfFormFields = (input: string): Promise<FormField[]> => call("pdf_form_fields", { input });
export interface FillReport { output_path: string; filled: number; flattened: boolean }
export const pdfFillForm = (input: string, output: string, values: { name: string; value: string }[], flatten: boolean, overwrite = false): Promise<FillReport> => call("pdf_fill_form", { input, output, overwrite, values, flatten });
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
export const forgetPreviews = (path: string): Promise<void> => call("forget_previews", { path });
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

export type LinkId = "ffmpeg" | "ffmpeg-license" | "mkvtoolnix" | "mkvtoolnix-license" | "iso639-2" | "mkvmerge-doc" | "libreoffice" | "libreoffice-license";
export const openUrl = (url: string): Promise<void> => call("open_url", { url });
export const openLink = (id: LinkId): Promise<void> => call("open_link", { id });

export interface ExplorerStatus {
  available: boolean;
  installed: boolean;
  current: boolean;
  complete: boolean;
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
