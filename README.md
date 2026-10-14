# Residuum

La versión semántica **v1** queda congelada en el tag Git `v1`.
Véanse [su alcance y exclusiones](docs/v1-scope.md) y la
[referencia modular LegalMonotonicCase](docs/legal-monotonic-v1.md).
La versión Cargo sigue siendo `0.1.0`; v1 no afirma completitud general.

El pipeline activo es:

```text
Formal → SurfaceAST → TypedAST → derivation/runtime
```

La [primera teoría experimental de la geometría inferencial](docs/inferential-geometry-experimental-theory.md)
reúne los estudios de clausura jurídica, bases extremas, fronteras mínimas e
identificabilidad mediante particiones. Distingue teoremas condicionados,
resultados finitos y preguntas abiertas; no modifica la ontología ni afirma una
geometría universal para todos los programas de Residuum.

El [experimento de recalcitrancia y funciones justificativas](docs/legal-recalcitrance-justificatory-functions.md)
mantiene la norma fija y filtra anticadenas de soportes mediante valoraciones
externas hipotéticas. Separa compatibilidad, identificabilidad y adecuación
jurídica; ninguna revisión se integra automáticamente.

La [continuación conversacional](docs/legal-conversational-justification-learning.md)
actualiza las once funciones compatibles con una séptima valoración estipulada,
conserva los empates entre preguntas minimax y diagnostica espacios vacíos sin
inventar funciones ni modificar la norma.

La [evaluación jurídica de las preguntas minimax](docs/legal-minimax-questions-juridical-assessment.md)
distingue deberes y pretensiones correlativas, permisos y evidencia de realización.
Formula las objeciones posibles sin añadir valoraciones al experimento.

El [experimento de expedientes y explicaciones tipadas](docs/legal-typed-explanations-fixed-theory.md)
calcula el cierre íntegro de LegalGeneral antes de evaluar explicaciones mediante
resultados compatibles, incompatibles o indeterminados. No reemplaza productores
de consecuencias ni convierte esas evaluaciones en hechos jurídicos.

El [caso histórico Riggs v. Palmer](docs/legal-riggs-fixed-reference-experiment.md)
certifica una atribución sucesoria literal bajo supuestos delimitados y examina
argumentos documentados sobre sus cierres. Conserva la teoría de referencia,
provenance y preguntas contrafactuales para deliberación jurídica independiente.

El [aprendizaje conversacional de argumentos de Riggs](docs/legal-riggs-conversational-argument-learning.md)
separa apoyo factual, valoración externa y permanencia de hipótesis. Comprueba
ramas hipotéticas de respuesta, particiones y preguntas posteriores conservando
los mismos cierres jurídicos.

El [aprendizaje experimental de reglas sobre Riggs](docs/legal-riggs-normative-rule-learning.md)
genera 64 cláusulas relacionales y conserva 16 compatibles con la observación
histórica. Sus impedimentos hipotéticos preservan la teoría literal; se verifican
fronteras, particiones y geometría sin resolver automáticamente la exclusión efectiva.

El [contrato de búsqueda relacional](docs/legal-relational-search-space-contract.md)
y la [enumeración estructural acotada](docs/legal-structural-generation-enumeration.md)
distinguen admisión tipada de adecuación normativa. La enumeración certifica un
subperfil y documenta la corrección pendiente de finitud por inserciones de `I`.
La [revisión de integración](docs/legal-geometry-integration-review.md) registra
36 tests individuales y las fuentes ontológicas locales verificadas.

La CLI mínima usa las declaraciones, seeds y queries del archivo existente:

La [evaluación por pasos](docs/stepwise-evaluation.md) añade `--steps N` y
`--trace` a los mismos comandos. Expone consecuencias inmediatas síncronas,
fronteras de búsqueda residual, refinamiento de modelos FOUR y fases de defaults;
una traza acotada distingue siempre cierre completo de prefijo incompleto.

```sh
./residuum run examples/legal/case_monotonic_formal.res
./residuum explain examples/legal/case_monotonic_formal.res
./residuum query examples/experiments/residual_proof.res
./residuum query examples/experiments/four_counterexample.res
```

`run` observa el cierre forward mediante `close_program`; `explain` presenta un
grafo de justificaciones del objetivo de cada escenario, con todas las reglas,
antecedentes, sustituciones y `derivedAt`. Distingue seeds y objetivos no derivados.
Ambos hacen derivación, sin ejecutar transformaciones de estado. En archivos con
defaults, cierran primero las reglas obligatorias, evalúan la selección una vez
y calculan el cierre con las lambdas seleccionadas. `explain` muestra además
aplicabilidad, evidencia de bloqueo, sustituciones y desplazamiento por prioridad.

`query` ejecuta el tipo de consulta declarado (`residual` o `fourFusion`) con su
contrato y límites actuales. Los requisitos son hipótesis, no nuevos hechos.
El residual usa la vista de reglas declaradas y no se identifica con la activación
forward de programas asociados. El expediente Legal se observa con `run` y
`explain`: sus parámetros auxiliares no determinados por el goal exceden el
fragmento de búsqueda residual existente, y `query` conserva ese error.

El ejecutable `residuum` en la raíz es el launcher seguro. Compila incrementalmente
y ejecuta el worker mediante `scripts/test_safe.py`: 1 GiB total, sin swap, timeout
de 45 s para compilación y 15 s para ejecución; se detiene al llegar a 768 MiB o
detectar crecimiento anormal. Requiere Linux/cgroup v2 y systemd de usuario y falla
si no puede imponer los límites. El worker Rust comprueba RAM/swap antes de leer
el archivo. La salida del programa va a stdout y los diagnósticos a stderr.
`./residuum --help` muestra los comandos. Para usar `residuum` desde otra carpeta,
añade la raíz del repositorio al `PATH`. Los archivos admitidos son Formal, de
hasta 1 MiB de fuente total entre los archivos únicos del grafo de imports,
incluidos el archivo raíz y los comentarios, con al menos un bloque `experiment`
local para ejecutar. Esta cota de entrada es configuración de implementación, no
una restricción semántica; el tag `v1` conserva su límite de 64 KiB. La resolución
admite hasta 64 archivos únicos y conserva los mismos controles de recursos.
El launcher mantiene además el control independiente de 64 KiB para el archivo
raíz en `test_safe.py --fixture`; las dependencias comparten la cota de 1 MiB.

La CLI y sus tests viven en el target binario, separado de los tests de la
biblioteca para reducir el consumo de compilación. Sus tests se seleccionan
individualmente con `scripts/test_safe.py --test-target cli --test <nombre>`.

Formal admite comentarios de línea `// ...` y `# ...`, y bloques `/* ... */`
de una o varias líneas, sin anidación. Se ignoran entre tokens, sin alterar el
AST; un bloque sin cerrar produce un error. Véase
[Transfer con comentarios](examples/transfer/comments_formal.res).

Formal admite imports de rutas relativas al archivo que los declara:

```text
module Sales
import "./legal_monotonic.res"
```

Cada dependencia se parsea y elabora por separado con sus propios imports.
El módulo consumidor recibe declaraciones ya tipadas; no se concatenan textos
ni SurfaceASTs. Todas las declaraciones salvo `experiment` están disponibles,
incluidas las dependencias transitivas. No hay `export`, aliases, nombres
cualificados ni imports selectivos. Los imports se resuelven antes de elaborar
las declaraciones locales; su orden conserva el orden de enlace de las reglas.
No se añade resolución de referencias locales hacia adelante.

La identidad del archivo es su ruta canónica: imports repetidos o compartidos
por dos dependencias no duplican símbolos. Rutas inexistentes o absolutas,
ciclos, símbolos desconocidos y nombres duplicados producen error. El espacio
de nombres es único, también entre categorías de declaración. Un módulo no
puede obtener símbolos de su consumidor ni de otra dependencia que no importe.
Los `experiment` se comprueban en el módulo que los declara y nunca se importan
ni aportan seeds al consumidor.

