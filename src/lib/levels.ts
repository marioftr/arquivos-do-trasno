// Niveis de calidade con nome, os mesmos en Converter e Comprimir. O texto en galego é a chave de tradución.

/** Imaxes: `jpg` vale para JPG, WebP, AVIF… e `png` é a calidade equivalente en PNG (100 = sen perder un píxel). */
export const IMAGE_LEVELS = [
  { id: "whatsapp", label: "WhatsApp", jpg: 55, png: 60, hint: "Máis lixeiro, ideal para enviar rápido" },
  { id: "web", label: "Web", jpg: 75, png: 80, hint: "Equilibrio entre peso e nitidez" },
  { id: "arquivo", label: "Arquivo", jpg: 95, png: 100, hint: "Case sen perdas, para gardar" },
];

/** Vídeo: o CRF que levan os botóns (menos é máis calidade). */
export const VIDEO_LEVELS = [
  { label: "Alta", crf: 23, hint: "Case idéntico ao orixinal." },
  { label: "Equilibrada", crf: 28, hint: "Bo equilibrio entre peso e calidade" },
  { label: "Forte", crf: 33, hint: "O máis pequeno; nótase perda de nitidez." },
];
