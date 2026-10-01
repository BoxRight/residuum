# Segunda auditoría del núcleo monotónico

Fecha: 2026-10-01. Referencias: [especificación](monotonic-core-specification.md)
y [auditoría inicial](monotonic-core-audit.md).

Este informe conserva el estado al corregir F1–F4. Los avances posteriores
de teorías/modelos y derivada aditiva están incorporados en la
[semántica operacional consolidada](monotonic-core-specification.md);
las menciones a trabajo pendiente abajo describen el momento de esta auditoría.

**F1–F4 están corregidas en los caminos inspeccionados y sus cuatro regresiones
pasaron.** Esto elimina los contraejemplos concretos de la primera auditoría;
no demuestra que todo el motor esté formalmente verificado. El contrato íntegro
continúa abierto por la realización de L, su puente con Known y las garantías
generales de búsqueda/incrementalidad. Esas cuestiones no se implementaron en
este incremento.

## Correcciones y evidencia

| Hallazgo | Cambio realizado | Resultado de la regresión |
|---|---|---|
| **F1: tiempo constante tratado como variable** | Forward, replay, asociaciones e incremental reciben el scope de parámetros. At(name) sólo liga si name pertenece a ese scope; fuera de él exige igualdad estructural. Los callers de defaults y de tests pasan también su scope explícito. | `A() @ sigma` no satisface `A() @ tau`; `@t` sigue ligándose. El test contrasta forward, replay, consulta local e incremental. |
| **F2: β-reduction temporal incompleta** | La aplicación conserva una sustitución tipada: PropositionTime usa SubstitutionValue::Time y los demás argumentos Term. Elaboración reutiliza la sustitución forward para cuerpo, tiempo proposicional y transición de estado. | La especialización `r(juan,tau)` coincide con su cláusula ground inline tanto en Horn como en residual; la transición queda indexada por tau y after(tau). |
| **F3: constantes mal tipadas en reglas** | Elaboración coteja cada argumento y anotación temporal con el parámetro o constante declarada. Las constantes no se convierten en binders inferidos. El chequeo utiliza compatibilidad nominal y opcional existente. | Se rechaza Conduct donde se exige Person/Thing y como PropositionTime; se acepta Conduct y Movable como payload de Thing? en la cláusula válida. |
| **F4: identidad runtime confundida con parámetro pendiente** | Los parámetros pendientes se comprueban en la plantilla y sus bindings antes de sustituir. Un valor ligado es opaco, incluso si su nombre coincide con el binder; la sustitución no vuelve a recorrer ese valor como plantilla. | El query certifica el goal con identidades `person`, `object` y tiempo `t`, que coinciden con los parámetros. Las hipótesis permiten alcanzar el goal sólo en el cierre hipotético; el cierre real vacío no lo contiene. |

Los cambios se encuentran en [derive.rs](../src/derive.rs),
[incremental.rs](../src/derive/incremental.rs) y [elab.rs](../src/elab.rs).
No se añadieron variantes de Term/TimeExpr, sintaxis ni reglas de inferencia.

La separación tiene un contexto explícito: Rule.parameters define qué nombres
son variables en una **plantilla**. Los términos y tiempos recibidos como eventos
ground son **valores**. Term::Var y TimeExpr::At conservan sus representaciones
históricas; no se infiere scope de un nombre encontrado después de sustituir.
Las APIs internas siguen requiriendo datos ground bien tipados del caller; no
obtienen un registro de tipos de ObjectId por inspeccionar su nombre.

## Revisión de T1–T12