La API con acceso a archivos es `load_formal_module(path)`; la CLI y el runner de
fixtures la utilizan. `parse_formal_to_typed(source)` continúa siendo una API de
texto sin contexto de ruta y rechaza imports sin resolver. `Module.origins`
conserva módulo y archivo de cada declaración. Las referencias `Verb.program`
siguen vinculadas al nombre único de su regla; `explain` presenta el origen de
verbs, reglas y defaults junto a las justificaciones existentes. Esta metadata
no participa en la identidad lógica de eventos ni modifica los registros Horn.

La separación experimental vive en [examples/legal/modules](examples/legal/modules):

| Archivo | Responsabilidad |
| --- | --- |
| [legal_monotonic.res](examples/legal/modules/legal_monotonic.res) | Todas las declaraciones y reglas reutilizables no específicas de compraventa del expediente original |
| [sales.res](examples/legal/modules/sales.res) | Vocabulario de compraventa, lambdas normalSale/retentionSale y reglas separadas para paquetes seleccionados y aportados |
| [sale_defaults.res](examples/legal/modules/sale_defaults.res) | Importa Sales y selecciona sus lambdas, sin redeclarar reglas |
| [defaults_scenario.res](examples/legal/modules/defaults_scenario.res) | Consumidor de SaleDefaults con los casos de reserva y asociación programática |
| [sales_monotonic_baseline.res](examples/legal/modules/sales_monotonic_baseline.res) | Versión congelada de Sales anterior a defaults, con las tres reglas originales sobre reciprocity |
| [scenario.res](examples/legal/modules/scenario.res) | Datos concretos y los dieciséis experiments del original; importa el baseline monotónico, sin defaults |
| [sells_scenario.res](examples/legal/modules/sells_scenario.res) | Consumidor de Sales: instancia positiva, ausencia y evidencia negativa de sells, sin repetir su programa |

`reciprocity` conserva su categoría Seeded y `selectedReciprocity` la categoría
Derived, sin puente automático. normalSale y retentionSale producen el segundo
en la ejecución de defaults; el baseline monotónico no lo produce. La referencia de la extracción
completa es [case_monotonic_formal.res](examples/legal/case_monotonic_formal.res),
conservada sin cambios. La comparación estricta verifica cada declaración y los
dieciséis cierres: eventos, sustituciones, tiempos proposicionales y de transición,
provenance, incluidos los antecedentes ordenados y la multiplicidad de los
registros. Los valores concretos de `derivedAt` se registran como diagnóstico en
`target/legal-monotonic-extraction-comparison.txt`, pero no se exige su igualdad:
la especificación deja las etiquetas al reloj externo entre ejecuciones
independientes. Las demás discrepancias hacen fallar la comparación; los
orígenes de archivo se verifican como metadata de resolución.

El consumidor de defaults está en
[defaults_scenario.res](examples/legal/modules/defaults_scenario.res), con ocho
experiments. [sale_defaults.res](examples/legal/modules/sale_defaults.res) conserva
su política e importa [sales.res](examples/legal/modules/sales.res), fuente única
de sus lambdas. La dependencia experimental SalesDefaultsReference se eliminó.
Los cuatro casos de reserva se comparan con
[case_sale_defaults_formal.res](examples/legal/case_sale_defaults_formal.res),
y los ocho casos con [scenario_inline.res](examples/legal/modules/scenario_inline.res).
La comparación conserva resultados, tiempos, sustituciones y registros de
provenance completos, excluyendo sólo los valores concretos de derivedAt.
El informe queda en `target/legal-sale-defaults-comparison.txt`.
La lambda de asociación anterior se conserva fuera de la ontología en
[associated_program.res](examples/legal/modules/associated_program.res);
[beta_scenario.res](examples/legal/modules/beta_scenario.res) la especializa
mediante la β-reduction existente.

```sh
./residuum run examples/legal/modules/scenario.res
./residuum explain examples/legal/modules/scenario.res
./residuum explain examples/legal/modules/defaults_scenario.res
./residuum run examples/legal/modules/beta_scenario.res
```

El expediente original y la compraventa anterior se conservan como referencias.
El desarrollo de Legal usa las capas modulares; `case_monotonic_formal.res`
permanece como baseline de regresión. Sales declara `sells = program givingProgram`
y conserva la referencia a la lambda importada de AssociatedGivingProgram.
El consumidor sólo importa Sales y aporta datos. Su cierre y provenance se
comparan con el experimento inline anterior, sin ejecutar los Effects ni
generar Act automáticamente.

La cadena activa es LegalMonotonicCase → Sales → SaleDefaults → DefaultScenario.
El runner excluye normalSale y retentionSale del cierre obligatorio y sólo
incorpora las lambdas elegidas después de applicability → blocking → priority.
Importar SaleDefaults por sí solo carga declaraciones, sin evaluar defaults ni
ejecutar experiments, como muestra
[sale_defaults_import_only.res](examples/legal/modules/sale_defaults_import_only.res).
La regresión de los dieciséis escenarios usa el baseline monotónico: ejecutar
Sales sin declaraciones de defaults trataría las dos lambdas como reglas
ordinarias. No se modifica el motor para alterar ese comportamiento ni se
añade consistency.

La extracción no cambia la semántica del default calculus, del closure o de los
efectos. La igualdad de los dieciséis cierres y sus registros de derivación es el
criterio de cierre, excluyendo únicamente el valor concreto de `derivedAt`.
Los sellos originales no se modifican y sus diferencias siguen visibles en el
informe.

El [experimento de oposición de defaults](examples/legal/modules/default_opposition_scenario.res)
aporta nueve contextos como programa Formal. El harness calcula el cierre de
cada lambda declarada, por separado, sobre una copia del Known obligatorio y
con las reglas obligatorias; las demás lambdas controladas quedan excluidas.
También inspecciona lambdas no seleccionadas o no aplicables como hipótesis
contrafácticas, identificándolas como tales. No ejecuta State ni añade las
consecuencias hipotéticas al expediente real.

El informe `target/default-opposition-experiment.txt` distingue oposición ya
presente, una consecuencia nueva opuesta al Known actual y dos consecuencias
nuevas opuestas entre sí. Compara verbo, argumentos, PropositionTime y polaridad,
conservando las justificaciones de ambos lados. Las diferencias de autoridad,
beneficiario o tiempo impiden formar el par exacto. Una oposición no produce
automáticamente inconsistent, blocked o defeated: consistency permanece
NotChecked y el evaluador actual no recibe estas observaciones.

El [contrato experimental para Program](docs/program-consistency-contract.md)
inspecciona Programs reales de Sales sobre una copia de Known. Reutiliza esos
nueve escenarios y añade tres en
[`program_consistency_scenario.res`](examples/legal/modules/program_consistency_scenario.res):
Known vacío, una conclusión negativa nueva contra un hecho positivo conocido,
y los tres ámbitos de oposición simultáneos. Son 24 cierres hipotéticos,
conservando las justificaciones anteriores y nuevas incluso cuando el evento
ya pertenece a Known. El informe está en
`target/program-consistency-observations.txt`; las observaciones no alimentan el
selector y ConsistencyStatus permanece NotChecked.

La [fixture de selección conjunta](examples/legal/modules/default_joint_opposition_scenario.res)
declara dos defaults supernormales independientes que, sobre la misma evidencia,
producen obligation y ~obligation. Sus cierres individuales no introducen
oposición; el cierre conjunto contiene un par nuevo. El harness conserva ambas
justificaciones y vincula cada una con su default contribuyente. No elimina
ninguno, ni marca blocked o defeated, y repite la comparación con el orden de
declaraciones invertido. El informe queda en
`target/default-joint-opposition-experiment.txt`. El Known obligatorio permanece
intacto. Este resultado muestra que una evaluación de consistency necesita
identificar el conjunto de lambdas examinado y las contribuciones de cada lado;
los resultados individuales no certifican la compatibilidad conjunta.

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
en un solo programa las declaraciones, reglas y dieciséis escenarios de actos,
cumplimiento, violación, patrimonio, titularidad, capacidades, legislación y
anulación. Autoridad y vigencia se comparan mediante datos explícitos; las marcas
de anulación y extinción conservan los hechos originales. Sus conclusiones son
`Derived`, incluidos los `Effect`s de sus reglas puente. Para observar
exclusivamente el cierre forward:

```sh
python3 scripts/test_safe.py --fixture examples/legal/case_monotonic_formal.res --forward
```

El escenario `actAndEvidentialNot` usa `Act(...)` y `~Act(...)` con el
`EvidentialNot` existente. `inconsistent` y `disputedPerformance` son verbos
`Derived` ordinarios, derivados por reglas del programa. Ninguno establece
prioridad, bloquea otra regla ni retracta evidencia.

La [fixture anterior con CounterAct](examples/legal/case_monotonic_counter_baseline.res)
queda como referencia de la comparación realizada con `run` y `explain`:

| Resultado del caso conjunto | Antes: Act + CounterAct | Ahora: Act + ~Act |
| --- | --- | --- |
| compliance y violation | Ambos | Ambos |
| activo y pasivo patrimoniales | Ambos | Ambos |
| inconsistent | No | Sí |
| disputedPerformance | No | Sí |
| hechos en el cierre | 11 | 13 |

En la comparación previa al incremento de reciprocidad, tras mapear
`CounterAct(...)` a `~Act(...)`, los cierres de los ocho escenarios conservaban
exactamente sus hechos anteriores. Sólo el caso conjunto incorporaba las
dos marcas nuevas. En `negativeActOnly` no hay cumplimiento ni inconsistencia;
en `unperformed`, la ausencia de Act no produce ~Act ni violación. No aparecen
conclusiones ajenas a las reglas del caso.

La provenance de cumplimiento y violación conserva reglas, sustituciones,
autoridad, vigencia y tiempo proposicional. El testigo de violación pasa de
CounterAct a la evidencia negativa explícita. Para observar esa provenance en la
referencia anterior se cambió únicamente la query del escenario a `violation`;
sus seeds y reglas se conservaron. `derivedAt` es el sello de cada derivación y
no establece precedencia entre conclusiones.

Como autoridad y vigencia son argumentos de Act, la regla `conflictingActEvidence`
exige que también coincidan. Evidencias con otros argumentos o distinto índice
temporal son proposiciones distintas; el programa no las identifica ni compara
su autoridad para decidir entre cumplimiento y violación.

El contrato del expediente separa evidencia histórica (`Act` / `~Act`),
conclusiones jurídicas en `Known` y ejecución de `Effect`s sobre `State`.
`give` desplaza patrimonio; `do` y `notDo` constituyen relaciones jurídicas de
hacer y abstenerse. No significan que ocurrió una conducta histórica. Ejecutar
un efecto no incorpora automáticamente evidencia a `Known`; `~Act` sólo recibe
consecuencias jurídicas mediante reglas explícitas del dominio.

`Relation.mode : PerformanceMode` es obligatorio. `do` inserta `Doing` y `notDo`
inserta `Abstaining` tanto en obligaciones del deudor como en derechos del
acreedor. El runtime existente almacena estas constantes como campos ordinarios
de `Value::Record`: ambas relaciones coexisten en `Set Relation` y cada una
deduplica sus inserciones idénticas. El elaborador resuelve constantes declaradas
antes del efecto, también dentro de records, manteniendo precedencia de sus
parámetros y los controles de tipo existentes. No cambia el parser ni el runtime.

Las reglas `obligationCreatesRelation`, `prohibitionCreatesAbstention` e
`historicalTransferEffect` están escritas en el expediente y derivan respectivamente
`do`, `notDo` y `give` en `after(tau)`. Las normas y la transferencia acreditada
permanecen en Known; autoridad y vigencia se conservan en sus justificaciones.
El record patrimonial almacena las partes, cosa, conducta y modalidad, sin
añadir metadata de autoridad o vigencia que su esquema no declara.

La verificación compara las conclusiones y justificaciones no-Effect con el
mismo programa sin las reglas puente: se conservan; los sellos `derivedAt` pueden
cambiar al añadirse derivaciones. Ejecuta además cada efecto integrado por
separado desde `State(tau)`, preservando el snapshot de entrada y el cierre.
No instala un planificador para efectos simultáneos.

La ejecución usa bindings explícitos del caller de símbolos como `juan`, `pedro`
y `car` a los `ObjectId` del Store inicializado, mediante la representación de
referencias que acepta la API runtime actual. El test conserva intacto el evento
simbólico en Known y comprueba que ejecutarlo sin esos bindings falla: no se ha
añadido resolución automática de constantes de entidades a objetos runtime.
`Doing`, `Abstaining` y las conductas permanecen como constantes simbólicas.

La reciprocidad llana y condicionada vive en el mismo archivo `.res`.
`sale(...)` conserva las mismas partes, cosa, precio, identificadores normativos,
autoridad y vigencia. `reciprocity(saleCase, Plain)` y
`reciprocity(saleCase, Retention)` son datos positivos del expediente, no una
evaluación de defaults. `price : Thing` es el objeto simbólico del pago, sin
aritmética de cantidades. `paymentFor` vincula el documento de pago con la venta;
`Act` debe acreditar pago del comprador al vendedor sobre ese precio.

| Escenario | Permiso de transmitir | Obligación de transmitir |
| --- | --- | --- |
| Plain sin pago | Sí | Sí |
| Retention sin evidencia de pago | Sí | No |
| Retention con pago correcto | Sí | Sí |
| Retention con sólo ~Act(pay) | Sí | No |
| Retention con Act(pay) y ~Act(pay) | Sí | Sí; también inconsistent |
| Retention con pago a otro destinatario | Sí | No |
| Ambos paquetes y pago | Sí | Sí; conserva ambas justificaciones |
| Venta y pago sin paquete | No | No |

`permissionFiled` conserva el ingreso externo y `permission` es ahora Derived,
igual que `obligationFiled` / `obligation`, para admitir permisos producidos por
reglas. Los escenarios previos de autoridad conservan sus documentos y derivan
los permisos correspondientes. La obligación condicionada conserva la autoridad
y vigencia de la venta; su provenance registra separadamente las del pago.
Todas estas proposiciones se observan en `tau`; la fixture no define el nacimiento
de una norma entre snapshots temporales distintos.

Los paquetes normativos pueden derivar `do` mediante la regla puente, pero no
incluyen una regla de transmisión ipso iure. El pago no deriva `Act(transfer)`,
`give`, titularidad o cumplimiento del vendedor. Aportar ambos paquetes acumula
consecuencias y justificaciones; este archivo no selecciona paquetes por defaults.
Para observar resultados y pruebas:

```sh
./residuum run examples/legal/case_monotonic_formal.res
./residuum explain examples/legal/case_monotonic_formal.res
```

La [compraventa con defaults](examples/legal/case_sale_defaults_formal.res) es un
programa separado que conserva los roles, normas y efectos del caso base. Declara
la política en Formal: `NormalSale` usa `normalSale`; `RetentionSale` es una
excepción condicionada por `retentionAgreed`. El mismo hecho bloquea explícitamente
`NormalSale` mediante `blocked when`.

| Escenario | Lambda seleccionada | Obligación de transmitir |
| --- | --- | --- |
| Sin reserva | normalSale | Sí |
| Reserva sin pago | retentionSale | No |
| Reserva con pago correcto | retentionSale | Sí |
| Observación del bloqueo normal | retentionSale; normalSale bloqueada | No |

Todos conservan el permiso de transmitir. La ausencia de obligación no produce
`~obligation`; el pago debe coincidir con comprador, precio, vendedor y tiempo.
Las consecuencias y sus pruebas se observan con:

