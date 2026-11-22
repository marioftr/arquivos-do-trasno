// Rexistro de cambios que se ve na app (Novidades) e que se usa como texto de cada release en GitHub. Cada versión nova engádese arriba.
// Escríbese en galego, castelán e inglés porque se le tal cal, sen pasar polo dicionario.
// Criterio: unha liña por función, só o que é novo para quen usa a app (sen detalles internos), e os arranxos e retoques
// pequenos agrúpanse nunha soa liña («Corrixidos pequenos erros…», «Mellorada a apariencia xeral»).

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
    version: "0.2.1",
    date: "2026-11-22",
    title: {
      gl: "Transcrición, subtítulos traducidos, quitar fondos e descargas por ligazón",
      es: "Transcripción, subtítulos traducidos, quitar fondos y descargas por enlace",
      en: "Transcription, translated subtitles, background removal and downloads by link",
    },
    changes: [
      { kind: "new", gl: "Transcrición de audio e vídeo a subtítulos e texto.", es: "Transcripción de audio y vídeo a subtítulos y texto.", en: "Audio and video transcription to subtitles and text." },
      { kind: "new", gl: "Quitar o fondo das imaxes, con vista previa en directo.", es: "Quitar el fondo de las imágenes, con vista previa en directo.", en: "Background removal for images, with a live preview." },
      { kind: "new", gl: "Traducir subtítulos a galego, español, catalán e inglés.", es: "Traducir subtítulos a gallego, español, catalán e inglés.", en: "Subtitle translation to Galician, Spanish, Catalan and English." },
      { kind: "new", gl: "Descarga de vídeo e audio por ligazón, tamén de listas.", es: "Descarga de vídeo y audio por enlace, también de listas.", en: "Video and audio download by link, including playlists." },
      { kind: "better", gl: "Extensións opcionais organizadas en Axustes, con descarga e retirada de cada unha.", es: "Extensiones opcionales organizadas en Ajustes, con descarga y retirada de cada una.", en: "Optional extensions organised in Settings, with download and removal of each one." },
      { kind: "better", gl: "Novos accesos no menú do trasno: engadir ficheiros, descargar por ligazón e pegar do portapapeis (ligazóns, rutas e imaxes).", es: "Nuevos accesos en el menú del trasno: añadir archivos, descargar por enlace y pegar del portapapeles (enlaces, rutas e imágenes).", en: "New shortcuts in the trasno menu: add files, download by link and paste from the clipboard (links, paths and images)." },
      { kind: "better", gl: "Mellorado o contraste do texto en toda a aplicación.", es: "Mejorado el contraste del texto en toda la aplicación.", en: "Improved text contrast across the app." },
      { kind: "fixed", gl: "Corrixidos pequenos erros da aplicación.", es: "Corregidos pequeños errores de la aplicación.", en: "Fixed minor bugs across the app." },
    ],
  },
  {
    version: "0.2.0",
    date: "2026-11-21",
    title: {
      gl: "PDF máis completo, novas ferramentas e apariencia renovada",
      es: "PDF más completo, nuevas herramientas y apariencia renovada",
      en: "A more complete PDF toolkit, new tools and a refreshed look",
    },
    changes: [
      { kind: "new", gl: "Engadidas novas ferramentas de PDF: formularios, índice, anexos, tapar zonas, limpar, táboas e Markdown.", es: "Añadidas nuevas herramientas de PDF: formularios, índice, anexos, tapar zonas, limpiar, tablas y Markdown.", en: "Added new PDF tools: forms, outline, attachments, redaction, cleaning, tables and Markdown." },
      { kind: "new", gl: "Engadidas novas ferramentas de imaxe: iconas, dividir, xuntar capturas e redimensionar.", es: "Añadidas nuevas herramientas de imagen: iconos, dividir, juntar capturas y redimensionar.", en: "Added new image tools: icons, split, stitch screenshots and resize." },
      { kind: "new", gl: "Engadidas novas ferramentas de vídeo: cortar silencios, subtítulos, restaurar e extraer fotogramas ou audio.", es: "Añadidas nuevas herramientas de vídeo: cortar silencios, subtítulos, restaurar y extraer fotogramas o audio.", en: "Added new video tools: cut silences, subtitles, restore and extract frames or audio." },
      { kind: "new", gl: "Etiquetas de música e igualar volume en audio.", es: "Etiquetas de música e igualar volumen en audio.", en: "Music tags and volume matching for audio." },
      { kind: "new", gl: "Conversión de vídeo a GIF e WebP animado.", es: "Conversión de vídeo a GIF y WebP animado.", en: "Video conversion to GIF and animated WebP." },
      { kind: "better", gl: "Mellorada a apariencia xeral da aplicación.", es: "Mejorada la apariencia general de la aplicación.", en: "Improved the overall look of the app." },
      { kind: "better", gl: "Mellorados os controis de zoom con touchpad e rato.", es: "Mejorados los controles de zoom con touchpad y ratón.", en: "Improved zoom controls for touchpad and mouse." },
      { kind: "better", gl: "Explicacións e nomes das funcións máis claros.", es: "Explicaciones y nombres de las funciones más claros.", en: "Clearer explanations and function names." },
      { kind: "fixed", gl: "Corrixidos pequenos erros da aplicación.", es: "Corregidos pequeños errores de la aplicación.", en: "Fixed minor bugs across the app." },
    ],
  },
  {
    version: "0.1.5",
    date: "2026-11-07",
    title: { gl: "Novo deseño, trasno axente e compatibilidade con Office", es: "Nuevo diseño, trasno agente y compatibilidad con Office", en: "New design, trasno agent and Office support" },
    changes: [
      { kind: "new", gl: "Novo deseño da aplicación.", es: "Nuevo diseño de la aplicación.", en: "New app design." },
      { kind: "new", gl: "O trasno como axente: comentarios, menú radial, historial e cola de procesos.", es: "El trasno como agente: comentarios, menú radial, historial y cola de procesos.", en: "The trasno as an agent: comments, radial menu, history and process queue." },
      { kind: "new", gl: "Compatibilidade con documentos de Office.", es: "Compatibilidad con documentos de Office.", en: "Support for Office documents." },
      { kind: "new", gl: "Novos formatos de imaxe: AVIF, JPEG 2000, JPEG XL e HEIC.", es: "Nuevos formatos de imagen: AVIF, JPEG 2000, JPEG XL y HEIC.", en: "New image formats: AVIF, JPEG 2000, JPEG XL and HEIC." },
      { kind: "new", gl: "Compresión con perdas en PNG e compresión de GIF.", es: "Compresión con pérdidas en PNG y compresión de GIF.", en: "Lossy PNG compression and GIF compression." },
      { kind: "new", gl: "Anotación e conversión de GIF animados.", es: "Anotación y conversión de GIF animados.", en: "Annotation and conversion of animated GIFs." },
      { kind: "better", gl: "Recoñecemento de texto en imaxes máis preciso.", es: "Reconocimiento de texto en imágenes más preciso.", en: "More accurate text recognition in images." },
      { kind: "better", gl: "Visor de PDF máis fluído.", es: "Visor de PDF más fluido.", en: "Smoother PDF viewer." },
      { kind: "better", gl: "Mellorada a apariencia xeral e a adaptación a xanelas estreitas.", es: "Mejorada la apariencia general y la adaptación a ventanas estrechas.", en: "Improved the overall look and the layout on narrow windows." },
      { kind: "fixed", gl: "Corrixidos pequenos erros da aplicación.", es: "Corregidos pequeños errores de la aplicación.", en: "Fixed minor bugs across the app." },
    ],
  },
  {
    version: "0.1.4",
    date: "2026-11-02",
    title: { gl: "Texto das imaxes, anotacións e máis información", es: "Texto de las imágenes, anotaciones y más información", en: "Image text, annotations and richer file info" },
    changes: [
      { kind: "new", gl: "Recoñecemento de texto en imaxes.", es: "Reconocimiento de texto en imágenes.", en: "Text recognition in images." },
      { kind: "new", gl: "Anotación de imaxes e textos en PDF.", es: "Anotación de imágenes y textos en PDF.", en: "Image annotation and text in PDFs." },
      { kind: "new", gl: "Selector de cor con conta-gotas.", es: "Selector de color con cuentagotas.", en: "Colour picker with an eyedropper." },
      { kind: "new", gl: "Información de ficheiro máis completa.", es: "Información de archivo más completa.", en: "Fuller file information." },
      { kind: "better", gl: "Cada ficheiro lembra os seus axustes.", es: "Cada archivo recuerda sus ajustes.", en: "Each file remembers its own settings." },
      { kind: "better", gl: "Mellorados o historial e o visor de vídeo.", es: "Mejorados el historial y el visor de vídeo.", en: "Improved history and video viewer." },
      { kind: "fixed", gl: "Corrixidos pequenos erros da aplicación.", es: "Corregidos pequeños errores de la aplicación.", en: "Fixed minor bugs across the app." },
    ],
  },
  {
    version: "0.1.3",
    date: "2026-11-02",
    title: { gl: "Instalar FFmpeg e MKVToolNix desde a app", es: "Instalar FFmpeg y MKVToolNix desde la app", en: "Install FFmpeg and MKVToolNix from the app" },
    changes: [
      { kind: "new", gl: "Descarga e instalación de FFmpeg e MKVToolNix desde a aplicación, con comprobación de integridade.", es: "Descarga e instalación de FFmpeg y MKVToolNix desde la aplicación, con comprobación de integridad.", en: "Download and install FFmpeg and MKVToolNix from the app, with an integrity check." },
      { kind: "better", gl: "O historial garda os ficheiros de saída dos traballos.", es: "El historial guarda los archivos de salida de los trabajos.", en: "History keeps the output files of each job." },
      { kind: "fixed", gl: "Corrixidos pequenos erros da aplicación.", es: "Corregidos pequeños errores de la aplicación.", en: "Fixed minor bugs across the app." },
    ],
  },
  {
    version: "0.1.2",
    date: "2026-11-01",
    title: { gl: "Converter e comprimir xuntos", es: "Convertir y comprimir juntos", en: "Convert and compress together" },
    changes: [
      { kind: "new", gl: "Converter e comprimir vídeo no mesmo menú.", es: "Convertir y comprimir vídeo en el mismo menú.", en: "Convert and compress video in the same menu." },
      { kind: "new", gl: "Privacidade, termos de uso e licenza en Sobre o trasno.", es: "Privacidad, términos de uso y licencia en Sobre el trasno.", en: "Privacy, terms of use and licence in About the trasno." },
      { kind: "better", gl: "Perfís máis claros: calquera cambio pasa a «Personalizado».", es: "Perfiles más claros: cualquier cambio pasa a «Personalizado».", en: "Clearer profiles: any change switches to “Custom”." },
      { kind: "better", gl: "Desfacer, refacer e restablecer máis accesibles.", es: "Deshacer, rehacer y restablecer más accesibles.", en: "Undo, redo and reset are easier to reach." },
      { kind: "fixed", gl: "Corrixido un erro ao converter AVI de miniDV.", es: "Corregido un error al convertir AVI de miniDV.", en: "Fixed an error when converting miniDV AVI files." },
      { kind: "fixed", gl: "Corrixidos pequenos erros da aplicación.", es: "Corregidos pequeños errores de la aplicación.", en: "Fixed minor bugs across the app." },
    ],
  },
  {
    version: "0.1.1",
    date: "2026-11-01",
    title: { gl: "Paneis e pistas", es: "Paneles y pistas", en: "Panels and tracks" },
    changes: [
      { kind: "new", gl: "Paneis laterais redimensionables e pregables.", es: "Paneles laterales redimensionables y plegables.", en: "Resizable, collapsible side panels." },
      { kind: "new", gl: "Selección de pistas de vídeo, audio e subtítulos no visor.", es: "Selección de pistas de vídeo, audio y subtítulos en el visor.", en: "Video, audio and subtitle track selection in the viewer." },
      { kind: "better", gl: "Panel do visor 3D redeseñado.", es: "Panel del visor 3D rediseñado.", en: "Redesigned 3D viewer panel." },
      { kind: "fixed", gl: "Corrixidos pequenos erros da aplicación.", es: "Corregidos pequeños errores de la aplicación.", en: "Fixed minor bugs across the app." },
    ],
  },
  {
    version: "0.1.0",
    date: "2026-11-01",
    title: { gl: "Primeira versión", es: "Primera versión", en: "First version" },
    changes: [
      { kind: "new", gl: "Converter, comprimir, editar e unir imaxes, audio, vídeo e PDF.", es: "Convertir, comprimir, editar y unir imágenes, audio, vídeo y PDF.", en: "Convert, compress, edit and merge images, audio, video and PDF." },
      { kind: "new", gl: "Perfís de saída para vídeo.", es: "Perfiles de salida para vídeo.", en: "Output profiles for video." },
      { kind: "new", gl: "Menú do clic dereito no Explorador.", es: "Menú de clic derecho en el Explorador.", en: "Explorer right-click menu." },
      { kind: "new", gl: "Xestor de procesos con historial.", es: "Gestor de procesos con historial.", en: "Process manager with history." },
      { kind: "new", gl: "Edición de pistas de MKV.", es: "Edición de pistas de MKV.", en: "MKV track editing." },
      { kind: "new", gl: "Onda de son e efectos de audio.", es: "Onda de sonido y efectos de audio.", en: "Waveform and audio effects." },
      { kind: "new", gl: "Atallos de teclado configurables e buscador de accións.", es: "Atajos de teclado configurables y buscador de acciones.", en: "Configurable keyboard shortcuts and action search." },
      { kind: "better", gl: "Visor de PDF con busca, texto seleccionable e ligazóns.", es: "Visor de PDF con búsqueda, texto seleccionable y enlaces.", en: "PDF viewer with search, selectable text and links." },
      { kind: "better", gl: "Reprodución fluída de vídeos AVI e MKV.", es: "Reproducción fluida de vídeos AVI y MKV.", en: "Smooth playback of AVI and MKV videos." },
      { kind: "better", gl: "Axuda «i» en cada axuste.", es: "Ayuda «i» en cada ajuste.", en: "An “i” help button on every setting." },
    ],
  },
];
