# Versión web

Un PWA (páxina que se instala desde o navegador) co que se usan no móbil as ferramentas máis comúns. Vive en `web/`, comparte co programa de
escritorio os textos (`src/lib/locales`), as cores (`src/styles/tokens.css`), as iconas e a letra, e non leva servidor: os ficheiros
procésanse no propio navegador e non se envían a ningunha parte. Vai en Vercel como ficheiros estáticos (`vercel.json`).

## Que hai

| Grupo | Ferramentas |
| --- | --- |
| Imaxe | Converter (JPG, PNG, WebP), Redimensionar, Comprimir, Xirar e voltear, **Recoñecer texto** (OCR en galego, castelán, inglés e catalán) e **Quitar o fondo** (modelo rápido de 5 MB ou mellor de 46 MB). Quitan os metadatos (GPS). |
| PDF | Unir, Dividir (extraer, por intervalos, cada páxina), Xirar, Imaxes a PDF, **PDF a imaxes** e **Comprimir** (volve gardar as páxinas como imaxe: perde o texto seleccionable). |
| Audio e vídeo | Converter audio (MP3, M4A, Opus, WAV; tamén saca o son dun vídeo), Cortar sen recodificar, Vídeo a GIF, **Transcribir** (Whisper tiny ou base, a SRT, VTT ou TXT, e opcionalmente traducido) e Traducir subtítulos (SRT ou VTT; galego, castelán, catalán e inglés). |

Non hai (nin pode haber sen servidor): Descargar por ligazón.

## De onde vén cada cousa

- O OCR (Tesseract) e o ONNX Runtime sérveos a propia web (`scripts/copy-web-assets.mjs` cópiaos de `node_modules` a `web/public`, que non vai a git).
- FFmpeg (ffmpeg.wasm, GPL-2.0+) **non** vai na web: descárgase de jsDelivr a primeira vez que se usa unha ferramenta de audio ou vídeo (uns 30 MB) e o navegador gárdao.
- Os modelos (Whisper e Opus-MT con transformers.js; U²-Net e IS-Net con ONNX Runtime; todos Apache-2.0) descárganse de Hugging Face a primeira vez e o navegador gárdaos; despois traballan sen conexión. Whisper tiny/base: 40/80 MB; cada sentido da tradución, uns 120 MB; ten límites de calidade (a tradución con galego é a máis frouxa).
- O procesado é sempre no aparello. Os únicos pedidos de rede son eses motores e modelos; a CSP de `vercel.json` só deixa jsDelivr e Hugging Face.

## Como está feito

- Interface primeiro para o móbil: unha columna, botón verde pegado abaixo, controis de 44 px ou máis, letra de 16 px, zonas seguras do iPhone,
  «i» que se abre ao tocar, tema claro e escuro do sistema, galego, castelán e inglés. En ordenador pasa a dúas columnas.
- Cada ferramenta é un obxecto en `web/src/lib/tools.ts` (opcións, botón e función): a pantalla `Workbench.svelte` constrúese soa.
- Imaxes con canvas (`image.ts`); PDF con `pdf-lib`, que só se descarga ao abrir unha ferramenta de PDF. O paquete inicial pesa uns 40 KB comprimido.
- Rutas por `#/`: o botón de volver do móbil funciona. `sw.js` garda a páxina para usala sen conexión.
- No móbil, Gardar abre a folla de compartir (Fotos, Ficheiros…); no ordenador, descarga.
- Límites: o ficheiro entra na memoria (máximo 250 MB en audio e vídeo); Safari non escribe WebP e limita o canvas a uns 16 Mpx (as fotos máis grandes redúcense e dise). Os PDF con contrasinal non se abren.

## Probas e publicación

`npm run test:web` (ver `docs/probas.md`). `npm run build:web` xera `dist-web/`. En Vercel basta importar o repositorio: `vercel.json` xa leva a orde
de compilación, a carpeta de saída e as cabeceiras de seguridade. Nada se publica sen pedilo.
