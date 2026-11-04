// O trasno como axente: lembra o que fixo a persoa (para contalo na barra inferior) e ten un
// repertorio de consellos que solta de cando en vez. Os consellos pódense apagar en Axustes.

import { writable, get } from "svelte/store";
import { tr } from "./i18n";

function persistedBool(key: string, fallback: boolean) {
  let v = fallback;
  try {
    const s = localStorage.getItem(key);
    if (s === "1" || s === "0") v = s === "1";
  } catch {
    // ignorar
  }
  const store = writable(v);
  store.subscribe((x) => {
    try {
      localStorage.setItem(key, x ? "1" : "0");
    } catch {
      // ignorar
    }
  });
  return store;
}

export const tipsOn = persistedBool("arquivos-do-trasno.tips", true);

export interface Action {
  text: string;
  at: number;
  /** Como deixar as cousas como estaban antes desta acción (se se pode). */
  undo?: () => void;
  /** Texto completo para copiar (os erros). */
  copy?: string;
}
/** Últimas accións, a máis recente primeiro. Sempre en terceira persoa: o programa conta o que fixo o trasno. */
export const actions = writable<Action[]>([]);
export function noteAction(gl: string, vars?: Record<string, string | number>, undo?: () => void, copy?: string) {
  const text = tr(gl, vars);
  actions.update((l) => (l[0]?.text === text && Date.now() - l[0].at < 4000 ? [{ text, at: Date.now(), undo: l[0].undo ?? undo, copy: copy ?? l[0].copy }, ...l.slice(1)] : [{ text, at: Date.now(), undo, copy }, ...l].slice(0, 300)));
}

/** Número de cambios que se desfán ao premer a acción `idx` (0 é a máis recente): ela e todas as posteriores. */
export const revertCount = (list: Action[], idx: number) => list.slice(0, idx + 1).filter((a) => a.undo).length;

/** Desfai, da máis nova á máis vella, a acción `idx` e todo o que veu despois. */
export function revertTo(idx: number): number {
  const l = get(actions);
  if (!l[idx]) return 0;
  let n = 0;
  for (const a of l.slice(0, idx + 1)) {
    try {
      if (a.undo) {
        a.undo();
        n++;
      }
    } catch {
      // un paso que non se pode desfacer non debe frear os demais
    }
  }
  actions.set(l.slice(idx + 1));
  noteAction(n === 1 ? "O trasno retrocedeu un cambio" : "O trasno retrocedeu {n} cambios", { n });
  return n;
}

/** Que fai o trasno ao premelo: abrir procesos, abrir o buscador ou soltar un comentario. */
export type ClickAction = "procs" | "palette" | "settings" | "say";
function persistedText<T extends string>(key: string, allowed: T[], fallback: T) {
  let v = fallback;
  try {
    const s = localStorage.getItem(key);
    if (s && (allowed as string[]).includes(s)) v = s as T;
  } catch {
    // ignorar
  }
  const store = writable<T>(v);
  store.subscribe((x) => {
    try {
      localStorage.setItem(key, x);
    } catch {
      // ignorar
    }
  });
  return store;
}
export const CLICK_ITEMS: { id: ClickAction; icon: string; label: string; action: string; tone: string; actionId?: string }[] = [
  { id: "palette", icon: "search", label: "Buscar", action: "Abrir o buscador", tone: "var(--accent)", actionId: "g.palette" },
  { id: "procs", icon: "layers", label: "Procesos", action: "Abrir os procesos", tone: "var(--accent-conversor)" },
  { id: "say", icon: "chat", label: "Comentario do trasno", action: "Soltar un comentario", tone: "var(--accent-pdf)" },
  { id: "settings", icon: "gear", label: "Axustes", action: "Abrir os axustes", tone: "var(--text-muted)" },
];
export const trasnoClick = persistedText<ClickAction>("arquivos-do-trasno.trasno-click", ["procs", "palette", "settings", "say"], "procs");

