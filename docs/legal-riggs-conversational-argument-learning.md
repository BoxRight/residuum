# Aprendizaje conversacional de argumentos sobre la referencia fija de Riggs

Experimento en `experiment-legal-order-geometry`. Integra el
[aprendizaje conversacional anterior](legal-conversational-justification-learning.md)
con el [modelo argumentativo de Riggs](legal-riggs-fixed-reference-experiment.md).
Se conserva íntegra la teoría sucesoria y se reutilizan sus seis expedientes.

**Resultado finito:** una valoración hipotética sobre el homicidio sin finalidad
adquisitiva actualiza las tres lecturas existentes. Sostener la objeción conserva
la lectura causal; rechazarla conserva la lectura con finalidad y la ruta formal;
suspender juicio conserva las tres. Después del rechazo, el expediente histórico
permite distinguir las dos supervivientes mediante otra valoración hipotética.

## 1. Contratos inspeccionados y componentes reutilizados

El experimento anterior clasificaba estados mediante funciones booleanas de
anticadenas. Este experimento conserva exactamente el catálogo de
[`riggs_arguments.tsv`](../examples/experiments/legal_geometry/riggs_arguments.tsv):

| Identificador | Condición de premisas existente | Papel argumentativo |
|---|---|---|
| EarlCausalWrong | linked_wrong | Objeción basada en vínculo causal |
| EarlAcquisitionPurpose | purposeful_acquisition | Lectura que exige finalidad adquisitiva |
| GrayStatutoryRoute | literal_route | Defensa de la ruta formal |

Se reutiliza sin cambios el evaluador de premisas de Riggs. El nuevo verificador
es un submódulo de tests que accede a ese evaluador, a sus auditorías y a los
expedientes originales. No se copian condiciones ni se modifican sus estados.

Del aprendizaje anterior se reutilizan directamente las funciones `kernel`,
`meet` y `finer`. Sólo se amplía su visibilidad entre módulos de tests. Se
reutiliza el procedimiento de filtrado de versiones y replay, adaptando el
contrato de compatibilidad a respuestas parciales. Minimax conserva todos los
empates y contabiliza explícitamente las hipótesis pendientes.

No se reutilizan las 168 funciones ni las seis etiquetas binarias anteriores.
Tampoco sus revisiones hipotéticas de productores de compliance ni el diagnóstico
basado en monotonía de aquellas funciones. Las tres hipótesis actuales son
lecturas argumentativas y su respuesta jurídica no está definida por una
clasificación de compliance.

## 2. Tres niveles separados

Para cada expediente S se calcula antes de conversar:

\[
X_T(S)=(S,Close_T(S),D_T(S)).
\]

T incluye las 27 reglas originales de LegalGeneral y las cuatro reglas de la
referencia sucesoria. D conserva antecedentes ordenados, sustituciones, tiempos,
reglas y módulos. Las objeciones documentadas, las fuentes de los argumentos y
los orígenes de las premisas se cargan desde las tablas existentes.

**Premisas.** El evaluador existente devuelve
`Compatible`, `Incompatible` o `Indeterminate`. Esta evaluación conserva su
significado factual anterior. Por sí sola no elimina una lectura del catálogo.

**Valoración externa.** El interlocutor hipotético responde a una pregunta común:

> ¿Las relaciones representadas fundamentan sostener una objeción a la atribución
> testamentaria literal?

Las respuestas son `sustain`, `reject` y `pending`. Su origen y fundamento quedan
en [`riggs_conversation_valuations.tsv`](../examples/experiments/legal_geometry/riggs_conversation_valuations.tsv),
con estado `hypothetical_external_assumption`. Todas las respuestas de esta etapa
son estipulaciones experimentales, incluidas las que examinan el expediente histórico.

**Permanencia.** Se compara la respuesta externa con el compromiso interpretativo
de la hipótesis, sin cambiar sus premisas ni el cierre jurídico.

## 3. Adaptación explícita: compromisos interpretativos

El antiguo evaluador de premisas no determina automáticamente si una razón
merece aceptación jurídica. Para poder contrastar lecturas ante la misma pregunta,
se declaran compromisos previos en
[`riggs_conversation_contract.tsv`](../examples/experiments/legal_geometry/riggs_conversation_contract.tsv):

| Lectura | Premisas Compatible | Premisas Incompatible | Premisas Indeterminate |
|---|---|---|---|
| Causal | sustain | reject | Ambas respuestas posibles |
| Con finalidad | sustain | reject | Ambas respuestas posibles |
| Ruta formal | reject | Ambas respuestas posibles | Ambas respuestas posibles |

Estos compromisos son interpretaciones hipotéticas delimitadas de las tres
lecturas existentes. Su fuente es `stipulation:riggs-conversation/...`, separada
de la fuente histórica del argumento. En particular, disponer de una prueba
literal no demuestra jurídicamente que deba rechazarse una objeción: esa respuesta
pertenece al compromiso experimental de la lectura formal.

