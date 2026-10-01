# Residuum

El pipeline activo es:

```text
Formal → SurfaceAST → TypedAST → derivation/runtime
```

Formal admite comentarios de línea `// ...` y `# ...`, y bloques `/* ... */`
de una o varias líneas, sin anidación. Se ignoran entre tokens, sin alterar el
AST; un bloque sin cerrar produce un error. Véase
[Transfer con comentarios](examples/transfer/comments_formal.res).

La [especificación del núcleo monotónico y su semántica operacional](docs/monotonic-core-specification.md)
fija el fragmento congelado para esta fase: programa → operador de consecuencia,
cierre Horn, activación asociada, derivada aditiva y consulta residual separada
del forward. Distingue obligaciones **THEOREM**, resultados **MODEL RESULT** y
comportamiento **EXPERIMENTAL OBSERVATION**, con dominios y límites explícitos.
Consolida el puente de teorías/modelos y la auditoría; eliminaciones, activación
incremental general y completitud residual permanecen como preguntas abiertas.

La [auditoría final](docs/monotonic-core-final-audit.md) clasifica las garantías,
la implementación y los resultados finitos por separado. Permite cerrar esta
fase con el alcance declarado, sin atribuir completitud general ni verificación
formal a los tests existentes.

El [puente entre evidencia, teorías y modelos](docs/theory-model-bridge.md)
compara soporte Horn y FOUR-fusión sobre clases de modelos. En el fragmento
ground probado, saturar hechos con Horn preserva la clase de modelos; interpretar
fórmulas generales requiere conservar esa clase, incluso si hay un modelo mínimo.
Este experimento no elige una semántica definitiva de M2 ni amplía el motor.

La [derivada discreta de close](docs/discrete-derivative.md) caracteriza el delta
como un fixpoint sobre eventos nuevos. Se contrasta con el mantenimiento existente
y, sólo en el harness, con adiciones conjuntas de evidencia y reglas. Distingue
composición secuencial de aditividad y conserva el alcance de la API con reglas fijas.

CNL se conserva como frontend experimental pausado. Sus tests históricos están
marcados `ignored`; Formal/CNL equality deja de ser requisito de cierre mientras
evoluciona el lenguaje. La línea futura es un drafter en dirección
`TypedAST → controlled natural language / legal prose`.

Los [programas experimentales ejecutables](docs/executable-fixtures.md) llevan
declaraciones, datos y consultas en el propio archivo Formal. La fixture
[ResidualProof](examples/experiments/residual_proof.res) contiene los tres
escenarios de seeds; [FourCounterexample](examples/experiments/four_counterexample.res)
muestra `Horn: false` y `FOUR fusion: true` para `~P`. Se ejecutan individualmente
con `python3 scripts/test_safe.py --fixture <archivo.res>`, bajo los mismos límites
de compilación y ejecución. `close` permanece como consecuencia Horn explícita.

El [expediente Legal monotónico](examples/legal/case_monotonic_formal.res) reúne
en un solo programa las declaraciones, reglas y ocho escenarios de actos,
cumplimiento, violación, patrimonio, titularidad, capacidades, legislación y
anulación. Autoridad y vigencia se comparan mediante datos explícitos; las marcas
de anulación y extinción conservan los hechos originales. Sus conclusiones son
`Derived`. Para observar exclusivamente el cierre forward:

```sh
python3 scripts/test_safe.py --fixture examples/legal/case_monotonic_formal.res --forward
```

La [activación de programas asociados a verbos](docs/active-verb-programs.md)
usa `close_program` para consultar `Verb.program` cuando una instancia Seeded o
Derived entra en Known. La fixture [SellsAssociatedProgram](examples/experiments/sells_associated_program.res)
se observa con `python3 scripts/test_safe.py --forward --fixture <archivo.res>`.
La derivación conserva sustituciones y provenance; las asociaciones Effect
permanecen inactivas y no se ejecutan transformaciones de estado.
La fixture [SellsComposedPrograms](examples/experiments/sells_composed_programs.res)
prueba `sells → owns → notDo`: el `owns` derivado activa su propio programa,
especializado mediante β-reduction. Se compara con
[reglas inline](examples/experiments/sells_composed_inline.res) sin asociaciones.
El [experimento de composición como estructura](docs/composition-structure-experiment.md)
comprueba deltas, monotonía en un orden operacional finito y el alcance de las
adjunciones entre evidencia y observables, sin identificar ese orden con `≤L`.