// ---- o globo do trasno (un só á vez)
export interface Bubble {
  id: number;
  text: string;
  /** «tip» leva o prefixo «Sabías que…»; «info» é un comentario ou aviso sen máis. */
  kind: "tip" | "info" | "warn";
  action?: { label: string; run: () => void };
  /** Texto que o botón de copiar deixa no portapapeis. */
  copy?: string;
  /** Notificación: o trasno pestanexa e queda sorprendido. */
  notify?: boolean;
}
export const bubble = writable<Bubble | null>(null);
let bubbleId = 0;
let bubbleTimer: ReturnType<typeof setTimeout> | undefined;
export function trasnoSay(text: string, kind: Bubble["kind"] = "info", ms = 14000, action?: Bubble["action"], extra: { copy?: string; notify?: boolean } = {}) {
  const id = ++bubbleId;
  bubble.set({ id, text, kind, action, copy: extra.copy, notify: extra.notify });
  clearTimeout(bubbleTimer);
  bubbleTimer = setTimeout(() => bubble.update((b) => (b?.id === id ? null : b)), ms);
}
/** Mentres o punteiro está sobre o globo non se vai; ao saír, queda un pouco máis para acabar de ler. */
export function holdBubble() {
  clearTimeout(bubbleTimer);
}
export function releaseBubble(ms = 5000) {
  const id = get(bubble)?.id;
  clearTimeout(bubbleTimer);
  bubbleTimer = setTimeout(() => bubble.update((b) => (b?.id === id ? null : b)), ms);
}
export function hideBubble() {
  clearTimeout(bubbleTimer);
  bubble.set(null);
}

/** O trasno non pode estar en dous sitios: mentres voa ata o botón de empezar, o da barra agóchase. */
export const trasnoAway = writable(false);

/** Ficheiro e ferramenta á vista: os comentarios do trasno tenen que ver con eles. */
export const viewCtx = writable<{ category?: string; tool?: string }>({});

/** Avance en directo do traballo da app en marcha (a lista de procesos só se refresca cada poucos segundos). */
export const liveProgress = writable<number | null>(null);

export const TIPS = [
  "Ctrl+K abre o buscador de accións.",
  "Arrastrar ficheiros á xanela é a forma máis rápida de engadilos.",
  "Coa ferramenta Seleccionar pódese mover, escalar e xirar calquera anotación.",
  "Ctrl+Z desfai o último cambio dos axustes, e Restablecer volve aos valores por defecto.",
  "Cada ficheiro lembra os seus propios axustes mentres a aplicación siga aberta.",
  "O botón central do rato move a imaxe aínda que estea activa unha ferramenta de anotación.",
  "Os ficheiros da lista pódense arrastrar para cambiarlles a orde.",
  "Se se marcan varios ficheiros do mesmo tipo, a ferramenta aplícase a todos á vez.",
  "Quitar un ficheiro da lista non o borra do disco, e pódese desfacer.",
  "O recadro de progreso da barra inferior abre a lista de procesos.",
  "O texto das imaxes pódese seleccionar e copiar coa ferramenta de texto.",
  "O historial da barra inferior permite volver ao estado de calquera acción anterior.",
  "Ctrl+O abre o selector de ficheiros.",
  "Ctrl+W quita da lista o ficheiro que se está vendo, sen borralo do disco.",
  "O que fai o trasno ao premelo pódese cambiar en Axustes.",
  "Mantendo o punteiro sobre o trasno sae un menú con accesos rápidos.",
  "Os atallos de teclado pódense cambiar en Axustes.",
  "Todo o que remata queda rexistrado no historial de procesos.",
  "Maiús + clic na lista marca todos os ficheiros entre dous.",
  "Se un proceso se cancela, non queda ningún ficheiro a medias.",
  "O buscador de accións (Ctrl+K) tamén atopa ferramentas e axustes.",
  "O número de versión da barra inferior abre as novidades.",
  "No historial da barra inferior, premer unha acción desfai ese cambio e os posteriores.",
];

