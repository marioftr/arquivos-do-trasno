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
    updated: "2026-11-04",
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
          {
            gl: "Tamén se gardan no teu usuario (cartafol de datos da app) os programas e paquetes que descargas desde a propia app: FFmpeg, MKVToolNix e os paquetes de idioma do recoñecemento de texto (OCR). Pódense quitar desde Axustes → Extensións. O historial de accións da barra inferior só vive na memoria e desaparece ao pechar a app.",
            es: "También se guardan en tu usuario (carpeta de datos de la app) los programas y paquetes que descargas desde la propia app: FFmpeg, MKVToolNix y los paquetes de idioma del reconocimiento de texto (OCR). Se pueden quitar desde Ajustes → Extensiones. El historial de acciones de la barra inferior solo vive en la memoria y desaparece al cerrar la app.",
            en: "The programs and packages you download from within the app are also stored in your user profile (the app's data folder): FFmpeg, MKVToolNix and the text-recognition (OCR) language packs. They can be removed in Settings → Extensions. The action history in the bottom bar lives in memory only and disappears when the app closes.",
          },
        ],
      },
      {
        h: { gl: "Conexións á rede", es: "Conexiones de red", en: "Network connections" },
        p: [
          {
            gl: "A app só se conecta a Internet nestes casos: (1) a busca de versións novas, que pide o ficheiro público «latest.json» dos Releases deste proxecto en GitHub e, se aceptas unha actualización, descarga o instalador de alí; (2) cando ti aceptas descargar FFmpeg (de gyan.dev) ou MKVToolNix (de mkvtoolnix.download); e (3) cando ti pides un paquete de idioma para o recoñecemento de texto (do proxecto Tesseract en GitHub). Nesas peticións non se envía información sobre ti nin sobre os teus ficheiros, máis alá do que revela calquera petición HTTPS (o teu enderezo IP, que o servidor trata segundo a súa propia política). As descargas de programas compróbanse (suma de verificación) antes de usalas.",
            es: "La app solo se conecta a Internet en estos casos: (1) la búsqueda de versiones nuevas, que pide el archivo público «latest.json» de los Releases de este proyecto en GitHub y, si aceptas una actualización, descarga el instalador de allí; (2) cuando tú aceptas descargar FFmpeg (de gyan.dev) o MKVToolNix (de mkvtoolnix.download); y (3) cuando tú pides un paquete de idioma para el reconocimiento de texto (del proyecto Tesseract en GitHub). En esas peticiones no se envía información sobre ti ni sobre tus archivos, más allá de lo que revela cualquier petición HTTPS (tu dirección IP, que el servidor trata según su propia política). Las descargas de programas se comprueban (suma de verificación) antes de usarlas.",
            en: "The app connects to the Internet only in these cases: (1) the check for new versions, which requests the public “latest.json” file from this project's GitHub Releases and, if you accept an update, downloads the installer from there; (2) when you agree to download FFmpeg (from gyan.dev) or MKVToolNix (from mkvtoolnix.download); and (3) when you ask for a language pack for text recognition (from the Tesseract project on GitHub). Those requests send no information about you or your files beyond what any HTTPS request reveals (your IP address, which the server handles under its own policy). Program downloads are verified (checksum) before use.",
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
            gl: "Para audio e vídeo a app usa FFmpeg (tamén para ler e escribir AVIF, JPEG 2000 e JPEG XL) e, para pistas de MKV, MKVToolNix. Non van no instalador: podes usar os que xa teñas instalados ou deixar que a app os descargue, só se o aceptas, en Axustes → Extensións. Execútanse no teu ordenador como programas independentes e a app non lles envía nada á rede. Para documentos de Office (Word, Excel, PowerPoint e OpenDocument) a app usa Microsoft Office ou LibreOffice, se os tes instalados: non os inclúe, non os descarga e non lles envía nada á rede. Con Microsoft Office, a app manexa Word, Excel ou PowerPoint en segundo plano, sen xanelas, e abre os teus documentos só para lectura (sempre se garda unha copia noutro ficheiro e o orixinal non se toca). Cada un deses programas ten a súa propia licenza e condicións de uso, que debes cumprir ti como usuario.",
            es: "Para audio y vídeo la app usa FFmpeg (también para leer y escribir AVIF, JPEG 2000 y JPEG XL) y, para pistas de MKV, MKVToolNix. No van en el instalador: puedes usar los que ya tengas instalados o dejar que la app los descargue, solo si lo aceptas, en Ajustes → Extensiones. Se ejecutan en tu ordenador como programas independientes y la app no les envía nada a la red. Para documentos de Office (Word, Excel, PowerPoint y OpenDocument) la app usa Microsoft Office o LibreOffice, si los tienes instalados: no los incluye, no los descarga y no les envía nada a la red. Con Microsoft Office, la app maneja Word, Excel o PowerPoint en segundo plano, sin ventanas, y abre tus documentos solo para lectura (siempre se guarda una copia en otro archivo y el original no se toca). Cada uno de esos programas tiene su propia licencia y condiciones de uso, que debes cumplir tú como usuario.",
            en: "For audio and video the app uses FFmpeg (also to read and write AVIF, JPEG 2000 and JPEG XL) and, for MKV tracks, MKVToolNix. They are not in the installer: you can use the ones you already have installed or let the app download them, only if you agree, in Settings → Extensions. They run on your computer as independent programs and the app sends nothing to the network on their behalf. For Office documents (Word, Excel, PowerPoint and OpenDocument) the app uses Microsoft Office or LibreOffice, if you have them installed: it does not include them, does not download them and sends nothing to the network on their behalf. With Microsoft Office, the app drives Word, Excel or PowerPoint in the background, without windows, and opens your documents read-only (a copy is always saved to another file and the original is never touched). Each of those programs has its own licence and terms of use, which you must comply with as the user.",
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
    updated: "2026-11-04",
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
          {
            gl: "Os programas e datos de terceiros que descargas desde a propia app (FFmpeg, MKVToolNix e os paquetes de idioma do OCR) descárganse só se o pides, non forman parte desta aplicación e rexense polas súas propias licenzas e termos, que debes aceptar para usalos.",
            es: "Los programas y datos de terceros que descargas desde la propia app (FFmpeg, MKVToolNix y los paquetes de idioma del OCR) se descargan solo si lo pides, no forman parte de esta aplicación y se rigen por sus propias licencias y términos, que debes aceptar para usarlos.",
            en: "Third-party programs and data you download from within the app (FFmpeg, MKVToolNix and the OCR language packs) are downloaded only if you ask, are not part of this application and are governed by their own licences and terms, which you must accept to use them.",
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
    title: { gl: "Asinado e verificación das descargas", es: "Firma y verificación de las descargas", en: "Signing and download verification" },
    updated: "2026-11-04",
    sections: [
      {
        h: { gl: "Aviso de Windows ao instalar", es: "Aviso de Windows al instalar", en: "Windows warning when installing" },
        p: [
          {
            gl: "Os instaladores de Arquivos do Trasno non levan unha sinatura de código de Windows (Authenticode), porque os certificados recoñecidos teñen un custo e a solicitude dun certificado gratuíto para proxectos de código aberto foi rexeitada por agora. Por iso Windows SmartScreen pode amosar un aviso («Windows protexeu o seu PC») ao executalo. Se descargaches o instalador da páxina oficial de versións do proxecto, podes continuar con «Máis información → Executar igualmente».",
            es: "Los instaladores de Arquivos do Trasno no llevan una firma de código de Windows (Authenticode), porque los certificados reconocidos tienen un coste y la solicitud de un certificado gratuito para proyectos de código abierto ha sido rechazada por ahora. Por eso Windows SmartScreen puede mostrar un aviso («Windows protegió su PC») al ejecutarlo. Si has descargado el instalador de la página oficial de versiones del proyecto, puedes continuar con «Más información → Ejecutar de todas formas».",
            en: "Arquivos do Trasno installers do not carry a Windows code signature (Authenticode): recognised certificates have a cost, and the application for a free certificate for open-source projects has been declined for now. Because of that, Windows SmartScreen may show a warning (“Windows protected your PC”) when you run it. If you downloaded the installer from the project's official releases page, you can continue with “More info → Run anyway”.",
          },
        ],
      },
      {
        h: { gl: "Como se comproba que o ficheiro é do proxecto", es: "Cómo se comprueba que el archivo es del proyecto", en: "How the file is verified as the project's" },
        p: [
          {
            gl: "Os binarios constrúense automaticamente con GitHub Actions a partir do código deste repositorio e publícanse en GitHub Releases. Cada instalador leva ademais un ficheiro «.sig»: é a sinatura propia das actualizacións automáticas, coa que a app comproba que o ficheiro descargado procede de quen publica o proxecto antes de instalalo.",
            es: "Los binarios se construyen automáticamente con GitHub Actions a partir del código de este repositorio y se publican en GitHub Releases. Cada instalador lleva además un archivo «.sig»: es la firma propia de las actualizaciones automáticas, con la que la app comprueba que el archivo descargado procede de quien publica el proyecto antes de instalarlo.",
            en: "Binaries are built automatically by GitHub Actions from this repository's source code and published on GitHub Releases. Each installer also has a “.sig” file: the auto-update signature, which the app uses to check that the downloaded file comes from the project's publisher before installing it.",
          },
          {
            gl: "Os programas que a app descarga por petición túa (FFmpeg e MKVToolNix) compróbanse coa suma SHA-256 que publica a propia fonte antes de instalarse.",
            es: "Los programas que la app descarga a petición tuya (FFmpeg y MKVToolNix) se comprueban con la suma SHA-256 que publica la propia fuente antes de instalarse.",
            en: "Programs the app downloads at your request (FFmpeg and MKVToolNix) are checked against the SHA-256 checksum published by their source before being installed.",
          },
          {
            gl: "Os paquetes de idioma do recoñecemento de texto (OCR) descárganse por HTTPS desde o repositorio oficial de Tesseract en GitHub e só se comproba que o tamaño sexa o dun paquete válido: non levan suma de verificación.",
            es: "Los paquetes de idioma del reconocimiento de texto (OCR) se descargan por HTTPS desde el repositorio oficial de Tesseract en GitHub y solo se comprueba que el tamaño sea el de un paquete válido: no llevan suma de verificación.",
            en: "The text-recognition (OCR) language packs are downloaded over HTTPS from the official Tesseract repository on GitHub, and only their size is checked to be that of a valid package: they have no checksum.",
          },
          {
            gl: "Cada versión publícase como instalador («.exe») e como paquete MSI, ambos con ficheiro de sinatura de actualizacións, e co ficheiro «latest.json» que le a app para buscar versións novas.",
            es: "Cada versión se publica como instalador («.exe») y como paquete MSI, ambos con archivo de firma de actualizaciones, y con el archivo «latest.json» que lee la app para buscar versiones nuevas.",
            en: "Each release is published as an installer (“.exe”) and an MSI package, both with an update-signature file, plus the “latest.json” file the app reads to look for new versions.",
          },
        ],
      },
    ],
  },
};
