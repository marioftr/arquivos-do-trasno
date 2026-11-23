/** O catálogo completo da web: as ferramentas de `tools.ts` e `tools-extra.ts`, en orde, e os grupos en que se amosan. */
import { tr } from "@app/lib/i18n";
import { buildTools as core, type Tool } from "./tools";
import { buildExtraTools } from "./tools-extra";

export type GroupId = Tool["group"];
export interface Group {
  id: GroupId;
  label: string;
  desc: string;
  icon: string;
  color: string;
}

/** Orde de cada ferramenta dentro do seu grupo (as máis usadas primeiro). */
const ORDER = [
  "converter", "redimensionar", "comprimir", "recortar", "xirar", "restaurar", "quitar-fondo", "ocr", "dividir-imaxe", "xuntar-imaxes", "iconas", "info-imaxe",
  "unir-pdf", "dividir-pdf", "ordenar-pdf", "xirar-pdf", "recortar-pdf", "marca-pdf", "numeros-pdf", "comprimir-pdf", "imaxes-a-pdf", "pdf-imaxes", "pdf-texto", "limpar-pdf", "info-pdf",
  "audio", "video", "cortar", "axustar", "unir-audios", "gif", "fotograma", "transcribir",
  "converter-subs", "resincronizar", "traducir",
];

export const buildTools = (): Tool[] => {
  const all = [...core(), ...buildExtraTools()];
  return ORDER.map((id) => all.find((t) => t.id === id)).filter((t): t is Tool => !!t).concat(all.filter((t) => !ORDER.includes(t.id)));
};

export const buildGroups = (): Group[] => [
  { id: "imaxe", label: tr("Imaxe"), desc: tr("Converter, comprimir, recortar, restaurar, quitar o fondo e ler o texto."), icon: "image", color: "var(--accent-imaxe)" },
  { id: "pdf", label: "PDF", desc: tr("Unir, dividir, ordenar, recortar, marcar e pasar a imaxes ou texto."), icon: "document", color: "var(--accent-pdf)" },
  { id: "audio", label: tr("Audio e vídeo"), desc: tr("Converter, cortar, axustar o son, comprimir vídeo e transcribir voz."), icon: "audio", color: "var(--accent-audio)" },
  { id: "subs", label: tr("Subtítulos"), desc: tr("Converter, resincronizar e traducir subtítulos."), icon: "type", color: "var(--accent-video)" },
];