// ---- comentarios segundo o ficheiro aberto e a ferramenta activa
export const CTX: Record<string, string[]> = {
  "*": ["Aquí sigo, listo para o que faga falta.", "Cando queiras, pásame outro ficheiro: non me canso.", "Se algo sae raro, un Ctrl+Z e aquí non pasou nada.", "Na barra de abaixo apunto todo o que se fai, por se hai que desfacer algo.", "Non hai présa: eu aquí sigo.", "Se tocas algo sen querer, o historial da barra de abaixo sabe volver atrás.", "Podes arrastrarme ficheiros directamente á xanela: eu encárgome do resto.", "Levo a conta de todo o que fas, pero non xulgo.", "Un trasno ordenado é un trasno feliz. Un ficheiro ordenado, tamén.", "Se algo tarda, probablemente é un vídeo. Os vídeos sempre tardan.", "Prefiro os ficheiros pequenos, pero respecto os grandes."],
  imaxe: ["O WebP adoita pesar bastante menos que o JPG coa mesma calidade.", "O PNG conserva a transparencia; o JPG non.", "O JPG perde un chisco de calidade cada vez que se garda: convén traballar sobre unha copia."],
  video: ["Un bitrate máis baixo dá ficheiros máis pequenos, ata que se nota: o truco é parar un chisco antes.", "O H.265 reduce o peso, pero non todos os reprodutores o len.", "Os vídeos entrelazados de cinta piden desentrelazado ao convertilos.", "Reducir a altura máxima a 720p aforra moito peso e apenas se nota nunha pantalla pequena."],
  audio: ["Con 128 kbps abonda para voz; para música convén 192 ou máis.", "O MP3 é o máis compatible; o FLAC garda todo sen perdas.", "Eu non xulgo o gusto musical de ninguén: só convirto.", "O WAV non leva compresión: pesa moito, pero é perfecto para editar.", "Subir o bitrate non mellora un audio que xa viña comprimido."],
  documento: ["Un PDF pesado adoita levar imaxes sen optimizar: Comprimir pode axudar moito.", "Un PDF escaneado é unha imaxe: sen OCR, o seu texto non se pode seleccionar.", "Ctrl+F busca texto dentro do PDF aberto."],
  modelo3d: ["No visor do modelo, arrastrar co rato xira a vista e a roda achega e afasta.", "Os modelos FBX e glTF ábrense co mesmo cargador do visor."],
  convert: ["Cambiar de formato nunca toca o ficheiro orixinal: sempre se garda unha copia.", "Eu non xulgo: convirto o que sexa, mentres teña solución.", "Con varios ficheiros marcados, a conversión faise en lote."],
  compress: ["En Comprimir, cada ficheiro lembra os seus axustes mentres a aplicación siga aberta.", "Estimar o tamaño antes de comprimir evita sorpresas."],
  annotate: ["A goma borra trazo a trazo; «Borrar todas as anotacións» limpa a imaxe de golpe.", "A letra, o tamaño e a cor dun texto pódense cambiar despois de escribilo.", "Cada ferramenta de debuxo lembra o seu propio groso."],
  info: ["A pestana Información amosa metadatos que o visor non ensina.", "A información dun ficheiro inclúe o seu SHA-256, que serve para comparar dous ficheiros."],
  extract: ["Extraer o son dun vídeo non modifica o vídeo orixinal."],
  mkv: ["Cambiar as pistas dun MKV non recodifica nada: só se reempaqueta."],
  merge: ["Na unión, a orde da lista é a orde final do resultado."],
  pages: ["Marcar as páxinas nas miniaturas é máis rápido que escribir os números."],
  marks: ["As marcas e a numeración aplícanse á copia, nunca ao orixinal."],
  edit: ["O resultado de Editar gárdase como unha copia: o orixinal queda intacto."],
  security: ["Se se perde o contrasinal dun PDF cifrado, non hai maneira de recuperalo."],
};

