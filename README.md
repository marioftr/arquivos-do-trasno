# Trasno

Aplicación de escritorio multiusos centrada en contido multimedia, en
galego: catro estudos (**Imaxe**, **Audio**, **PDF**, **Vídeo**) máis un
**conversor universal de ficheiros**, todo na mesma app.

A idea non é clonar Photoshop, Premiere ou Acrobat, senón cubrir o 80% do
que a xente usa deses programas día a día, cunha soa aplicación lixeira,
rápida e en galego.

## Descarga

Os instaladores para Windows están en [Releases](https://github.com/marioftr/arquivos-do-trasno/releases/latest).
A app busca versións novas ao abrirse (pódese desactivar en Novidades → Actualizacións).

## Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org/).

Roles of the team members:

- **Authors, reviewers and approvers:** [marioftr](https://github.com/marioftr)

Binaries are built automatically by GitHub Actions from the source code in this repository (see
`.github/workflows/release.yml`); every release needs manual approval before it is signed.

## Privacy policy

This program does not collect, store or send personal data, and it does not send telemetry. The files you open
are processed locally and the originals are never modified.

The only network connection the program makes by itself is a check for new versions: it requests the public file
`latest.json` from this project's GitHub Releases (and, if you accept an update, downloads the installer from there).
No information about you or your files is sent in that request beyond what any HTTPS request reveals (your IP address).
You can turn the automatic check off in *Novidades → Actualizacións*. Links shown in the app (for example, where to
download ffmpeg) open in your browser only when you click them.

The program only changes system configuration when you ask it to: the *Explorer right-click menu* is added or removed
with explicit buttons in *Axustes → Xeral*, writes only to your user registry (no administrator rights) and is removed
by uninstalling the program or with the *Quitar do menú* button.

## Licenza

[MIT](LICENSE).

## Estado actual (fase 0 → 1)

Este é un proxecto novo, construído dende cero. O que xa funciona:

- **Motor de conversión** (`crates/trasno-engine`), Rust puro, con tests:
  - **Imaxes**: PNG, JPEG, GIF, BMP, ICO, TIFF, WebP, PNM/PBM/PGM/PPM, TGA,
    QOI, AVIF — decodificación e codificación sen depender de ningún
    programa externo.
  - **PDF**: unir, dividir/extraer páxinas e rotar, con `lopdf`, incluíndo
    herdanza correcta de atributos de páxina (MediaBox, Resources...).
  - **Audio/vídeo**: wrapper sobre `ffmpeg`/`ffprobe` do sistema (conversión
    de formato, recorte sen recodificar, xuntar clips).
  - **Integridade garantida**: nunca se escribe directamente sobre o
    ficheiro de destino. Todo pasa primeiro por un temporal no mesmo
    directorio, valídase (tamaño > 0, decodificación/lectura correcta,
    `ffprobe` cando aplica) e só despois se fai un `rename` atómico. Se
    algo falla a medio camiño, o destino final nunca chega a existir.
- **Aplicación Tauri** (`src-tauri`) que expón ese motor á UI mediante
  comandos (`convert_file`, `pdf_merge`, `trim_media`...).
- **Interface** (`src/`, Svelte + TypeScript): pantalla de inicio cos catro
  estudos + conversor (ver capturas orixinais que inspiraron o deseño), e
  un **Conversor** xa funcional de punta a punta para imaxes (e para
  audio/vídeo se hai `ffmpeg` instalado no sistema).

Os catro estudos (Imaxe/Audio/PDF/Vídeo) están de momento como pantallas
"en construción" que xa amosan a folla de ruta e enlazan ao conversor
mentres se desenvolven — ver [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Por que Tauri

- Binarios pequenos e baixo consumo de RAM (importante se máis adiante hai
  un proceso lixeiro sempre en segundo plano para o menú contextual).
- Backend en Rust: bo acceso a librarías de imaxe/PDF/vídeo e control fino
  sobre integridade de ficheiros.
- Frontend web normal (Svelte + TypeScript) para a UI.

## Desenvolvemento

Requisitos: Rust + Cargo, Node.js 20+, e (opcional pero recomendado para
audio/vídeo) `ffmpeg`/`ffprobe` no PATH.

```bash
npm install
npm run tauri dev      # app de escritorio en modo desenvolvemento
```

Só backend (sen UI):

```bash
cargo test -p trasno-engine   # tests do motor de conversión
cargo check --workspace       # compila motor + app Tauri
```

Só frontend:

```bash
npm run dev          # servidor Vite en http://localhost:1420
npm run build        # build de produción a dist/
npm run typecheck    # svelte-check
```

## Estrutura

```
crates/trasno-engine/   Motor de conversión (Rust, sen GUI, con tests)
src-tauri/              App de escritorio Tauri (Rust) + comandos expostos á UI
src/                    Frontend Svelte + TypeScript
docs/ROADMAP.md         Fases previstas do proxecto
```

## Seguridade e integridade de ficheiros

- Nunca se sobrescribe un ficheiro sen permiso explícito (`overwrite`).
- Toda escritura pasa por un ficheiro temporal + `rename` atómico.
- Cada conversión valídase reabrindo/decodificando o resultado antes de
  darlla por boa ao usuario; se falla, bórrase o temporal e devólvese un
  erro claro en galego, nunca un ficheiro corrompido.
- O ficheiro orixinal nunca se modifica nin se borra como parte dunha
  conversión.