La [actualización incremental de close](docs/incremental-close.md) mantiene una
base cerrada bajo reglas fijas, usando una frontera de eventos nuevos. El programa
[ForwardIncremental](examples/experiments/forward_incremental.res) compara
`close(S ∪ ΔS)` con `close(S) ∪ Dclose(S, ΔS)` mediante
`python3 scripts/test_safe.py --fixture examples/experiments/forward_incremental.res --incremental`.
El modo externo reutiliza los bloques existentes sin cambiar la gramática ni
ejecutar consultas residuales o efectos. La identidad lógica y las justificaciones
se comparan por separado; los tiempos de derivación proceden de un reloj externo.

El experimento [OwnsProgram](examples/experiments/owns_program.res) compone
`transfer → owns → notDo(charlie, molest, car, bob)` con dos tiempos `after`.
Especializa una regla-lambda existente mediante β-reduction y reutiliza el cierre
incremental. El [análisis del experimento](docs/owns-program-experiment.md)
distingue la proposición owns de la regla que expresa sus consecuencias;
no introduce una categoría ProgramVerb ni generación de reglas como valores.

El [experimento de reutilización](docs/owns-reuse-experiment.md) separa la lambda
en [Property](examples/experiments/property.res) y su referencia en
[Protection](examples/experiments/protection.res). Compara su β-reduction, cierre,
provenance y deltas con una referencia inline. `--library` compone estos archivos
en el harness externo; no introduce imports en el lenguaje.

El [experimento de interfaz/implementación](docs/owns-interface-experiment.md)
separa una garantía Horn de owns, dos cuerpos y un consumidor. Un checker de
tests verifica igualdad de la cláusula tipada tras β-reduction antes de entregar
la interfaz al consumidor; la garantía no participa en verificar su propio cuerpo.
Es un criterio suficiente de este caso, sin sintaxis Contract ni un checker
general incorporado al compilador.

El [refinamiento finito de owns](docs/owns-refinement-experiment.md) verifica las
consecuencias exigidas mediante el cierre real de la implementación sobre ocho
casos ground. Acepta protección derivada en varios pasos y consecuencias adicionales;
no identifica aún esa relación operacional con el orden algebraico ≤L.

`Legal` queda congelado como caso de regresión. Las fixtures de
`Residual` conservan el experimento de estructura tipada y transformación:
[horn_formal.res](examples/residual/horn_formal.res) expresa `A & B <= C` y
[curried_formal.res](examples/residual/curried_formal.res) expresa `A <= B -o C`.
`⊸` es el equivalente Unicode de `-o`. Este primer fragmento tiene un solo nivel
de residual; `A` y `B` pueden ser conjunciones y `C` es un evento.

El AST conserva `RuleBody::ResidualClause` con `Residual { required, consequent }`.
`residuate(&rule)` separa la conjunción exterior de la cláusula Horn y conserva
nombre, parámetros, tiempos, polaridad y estructura de ambos operandos;
`unresiduate(&rule)` reconstruye esa misma cláusula. Esta transformación estructural
es el primer test de la ley `A & B <= C` ⇔ `A <= B ⊸ C`; todavía no define una
álgebra residuada general.

Ambas formas pasan por el mismo chequeo M2 sobre `A & B` y `C`. La β-reduction
conserva el cuerpo residual al especializar parámetros. Para forward inference,
el matcher utiliza únicamente la forma Horn equivalente: necesita evidencia de
`A` y de `B`, conserva la misma provenance y nunca inserta un residual en `Known`.
La transformación no inventa evidencia para `B` ni ejecuta efectos.

La unidad monoidal `I` aparece explícitamente como `SurfaceAntecedent::Unit` y
`Antecedent::Unit`; la fixture [unit_formal.res](examples/residual/unit_formal.res)
la usa en Horn y en ambos operandos del residual. Es la unidad del producto
lógico, con un único testigo neutro: conserva los bindings existentes y no agrega
eventos, tiempos o evidencia. No es una proposición del mundo sin índice temporal.
En sintaxis consume el token `I`; `I(...) @ t` sigue siendo una aplicación de verbo
si existe esa declaración. Las conjunciones mantienen separadores obligatorios.

