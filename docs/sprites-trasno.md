# Sprites do trasno que fan falta

Todos en `public/trasno/`, PNG de 32 × 32 con fondo transparente (a app amplíaos a 64 px, sen suavizar).
O compoñente `src/lib/components/ui/TrasnoSprite.svelte` ten unha secuencia por estado: cambiar os PNG co mesmo nome
substitúe a arte en toda a app. Os que faltan indícanse como «novo».

## Xa existen
- `base.png` — repouso
- `medio.png` — ollos a medio pechar (pestanexo)
- `pechado.png` — ollos pechados

## Para a barra inferior
- `traballando-1.png`, `traballando-2.png` — novo: bucle mentres hai procesos (por exemplo, bombeando ou escribindo)
- `aviso.png` — novo: cun «!» ou a man levantada, para avisos importantes (falta FFmpeg, versión nova)
- `rematado.png` — novo: contento, ao acabar un proceso
- `tropezou.png` — novo: cun erro, tropezando ou confuso
- `fala.png` — novo: boca aberta, mentres sae un globo de comentario ou consello
- `dormido-1.png`, `dormido-2.png` — novo: se non hai actividade durante moito tempo

## Para a animación de empezar un proceso
- `cae.png` — novo: caendo en picado cara á barra
- `sinala.png` — novo: aparece por riba, sinalando o botón que se acaba de premer
- `agacha.png` — novo: agachándose/saíndo da barra (baixar e subir)

## Para o menú radial
- `ola.png` — novo: levanta a man cando se pasa o rato por riba

## Para pantallas grandes (máis adiante)
- Sen ficheiros: trasno en grande esperando (versión 64 × 64 ou 96 × 96)
- Procesos: un trasno grande con folla de ruta
- Ferramenta sen resultados / erro grave: trasno desconcertado
