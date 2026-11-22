/** Procesado de imaxes no navegador con canvas. Re-codificar tamén quita os metadatos (GPS, cámara…). */

/** Máximo de píxeles dun canvas que aguantan todos os navegadores (o límite de iOS Safari é 16,7 Mpx). */
const MAX_AREA = 16_000_000;

export type Mime = "image/jpeg" | "image/png" | "image/webp";
export const EXT: Record<Mime, string> = { "image/jpeg": "jpg", "image/png": "png", "image/webp": "webp" };

/** Formatos que este navegador sabe escribir (Safari non escribe WebP: devolvería un PNG). */
export const encoders = { webp: false };
export async function detectEncoders(): Promise<void> {
  try {
    const c = document.createElement("canvas");
    c.width = c.height = 1;
    const b = await new Promise<Blob | null>((r) => c.toBlob(r, "image/webp", 0.8));
    encoders.webp = b?.type === "image/webp";
  } catch {
    encoders.webp = false;
  }
}

export interface RenderOptions {
  mime: Mime;
  /** 0–1 (só en JPG e WebP). */
  quality?: number;
  /** Lado maior máximo en píxeles (non amplía). */
  maxSide?: number;
  /** Porcentaxe do tamaño orixinal. */
  percent?: number;
  /** Xiro en graos, sentido horario. */
  rotate?: 0 | 90 | 180 | 270;
  flipH?: boolean;
  flipV?: boolean;
}

export interface Rendered {
  blob: Blob;
  width: number;
  height: number;
  /** Reduciuse máis do pedido porque o navegador non aguanta imaxes tan grandes. */
  capped: boolean;
}

type Source = { img: CanvasImageSource; width: number; height: number; close: () => void };

async function open(file: File): Promise<Source> {
  try {
    const bmp = await createImageBitmap(file, { imageOrientation: "from-image" });
    return { img: bmp, width: bmp.width, height: bmp.height, close: () => bmp.close() };
  } catch {
    // Safari antigo ou formato que só entende <img>.
  }
  const url = URL.createObjectURL(file);
  const el = new Image();
  el.decoding = "async";
  el.src = url;
  try {
    await el.decode();
  } catch {
    URL.revokeObjectURL(url);
    throw new Error("formato");
  }
  return { img: el, width: el.naturalWidth, height: el.naturalHeight, close: () => URL.revokeObjectURL(url) };
}

export async function render(file: File, o: RenderOptions): Promise<Rendered> {
  const src = await open(file);
  try {
    let k = 1;
    if (o.percent) k = o.percent / 100;
    else if (o.maxSide) k = Math.min(1, o.maxSide / Math.max(src.width, src.height));
    let w = Math.max(1, Math.round(src.width * k));
    let h = Math.max(1, Math.round(src.height * k));
    let capped = false;
    if (w * h > MAX_AREA) {
      const s = Math.sqrt(MAX_AREA / (w * h));
      w = Math.max(1, Math.floor(w * s));
      h = Math.max(1, Math.floor(h * s));
      capped = true;
    }
    const turn = (o.rotate ?? 0) % 180 === 90;
    const canvas = document.createElement("canvas");
    canvas.width = turn ? h : w;
    canvas.height = turn ? w : h;
    const g = canvas.getContext("2d");
    if (!g) throw new Error("canvas");
    if (o.mime === "image/jpeg") {
      // O JPG non ten transparencia: o que era transparente queda en branco.
      g.fillStyle = "#fff";
      g.fillRect(0, 0, canvas.width, canvas.height);
    }
    g.imageSmoothingQuality = "high";
    g.translate(canvas.width / 2, canvas.height / 2);
    g.rotate(((o.rotate ?? 0) * Math.PI) / 180);
    g.scale(o.flipH ? -1 : 1, o.flipV ? -1 : 1);
    g.drawImage(src.img, -w / 2, -h / 2, w, h);
    const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, o.mime, o.mime === "image/png" ? undefined : (o.quality ?? 0.85)));
    const width = canvas.width;
    const height = canvas.height;
    canvas.width = canvas.height = 0; // devolve a memoria (iOS é xusto)
    if (!blob) throw new Error("codificar");
    return { blob, width, height, capped };
  } finally {
    src.close();
  }
}

/** Mime de saída «igual que o orixinal» (o que non se pode escribir, JPG ou PNG). */
export function sameMime(file: File): Mime {
  if (file.type === "image/png") return "image/png";
  if (file.type === "image/webp" && encoders.webp) return "image/webp";
  if (file.type === "image/jpeg") return "image/jpeg";
  return file.type === "image/gif" || file.type === "image/bmp" || file.type === "image/svg+xml" ? "image/png" : "image/jpeg";
}

/** Miniatura pequena (96 px) para a lista: así non se descodifica unha foto de 12 Mpx só para amosala en 3 rem. */
const queue: (() => Promise<void>)[] = [];
let working = false;
async function drain() {
  if (working) return;
  working = true;
  for (let job = queue.shift(); job; job = queue.shift()) await job();
  working = false;
}
export function thumbnail(file: File, size = 96): Promise<string> {
  return new Promise((resolve) => {
    queue.push(async () => {
      try {
        const bmp = await createImageBitmap(file, { imageOrientation: "from-image", resizeWidth: size * 2, resizeQuality: "low" }).catch(() => createImageBitmap(file));
        const c = document.createElement("canvas");
        c.width = c.height = size;
        // Recortada ao centro, en cadrado.
        const k = Math.max(size / bmp.width, size / bmp.height);
        c.getContext("2d")?.drawImage(bmp, (size - bmp.width * k) / 2, (size - bmp.height * k) / 2, bmp.width * k, bmp.height * k);
        bmp.close();
        const blob = await new Promise<Blob | null>((r) => c.toBlob(r, "image/jpeg", 0.7));
        resolve(blob ? URL.createObjectURL(blob) : "");
      } catch {
        resolve(""); // sen miniatura (formato que este navegador non abre): amósase a icona
      }
    });
    void drain();
  });
}
