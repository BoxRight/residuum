# Auditoría final del núcleo monotónico

Fecha: 2026-10-01. Referencia:
[semántica operacional consolidada](monotonic-core-specification.md).
Método: inspección del código actual, argumentos de la especificación, tests
existentes y sus registros de ejecución. No se modificó el motor, no se
ejecutaron compilación, tests ni fixtures y no se diseñaron extensiones.

**Conclusión:** no se encontró una contradicción operacional en el fragmento
inspeccionado respecto de su especificación y precondiciones. Se corrigió una
discrepancia documental sobre sintaxis: el producto se escribe `&` en Formal;
`⊗` es la notación matemática, no un alias aceptado por el lexer actual.
El núcleo puede quedar cerrado para esta fase con sus restricciones explícitas.
Esto no certifica ausencia de bugs ni equivalencia para todas las entradas.

## 1. Qué significa cada clasificación

| Clase | Criterio de esta auditoría |
|---|---|
| **THEOREM** | Definición o argumento matemático identificado, con sus hipótesis; no prueba automática del código |
| **MODEL RESULT** | Resultado de una familia finita enumerada, con carrier y cuantificación declarados |
| **IMPLEMENTED + TESTED** | Camino implementado inspeccionado y comportamiento específico con registro de test aprobado |
| **IMPLEMENTED + LIMITED** | Implementación presente, pero la formulación general necesita restricciones, precondiciones o una API distinta |
| **NOT IMPLEMENTED** | Garantía o mecanismo general ausente del motor productivo; un trait o harness no lo implementa |

Las clases califican afirmaciones concretas. Un argumento de cierre puede ser
THEOREM, su algoritmo tener regresiones IMPLEMENTED + TESTED y su API ser
IMPLEMENTED + LIMITED. No se colapsan esas tres afirmaciones en una etiqueta
de «todo demostrado».

TESTED significa evidencia histórica verificada leyendo registros, no una
reejecución durante esta auditoría. Parte de los registros es anterior a las
correcciones F1–F4. Esas correcciones tienen sus propias regresiones posteriores;
no se afirma que todas las familias antiguas se hayan repetido con el código
actual. La última compilación de tests tampoco demuestra que todos hayan pasado.

## 2. Requisitos de la especificación, T1–T12

