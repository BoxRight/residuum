# Identificabilidad mediante casos discriminantes

Investigación experimental en `experiment-legal-order-geometry`, sobre las hipótesis
ya construidas. No se generan nuevas revisiones ni se responde a un caso jurídico.
No se modifica el motor. Continúa el
[análisis de dependencias](legal-revision-dependencies-and-justification-boundary.md).

La pregunta operacional es qué evidencia separa dos hipótesis compatibles con las
mismas observaciones. Identificar un comportamiento dentro de esta familia no
identifica por sí mismo una justificación normativa desconocida.

## Espacio de hipótesis y observación

Se conservan las nueve hipótesis anteriores:

- Las cinco extensiones mínimas: AP, AD+AM, AD+APM, APD+AM, APD+APM.
- Las cuatro variantes con controles AV o APV, cargados de `feedback_completions.res`.

Cada nombre indica qué soportes ground producen C. Por ejemplo AD+AM contiene
las reglas hipotéticas AD→C y AM→C; AD+AM+AV agrega el control AV→C.
No se añaden estos paquetes a la ontología real. Los nueve pasan las mismas
13 etiquetas externas ya declaradas. No hay una etiqueta adicional asignada aquí.

Las abreviaturas conservan los eventos ground de los informes anteriores:

| Símbolo | Evento |
|---|---|
| A | `Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau` |
| P | `permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau` |
| M | `omissionProhibition(...)` con los argumentos de P |
| D | `disputedPerformance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau` |
| V | `violation(...)` con los argumentos de D |
| C | `compliance(...)` con los argumentos de D; conclusión observada |

O y R, presentes en el universo, siguen siendo `obligationFiled` y `right`.
Todos los eventos de esta ventana tienen polaridad positiva. D y V son premisas
Derived de instantáneas de teoría, no seeds ni afirmaciones de hechos históricos.

Se utiliza el mismo universo cerrado E' de 12 eventos y sus 4.096 subconjuntos.
También se estudia separadamente el protocolo que sólo admite los cinco eventos
Seeded de E': A, O, R, P y M. No se supone que toda conclusión Derived pueda
obtenerse o evaluarse independientemente en un expediente real.

## Búsqueda por inclusión, sin distancia

Para cada una de las 36 parejas de hipótesis distintas se calcula:

