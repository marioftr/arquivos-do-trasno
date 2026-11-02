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
    version: "0.1.4",
    date: "2026-11-02",
    title: { gl: "Texto das imaxes, anotacións e máis información", es: "Texto de las imágenes, anotaciones y más información", en: "Image text, annotations and richer file info" },
    changes: [
      { kind: "new", gl: "Texto da imaxe: o visor le o texto dunha imaxe (Tesseract, dentro da app) para seleccionalo e copialo. Os idiomas descárganse a petición e a lectura segue en marcha ao cambiar de ficheiro.", es: "Texto de la imagen: el visor lee el texto de una imagen (Tesseract, dentro de la app) para seleccionarlo y copiarlo. Los idiomas se descargan a petición y la lectura sigue en marcha al cambiar de archivo.", en: "Image text: the viewer reads the text in an image (Tesseract, inside the app) so it can be selected and copied. Languages are downloaded on request and reading carries on when switching files." },
      { kind: "new", gl: "Anotar imaxes: lapis, marcador, liñas, frechas, rectángulos, elipses e texto, con cor, groso, recheo, xiro, escala e letra, e copia anotada sen tocar o orixinal.", es: "Anotar imágenes: lápiz, marcador, líneas, flechas, rectángulos, elipses y texto, con color, grosor, relleno, giro, escala y letra, y copia anotada sin tocar el original.", en: "Annotate images: pencil, highlighter, lines, arrows, rectangles, ellipses and text, with colour, thickness, fill, rotation, scale and font, saving an annotated copy without touching the original." },
      { kind: "new", gl: "Textos en PDF: varios textos por documento, con letra do sistema, negriña, cursiva, cor, tamaño e xiro, que se colocan e se arrastran sobre a páxina.", es: "Textos en PDF: varios textos por documento, con letra del sistema, negrita, cursiva, color, tamaño y giro, que se colocan y se arrastran sobre la página.", en: "PDF text: several texts per document, with system fonts, bold, italic, colour, size and rotation, placed and dragged on the page." },
      { kind: "new", gl: "Selector de cor con conta-gotas que recolle a cor de calquera punto da pantalla.", es: "Selector de color con cuentagotas que recoge el color de cualquier punto de la pantalla.", en: "Colour picker with an eyedropper that picks the colour of any point on screen." },
      { kind: "new", gl: "Información do ficheiro máis completa: resumo breve, vista avanzada con datos técnicos, capítulos pregables e datos en bruto.", es: "Información del archivo más completa: resumen breve, vista avanzada con datos técnicos, capítulos plegables y datos en bruto.", en: "Fuller file information: a short summary, an advanced view with technical data, collapsible chapters and raw data." },
      { kind: "new", gl: "A edición de imaxe xira e espella o que se ve, e o recorte traballa sobre a imaxe tal como aparece.", es: "La edición de imagen gira y refleja lo que se ve, y el recorte trabaja sobre la imagen tal como aparece.", en: "Image editing rotates and flips what is shown, and cropping works on the image as it appears." },
      { kind: "better", gl: "Cada ficheiro lembra os seus axustes (relación de aspecto, perfil, recorte...) ao cambiar de ficheiro e volver; os ficheiros novos empezan cos valores por defecto.", es: "Cada archivo recuerda sus ajustes (relación de aspecto, perfil, recorte...) al cambiar de archivo y volver; los archivos nuevos empiezan con los valores por defecto.", en: "Each file remembers its own settings (aspect ratio, profile, crop...) when switching away and back; new files start from the defaults." },
      { kind: "better", gl: "O historial rexistra tamén as copias anotadas, e as súas iconas levan ao ficheiro realmente gardado.", es: "El historial registra también las copias anotadas, y sus iconos llevan al archivo realmente guardado.", en: "History also records annotated copies, and its icons open the file that was actually saved." },
      { kind: "better", gl: "O vídeo lembra a posición ao cambiar de ficheiro, a pantalla completa ten barra e pistas propias e a vista previa dun fotograma funciona con calquera vídeo.", es: "El vídeo recuerda la posición al cambiar de archivo, la pantalla completa tiene barra y pistas propias y la vista previa de un fotograma funciona con cualquier vídeo.", en: "Video remembers its position when switching files, full screen has its own bar and tracks panel, and frame preview works with any video." },
      { kind: "better", gl: "Os avisos da app teñen máis personalidade e o visor de imaxe adáptase a xanelas estreitas.", es: "Los avisos de la app tienen más personalidad y el visor de imagen se adapta a ventanas estrechas.", en: "The app's notices have more personality and the image viewer adapts to narrow windows." },
      { kind: "fixed", gl: "Reinstalar FFmpeg ou MKVToolNix xa non deixa sen programa se a descarga falla.", es: "Reinstalar FFmpeg o MKVToolNix ya no deja sin programa si la descarga falla.", en: "Reinstalling FFmpeg or MKVToolNix no longer leaves the tool missing if the download fails." },
      { kind: "fixed", gl: "O recorte respecta a orientación EXIF das fotos de móbil e «Amosar no cartafol» selecciona o ficheiro correcto.", es: "El recorte respeta la orientación EXIF de las fotos de móvil y «Mostrar en la carpeta» selecciona el archivo correcto.", en: "Cropping respects the EXIF orientation of phone photos, and “Show in folder” selects the right file." },
      { kind: "fixed", gl: "Suprimir xa non pecha o ficheiro aberto (agora é Ctrl+W) e o visor de PDF xa non parpadea ao abrir.", es: "Suprimir ya no cierra el archivo abierto (ahora es Ctrl+W) y el visor de PDF ya no parpadea al abrir.", en: "Delete no longer closes the open file (it is now Ctrl+W) and the PDF viewer no longer flickers when opening." },
    ],
  },
  {
    version: "0.1.3",
    date: "2026-11-02",
    title: { gl: "Instalar FFmpeg e MKVToolNix desde a app", es: "Instalar FFmpeg y MKVToolNix desde la app", en: "Install FFmpeg and MKVToolNix from the app" },
    changes: [
      { kind: "new", gl: "Se unha acción precisa FFmpeg ou MKVToolNix e non están, a app avisa e permite descargalos e instalalos directamente, con barra de progreso, ou ir á páxina oficial.", es: "Si una acción necesita FFmpeg o MKVToolNix y no están, la app avisa y permite descargarlos e instalarlos directamente, con barra de progreso, o ir a la página oficial.", en: "If an action needs FFmpeg or MKVToolNix and they are missing, the app says so and lets you download and install them directly, with a progress bar, or go to the official page." },
      { kind: "new", gl: "Cada descarga verifícase coa suma SHA-256 publicada pola fonte antes de instalarse; instálase sen permisos de administrador e pódese quitar desde Axustes → Extensións.", es: "Cada descarga se verifica con la suma SHA-256 publicada por la fuente antes de instalarse; se instala sin permisos de administrador y se puede quitar desde Ajustes → Extensiones.", en: "Each download is verified against the SHA-256 checksum published by the source before installing; it needs no administrator rights and can be removed from Settings → Extensions." },
      { kind: "new", gl: "As xanelas do clic dereito tamén ofrecen a descarga e continúan o traballo cando remata.", es: "Las ventanas del clic derecho también ofrecen la descarga y continúan el trabajo al terminar.", en: "The right-click windows also offer the download and continue the job when it finishes." },
      { kind: "better", gl: "O historial garda os ficheiros de saída dos traballos da aplicación, con botóns para abrilos na app ou amosalos no cartafol.", es: "El historial guarda los archivos de salida de los trabajos de la aplicación, con botones para abrirlos en la app o mostrarlos en la carpeta.", en: "History keeps the output files of the app's jobs, with buttons to open them in the app or show them in the folder." },
      { kind: "better", gl: "O diálogo principal adáptase a xanelas pequenas.", es: "El diálogo principal se adapta a ventanas pequeñas.", en: "The main dialog adapts to small windows." },
      { kind: "fixed", gl: "O historial amosaba «antes eran 0 B» como tamaño orixinal.", es: "El historial mostraba «antes eran 0 B» como tamaño original.", en: "History showed “was 0 B” as the original size." },
      { kind: "fixed", gl: "O último botón das páxinas legais quedaba cortado por abaixo.", es: "El último botón de las páginas legales quedaba cortado por abajo.", en: "The last button on the legal pages was cut off at the bottom." },
    ],
  },
  {
    version: "0.1.2",
    date: "2026-11-01",
    title: { gl: "Converter e comprimir xuntos", es: "Convertir y comprimir juntos", en: "Convert and compress together" },
    changes: [
      { kind: "new", gl: "Converter e comprimir un vídeo está agora no mesmo menú: «A MP4 con perfís» ou «A outro formato».", es: "Convertir y comprimir un vídeo está ahora en el mismo menú: «A MP4 con perfiles» o «A otro formato».", en: "Converting and compressing a video now live in the same menu: “To MP4 with profiles” or “To another format”." },
      { kind: "new", gl: "Sobre o trasno ten agora a política de privacidade, os termos de uso, a licenza e a política de asinado de código.", es: "Sobre el trasno tiene ahora la política de privacidad, los términos de uso, la licencia y la política de firma de código.", en: "About the trasno now has the privacy policy, terms of use, licence and code signing policy." },
      { kind: "new", gl: "O desinstalador quita as entradas do menú do clic dereito e pregunta se tamén queres borrar os datos da app.", es: "El desinstalador quita las entradas del menú de clic derecho y pregunta si también quieres borrar los datos de la app.", en: "The uninstaller removes the right-click menu entries and asks whether to delete the app data too." },
      { kind: "better", gl: "Se tocas calquera axuste dun perfil, o perfil pasa a «Personalizado». Gardar un perfil só sae aí, enriba do botón de converter.", es: "Si tocas cualquier ajuste de un perfil, el perfil pasa a «Personalizado». Guardar un perfil solo sale ahí, encima del botón de convertir.", en: "If you change any profile setting, the profile switches to “Custom”. Saving a profile only appears there, above the convert button." },
      { kind: "better", gl: "Desfacer, refacer e restablecer pasaron enriba dos modos de cada ferramenta.", es: "Deshacer, rehacer y restablecer pasaron encima de los modos de cada herramienta.", en: "Undo, redo and reset moved above each tool's modes." },
      { kind: "better", gl: "O volume lémbrase ao cambiar de ficheiro, e se a vista previa falla hai un botón para tentar de novo.", es: "El volumen se recuerda al cambiar de archivo, y si la vista previa falla hay un botón para intentarlo de nuevo.", en: "Volume is remembered when switching files, and if a preview fails there is a button to try again." },
      { kind: "fixed", gl: "Converter un AVI de miniDV daba erro cando o nome de saída non levaba extensión.", es: "Convertir un AVI de miniDV daba error cuando el nombre de salida no llevaba extensión.", en: "Converting a miniDV AVI failed when the output name had no extension." },
      { kind: "fixed", gl: "A estimación do tamaño volve calcularse ao cambiar de perfil e hai un botón «Estimar de novo».", es: "La estimación del tamaño vuelve a calcularse al cambiar de perfil y hay un botón «Estimar de nuevo».", en: "The size estimate is recalculated when the profile changes, and there is an “Estimate again” button." },
      { kind: "fixed", gl: "O aviso de cancelado xa non se mostra en verde e corrixiuse o espazado dos separadores «·».", es: "El aviso de cancelado ya no se muestra en verde y se ha corregido el espaciado de los separadores «·».", en: "The cancelled notice is no longer shown in green and the spacing of “·” separators has been corrected." },
    ],
  },
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