| Requisito | Clasificación de su realización actual | Código/evidencia y alcance |
|---|---|---|
| **T1: especialización y β-reduction** | **IMPLEMENTED + LIMITED** | [lower_rule](../src/elab.rs) aplica prefijos de constantes declaradas de tipo exacto, conserva parámetros restantes y sustituye Horn/residual. No implementa aplicación arbitraria ni prueba universal de preservación de tipos. La sustitución temporal tiene regresión F2 aprobada. |
| **T2: identidad e índices** | **IMPLEMENTED + LIMITED** | [Event](../src/typed.rs) define igualdad/orden por verbo, argumentos, PropositionTime y polaridad. Elaboración resuelve tiempos; datos ground bien tipados son precondición de las APIs internas y los structs Rust son públicos. No hay garantía para cualquier Event construido externamente. |
| **T3: realización proposicional general del lattice residuado** | **NOT IMPLEMENTED** | [Traits algebraicos](../src/algebra.rs), intérprete con modelo suministrado y modelos finitos existen; ningún TypedAST/Closure implementa el carrier general, un decisor de ≤_L o equivalencia proposicional general. Las leyes de los modelos se clasifican aparte. |
| **T4: unidad y contexto** | **IMPLEMENTED + TESTED** | [Matcher](../src/derive.rs): Unit devuelve el witness neutro, sin bindings ni eventos adicionales; And comparte bindings. Los axiomas ground se procesan en la ronda inicial. M9 incluye todos los subconjuntos de reglas de su familia, incluido el axioma. I no representa Known vacío. |
| **T5: least Horn closure** | **IMPLEMENTED + LIMITED** | [close](../src/derive.rs) acumula claves y conserva registros hasta que la frontera de claves queda vacía. Requiere reglas/datos válidos y estabilización para retornar; no computa entailment FOUR completo ni garantiza terminación universal. M8/M9 contrastan familias finitas. |
| **T6: activación monotónica** | **IMPLEMENTED + LIMITED** | [close_program](../src/derive.rs) activa asociaciones positivas de verbos Seeded/Derived, fija argumentos/tiempo y conserva llamadas esperando evidencia. Excluye definiciones asociadas del pool ordinario y rechaza módulos con defaults. Effect.program permanece inactivo. No es la modalidad residual o incremental. |
| **T7: evidencia y provenance** | **IMPLEMENTED + TESTED** | Instanciación produce UnstampedDerivation; stamp recibe tiempo externo. El cierre conserva distintas parejas evento/record y no duplica claves Known. M9 compara nuevos registros y sellos previos; las asociaciones incluyen su trigger implícito cuando corresponde. No se exige igualdad de schedule entre ejecuciones. |
| **T8: EvidentialNot** | **IMPLEMENTED + TESTED** | El matcher coteja polaridad; ambos signos pueden coexistir. No hay generación por ausencia ni retractación. M9 incluye signos, conjunción de opuestos y conclusiones negativas. [execute_effect](../src/runtime.rs) rechaza eventos negativos; el modelo FOUR no instala contraposition automática. |
| **T9: frontera de consulta residual** | **IMPLEMENTED + LIMITED** | Las APIs reciben evidencia prestada/inmutable y no sellan requisitos. [query_residual](../src/residual/query.rs) tiene [] ambiguo entre head incompatible y nada pendiente; [query_program_residual](../src/residual/program.rs) distingue resultado sin explicación, Unit y Err. No es satisfacción general de L. |
| **T10: suficiencia certificada** | **IMPLEMENTED + LIMITED** | replay_proof verifica cada paso con instantiate_from_witness y polaridad/admisibilidad de hojas. Grounding/tipos de identidades runtime son precondiciones; no enumera variables del body y exige parámetros determinados por el head. La garantía corresponde a reglas explícitas, no a la activación general de Verb.program. F4 tiene regresión posterior. |
| **T11: alternativas y ausencia de preferencia** | **IMPLEMENTED + TESTED** | El query retiene pruebas separadas, resume alternativas con Join y combina requisitos con Tensor. Deduplica árboles idénticos, no desigualdades semánticas; simplifica I sin eliminar factores repetidos. Las regresiones de alternativas y multiplicidad tienen registros aprobados. No elige una explicación mínima. |
| **T12: mantenimiento incremental aditivo** | **IMPLEMENTED + LIMITED** | [IncrementalClosure](../src/derive/incremental.rs) establece la base con close y actualiza por pivotes de frontera; sus reglas son privadas y fijas. Delta lógico y nuevas justificaciones son distintos. M9 verifica el algoritmo y minimalidad en su familia; eliminaciones, cambios productivos de reglas y activación general no están incluidos. |

No se encontró un caso en esta inspección que contradiga esas garantías dentro
de sus precondiciones. LIMITED no significa que falte una feature requerida
por el fragmento cerrado; delimita qué afirmación general no puede hacerse.

## 3. Argumentos matemáticos frente a implementación

| Afirmación | Clase | Justificación y frontera |
|---|---|---|
| K = P(E), ≤_K es inclusión y ∨_K es unión | **THEOREM** | Definición del dominio abstracto. La representación Rust almacena eventos finitos y metadata separada. |
| T_R es monótono para matching positivo reusable | **THEOREM** | Los testigos de una instancia siguen disponibles al ampliar evidencia; reglas, scope y tipos permanecen fijos. No declara un teorema de verificación del matcher Rust. |
| Close_R(S) = μF.(S ∪ F ∪ T_R(F)) es extensivo, monótono e idempotente | **THEOREM** | Argumento T5 de menor extensión cerrada y cuerpos finitos. Puede ser infinito; retorno del algoritmo requiere estabilización. |
| T_Π de asociaciones positivas es monótono para Π fijo | **THEOREM** | Los triggers y testigos no se retiran. Es la modalidad activa descrita por T6; no una reducción demostrada a la API incremental de reglas fijas. |
| Clave lógica excluye transition, categoría y derivación | **THEOREM** | Definición explícita de logical_key y de Eq/Ord. Tipos y metadata se cotejan aparte; la equivalencia no certifica metadatos operacionales iguales. |
| Mod_D(S,R) = Mod_D(Close_R(S),R) | **THEOREM** | Argumento condicional de soundness en el fragmento ground y políticas de soporte/fusión declaradas. No extiende modelos FOUR a asociaciones arbitrarias. |
| DClose es el fixpoint de delta sobre una base cerrada | **THEOREM** | [Caracterización](discrete-derivative.md): B ∪ D es cerrado y está contenido en toda extensión cerrada de la entrada ampliada. La resta de dos cierres es consecuencia extensional, no algoritmo. |
| Composición secuencial de deltas | **THEOREM** | D_R(S,Δ₁ ∪ Δ₂) = D_R(S,Δ₁) ∪ D_R(S ∪ Δ₁,Δ₂). El segundo utiliza la nueva base cerrada; no afirma aditividad desde una misma base. |
| Suficiencia de una rama con certificado forward válido | **THEOREM** | Inducción sobre las hojas de evidencia/hipótesis y aplicaciones de reglas. Se refiere al cierre Horn abstracto, incluso si otras reglas generan un cierre infinito. |
| Leyes abstractas de un lattice residuado y sus consecuencias | **THEOREM** | Condicionadas a que el modelo suministrado satisfaga los axiomas. Los traits no prueban que una instancia Rust los satisfaga ni eligen L de M2. |

