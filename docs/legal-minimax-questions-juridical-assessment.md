# Evaluación jurídica de las dos preguntas minimax

Continuación documental del [experimento conversacional](legal-conversational-justification-learning.md),
en `experiment-legal-order-geometry`. No se añaden valoraciones, hipótesis, hechos
o reglas. Se examina si OR y RP son configuraciones coherentes y qué objeción
podrían plantear. Las cinco funciones permanecen en el espacio de versiones.

**Conclusión:** ambas combinaciones admiten una lectura jurídica coherente, pero
ninguna contiene evidencia de una prestación realizada. Su valor argumentativo
consiste en cuestionar qué se está clasificando: la existencia de posiciones
normativas, una razón para valorar favorablemente el caso o el cumplimiento
de una prestación. El empate minimax no resuelve esa diferencia.

## 1. Qué dicen exactamente los casos

Los casos están en [conversation_cases.res](../examples/experiments/legal_geometry/conversation_cases.res).
O, R y P comparten todos estos argumentos:

```text
gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate
```

O es `obligationFiled`, R es `right` y P es `permissionFiled`. Los tres tienen
índice proposicional `gTau`. La fecha de vigencia es `gDate`; no se confunde
con ese índice ni con `derivedAt`. El objetivo contiene además `gAct`:

```text
compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
```

Ni OR ni RP aportan `Act(gAct,...)`, ni tampoco `~Act(gAct,...)`. El identificador
gAct sólo aparece en el objetivo. La consulta no acredita que un acto histórico
haya ocurrido o dejado de ocurrir.

Los hechos no describen dos negocios distintos, obligaciones recíprocas entre
partes intercambiadas ni documentos de autoridades diferentes. Comparten gNorm
y la misma dirección de la prestación. Fact es opaco: compartir un identificador
no prueba por sí solo cómo habría que individualizar la fuente jurídica real.

En la biblioteca importada:

```text
obligationFiled(...) -> obligation(...)
permissionFiled(...) -> permission(...)
obligation(...) & Act(...) -> compliance(...)
right(...) & Act(...) -> compliance(...)
```

No existe un puente automático `right -> obligation`. Tampoco existe uno desde
derecho, obligación, permiso o ejecución de do hacia Act.
Los [cierres ya certificados](legal-geometry/conversation/normative-invariance.tsv)
dan `N(OR)=0` y `N(RP)=0`: compliance no se deriva. Esto expresa falta de prueba
en el expediente, no una constatación histórica de incumplimiento.

## 2. Lectura de derecho, obligación y permiso

**Inferencia interpretativa a partir del código:** right funciona aquí como
una pretensión relativa a una prestación. Sus campos subject/conduct/object/
beneficiary coinciden con los del Act que puede satisfacerla; rightEnforceability
usa esa misma correspondencia con ~Act. La lectura más consistente asigna al
subject el papel de quien realiza la prestación y al beneficiary el de su
beneficiario. No se debe convertir a subject en titular de una libertad de
actuar sólo porque el verbo se llama right.

