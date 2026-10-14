# Ámbito de consulta inducido por las dependencias del AST

La [corrección conceptual posterior](residual-activation-contract.md) formula
por separado suficiencia Horn y suficiencia respecto de la activación dinámica.
No amplía la búsqueda existencial ni cambia el resultado del query actual.

El AST permite obtener un cono conservador de reglas potencialmente relevantes.
Ese cono no determina por sí solo cuáles están habilitadas, no aporta valores
para los parámetros ocultos y no reemplaza todavía el contexto operacional de una
consulta. La investigación permanece en el harness; no se cambia el buscador.

**Método y alcance.**

Se indexan las cabezas por `(verb, polarity, arity)`. Desde la firma del objetivo
se conservan todos sus productores, después los productores de sus antecedentes,
hasta estabilizar el conjunto. Un ResidualClause se observa mediante su
unresiduación existente. And conserva todos sus factores; Unit no aporta un
antecedente Event. No se añade ninguna hipótesis a Known.

El recorrido ignora inicialmente argumentos, constantes y tiempos: es una
sobreaproximación estructural, no matching ground ni resolución de restricciones.
La aridad y la polaridad se conservan; Act y ~Act son nodos distintos. Los tipos
de cada parámetro se obtienen de Arg en el AST elaborado. Las referencias y los
módulos de origen se conservan en las reglas seleccionadas.

Se usan las fuentes generales y patrimoniales ya fijadas por SHA-256 en el
[informe geométrico](legal-ontology-order-geometry.md), la fixture de sustituciones
explícitas y los escenarios existentes de Sales/defaults y programas asociados.
No se modifica ni duplica la ontología, ni se incorporan datos de 12hectareas.

**MODEL RESULT: compliance requiere un cono de siete reglas.**

| Regla conservada | Parámetros que sólo aparecen en el antecedente |
|---|---|
| obligationCompliance | conduct, actAuthority, actEffective |
| rightCompliance | conduct, actAuthority, actEffective |
| filedObligation | ninguno |
| statutoryObligation | ninguno |
| enactLaw | act, legislatorPerson, powerCause, powerAuthority, powerEffective |
| capacityPower | object |
| grantedCapacity | ninguno |

El cono contiene diez firmas proposicionales y doce ocurrencias de parámetros
ocultos, en sus respectivos ámbitos. Los tipos de esas ocurrencias se exportan,
incluidos PropositionTime y Authority. No se identifican dos parámetros sólo por
tener el mismo nombre en reglas distintas.

La cadena es:

```text
powerGranted -> legislativeCapacity -> legislativePower
Act & legislativePower & legislationText -> law -> obligation
obligationFiled -> obligation
obligation & Act -> compliance
right & Act -> compliance
```

Las flechas resumen las reglas reales; cada conjunción exige todos sus factores.
Las aristas completas, con las reglas y los patrones de eventos, están en
[query-dependency-edges.tsv](legal-geometry/data/query-dependency-edges.tsv).
El [grafo bipartito generado](legal-geometry/query-compliance-dependencies.svg)
conserva los nodos de reglas: varias entradas a una regla son premisas conjuntas,
no vías alternativas independientes. Se genera desde el export, no de una
enumeración manual de nombres.
La extensión complianceAsset no entra en este cono: parte de compliance hacia
patrimonio, no constituye otra vía que produzca compliance.

Este resultado difiere del programa contextual de tres reglas del
[experimento anterior](legal-ontology-explicit-query-substitutions.md). El
recorrido sobre toda la ontología **no puede descartar la cadena legislativa**
sólo porque sus parámetros estén sin resolver, o porque las dos vías conocidas
basten en el universo finito probado. Hacerlo alteraría las alternativas admitidas.

**THEOREM: preservación para reglas Horn ordinarias fijas.**

Sin defaults ni activaciones de Verb.program, sean R todas las reglas y Rq las
seleccionadas por este recorrido. Para cualquier evidencia S y evento ground
cuya firma sea la del objetivo, su pertenencia al least Horn closure es la misma
con R y Rq. Toda instancia que produce esa firma utiliza una regla conservada;
los productores de cada antecedente también se conservan. Por inducción sobre
las derivaciones finitas, cualquier prueba en R tiene la misma prueba en Rq.
La inclusión inversa se sigue de Rq ⊆ R.