## 4. Resultados finitos M1–M9

| Resultado | Clase | Dominio y qué no demuestra |
|---|---|---|
| **M1: leyes algebraicas** | **MODEL RESULT** | Carriers finitos Z/2Z-powerset, Łukasiewicz de tres valores y realizaciones FOUR; [assert_laws](../src/algebra/tests.rs). No demuestra una realización general del AST proposicional. |
| **M2: soundness/completeness FOUR** | **MODEL RESULT** | 4.016 teorías y 20.080 consultas por configuración de la familia del [informe](four-model-entailment.md). Fusión/filtro I: cero fallos de soundness y 680 de completeness; no instala inferencias algebraicas en close. |
| **M3: composición y orden operacional** | **MODEL RESULT** | [Familias de composición](composition-structure-experiment.md), 4.608 comparaciones por argumento. Orden por inclusión de observables; no se identifica con ≤_L ni generaliza a todo feedback. |
| **M4: propiedades de EvidentialNot en FOUR** | **MODEL RESULT** | Intercambio de componentes, involución, monotonía de conocimiento y antitonicidad de verdad. No es negación compuesta universal del parser/AST. |
| **M5: explicaciones incomparables** | **MODEL RESULT** | Dos ventas distintas justifican el mismo owns en el dominio finito. Incomparabilidad en los órdenes examinados; no un teorema de incomparabilidad en L ni completitud del query para vendedor libre. |
| **M6: composición incremental restringida** | **MODEL RESULT** | 512 actualizaciones de dos familias autoancladas: cierres, registros y deltas coinciden. El adapter es del harness; no implementa actualización asociada general. |
| **M7: adjunción de evidencia y observables** | **MODEL RESULT** | 64 entradas × 256 observables, 16.384 pares por orden; falla la menor evidencia suficiente para 240 objetivos por orden. No es una adjunción universal Verb ↔ Rule. |
| **M8: puente de teorías/modelos** | **MODEL RESULT** | 4.016 teorías por política; cero cambios al saturar, 640 diferencias Horn/mínimo FOUR y 680 literales adicionales. 2.170.880 pares en la adjunción evidencia/modelos y 1.592 triples de fórmulas en seis contextos. No basta la valoración del modelo mínimo para fórmulas generales. |
| **M9: derivada aditiva** | **MODEL RESULT** | 8.192 actualizaciones con reglas fijas, 20.736 adiciones conjuntas de hechos/reglas y 8.192 caminos secuenciales. Dos átomos firmados con tiempo fijo; no generaliza por ensayo a dominios infinitos o activaciones arbitrarias. |

Los tests de esas familias existen y sus registros se conservan. MODEL RESULT
indica el resultado matemático finito observado, no que su interpretación se
haya adoptado como semántica proposicional universal de Residuum.

## 5. APIs y componentes adicionales

