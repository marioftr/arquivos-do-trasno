/** Máis ferramentas da web: as mesmas que ten o programa de escritorio e caben nun navegador. */
import { tr } from "@app/lib/i18n";
import { EXT, encoders, sameMime, type Mime } from "./image";
import { formatBytes, stemOf, uniqueName, UserError, type Output } from "./files";
import { IMG, MEDIA, MIME_OF, outName, sizeNote, type Choice, type Tool, type Values } from "./tools";

const formats = (): Choice[] => [
  { value: "image/jpeg", label: "JPG" },
  { value: "image/png", label: "PNG" },
  ...(encoders.webp ? [{ value: "image/webp", label: "WebP" }] : []),
];
const num = (v: string) => Number(v.trim().replace(",", "."));
const VIDEO = "video/*,.mp4,.mov,.mkv,.webm";
const SUBS = ".srt,.vtt";
const PDF = "application/pdf,.pdf";
const txt = (text: string) => new Blob([text], { type: "text/plain;charset=utf-8" });

export const buildExtraTools = (): Tool[] => [
  // ---- Imaxe
  {
    id: "recortar",
    group: "imaxe",
    icon: "crop",
    title: tr("Recortar imaxes"),
    desc: tr("A unha proporción: cadrada, 16:9, retrato…"),
    info: tr("Corta a imaxe á proporción elixida, sen deformala. Elixe que parte queda; o resto non vai no resultado."),
    accept: IMG,
    multiple: true,
    options: [
      {
        id: "ratio",
        type: "seg",
        label: tr("Proporción"),
        choices: () => [
          { value: "1:1", label: "1:1", sub: tr("cadrada") },
          { value: "4:5", label: "4:5", sub: tr("retrato") },
          { value: "4:3", label: "4:3" },
          { value: "3:2", label: "3:2" },
          { value: "16:9", label: "16:9", sub: tr("panorámica") },
          { value: "9:16", label: "9:16", sub: tr("vertical") },
        ],
      },
      {
        id: "align",
        type: "seg",
        label: tr("Que parte queda"),
        choices: () => [
          { value: "0.5", label: tr("O centro") },
          { value: "0", label: tr("Arriba ou esquerda") },
          { value: "1", label: tr("Abaixo ou dereita") },
        ],
      },
    ],
    defaults: { ratio: "1:1", align: "0.5" },
    action: (n) => (n === 1 ? tr("Recortar 1 imaxe") : tr("Recortar {n} imaxes", { n })),
    each: async (file, v, taken) => {
      const { crop } = await import("./imgtools");
      const [a, b] = v.ratio.split(":").map(Number);
      const mime = sameMime(file);
      const r = await crop(file, a / b, Number(v.align), mime);
      return { name: outName(file, mime, taken, "-recortado"), blob: r.blob, note: `${r.width} × ${r.height} · ${formatBytes(r.blob.size)}` };
    },
  },
  {
    id: "dividir-imaxe",
    group: "imaxe",
    icon: "slice",
    title: tr("Dividir unha imaxe"),
    desc: tr("En partes iguais: carrusel ou cuadrícula."),
    info: tr("Corta a imaxe en columnas e filas iguais. Cada parte sae como unha imaxe, en orde de lectura."),
    accept: IMG,
    multiple: false,
    options: [
      {
        id: "grid",
        type: "seg",
        label: tr("Partes (columnas × filas)"),
        choices: () => [
          { value: "2x1", label: "2 × 1" },
          { value: "3x1", label: "3 × 1", sub: tr("carrusel") },
          { value: "1x2", label: "1 × 2" },
          { value: "1x3", label: "1 × 3" },
          { value: "2x2", label: "2 × 2" },
          { value: "3x3", label: "3 × 3" },
        ],
      },
    ],
    defaults: { grid: "3x1" },
    action: () => tr("Dividir a imaxe"),
    all: async ([file], v) => {
      const { split } = await import("./imgtools");
      const [cols, rows] = v.grid.split("x").map(Number);
      const mime = sameMime(file);
      const pieces = await split(file, cols, rows, mime);
      const taken = new Set<string>();
      return pieces.map((p, i) => ({ name: uniqueName(`${stemOf(file.name)}-${i + 1}.${EXT[mime]}`, taken), blob: p.blob, note: `${p.width} × ${p.height}` }));
    },
  },
  {
    id: "xuntar-imaxes",
    group: "imaxe",
    icon: "stitch",
    title: tr("Xuntar imaxes"),
    desc: tr("Unha tras outra nunha soa imaxe."),
    info: tr("Pon as imaxes unha tras outra, na orde da lista, co mesmo ancho (ou alto). Serve para xuntar capturas de pantalla."),
    accept: IMG,
    multiple: true,
    ordered: true,
    min: 2,
    options: [
      { id: "dir", type: "seg", label: tr("Dirección"), choices: () => [{ value: "v", label: tr("Unha debaixo de outra") }, { value: "h", label: tr("Unha ao lado de outra") }] },
      { id: "format", type: "seg", label: tr("Formato"), choices: formats },
    ],
    defaults: { dir: "v", format: "image/jpeg" },
    action: (n) => tr("Xuntar {n} imaxes", { n }),
    all: async (files, v, ctx) => {
      const { stitch } = await import("./imgtools");
      const mime = v.format as Mime;
      const r = await stitch(files, v.dir === "v", mime, (f) => ctx.progress(f));
      return [{ name: `${stemOf(files[0].name)}-xuntas.${EXT[mime]}`, blob: r.blob, note: `${r.width} × ${r.height} · ${formatBytes(r.blob.size)}` }];
    },
  },
  {
    id: "iconas",
    group: "imaxe",
    icon: "app-icons",
    title: tr("Iconas dunha imaxe"),
    desc: tr("ICO e PNG de varios tamaños para unha web ou app."),
    info: tr("Fai un .ico (16 a 256 px) e PNG de 32, 180, 192 e 512 px. Quedan mellor cunha imaxe cadrada de 512 px ou máis."),
    accept: IMG,
    multiple: false,
    options: [],
    defaults: {},
    action: () => tr("Crear as iconas"),
    all: async ([file]) => {
      const { squares, ico } = await import("./imgtools");
      const png = [512, 256, 192, 180, 128, 64, 48, 32, 16];
      const all = await squares(file, png);
      const stem = stemOf(file.name);
      const out: Output[] = [{ name: `${stem}.ico`, blob: await ico(new Map([16, 32, 48, 64, 128, 256].map((s) => [s, all.get(s)!]))), note: "16 · 32 · 48 · 64 · 128 · 256 px" }];
      for (const s of [512, 192, 180, 32]) out.push({ name: `${stem}-${s}.png`, blob: all.get(s)!, note: `${s} × ${s} px` });
      return out;
    },
  },
  {
    id: "info-imaxe",
    group: "imaxe",
    icon: "info",
    title: tr("Información da imaxe"),
    desc: tr("Tamaño, cámara, data e localización."),
    info: tr("Amosa o que leva a imaxe: tamaño, cámara, data e se garda a localización. Non modifica nada."),
    accept: IMG,
    multiple: true,
    options: [],
    defaults: {},
    action: (n) => (n === 1 ? tr("Ver a información") : tr("Ver a información de {n} imaxes", { n })),
    each: async (file, _v, taken) => {
      const { readExif } = await import("./exif");
      const { open } = await import("./image");
      let size = "";
      try {
        const s = await open(file);
        size = `${s.width} × ${s.height} px`;
        s.close();
      } catch {
        throw new UserError(tr("Non se puido abrir esta imaxe. O navegador non admite ese formato."));
      }
      const e = await readExif(file);
      const rows: [string, string][] = [
        [tr("Nome"), file.name],
        [tr("Tipo"), file.type || "—"],
        [tr("Tamaño do ficheiro"), formatBytes(file.size)],
        [tr("Dimensións"), size],
        [tr("Cámara"), [e.make, e.model].filter(Boolean).join(" ")],
        [tr("Obxectivo"), e.lens ?? ""],
        [tr("Data"), e.date ?? ""],
        [tr("Exposición"), [e.exposure, e.fnumber, e.iso ? `ISO ${e.iso}` : "", e.focal].filter(Boolean).join(" · ")],
        [tr("Programa"), e.software ?? ""],
        [tr("Localización"), e.lat !== undefined && e.lon !== undefined ? `${e.lat.toFixed(5)}, ${e.lon.toFixed(5)}` : ""],
      ];
      const text = rows.filter(([, v]) => v).map(([k, v]) => `${k}: ${v}`).join("\n");
      const note = e.lat !== undefined ? tr("Leva a localización. Converter imaxes quítaa.") : undefined;
      return { name: uniqueName(`${stemOf(file.name)}-info.txt`, taken), blob: txt(text), text, note };
    },
  },

  // ---- PDF
  {
    id: "ordenar-pdf",
    group: "pdf",
    icon: "pages",
    title: tr("Ordenar PDF"),
    desc: tr("Quita, inverte ou reordena as páxinas."),
    info: tr("Quita páxinas, dálle a volta á orde ou escribe a orde nova (3, 1, 2-4). Unha páxina pódese repetir."),
    accept: PDF,
    multiple: false,
    options: [
      {
        id: "mode",
        type: "seg",
        label: tr("Como"),
        choices: () => [
          { value: "remove", label: tr("Quitar páxinas") },
          { value: "reverse", label: tr("Dar a volta") },
          { value: "order", label: tr("Escribir a orde") },
        ],
      },
      { id: "remove", type: "text", label: tr("Páxinas a quitar"), placeholder: "2, 5-7", when: (v) => v.mode === "remove" },
      { id: "order", type: "text", label: tr("Nova orde das páxinas"), placeholder: "3, 1, 2-4", when: (v) => v.mode === "order" },
    ],
    defaults: { mode: "remove", remove: "", order: "" },
    action: () => tr("Ordenar PDF"),
    all: async ([file], v) => [await (await import("./pdf-more")).reorder(file, v.mode as "remove", v.mode === "order" ? v.order : v.remove)],
  },
  {
    id: "recortar-pdf",
    group: "pdf",
    icon: "crop",
    title: tr("Recortar márxenes"),
    desc: tr("Quita o bordo en branco de todas as páxinas."),
    info: tr("Corta o mesmo bordo en todas as páxinas. O contido non se borra: o PDF só amosa menos área."),
    accept: PDF,
    multiple: true,
    options: [
      {
        id: "mm",
        type: "seg",
        label: tr("Bordo a cortar"),
        choices: () => [5, 10, 20, 30].map((n) => ({ value: String(n), label: `${n} mm` })),
      },
    ],
    defaults: { mm: "10" },
    action: (n) => (n === 1 ? tr("Recortar 1 PDF") : tr("Recortar {n} PDF", { n })),
    each: async (file, v) => (await import("./pdf-more")).cropMargins(file, Number(v.mm)),
  },
  {
    id: "marca-pdf",
    group: "pdf",
    icon: "stamp",
    title: tr("Marca de auga"),
    desc: tr("Un texto en diagonal en todas as páxinas."),
    info: tr("Escribe un texto en diagonal en cada páxina, como «BORRADOR» ou «CONFIDENCIAL». Admite letras latinas e acentos."),
    accept: PDF,
    multiple: true,
    options: [
      { id: "text", type: "text", label: tr("Texto"), placeholder: "BORRADOR" },
      {
        id: "opacity",
        type: "seg",
        label: tr("Intensidade"),
        choices: () => [
          { value: "0.15", label: tr("Suave") },
          { value: "0.3", label: tr("Normal") },
          { value: "0.5", label: tr("Forte") },
        ],
      },
    ],
    defaults: { text: "BORRADOR", opacity: "0.3" },
    action: (n) => (n === 1 ? tr("Marcar 1 PDF") : tr("Marcar {n} PDF", { n })),
    each: async (file, v) => (await import("./pdf-more")).watermark(file, v.text, Number(v.opacity)),
  },
  {
    id: "numeros-pdf",
    group: "pdf",
    icon: "hash",
    title: tr("Numerar páxinas"),
    desc: tr("Engade o número en cada páxina."),
    info: tr("Escribe o número de páxina ao pé. Elixe o lado e se leva o total."),
    accept: PDF,
    multiple: true,
    options: [
      { id: "side", type: "seg", label: tr("Posición"), choices: () => [{ value: "left", label: tr("Esquerda") }, { value: "center", label: tr("Centro") }, { value: "right", label: tr("Dereita") }] },
      { id: "total", type: "seg", label: tr("Formato"), choices: () => [{ value: "no", label: "1" }, { value: "yes", label: "1 / 12" }] },
    ],
    defaults: { side: "center", total: "no" },
    action: (n) => (n === 1 ? tr("Numerar 1 PDF") : tr("Numerar {n} PDF", { n })),
    each: async (file, v) => (await import("./pdf-more")).numberPages(file, v.side as "left", v.total === "yes"),
  },
  {
    id: "limpar-pdf",
    group: "pdf",
    icon: "eraser",
    title: tr("Limpar PDF"),
    desc: tr("Quita metadatos ou aplana formularios."),
    info: tr("Quita autor, título e outros datos ocultos, ou deixa os campos dun formulario como texto fixo. O contido non cambia."),
    accept: PDF,
    multiple: true,
    options: [
      {
        id: "what",
        type: "seg",
        label: tr("Que facer"),
        choices: () => [
          { value: "meta", label: tr("Quitar metadatos") },
          { value: "flat", label: tr("Aplanar formulario") },
          { value: "both", label: tr("Os dous") },
        ],
      },
    ],
    defaults: { what: "meta" },
    action: (n) => (n === 1 ? tr("Limpar 1 PDF") : tr("Limpar {n} PDF", { n })),
    each: async (file, v) => (await import("./pdf-more")).clean(file, v.what !== "flat", v.what !== "meta"),
  },
  {
    id: "info-pdf",
    group: "pdf",
    icon: "info",
    title: tr("Información do PDF"),
    desc: tr("Páxinas, tamaño, autor e datas."),
    info: tr("Amosa o que leva o PDF: páxinas, tamaño de páxina, autor e datas. Non modifica nada."),
    accept: PDF,
    multiple: false,
    options: [],
    defaults: {},
    action: () => tr("Ver a información"),
    all: async ([file]) => [await (await import("./pdf-more")).info(file)],
  },

  // ---- Audio e vídeo
  {
    id: "video",
    group: "audio",
    icon: "video",
    title: tr("Comprimir vídeo"),
    desc: tr("A MP4, máis pequeno. Vale para case todo."),
    info: tr("Volve gardar o vídeo en MP4 (H.264) con menos peso. No navegador tarda máis que na app: mellor con vídeos curtos."),
    accept: VIDEO,
    multiple: false,
    options: [
      {
        id: "crf",
        type: "seg",
        label: tr("Calidade"),
        choices: () => [
          { value: "23", label: tr("Alta") },
          { value: "27", label: tr("Media") },
          { value: "31", label: tr("Baixa") },
        ],
      },
      {
        id: "height",
        type: "seg",
        label: tr("Tamaño"),
        info: tr("Altura máxima en píxeles. Non amplía vídeos máis pequenos."),
        choices: () => [
          { value: "0", label: tr("Orixinal") },
          { value: "1080", label: "1080p" },
          { value: "720", label: "720p" },
          { value: "480", label: "480p" },
        ],
      },
    ],
    defaults: { crf: "27", height: "720" },
    action: () => tr("Comprimir o vídeo"),
    each: async (file, v, taken, ctx) => {
      const { run } = await import("./media");
      const vf = Number(v.height) ? ["-vf", `scale=-2:'min(${Number(v.height)},ih)'`] : [];
      const blob = await run(file, "mp4", (i, o) => ["-i", i, "-c:v", "libx264", "-preset", "veryfast", "-crf", v.crf, ...vf, "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "128k", "-movflags", "+faststart", o], ctx, tr("Comprimindo…"));
      return { name: uniqueName(`${stemOf(file.name)}-comprimido.mp4`, taken), blob: new Blob([blob], { type: "video/mp4" }), note: sizeNote(file, blob, false) };
    },
  },
  {
    id: "axustar",
    group: "audio",
    icon: "sliders",
    title: tr("Velocidade e volume"),
    desc: tr("Máis rápido, máis lento, máis ou menos volume."),
    info: tr("Cambia a velocidade ou o volume, ou quita o son dun vídeo. O vídeo sae en MP4 e o audio en MP3."),
    accept: MEDIA,
    multiple: false,
    options: [
      { id: "mode", type: "seg", label: tr("Que cambiar"), choices: () => [{ value: "speed", label: tr("Velocidade") }, { value: "volume", label: tr("Volume") }, { value: "mute", label: tr("Quitar o son") }] },
      { id: "speed", type: "seg", label: tr("Velocidade"), when: (v) => v.mode === "speed", choices: () => ["0.5", "0.75", "1.25", "1.5", "2"].map((s, i) => ({ value: s, label: ["½×", "¾×", "1¼×", "1½×", "2×"][i] })) },
      { id: "volume", type: "seg", label: tr("Volume"), when: (v) => v.mode === "volume", choices: () => ["0.5", "1.5", "2", "3"].map((s) => ({ value: s, label: `${Number(s) * 100} %` })) },
    ],
    defaults: { mode: "speed", speed: "1.5", volume: "1.5" },
    action: () => tr("Aplicar"),
    each: async (file, v, taken, ctx) => {
      const { run, isVideo } = await import("./media");
      const video = isVideo(file);
      if (v.mode === "mute" && !video) throw new UserError(tr("Quitar o son só vale para vídeo."));
      const ext = video ? "mp4" : "mp3";
      const audio = video ? ["-c:a", "aac", "-b:a", "160k"] : ["-c:a", "libmp3lame", "-b:a", "192k"];
      const args = (i: string, o: string): string[] =>
        v.mode === "mute"
          ? ["-i", i, "-an", "-c:v", "copy", o]
          : v.mode === "volume"
            ? ["-i", i, ...(video ? ["-c:v", "copy"] : ["-vn"]), "-af", `volume=${v.volume}`, ...audio, o]
            : ["-i", i, ...(video ? ["-vf", `setpts=PTS/${v.speed}`, "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p"] : ["-vn"]), "-af", `atempo=${v.speed}`, ...audio, o];
      const blob = await run(file, ext, args, ctx, tr("Aplicando…"));
      return { name: uniqueName(`${stemOf(file.name)}-${v.mode === "speed" ? "velocidade" : v.mode === "volume" ? "volume" : "sen-son"}.${ext}`, taken), blob: new Blob([blob], { type: video ? "video/mp4" : "audio/mpeg" }), note: formatBytes(blob.size) };
    },
  },
  {
    id: "unir-audios",
    group: "audio",
    icon: "merge",
    title: tr("Unir audios"),
    desc: tr("Xunta varios audios nun só."),
    info: tr("Xunta os audios na orde da lista nun ficheiro novo. Usa as frechas para cambiar a orde."),
    accept: "audio/*,.mp3,.m4a,.wav,.ogg,.opus,.flac",
    multiple: true,
    ordered: true,
    min: 2,
    options: [{ id: "format", type: "seg", label: tr("Formato"), choices: () => [{ value: "mp3", label: "MP3" }, { value: "m4a", label: "M4A" }, { value: "wav", label: "WAV", sub: tr("sen perdas") }] }],
    defaults: { format: "mp3" },
    action: (n) => tr("Unir {n} audios", { n }),
    all: async (files, v, ctx) => {
      const { runMany } = await import("./media");
      const codec = v.format === "wav" ? ["-c:a", "pcm_s16le"] : ["-c:a", v.format === "m4a" ? "aac" : "libmp3lame", "-b:a", "192k"];
      const blob = await runMany(
        files,
        v.format,
        (ins, o) => [...ins.flatMap((i) => ["-i", i]), "-filter_complex", `${ins.map((_, k) => `[${k}:a]`).join("")}concat=n=${ins.length}:v=0:a=1[a]`, "-map", "[a]", ...codec, o],
        ctx,
        tr("Xuntando…"),
      );
      return [{ name: `${stemOf(files[0].name)}-unido.${v.format}`, blob: new Blob([blob], { type: MIME_OF[v.format] }), note: formatBytes(blob.size) }];
    },
  },
  {
    id: "fotograma",
    group: "audio",
    icon: "extract",
    title: tr("Fotograma dun vídeo"),
    desc: tr("Garda unha imaxe do instante que elixas."),
    info: tr("Saca unha imaxe do vídeo no instante indicado. Exemplos: 45, 1:30, 1:02:03"),
    accept: VIDEO,
    multiple: false,
    options: [
      { id: "at", type: "text", label: tr("Instante"), placeholder: "0:10" },
      { id: "format", type: "seg", label: tr("Formato"), choices: () => [{ value: "jpg", label: "JPG" }, { value: "png", label: "PNG" }] },
    ],
    defaults: { at: "", format: "jpg" },
    action: () => tr("Sacar a imaxe"),
    each: async (file, v, taken, ctx) => {
      const { run, parseTime } = await import("./media");
      const t = v.at.trim() ? parseTime(v.at) : 0;
      if (t === null) throw new UserError(tr("Escribe o tempo así: 45, 1:30 ou 1:02:03."));
      const blob = await run(file, v.format, (i, o) => ["-ss", String(t), "-i", i, "-frames:v", "1", ...(v.format === "jpg" ? ["-q:v", "2"] : []), o], ctx, tr("Sacando a imaxe…"));
      if (!blob.size) throw new UserError(tr("Non hai imaxe nese instante. Proba cun tempo anterior."));
      return { name: uniqueName(`${stemOf(file.name)}-fotograma.${v.format}`, taken), blob: new Blob([blob], { type: v.format === "jpg" ? "image/jpeg" : "image/png" }), note: formatBytes(blob.size) };
    },
  },

  // ---- Subtítulos
  {
    id: "converter-subs",
    group: "subs",
    icon: "convert",
    title: tr("Converter subtítulos"),
    desc: tr("Entre SRT, VTT e texto simple."),
    info: tr("Pasa os subtítulos a SRT ou VTT, ou deixa só o texto, sen tempos."),
    accept: SUBS,
    multiple: true,
    options: [{ id: "format", type: "seg", label: tr("Formato"), choices: () => [{ value: "srt", label: "SRT" }, { value: "vtt", label: "VTT" }, { value: "txt", label: tr("Só texto") }] }],
    defaults: { format: "srt" },
    action: (n) => (n === 1 ? tr("Converter 1 ficheiro") : tr("Converter {n} ficheiros", { n })),
    each: async (file, v, taken) => {
      const subs = await import("./subs");
      const cues = subs.parseSubs(await file.text());
      if (!cues.length) throw new UserError(tr("Non atopei subtítulos neste ficheiro."));
      const text = v.format === "vtt" ? subs.toVtt(cues) : v.format === "txt" ? subs.toTxt(cues) : subs.toSrt(cues);
      return { name: uniqueName(`${stemOf(file.name)}.${v.format}`, taken), blob: txt(text), text, note: tr("{n} subtítulos", { n: cues.length }) };
    },
  },
  {
    id: "resincronizar",
    group: "subs",
    icon: "clock",
    title: tr("Resincronizar subtítulos"),
    desc: tr("Adianta, atrasa ou cambia a cadencia."),
    info: tr("Move todos os subtítulos uns segundos (en negativo, adianta) ou cambia a cadencia se van máis rápido ou máis lento que o vídeo."),
    accept: SUBS,
    multiple: true,
    options: [
      { id: "shift", type: "text", label: tr("Desprazar (segundos)"), placeholder: "2,5  ·  -1" },
      {
        id: "fps",
        type: "seg",
        label: tr("Cadencia"),
        choices: () => [
          { value: "1", label: tr("Igual") },
          { value: "23.976/25", label: "23,976 → 25" },
          { value: "25/23.976", label: "25 → 23,976" },
          { value: "24/25", label: "24 → 25" },
          { value: "25/24", label: "25 → 24" },
        ],
      },
    ],
    defaults: { shift: "", fps: "1" },
    action: (n) => (n === 1 ? tr("Resincronizar 1 ficheiro") : tr("Resincronizar {n} ficheiros", { n })),
    each: async (file, v: Values, taken) => {
      const subs = await import("./subs");
      const shift = v.shift.trim() ? num(v.shift) : 0;
      if (Number.isNaN(shift)) throw new UserError(tr("Escribe os segundos así: 2,5 ou -1."));
      const [a, b] = v.fps.split("/").map(Number);
      const ratio = b ? a / b : 1;
      if (!shift && ratio === 1) throw new UserError(tr("Escribe canto queres desprazar ou elixe un cambio de cadencia."));
      const cues = subs.parseSubs(await file.text());
      if (!cues.length) throw new UserError(tr("Non atopei subtítulos neste ficheiro."));
      const done = subs.retime(cues, shift, ratio);
      const vtt = /\.vtt$/i.test(file.name);
      return { name: uniqueName(`${stemOf(file.name)}-sincro.${vtt ? "vtt" : "srt"}`, taken), blob: txt(vtt ? subs.toVtt(done) : subs.toSrt(done)), text: subs.toTxt(done), note: tr("{n} subtítulos", { n: done.length }) };
    },
  },
];
