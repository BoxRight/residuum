# Contrato conceptual de consultas residuales con activación dinámica

Este documento propone la relación que una consulta de módulo tendría que
preservar. No modifica el lenguaje, las APIs residuales, close_program ni la
resolución existencial. La prueba nueva usa el forward existente como oráculo
de suficiencia de hipótesis ground, no como implementación de un solver nuevo.

**Dos contratos que deben nombrarse por separado.**

La garantía vigente de T10 en la
[especificación monotónica](monotonic-core-specification.md) se refiere al
conjunto R(Π) de declaraciones Rule tratadas como cláusulas explícitas:

\[
H\in Q_{Horn}(R(\Pi),K,G)
\Longrightarrow G\in Close_{R(\Pi)}(K\cup Leaves(H)).
\]

Ese contrato no implica suficiencia bajo close_program. Una definición
referenciada por Verb.program deja de ser una regla ordinaria en el forward
del módulo y exige un trigger positivo conocido. La
[investigación de dependencias](legal-ontology-query-dependency-scope.md)
ya muestra un caso en que el certificado de cláusulas omite ese trigger.
No se modifica retrospectivamente el contrato de T10 ni se declara una nueva
garantía implementada.

Para una consulta **sobre el comportamiento del módulo**, la obligación propuesta
es diferente:

\[
H\in Q_{active}(\Pi,\Omega,K,G)
\Longrightarrow
G\in Close_{\Pi,\Omega}(K\cup Leaves(H)).
\]

Close en esta expresión es el least closure con la misma política de activación
del forward. La ejecución concreta debe alcanzar su punto fijo para afirmar
igualdad de resultados completos; una prueba finita justificaría pertenencia al
cierre abstracto aunque la ejecución global no terminase. No es entailment FOUR.

Ω denota el contexto operacional elegido para la consulta: reglas obligatorias,
Programs que ya seleccionó el cálculo de defaults, definiciones asociadas y sus
orígenes/bindings. No es una nueva construcción Formal o Rust. Sirve para dejar
explícito qué comportamiento se está consultando y qué selección permanece fija.

**Qué significa habilitar una regla.**

| Procedencia | Condición necesaria para utilizarla |
|---|---|
| Regla ordinaria | Pertenecer al entorno obligatorio; el body debe tener testigos compatibles |
| RuleUse de un Program seleccionado | Pertenecer al Program seleccionado, con sus bindings y scope; el body debe tener testigos |
| Regla/RuleUse de Verb.program | Existir una instancia positiva conocida del verbo que fije los parámetros compartidos y el tiempo; además, satisfacer el body especializado |

La referencia en el AST no es una activación. La selección de un Program por
un default tampoco demuestra que sus cláusulas tengan todos sus antecedentes.
Que un body sea I elimina sus factores explícitos, pero no la condición de
activación de una asociación. Effect no activa aquí comportamiento adicional
ni se ejecuta State.

Los triggers tienen la identidad completa del Event: verbo, argumentos,
PropositionTime y polaridad. No basta la firma del verbo. Los argumentos y
el índice temporal fijan los parámetros compartidos; los locales siguen
teniendo su propio ámbito. La sustitución inducida por el goal debe ser compatible
con esos bindings, no reemplazarlos para fabricar una activación distinta.

**La activación debe poder justificarse en el cierre hipotético.**

No basta con conservar sólo Activated(K) al comenzar una consulta abductiva:
una hipótesis Seeded puede activar un programa, y una consecuencia Derived de
las hipótesis puede activar otro. Las activaciones pertinentes son las del
cierre hipotético que parte de K ∪ Leaves(H), no sólo las de K.

Conceptualmente, con una selección Ω fija:

\[
F_0=K\cup Leaves(H),
\qquad
F_{n+1}=F_n\cup T_{ordinary,\Omega}(F_n)
\cup T_{associated,\Pi}(F_n).
\]

T_associated conserva las guardas y la especialización de cada instancia,
incluido el trigger en su justificación. No identifica todas las definiciones
importadas con reglas ordinarias ni ejecuta cierres separados por cada RuleUse.
Un trigger Derived necesita una prueba; no se convierte en hoja abducible. La
admisibilidad de una hipótesis Seeded depende de la política abducible elegida.
~V no activa el programa positivo y la ausencia de V no demuestra ~V.

El punto de esta corrección no es añadir búsqueda de variables: todos estos
ejemplos son ground. Si no se puede determinar una instancia compatible de la
guarda con los parámetros disponibles, eso no autoriza a devolver Unit. La
limitación debe permanecer visible, sin inventar un valor ni descartar la regla
como si se hubiera demostrado que no puede aportar una explicación.

**Qué tendría que conservar el certificado.**

Para un paso asociado, una prueba suficiente debe conservar conceptualmente:

1. La asociación del verbo con el comportamiento y el RuleUse empleado.
2. El Event positivo que lo activa y su justificación: evidencia de K, hipótesis
   admisible o derivación anterior dentro de la prueba.
3. Los bindings de activación, compatibles con los bindings del body y del goal.
4. Los testigos del antecedente especializado y la consecuencia resultante.
5. Los orígenes de regla, Program y módulo, donde corresponda.

La guarda implícita no se añade a las declaraciones jurídicas ni se reescribe
RuleBody para presentarla como una cláusula ordinaria. Es una obligación del
paso operacional certificado. ProgramActivation y DerivationRecord ya permiten
observar buena parte de esta información en el forward; ResidualProof vigente
no posee un paso de activación que la certifique como tal.

