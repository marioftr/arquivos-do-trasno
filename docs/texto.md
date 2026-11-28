# Texto da interface

Regras que cumpre todo texto visible. `npm run check:copy` comproba as que se poden comprobar; `npm run list:info` lista todas as «i» para revisalas xuntas.

| Regra | Exemplo |
|---|---|
| Sen recheo: nada que o usuario xa dea por sabido. | Non «O orixinal non se toca», «Crea un ficheiro novo». |
| O trasno fala en primeira persoa nas mensaxes, avisos e erros. O historial de accións (`noteAction`) e Sobre o trasno van en terceira. | «Non puiden ler o PDF», non «O trasno non puido…». |
| Subtítulos, nunca «lexendas». | «Resincronizar os subtítulos». |
| Unha «i» (e todo texto explicativo: `hint`, `info`, `GroupNote`, `LockNote`) é impersoal e profesional, NUNCA en primeira persoa: nada de «uso», «busco», «convirto», «só sei». | «Convértese a PNG», non «Convértoo a PNG». |
| A explicación vai nunha «i» ao carón do que explica: nin hover (`title`) nin liñas soltas. O hover só vale para iconas sen texto. | Selector con `hint` en cada opción: a «i» do título lista unha por liña. |
| Unha «i» vai ao grano: máximo 140 caracteres, funcional, sen repetir a etiqueta. Se a etiqueta xa o di, non leva «i». | «En puntos; 12 pt é o texto normal dun documento.» |
| Todo selector con perfís ten «Personalizado», sen texto de máis. | Nivel de calidade: WhatsApp, Web, Arquivo, Personalizado (os mesmos nomes de uso en imaxes e vídeo). |
| Os botóns dos niveis levan o valor: CRF no vídeo, % nas imaxes. | «Web · CRF 28», «Web · 75 %». |
| Os botóns de cálculo son curtos, á esquerda, na fila `.estimate`. | «Estimar tamaño», «Analizar silencios». |
| Os datos van ordenados (táboa), non en prosa. | «Así quedará o ficheiro». |
| O fin dun traballo lese en voz alta aos lectores de pantalla coa mesma mensaxe do trasno (rexión viva en `App.svelte`). Os comentarios de compañía non. | `trasnoSay(…, { quiet: true })`. |
| Un selector pode levar un debuxo (`glyph`) enriba do nome só se o debuxo di algo que o texto non di: forma, orde, sentido, cor. Nunca decorativo. | Proporcións do recorte (rectángulo coa forma), numerar por filas ou columnas, ángulo das páxinas, mostra de cor. |
| Se un ficheiro de saída xa existe, o « (2)» vai sempre ao final do nome, antes da extensión; nunca no medio. | `foto-1 (2).png`, `uno-redimensionado (2).png`. |
| A etiqueta dun interruptor é curta; o detalle vai na «i». O que non se pode usar queda en gris, sen texto que o explique. | «Manter metadatos» (non «Manter os metadatos da foto»), «Non ampliar», «Manter GPS». |
| Os campos de número levan frechas á dereita e non admiten valores fóra dos seus límites (nin negativos se non valen). | Ancho, alto, columnas, filas. |
| O botón verde principal non leva icona: só o texto. Vale para todos os menús. | «Xuntar 3 imaxes», «Unir nun só PDF». |
| O que depende dun interruptor de arriba aparece só cando vale; o que está bloqueado por outro motivo queda en gris. | «Recortar polo centro / Estirar» só sen «Manter proporción». |
| Unha opción de «cortar» ou «encaixar» leva un debuxo que a explique (o que se corta, o que se deforma, as marxes). | Recortar, Estirar, Encaixar. |
| Un debuxo de «igualar medidas» amosa o resultado: as pezas escalan sen cortarse nin deformarse (contorno a trazos = tamaño de antes, frecha = sentido). | Á máis estreita, Á máis ancha. |
| Unha comparación que só vale mentres se mantén premido é un botón curto na fila `.estimate`, enriba do botón verde, ; non un interruptor. | «Ver orixinal». |
| Os botóns con debuxo (e os selectores con debuxo) reparten o ancho a partes iguais. | Recortar polo centro / Estirar: 50 % cada un. |
| Unha ferramenta dunha soa tarefa non leva cabeceira nin liña de descrición soltas: a descrición vai na «i» ao carón do título do panel (`PanelInfo`). Só os paneis con varios grupos levan cabeceiras. | «Completar (i)», «Tapar zonas (i)». |
| Os nomes de función van sen artigo, coma unha orde breve, onde non quede mal. | «Quitar son», «Igualar volume», «Gardar copia…», «Márxenes iguais». |
| Se unha ferramenta precisa unha extensión (programa ou modelo), no seu sitio sae só o aviso (`ExtensionGate`): «Falta instalar:», cada cousa co seu nome, para que serve e o tamaño, e un botón «Descargar e instalar · tamaño total». Nunca se descarga sen premelo. | Transcribir, Quitar fondo, Descargar por ligazón. |
| Nin as «i» nin os textos das ferramentas din canto tarda algo nin que se fai no propio equipo: nesta app todo é local e xa se dá por sabido. | Non «Tarda uns segundos», non «…no propio equipo». |
| Axustes → Extensións: todas as tarxetas igual: nome, estado («Instalado» ou «Sen instalar», nunca «Descargado»), para que serve, «Desbloquea», e unha liña con tamaño · licenza · ligazón ao proxecto. Os modelos levan o seu nome (IS-Net, Whisper small), nunca «Modelo para…». | IS-Net, whisper.cpp, yt-dlp. |
| O fondo para compoñer unha imaxe con transparencia é sempre o mesmo selector (`BackgroundPicker`): botóns grandes con debuxo (Transparente se vale, Branco, Negro, Gris, Personalizado) e, en Personalizado, o selector de cor. O fondo vese no visor en directo. | Converter, Editar, Quitar fondo. |
| Unha función con xanela propia (Descargar por ligazón) leva o trasno no cabezallo, é estreita, pregunta onde gardar ao premer o botón verde e, ao empezar, péchase e o traballo segue nos procesos. | Descargar por ligazón. |
| Cada «i» explica que fai o axuste e a que afecta (que se comprime, canto), non repite a etiqueta. | Comprimir PDF: só se reducen as imaxes JPG. |
| Unha vista previa que custa traballo (Quitar fondo) non sae soa: botón curto «Previsualizar» sobre o verde, cos axustes sempre dispoñibles. Un fallo di no trasno que falla e por que, nunca en liña amarela solta nin cun «Cancelei o traballo» sen máis. | Quitar fondo. |
| Todo o texto chega a 4,5:1 de contraste (3:1 se é grande) en tema escuro e claro; o texto atenuado dentro dun botón verde herda a súa cor. Proba `contraste`. | Toda a app. |
| Se unha función precisa dúas descargas, o aviso di para que serve cada unha (programa que fai o traballo, modelo que recoñece a voz). Transcribir só pasa voz a texto, non texto a voz. | Transcribir. |
