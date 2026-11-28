// Niveis de calidade con nome, os mesmos en Converter e Comprimir. O texto en galego é a chave de tradución.

/** Imaxes: a mesma calidade (%) en Converter e Comprimir, para todos os formatos. */
export const IMAGE_LEVELS = [
  { id: "whatsapp", label: "WhatsApp", q: 55, hint: "Máis lixeiro, ideal para enviar rápido" },
  { id: "web", label: "Web", q: 75, hint: "Equilibrio entre peso e nitidez" },
  { id: "arquivo", label: "Arquivo", q: 95, hint: "Case sen perdas, para gardar" },
];

/** Vídeo: os mesmos nomes de uso ca nas imaxes, do máis lixeiro ao de máis calidade, co CRF que levan os botóns (menos é máis calidade). */
export const VIDEO_LEVELS = [
  { label: "WhatsApp", crf: 33, hint: "O máis pequeno; nótase perda de nitidez." },
  { label: "Web", crf: 28, hint: "Bo equilibrio entre peso e calidade" },
  { label: "Arquivo", crf: 23, hint: "Case idéntico ao orixinal." },
];