Del mismo modo, `reject` cuando una premisa está explícitamente refutada significa
que esa lectura no sostiene la objeción **por los fundamentos representados**.
No afirma que cualquier otro argumento jurídico sea imposible. Las lecturas se
tratan como explicaciones completas únicamente de esta pregunta delimitada.

El catálogo sigue teniendo los mismos tres identificadores y condiciones. No se
agregan soportes, nuevos tipos de razones, reglas jurídicas ni condiciones al
lenguaje de hipótesis. Se hace explícita la correspondencia necesaria para
comparar estas lecturas con valoraciones externas.

Sea P_h(S) el conjunto de respuestas que admite una lectura:

\[
P_h(S)\in\{\{+\},\{-\},\{+,-\}\}.
\]

`Indeterminate` produce siempre `{+,-}`. Para una valoración y:

\[
\mathcal H_{n+1}=\{h\in\mathcal H_n:
y=\text{pending}\ \lor\ y\in P_h(S)\}.
\]

Una premisa incompatible puede permanecer si la lectura responde correctamente
que no sostiene la objeción en ese caso. Una premisa compatible puede acompañar
una eliminación si la valoración externa rechaza la respuesta que propone esa
lectura. La falta de evidencia siempre conserva la hipótesis pendiente.

Los motivos y los tres niveles se archivan en
[`decisions.tsv`](legal-geometry/riggs-conversation/decisions.tsv): estado de premisas,
respuesta propuesta, valoración externa, permanencia, testigos, referencias a
derivaciones, fuente del argumento, páginas, fuente del compromiso y fundamento
externo. Las eliminaciones se refieren a la historia hipotética específica.

## 4. Primer ciclo y ramas independientes

En `killingWithoutAcquisitionPurpose`, el cierre literal permanece completo.
Las premisas de las lecturas causal y formal son compatibles; la premisa de
finalidad está explícitamente negada para el acto vinculado.

| Valoración externa hipotética | Supervivientes | Eliminadas |
|---|---|---|
| Sostener la objeción | EarlCausalWrong | EarlAcquisitionPurpose, GrayStatutoryRoute |
| Rechazar la objeción | EarlAcquisitionPurpose, GrayStatutoryRoute | EarlCausalWrong |
| Suspender juicio | Las tres | Ninguna |

Las ramas parten del mismo catálogo. No se combinan respuestas opuestas como si
fueran una sola conversación. Los [espacios de versiones](legal-geometry/riggs-conversation/versions.tsv)
conservan cada historia y se verifican mediante replay independiente del catálogo.

El control `causalLinkUnestablished` tiene premisas indeterminadas para ambas
lecturas de la objeción. Sostenerla conserva esas dos y elimina la lectura formal
por desacuerdo con su compromiso; rechazarla conserva las tres. Ninguna de las
respuestas establece la evidencia causal pendiente.
También se archivan sus versiones, particiones y preguntas siguientes. Tras
sostenerla en ese control, el homicidio sin finalidad distingue las dos lecturas
pendientes; tras rechazarla siguen disponibles las dos preguntas iniciales.

## 5. Particiones: predicción y feedback

Se reutiliza el orden «más fina ≤ más gruesa». Se calculan dos particiones sobre
el mismo catálogo fijo:

- **Predicción:** igualdad de los conjuntos P_h(S) en los expedientes examinados.
- **Feedback:** igualdad de las firmas de compatibilidad con las respuestas
  externas efectivamente recibidas.

Las firmas de predicción codifican dos posibilidades booleanas, no un valor
binario ficticio para una premisa indeterminada. Las posibilidades de supervivencia
tras `sustain` y `reject` pueden solaparse cuando una hipótesis está pendiente;
ese solapamiento no se usa como relación de equivalencia.

Para ambas particiones, agregar una observación concatena su firma:

\[
\Pi_{n+1}=\Pi_n\wedge\pi_{n+1}.
\]

**ARGUMENTO GENERAL DEL PROCEDIMIENTO:** igualdad de firmas concatenadas equivale
a igualdad de ambas componentes; su núcleo es la intersección de las
equivalencias anteriores. Filtrar iterativamente equivale a exigir todas las
compatibilidades de la historia, que es el replay del catálogo fijo.

En el orden de catálogo `[Causal,Finalidad,Formal]`:

| Etapa | Partición predictiva | Partición de feedback |
|---|---|---|
| Inicial | 000 | 000 |
| Primera respuesta sustain | 011 | 011 |
| Primera respuesta reject | 011 | 011 |
| Primera respuesta pending | 011 | 000 |
| Tras reject, segunda respuesta determinada en histórico | 012 | 012 |

