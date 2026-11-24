# Ferramentas da aplicación

Lista de todo o que fai Arquivos do Trasno, por tipo de ficheiro (cada pestana do panel da dereita). Mantense ao día con cada versión: se engades ou cambias unha ferramenta, actualiza esta páxina (e `src/lib/components/Tools.svelte`, onde se definen as pestanas).

Un resultado sempre é un ficheiro novo: o orixinal non se modifica. As extensións marcadas con ⬇ non van incluídas: descárganse só se se piden (Axustes → Extensións) e traballan no propio equipo.

## Imaxes (PNG, JPG, WebP, GIF, BMP, ICO, TIFF, AVIF, HEIC…)

| Pestana | Que fai |
|---|---|
| Converter | Cambiar de formato; calidade, sen perdas, ancho ou lado máximo, fondo para a transparencia, metadatos e GPS. |
| Comprimir | Reducir o peso por nivel de calidade ou ata un tamaño máximo; estimación do tamaño. |
| Editar | Recortar (proporcións), xirar e espellar, brillo, contraste, desenfoque, branco e negro e inverter cores. |
| Restaurar | Sen IA xerativa: luz, cor, ruído e nitidez. Perfís (Automático, Máis luz, Documento, Branco e negro, Menos ruído, Máis nitidez, Personalizado) e axustes de niveis automáticos (estiran a luz sen converter en día unha escena de noite), equilibrio de cor, brillo, contraste, sombras, saturación, ruído, nitidez, desenfoque e inverter cores. Os perfís son botóns con icona e fixan tamén o tipo (cor, documento, branco e negro). O visor amosa o resultado en directo; «Ver orixinal» (premido) compara. Imaxes soas ou en lote. |
| Quitar fondo ⬇ | Recoñece o obxecto principal e quita o resto. «Previsualizar» (botón pequeno sobre o verde) analiza a imaxe e o visor amosa o resultado en directo, co fondo elixido (transparente, branco, negro, gris ou personalizado) e axustes de recorte, suavidade e detalle das beiras. PNG, WebP ou JPG. |
| Redimensionar | Por porcentaxe ou píxeles (tamaños con nome), manter proporción, recortar polo centro ou estirar; imaxes soas ou en lote. |
| Dividir | Cortar en partes iguais (columnas × filas) dentro dunha zona; plantillas de carrusel e cuadrícula. |
| Anotar | Lapis, marcador, liñas, frechas, figuras e texto, con cor, groso e recheo; tamén en GIF animados. |
| Iconas | Paquete de iconas (ICO, ICNS, web e PNG) desde unha imaxe. |
| Recoñecer texto (OCR) | Ler o texto dunha imaxe (Tesseract; idiomas descargables) para seleccionalo e copialo. |
| Información | Datos do ficheiro, metadatos e EXIF. |

Varias imaxes xuntas: Converter, Comprimir, Restaurar, Redimensionar, Unir (nun PDF) e Xuntar (capturas de desprazamento con cabeceira e pé fixos, solapamento e «Quitar tramos repetidos»).

## Vídeo

| Pestana | Que fai |
|---|---|
| Converter | A MP4 con perfís de saída (Universal, Mini DV, YouTube, móbil e os propios) ou a outro formato; GIF e WebP animado. |
| Editar | Cortar, velocidade, esvaecer, volume. |
| Son | Axustes (volume, igualar volume, quitar ruído, graves e agudos, eco, dar volta ao son, quitar son) e Silencios (cortar pausas). |
| Restaurar | Estabilizar, quitar ruído, nitidez, desentrelazar, parpadeo e cor; «Ver orixinal» para comparar. |
| Transcribir ⬇ | Voz a texto con whisper.cpp: subtítulos SRT ou VTT, ou texto; idioma, tres calidades e traducir (de momento só ao inglés: Whisper non traduce a outros idiomas). |
| Subtítulos | Incrustar, queimar na imaxe ou extraer a ficheiro. |
| Pistas | (Só MKV, WebM) Quitar, reordenar e renomear pistas con MKVToolNix. |
| Extraer | Un fotograma (por instante ou o do visor) como imaxe, ou o audio. |
| Información | Datos técnicos, capítulos e datos en bruto. |