El matcher satisface las leyes de unidad `I & A` y `A & I`: producen exactamente
los matches, sustituciones y evidencias de `A`. El AST conserva esos operandos
para que `residuate` y `unresiduate` sigan siendo transformaciones reversibles;
la igualdad aquí es semántica, no igualdad textual de árboles. Esto incorpora
la unidad al fragmento, sin afirmar una implementación completa de toda el álgebra.
`close` realiza una primera ronda incluso con seeds vacías, de modo que una regla
ground `I <= C` derive `C` con provenance de antecedente vacío. `I` nunca entra
como evento en `Known`.

La consulta operacional es `query_residual(&rule, &goal, &known)`:
para `requests(alice, car) @ tau` y el objetivo `eligible(alice, car) @ after tau`,
devuelve el requisito `available(car) @ tau`. El resultado es un
`ResidualRequirement { rule, evidence, goal_substitution, substitution, goal,
required, parameters }`, separado de `Event`, `DerivedEvent` y `Closure`.
Primero unifica el consecuente con el goal ground; después reduce el antecedente
mediante evidencia compatible. `goal_substitution` conserva las restricciones del
goal; `substitution` incorpora también bindings de evidencia. Los parámetros que
siguen libres conservan su scope y tipo. Los identificadores runtime que coincidan
con nombres de parámetros no se vuelven variables por esa coincidencia.

La consulta recibe `Known` por referencia y no afirma el requisito ni el objetivo,
asigna `DerivationTime` o modifica snapshots. Con `Known` vacío devuelve el
antecedente completo instanciado; los bindings proceden del goal. El producto de
cero factores de evidencia utilizada es `I`, pero `Known` vacío no es la unidad
lógica. Con evidencia parcial conserva sólo el requisito
pendiente. `I` como requisito ya está satisfecho. Un resultado vacío puede indicar
un head incompatible o un antecedente completamente satisfecho; no afirma el goal.
`close` sigue siendo quien puede derivar `C`.
La ausencia de evidencia nunca produce `EvidentialNot`.

La API consulta una sola regla Horn o residual. La reducción es local; no realiza
búsqueda entre reglas ni garantiza requisitos globalmente mínimos o todas las
alternativas abductivas. Los matches compatibles conservan sus testigos separados.

La API `query_program_residual(&program, &evidence, &goal, limits)`
busca recursivamente entre las reglas del módulo. Devuelve ramas con testigos y
un resumen `RequirementFormula` con `Unit`, eventos, `Tensor` y `Join` para las
alternativas. `Verb::is_abducible` fija `Seeded ⇒ Abducible` por defecto, con
hipótesis de ambas polaridades explícitas; no convierte ausencia en evidencia
negativa. `query_program_residual_with_abducibles` permite restringir esa lista
por verbo y polaridad. La
búsqueda exige parámetros determinados por el head; ciclos, variables pendientes
y límites agotados devuelven error, sin resultados truncados. Cada resultado
se filtra contra los abducibles y se certifica mediante replay finito con el
matcher forward del programa. `failure` distingue `NotAbducible` de
`NotForwardSupported`; ambas condiciones dejan `required: None`. Conserva las ramas
sin preferencia por `≤_L`, y no modifica el conocimiento ni ejecuta efectos.
El [contrato de la consulta sobre programas](docs/program-residual-query.md)
precisa ese alcance y sus tests de suficiencia y cobertura finita.
El bloque abductivo queda congelado sin preferencia. El experimento de valoración
y satisfacción compara el puente con el cierre ground, antes de incrementalizar
el cierre forward y sin cambios a esta búsqueda.

`Event` tiene identidad lógica `(verb, arguments, propositionTime, polarity)`.
Su igualdad y orden ignoran `transition` y la clasificación proposicional; los
chequeos de tipos siguen siendo independientes. El cierre deduplica por esa clave,
incluidas las seeds de entrada. Conserva la primera representación sin fusionar
transiciones y mantiene las justificaciones de derivación separadas. La ejecución
sigue exigiendo tiempos de estado resueltos y coherentes; las aserciones sobre
metadata operacional deben comparar `transition` explícitamente.

El test externo `legal_sales_residual_requirement_then_real_evidence_derives_give`
usa la fixture original `Legal.sales`: con sólo `agreesPrice(alice, car, bob) @ tau`,
consultar `give(alice, car, bob) @ after tau` devuelve el requisito
`agreesObject(bob, car, alice) @ tau`. La consulta conserva el cierre original;
cuando el caller aporta esa evidencia, desaparece el requisito pendiente y
el fixpoint forward deriva `give` con los dos antecedentes y `DerivationTime`.