| Requisito concreto | Clase | Alcance inspeccionado |
|---|---|---|
| Formal → SurfaceAST → TypedAST | **IMPLEMENTED + TESTED** | Parser Chumsky, elaboración y fixtures actuales. Regresión reciente de SurfaceAST contra parser manual en 25 fixtures; no prueba corrección de toda la gramática. |
| Comentarios Formal | **IMPLEMENTED + TESTED** | //, # y bloques sin anidación; tres tests recientes y la regresión de fixtures. Conservan SurfaceAST/TypedAST, EOF y error de bloque incompleto. CNL sigue pausado. |
| Jerarquía Seeded/Derived/Effect con tiempo igual | **IMPLEMENTED + TESTED** | PropositionType.is_subtype_of exige igualdad de tiempo y usa la tabla nominal de categorías; regresión Effect → Derived/Prop aprobada. No es ≤_L. |
| Distinción tiempo constante/parámetro | **IMPLEMENTED + TESTED** | F1: el scope liga sólo nombres de Rule.parameters; los tiempos restantes son literales. Regresión temporal de forward/replay/incremental aprobada. |
| Constantes y sustitución temporal | **IMPLEMENTED + TESTED** | F2/F3: β-reduction usa Term o Time tipado; los argumentos de reglas cotejan constantes y compatibilidad nominal/opcional. No implica aceptar cualquier subtipo en la aplicación parcial, que exige tipo exacto. |
| RecordLiteral, Set y nested paths | **IMPLEMENTED + LIMITED** | Elaboración de campos/literales y runtime de referencias anidadas existen. Snapshot y términos deben satisfacer el esquema/precondiciones del caller; no se asume un registro runtime universal de tipos de ObjectId. |
| derive ≠ execute y ejecución funcional | **IMPLEMENTED + TESTED** | Forward no llama execute_effect; éste construye otro snapshot clonando Store. Registros aprobados de Transfer, cadena Legal y fallo en segunda operación que preserva input. No exige monotonía del Store. |
| Interpretar requisitos en un modelo suministrado | **IMPLEMENTED + LIMITED** | [interpret_requirement](../src/algebra/requirement.rs) evalúa Unit/Event/Tensor/Join con valoración explícita; no comprueba todos los modelos de una teoría ni decide ≤_L general. |
| Límites de ejecución/selección individual | **IMPLEMENTED + TESTED** | [Runner](../scripts/test_safe.py) verifica cgroup 1 GiB/swap cero, timeout, umbral preventivo y un job/hilo. Los registros leídos son de tests individuales; no equivalen a ejecutar toda la suite. |
| Sintaxis Formal literal ⊗ | **NOT IMPLEMENTED** | El lexer acepta &; ⊗ es notación matemática. La discrepancia se corrigió en documentación, sin ampliar sintaxis. |
| Drafter TypedAST → CNL | **NOT IMPLEMENTED** | Trabajo futuro; el frontend CNL está conservado y sus tests históricos ignorados. |

## 6. Extensiones fuera de las garantías del núcleo

| Extensión | Clase | Situación exacta |
|---|---|---|
| ! como negación por ausencia estratificada | **NOT IMPLEMENTED** | No hay esa operación del lenguaje/motor en el contrato monotónico. |
| Defaults/Reiter con consistencia completa | **NOT IMPLEMENTED** | [Harness de defaults](../src/defaults.rs) conserva aplicabilidad, bloqueo y prioridad experimentales. Su existencia no implementa la consistencia completa ni incorpora defaults al cierre monotónico. |
| Eliminar hechos/reglas con retractación | **NOT IMPLEMENTED** | La acción incremental es unión positiva; retirar soporte requiere otro contrato. |
| API productiva de cambios ΔR | **NOT IMPLEMENTED** | La caracterización aditiva conjunta y el evaluador ground de M9 sólo viven en documentación/tests. |
| Incrementalidad general de Verb.program | **NOT IMPLEMENTED** | IncrementalClosure no consulta α ni mantiene llamadas activadas. El caso autoanclado M6 no cambia esa frontera. |
| Activación automática de Effect.program | **NOT IMPLEMENTED** | La referencia tipada puede existir; la modalidad activa actual no ejecuta ni activa esas asociaciones. |
| Completitud general del residual | **NOT IMPLEMENTED** | No hay prueba/decisor para toda consecuencia Horn, todo entailment FOUR o todos los elementos de L. No se confunde con suficiencia certificada. |
| Residual incremental o derivada sobre Store | **NOT IMPLEMENTED** | No son resultados de DClose ni de M9. |
| Preferencia abductiva/minimalidad general | **NOT IMPLEMENTED** | No se elige hipótesis por cardinalidad, inclusión o ≤_L. Experimentos de minimalidad local no implementan un selector general. |
| interface, Contract, ensures o categoría ProgramVerb nueva | **NOT IMPLEMENTED** | Experimentos de interfaz/refinamiento y asociaciones existentes no añaden esas construcciones. |