Una prueba de trigger no puede justificarse circularmente por el resultado que
pretende habilitar sin evidencia inicial suficiente. Un certificado finito
debe ordenar sus pasos o ser un DAG con dependencias anteriores verificables.
Las justificaciones alternativas no se descartan por producir el mismo evento.
derivedAt sigue siendo un sello operacional; las consultas no sellan requisitos.

**THEOREM condicional de suficiencia.** Si las hojas pertenecen a K o a las
hipótesis admisibles y cada paso verifica tanto su habilitación como su instancia
forward, la inducción sobre la prueba finita sitúa cada conclusión en el cierre
hipotético activo. Para un paso asociado, el trigger ya pertenece a ese cierre,
por lo que existe la activación con esos bindings; los testigos del body hacen
aplicable su regla especializada. Esto prueba suficiencia del contrato, no que
el buscador actual lo implemente ni que una búsqueda futura sea completa.

**MODEL RESULT: el requisito pendiente es el trigger.**

Se usa exclusivamente la fixture existente
[associated_program_waits.res](../examples/experiments/associated_program_waits.res).
El Known inicial es su cierre real desde signals(juan) @ tau: contiene ready,
pero no starts ni done. El objetivo es done(juan) @ after tau.

| Hipótesis aportada únicamente al cierre hipotético | Nueva activación onStart | Objetivo derivado |
|---|---:|---|
| Ninguna | no | no |
| starts(juan) @ sigma | sí, en sigma | no |
| ~starts(juan) @ tau | no | no |
| starts(juan) @ tau | sí, en tau | sí |
| ~starts y starts, ambos en tau | sí, en tau | sí |

En este caso, la condición esperada por el contrato activo es
starts(juan) @ tau, no Unit. La consulta residual actual continúa devolviendo
Unit bajo su semántica de cláusulas aisladas: el experimento no corrige su
resultado ni afirma que ya encuentre la hipótesis.

La hipótesis ground candidata procede del AST de la misma fixture. Su
suficiencia se verifica calculando el cierre real; no se aporta done como
evidencia. Al admitir sólo esa instancia en un dominio ground singleton, los
dos subconjuntos posibles demuestran que es el único soporte mínimo suficiente
de ese dominio. No se declara unicidad en todo el lenguaje ni se introduce una
declaración nueva de dominio abducible.

Para cada una de las cinco entradas se comparan recomputación e incremento,
usando close_program e IncrementalProgramClosure existentes. Coinciden eventos,
tipos, tiempos, metadata y todas las parejas evento/DerivationRecord, permitiendo
distintos derivedAt. La justificación de done conserva exactamente los testigos
ordenados starts y ready y los bindings de activación se observan en el delta.
El Known y sus justificaciones originales permanecen intactos.
La coexistencia de ambos signos no elimina hechos ni impone inconsistencia.

**Defaults: contexto fijo, no nueva política.**

Para consultar consecuencias de un Program seleccionado, la evidencia de
selección y su identidad forman parte de Ω. Una cláusula de un Program no
seleccionado no se vuelve elegible sólo porque su head unifique con el goal.
La regla compartida del entorno conserva su condición de regla obligatoria.

Este contrato no decide qué hacer si H cambiaría applicability, blocking o
priority al volver a evaluar defaults. La garantía es relativa a la selección
fijada por Ω. Una consulta que prometa reevaluar defaults bajo hipótesis tendría
un contrato distinto que sigue pendiente. No se abducen decisiones del selector,
ni se conecta consistency con oposición P/~P.

**Estado y siguiente frontera.**

- IMPLEMENTED + TESTED: activación forward por triggers positivos, su tiempo,
  provenance e incremento en las instancias descritas.
- MODEL RESULT: cinco cierres hipotéticos y el dominio ground singleton anterior.
- PROPOSED CONTRACT: suficiencia residual relativa al cierre con activaciones y
  al contexto Ω, con certificados que conservan las guardas.
- NOT IMPLEMENTED: integración de ese contrato en la consulta residual; tampoco
  nueva resolución existencial o reevaluación hipotética de defaults.

El ajuste conceptual consiste en no llamar a una prueba válida sólo bajo
R(Π) una prueba del comportamiento activo de Π. No requiere otra regla jurídica,
una política global de inconsistencia ni ampliar la unificación.

La [investigación posterior de extensión mínima](residual-active-query-minimal-extension.md)
prueba la búsqueda con guardas en una vista temporal del AST y certifica sus
hipótesis con el close_program original. Es un prototipo del harness, no una
modificación de la API residual.

Verificador: `src/bin/residuum/cli/legal_geometry_residual_activation_tests.rs`.
Export: `docs/legal-geometry/data/residual-activation-contract.tsv`.
Rama experiment-legal-order-geometry; main, v1, motor y ontología intactos.

**Verificación individual.** Se ejecutaron únicamente el nuevo oráculo de
activación hipotética y la regresión previa que expone la guarda implícita,
mediante `python3 scripts/test_safe.py`. Ambos pasaron; no se ejecutó la suite.
Compilación: 3,4499 s y 387,68 MiB RSS. Tests: 0,0213 s / 15,56 MiB y
0,0415 s / 15,55 MiB, respectivamente. Se conservaron 1 GiB total, sin swap,
un trabajo de Cargo y un hilo. Informe:
`target/test-diagnostics/20261010T013735899643Z-675452`.
