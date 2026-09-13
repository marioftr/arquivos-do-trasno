# Folla de ruta de Trasno

Proxecto longo, dividido en fases. Cada fase entrega algo funcional e
probado antes de pasar á seguinte; non se avanza construíndo sobre
arquitectura sen validar.

## Fase 0 — Base (feita nesta sesión)

- Workspace Rust (`trasno-engine` + `src-tauri`) e frontend Svelte+TS.
- Motor de conversión de imaxes (Rust puro, sen dependencias externas).
- Motor de operacións PDF básicas (unir, extraer/dividir, rotar).
- Wrapper de ffmpeg/ffprobe para audio e vídeo (require ffmpeg instalado).
- Garantías de integridade (escritura atómica, verificación de resultado).
- UI: pantalla de inicio (4 estudos + conversor) e Conversor universal
  funcional de punta a punta.
- Tests automatizados do motor (imaxe + PDF).

## Fase 1 — Conversor universal completo

- Empaquetar `ffmpeg`/`ffprobe` como *sidecars* de Tauri por plataforma
  (Windows/macOS/Linux), para que o usuario non teña que instalalos á
  parte. Require decidir entre build LGPL (sen algúns códecs con patente,
  ex. libx264 con certas licenzas) ou distribuír por separado un build
  GPL opcional.
- Conversión de documentos: DOCX/ODT/TXT → PDF (via unha librería ou un
  motor de renderizado embebido), e PDF → imaxes (páxina a páxina).
- Conversión básica de modelos 3D (OBJ ↔ STL ↔ PLY primeiro, por ser
  formatos de malla simples; glTF/FBX máis adiante).
- Cola de conversión en paralelo con límite de tarefas concorrentes, barra
  de progreso real (actualmente é binario: pendente/feito).
- Perfís de conversión gardados (ex. "WhatsApp", "Web", "Arquivo").

## Fase 2 — Trasno Imaxe

- Lenzo con capas (mover, opacidade, modos de fusión, bloquear/ocultar).
- Ferramentas: selección, recorte, pincel, goma, texto, formas.
- Axustes non destrutivos (brillo/contraste/saturación/curvas).
- Exportar mantendo capas nun formato propio do proxecto + exportar
  "achatado" en calquera formato do conversor.

## Fase 3 — Trasno Audio

- Liña de tempo multipista.
- Recorte, fundidos, normalización, redución de ruído básica.
- Mestura a un só ficheiro final (podcast).
- Todo apoiado en ffmpeg para a codificación final.

## Fase 4 — Trasno Vídeo

- Liña de tempo con varios planos/clips e pistas de audio.
- Recorte, xuntar clips (copia de stream cando os formatos coincidan,
  recodificación cando non), transicións simples.
- Perfís de compresión (tamaño obxectivo vs. calidade) e cambio de
  formato/contedor.

## Fase 5 — Trasno PDF (ampliación)

- Vista previa de páxinas con arrastrar para reordenar (xa hai motor).
- Anotacións e comentarios.
- Sinatura dixital (certificado local ou sinatura visual + selo).
- OCR opcional para PDFs escaneados.

## Fase 6 — Integración co sistema operativo

Deliberadamente despois de ter o motor e polo menos un estudo maduros,
para non deseñar a arquitectura de dous procesos ás cegas:

- **Trasno Lite**: proceso lixeiro que arrinca co sistema (bandexa do
  sistema), sen cargar a UI completa, que expón o motor de conversión.
- Entrada de menú contextual do explorador de ficheiros ("Converter con
  Trasno a...") en Windows (rexistro/Explorer), macOS (Servizos/Quick
  Actions) e Linux (Nautilus/Dolphin scripts), que fala co proceso lixeiro
  en vez de lanzar a app completa.
- A app completa (esta) e o proceso lixeiro comparten o mesmo
  `trasno-engine`, así que o comportamento (e as garantías de integridade)
  son idénticos en ambos os camiños.

## Fase 7 — Pulido

- Localización completa (o galego é o idioma principal; posibilidade de
  engadir outros).
- Accesibilidade (foco de teclado, contraste, lectores de pantalla).
- Actualizacións automáticas (Tauri updater).
- Telemetría opcional e opt-in para saber que falla máis (nunca datos dos
  ficheiros do usuario).