El orden de trabajo del núcleo monotónico queda fijado: representación y
transformación residual; después consulta/abducción residual; puente de valoración
y satisfacción; incrementalidad
forward y luego residual; finalmente evaluar la vista de verbo-programa.
`!` estratificado y consistencia de defaults/Reiter quedan para después.

El [contrato algebraico del núcleo monotónico](docs/monotonic-core-laws.md) fija
el objetivo de un lattice residuado conmutativo `(L, ∨, ∧, ⊗, I, ≤, ⊸)`, las
leyes del orden, lattice, monoide y adjunción, junto con su cobertura actual.
[src/algebra.rs](src/algebra.rs) define la interfaz compartida;
[sus property tests](src/algebra/tests.rs) recorren exhaustivamente dos modelos
finitos de referencia donde tensor y meet son distintos. Los traits no prueban
las leyes y los modelos todavía no interpretan `TypedAST`.

El [experimento de orden de requisitos](docs/requirement-order-experiment.md)
interpreta el `Join` y el `Tensor` de una consulta real mediante
`algebra::interpret_requirement`, en modelos y valoraciones explícitos.
`Preorder::compare` usa el mismo orden en ambas direcciones, sin añadir preferencia.
La fixture [order_formal.res](examples/residual/order_formal.res) muestra que
`A` y `A ⊗ B` pueden quedar ordenados en cualquier dirección, ser equivalentes
o incomparables según el modelo. `B ≤ I` basta para justificar `A ⊗ B ≤ A`;
no adoptamos integralidad ni seleccionamos explicaciones todavía.

El [experimento con pares de evidencia](docs/evidence-pair-models.md) construye
FOUR con órdenes separados de conocimiento y verdad, y EvidentialNot como
intercambio de componentes. Dos productos pasan las leyes residuadas: meet de
verdad y fusión. Los tests conservan sus diferencias de unidad, monotonía y
compatibilidad con negación, junto con contraejemplos del puente hacia Horn.
Son modelos de referencia; no cambian la semántica de `query` ni `close`.

Distinguimos `⊥_K` (conocimiento vacío), `I` (unidad lógica) y `0_L` (bottom lógico,
si el modelo lo incluye). En FOUR, el conocimiento vacío asigna U a cada átomo;
la unidad es B en el modelo de fusión y T en el modelo de meet, y el bottom de
verdad es F. No se fija `I=B` para M2. El
[experimento de satisfacción](docs/satisfaction-bridge-experiment.md) comprueba
que soporte positivo coincide con matching y cierre ground en ambos modelos.
También conserva un cierre Horn estable que incumple la desigualdad puntual en
el orden de verdad. Esa desigualdad no sustituye la consecuencia operacional.

La [comparación por modelos FOUR](docs/four-model-entailment.md) enumera 4.016
teorías pequeñas por configuración, más familias con tensor, cadenas, ciclos y
conclusiones opuestas. Horn resulta sound para sus consultas atómicas, pero
incompleto frente a reglas de verdad: `{~Q, P <= Q}` implica semánticamente `~P`
y Horn no lo deriva. Con satisfacción por `I ≤ valor`, regla y fórmula residual
comparten el mismo entailment. Los controles de soporte coinciden con Horn,
pero no conservan toda esa identificación algebraica.

Ése es el criterio para las futuras consultas: encontrar átomos faltantes no
demuestra por sí solo que se calcule el adjunto. La conmutatividad se adopta
explícitamente; evidencia reutilizable, conocimiento y ejecución necesitan un
puente propio. El join lógico no se identifica con la unión de `Known`, ni la
unidad monoidal con el mínimo del dominio de conocimiento.

El [harness de LegalAbduction](docs/legal-abduction-harness.md) reutiliza la fixture
`sales_delivery_formal.res` y encadena consultas `delivery → sales` con una lista
externa de acuerdos abducibles. Con `Known=∅`, `{A}` y `{A,B}`, obtiene los
requisitos `A ⊗ B`, `B` e `I`, respectivamente; verifica suficiencia en un cierre
hipotético separado y comprueba
que ninguna hipótesis individual puede eliminarse en estos casos. Es un harness
acotado de tests, sin sintaxis nueva, selección general mediante `≤_L`, alternativas
ni evaluación de inconsistencias. No demuestra una realización completa del álgebra.

