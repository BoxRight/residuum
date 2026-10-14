# Actualización conversacional de justificaciones: experimento finito

En `experiment-legal-order-geometry` se continúa el
[experimento de recalcitrancia](legal-recalcitrance-justificatory-functions.md)
sin modificar la norma. Una séptima valoración hipotética reduce las once
funciones compatibles a cinco. Los controles distinguen contradicción entre
valoraciones, incompatibilidad con monotonía, catálogo insuficiente y entrada
fuera del ámbito. No se construyen funciones nuevas.

## 1. Norma, lenguaje y fuentes de las valoraciones

El universo sigue siendo `Σ={O,R,A,P}`, abreviaturas de eventos ground Seeded:

| Símbolo | Hecho |
|---|---|
| O | `obligationFiled` |
| R | `right` |
| A | `Act` |
| P | `permissionFiled` |

Se conservan los argumentos, autoridad, fecha e índice `gTau` del experimento
anterior. Su objetivo fijo es `compliance(...) @ gTau`. No hay datos de un
expediente real. La norma se evalúa con las 27 reglas originales de LegalGeneral:

\[
N(S)=1\iff compliance\in Close_G(S),\qquad \mathcal B_N=\{OA,RA\}.
\]

Se carga el [catálogo anterior de 168 funciones](legal-geometry/recalcitrance/functions.tsv)
sin regenerarlo. Cada función es una anticadena de soportes:

\[
J_\theta(S)=1\iff\exists B\in\theta:\ B\subseteq S.
\]

El harness valida las anticadenas, sus clasificaciones en los 16 estados y el
primer descarte de cada función por las seis valoraciones anteriores. Reproducir
esas valoraciones deja exactamente las mismas once funciones.

Los [casos Formal nuevos](../examples/experiments/legal_geometry/conversation_cases.res)
importan la fixture anterior y aportan únicamente experiments con hechos existentes.
Las [respuestas externas](../examples/experiments/legal_geometry/conversation_valuations.tsv)
se declaran separadamente, con fuente `stipulation:conversation/...`, motivo y
estado `hypothetical_external_assumption`. No las calcula la norma ni representan
una deliberación jurídica efectivamente realizada.

Los [metadatos de las preguntas](../examples/experiments/legal_geometry/conversation_questions.tsv)
registran que todas son representables como inputs Seeded, pero su adecuación
jurídica queda `pending_external_deliberation`. Un caso sin A es evidencia
parcial, no `~Act`. Una respuesta externa 0 tampoco es EvidentialNot.

## 2. Séptima pregunta y actualización

Se calculan ambos grupos de predicciones y el mayor grupo que podría sobrevivir.
El criterio minimax conserva todos los empates:

| Entrada | Predicen 1 | Predicen 0 | Peor supervivencia | Minimax |
|---|---:|---:|---:|---|
| R | 2 | 9 | 9 | No |
| OR | 6 | 5 | 6 | Sí |
| OP | 5 | 6 | 6 | Sí |
| RP | 6 | 5 | 6 | Sí |
| ORP | 10 | 1 | 10 | No |

**MODEL RESULT.** El empate es `{OR,OP,RP}`. El guion externo estipula la
pregunta OP y su respuesta positiva; el test comprueba que pertenece al empate
completo. No se adjudica prioridad entre las preguntas.

Se exige `J(OP)=1`, mientras la norma mantiene `N(OP)=0`. Es subinclusión
según el supuesto externo, no una revisión jurídica aprobada. El filtro aplica:

\[
\mathcal H_7=\{J\in\mathcal H_6:J(OP)=1\}.
\]

| Función superviviente | Soportes mínimos |
|---|---|
| H032 | R, OP, AP |
| H065 | OR, RA, OP, RP, AP |
| H066 | OR, RA, OP, AP |
| H120 | RA, OP, RP, AP |
| H121 | RA, OP, AP |

Se eliminan H033, H068, H070, H123, H126 y H128: exactamente las seis que
predicen 0 en OP. Los [espacios de versiones](legal-geometry/conversation/versions.tsv)
y la [procedencia de las eliminaciones](legal-geometry/conversation/elimination-provenance.tsv)
conservan los identificadores originales.