## 7. Evidencia de ejecución consultada

Se inspeccionaron las filas de resultados aprobados de los siguientes registros;
no se lanzaron sus comandos de nuevo:

| Registro | Evidencia usada |
|---|---|
| [Modelos algebraicos](../target/test-diagnostics/20260930T232639451467Z-776193/report.md), [pares FOUR](../target/test-diagnostics/20260930T235009647395Z-800727/report.md) y [fusión](../target/test-diagnostics/20261001T024821556133Z-973934/report.md) | Leyes de carriers finitos M1/M4 y límites del orden de conocimiento |
| [F1–F4](../target/test-diagnostics/20261001T073323129009Z-64255/report.md) | Cuatro regresiones de scope, β-tiempo, tipos de constantes y certificado residual |
| [Activación](../target/test-diagnostics/20261001T042741896698Z-1050960/report.md) | Seeds, Derived, I con trigger y espera de evidencia compatible |
| [Regresiones forward](../target/test-diagnostics/20261001T042828532651Z-1051744/report.md) | Matching conjuntivo, especialización, justificaciones y equivalencia seed/hipótesis de sells |
| [Composición](../target/test-diagnostics/20261001T063207593845Z-18451/report.md) | Tres tests de M3/M5/M6/M7 |
| [Teorías/modelos](../target/test-diagnostics/20261001T075651354398Z-81257/report.md) y [ampliación](../target/test-diagnostics/20261001T075945629709Z-83478/report.md) | M8, adjunción de clases y contraejemplo del mínimo |
| [Derivada](../target/test-diagnostics/20261001T082336826196Z-100620/report.md) y [minimalidad](../target/test-diagnostics/20261001T082812026830Z-104205/report.md) | Los tres tests de M9 y repetición del único test ampliado |
| [Residual](../target/test-diagnostics/20260930T234022300123Z-789108/report.md) | Alternativas, multiplicidad y oráculo finito de suficiencia/cobertura |
| [Identidad](../target/test-diagnostics/20260930T222941793484Z-731015/report.md) | Exclusión de metadata operacional de la identidad lógica |
| [Jerarquía/Transfer](../target/test-diagnostics/20260930T192515202111Z-578145/report.md) | Compatibilidad Effect y ejecución funcional |
| [Legal runtime](../target/test-diagnostics/20260930T223411053816Z-734470/report.md) y [fallo funcional](../target/test-diagnostics/20260930T180702958221Z-508456/report.md) | Cadena ejecutable y preservación de input ante error |
| [Comentarios Formal](../target/test-diagnostics/20261001T182712936184Z-505307/report.md) | Tres tests de comentarios y comparación de 25 fixtures con parser manual |

Las propiedades históricas de los otros modelos permanecen acotadas por sus
[tests](../src/algebra/tests.rs) e [informes](evidence-pair-models.md), sin
presentarlas como revalidación universal del checkout actual.

## 8. Decisión de cierre

La separación programa → T_Π, menor cierre, adiciones incrementales, ejecución
de efectos y consulta certificada coincide con las modalidades inspeccionadas.
La diferencia Horn/FOUR, la API residual local ambigua, el uso de cuerpos finitos
con posible cierre infinito y los límites de asociaciones están declarados;
no se clasifican como contradicciones ocultas de este contrato.

La única corrección de esta auditoría aclara notación/sintaxis documental.
No se añade ⊗ al parser ni se cambian leyes algebraicas. Con esa aclaración,
se puede congelar el núcleo operativo de esta fase. Una nueva capa o una
generalización debe declarar su propio contrato y sus relaciones con éste;
un caso que contradiga una garantía dentro de su alcance debe reabrirla como
tal, sin ampliar silenciosamente los fundamentos.