La fixture [sales_delivery_formal.res](examples/legal/sales_delivery_formal.res)
reúne `sales`, `traditio` y la especialización `delivery = traditio(deliver)`.
El cierre deriva `give @ after(tau)` y `do @ after(after(tau))`; la ejecución
explícita aplica cada efecto al snapshot con el tiempo de entrada correspondiente.
Derivar un efecto no modifica el estado.

`give(subject, object, recipient)` desplaza exclusivamente el `Thing` entre
`subject.patrimony.assets` y `recipient.patrimony.assets`. `do(debtor, deliver,
object, creditor)` crea la relación de entrega en las obligaciones del deudor
y los derechos del acreedor, sin mover assets. La regla `delivery` conecta ambas
proposiciones; ejecutar `give` por sí solo no crea relaciones.

La fixture [sales_retention_formal.res](examples/legal/sales_retention_formal.res)
usa exactamente los mismos antecedentes en `sales` y `retention`. La segunda
deriva provisionalmente `do(debtor, deliver, object, creditor)` para observar una
relación sin desplazar assets; aún no es un modelo definitivo de reserva de dominio.
`RetentionSale = exception retention to NormalSale` conserva referencias a la
lambda `retention` y al default desplazado. Las referencias deben estar declaradas
antes y ambas firmas deben tener los mismos tipos de parámetros, en el mismo orden.

`select_active_rules(&module, &["NormalSale", "RetentionSale"])` se conserva como
harness provisional de prioridad: recibe candidatos externos y omite aplicabilidad,
bloqueo y consistencia. Devuelve sólo la lambda `retention`; con sólo `NormalSale`, devuelve
`sales`. Esta función no implementa por sí sola el cálculo supernormal.

La fixture [sales_retention_when_formal.res](examples/legal/sales_retention_when_formal.res)
añade `when retentionAgreed(debtor, object, creditor) @ t` a la excepción.
El AST conserva `condition: Option<Antecedent>` separado de la referencia de prioridad.
Las condiciones reutilizan los eventos, conjunciones y tiempos de M1; sus términos
deben ser parámetros de la lambda o constantes tipadas y sus tiempos deben ser
parámetros `PropositionTime`. No se introducen variables locales nuevas en este paso.

`evaluate_defaults(&module, &known)` recibe sólo proposiciones conocidas, por ejemplo
`Closure.known`. Una condición necesita al menos un match; la conjunción comparte
una sustitución compatible. Sin condición, el default es siempre candidato (`top`).
La `DefaultExtension` registra los candidatos con sus eventos y sustituciones de
aplicabilidad, los defaults desplazados por prioridad y los seleccionados. Su campo
`consistency` es explícitamente `NotChecked`: todavía no verifica consistencia M1↔M2
ni calcula un fixpoint conjunto entre defaults y Horn.

```rust
let extension = evaluate_defaults(&module, &known)?;
let closure = close(seeds, extension.rules(), &mut clock);
```

La selección permanece a nivel del módulo: cualquier match hace candidata a la
lambda completa, y `.rules()` conserva esa parametrización. No selecciona por
instancia ground ni ejecuta efectos. La evidencia de aplicabilidad permanece en
la extensión, separada de la provenance de derivación de `close`.

La fixture [sales_blocking_formal.res](examples/legal/sales_blocking_formal.res)
declara marcas `Derived` ordinarias y relaciona cada marca con un default concreto:

```text
default NormalSale =
    supernormal sales
    blocked when
        inconsistentSale(debtor, object, creditor) @ t
```

El AST conserva `blocking: Option<Antecedent>` separado de `condition` y prioridad.
La evaluación sigue **aplicabilidad → bloqueo → prioridad → lambdas seleccionadas**.
Sólo se chequean bloqueadores de defaults aplicables, contra el mismo `Known`
recibido. Cada match conserva un `DefaultBlock { default, evidence, substitution }`
en `DefaultExtension.blocked`; se conservan todas las justificaciones de bloqueo.
Un default bloqueado no aporta prioridad, tampoco como eslabón de una cadena de
excepciones. Por eso una excepción bloqueada deja disponible su base no bloqueada.
`defeated` registra únicamente el desplazamiento por prioridad, separado de `blocked`.

`blocking_scope = BlockingScope::RuleLambda` es una limitación provisional explícita:
cualquier sustitución que satisfaga el bloqueador excluye la lambda completa durante
esa evaluación, incluso si describe otra venta o tiempo. No se bloquean instancias
ground ni se modifica la regla. El caller puede derivar primero las marcas mediante
reglas ordinarias, evaluar defaults sobre ese cierre y luego pasar las lambdas
seleccionadas a `close`. El evaluador no deriva nuevas marcas ni itera con Horn;
`consistency: NotChecked` sigue señalando que la consistencia general M1↔M2 está pendiente.