Las cinco funciones comparten OP y AP como soportes mínimos. Todas incluyen
RA, pero RA no es mínimo de H032, porque R ya basta. La
[tabla de condiciones compartidas](legal-geometry/conversation/shared-conditions.tsv)
separa soportes mínimos comunes de clasificaciones unánimes en los 16 estados.

Como rama contrafactual separada, OP con respuesta negativa deja seis funciones.
No se agrega simultáneamente a la historia positiva.

## 3. Particiones y próximas preguntas

Cada caso S induce una partición por predicciones iguales. La etiqueta externa
selecciona un bloque; no redefine esa partición. En el orden «más fina ≤ más
gruesa»:

\[
\Pi_7=\Pi_6\wedge\pi_{OP}.
\]

**THEOREM del procedimiento.** Coincidir en la firma extendida equivale a
coincidir en la firma anterior y en la nueva predicción. Por tanto el nuevo
núcleo es la intersección de ambas equivalencias, su meet. Asimismo, filtrar
iterativamente equivale a reproducir toda la historia: ambos conservan exactamente
las funciones del catálogo que satisfacen todas las valoraciones recibidas.

**MODEL RESULT.** Sobre las 168 funciones, las seis observaciones inducen
15 bloques y las siete inducen 24. Sobre las once supervivientes anteriores,
OP separa un bloque de cinco y otro de seis. Las cinco retenidas permanecen
indistinguibles según las valoraciones aceptadas.

Las [particiones completas](legal-geometry/conversation/hypothesis-partitions.tsv)
se comparan con un meet calculado independientemente y se verifica refinamiento.
No se enumera la retícula completa de particiones de 168 elementos.

| Próxima entrada | Predicen 1 | Predicen 0 | Informativa | Minimax |
|---|---:|---:|---|---|
| R | 1 | 4 | Sí | No |
| OR | 3 | 2 | Sí | Sí |
| OP | 5 | 0 | No; ya respondida | No |
| RP | 3 | 2 | Sí | Sí |
| ORP | 5 | 0 | No | No |

ORP queda determinada positivamente por las hipótesis, pero no recibió una
valoración externa nueva. OR y RP son las siguientes preguntas óptimas, con
empate conservado. En la rama contrafactual negativa dividen las seis funciones
en grupos 3/3. La [tabla de preguntas](legal-geometry/conversation/questions.tsv)
incluye las tres etapas.

Los [discriminantes mínimos por pareja](legal-geometry/conversation/minimal-discriminants.tsv)
se calculan por inclusión en los 16 estados. Su unión tras el turno es
`{R,OR,RP}`. Minimalidad por pareja y optimalidad minimax para todo el espacio
de versiones son criterios diferentes.

## 4. Diagnósticos de agotamiento

Los controles tienen historias separadas. «Aceptada en ámbito» significa que
la respuesta tipada entra al registro, no que su valoración se apruebe jurídicamente.

| Control | Antes | Después | Diagnóstico |
|---|---:|---:|---|
| Tras el séptimo turno, O con 1 | 5 | 0 | O ⊂ OA, pero la valoración anterior de OA era 0 |
| Tras OP con 1, el mismo OP con 0 | 5 | 0 | Valoraciones opuestas del mismo input tipado y objetivo |
| Catálogo restringido a H033; OP con 1 tras las seis observaciones | 1 | 0 | El catálogo completo existente conserva cinco soluciones |
| Proponer AP junto con `~Act` | 11 | 11, sin filtrar | Hecho fuera de las cuatro características declaradas |
| Proponer OP y el objetivo en `gOtherTau` | 11 | 11, sin filtrar | Otro contexto temporal, fuera del objetivo fijo |

**THEOREM condicionado.** Si J es monotónica y O ⊆ OA, entonces `J(O)=1`
exige `J(OA)=1`. Ninguna anticadena positiva sobre este universo satisface ambas
valoraciones del primer control. Agregar más soportes monotónicos no lo corrige.