\[
\mathcal D_{ij}=\{S\subseteq E':C_{H_i}(S)\ne C_{H_j}(S)\},
\]

\[
\min_{\subseteq}\mathcal D_{ij}
=\{S\in\mathcal D_{ij}:\nexists T\subsetneq S,\ T\in\mathcal D_{ij}\}.
\]

Se conservan todos los mínimos; ni el número de hechos ni su orden en un archivo
establecen una preferencia. Los identificadores numéricos de casos son máscaras
para reproducir conjuntos, no distancias ni puntuaciones jurídicas.

La diferencia de dos cierres monótonos **no es necesariamente ascendente**.
Por ejemplo, AP distingue la hipótesis AP de AD+AM, pero al agregar C como premisa
ambas producen el mismo cierre. Por tanto no se extiende un caso discriminante a
todo su cono ni se verifica minimalidad sólo mirando discrepancias en sus vecinos.

El algoritmo propaga si hay algún testigo dentro de cada subconjunto y excluye
cualquier candidato que tenga un testigo propio inferior. El resultado se contrasta
con otra enumeración que compara todos los subconjuntos propios. Se comprueba que
cada estado discrepante contiene al menos uno de los mínimos calculados.

**MODEL RESULT:** los 36 pares tienen algún caso discriminante. Se obtienen
70 incidencias pareja/caso mínimo y cinco conjuntos distintos al unir los resultados.
Cada conjunto es mínimo para las parejas que lo reportan; la unión no es una
antichain global y no debe podarse indiscriminadamente.

Un ejemplo muestra por qué inclusión no equivale a menor cardinalidad:

| Pareja | Mínimos discriminantes |
|---|---|
| APD+AM frente a APD+APM+APV | AM, APV |

AM tiene dos hechos y APV tres. Son incomparables por inclusión y ambos se
conservan. No se descarta APV por ser más grande.

## Casos informativos y predicciones pendientes

[identifiability_cases.res](../examples/experiments/legal_geometry/identifiability_cases.res)
contiene las cinco instantáneas completamente tipadas. No contiene respuestas
externas. El test comprueba que la fixture coincide con los casos que encontró
la enumeración. También los exporta a `target/legal-geometry/pending-discriminating-cases.res`.

| Caso | Evidencia | Identificador reproducible |
|---|---|---|
| AD | Act + disputedPerformance | case_0005 |
| AM | Act + omissionProhibition | case_0065 |
| AP | Act + permissionFiled | case_0513 |
| AV | Act + violation | case_2049 |
| APV | Act + permissionFiled + violation | case_2561 |

El frontend exige `premises` con `query fourFusion`; se mantiene esa sintaxis.
El harness utiliza input y goal elaborados para comparar cierres Horn, **no
entailment FOUR**. Una ejecución aislada de FOUR en la CLI no sustituye este
experimento de nueve módulos hipotéticos.

Estas son las predicciones de derivabilidad de C, no evaluaciones jurídicas:

| Hipótesis | AP | AM | AD | AV | APV |
|---|---|---|---|---|---|
| AP | sí | no | no | no | sí |
| AM+AD | no | sí | sí | no | no |
| AM+AD+AV | no | sí | sí | sí | sí |
| APM+AD | no | no | sí | no | no |
| APM+AD+AV | no | no | sí | sí | sí |
| AM+APD | no | sí | no | no | no |
| AM+APD+APV | no | sí | no | no | sí |
| APM+APD | no | no | no | no | no |
| APM+APD+APV | no | no | no | no | sí |

**MODEL RESULT:** las nueve firmas de predicciones son diferentes. Si se pudieran
valorar independientemente estos cinco casos y las respuestas fueran compatibles
con alguna hipótesis de la familia, quedaría una sola clase de comportamiento de
esta familia finita. No se afirma que hagan falta exactamente cinco preguntas:
no se optimiza el número de consultas ni se impone una estrategia adaptativa.

En estos programas, comparar C basta para comparar el cierre lógico completo.
Se comprueba esa coincidencia para los 4.096 estados y las 36 parejas, no se
presupone para otros programas. Aquí las reglas que varían sólo añaden C; las
reglas posteriores son comunes. El resultado no convierte toda conclusión
jurídica en una observación intercambiable con cualquier otra.

El pool incluye AV y APV, aunque AV⊂APV: para AD+AM y AD+AM+AV basta AV,
pero para APD+AM y APD+AM+APV el mínimo es APV. Podar APV globalmente por tener
un subconjunto en el pool dejaría esa última pareja sin su caso discriminante.

## Actualización condicionada a una valoración externa

La pregunta a la persona que valora el caso es si C debe seguirse jurídicamente
de esa evidencia, bajo los supuestos del expediente. Debe mantener separadas esa
valoración y la afirmación de que las premisas son admisibles o efectivamente conocidas.
El valor calculado por una hipótesis nunca proporciona su propia etiqueta verdadera.

Dada una respuesta externa y para un caso S, la actualización propuesta en el
harness es exclusivamente:

\[
\mathcal V_y(S)=\{H\in\mathcal V:\mathbf1[C\in C_H(S)]=y\}.
\]

`identifiability-predictions.tsv` muestra ambas ramas, positiva y negativa, para
cada caso. Todos los campos `external_answer` siguen como **pending**. Ningún
candidato se descarta ni se integra: el espacio de hipótesis real permanece en nueve.

Se enumeran sólo como control las 32 combinaciones posibles de respuestas a los
cinco casos, en el orden AD, AM, AP, AV, APV del artefacto:

- Nueve patrones dejarían una hipótesis de esta familia.
- Veintitrés patrones dejarían el espacio vacío.

No son etiquetas suministradas ni casos nuevos. Si las respuestas reales agotaran
el espacio, se informaría que la familia no las representa; no se elegiría un
candidato arbitrariamente ni se modificarían las etiquetas para salvarlo.

## Límite de acceso a los casos: sólo seeds

Al limitar los inputs a los cinco hechos Seeded de E', hay 32 casos posibles.
Las nueve hipótesis se agrupan en **tres clases de cierre**, de tamaños 1, 4 y 4:

| Clase observable con seeds de E' | Hipótesis |
|---|---|
| AP | AP |
| AM | AD+AM; AD+AM+AV; APD+AM; APD+AM+APV |
| APM | AD+APM; AD+APM+AV; APD+APM; APD+APM+APV |

En ese protocolo no se puede distinguir, por eventos derivados, AD de APD ni
los controles AV/APV: D y V no se aportan y no se producen en esa ventana.
No se extrapola a nuevos seeds fuera de E', como `~Act` o `prohibition`, que aquí
no se enumeran y podrían habilitar V. Tampoco se supone que una persona pueda
valorar D o V independientemente sólo porque el tipo permite una premisa Derived.

La accesibilidad de un caso discriminante importa tanto como su existencia formal.

La [continuación sobre observabilidad](legal-identifiability-observability.md)
clasifica los cinco casos, incorpora `prohibition` y `~Act` como entradas ya
disponibles y separa las tres clases anteriores en siete. Distingue equivalencia
de eventos, equivalencia de provenance y un argumento general condicionado para
las dos parejas que conservan las mismas consecuencias desde seeds.
Los cinco casos de instantáneas de teoría dan nueve firmas; restringir los casos
a seeds da tres. Son protocolos observacionales distintos sobre la misma familia.

## Equivalencia e identificabilidad de justificaciones

Como control ya conocido, AP y la presentación redundante AP+APD tienen el mismo
operador sobre E'. No existe un caso discriminante de cierre entre ellas en este
universo. Su provenance puede diferir: AP+APD tiene una regla adicional que puede
justificar el mismo evento. Esa diferencia de implementación no es una valoración
jurídica independiente y no determina una razón normativa.

**THEOREM, condicional:** si dos hipótesis producen el mismo observable para
cada caso admisible, ningún protocolo que consulte sólo ese observable y esos
casos puede distinguirlas. Para una familia finita y un dominio finito, dos
operadores distintos tienen un conjunto no vacío de discrepancias y éste tiene
algún mínimo por inclusión. Estas afirmaciones no garantizan que el mínimo sea
un caso jurídico realizable.

**MODEL RESULT:** separación de nueve comportamientos en E', tres clases bajo
inputs Seeded y equivalencia de AP/AP+APD. Se verifica minimalidad y cobertura de
todos los testigos dentro de E'.

**IMPLEMENTED + TESTED:** generación y tipado de casos informativos, comparación
de predicciones, particiones condicionadas a respuestas y conservación de cierres
y provenance. No hay nueva sintaxis ni API de producción.

No se ha representado una razón normativa independiente J ni se ha demostrado un
puente que permita identificarla con una de estas reglas o familias de soportes.
La investigación acerca el siguiente paso del aprendizaje activo simbólico:
proponer casos informativos y recibir valoración externa. Identificar el candidato
operacional restante, si llegara a ocurrir, seguiría sin demostrar que es la
justificación subyacente de la norma.

## Verificación y fuentes

[Harness](../src/bin/residuum/cli/legal_geometry_identifiability_tests.rs).
[Tablas y registros](legal-geometry/identifiability/):

- `identifiability-pairs.tsv`: 36 parejas y todos sus mínimos.
- `identifiability-minimal-cases.tsv`: predicciones y diferencia exacta de eventos.
- `identifiability-predictions.tsv`: ramas posibles y respuesta externa pendiente.
- `identifiability-engine-closures.tsv`, `identifiability-provenance.tsv`.
- `identifiability-counterfactual-answer-patterns.tsv`.
- `identifiability-seeded-observation-classes.tsv`, `verification.tsv`.

Se reproducen 45 cierres —nueve hipótesis por cinco casos— con el motor incremental
y el cierre de referencia, independientemente. Coinciden eventos completos y listas
de justificaciones, incluyendo reglas, argumentos, tipos, tiempos, antecedentes
ordenados, sustituciones, multiplicidad y módulos de origen. Se omite sólo
`derivedAt` al comparar ejecuciones. Cada registro se audita contra la regla y sus
testigos. Módulos originales e inputs permanecen intactos; no se ejecuta State.

Compilación y tests individuales sólo mediante `python3 scripts/test_safe.py`,
1 GiB, sin swap, un trabajo de Cargo y un hilo. No se corre la suite completa ni
se modifica el presupuesto. El runner y la prueba de regresión de dependencias
se registran en `verification.tsv`.

La ontología original y sus dos módulos vendor mantienen sus SHA-256. Main y v1
no cambian. No hay commit, merge, integración de revisiones ni push en esta etapa.