El argumento también conserva los testigos, las sustituciones y las reglas de
cada justificación Horn directa del objetivo: todas las instancias relevantes
están disponibles en ambos cierres. Se consideran cierres completados; no se
identifican órdenes de evaluación, sellos de reloj o ejecuciones interrumpidas.
Esto no demuestra terminación general de Residuum ni equivalencia del cierre
completo: otras conclusiones pueden desaparecer al recortar reglas.
La afirmación matemática es sobre identidad lógica de eventos y justificaciones;
no impone estabilidad de la representación operacional de transition. Los tests
sí contrastan los campos presentes de sus eventos y registros en este dominio.

**MODEL RESULT de esa preservación.** Se comparan nueve entradas por ontología:
los ocho subconjuntos de obligationFiled, right y Act, más el conjunto total de
evidencia sintética de la investigación. En las 18 comparaciones coinciden la
presencia del objetivo y sus justificaciones completas, salvo derivedAt. Se
auditan los registros forward de las vistas recortadas. La consulta residual
sobre esos siete productores sigue devolviendo Err por parámetros pendientes.

**Objetivo, sustituciones explícitas y parámetros pendientes.**

El análisis separa parámetros visibles en la cabeza, parámetros usados sólo en
el cuerpo y parámetros declarados sin ocurrencias. La visibilidad es sintáctica;
no garantiza un match para cualquier objetivo. Para las dos reglas directas de
compliance se usa además query_residual real: el objetivo ground fija ocho
parámetros y deja tres pendientes, exactamente los indicados en la tabla.
En las reglas interiores no se pretende que el objetivo original haya fijado
sus parámetros: se documentan como candidatos a restringir o resolver en cada
ámbito durante una búsqueda posterior.

Tras **elegir explícitamente** CompliancePartial, ComplianceBound o
ComplianceGround, β conserva sus bindings y el recorrido devuelve los tres
RuleUses compilados. Bound y Ground mantienen los mismos dos árboles de prueba
que antes de aplicar el recorrido. Partial todavía devuelve Err. Las constantes
ya aplicadas permanecen constantes; los parámetros pendientes mantienen su tipo
y su identidad higiénica.

Declarar esos Programs en un módulo no elimina las 27 reglas importadas ni elige
uno de sus contextos para el query. Las dependencias reducen un ámbito ya elegido;
no eligen sus bindings, el Program ni el dominio de variables existenciales.

**Relevancia estructural ≠ selección de defaults ≠ antecedente satisfecho.**

En los cuatro escenarios existentes se observa la selección real mediante
run_default_experiments, sin modificar applicability, blocking, priority o
consistency. El AST permite identificar cinco reglas fuente controladas por
los dos Programs de defaults; la regla de permiso es compartida. Sus referencias
originales no se promocionan a reglas obligatorias por aparecer en un recorrido.
Los RuleUses compilados del Program seleccionado se distinguen de las reglas
obligatorias del entorno y de los cuerpos asociados a Verb.program.

| Escenario | Program seleccionado | Consecuencia observada |
|---|---|---|
| noRetention | normalSaleProgram | obligación de transmitir |
| retentionUnpaid | retentionSaleProgram | no obligación de transmitir |
| retentionPaid | retentionSaleProgram | obligación de transmitir |
| normalBlocked | retentionSaleProgram | selectedReciprocity Retention |

En cada cono activo aparecen miembros compilados del Program seleccionado y
ninguno del otro; no aparecen normalSale o retentionSale como reglas fuente
obligatorias. Las reglas que realmente justifican el objetivo están en ese cono.
No se exige que todo miembro seleccionado tenga un match: en retentionUnpaid
la regla de pago está disponible, pero carece de los antecedentes necesarios.
En estos cuatro casos se verifica que no aparece ningún trigger positivo de
los comportamientos asociados, ni siquiera en el cierre final. El catálogo
activo utilizado para este análisis omite esas definiciones por ese motivo;
no es un materializador general de activaciones para otros escenarios.

En el grafo de firmas sin contexto aparecen las dos entradas normalSale y
retentionSale. Para normalBlocked esto es también una sobreaproximación por
constantes: la cabeza Plain no unifica con un objetivo Retention. Por tanto,
incluso antes de discutir prioridad, alcance por firma no significa match ground.
No se afirma que estas firmas determinen applicability ni qué default debe ganar.

**Frontera operacional: la guarda implícita de Verb.program.**

La fixture existente sells_unit_associated_program.res declara sells asociado a
una regla `I <= give(...) @ after t`. El grafo de cláusulas aisladas no contiene
una arista desde sells. La asociación del AST proporciona esa dependencia de
activación: se requiere una instancia **positiva conocida** del verbo asociado.