El segundo control es incompatible incluso con una función determinista no
monotónica del mismo input y objetivo. Es una contradicción entre valoraciones
externas, diferente de `Act/~Act`, que el motor permite sin explosión.

El tercer control restringe deliberadamente el catálogo a la función ya existente
H033. Se diagnostica mediante replay de las 168 funciones preexistentes; no se
generan funciones ni se reintegran automáticamente las cinco compatibles.

Los dos últimos inputs son válidos para la ontología, pero exceden el lenguaje
de hipótesis declarado. Se rechazan antes de proyectar o filtrar: no se ignora
`~Act`, no se colapsan tiempos ni se convierte un error de ámbito en espacio
vacío. Sus historias y funciones permanecen intactas.

Los [diagnósticos](legal-geometry/conversation/diagnostics.tsv),
[testigos](legal-geometry/conversation/diagnostic-witnesses.tsv) y
[eliminaciones por control](legal-geometry/conversation/diagnostic-elimination-provenance.tsv)
permiten reconstruir las diferencias. Si el espacio queda vacío, no se recomienda
una pregunta discriminante ni se escoge una función «más cercana».

No se decide cuál valoración revisar, si faltan características del mundo real
o si convendría abandonar monotonía. Esas explicaciones requieren deliberación
externa. No se implementa un lenguaje de soportes negativos o excepciones.

## 5. Conservación jurídica y provenance

El [harness](../src/bin/residuum/cli/legal_geometry_conversation_tests.rs) opera
en estructuras de tests separadas del Module y de Known. No sustituye cláusulas
jurídicas ni incorpora etiquetas como hechos. La fixture elaborada conserva
las 27 reglas; los experiments importados no se ejecutan automáticamente.

Se calculan independientemente los 16 cierres originales antes y después de
las actualizaciones, incluida una que agota las hipótesis. Se comparan los
eventos con términos y tipos completos, polaridad y tiempo, y los multiconjuntos
de justificaciones: reglas, programas cuando corresponden, módulos, antecedentes
ordenados y sustituciones, conservando multiplicidad. Se auditan sus orígenes.
Sólo el sello operacional `derivedAt` queda fuera de la comparación. No se
ejecutan Effects sobre State.

**EXPERIMENTAL OBSERVATION.** Todos los cierres y justificaciones coinciden.
La [invariancia normativa](legal-geometry/conversation/normative-invariance.tsv),
la [provenance jurídica](legal-geometry/conversation/legal-provenance.tsv) y
la [procedencia de las observaciones](legal-geometry/conversation/observation-provenance.tsv)
registran inputs, objetivos, fuentes y módulos. La razón de descarte de una
función se distingue de una derivación jurídica.

## 6. Alcance y reproducción

| Afirmación | Alcance |
|---|---|
| Filtrado = replay; actualización de particiones por meet | Argumento general del procedimiento sobre un catálogo fijo |
| Minimax, supervivientes, particiones y diagnósticos concretos | MODEL RESULT: cuatro características, 16 estados, 168 funciones |
| Conservación de norma y justificaciones | Comparación experimental de 16 cierres originales |
| Identificación de una justificación jurídica verdadera | No establecida; ninguna función se declara verdadera |
| Ampliación del lenguaje de hipótesis | Etapa posterior, no implementada |

Se implementó una reproducción de turnos en el harness, no una conversación
interactiva en la CLI. Pasaron tres tests individuales mediante `test_safe.py`:
actualización/particiones/preguntas, diagnósticos y conservación jurídica.
No se ejecutó la suite completa.

La [verificación](legal-geometry/conversation/verification.md) registra comandos,
tiempos, memoria y logs. El [registro SHA-256](legal-geometry/conversation/input-sha256.txt)
fija fuentes y entradas. Se mantiene 1 GiB total, sin swap, un trabajo de Cargo
y un hilo. No hubo merge ni cambios a `main` o `v1`.

Esta etapa demuestra selección explicable y detección de incompatibilidad
dentro de un lenguaje fijo. Construir funciones fuera de él sigue siendo una
investigación separada.
