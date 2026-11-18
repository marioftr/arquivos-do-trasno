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
| Todo selector con perfís ten «Personalizado», sen texto de máis. | Nivel de calidade: Alta, Equilibrada, Forte, Personalizado. |
| Os botóns dos niveis levan o valor: CRF no vídeo, % nas imaxes. | «Equilibrada · CRF 28», «Web · 75 %». |
| Os botóns de cálculo son curtos, á esquerda, na fila `.estimate`. | «Estimar o tamaño», «Analizar silencios». |
| Os datos van ordenados (táboa), non en prosa. | «Así quedará o ficheiro». |
| O fin dun traballo lese en voz alta aos lectores de pantalla coa mesma mensaxe do trasno (rexión viva en `App.svelte`). Os comentarios de compañía non. | `trasnoSay(…, { quiet: true })`. |