Varios vídeos xuntos: Converter, Unir e Pistas.

## Audio

Converter, Editar (cortar, velocidade, esvaecer, volume), Son (axustes e silencios), **Transcribir ⬇**, Etiquetas (música: título, artista, álbum, portada…), Pistas (MKA, MKV) e Información. Varios audios xuntos: Converter, Unir e Pistas.

## PDF

| Pestana | Modos |
|---|---|
| Converter | Formatos (PNG, JPG, TXT, Markdown e Office se hai) e Táboas (CSV). |
| Páxinas | Ordenar, Extraer e Dividir. |
| Xirar e recortar | Xirar páxinas e Recortar márxenes. |
| Completar | Formulario (encher e aplanar), Índice (marcadores) e Anexos. |
| Marcas | Marca de auga, Números de páxina, Texto e Firma / imaxe. |
| Comprimir e reparar | Comprimir (nivel e tamaño das imaxes) e Reparar. |
| Seguridade | Protexer, Desbloquear, Limpar (metadatos, scripts, anexos, ligazóns, comentarios) e Tapar (zonas ou datos buscados: correos, teléfonos, IBAN e DNI; elimínase o contido de verdade). |
| Recoñecer texto (OCR) | Facer pesquisable un PDF escaneado. |
| Información | Datos do ficheiro. |

Varios PDF xuntos: Unir.

## Outros

- Subtítulos (SRT, VTT, ASS…): Converter, Resincronizar (desprazar, dous puntos, cambio de cadencia), **Traducir** (galego, español, catalán e inglés con Opus-MT en WebAssembly; baixa uns 120 MB por sentido a primeira vez de Hugging Face e despois traballa sen conexión; o inglés é a ponte; tradución automática, a revisar) e Información.
- Texto e documentos de Office: Converter (Office con Microsoft Office ou LibreOffice) e Información.
- Modelos 3D (glTF, FBX, OBJ…): Converter, Escena (visor, apariencia, luz, materiais, texturas) e Información.

## Fóra das pestanas

- **Descargar por ligazón** ⬇: vídeo ou audio dunha ligazón con yt-dlp. Comproba a ligazón co botón Buscar (ou soa, se vén do botón de pegar do trasno) e amosa miniatura, plataforma, título, duración, data (día, mes e ano) e visualizacións; unha lista vai debaixo dos detalles, deixa elixir formato (MP4 ou MKV) e calidade («Orixinal (1080p)» ou unha altura) en vídeo, e formato e calidade en audio. Os erros coñecidos (DRM, vídeo privado, sesión, país, retirado) saen en galego; un vídeo con DRM non se pode descargar, e di como sairá o ficheiro. Se a ligazón é unha lista, amósaa enteira e descarga os vídeos marcados (se un falla, seguen os demais). O sitio onde gardar pregúntase ao premer o botón verde, como no resto da app, e a descarga segue en segundo plano nos procesos. Está no menú do trasno e no buscador (Ctrl+K); o resultado entra na lista. Precisa tamén FFmpeg.
- **Torrents**: pestana aparte («Torrents») do diálogo de descargas, con selector Ligazóns/Torrents arriba. Acepta ligazóns magnet, ligazóns a un `.torrent` ou un ficheiro `.torrent` (botón «Elixir un ficheiro .torrent», ou pegar do portapapeis). Usa un cliente BitTorrent integrado (librqbit, Apache-2.0): non precisa yt-dlp, FFmpeg nin programas externos. «Buscar» lé o nome e os ficheiros (cunha ligazón magnet busca xente que o teña: precisa internet) e deixa marcar cales baixar. Baixa a un cartafol temporal dentro do destino e, ao completarse, móveo co nome do torrent sen pisar nada; ao rematar déixase de compartir, e non se abre ningún porto de entrada. Cancelar borra o que levase. A descarga segue nos procesos.
- Menú do trasno (círculo interior): Engadir ficheiros, Descargar por ligazón, Procesos e Axustes; círculo exterior: Descargar torrent, Comentario e Buscar. Pegar do portapapeis vai con Ctrl+V (e no buscador): (unha ligazón abre Descargar por ligazón xa comprobándoa; rutas de ficheiros engádense á lista; unha imaxe copiada gárdase como PNG en Imaxes › Arquivos do Trasno e entra na lista), Comentario e Buscar. Mentres o trasno está na xanela, os seus avisos agardan e saen cando volve. O trasno voa da barra á xanela ao abrir Descargar por ligazón, volve ao pechala e vai á barra co billete ao empezar unha descarga.
- **Titorial**: na primeira execución sae unha benvida co trasno en grande; se a persoa quere, seguen nove pasos sobre unha copia da xanela real (lista de ficheiros, visor, columna de ferramentas, botón verde, barra de progreso, menú do trasno e buscador). Antes da xanela de extensións. Repítese desde Axustes → Xeral ou no buscador (Ctrl+K).
- **Actualizacións automáticas**: ao abrir, a app busca unha versión nova en GitHub, descárgaa e instálaa soa cando non hai ningún proceso en marcha (o instalador reinicia a app). Axustes → Xeral e Novidades → Actualizacións teñen o interruptor «Actualizar automaticamente» (acendido por defecto): apagado, non busca nada. Non hai botón manual.
- Buscador de accións (Ctrl+K), atallos de teclado configurables, procesos e historial, Novidades, tema claro e escuro.
- Menú do clic dereito no Explorador de Windows: converter, comprimir, redimensionar e xirar sen abrir a app. Ponse ao día soo con cada versión e cada cambio de idioma (só se as entradas son desta copia da app); Axustes → Xeral permite instalalo, actualizalo ou quitalo.
- Visores propios: imaxe, PDF, audio e vídeo (con pistas), 3D, texto e Office.