Se usa IncrementalProgramClosure real para observar las activaciones:

| Evidencia | Activaciones de sellsProgram | give derivado |
|---|---:|---|
| ∅ | 0 | no |
| ~sells | 0 | no |
| sells | 1 | sí |

La activación registra el trigger, sus bindings y la referencia al comportamiento;
la justificación de give conserva sells como antecedente aunque el cuerpo sea I.
No se ejecuta State. La correspondencia temporal y las instancias concretas
siguen siendo restricciones de la activación, no simples aristas de firmas.

Hay dos contrastes importantes con el buscador residual actual:

1. Consultar give en el módulo anterior sin evidencia genera un candidato, pero
   devuelve NotForwardSupported. El replay de la cláusula aislada con antecedente
   Unit no determina sus parámetros desde un testigo; esto **no demuestra** que
   el query haya comprobado la guarda de sells. La primera versión del test había
   supuesto que retornaría Unit; esa expectativa fue rechazada y corregida, sin
   cambiar el motor.
2. En associated_program_waits.res, signals activa prepare y produce ready,
   pero no existe starts para activar onStart. El forward no deriva done.
   La consulta residual con ese Known devuelve Unit, certificada usando
   `ready <= done` como regla Horn ordinaria. Esa prueba no incluye starts.
   Volver a cerrar el Known con close_program sigue sin derivar done.

El segundo caso es un contraejemplo a usar el query de cláusulas como una
consulta automáticamente equivalente a la activación de programas. No es una
consecuencia jurídica nueva ni un fallo de monotonicidad del cierre. La
certificación Horn aislada y la semántica del módulo activable son relaciones
distintas en este caso. Se conservan Known y provenance; no se parchea el query
ni se añade la guarda al cuerpo de las reglas.

**Qué parte del adaptador podría sustituirse.**

Puede automatizarse la obtención del cono sobre un conjunto explícito de reglas
habilitadas, conservando todas las alternativas y los orígenes. También puede
extraerse del AST qué definiciones están controladas por defaults o asociaciones
y qué parámetros no aparecen en sus cabezas. El análisis debe informar esa
condicionalidad; no transformar una definición condicional en una regla ordinaria.

No basta con el AST para elegir defaults aplicables a un Known concreto o
instancias activadas: esas decisiones se observan en el evaluador existente. Una
vista futura de query tendrá que preservar la guarda, la especialización y su
testigo, además de los bindings proporcionados. La API residual vigente no
representa ese contexto de habilitación. Aquí se detiene la sustitución automática
del adaptador; no se implementa una nueva semántica para resolverlo.

La resolución existencial continúa pendiente. Un parámetro sin determinar no
autoriza a excluir una regla ni a inventar un valor; tampoco se enumera un dominio
abducible a partir de este recorrido. No se demuestra completitud residual.

**Verificación y artefactos.**

El harness está en `src/bin/residuum/cli/legal_geometry_query_scope_tests.rs`.
Las tablas `query-dependency-*.tsv` se exportan desde el AST y las ejecuciones
reales a `docs/legal-geometry/data/`. No hay nuevas fixtures, declaraciones
jurídicas, sintaxis ni cambios de producción. Main y v1 permanecen intactos en
experiment-legal-order-geometry. Los tests se ejecutan individualmente mediante
scripts/test_safe.py, sin elevar recursos ni ejecutar la suite completa.

Pasaron cuatro tests nuevos y dos regresiones de sustituciones explícitas. La
compilación final consumió 434.02 MiB RSS y 4.0424 s. Los tests nuevos tardaron
0.3467, 0.1843, 0.2253 y 0.0417 s, todos con menos de 16 MiB RSS. Las regresiones
tardaron 0.1839 y 0.6310 s. Se mantuvieron 1 GiB, cero swap, un trabajo de Cargo,
un hilo y los timeouts del runner. Log final:
`target/test-diagnostics/20261010T012522059955Z-665437`.
La ejecución previa se detuvo ante una aserción incorrecta del test Unit;
el test corregido también pasó individualmente antes de la ejecución final.

`scripts/analyze_legal_geometry.py --render` publica las tablas y genera el grafo
y `query-dependency-results.json`, verificando los hashes de las fuentes fijadas.
Ese script sólo resume exports: no implementa matching, selección ni cierre.