**RESULTADO FINITO:** una suspensión permite observar qué lecturas responderían
de forma distinta, pero no aporta una distinción jurídica externa que descarte
candidatos. Las [particiones archivadas](legal-geometry/riggs-conversation/partitions.tsv)
se comparan con meets calculados por las funciones del experimento anterior.

## 6. Próximas preguntas discriminantes

Se examinan únicamente los seis expedientes existentes, con sus sujetos,
instrumentos, prestaciones causales y tiempos conservados. Para cada uno se
cuentan las supervivencias posibles bajo ambas respuestas determinadas:

\[
w(S)=\max\bigl(|\{h:+\in P_h(S)\}|,
|\{h:-\in P_h(S)\}|\bigr).
\]

Las hipótesis indeterminadas se cuentan en ambas ramas. Una pregunta se considera
informativa cuando `w(S)` es menor que el número de supervivientes. Minimax se
aplica a esas preguntas y conserva empates. La garantía de reducción es condicional
a recibir una respuesta determinada; una suspensión no reduce el espacio.

| Etapa | Preguntas minimax disponibles | Peor supervivencia determinada |
|---|---|---:|
| Inicial, tres hipótesis | Histórico y homicidio sin finalidad | 2 |
| Tras sostener, una hipótesis | Ninguna discriminante en este catálogo | — |
| Tras rechazar, dos hipótesis | Histórico | 1 |
| Tras suspender, tres hipótesis | Las dos iniciales siguen pendientes | 2 |

Tras el rechazo, la pregunta siguiente es examinar los mismos fundamentos cuando
sí consta finalidad adquisitiva. En el expediente histórico, la lectura con
finalidad propone sostener la objeción y la lectura formal propone rechazarla.
Los turnos `followSustain` y `followReject` prueban por separado ambas valoraciones
hipotéticas, dejando respectivamente una lectura. No se importa el fallo histórico
como respuesta ni se pretende reproducirlo automáticamente.

`causalLinkUnestablished` no ofrece la misma garantía: la lectura con finalidad
permanece pendiente y puede sobrevivir bajo ambas respuestas. La
[tabla de preguntas](legal-geometry/riggs-conversation/questions.tsv) conserva
esas posibilidades y la adecuación jurídica `pending_external_deliberation`.

## 7. Conservación de la teoría y certificación

El [nuevo verificador](../src/bin/residuum/cli/legal_geometry_riggs_conversation_tests.rs)
carga los seis cierres completos una vez como estructura de referencia y evalúa
las hipótesis sobre ellos. El aprendizaje opera en State del harness, no en el
State patrimonial de Residuum ni en Module o Known.

Se recalculan independientemente los seis cierres después de cada una de las siete
valoraciones del guion. Las 42 comparaciones conservan eventos completos, tipos,
polaridades, tiempos y multiconjuntos de justificaciones: reglas, antecedentes
ordenados, sustituciones y módulos. Sólo `derivedAt` se ignora entre ejecuciones.
El AST elaborado y los orígenes de todas las declaraciones originales coinciden
con la referencia. La atribución literal histórica conserva 12 eventos y seis
justificaciones durante todo el aprendizaje.

Los [contextos jurídicos](legal-geometry/riggs-conversation/legal-contexts.tsv),
la [provenance](legal-geometry/riggs-conversation/legal-provenance.tsv) y las
[comparaciones](legal-geometry/riggs-conversation/legal-invariance.tsv) permiten
reconstruir el cierre desde sus seeds y eventos derivados. Las tablas originales
de argumentos, objeciones y fuentes se archivan junto a los compromisos y valoraciones.

Pasaron tres tests nuevos y dos regresiones individuales: actualización de
versiones, particiones/preguntas, conservación jurídica, el séptimo turno anterior
y el evaluador original de argumentos de Riggs. La
[verificación](legal-geometry/riggs-conversation/verification.md) registra comandos,
memoria y tiempos. Se mantuvieron 1 GiB, swap 0, un trabajo y un hilo.

## 8. Qué queda establecido y pendiente

**OBSERVACIÓN EXPERIMENTAL:** se obtiene una actualización conversacional
verificable y explicable sobre la teoría fija, con controles de indeterminación.
**ALCANCE FINITO:** tres lecturas existentes, seis expedientes y siete respuestas
estipuladas en ramas separadas. Una única superviviente significa identificabilidad
dentro de esos compromisos y esa historia, no aprobación jurídica general.

Quedan para deliberación independiente la adecuación de los compromisos
interpretativos, el fundamento de cada valoración, la suficiencia de las razones
y las competencias para limitar la atribución testamentaria. El harness conserva
el texto de las objeciones; no interpreta automáticamente su fundamento ni genera
razones nuevas. No se amplió el catálogo ni se integró revisión normativa alguna.

No hubo cambios en el motor, la ontología, las fixtures de Riggs, `main` o `v1`.
No se creó commit ni se realizó merge.
