// Instala (ou actualiza) as entradas do menú do clic dereito do Explorador co idioma actual.

import { get } from "svelte/store";
import { getVersion } from "@tauri-apps/api/app";
import { t, lang } from "./i18n";
import * as engine from "./engine";

export async function installExplorerMenu(): Promise<void> {
  const tt = get(t);
  await engine.explorerInstall({
    title: "Arquivos do Trasno",
    open: tt("Abrir en Arquivos do Trasno"),
    convert: tt("Converter a {ext}", { ext: "{ext}" }),
    more: tt("Máis formatos"),
    resize: tt("Cambiar tamaño"),
    scale: tt("Escalar"),
    rotate: tt("Xirar"),
    left: tt("Á esquerda"),
    right: tt("Á dereita"),
    half: tt("Dar a volta (180°)"),
    compress: tt("Comprimir"),
    quality: tt("Calidade {p} %", { p: "{p}" }),
    percent: tt("{p} %", { p: "{p}" }),
  });
  try {
    localStorage.setItem(STAMP, await stampNow());
  } catch {
    // ignorar
  }
}

const STAMP = "arquivos-do-trasno.menu-explorador";
const stampNow = async () => `${await getVersion().catch(() => "")}|${get(lang)}`;

/** Se hai entradas desta copia da app, ponas ao día ao cambiar a versión ou o idioma (e se falta algún tipo de ficheiro). Nunca toca as dunha copia distinta. */
export async function keepMenuFresh(): Promise<void> {
  try {
    const s = await engine.explorerStatus();
    if (!s.installed || !s.current) return;
    const stamp = await stampNow();
    if (s.complete && localStorage.getItem(STAMP) === stamp) return;
    await installExplorerMenu();
  } catch {
    // sen menú que axustar (outro sistema) ou sen almacenamento: nada que facer
  }
}

/** Arranca a vixilancia: unha comprobación agora e outra cada vez que cambia o idioma. */
export function watchExplorerMenu(): void {
  let first = true;
  lang.subscribe(() => {
    if (first) return void (first = false);
    void keepMenuFresh();
  });
  void keepMenuFresh();
}