## Extensións opcionais

| Extensión | Para que | Tamaño |
|---|---|---|
| FFmpeg | Todo o audio e vídeo, extraer fotogramas, formatos de imaxe raros. | 115 MB |
| MKVToolNix | Pistas de MKV. | 31 MB |
| whisper.cpp + modelo de voz | Transcribir (Rápida 57 MB, Equilibrada 181 MB, Precisa 514 MB; modelos Whisper cuantizados a 5 bits). | 9 MB + modelo |
| IS-Net (modelo de imaxe) | Quitar fondo. | 170 MB |
| yt-dlp | Descargar por ligazón. | 24 MB |
| Paquetes de idioma de Tesseract | OCR. | 2–15 MB cada un |

Aínda sen facer (ver a folla de ruta): ampliar imaxes con IA (distinto de Redimensionar, que só remostrea), texto a voz e detección de duplicados.

## Versión web (https://arquivos-do-trasno.vercel.app)

Páxinas: **Inicio** (presenta a web, co trasno animado que pestanexa), **Ferramentas** (todas, con buscador e filtro por tipo), **Descarga** (o último instalador de Windows directo desde GitHub Releases, máis ligazóns ao código, ás versións e aos erros) e **Privacidade**. Os ficheiros procésanse no navegador; a páxina de descarga pregunta a GitHub pola última versión só ao abrila.

| Tipo | Ferramentas |
|---|---|
| Imaxe | Converter, Redimensionar, Comprimir, Recortar, Xirar, Restaurar, Quitar o fondo, Recoñecer texto, Dividir, Xuntar, Iconas e Información (EXIF) |
| PDF | Unir, Dividir, Ordenar, Xirar, Recortar márxenes, Marca de auga, Numerar, Comprimir, Imaxes a PDF, PDF a imaxes, PDF a texto, Limpar (metadatos e formularios) e Información |
| Audio e vídeo | Converter audio, Comprimir vídeo, Cortar, Velocidade e volume, Unir audios, Vídeo a GIF, Fotograma e Transcribir |
| Subtítulos | Converter, Resincronizar e Traducir |

Fican só na app: anotar, editar vídeo con vista previa, pistas MKV, descargas por ligazón e torrents, tapar datos nun PDF, OCR de PDF, cifrar PDF, visor 3D, Office e o menú do Explorador.
