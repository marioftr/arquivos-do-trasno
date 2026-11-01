// Documentos legais que se len na app (Sobre o trasno). Escritos nos tres idiomas; lense tal cal, sen pasar polo dicionario.
// Son textos informativos para unha aplicación de código aberto que non recolle datos: revísaos cunha persoa asesora
// se o proxecto medra ou se distribúe comercialmente.

export type DocId = "privacy" | "terms" | "license" | "signing";
type L = { gl: string; es: string; en: string };
export interface Section {
  h: L;
  p: L[];
}
export interface LegalDoc {
  title: L;
  updated: string;
  sections: Section[];
}

const REPO = "https://github.com/marioftr/arquivos-do-trasno";
export const LINKS = { repo: REPO, issues: `${REPO}/issues`, releases: `${REPO}/releases`, license: `${REPO}/blob/HEAD/LICENSE`, readme: `${REPO}#readme` };

export const DOCS: Record<DocId, LegalDoc> = {
  privacy: {
    title: { gl: "Política de privacidade", es: "Política de privacidad", en: "Privacy policy" },
    updated: "2026-11-01",
    sections: [
      {
        h: { gl: "Resumo", es: "Resumen", en: "Summary" },
        p: [
          {
            gl: "Arquivos do Trasno non recolle datos persoais, non ten rexistro de usuarios nin telemetría. Os ficheiros que abres procésanse no teu ordenador e os orixinais non se modifican.",
            es: "Arquivos do Trasno no recoge datos personales, no tiene registro de usuarios ni telemetría. Los archivos que abres se procesan en tu ordenador y los originales no se modifican.",
            en: "Arquivos do Trasno does not collect personal data, has no user accounts and no telemetry. The files you open are processed on your computer and the originals are never modified.",
          },
        ],
      },
      {
        h: { gl: "Que se garda no teu ordenador", es: "Qué se guarda en tu ordenador", en: "What is stored on your computer" },
        p: [
          {
            gl: "Para funcionar, a app garda en local: os teus axustes (idioma, tema, cartafol de saída), os atallos de teclado, os perfís de saída que crees, a lista de ficheiros da última sesión (as súas rutas), o historial de procesos (nomes e rutas dos ficheiros, ata 40 entradas ou 14 días) e copias temporais para as vistas previas, que se borran ao abrir a app. Nada diso sae do teu ordenador. Podes borralo desde a propia app (historial) ou eliminando os datos da aplicación.",
            es: "Para funcionar, la app guarda en local: tus ajustes (idioma, tema, carpeta de salida), los atajos de teclado, los perfiles de salida que crees, la lista de archivos de la última sesión (sus rutas), el historial de procesos (nombres y rutas de archivos, hasta 40 entradas o 14 días) y copias temporales para las vistas previas, que se borran al abrir la app. Nada de eso sale de tu ordenador. Puedes borrarlo desde la propia app (historial) o eliminando los datos de la aplicación.",
            en: "To work, the app stores locally: your settings (language, theme, output folder), keyboard shortcuts, the output profiles you create, the list of files from your last session (their paths), the process history (file names and paths, up to 40 entries or 14 days) and temporary copies for previews, which are deleted when the app opens. None of this leaves your computer. You can delete it from the app (history) or by removing the application data.",
          },
        ],
      },
      {
        h: { gl: "Conexións á rede", es: "Conexiones de red", en: "Network connections" },
        p: [
          {
            gl: "A única conexión que a app fai por si soa é a busca de versións novas: pide o ficheiro público «latest.json» dos Releases deste proxecto en GitHub e, se aceptas unha actualización, descarga o instalador de alí. Nesa petición non se envía información sobre ti nin sobre os teus ficheiros, máis alá do que revela calquera petición HTTPS (o teu enderezo IP, que GitHub trata segundo a súa propia política de privacidade). Podes desactivar a busca automática en Novidades → Actualizacións.",
            es: "La única conexión que la app hace por sí sola es la búsqueda de versiones nuevas: pide el archivo público «latest.json» de los Releases de este proyecto en GitHub y, si aceptas una actualización, descarga el instalador de allí. En esa petición no se envía información sobre ti ni sobre tus archivos, más allá de lo que revela cualquier petición HTTPS (tu dirección IP, que GitHub trata según su propia política de privacidad). Puedes desactivar la búsqueda automática en Novedades → Actualizaciones.",
            en: "The only connection the app makes by itself is the check for new versions: it requests the public “latest.json” file from this project's GitHub Releases and, if you accept an update, downloads the installer from there. That request sends no information about you or your files beyond what any HTTPS request reveals (your IP address, which GitHub handles under its own privacy policy). You can turn the automatic check off in What's new → Updates.",
          },
          {
            gl: "As ligazóns que se amosan na app (por exemplo, onde descargar ffmpeg) só se abren no teu navegador cando as premes.",
            es: "Los enlaces que se muestran en la app (por ejemplo, dónde descargar ffmpeg) solo se abren en tu navegador cuando los pulsas.",
            en: "Links shown in the app (for example, where to download ffmpeg) only open in your browser when you click them.",
          },
        ],
      },
      {
        h: { gl: "Programas externos", es: "Programas externos", en: "External programs" },
        p: [
          {
            gl: "Para audio e vídeo a app usa FFmpeg e, para pistas de MKV, MKVToolNix, se os tes instalados. Execútanse no teu ordenador e a app non os instala nin lles envía nada á rede.",
            es: "Para audio y vídeo la app usa FFmpeg y, para pistas de MKV, MKVToolNix, si los tienes instalados. Se ejecutan en tu ordenador y la app no los instala ni les envía nada a la red.",
            en: "For audio and video the app uses FFmpeg and, for MKV tracks, MKVToolNix, if you have them installed. They run on your computer and the app neither installs them nor sends anything to the network.",
          },
        ],
      },
      {
        h: { gl: "Cambios no sistema", es: "Cambios en el sistema", en: "System changes" },
        p: [
          {
            gl: "A app só toca a configuración do sistema cando llo pides: o menú do clic dereito do Explorador engádese e quítase cos botóns de Axustes → Xeral, escribe só no rexistro do teu usuario (sen permisos de administrador) e hai que quitalo antes de desinstalar a app.",
            es: "La app solo toca la configuración del sistema cuando se lo pides: el menú de clic derecho del Explorador se añade y se quita con los botones de Ajustes → General, escribe solo en el registro de tu usuario (sin permisos de administrador) y hay que quitarlo antes de desinstalar la app.",
            en: "The app only changes system configuration when you ask it to: the Explorer right-click menu is added and removed with the buttons in Settings → General, writes only to your user registry (no administrator rights) and should be removed before uninstalling the app.",
          },
        ],
      },
      {
        h: { gl: "Os teus dereitos e contacto", es: "Tus derechos y contacto", en: "Your rights and contact" },
        p: [
          {
            gl: "Como o proxecto non ten datos persoais teus, non hai nada que acceder, rectificar nin suprimir. Se tes dúbidas ou queres sinalar un problema de privacidade, abre unha incidencia no repositorio do proxecto. Se esta política cambia, o cambio publicarase no rexistro de cambios e aquí, coa data de actualización.",
            es: "Como el proyecto no tiene datos personales tuyos, no hay nada que acceder, rectificar ni suprimir. Si tienes dudas o quieres señalar un problema de privacidad, abre una incidencia en el repositorio del proyecto. Si esta política cambia, el cambio se publicará en el registro de cambios y aquí, con la fecha de actualización.",
            en: "Since the project holds no personal data about you, there is nothing to access, correct or delete. If you have questions or want to report a privacy issue, open an issue in the project's repository. If this policy changes, the change will be published in the changelog and here, with the update date.",
          },
        ],
      },
    ],
  },

  terms: {
    title: { gl: "Termos e condicións de uso", es: "Términos y condiciones de uso", en: "Terms of use" },
    updated: "2026-11-01",
    sections: [
      {
        h: { gl: "Aceptación", es: "Aceptación", en: "Acceptance" },
        p: [
          {
            gl: "Ao instalar ou usar Arquivos do Trasno aceptas estes termos. Se non estás de acordo, non uses a aplicación e desinstálaa.",
            es: "Al instalar o usar Arquivos do Trasno aceptas estos términos. Si no estás de acuerdo, no uses la aplicación y desinstálala.",
            en: "By installing or using Arquivos do Trasno you accept these terms. If you do not agree, do not use the application and uninstall it.",
          },
        ],
      },
      {
        h: { gl: "Licenza", es: "Licencia", en: "Licence" },
        p: [
          {
            gl: "A aplicación é software libre baixo licenza MIT (ver «Licenza»): podes usala, copiala, modificala e distribuíla segundo esa licenza. As bibliotecas e programas de terceiros que inclúe ou usa teñen as súas propias licenzas, listadas en «Créditos e licenzas».",
            es: "La aplicación es software libre bajo licencia MIT (ver «Licencia»): puedes usarla, copiarla, modificarla y distribuirla según esa licencia. Las bibliotecas y programas de terceros que incluye o usa tienen sus propias licencias, listadas en «Créditos y licencias».",
            en: "The application is free software under the MIT licence (see “Licence”): you may use, copy, modify and distribute it under that licence. Third-party libraries and programs it includes or uses have their own licences, listed in “Credits and licences”.",
          },
        ],
      },
      {
        h: { gl: "Sen garantía", es: "Sin garantía", en: "No warranty" },
        p: [
          {
            gl: "A aplicación ofrécese «tal cal», sen garantía de ningún tipo, expresa ou implícita, incluídas as de comerciabilidade, adecuación a un fin concreto e non infracción. Fíxose todo o posible para non modificar nunca os orixinais (as ferramentas gardan copias novas e non pisan ficheiros existentes), pero non se garante que o resultado dunha conversión sexa idéntico ao orixinal nin que a aplicación estea libre de erros.",
            es: "La aplicación se ofrece «tal cual», sin garantía de ningún tipo, expresa o implícita, incluidas las de comerciabilidad, idoneidad para un fin concreto y no infracción. Se ha hecho todo lo posible para no modificar nunca los originales (las herramientas guardan copias nuevas y no pisan archivos existentes), pero no se garantiza que el resultado de una conversión sea idéntico al original ni que la aplicación esté libre de errores.",
            en: "The application is provided “as is”, without warranty of any kind, express or implied, including merchantability, fitness for a particular purpose and non-infringement. Every effort is made never to modify originals (tools save new copies and never overwrite existing files), but there is no guarantee that a conversion's result is identical to the original or that the application is error-free.",
          },
        ],
      },
      {
        h: { gl: "As túas copias e os teus ficheiros", es: "Tus copias y tus archivos", en: "Your backups and your files" },
        p: [
          {
            gl: "Es responsable de manter copias de seguridade dos teus ficheiros e de que tes dereito a procesalos (por exemplo, os dereitos de autor do contido que converte, comprime ou edita). O uso da aplicación para infrinxir dereitos de terceiros ou a lei non está permitido.",
            es: "Eres responsable de mantener copias de seguridad de tus archivos y de tener derecho a procesarlos (por ejemplo, los derechos de autor del contenido que conviertes, comprimes o editas). No se permite usar la aplicación para infringir derechos de terceros o la ley.",
            en: "You are responsible for keeping backups of your files and for having the right to process them (for example, the copyright of the content you convert, compress or edit). Using the application to infringe third-party rights or the law is not permitted.",
          },
        ],
      },
      {
        h: { gl: "Limitación de responsabilidade", es: "Limitación de responsabilidad", en: "Limitation of liability" },
        p: [
          {
            gl: "Na medida máxima permitida pola lei, as persoas autoras e colaboradoras non serán responsables de ningún dano directo ou indirecto (perda de datos, de beneficios ou doutro tipo) derivado do uso ou da imposibilidade de usar a aplicación. Isto non limita os dereitos irrenunciables que che recoñeza a lei como persoa consumidora.",
            es: "En la máxima medida permitida por la ley, las personas autoras y colaboradoras no serán responsables de ningún daño directo o indirecto (pérdida de datos, de beneficios o de otro tipo) derivado del uso o de la imposibilidad de usar la aplicación. Esto no limita los derechos irrenunciables que la ley te reconozca como persona consumidora.",
            en: "To the maximum extent permitted by law, the authors and contributors are not liable for any direct or indirect damage (loss of data, profits or otherwise) arising from the use of, or inability to use, the application. This does not limit any non-waivable rights the law gives you as a consumer.",
          },
        ],
      },
      {
        h: { gl: "Actualizacións e cambios", es: "Actualizaciones y cambios", en: "Updates and changes" },
        p: [
          {
            gl: "A aplicación pode ofrecerche versións novas; instalalas é decisión túa. Estes termos poden cambiar: a versión vixente é a que se amosa aquí, coa data de actualización.",
            es: "La aplicación puede ofrecerte versiones nuevas; instalarlas es decisión tuya. Estos términos pueden cambiar: la versión vigente es la que se muestra aquí, con la fecha de actualización.",
            en: "The application may offer you new versions; installing them is your decision. These terms may change: the current version is the one shown here, with its update date.",
          },
        ],
      },
      {
        h: { gl: "Lei aplicable e contacto", es: "Ley aplicable y contacto", en: "Governing law and contact" },
        p: [
          {
            gl: "Estes termos réxense pola lexislación española, sen prexuízo dos dereitos que che recoñeza a lei do teu lugar de residencia como persoa consumidora. Para calquera consulta, abre unha incidencia no repositorio do proxecto.",
            es: "Estos términos se rigen por la legislación española, sin perjuicio de los derechos que te reconozca la ley de tu lugar de residencia como persona consumidora. Para cualquier consulta, abre una incidencia en el repositorio del proyecto.",
            en: "These terms are governed by Spanish law, without prejudice to the rights the law of your place of residence gives you as a consumer. For any question, open an issue in the project's repository.",
          },
        ],
      },
    ],
  },

  license: {
    title: { gl: "Licenza do programa (MIT)", es: "Licencia del programa (MIT)", en: "Program licence (MIT)" },
    updated: "2026-11-01",
    sections: [
      {
        h: { gl: "MIT License", es: "MIT License", en: "MIT License" },
        p: [
          { gl: "Copyright (c) 2026 marioftr", es: "Copyright (c) 2026 marioftr", en: "Copyright (c) 2026 marioftr" },
          {
            gl: "Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:",
            es: "Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:",
            en: "Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:",
          },
          {
            gl: "The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.",
            es: "The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.",
            en: "The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.",
          },
          {
            gl: "THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.",
            es: "THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.",
            en: "THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.",
          },
        ],
      },
    ],
  },

  signing: {
    title: { gl: "Política de asinado de código", es: "Política de firma de código", en: "Code signing policy" },
    updated: "2026-11-01",
    sections: [
      {
        h: { gl: "Quen asina e como", es: "Quién firma y cómo", en: "Who signs and how" },
        p: [
          {
            gl: "Free code signing provided by SignPath.io, certificate by SignPath Foundation. (Cando o certificado estea activo, os instaladores publicados levarán esta sinatura de Windows.)",
            es: "Free code signing provided by SignPath.io, certificate by SignPath Foundation. (Cuando el certificado esté activo, los instaladores publicados llevarán esta firma de Windows.)",
            en: "Free code signing provided by SignPath.io, certificate by SignPath Foundation. (Once the certificate is active, published installers will carry this Windows signature.)",
          },
          {
            gl: "Roles: autoría, revisión e aprobación — marioftr (GitHub). Os binarios constrúense automaticamente con GitHub Actions a partir do código deste repositorio, e cada versión require aprobación manual antes de asinarse.",
            es: "Roles: autoría, revisión y aprobación — marioftr (GitHub). Los binarios se construyen automáticamente con GitHub Actions a partir del código de este repositorio, y cada versión requiere aprobación manual antes de firmarse.",
            en: "Roles: authors, reviewers and approvers — marioftr (GitHub). Binaries are built automatically by GitHub Actions from this repository's source code, and every release needs manual approval before it is signed.",
          },
        ],
      },
      {
        h: { gl: "Dúas sinaturas distintas", es: "Dos firmas distintas", en: "Two different signatures" },
        p: [
          {
            gl: "Cada instalador leva ademais un ficheiro «.sig»: é a sinatura propia das actualizacións automáticas, coa que a app comproba que o ficheiro descargado é de quen publica o proxecto. É independente da sinatura de Windows.",
            es: "Cada instalador lleva además un archivo «.sig»: es la firma propia de las actualizaciones automáticas, con la que la app comprueba que el archivo descargado es de quien publica el proyecto. Es independiente de la firma de Windows.",
            en: "Each installer also has a “.sig” file: that is the auto-update signature, which the app uses to check that the downloaded file comes from the project's publisher. It is independent of the Windows signature.",
          },
        ],
      },
    ],
  },
};