| Obligación | Estado después del incremento | Clase y límite de evidencia |
|---|---|---|
| **T1: sustitución tipada y β-reduction** | F2/F3 resueltas en el fragmento actual: términos, tiempos y transición se sustituyen; constantes de cláusulas se comprueban. | **EXPERIMENTAL OBSERVATION**: inspección y regresiones Horn/residual y tipado. Aplicación continúa siendo por prefijos de constantes de tipo exacto; no hay teorema de preservación para todos los AST construibles externamente. |
| **T2: identidad e índices** | Clave lógica sin cambios. F1/F4 ya no alteran un literal temporal ni rechazan un valor sólo por compartir nombre con un binder. | **THEOREM** local para la clave por definición; **EXPERIMENTAL OBSERVATION** para los nuevos controles. Las estructuras Rust públicas y las precondiciones de entrada siguen siendo límites. |
| **T3: realización proposicional de L** | Pendiente; traits/modelos no se convirtieron en una semántica proposicional. | **MODEL RESULT** histórico para los carriers finitos. No hay implementación general de orden, meet ni equivalencia en M2. |
| **T4: unidad** | Sin cambio: realizada en Unit/Event/And. | **THEOREM** local de neutralidad del matcher; regresiones de este incremento usan I, pero no reejecutan todo el catálogo histórico. |
| **T5: least Horn closure** | Los dos contraejemplos concretos de scope F1/F4 dejan de aplicar. El argumento de cierre sigue condicionado a matching correcto, reglas fijas, datos ground válidos y estabilización si se exige retorno finito. | **THEOREM** abstracto condicional y **EXPERIMENTAL OBSERVATION** de los caminos corregidos. No constituye prueba formal completa de implementación ni terminación general. |
| **T6: activación monotónica** | Sin ampliación: conserva triggers positivos Seeded/Derived; ahora comparte matching scoped. | **THEOREM** condicional e inspección; los **MODEL RESULT** históricos de asociaciones no se reejecutaron. Effect.program sigue inactivo. |
| **T7: provenance** | Se conservan regla, witnesses y bindings. Un tiempo literal ya no produce un binding ficticio; una identidad ligada no impide registrar el paso. | **EXPERIMENTAL OBSERVATION**: la regresión F1 verifica sustitución vacía para una regla ground. No se promete igualdad de schedule o de sellos entre ejecuciones. |
| **T8: EvidentialNot** | Sin cambio semántico: polaridad explícita, sin ausencia, retractación ni explosión automática. | Argumento **THEOREM** sobre fuentes del cierre; modelos/tests históricos no se reejecutaron. |
| **T9: frontera residual** | Inmutabilidad y scope se conservan; el forward/replay ahora respetan el literal temporal igual que el unificador local. | **THEOREM** local de inmutabilidad y **EXPERIMENTAL OBSERVATION** F1/F4. La ambigüedad local `[]` permanece; no se implementó el adjunto general de L. |
| **T10: suficiencia abductiva** | Replay ya admite identidades ligadas con nombres de binders. Mantiene el rechazo de variables del consecuente sin binding; no usa bindings del goal para inventar soporte forward. | **THEOREM** condicional por inducción del certificado; **EXPERIMENTAL OBSERVATION** de la regresión F4. El certificado continúa limitado a reglas ordinarias. |
| **T11: alternativas** | Sin cambio: ramas/pruebas separadas y resumen Join sin preferencia. | **EXPERIMENTAL OBSERVATION** de la API por inspección; **MODEL RESULT** histórico no reejecutado. No implica todas las pruebas posibles ni minimalidad. |
| **T12: incrementalidad** | La API add-only con reglas fijas utiliza el mismo scope que forward. El contraejemplo de tiempo constante también queda bloqueado en update. | Argumento **THEOREM** condicional del pivote delta y **EXPERIMENTAL OBSERVATION** F1. Las 512 actualizaciones históricas no se reejecutaron; derivada general y asociaciones arbitrarias siguen pendientes. |

## Qué permanece pendiente o restringido

No cambia la clasificación de las leyes algebraicas: unidad/asociatividad del
matcher tienen argumentos locales, la conmutatividad de soporte requiere la
interpretación scoped, y la residuación estructural conserva el fragmento
Horn/residual. Los modelos finitos no demuestran esas leyes para todo L.

Continúan abiertos: la realización proposicional de L y su puente general con
Known, satisfacción de fórmulas generales, completitud residual fuera de la
búsqueda acotada y derivada general. Se mantienen la restricción de parámetros
determinados por el goal, los errores por ciclos/límites, ausencia de preferencia
abductiva y separación entre derive y execute. No se amplió la semántica FOUR
ni el cálculo de defaults, ni se introdujo negación por ausencia. Los callers
de defaults sólo se adaptaron a la firma scoped del matcher compartido.

## Verificación efectivamente ejecutada

Sólo formato, compilación de la biblioteca de tests y los cuatro tests elegidos,
individuales, mediante test_safe.py. Se mantuvieron 1 GiB total, swap cero,
un job de Cargo y un hilo del harness. La compilación tardó 7,35 s, con
639,94 MiB RSS y 659,18 MiB de pico de cgroup; no hubo OOM ni detención por límites.

| Test | Exit | Tiempo | Pico RSS |
|---|---:|---:|---:|
| `monotonic_regression_tests::constant_time_is_literal_in_forward_replay_and_incremental_matching` | 0 | 0,0212 s | 15,37 MiB |
| `monotonic_regression_tests::beta_reduction_substitutes_time_in_horn_residual_and_state_indices` | 0 | 0,0214 s | 15,29 MiB |
| `monotonic_regression_tests::rule_constants_keep_declared_types_in_arguments_and_time` | 0 | 0,0212 s | 15,38 MiB |
| `residual::program::tests::forward_certificate_preserves_runtime_identities_that_share_parameter_names` | 0 | 0,0213 s | 15,27 MiB |

Código de las regresiones: [monotonic_regression_tests.rs](../src/monotonic_regression_tests.rs)
y [program/tests.rs](../src/residual/program/tests.rs).
Registro del runner: [report.md](../target/test-diagnostics/20261001T073323129009Z-64255/report.md).
Los nueve tests CNL permanecen ignored. No se ejecutó la suite completa ni otros
tests seleccionados. Compilar todos los tests no demuestra que todos pasen.

El siguiente estado de proyecto es, por tanto, **cuatro discrepancias concretas
corregidas con evidencia regresiva limitada**, con las obligaciones semánticas
abiertas explícitamente conservadas.
