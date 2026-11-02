# Arquivos do Trasno

Aplicación de escritorio para Windows, de código aberto e en galego, para converter, comprimir, editar e inspeccionar
imaxes, vídeo, audio, PDF e modelos 3D. Todo se procesa no propio equipo: non hai contas, nin subida de ficheiros, nin
telemetría. Os ficheiros orixinais nunca se modifican.

## Funcións

- **Converter e comprimir** imaxes, audio e vídeo, con perfís de saída (universal, Mini DV a MP4, YouTube, móbil) e perfís propios gardables.
- **Editar**: recortar, unir, efectos de audio, ferramentas de imaxe e de PDF (unir, dividir, rotar, reordenar, marca de auga, protexer, comprimir…).
- **Visores** para cada tipo de ficheiro (imaxe, audio con onda, vídeo con selección de pistas, PDF con busca e selección de texto, modelos 3D).
- **Menú do clic dereito** do Explorador de Windows para converter e comprimir sen abrir a aplicación.
- **Pistas de MKV** (con MKVToolNix, opcional): quitar, reordenar, renomear e mesturar.
- Xestor de procesos con historial, atallos de teclado configurables, tema claro/escuro e interface en galego, castelán e inglés.

Para audio e vídeo úsase [FFmpeg](https://ffmpeg.org/), que non se inclúe na aplicación: debe estar instalado no sistema
(a aplicación indica como facelo en *Axustes → Extensións*).

## Descarga

Os instaladores para Windows están en [Releases](https://github.com/marioftr/arquivos-do-trasno/releases/latest).
A aplicación busca versións novas ao abrirse; pódese desactivar en *Novidades → Actualizacións*.

Os instaladores non están asinados cun certificado de Windows, polo que SmartScreen pode amosar un aviso ao instalar
(ver [Asinado e verificación](#asinado-e-verificación)).

## Asinado e verificación

Os instaladores non levan unha sinatura de código de Windows, así que SmartScreen pode amosar un aviso ao instalar.
Os binarios constrúense con GitHub Actions a partir do código deste repositorio (ver
[`.github/workflows/release.yml`](.github/workflows/release.yml)) e cada versión inclúe un ficheiro `.sig` coa
sinatura que a aplicación comproba antes de instalar unha actualización. Os programas que a aplicación descarga
por petición do usuario (FFmpeg e MKVToolNix) compróbanse coa suma SHA-256 publicada pola súa fonte.

## Privacy policy

This program does not collect, store or send personal data, and it does not send telemetry. The files you open
are processed locally and the originals are never modified.

The only network connection the program makes by itself is a check for new versions: it requests the public file
`latest.json` from this project's GitHub Releases (and, if you accept an update, downloads the installer from there).
No information about you or your files is sent in that request beyond what any HTTPS request reveals (your IP address).
You can turn the automatic check off in *Novidades → Actualizacións*. Links shown in the app (for example, where to
download FFmpeg) open in your browser only when you click them.

The program only changes system configuration when you ask it to: the Explorer right-click menu is added or removed
with explicit buttons in *Axustes → Xeral*, writes only to your user registry (no administrator rights) and is removed
by the uninstaller or with the *Quitar do menú* button.

## Integridade dos ficheiros

- Nunca se sobrescribe un ficheiro sen permiso explícito.
- Toda escritura pasa por un ficheiro temporal no mesmo directorio, que se valida (tamaño, lectura ou decodificación,
  `ffprobe` cando aplica) antes dun `rename` atómico. Se algo falla, o destino final nunca chega a existir.
- O ficheiro orixinal nunca se modifica nin se borra como parte dunha conversión.

## Desenvolvemento

Tecnoloxías: [Tauri](https://tauri.app/) 2, Rust (motor de conversión) e Svelte + TypeScript (interface).
Requisitos: Rust con Cargo, Node.js 20 ou superior e, para audio e vídeo, `ffmpeg`/`ffprobe` no PATH.

```bash
npm install
npm run tauri dev                  # aplicación en modo desenvolvemento
npm run tauri build -- --no-bundle # executable de release en target/release
```

Probas e comprobacións:

```bash
cargo test -p trasno-engine   # probas do motor de conversión
npm run typecheck             # svelte-check
npm run check:i18n            # traducións completas (gl, es, en)
```

Estrutura:

```
crates/trasno-engine/   Motor de conversión (Rust, sen interface, con probas)
src-tauri/              Aplicación Tauri (Rust): comandos, menú do Explorador, instalador
src/                    Interface Svelte + TypeScript
docs/ROADMAP.md         Liñas de traballo futuras
```

## Licenza

[MIT](LICENSE). As dependencias de terceiros manteñen as súas propias licenzas; ver *Sobre o trasno → Créditos e licenzas* na aplicación.
