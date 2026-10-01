# Auditoría del núcleo monotónico

Fecha: 2026-10-01. Referencia: [especificación provisional](monotonic-core-specification.md).
Este documento conserva la auditoría **anterior a las correcciones** de F1–F4.
La [segunda auditoría](monotonic-core-audit-followup.md) registra el estado actual,
las cuatro regresiones ejecutadas y las obligaciones que siguen pendientes.

Revisión por inspección del código, argumentos locales y evidencia ya registrada.
No se modificaron código, fixtures ni la especificación; no se compiló ni se
ejecutaron tests. Los resultados históricos no constituyen una verificación nueva.

**Resultado: el núcleo no está terminado respecto del contrato completo.** Hay
un motor Horn monotónico, consultas certificadas y mantenimiento incremental en
fragmentos concretos. Todavía falta una realización proposicional de L y su
puente general con Known. Además, la inspección encuentra cuatro discrepancias
de tipado/scope/tiempo que impiden declarar satisfechas T1, T2 y la realización
general de T5. No requieren inventar operadores nuevos para poder describirlas.

## Criterios de clasificación

Estado y clase de evidencia son dimensiones distintas:

| Estado | Criterio |
|---|---|
| Realizado en un fragmento | Hay un mecanismo concreto y sus precondiciones se pueden identificar |
| Parcial, con discrepancia | Hay implementación, pero algún camino contradice la obligación declarada |
| Pendiente | No existe una realización de la obligación en el dominio prometido |
| Fuera del contrato | No se ha prometido esa capacidad en este núcleo |

| Evidencia | Uso en esta auditoría |
|---|---|
| **THEOREM** | Argumento general explicitado, con sus hipótesis; no significa que Rust esté verificado formalmente |
| **MODEL RESULT** | Enumeración exhaustiva de un carrier o familia finita identificada, registrada previamente |
| **EXPERIMENTAL OBSERVATION** | APIs, ramas de código, tests y fixtures concretos; se indica si el hallazgo procede sólo de inspección |

T1–T12 nombran las obligaciones de la especificación. Que una fila remita a T5
no convierte automáticamente su algoritmo en una demostración. Un oráculo que
reutiliza el mismo matcher puede detectar diferencias entre algoritmos, pero
puede compartir sus errores de interpretación.

## Tabla original de obligaciones T1–T12

