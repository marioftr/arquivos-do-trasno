// Instala (ou actualiza) as entradas do menú do clic dereito do Explorador co idioma actual.

import { get } from "svelte/store";
import { t } from "./i18n";
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
}