```sh
./residuum run examples/legal/case_sale_defaults_formal.res
./residuum explain examples/legal/case_sale_defaults_formal.res
```

`blocked` registra un bloqueador explícito; `defeated` registra desplazamiento
por prioridad. El bloqueo excluye la lambda completa, con alcance `RuleLambda`.
La selección usa el cierre obligatorio como contexto y no se reevalúa con las
consecuencias de los defaults. `consistency: NotChecked` sigue siendo una
limitación: no implementa el cálculo completo de Reiter. `query` no admite estos
archivos con defaults. Esta fixture anterior conserva su ontología inline como
referencia; la versión modular reutiliza las capas de `examples/legal/modules`.
El expediente monotónico queda como referencia independiente.

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
en el harness externo de ese experimento anterior; es distinto de la resolución
de imports de archivos disponible ahora.

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
se filtra contra los abducibles y se certifica mediante replay finito de las
cláusulas Horn explícitas. Ese certificado no comprueba la activación dinámica
de `Verb.program`. `failure` distingue `NotAbducible` de
`NotForwardSupported`; ambas condiciones dejan `required: None`. Conserva las ramas
sin preferencia por `≤_L`, y no modifica el conocimiento ni ejecuta efectos.
El [contrato de la consulta sobre programas](docs/program-residual-query.md)
precisa ese alcance y sus tests de suficiencia y cobertura finita.

La extensión experimental `query_active_program_residual(&module, &evidence,
&goal, limits, &mut clock)` devuelve `ActiveProgramResidualQuery`. Incluye las
guardas de activación de `Verb.program` y certifica cada hipótesis mediante el
`close_program` original sobre evidencia hipotética, conservando provenance de
Rule/Program/Module. No modifica Known ni ejecuta State. La API Horn anterior
conserva su nombre, firma y comportamiento; la CLI aún utiliza sus rutas previas.
El [contrato de la API activa](docs/active-residual-api.md) resume sus garantías,
precondiciones y regresiones independientes del harness jurídico.

| Resultado de la consulta activa | Significado |
|---|---|
| Rama con `candidate.required = Unit` | Explicación certificada sin hipótesis pendientes |
| Rama con eventos o conjunciones pendientes | Hipótesis suficientes si se aportan, fuera de Known |
| `required = None`, cero explicaciones y `failure` | No se certificó una explicación en el fragmento buscado |
| `Err` | Búsqueda/validación no completada; no es una respuesta negativa ni `I` |

El resumen `Join` puede conservar `I` junto con alternativas pendientes; deben
examinarse las ramas sin descartar sus justificaciones. La API activa requiere
objetivos ground y parámetros determinados por el head, no admite defaults ni
amplía existenciales. `SearchLimits` acota búsqueda, no el cierre hipotético: su
terminación es una precondición del llamador. El
[informe de preparación de integración](docs/residual-active-query-integration-review.md)
separa estos contratos y la verificación individual disponible.

El [oráculo jurídico de la consulta activa](docs/legal-ontology-active-residual-oracle.md)
contrasta suficiencia y cobertura con las fronteras mínimas del cierre real:
16 casos controlados y 146 objetivos de la ontología, conservando como límites
explícitos los parámetros no determinados y los requisitos fuera del catálogo.

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

### Comparación experimental de defaults con Catala

Las fixtures de [default_comparison](examples/legal/modules/default_comparison/)
reutilizan `LegalMonotonicCase → Sales → SaleDefaults (variante) → Scenario`.
`priority.res` y `unordered.res` referencian las lambdas reales `normalSale` y
`retentionSale`, sin copiar la ontología ni modificar el selector. Los datos
concretos viven en `data.res`. No se ejecuta Catala: la comparación externa usa
su documentación oficial; los resultados de Residuum son observaciones de estas
fixtures, no un teorema de equivalencia entre ambos lenguajes.

| Caso | Resultado actual de Residuum | Comparación con Catala para definiciones de una misma variable |
|---|---|---|
| Sólo `NormalSale` aplicable | Selecciona `NormalSale`; deriva `Plain`, permiso y obligación | Usa la única definición aplicable |
| Ambos aplicables, prioridad explícita | Selecciona `RetentionSale`; `NormalSale` queda `defeated`, no `blocked`; sin pago no deriva la obligación | Usa la definición con prioridad sobre las otras aplicables |
| Ambos aplicables sin prioridad | Conserva ambos; deriva `Plain` y `Retention`, sin error ni ganador arbitrario | Señala conflicto si no existe una definición que prevalezca |
| Ambos producen `P/~P` | Conserva ambas proposiciones y sus justificaciones; selección sin cambios | El conflicto anterior se refiere a asignaciones de la misma variable, no a EvidentialNot |
| `Known` vacío | `NormalSale` sigue aplicable y seleccionado; no hay venta que haga disparar su lambda | No equivale al caso de ninguna definición aplicable |

