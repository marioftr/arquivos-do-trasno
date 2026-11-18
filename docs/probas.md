# Probas

Tres niveis, do máis rápido ao máis completo. Usa o que corresponda ao cambio; non fai falta pasalos todos sempre.

| Que cambiou | Que correr | Canto tarda |
|---|---|---|
| Código do motor (`crates/trasno-engine`) | `cargo test -p trasno-engine --test <ficheiro>` ou `--lib <módulo>` (o conxunto por defecto tarda ≈ 1 min) | segundos |
| Interface (Svelte) | `npm run typecheck`, `npm run check:i18n` e `npm run check:copy` (regras de texto, ver [texto.md](texto.md)) | segundos |
| Un fluxo que ve o usuario | `cargo build -p trasno` e `npm run e2e -- <escenario>` | 20-60 s |
| Antes de pechar unha versión | `cargo test -p trasno-engine -- --include-ignored` e `npm run e2e` | varios minutos |

## Banco da app real (`scripts/e2e`)

`npm run e2e` lanza o executable de depuración (`target/debug/trasno.exe`) cun perfil illado (carpeta temporal propia: non toca os teus
axustes nin ficheiros), conéctase por CDP e fai o que faría unha persoa: soltar un ficheiro, premer unha pestana, mover un
desprazador, exportar. Cada escenario comproba o que se vería: textos, duracións, o ficheiro que sae, que o orixinal non cambie.

```
npm run e2e                      # todos os escenarios
npm run e2e -- editar-video      # só un
```

Escenarios (en `scripts/e2e/scenarios/`): `editar-video`, `son-procesado`, `reloxo-longo`, `ocr-imaxe`, `iconas-info`, `silencios-selector`, `accesibilidade`, `niveis-calidade`, `modelo-3d`, `pdf-niveis`, `subtitulos-video`, `etiquetas-musica`, `restaurar-video`, `redimensionar-imaxes`, `xuntar-capturas`. Para engadir un, copia un deles:
exporta `{ about, run(app, check) }` e usa as axudas de `harness.mjs` (`open`, `click`, `slider`, `panel`, `waitFor`, `video`, `textImage`…).
Se falla, garda unha captura en `%TEMP%\trasno-e2e\erro-<escenario>.png`.

Requisitos: FFmpeg no PATH (ou en `C:\ffmpeg\bin`), `npm install` feito e, para o OCR, un paquete de idioma instalado na app (copiase ao perfil illado).

## Probas lentas

`tests/matriz.rs` pasa a edición, os silencios, o igualado de volume e os subtítulos (incrustar, queimar, sacar) por todos os contedores de audio e vídeo, con rutas raras e subtítulos en varias codificacións: comproba que o resultado decodifica limpo, dura o agardado e que o orixinal non cambia nin un byte (`cargo test -p trasno-engine --test matriz -- --include-ignored`, ≈ 1 min).

As baterías de `battery.rs` e `battery2.rs` (todas as conversións e edicións con todas as mostras) e a de Word en `office.rs` (abre Word de
verdade) tardan minutos en total (≈ 5). Corren só co indicador `--include-ignored` (antes de unha versión ou se tocas conversións de imaxe en xeral).