| Obligación | Estado actual | Evidencia y alcance | Falta o discrepancia |
|---|---|---|---|
| **T1: sustitución tipada y β-reduction** | Parcial, con discrepancias | **EXPERIMENTAL OBSERVATION**: [lower_rule](../src/elab.rs#L594) especializa prefijos de parámetros mediante constantes de tipo exactamente igual; sustituye términos de ambos cuerpos. Los fixtures de Conduct y los [tests del residual](../src/residual/tests.rs#L1046) dejan libre el tiempo. | No sustituye índices temporales al aplicar un argumento PropositionTime (F2). Las constantes dentro de cláusulas no se cotejan con sus tipos declarados (F3). No hay garantía general de preservación de tipos/scope. |
| **T2: identidad lógica e índices** | Clave realizada; admisión temporal y de scope parcial | **THEOREM**, por definición de [logical_key/Eq/Ord](../src/typed.rs#L328): categoría, transición y provenance no determinan igualdad. **EXPERIMENTAL OBSERVATION**: elaboración exige `@`; After se conserva y el subtipado exige el mismo índice. | Exigir una anotación no basta para comprobar su scope/tipo. El forward puede sustituir un tiempo constante (F1); un valor runtime puede confundirse con un parámetro libre (F4). Las estructuras públicas permiten construir eventos fuera de las precondiciones. |
| **T3: lattice residuado conmutativo proposicional** | Pendiente en M2; realizado en modelos de referencia | **MODEL RESULT**: [assert_laws](../src/algebra/tests.rs#L10), FOUR-meet y FOUR-fusión recorren carriers finitos completos. [Los traits](../src/algebra.rs) sólo declaran operaciones y leyes. | Ningún Event, Antecedent o RequirementFormula implementa la realización completa: faltan interpretación proposicional elegida, equivalencia y orden general. No se puede transferir el resultado de los modelos al AST. |
| **T4: I como unidad y contexto** | Realizado para Unit/Event/And | **THEOREM**, argumento local: Unit devuelve un único match con bindings y witnesses intactos; componerlo por cualquier lado no cambia el resultado. **EXPERIMENTAL OBSERVATION**: [tests de unidad](../src/residual/tests.rs#L111) y ronda inicial de close. | La ley sobre todo L depende de T3. I no representa Known vacío; un axioma `I <= C` justifica sólo su C ground. |
| **T5: least Horn closure** | Realizado condicionalmente en el fragmento del matcher; parcial frente al scope pretendido | **THEOREM**, argumento abstracto de la especificación y argumento del bucle descrito abajo. **MODEL RESULT**: familias ground de soporte. [close_with_programs](../src/derive.rs#L256) acumula claves hasta estabilizar. | La correspondencia entre matching operacional y la cláusula con scope falla en F1/F4. No hay terminación general ni exhaustividad algebraica. |
| **T6: activación monotónica** | Realizado para instancias positivas Seeded/Derived con programa fijo | **EXPERIMENTAL OBSERVATION**: [close_program](../src/derive.rs#L146) resuelve RuleRef y firma, excluye definiciones del pool ordinario y conserva activaciones. **THEOREM** condicional: triggers y witnesses no se retiran. **MODEL RESULT**: familias de composición. | Hereda los límites de matching/scope. No activa asociaciones Effect; residual e incremental no descubren asociaciones. La biblioteca exige estabilización, sin comprobarla universalmente. |
| **T7: provenance** | Realizado para justificaciones del motor actual | **EXPERIMENTAL OBSERVATION**: [finish_instantiation/stamp](../src/derive.rs#L36) separan evento, registro y reloj; el cierre deduplica claves y conserva registros distintos. | Una justificación reproducible por el mismo matcher no demuestra corrección respecto del scope pretendido (F1/F4). El replay residual no interpreta triggers implícitos de asociaciones. No se promete igualdad del schedule ni de los sellos entre recomputaciones. |
| **T8: EvidentialNot** | Realizado como polaridad atómica explícita | **THEOREM**, argumento por las únicas fuentes del cierre: seeds o consecuentes explícitos; ninguna rama crea negación por ausencia ni elimina el positivo. **EXPERIMENTAL OBSERVATION**: matcher exige signo y execute_effect rechaza el efecto negativo. **MODEL RESULT**: swap e involución en FOUR. | No equivale a negación compuesta general ni negation-as-failure; éstas no son obligaciones implementadas de este fragmento. |
| **T9: frontera de consulta residual** | Inmutabilidad realizada; resolución local restringida | **THEOREM**, por APIs con referencias compartidas sin mutabilidad interior en estos datos: la consulta no modifica evidencia ni llama a stamp o execute_effect. **EXPERIMENTAL OBSERVATION**: [query_residual](../src/residual/query.rs#L18) y búsqueda de programa. | `[]` local mezcla no-match y satisfacción completa. El unificador local sí usa scope; no coincide siempre con forward (F1/F4). No realiza el adjunto general de L. |
| **T10: suficiencia abductiva certificada** | Realizado respecto del forward ordinario, con precondiciones | **THEOREM** condicional por inducción sobre el certificado. **EXPERIMENTAL OBSERVATION**: [replay_proof](../src/residual/program.rs#L267) exige una instancia forward por nodo y coincidencia del goal. **MODEL RESULT**: oráculo de extensiones finitas. | Certifica consecuencias del matcher existente, no soundness de ese matcher en todos los scopes. Grounding/tipos de identidades son precondiciones del caller. No incluye activación general de Verb.program. |
| **T11: alternativas sin preferencia** | Realizado para ramas admitidas y generadas | **EXPERIMENTAL OBSERVATION**: el resultado conserva pruebas separadas y deduplica sólo árboles idénticos del resumen Join. **MODEL RESULT**: alternativas y productos cartesianos finitos. | No enumera todas las pruebas posibles: evidencia directa y Seeded son terminales. No decide equivalencia en L ni preferencia; no promete explicación única. |
| **T12: mantenimiento incremental** | Realizado para adiciones, reglas fijas y cierres que estabilizan | **THEOREM** condicional: toda justificación nueva usa alguna clave nueva, por lo que un pivote delta basta para enumerarla. **EXPERIMENTAL OBSERVATION**: [IncrementalClosure](../src/derive/incremental.rs#L35) conserva base/reglas/sellos. **MODEL RESULT**: secuencias finitas y 512 actualizaciones de composición. | Reutiliza los límites semánticos del matcher. No mantiene asociaciones arbitrarias, cambios de reglas, borrado, residual ni State. La derivada general sigue pendiente. |

## Leyes algebraicas y su realización operacional

Esta tabla evita decir «el & del lenguaje es un lattice residuado» porque el
matcher acepta conjunciones. Sus propiedades de soporte son más limitadas.

| Ley o capacidad | Resultado auditado |
|---|---|
| `&` asociativo | **THEOREM local** para matches: reagrupación preserva las secuencias de matches, bindings y concatenación de witnesses. And no ejecuta otra operación. **MODEL RESULT** para tensor en los modelos. La ley en L proposicional queda pendiente. |
| `&` conmutativo | En el fragmento con matching que respeta scope, intercambiar factores conserva las asignaciones compatibles y las conclusiones; **THEOREM condicional** por simetría de las restricciones de igualdad. Cambia el orden de witnesses/provenance; no hay igualdad literal de árboles ni registros. **MODEL RESULT** para tensor de los modelos. |
| `I & A` y `A & I` | **THEOREM local** para el matcher: exactamente los mismos bindings y witnesses; existen tests explícitos. En modelos, **MODEL RESULT**. No identifica I con vacío de K. |
| Reflexividad/transitividad de `<=_L` | **MODEL RESULT** en los modelos; **THEOREM** sólo como hipótesis del contrato. Encadenar reglas Horn no proporciona un decisor general de este orden. |
| Join como least upper bound; meet como greatest lower bound | **MODEL RESULT** en los carriers. RequirementFormula::Join representa alternativas, pero no tiene `le` ni `meet` proposicionales; capacidad general pendiente. |
| Monotonía de tensor; variancia del residual | **MODEL RESULT** para el orden propio de los modelos; consecuencias **THEOREM** condicionales de la adjunción. No se verifican en el AST general. Fusión FOUR no es monótona en el knowledge order; eso no contradice el contrato sobre truth order. |
| Residuación en ambas direcciones | **THEOREM estructural**: para Horn con And exterior, `unresiduate(residuate(r)) = r`; para ResidualClause, el recorrido inverso reconstruye ese fragmento. [Las funciones](../src/residual.rs#L21) copian parámetros, operandos y head. **MODEL RESULT**: adjunción numérica/ordenada exhaustiva. La transformación no es por sí sola un residual semántico universal. |
| Unidad/counidad del adjunto | **THEOREM** condicionado a T3 y **MODEL RESULT** en carriers finitos; no hay evaluador de esas fórmulas generales en M2. |
| Distribución de tensor sobre join | Igual clasificación: consecuencia condicional de adjunción y **MODEL RESULT**. El matcher no ejecuta joins arbitrarios. |
| Residual preserva meets en el resultado | Igual clasificación. No hay meet/residual general representable en el TypedAST actual. |
| Currificación/residual iterado | **MODEL RESULT** en los modelos; el lenguaje conserva un solo nivel de residual en RuleBody. No implementado como familia arbitraria de fórmulas. |
| Integralidad, weakening, contraction o tensor idempotente | **Fuera del contrato**. Reusar un evento en un match no demuestra esas leyes en L. No deben añadirse como consecuencias tácitas. |
| `Known ↔ L` | **Parcial**: relación de soporte con witnesses para Unit/Event/And y [adaptador de requirements](../src/algebra/requirement.rs) hacia una valoración explícita. No existe una equivalencia general ni un puente completo elegido. |
| Completitud residual | **Pendiente en general**; cobertura **MODEL RESULT** en una fixture finita. Hay límites, errores de ciclo y parámetros no determinados; no son pruebas de inexistencia lógica. |
| Derivada general | **Pendiente**. T12 realiza mantenimiento de un cierre add-only con reglas fijas; no implica una operación diferencial sobre todo L o todo programa activo. |

### Argumentos generales que sí se pueden aislar

**Unidad/asociatividad del matcher.** Cada And compone búsquedas con la misma
sustitución y concatena witnesses. Unit es la identidad de esa composición.
Asociar tres composiciones de búsquedas finitas enumera los mismos triples y
concatena la misma lista. Este argumento no depende de un carrier FOUR ni de
que Event implemente Preorder. Tampoco prueba una interpretación fiel de L.

**Cierre.** Con matching válido, reglas fijas y cuerpos finitos, añadir evidencia
conserva todos los witnesses. El bucle sólo añade resultados de instanciación;
por inducción, todos pertenecen al cierre abstracto. Si termina, la última ronda
ha examinado todas las reglas contra el conocimiento final y no añadió claves:
ese conjunto está cerrado. Cualquier conjunto cerrado que contiene las seeds
contiene todos los pasos, por lo que el resultado es el menor. Para asociaciones
se necesita además que cada trigger conocido haya activado su definición; la
frontera cubre cada clave nueva y las activaciones persisten.

Hay dos afirmaciones diferentes: menor cierre para las operaciones que realmente
ejecuta el matcher, y menor cierre de las cláusulas con scope de T5. F1/F4 impiden
identificarlas sin restricciones. El argumento no asegura terminación ni
soundness universal frente a todos los modelos FOUR.

**Incrementalidad.** La base ya contiene todas sus justificaciones de un paso.
Una justificación nueva no puede usar exclusivamente claves antiguas. Elegir
uno de sus witnesses nuevos como pivote, buscar los demás en Known y restaurar
el orden original enumera esa justificación. Unit sólo origina axiomas de la
base. La frontera repite este razonamiento para cada conclusión nueva; `seen`
evita sellar dos veces la misma justificación. Es un argumento condicional al
matching válido y estabilización, no una prueba de tipado/scope del matcher.

## Discrepancias concretas encontradas

Los ejemplos siguientes son trazas mínimas deducidas de ramas de código. Se
incluyen como texto del informe: **no se crearon ni ejecutaron fixtures nuevos**.

### F1 — un tiempo ground se trata como variable forward

Obligaciones afectadas: T2, T5, T7 y la interpretación del certificado T10.

Con tau y sigma declarados PropositionTime, y verbos A Seeded y B Derived:

```text
rule fixed() = A() @ tau <= B() @ tau
```

La lambda no tiene parámetros: elaboración elimina tau de los binders inferidos
por ser constante temporal. Sin embargo, [bind_time](../src/derive.rs#L422)
acepta cualquier `TimeExpr::At(name)` como variable, sin recibir el scope.
Ante `A() @ sigma` obtiene `tau ↦ sigma`; la sustitución produce `B() @ sigma`.
La regla ground pretendida no autoriza ese paso.

El [unificador residual](../src/residual/query.rs#L188) sí exige igualdad literal
cuando el nombre no está en el scope, y no hace ese match. Es una divergencia
forward/backward, no la incompletitud esperada frente al entailment FOUR.
El [runner FOUR](../src/experiments/four.rs) ya reconoce esta limitación y exige
un único índice compartido; sus resultados no cubren este contraejemplo.

**Clasificación:** discrepancia de implementación/scope. El dominio confirmado
debe excluir estos matches entre tiempos constantes distintos hasta corregirla.

### F2 — β-reduction elimina el binder temporal sin sustituir su uso

Obligaciones afectadas: T1 y coherencia temporal de T2.

Con tau constante PropositionTime y Q Derived:

```text
rule at(t) = I <= Q() @ t
rule fixed = at(tau)
```

[lower_rule](../src/elab.rs#L674) acepta el tipo, registra
`t ↦ Term::Const("tau")` y elimina t de parameters. Pero
[substitute_event_terms](../src/elab.rs#L783) copia proposition y transition
sin sustituir sus tiempos. El cuerpo de fixed conserva `Q() @ t`, en vez de
`Q() @ tau`. Al no quedar parámetros, la comprobación de parámetros pendientes
no recupera el binder eliminado.

Los tests de β-reduction auditados especializan argumentos de entidades y dejan
libre PropositionTime; no ejercitan esta aplicación. La sustitución temporal del
motor forward sí existe, pero no es la usada para esta β-reduction del elaborador.

**Clasificación:** incumplimiento de especialización temporal; no es una nueva
construcción faltante ni una incompatibilidad algebraica.

### F3 — constantes en reglas no se validan contra la firma

Obligaciones afectadas: T1 y el universo bien tipado de T2/T5.

Ejemplo usando tipos y construcciones existentes:

```text
entity Person
entity Conduct
const molest : Conduct
seeded verb trigger()
derived verb marked(subject : Person)
rule bad(t) = trigger() @ t <= marked(molest) @ t
```

[infer_rule_params/infer_event_args](../src/elab.rs#L1048) infieren Person para
el nombre molest por su posición en marked, sin consultar la declaración de
la constante. Sólo devuelven los parámetros solicitados. Después lower_event
convierte molest en Term::Const sin cotejar Conduct con Person. La ruta de
elaboración no contiene el rechazo necesario para esta cláusula.

En contraste, [lower_ground_event](../src/elab.rs#L299) sí coteja tipos de
constantes y tiempo en datos/goals de experimentos. Ese control no alcanza los
antecedentes y consecuentes ordinarios. La presencia de una anotación temporal
tampoco asegura que una constante usada allí sea PropositionTime.

**Clasificación:** hueco de tipado dentro de reglas; los tests de conflictos
entre usos de una variable no justifican el control de constantes declaradas.

### F4 — identidades runtime pueden confundirse con binders pendientes

Obligaciones afectadas: T2, realización de T5 y cobertura/suficiencia T10.

Para `r(x,t): P(x)@t <= Q(x)@t`, considere evidencia cuyo objeto ground en la
API actual es `Term::Var("x")`, usado como identidad runtime opaca. El matcher
liga x a ese valor; al sustituir, el resultado sigue representado por Var("x").
[has_unresolved_rule_parameter](../src/derive.rs#L95) ve el mismo nombre y
descarta la instancia como no ground. No distingue una referencia al binder
de un valor runtime ya ligado.

El [test existente](../src/residual/program/tests.rs#L686)
`forward_certificate_does_not_hide_runtime_identifier_scope_limitations`
documenta esa familia de rechazos: el certificado impide devolver una explicación
que close no podrá reproducir. Ese rechazo preserva la suficiencia operacional,
pero no demuestra completitud ni ausencia de la consecuencia lógica pretendida.

**Clasificación:** limitación de representación/scope ya observable en tests.
El fragmento seguro necesita identidades que no se confundan con binders, o
valores Const apropiadamente declarados y tipados.

## Límites que no deben clasificarse como errores

| Límite | Interpretación correcta |
|---|---|
| `~Q, P <= Q` no deriva `~P` por Horn | Diferencia esperada frente al entailment FOUR completo; no hay contraposition implícita. |
| `~P` no aparece por ausencia de P | Garantía T8, no falta de completitud. |
| El query devuelve error por un ciclo o binder no determinado por el head | Búsqueda fuera del fragmento; no prueba que no exista explicación. |
| Un goal ya evidenciado o un Seeded detiene expansión | Política actual. No promete todas sus pruebas ni explorar productores alternativos de un Seeded. |
| `close` no descubre Verb.program | Es la API ordinaria; close_program es la modalidad activa. |
| IncrementalClosure no activa asociaciones arbitrarias | Extensión no realizada; los tests de asociaciones autoancladas no la sustituyen. |
| Effect.program inactivo | Restricción explícita de T6 actual, separada de ejecutar StateTransform. |
| El Store pierde un asset al ejecutar give | No contradice la monotonía de Known. execute_effect construye otro snapshot. |
| No hay explicación abductiva mínima/preferida | Decisión explícita de T11; no es deuda para este cierre. |
| No hay adjunción general Verb ↔ Rule ni imports/interfaces | No son garantías del contrato. El linking y los checkers experimentales son harnesses. |

## Alcance real de la evidencia finita

| Resultado existente | Lo que establece | Lo que no establece |
|---|---|---|
| Leyes exhaustivas en powerset Z/2Z, cadena Łukasiewicz de tres valores y FOUR | **MODEL RESULT** sobre esos carriers y operaciones | Que las fórmulas de M2 implementen L o una semántica definitiva |
| 4.016 teorías / 20.080 consultas por configuración | En fusión/filtro I: cero fallos de soundness y 680 de completeness en esa familia; controles de soporte coinciden | Soundness universal, completitud FOUR ni validez con tiempos constantes diferentes |
| Oráculo residual: 16 evidencias × 16 extensiones de cuatro seeds | **MODEL RESULT** de suficiencia y cobertura de esa fixture | Búsqueda completa con variables existenciales, ciclos o programas asociados generales |
| 64 secuencias de adiciones de dos seeds | Coincidencia de incremental con recomputación, incluidas justificaciones/sellos anteriores | Corrección de ambos respecto de otro matcher o todo lenguaje |
| 512 actualizaciones de composición | Igualdad de eventos, metadata y registros en asociaciones autoancladas acíclicas | Mantenimiento general de activaciones o feedback |
| 4.608 comparaciones por argumento de composición | Monotonía en el orden operacional finito declarado | Identificación con `<=_L` o composición universal de closures con feedback |
| Adjunción evidencia/observables: 64 × 256 pares por orden | **MODEL RESULT** de una adjunción en los mapas finitos elegidos; 240 objetivos sin menor evidencia suficiente | Adjunción verbo/programa, abducción única ni un residual general de M2 |

Fuentes: [modelos FOUR](evidence-pair-models.md), [entailment](four-model-entailment.md),
[consulta sobre programas](program-residual-query.md),
[incrementalidad](incremental-close.md) y [composición](composition-structure-experiment.md).
Las igualdades comprobadas por dos algoritmos que usan match_event no resuelven
F1/F4. Los experimentos de refinamiento tampoco identifican su orden operacional
con L; [la comparación existente](refinement-four-order-comparison.md) conserva
contraejemplos de esa identificación concreta.

## Decisión derivada de la auditoría

No corresponde declarar completado el contrato íntegro ni añadir otra feature
para ocultar las discrepancias. Hay dos trabajos de naturaleza distinta:

1. **Cumplir obligaciones existentes:** scope de tiempos ground, β-reduction
   temporal, tipado de constantes dentro de reglas y distinción entre valores
   runtime y binders. F1–F4 requieren correcciones delimitadas si se decide
   continuar implementación; esta auditoría no las aplica.
2. **Precisar/realizar la parte algebraica pendiente:** fijar el dominio
   proposicional L y su puente con el soporte reusable de K. Los modelos
   experimentales no eligen por sí mismos esa interpretación. Completitud residual
   y derivada general necesitan alcance explícito antes de tratarlas como
   obligaciones de una implementación concreta.

No se encontró una prueba de incompatibilidad del contrato abstracto completo
con todo motor posible. Sí hay resultados que rechazan identificaciones
particulares: I con vacío de K, fusión con acumulación monotónica en knowledge
order, cierre Horn con todo entailment FOUR, y orden operacional con ciertos
órdenes FOUR de programas. La especificación ya separa esas nociones; no se ha
reescrito para hacer desaparecer los hallazgos de código.

Se puede conservar como baseline el **fragmento operativo con restricciones
explícitas**. «Núcleo monotónico terminado» sólo sería correcto tras resolver
esas discrepancias y acordar qué obligaciones algebraicas forman parte del
criterio de cierre, con evidencia adecuada para cada una.