La fixture [evidential_not_formal.res](examples/transfer/evidential_not_formal.res)
introduce `~event` como `Polarity::EvidentialNot`: evidencia negativa explícita,
con el mismo tipo proposicional, roles y tiempo que el evento correspondiente.
No es negación por ausencia. Las seeds negativas entran mediante la API externa
de eventos; no se añade una sintaxis de programa para cargar seeds.

El matcher exige la misma polaridad y la sustitución la conserva, incluso tras
especializar una regla por β-reduction. Una regla puede derivar evidencia negativa
de un verbo `derived` o `effect` (ya `Effect <: Derived`), con `derived_at` y provenance normales;
un verbo `seeded` no puede ser consecuente negativo. El cierre acumula `P` y `~P`
como dos eventos distintos, sin retractación ni chequeo de contradicciones.
`execute_effect` rechaza `EvidentialNot`: la proposición negativa no ejecuta el
`StateTransform` del efecto positivo. `!` y negación estratificada siguen pendientes.

`derived verb` declara una proposición `Derived(t) <: Prop(t)` pura, sin cuerpo
operacional ni transición de estado. La fixture
[inconsistent_formal.res](examples/transfer/inconsistent_formal.res) usa esta
declaración ordinaria para derivar `inconsistent(...) @ after t` únicamente mediante
una regla con `delivers(...) @ t & ~delivers(...) @ t` como antecedentes. Ambos
eventos permanecen en el cierre. Sin esa regla, la oposición no deriva ninguna marca.
`inconsistent` no es un nombre reservado: la misma construcción puede llamarse
`violation` o cualquier otro verbo declarado. Estas conclusiones pueden servir como
antecedentes de otras reglas, con provenance normal, pero no son ejecutables como
efectos. Su presencia sólo bloquea un default si coincide con su `blocked when`
declarado; ningún nombre de verbo concede un privilegio implícito.

En runtime, `Store` asocia `(ObjectId, FieldName)` a `Value`. Los campos que
conectan entidades, como `Person.patrimony`, contienen referencias `ObjectId`;
los paths anidados siguen esas referencias. Los records construidos por un
efecto son valores con nombre de entidad y campos, y los conjuntos los comparan
estructuralmente, independientemente del orden de los campos del literal.
`none` se materializa como `Value::None`; una constante como `deliver` conserva
su nombre en `Value::Const`, sin inventar una inicialización. Los valores presentes
de un optional se almacenan directamente como su payload.

`execute_effect(event, verb, input)` recibe un evento ground y un verbo ya
tipado, junto con un store inicializado por el caller. Comprueba los tiempos
resueltos de entrada y salida, sigue los paths y exige conjuntos en los destinos.
Devuelve un snapshot nuevo: ni una ejecución correcta ni un error intermedio
modifican `input`. La API no añade un planificador de efectos ni valida por sí
misma todo el esquema del store.

Para verificar cambios, usa el runner con límites de recursos:

```sh
python3 scripts/test_safe.py --test tests::close_uses_beta_reduced_delivery_rule
```

El runner requiere Linux, cgroup v2 y una sesión de systemd del usuario. Compila
sin ejecutar tests, con un trabajo de Cargo y timeout de 45 segundos. Después
ejecuta cada test seleccionado en un proceso y cgroup separados, con 1 GiB total,
sin swap, un hilo del harness y timeout de 15 segundos.

Se detiene ante un fallo, timeout, consumo de 768 MiB o crecimiento de al menos
64 MiB en un segundo cuando el test supera 128 MiB. Si no puede imponer los
límites, retorna error sin compilar ni ejecutar el binario. Los registros de exit
status, tiempo y pico RSS quedan en `target/test-diagnostics/`.

Puedes repetir `--test` para seleccionar varios tests, usar `--binary PATH` para
mantener un binario ya compilado, o usar `--compile-only`, `--fmt-check` y
`--format` para separar las fases. Sin `--test`, ejecuta todos los tests activos
individualmente; esa opción debe respetar la autorización vigente del usuario.

El runner limita sus propios procesos; una invocación directa de `cargo test`
no adquiere esos límites. No uses esa invocación para las verificaciones del
proyecto.