/** As últimas frases ditas: non se repite ningunha ata que pasen outras 10. */
const recent: string[] = [];
/** Un comentario ao chou, con preferencia polo que ten que ver co ficheiro e a ferramenta. */
export function pickComment(category?: string, tool?: string): Bubble {
  const ctx = [...(CTX[tool ?? ""] ?? []), ...(CTX[category ?? ""] ?? []), ...CTX["*"]].map((text) => ({ text, kind: "info" as const }));
  const tips = TIPS.map((text) => ({ text, kind: "tip" as const }));
  const all = [...ctx, ...ctx, ...tips];
  let pool = all.filter((x) => !recent.includes(x.text));
  if (!pool.length) pool = all;
  const c = pool[Math.floor(Math.random() * pool.length)];
  recent.push(c.text);
  if (recent.length > 10) recent.shift();
  return { id: 0, text: c.text, kind: c.kind };
}

// ---- axustes cambiados: o trasno conta o que se tocou
const LABEL: Record<string, string> = {
  targetExt: "o formato", quality: "a calidade", audioBitrate: "o bitrate de audio", audioKbps: "o bitrate de audio", bitrate: "o bitrate", videoKbps: "o bitrate de vídeo",
  crf: "a calidade (CRF)", profile: "o perfil", maxHeight: "a altura máxima", maxSide: "o lado máximo", resizeW: "o ancho", resizeH: "o alto", keepRatio: "manter a proporción",
  dar: "a relación de aspecto", deinterlace: "o desentrelazado", rotate: "o xiro", flipH: "o espello horizontal", flipV: "o espello vertical", grayscale: "a escala de grises",
  invert: "a inversión de cor", blur: "o desenfoque", brightness: "o brillo", contrast: "o contraste", volume: "o volume", speed: "a velocidade", pageSpec: "as páxinas",
  cropOn: "o recorte", cw: "o recorte", cx: "o recorte", cy: "o recorte", wmText: "o texto da marca", wmSize: "o tamaño da marca", wmColor: "a cor da marca",
  wmOpacity: "a opacidade da marca", wmAngle: "o xiro da marca", wmFamily: "a letra da marca", wmBold: "a negriña da marca", wmItalic: "a cursiva da marca",
  lossless: "o modo sen perdas", mode: "o modo", compat: "a compatibilidade", sizeValue: "o tamaño obxectivo", select: "a pista escollida", kind: "o modo",
};
// Valores de texto que se amosan tal cal; para os demais amósase só o que cambiou (nunca un código interno).
const RAW_VALUE = new Set(["targetExt", "pageSpec", "wmText", "wmFamily", "dar", "maxHeight", "wmColor"]);
const VALUE_LABEL: Record<string, Record<string, string>> = { mode: { quality: "calidade", bitrate: "bitrate", size: "tamaño máximo" } };
export function describeForm(before: Record<string, any>, after: Record<string, any>, restore?: () => void) {
  const seen = new Set<string>();
  for (const k of Object.keys(after)) {
    if (before[k] === after[k] || !LABEL[k] || seen.has(LABEL[k])) continue;
    const v = after[k];
    if (v === "" || v === null || v === undefined || typeof v === "object") continue;
    seen.add(LABEL[k]);
    const what = tr(LABEL[k]);
    if (typeof v === "boolean") noteAction(v ? "O trasno activou {what}" : "O trasno desactivou {what}", { what }, restore);
    else if (typeof v === "number" || RAW_VALUE.has(k)) noteAction("O trasno cambiou {what} a {value}", { what, value: k === "targetExt" ? String(v).toUpperCase() : String(v) }, restore);
    else if (VALUE_LABEL[k]?.[v]) noteAction("O trasno cambiou {what} a {value}", { what, value: tr(VALUE_LABEL[k][v]) }, restore);
    else noteAction("O trasno cambiou {what}", { what }, restore);
    if (seen.size >= 2) break;
  }
  if (!seen.size && Object.keys(after).some((k) => before[k] !== after[k])) noteAction("O trasno cambiou un axuste", undefined, restore);
}