Esta lectura tiene una comparación doctrinal útil: Hohfeld distingue la
pretensión, correlativa del deber de otro, de la libertad o privilegio. También
explica que el deber de realizar una conducta puede coexistir con la libertad
de realizarla: esta última se opone al deber de abstenerse, no al deber de hacerla.
[Hohfeld, texto de 1913, pp. 30–33](https://persweb.wabash.edu/facstaff/helmang/phi213-1314S/phi213-txtbrwsr/Hohfeld/Hohfeld1913.html).

La comparación no incorpora esas equivalencias al motor. permissionFiled es
un registro positivo del expediente; no implementa por sí mismo todas las
propiedades de una libertad hohfeldiana. La evaluación jurídica exige especificar
qué posición se afirma y de quién, manteniendo los papeles del caso existente.

## 3. OR: obligación y pretensión sobre la misma prestación

Lectura condicional: gSubject debe realizar gConduct respecto de gObject a favor
de gBeneficiary, y este último tiene una pretensión a esa prestación.
La configuración es coherente si son posiciones correlativas de la misma relación.
En esa lectura, OR puede registrar sus dos perspectivas; no demuestra dos razones
normativas independientes ni dos prestaciones cumplidas.

La objeción relevante puede formularse así:

> Se aporta un deber y la pretensión correlativa del beneficiario, pero no evidencia
> del acto. ¿Qué razón permitiría pasar de esa relación normativa a considerar
> cumplida la prestación identificada por gAct? ¿Registrar ambos lados de la relación
> cambia algo respecto de registrar sólo uno?

Esta pregunta comprueba si una función trata la duplicación de posiciones como
evidencia suficiente. Una respuesta positiva necesita una razón independiente
que explique ese salto; la cantidad de hechos aportados no lo justifica.

Además, si la deliberación adopta una identificación jurídica fuerte entre
pretensión y deber correlativo, habrá que examinar las valoraciones previas
`J(OA)=0` y `J(RA)=1`. Podrían reflejar una distinción documental relevante o una
incoherencia de la interpretación. El AST permite entradas diferentes y no decide
esa cuestión. No se identifican automáticamente ambos estados de conocimiento.

## 4. RP: pretensión del beneficiario y permiso para la prestación

Lectura condicional: el beneficiario tiene una pretensión a gConduct, mientras
el sujeto de la prestación tiene un permiso registrado para realizarla.
No es necesariamente «la misma persona tiene derecho y permiso para hacer algo»:
esa traducción invertiría los papeles que utiliza rightCompliance.

La combinación puede ser coherente si permiso y pretensión se refieren a la
misma conducta positiva. No debe leerse como permiso para omitirla: la fixture
no incorpora una conducta opuesta ni una polaridad negativa del permiso.
El permiso afirma una posición normativa, no que se haya ejercido.

La pregunta sustantiva sería:

> Se aporta una pretensión a una prestación y un permiso para realizarla, sin
> acreditar el acto. ¿El permiso aporta una razón para considerar cumplida esa
> prestación, o sólo responde a si su realización estaba permitida? ¿Cuál sería
> la conexión entre la habilitación y la satisfacción efectiva de la pretensión?

La comparación OR/RP distingue el carácter debido del carácter permitido de
la conducta en los registros aportados. No permite sustituir sin más una
obligación por un permiso ni declarar inexistente el deber cuando O no se aporta.

## 5. Existencia de la relación y cumplimiento

Como referencia delimitada, el artículo 2062 del Código Civil Federal vincula
el cumplimiento con la entrega de lo debido o la prestación del servicio.
Esto ayuda a distinguir la posición normativa de su satisfacción material.
[Código Civil Federal, art. 2062, página 200 del PDF oficial](https://www.diputados.gob.mx/LeyesBiblio/pdf/CCF.pdf#page=200).

No se afirma que ese precepto rija automáticamente estas fixtures: gConduct
es genérica y administrative no determina un régimen jurídico concreto.
La referencia civil no es una valoración de OR o RP ni una norma nueva del
experimento. El requisito de Act observado procede de las reglas existentes.

Si J se pretende leer como «hay cumplimiento acreditado de esta prestación»,
ambas preguntas detectan un problema probatorio: faltan datos de realización.
Si J expresa «hay una razón normativa para valorar favorablemente el caso»,
la conversación debe decir cuál es esa razón y por qué no equivale a probar
compliance. Esa distinción puede investigarse sin cambiar todavía el lenguaje.

También sigue pendiente la séptima estipulación `J(OP)=1` sin Act. Responder
negativamente a OR y RP no eliminaría ese supuesto ni convertiría automáticamente
a la función restante en una justificación adecuada del cumplimiento.

## 6. Efecto explicable de respuestas futuras

La siguiente tabla se calcula a partir de los soportes de las cinco funciones
ya conservadas. Son ramas contrafactuales, no nuevas valoraciones aceptadas:

| Función | J(OR) | J(RP) |
|---|---:|---:|
| H032 | 1 | 1 |
| H065 | 1 | 1 |
| H066 | 1 | 0 |
| H120 | 0 | 1 |
| H121 | 0 | 0 |

OR con respuesta 1 conservaría H032/H065/H066; con 0, H120/H121.
RP con respuesta 1 conservaría H032/H065/H120; con 0, H066/H121.
Ambas preguntas tienen el mismo reparto 3/2, pero separan funciones distintas.

Si se valoraran las dos, las respuestas conjuntas producirían cuatro bloques:

| OR | RP | Compatibles |
|---:|---:|---|
| 1 | 1 | H032, H065 |
| 1 | 0 | H066 |
| 0 | 1 | H120 |
| 0 | 0 | H121 |

Las respuestas positivas conjuntas aún dejarían una indeterminación, separable
por R. Las negativas conjuntas identificarían H121 dentro del catálogo, pero
esa identificabilidad finita no prueba adecuación jurídica. Ninguna de estas
ramas se aplica al espacio de versiones actual.

## 7. Resultado de la evaluación

- **Coherencia condicional:** OR y RP admiten configuraciones de posiciones
  normativas compatibles, conservando sujeto, beneficiario y prestación.
- **Relevancia argumentativa:** permiten cuestionar la confusión entre relación
  normativa, habilitación y cumplimiento acreditado.
- **Interpretación aún abierta:** es necesario fijar qué clasifica J, el sentido
  jurídico concreto de right/permission y la razón de una valoración externa.
- **Resultado no establecido:** no hay todavía respuesta deliberada independiente
  ni demostración de que una pregunta produjo una razón normativa sustantiva.

El resultado de esta etapa es una formulación jurídicamente controlada de las
preguntas y de sus posibles efectos sobre las hipótesis. El siguiente turno
deberá registrar el motivo de la respuesta, además de su valor binario. Una
objeción al significado del caso no se convierte automáticamente en etiqueta 0.

Sólo se añadió documentación. Se conservaron fixtures, etiquetas, funciones,
motor, ontología, main y v1. No se compilaron ni ejecutaron nuevos tests; la
[verificación anterior](legal-geometry/conversation/verification.md) sigue siendo
el respaldo computacional de los resultados utilizados. Sin commit ni merge.