La comparación de selección se apoya en el
[algoritmo documentado de excepciones de Catala](https://book.catala-lang.org/en/2-2-conditionals-exceptions.html).
Catala agrupa definiciones por variable y permite indicar la base de una
excepción mediante una etiqueta.
[Referencia de definiciones y excepciones](https://book.catala-lang.org/en/5-4-definitions-exceptions.html).
Residuum mantiene deliberadamente la selección de lambdas completas y la
acumulación de sus consecuencias; este experimento no copia la política de error
ni introduce una elección basada en oposición evidencial.

**Frontera encontrada:** el parser y el elaborador sólo admiten `when` en
`exception`, no en `supernormal`. Por eso la variante sin prioridad utiliza dos
supernormales incondicionales. No equivale a quitar sólo una arista conservando
la condición de `RetentionSale`. Para comparar ambos con idéntica aplicabilidad
falta poder expresar un default condicionado sin relación de excepción; aquí no
se implementa esa capacidad.

En un grafo no vacío de defaults expresable actualmente, las cadenas de excepción
terminan en supernormales incondicionales. Por tanto, **ninguno aplicable** no es
un estado representable por ese programa. **Ninguno seleccionado tras bloqueo**
es otro estado; tampoco debe confundirse con una lambda seleccionada que no
encuentra antecedentes. El módulo canónico `sale_defaults.res` conserva su
`blocked when`: con reserva, `NormalSale` queda bloqueado antes de evaluar la
prioridad, no desplazado por ella.

`DefaultExtension` ya conserva aplicabilidad con evidencia/sustitución, bloqueos,
defaults desplazados y todos los seleccionados. Le falta un contrato que indique
qué lambdas compiten por una misma decisión, y un estado que registre esa
ambigüedad sin resolverla. Tener varias lambdas no implica conflicto universal:
podrían ser independientes. En este ejemplo la decisión observada es el paquete
de `selectedReciprocity(saleCase, ...)`; ni `Plain/Retention` son `P/~P`, ni la
oposición de `comparisonP` convierte automáticamente un default en bloqueado o
desplazado. `consistency` permanece `NotChecked`.

Los dos tests seleccionados comprueban ambas provenance, los tiempos y
sustituciones, dos justificaciones para el permiso compartido, oposición sólo en
el caso que la declara, y conservación de los resultados al invertir el orden de
declaraciones. Sólo permiten diferencias del sello externo `derivedAt`.
Las comprobaciones individuales y conjuntas usan copias de `Known`; no ejecutan
`StateTransform` ni modifican el contexto original. El primer test escribe
`target/catala-default-comparison.txt` con la tabla y las justificaciones.

```sh
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::default_comparison::modular_defaults_distinguish_applicability_priority_and_unordered_candidates \
  --test cli::tests::default_comparison::unordered_sale_defaults_preserve_opposed_proofs_without_choosing_a_winner
```

### Contrato experimental de competencia entre defaults

El experimento siguiente separa dos relaciones:

```text
competes(D1, D2)          // simétrica; declaración explícita del experimento
priority(D1, D2)          // direccional; exception ... to ... ya existente
```

Estas líneas son notación del contrato, **no sintaxis nueva de Residuum**.
`competes` se declara únicamente en el harness mediante dos `DefaultRef`
resueltos contra el módulo. Los defaults no son términos de proposiciones y
actualmente no podemos escribir esa relación directamente en una fixture.
No se infiere competencia por compartir antecedentes, por producir diferentes
paquetes ni por encontrar `P/~P`. Tampoco se supone transitividad.

Con `D1 = NormalSale` y `D2 = RetentionSale`, se observan:

| Candidatos sin bloqueo | Prioridad | Contrato experimental | Selector actual |
|---|---|---|---|
| Sólo D1 | Irrelevante | `Selected(D1)` | Selecciona D1 |
| D1 y D2 compiten | D1 > D2 | `Selected(D1)` | Selecciona D1 y desplaza D2 |
| D1 y D2 compiten | Ninguna | `Conflict(D1, D2)` | Conserva ambos |
| D1 y D2 sin competencia declarada | Ninguna | `Selected(D1, D2)` | Conserva ambos |

El caso de prioridad usa
[competition_priority_scenario.res](examples/legal/modules/default_comparison/competition_priority_scenario.res),
que importa `normal_priority.res`, las lambdas reales de `Sales` y la ontología
base. Los otros casos reutilizan `priority_scenario.res` y
`unordered_scenario.res`. La asimetría existente de condiciones se mantiene:
este experimento no afirma que las tres fuentes difieran sólo en una arista.

`Conflict` es un diagnóstico del harness, no una excepción de ejecución ni un
`ConsistencyStatus` del motor. Conserva ambos defaults y todos sus matches de
aplicabilidad; el reporte conserva además las derivaciones con origen, tiempos,
antecedentes y sustituciones. No elige un ganador, no añade marcas `blocked` o
`defeated`, no filtra el cierre real y no modifica `Known`.

La prueba cubre la simetría de competencia, la dirección de prioridad y la
invariancia al invertir las declaraciones. También comprueba que `P/~P` sin
competencia declarada no produce ese diagnóstico. El alcance es un par explícito
de lambdas completas; no un grafo general, competencia por instancia ground ni
un contrato definitivo del lenguaje. Falta decidir cómo representar en el AST
la relación declarada entre defaults; aquí no se implementa ni se elige sintaxis.

```sh
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::competition_contract::declared_competition_reports_conflict_without_changing_selection
```

El test escribe `target/default-competition-contract.txt`. El código experimental
está en `src/bin/residuum/cli/competition_contract_tests.rs`; no cambia el selector.

### Competencia explícita frente a inferencia de slots

El segundo experimento compara el contrato anterior con una heurística estructural:
las consecuencias directas de dos defaults tienen el mismo predicado, polaridad,
tiempo y argumentos, excepto una posición. Esa posición se toma como **valor
candidato**; las demás como **clave candidata**. La atribución a cada default usa
su `RuleRef` y las derivaciones de su cierre hipotético individual, no una tabla
especial de nombres de verbos. Se observan los candidatos sin bloqueo antes de
prioridad, de modo que una prioridad puede resolver un slot compartido.

| Caso finito | Slot candidato por forma | Competencia explícita en el harness | Resultado del contrato |
|---|---|---|---|
| NormalSale solo | No hay par | Sí, pero el otro no es aplicable | Selecciona NormalSale |
| Ambos, NormalSale con prioridad | `selectedReciprocity(sale, package)` | Sí | Selecciona NormalSale |
| Ambos sin prioridad | Mismo slot candidato | Sí | Conflict |
| Derechos sobre car y bike | `entitledTo(person, thing)` | No | Conserva ambos |
| Dos defaults independientes producen P/~P | Ninguno por esta heurística; polaridades distintas | No | Conserva ambos, con oposición evidencial |

[cumulative_rights_scenario.res](examples/legal/modules/default_comparison/cumulative_rights_scenario.res)
es el control acumulativo. Ambos derechos están expresados en Residuum; convertir
su coincidencia estructural en `competes` produce un falso positivo. El mismo AST
y el mismo cierre dan resultados diferentes si el harness supone exclusividad.
Esto refuta **esa heurística como criterio suficiente**, no toda posible técnica
de inferencia de competencia.

En Sales, Plain ya produce **obligación y permiso**; Retention produce permiso y
condiciona la obligación al pago. Obligación frente a permiso no representa la
incompatibilidad buscada. La decisión candidata es el paquete de reciprocidad.
Pero `selectedReciprocity` es hoy una relación ordinaria: su firma no garantiza un
único paquete por venta/tiempo. El motor admite ambos, igual que admite derechos
sobre varias cosas.

**Resultado provisional:** podemos inferir posibles claves/valores; no hemos
inferido exclusividad. Un contrato de slot exclusivo permitiría obtener las
parejas competidoras sin declarar cada `competes(D1,D2)`, pero ese contrato aún no
está en el AST. No se añade sintaxis ni se elige entre declarar pares, declarar
slots o expresar incompatibilidad mediante reglas del dominio. `Conflict` sigue
siendo un diagnóstico experimental separado de la oposición P/~P.

La prueba conserva ambos caminos de provenance, compara también el orden inverso
de declaraciones y mantiene Known y `consistency: NotChecked` intactos. Su alcance
son cinco casos finitos, consecuencias directas ground y una posición variable;
no es una inferencia completa de slots para cualquier programa.

```sh
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::competition_contract::inferred_slot_shape_does_not_establish_exclusive_competition
```

El reporte es `target/default-slot-inference-comparison.txt`; el harness sigue en
`src/bin/residuum/cli/competition_contract_tests.rs`.

### Aplicación parcial y cuerpo compuesto propuesto

[partial_normal_sale_scenario.res](examples/legal/modules/default_comparison/partial_normal_sale_scenario.res)
comprueba la compatibilidad de β-reduction antes de implementar un cuerpo
compuesto. Declara dos relaciones separadas con los mismos parámetros
`(t, seller, object, buyer)`. Ambas se especializan con `(tau, juan)` y conservan
los parámetros residuales `(object : Thing, buyer : Person)`:

```text
saleBasis(juan, object, buyer) @ tau
    <= normalChosen(juan, object, buyer) @ after(tau)

normalChosen(juan, object, buyer) @ after(tau)
    <= obligation(duty, juan, transfer, object, buyer, private, effectiveDate)
       @ after(tau)
```

El test comprueba las dos relaciones completas, sus tipos residuales, la igualdad
del evento intermedio y que aplicar después `(car, pedro)` produce el mismo cuerpo
que aplicar todos los argumentos directamente. Las definiciones originales
permanecen intactas. La aplicación actual admite constantes tipadas como prefijo
posicional, no cualquier subconjunto de parámetros ni argumentos lambda generales.

Los parámetros restantes siguen ligados por la lambda residual; sus valores están
pendientes. Son variables en las cláusulas, pero no variables globalmente libres.

**Límite:** el AST actual no tiene cuerpo compuesto. Esta prueba verifica β en
cada componente; no demuestra que exista ya una lambda contenedora. Para reglas
internas sin binders propios, extender la sustitución mediante un recorrido de
todos los componentes conserva ese comportamiento. Las reglas internas con
binders propios necesitan un recorrido que respete ámbitos, aún no implementado.
No se modifica la representación de Rule, el elaborador ni la selección de defaults.

```sh
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::partial_package::partial_beta_is_compatible_with_each_relation_of_a_shared_parameter_body
```

El test escribe `target/partial-normal-sale-compatibility.txt`.

### Rule como denotación de un programa monotónico

Este experimento conserva `RuleBody`, `DerivationRecord`, `close`, defaults y el
elaborador. Añade sólo fixtures y verificaciones. La referencia externa es
[Datafun](https://www.rntz.net/files/datafun.pdf): distingue funciones monotónicas
y dominios ordenados, y restringe sus fixpoints para garantizar terminación.
Aquí usamos esa distinción conceptual; no importamos su sistema de tipos ni sus
restricciones de finitud.

**Lo representado por el AST.** Después de β-reduction, una `Rule` sigue siendo
un nombre, los parámetros restantes y **una** `HornClause` o `ResidualClause`.
La aplicación consume un prefijo de constantes tipadas, sustituye términos y
tiempos y conserva los binders residuales. No produce un valor función general,
un conjunto de cláusulas ni una captura de las reglas importadas.

**Definición semántica sobre ese AST.** Para una regla fija `r`, sea:

\[
T_r(F)=\{\operatorname{head}(r\theta)\mid
         \theta\text{ es un match compatible de su antecedente en }F\}.
\]

En el caso residual, este operador usa la vista Horn existente por
`unresiduate`; no afirma que el forward compute todo entailment algebraico.
`instantiate_from_known` calcula los resultados y sus justificaciones;
\(F\cup T_r(F)\) es la transformación inflacionaria correspondiente.

**Argumento general de monotonicidad para el fragmento actual:** si
\(F\subseteq G\), cada testigo de evento de `F` sigue en `G`. `I` conserva su
testigo neutral y `&` conserva los matches con sustitución compartida. Un
testigo negativo requiere el mismo evento `EvidentialNot`, nunca una ausencia.
Por tanto, cada resultado de \(T_r(F)\) sigue en \(T_r(G)\). Las comprobaciones
de constantes, tiempos y parámetros pendientes dependen del match y de la regla
fija, no de que falten otros hechos. Esto demuestra monotonicidad en **Known**;
no demuestra monotonicidad en un orden de verdad ni establece una ley nueva
del álgebra residuada.

Para un conjunto fijo `R` de reglas existentes, definimos:

\[
P_R(F)=\operatorname{Known}(Close_R(F)).
\]

Las iteraciones \(F_{n+1}=F_n\cup\bigcup_{r\in R}T_r(F_n)\) preservan inclusión
por inducción; su unión también. De ahí la monotonicidad de esta denotación.
Que la ejecución de `close` alcance esa unión en tiempo finito requiere además
terminación; los constructores temporales no ofrecen esa garantía general.

La firma conceptual más precisa para una regla parametrizada es:

\[
\llbracket r\rrbracket:X\longrightarrow(K\longrightarrow^+K).
\]

Los argumentos de entidad y tiempo no tienen hoy un orden semántico general que
permita afirmar \(X\to^+K\). Darles un orden discreto sería una interpretación
posible, no una feature implementada. `Known` vacío sigue siendo
\(\varnothing\in K\), distinto de la unidad lógica `I`.

**NormalSale y RetentionSale.** Sea `E` el entorno fijo de reglas obligatorias
de Sales y Legal. Cada entrada seleccionada `r` denota contextualmente:

\[
P_{r,E}(F)=\operatorname{Known}(Close_{E\cup\{r\}}(F)).
\]

El selector actual escoge `normalSale` o `retentionSale`, no cada consecuencia.
La entrada produce `selectedReciprocity`; el entorno produce los permisos,
obligaciones y efectos correspondientes. `reciprocity` permanece un canal Seeded
independiente, sin puente automático. El nombre `NormalSale` identifica el
default; `normalSale` identifica su lambda de entrada.

| Caso existente | Entrada | Permiso | Obligación | Effect `do` |
|---|---|---|---|---|
| Sin reserva | normalSale | Sí | Sí | Sí |
| Reserva sin pago | retentionSale | Sí | No | No |
| Reserva con pago | retentionSale | Sí | Sí | Sí |
| NormalSale bloqueado | retentionSale | Sí | No | No |

**MODEL RESULT:** los cuatro escenarios coinciden con la fixture inline en
eventos, tiempos proposicionales y de transición, sustituciones y justificaciones.
Sólo se omiten los sellos externos `derivedAt`. También coinciden con la ejecución
en dos fases, entrada seguida de `E`, conservando los proofs de la primera fase.
En estas entradas eso funciona porque `E` no deriva nuevos eventos `sale` que
puedan volver a habilitar la entrada. Las definiciones de `Verb.program` se
excluyen de la lista de reglas ordinarias; estos cuatro casos no aportan `sells`
y no activan su programa. No generalizamos esta prueba a la activación asociada.

**MODEL RESULT:** para cinco eventos de entrada definidos en las fixtures
actuales, incluyendo `Act` y `~Act`, se comprobaron 64 cierres y 486 pares
comparables. Cada programa fijo conserva eventos y justificaciones al añadir
evidencia. En cambio, añadir `retentionAgreed` al caso sin reserva cambia la
selección y elimina su obligación normal del nuevo resultado. La función que
incluye selección de defaults no es ese mismo programa monotónico fijo.

**Composición y realimentación.** Componer funciones monotónicas es monotónico:
si \(F\subseteq G\), entonces \(P(F)\subseteq P(G)\) y
\(Q(P(F))\subseteq Q(P(G))\). Pero no necesariamente:

\[
Q(P(F))=Close_{R_P\cup R_Q}(F).
\]

[rule_composition_feedback.res](examples/experiments/rule_composition_feedback.res)
da un contraejemplo acíclico: `P` produce `intermediate`, `Q` produce `returned`,
y una relación de `P` necesita `returned` para producir `completed`. Una regla
de observación en `P` permite consultar `completionObserved @ tau` con la sintaxis
actual. Una sola pasada `Q(P(F))` omite `completed` y esa observación. El cierre
conjunto sí los obtiene. En esta fixture una segunda pasada de `P` basta; no es
un límite general de iteraciones.
Podemos pasar el `Known` resultante a `close` con la API existente, pero esa API
no transporta automáticamente los `DerivationRecord` de la ejecución anterior.
El harness los conserva y deduplica por justificación, sin alterar el motor.

**Sintaxis temporal y composición interna son cuestiones distintas.** La
producción fuente actual admite `@ nombre` y `@ after nombre`. No admite escribir
directamente `@ after(after t)` ni `@ after after t`. La fixture usa únicamente
las formas admitidas; no se amplía el parser.

En cambio, `TimeExpr::After(Box<TimeExpr>)` es recursivo. Cuando una regla con
antecedente `@ t` recibe un evento en `after(tau)`, el matcher puede ligar
`t = after(tau)`; su consecuente `@ after t` produce entonces
`after(after(tau))`. La prueba inspecciona estructuralmente los tiempos y las
sustituciones de provenance:

| Evento | Tiempo interno esperado |
|---|---|
| start | tau |
| intermediate | after(tau) |
| returned | after(after(tau)) |
| completed | after(after(after(tau))) |
| completionObserved | tau, por la regla explícita de observación |

La observación no cambia ni colapsa el tiempo de `completed`. El test también
comprueba que los tiempos de una, dos y tres aplicaciones de `After` son distintos.
No interpreta `after` como sucesor discreto ni asigna una duración al constructor.

β-reduction fija el parámetro temporal mediante `pEntry(tau)` y sustituye ambos
lados de su cláusula: `@ tau` y `@ after tau`. Las aplicaciones `qStep(item)` y
`pFeedback(item)` fijan sólo el objeto; conservan `t` como parámetro residual.
Durante la composición, esos parámetros reciben las expresiones anidadas del
evento anterior. El programa especializado y el genérico producen los mismos
eventos, tiempos y justificaciones bajo la correspondencia β explícita de
nombres/binders, ignorando sólo el sello externo `derivedAt`.

Este **MODEL RESULT** demuestra anidamiento hasta profundidad tres en la fixture.
La representación recursiva y la sustitución conservan estructura, pero aquí no
se promete terminación ni completitud para cualquier cadena temporal. La
restricción de la sintaxis fuente no debe presentarse como un límite de profundidad
de la semántica interna.

**Aplicación parcial y su frontera.**
[program_specialization.res](examples/legal/modules/default_comparison/program_specialization.res)
contiene `normalSale(saleCase, juan)` y `retentionSale(saleCase, juan)` usando sólo
la aplicación existente. Quedan ocho parámetros ligados pendientes. En los
cuatro casos anteriores, especializar la entrada y mantener `E` conserva el
comportamiento. La provenance registra el alias y sólo los matches de los
parámetros restantes; el test relaciona esa prueba con la original usando la
sustitución β explícita, sin modificar la provenance de producción.

Esto **no** equivale a sustituir esos argumentos en todas las reglas de `E`.
La misma fixture aporta dos ventas con el mismo identificador y vendedores
distintos. La entrada especializada en Juan emite
`selectedReciprocity(saleCase, Plain)`; el entorno genérico puede aplicarlo a
ambas ventas. Especializar además las reglas de permiso/obligación excluye la
consecuencia del segundo vendedor. No asumimos una restricción de unicidad de
`saleId` que el lenguaje no declara. El caso identifica exactamente la diferencia
entre β de una entrada y β de un cuerpo completo con parámetros compartidos.

**Provenance.** El `DerivationRecord` actual identifica la regla productora,
testigos y sustitución. El contexto de evaluación aporta el default y la entrada
seleccionados; el grafo de testigos permite buscar soporte desde esa entrada.
Esto basta para explicar el experimento sin añadir campos. No da a cada regla
compartida un propietario único: `suppliedPlainObligation` también produce una
obligación desde el canal Seeded independiente, sin ningún ancestro `normalSale`.
Una misma consecuencia puede tener varias justificaciones. El módulo de origen
de una regla tampoco es automáticamente la identidad de su programa.

| Nivel | Capacidad |
|---|---|
| AST actual | Una cláusula bajo parámetros; referencias, imports, β parcial y provenance de regla |
| Definible semánticamente | Operador monotónico de reglas fijas, composición funcional y cierre conjunto; programa contextual entrada + entorno |
| No representado como valor del lenguaje | Una composición con entorno capturado, identidad propia y binders compartidos que pueda ser objetivo directo de un default |

La extensión mínima que faltaría para ese último objetivo es **poder referenciar
una composición/captura de reglas existentes como una unidad parametrizada**, y
aplicar su sustitución a los componentes que pertenecen a ese ámbito. Hoy un
default referencia sólo `RuleRef`; una Rule no registra cuáles reglas componen su
programa. Esa capacidad no obliga por sí misma a un segundo AST de cláusulas,
pero requiere una representación explícita de asociación, ámbito e identidad.
Este experimento no la diseña ni la implementa.

Los tests están en `src/bin/residuum/cli/program_denotation_tests.rs`. Los informes
son `target/sale-program-denotation.txt`, `target/sale-program-monotonicity.txt`,
`target/rule-program-composition-feedback.txt` y
`target/sale-program-specialization-scope.txt`. Distinguen los argumentos generales,
resultados finitos y fronteras no implementadas.

### Investigación de Program como unidad autónoma

El prototipo en `src/bin/residuum/cli/autonomous_program_tests.rs` está limitado
al harness. No añade `Program` a Formal, SurfaceAST o TypedAST ni modifica Rule,
`close`, defaults, el elaborador, State o `DerivationRecord`. La pertenencia al
programa y el enlace desde el default son declaraciones **del experimento**,
no capacidades que el lenguaje ya tenga. No incorpora `competes`.

La representación conceptual mínima resultante es:

```text
Program {
    identity,
    parameters,
    rules: [RuleUse],
    environment
}

RuleUse {
    reference: regla individual resuelta con su módulo de origen,
    sharedBindings: parámetro de la regla -> parámetro del programa,
    captures: constantes tipadas fijadas para este miembro
}

Environment {
    symbols: referencias/imports resueltos y valores capturados,
    activeRules: dependencias importadas que participan en el cierre
}
```

`identity` es necesaria para explicar el origen programático; no cambia la
identidad lógica de los eventos. `rules` reutiliza las Rule de una sola cláusula.
No contiene un segundo árbol de expresiones lógicas. Las proyecciones de binders
son necesarias porque una regla puede tener parámetros locales adicionales o
nombres distintos: la coincidencia de nombres no define por sí sola el ámbito.
El harness valida los enlaces declarados contra los tipos actuales y comprueba
que renombrar un parámetro del programa conserva el resultado al ajustar su enlace.

El entorno separa **resolver** una regla importada de **activarla**. Capturar un
Module como tabla de símbolos no habilita automáticamente todas sus reglas.
El prototipo guarda una instantánea resuelta; una implementación podría guardar
referencias inmutables a las declaraciones sin copiar toda esa tabla. La unidad
es cerrada respecto de sus definiciones, aunque recibe evidencia `Known` como
entrada. No depende de cuáles reglas estén activas en el módulo consumidor.

Para el caso de prueba se declaran explícitamente estas unidades:

| Programa experimental | Miembros de Sales | Captura | Dependencia ejecutable importada |
|---|---|---|---|
| NormalSale | normalSale, reciprocalTransferPermission, plainTransferObligation | package = Plain | obligationCreatesRelation |
| RetentionSale | retentionSale, reciprocalTransferPermission, paidRetentionObligation | package = Retention | obligationCreatesRelation |

Sus diez parámetros compartidos proceden de la firma de la entrada actual de
Sales. Los parámetros adicionales de `paidRetentionObligation` siguen siendo
locales a esa Rule. La dependencia `obligationCreatesRelation` pertenece a
LegalMonotonicCase y conserva sus propios binders: su `t` no es el `t` de Program.
La captura `Plain`/`Retention` da una interpretación explícita al miembro de
permiso genérico. Es equivalente en los cuatro escenarios examinados; no
afirmamos equivalencia para cualquier contexto con marcadores adicionales.

La ejecución usa un único cierre existente:

\[
\llbracket P\rrbracket(K)
=Known\bigl(Close(K,Rules(P)\cup ActiveRules(Env(P)))\bigr).
\]

Así la selección, el permiso y la obligación son producidos por reglas que se
habilitan entre sí en el mismo fixpoint. La obligación habilita la dependencia
importada, que deriva `do @ after(tau)`. No se ejecuta ese Effect sobre State.
Antes de seleccionar la unidad sólo está habilitado su entorno ejecutable;
en estos escenarios éste no produce las consecuencias internas del programa.

**Aplicación parcial y β compartida.** El modelo parcial fija
`saleId=saleCase, seller=juan` en todos los miembros y conserva ocho binders del
programa. Una segunda aplicación con los argumentos restantes coincide con la
aplicación completa directa. Las tres cláusulas completas se comparan contra
el oráculo source
[autonomous_program_beta_reference.res](examples/legal/modules/default_comparison/autonomous_program_beta_reference.res).
El modelo también excluye la obligación del segundo vendedor en el contraejemplo
de dos ventas con el mismo identificador. El entorno importado permanece idéntico.

**Corrección posterior de β temporal.** La investigación encontró que
`effective : PropositionTime` quedaba como `Term::Var("effective")` al aparecer
como argumento de un verbo después de aplicar la Rule y retirar su binder.
La corrección independiente en el elaborador sustituye ahora las constantes
tipadas tanto en posiciones `Term` como `TimeExpr`, incluidos los índices de
transición. Las aplicaciones reales completas de `normalSale`,
`reciprocalTransferPermission` y `plainTransferObligation` coinciden ahora con las
cláusulas ground del oráculo. El test exige esa igualdad, no sólo la registra.

[temporal_beta_arguments.res](examples/experiments/temporal_beta_arguments.res)
comprueba aplicaciones parciales/sucesivas/directas en Horn y residual,
incluido un mismo binder temporal en argumento e índice. Rechaza matches con
otro argumento temporal o tiempo proposicional y conserva el parámetro `t`
cuando aún no se ha aplicado. El test de composición sigue comprobando los
tiempos internos anidados. No cambia el parser ni añade términos temporales al
runtime; la corrección afecta sólo a la aplicación de constantes del elaborador.

**Provenance.** Cada ejecución del harness acompaña el `DerivedEvent` intacto con
un `ProgramProof(program, producerRule, origin, imported)`. Las reglas de Sales
y la regla de Legal conservan su origen y sus testigos. La comparación con Sales
reproduce la justificación en la Rule original: restaura la correspondencia de
los parámetros consumidos por β y la captura de `package`, sin alterar las
derivaciones de producción. Se permite variar sólo `derivedAt`. Estas etiquetas
son contexto de ejecución, no una regla de prevalencia o una propiedad jurídica.
Si varios programas compartiesen una derivación, su atribución podría tener
varios soportes; no hemos establecido un propietario único para ese caso.

**Selección del programa completo.** El harness usa sin cambios
`evaluate_defaults` y su resultado actual. Un registro experimental explícito
relaciona cada RuleRef seleccionada con la unidad capturada correspondiente y
habilita sus miembros conjuntamente. Coinciden selección, bloqueos y los cierres
de los cuatro escenarios de Sales. Esto comprueba la compatibilidad del enlace
con la política actual, pero **no** que un default pueda ya referenciar un Program
en Residuum: DefaultBody y DefaultExtension todavía transportan RuleRef/Rule.

| Obligación | Resultado de esta investigación |
|---|---|
| Rule sigue siendo una cláusula | Conservado |
| Programa con varias relaciones relacionadas | Tres miembros y una dependencia importada, sólo en el harness |
| β parcial sobre el programa entero | Verificada en el modelo contra las aplicaciones actuales parciales |
| β completa sobre todos los parámetros compartidos | Modelo verificado contra cláusulas ground; β real de Rule también coincide tras corregir `effective` |
| Activación mutua en un fixpoint | Verificada mediante `close` existente |
| Entorno importado y sus ámbitos | Capturado y preservado durante β |
| Programa + regla en provenance | Anotación de harness; DerivationRecord intacto |
| Default selecciona la unidad | Adaptador experimental; referencia nativa a Program pendiente |
| Equivalencia con Sales | Cuatro escenarios, eventos/tiempos/sustituciones/provenance bajo correspondencia explícita |

Para llevar esta representación al lenguaje faltan la declaración/referencia
tipada de Program, sus enlaces de ámbito/captura, la generalización del target de
default y el contexto de provenance al aplanar sus reglas. La política de
applicability/blocking/priority puede conservarse, y `close` puede seguir
recibiendo Rule individuales. El defecto posicional en la sustitución de
constantes temporales ya está corregido como paso independiente. En esa fase Program y
ProgramRef seguían pendientes; la implementación mínima posterior se documenta abajo.

**MODEL RESULT:** pasaron dos tests individuales con `test_safe.py` en el target
CLI; la compilación inicial de ese target tuvo pico RSS de 309,79 MiB y los tests
15,30/15,42 MiB. El primer intento en el target de tests de biblioteca fue detenido
por el runner al alcanzar el umbral preventivo de memoria, antes de ejecutar
tests y sin OOM. Se trasladó el harness al target CLI, sin subir los límites ni
repetir aquella compilación. No se ejecutó la suite completa.

Los informes son `target/autonomous-program-selection.txt` y
`target/autonomous-program-beta.txt`. El experimento define una representación
candidata y verifica casos concretos, no demuestra equivalencia general ni
declara implementado Program como construcción de Residuum.

La corrección posterior pasó tres tests individuales en el target CLI mediante
`test_safe.py`: regresión temporal, equivalencia de β del harness con Rule reales
y composición temporal. La compilación tuvo pico RSS de 259,79 MiB y los tests
15,45/15,37/15,49 MiB. No se compiló de nuevo el target de tests de biblioteca.

### Program mínimo: implementación Formal

La propuesta anterior con `shared`, `captures` y `environment` fue sustituida
por un único mapa de bindings y el ámbito del módulo de definición:

```text
program Chain(object : Thing, t : PropositionTime) {
    use first { local = object, when = t }
    use second { local = object, when = t }
}
program Partial = Chain(item)
program Ground = Partial(tau)
```

Rule sigue siendo una sola cláusula. Program conserva parámetros, referencias a
Rules, bindings y módulo de definición. β alcanza todos los usos, incluidos
tiempos y records anidados; los parámetros no aplicados y los locales mantienen
sus ámbitos. Las Rules compiladas participan juntas en un único `close`.
`Verb.program` puede resolver una Rule anterior o un Program compuesto. Una
instancia positiva conocida de un Seeded/Derived activa el paquete completo,
sin ejecutar efectos de State. `explain` conserva Program + Rule + Module.

El caso real está en
[`sales.res`](examples/legal/modules/sales.res): `normalSaleProgram` contiene
`normalSale`, `reciprocalTransferPermission` y `plainTransferObligation`.
`obligationCreatesRelation` permanece como regla general de LegalMonotonicCase;
los miembros y el entorno participan en un mismo cierre. El alias
[`normal_sale.res`](examples/legal/modules/programs/normal_sale.res), la aplicación parcial y
la comparación inline están en
[`normal_sale_scenario.res`](examples/legal/modules/programs/normal_sale_scenario.res).
La asociación activa y sus casos Seeded/Derived están en
[`associated_multi_rule_program.res`](examples/experiments/associated_multi_rule_program.res),
y el consumidor importado en
[`multi_rule_program_consumer.res`](examples/experiments/multi_rule_program_consumer.res).

Pasaron nueve tests individuales del target CLI con `test_safe.py`, incluyendo
las regresiones de tiempos, imports, asociación previa y defaults. Este resultado
verifica esos casos, no demuestra equivalencia general. La integración posterior
añade selección nativa de ProgramRef, descrita abajo. Residual no se ha extendido
a Program: el runner de consultas rechaza esos módulos en lugar de fingir esa
integración. Véase [el contrato y sus límites](docs/minimal-programs.md).

### Selección nativa de Program desde defaults

[`sale_defaults.res`](examples/legal/modules/sale_defaults.res) referencia
`normalSaleProgram` y `retentionSaleProgram`, definidos en Sales. El elaborador
resuelve la sintaxis existente `supernormal Nombre` / `exception Nombre to Base`
a RuleRef o ProgramRef según la declaración; no hay un adaptador externo en
la selección real.

Se conserva applicability → blocking → priority → selección de lambdas completas.
Sólo después se compilan todos los RuleUse seleccionados, aplicando sus bindings,
para un único cierre con el entorno obligatorio. Los miembros controlados no
participan individualmente antes de seleccionar su paquete. Una declaración
importada no ejecuta defaults ni experiments.

Los cuatro casos de [`defaults_scenario.res`](examples/legal/modules/defaults_scenario.res)
coinciden con el baseline de defaults en eventos, tiempos, antecedentes y
sustituciones completas. Provenance añade Program + Rule + Module; se compara
mediante el origen de cada miembro y sólo se excluye el valor concreto de
`derivedAt`. [`program_environment_scenario.res`](examples/legal/modules/program_environment_scenario.res)
comprueba obligaciones ajenas a Sales y una obligación producida por NormalSale:
la regla general aporta una sola justificación y no pertenece al paquete.
[`program_priority_scenario.res`](examples/legal/modules/program_priority_scenario.res)
aísla prioridad sin bloqueo. Consistency permanece `NotChecked`.

El [contrato de integración](docs/program-defaults.md) distingue esta implementación
de los adaptadores conservados como oráculos históricos de los experimentos.

La fixture [`program_isolation_scenario.res`](examples/legal/modules/program_isolation_scenario.res)
añade tres controles y tres casos con evidencia suficiente para las reglas del
paquete no seleccionado. Una sonda local aporta su hecho intermedio sin activar
su ProgramRef. En todos los casos sólo se justifican consecuencias mediante los
miembros seleccionados y las reglas generales del entorno, sin duplicados.
Pasaron dos tests individuales con `test_safe.py`; el experimento no cambia la
selección ni consistency y no demuestra una ley universal de aislamiento.

### Verificación con límites

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
