// Rexistro de cambios que se ve na app (Novidades). Cada versión nova engádese arriba.
// Escríbese en galego, castelán e inglés porque se le tal cal, sen pasar polo dicionario.

export type Kind = "new" | "better" | "fixed";
export interface Change {
  kind: Kind;
  gl: string;
  es: string;
  en: string;
}
export interface Release {
  version: string;
  date: string; // AAAA-MM-DD
  title: { gl: string; es: string; en: string };
  changes: Change[];
}

export const CHANGELOG: Release[] = [
  {
    version: "0.1.1",
    date: "2026-11-01",
    title: { gl: "Paneis e pistas", es: "Paneles y pistas", en: "Panels and tracks" },
    changes: [
      { kind: "new", gl: "Os dous paneis laterais pódense axustar de ancho e pregar para ver o contido máis grande; un botón na divisoria volve despregalos.", es: "Los dos paneles laterales se pueden ajustar de ancho y plegar para ver el contenido más grande; un botón en la divisoria vuelve a desplegarlos.", en: "Both side panels can be resized and folded away to see the content bigger; a button on the divider unfolds them again." },
      { kind: "new", gl: "Nos vídeos podes escoller que pista de vídeo, audio ou subtítulos reproducir, nun panel novo xunto ao visor.", es: "En los vídeos puedes elegir qué pista de vídeo, audio o subtítulos reproducir, en un panel nuevo junto al visor.", en: "In videos you can choose which video, audio or subtitle track to play, in a new panel beside the viewer." },
      { kind: "better", gl: "O panel do visor 3D está redeseñado: acoplado ao lado (sen tapar o modelo), con seccións pregables e ancho axustable.", es: "El panel del visor 3D está rediseñado: acoplado al lado (sin tapar el modelo), con secciones plegables y ancho ajustable.", en: "The 3D viewer panel is redesigned: docked beside the model (not covering it), with collapsible sections and adjustable width." },
      { kind: "fixed", gl: "Ao redimensionar paneis, o visor 3D e o PDF xa non parpadean.", es: "Al redimensionar paneles, el visor 3D y el PDF ya no parpadean.", en: "Resizing panels no longer makes the 3D viewer and PDF flicker." },
    ],
  },
  {
    version: "0.1.0",
    date: "2026-11-01",
    title: { gl: "Primeira versión", es: "Primera versión", en: "First version" },
    changes: [
      { kind: "new", gl: "Converter, comprimir, editar e unir imaxes, audio, vídeo e PDF, con visor para cada tipo e as ferramentas ao carón.", es: "Convertir, comprimir, editar y unir imágenes, audio, vídeo y PDF, con visor para cada tipo y las herramientas al lado.", en: "Convert, compress, edit and merge images, audio, video and PDF, with a viewer for each type and the tools beside it." },
      { kind: "new", gl: "Perfís de saída para vídeo (universal, Mini DV a MP4, YouTube, móbil) e perfís propios que se poden gardar.", es: "Perfiles de salida para vídeo (universal, Mini DV a MP4, YouTube, móvil) y perfiles propios que se pueden guardar.", en: "Video output profiles (universal, Mini DV to MP4, YouTube, mobile) and your own saved profiles." },
      { kind: "new", gl: "Menú do clic dereito no Explorador para converter e comprimir sen abrir a app, con xanela de progreso e cancelación.", es: "Menú de clic derecho en el Explorador para convertir y comprimir sin abrir la app, con ventana de progreso y cancelación.", en: "Explorer right-click menu to convert and compress without opening the app, with a progress window and cancel." },
      { kind: "new", gl: "Xestor de procesos con historial, tamén para o que se lanza desde o clic dereito.", es: "Gestor de procesos con historial, también para lo que se lanza desde el clic derecho.", en: "Process manager with history, including jobs started from the right-click menu." },
      { kind: "new", gl: "Pistas de MKV (con MKVToolNix): quitar, reordenar, renomear e mesturar pistas de varios ficheiros.", es: "Pistas de MKV (con MKVToolNix): quitar, reordenar, renombrar y mezclar pistas de varios archivos.", en: "MKV tracks (with MKVToolNix): remove, reorder, rename and mix tracks from several files." },
      { kind: "new", gl: "Onda de son, efectos de audio e cortar sobre o visor.", es: "Onda de sonido, efectos de audio y recorte sobre el visor.", en: "Waveform, audio effects and trimming on the viewer." },
      { kind: "new", gl: "Atallos de teclado configurables (ata dous por acción), buscador de accións con Ctrl+K e tema claro/escuro.", es: "Atajos de teclado configurables (hasta dos por acción), buscador de acciones con Ctrl+K y tema claro/oscuro.", en: "Configurable keyboard shortcuts (up to two per action), Ctrl+K action search and light/dark theme." },
      { kind: "better", gl: "PDF: texto seleccionable, ligazóns pulsables, busca, axuste ao ancho e debuxo preguiceiro das páxinas con caché.", es: "PDF: texto seleccionable, enlaces pulsables, búsqueda, ajuste al ancho y dibujo perezoso de las páginas con caché.", en: "PDF: selectable text, clickable links, search, fit-to-width and lazy page rendering with cache." },
      { kind: "better", gl: "Vídeos AVI e MKV: vense á resolución orixinal, empezan ao instante e pódese saltar a calquera punto sen esperar.", es: "Vídeos AVI y MKV: se ven a la resolución original, empiezan al instante y se puede saltar a cualquier punto sin esperar.", en: "AVI and MKV videos: shown at original resolution, start instantly and you can jump anywhere without waiting." },
      { kind: "better", gl: "Un icono «i» ao carón de cada axuste explica o que fai.", es: "Un icono «i» junto a cada ajuste explica lo que hace.", en: "An “i” icon next to every setting explains what it does." },
    ],
  },
];
