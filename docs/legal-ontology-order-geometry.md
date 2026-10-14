# Geometría de orden de la ontología jurídica

Investigación en `experiment-legal-order-geometry`, desde el `main`
`7b14cb8ac86d07c8d101dc36ca9f1161580f6c22`. El motor y las tres fuentes
jurídicas permanecen intactos. Los datos son sintéticos; no proceden de ningún
expediente de `12hectareas`.

Continuación: [puntos extremos y bases mínimas](legal-ontology-extreme-points.md),
con certificados de recuperación y comparación general/patrimonial.

Siguiente estudio: [fronteras mínimas de conclusiones](legal-ontology-minimal-frontiers.md),
separando premisas de teoría y evidencia Seeded.

El resultado central es una estructura de cierre sobre evidencia, con regiones
ascendentes de derivabilidad y fronteras que pueden tener varios soportes
incomparables. Además, la aciclicidad de estas reglas permite demostrar
antiintercambio: en un universo finito cerrado, su clausura matemática define una
geometría convexa **combinatoria**. Esto no afirma una representación euclidiana,
una métrica ni una identificación con el álgebra de fórmulas FOUR.

**Clasificación de afirmaciones.** `THEOREM` significa argumento matemático sobre
las reglas inspeccionadas y su clausura Horn; no una verificación universal del
código Rust. `MODEL RESULT` significa comprobación del evaluador real en los
universos declarados. `CONJECTURE` identifica una posibilidad sin demostración.

**Fuentes y correspondencia.**

| Fuente | Entidades | Constantes | Verbos | Reglas |
|---|---:|---:|---:|---:|
| `legal_monotonic.res`, `LegalMonotonicCase` | 8 | 12 | 31 | 28 |
| vendor `legal_general.res`, `LegalGeneral` | 8 | 10 | 31 | 27 |
| vendor `legal_patrimonial.res`, junto con su import | 8 | 12 | 31 | 28 |

SHA-256, en ese orden:

```text
24655c98ede7b128b8cd82df3a4473c114428b987bb0fe6a6d370c2b37f5575e
639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc
f7daf4d5eb937a7fde8534dba5618acc50d5aaa5b05ff86c9656892eb3a03fbf
```

`MODEL RESULT`: las declaraciones elaboradas de la referencia y de la partición
patrimonial son iguales por nombre y cuerpo. La parte general es exactamente su
subconjunto, excluyendo `deliver`, `molest` y `complianceAsset`. Esto incluye tipos,
parámetros, records, polaridad, tiempos y cuerpos de efectos.

Se compara también el comportamiento: en 396 estados de entrada de cinco
ventanas y en el estado sintético conjunto, referencia y partición tienen los
mismos eventos completos y el mismo multiconjunto de justificaciones. Se
conservan regla, antecedentes **ordenados**, sustituciones canónicas, tipos y
transiciones. Se ignora únicamente el valor concreto del sello `derivedAt`.

La provenance operacional **no es literalmente igual**: las 27 reglas generales
pasan de `LegalMonotonicCase` a `LegalGeneral`; `complianceAsset` pasa a
`LegalPatrimonial`. Los paths de origen también cambian. La comparación verifica
por separado esa correspondencia exacta. Las filas de
[whole-universe-provenance.tsv](legal-geometry/data/whole-universe-provenance.tsv)
conservan ambas procedencias y los sellos observados. Cada una de las 28 reglas
tiene al menos un testigo real en el estado conjunto.

**Inventario elaborado.**

| Tipo | Campos |
|---|---|
| `Person` | `patrimony : Patrimony` |
| `Thing` | ninguno |
| `Conduct` | ninguno |
| `PerformanceMode` | ninguno |
| `Relation` | `debtor : Person`, `thing : Thing?`, `conduct : Conduct?`, `creditor : Person`, `mode : PerformanceMode` |
| `Patrimony` | `assets : Set Thing`, `rights : Set Relation`, `obligations : Set Relation` |
| `Authority` | ninguno |
| `Fact` | ninguno |

No hay declaraciones de subtipado en estas fuentes. `Fact` es un identificador;
no reifica por sí mismo una proposición del core.

Constantes generales: `Doing`, `Abstaining : PerformanceMode`;
`transfer`, `enact`, `use : Conduct`; `private`, `legislative`, `judicial`,
`administrative`, `constitutional : Authority`. La extensión añade
`deliver`, `molest : Conduct`; ninguna de las 27 reglas generales las referencia.

| Categoría | Verbos |
|---|---|
| `Seeded` (12) | `Act`, `right`, `obligationFiled`, `prohibition`, `omissionProhibition`, `permissionFiled`, `extinguished`, `higherAuthority`, `notEarlier`, `patrimonialHolding`, `powerGranted`, `legislationText` |
| `Derived` (16) | `obligation`, `permission`, `compliance`, `violation`, `enforceability`, `inconsistent`, `disputedPerformance`, `patrimonialAsset`, `patrimonialLiability`, `transferred`, `owns`, `capacity`, `legislativeCapacity`, `legislativePower`, `law`, `annulled` |
| `Effect` (3) | `give`, `do`, `notDo` |

Las firmas completas, con cada argumento y su tipo, están en
[general-verbs.tsv](legal-geometry/data/general-verbs.tsv). El inventario exacto,
incluyendo todos los parámetros y términos de las reglas, está en
[general-inventory.tsv](legal-geometry/data/general-inventory.tsv) y
[patrimonial-inventory.tsv](legal-geometry/data/patrimonial-inventory.tsv).

Todos los eventos poseen tiempo proposicional. Las reglas comparten el binder
`t` entre antecedentes; sus conclusiones conservan `@ t`, salvo los tres puentes
a efectos, que concluyen `@ after t`. `effective` es otro argumento de tipo
`PropositionTime`; no se identifica con `t` ni con `DerivationTime`.

La única forma negativa usada por los cuerpos es `~Act`. No es `notDo` ni
negación por ausencia. Todos los consecuentes de estas fuentes son positivos.
Ningún verbo tiene programa asociado y no hay `Program`, defaults ni residuals
en estos tres módulos. Se usa `close_program` sin activar programas ocultos.

Las siguientes filas resumen dependencias; **no sustituyen los cuerpos exactos**:

| Regla | Antecedentes → consecuencia |
|---|---|
| `obligationCreatesRelation` | obligation → do, a `after t` |
| `prohibitionCreatesAbstention` | prohibition → notDo, a `after t` |
| `historicalTransferEffect` | transferred → give, a `after t` |
| `filedObligation` | obligationFiled → obligation |
| `filedPermission` | permissionFiled → permission |
| `obligationCompliance` | obligation & Act → compliance |
| `rightCompliance` | right & Act → compliance |
| `obligationViolation` | obligation & ~Act → violation |
| `rightEnforceability` | right & ~Act → enforceability |
| `prohibitedAct` | prohibition & Act → violation |
| `prohibitedOmission` | omissionProhibition & ~Act → violation |
| `conflictingActEvidence` | Act & ~Act → inconsistent |
| `performanceDispute` | compliance & violation → disputedPerformance |
| `violationLiability` | violation → patrimonialLiability |
| `existingAsset` | patrimonialHolding → patrimonialAsset |
| `patrimonyTitle` | patrimonialAsset → owns |
| `transferOccurrence` | Act, con conducta transfer → transferred |
| `transferTitle` | transferred → owns del destinatario |
| `ownershipCapacity` | owns → capacity de use |
| `grantedCapacity` | powerGranted de enact → legislativeCapacity |
| `capacityPower` | legislativeCapacity → legislativePower |
| `enactLaw` | Act de enact & legislativePower & legislationText → law |
| `statutoryObligation` | law → obligation |
| `permissionAnnulsProhibition` | permission & prohibition & higherAuthority & notEarlier → annulled |
| `prohibitionAnnulsPermission` | prohibition & permission & higherAuthority & notEarlier → annulled |
| `obligationAnnulsProhibition` | obligation & prohibition & higherAuthority & notEarlier → annulled |
| `prohibitionAnnulsObligation` | prohibition & obligation & higherAuthority & notEarlier → annulled |
| `complianceAsset` (sólo extensión) | compliance → patrimonialAsset del beneficiario |

Los joins inferenciales requieren compatibilidad de todos los parámetros
compartidos; una arista entre predicados no significa que una premisa aislada sea
suficiente. Autoridad y vigencia del acto son binders distintos de los de la norma
en las reglas de cumplimiento/violación. La oposición exacta de Act/~Act sí exige
los mismos argumentos, autoridad, vigencia e índice proposicional.

**Efectos y State.**

| Efecto | Operaciones declaradas |
|---|---|
| `give(Person, Thing, Person)` | retira la cosa de `subject.patrimony.assets`; la inserta en `recipient.patrimony.assets` |
| `do(Person, Conduct, Thing?, Person)` | inserta el mismo Relation en obligations del sujeto y rights del beneficiario; `mode = Doing` |
| `notDo(Person, Conduct, Thing?, Person)` | esas dos inserciones, con `mode = Abstaining` |

[general-effects.tsv](legal-geometry/data/general-effects.tsv) contiene las rutas,
los tipos de sus bases y destinos, los records y las transiciones declaradas.
La declaración del efecto mantiene extremos temporales sin resolver; en los
eventos producidos por las tres reglas puente se observa
`State(t) → State(after(t))`. `after` sigue siendo un constructor simbólico.
Este estudio no ejecuta efectos ni afirma que un Relation llegue a un Store:
observa la operación tipada declarada y el Effect conocido. No hay una arista
inferencial State → Act/~Act, ni el cierre histórico acredita una ejecución.

**Grafos y módulos.**

Los grafos se generan recorriendo `Module.declarations` elaborado. Los campos
aportan referencias estructurales con etiquetas `Set`/`Optional`; las reglas
aportan hiperaristas mediante un nodo por regla, conservando la conjunción. Se
proyectan esas hiperaristas sólo para calcular componentes y el orden acíclico.
Los nodos de proposición distinguen `Act` y `~Act`; sus categorías se etiquetan.

| Grafo | Resultado |
|---|---|
| Tipos | 8 nodos; 3 componentes débiles |
| SCC de tipos no trivial | `{Person, Patrimony, Relation}` |
| Otras componentes estructurales | `Authority` aislado; `Fact` aislado |
| Inferencia general | 32 nodos de literal; 27 nodos de regla; 76 aristas de incidencia |
| Inferencia patrimonial | 32 nodos de literal; 28 nodos de regla; 78 aristas de incidencia |
| Componentes inferenciales débiles | una con 31 literales y otra con `extinguished` |
| SCC inferenciales | todas singleton, sin bucles: DAG en ambos casos |

La SCC del esquema no introduce recursión Horn. `extinguished` está declarado,
pero ninguna regla lo consume: añadirlo no detiene compliance en el caso probado.
La anulación tampoco retira la norma ni desactiva sus consecuencias.

Se distinguen zonas regulativas, históricas, patrimoniales, legislativas y de
jerarquía, pero están conectadas. Por ejemplo, legislación produce obligaciones;
éstas producen compliance/violation y efectos; actos de transferencia producen
titularidad. No se ha obtenido una descomposición en retículas independientes.

Artefactos: [tipos](legal-geometry/general-types.svg),
[inferencia general](legal-geometry/general-inference.svg),
[inferencia patrimonial](legal-geometry/patrimonial-inference.svg),
[efectos](legal-geometry/general-effects.svg), y sus fuentes `.dot`.
Los TSV de incidencia conservan el nombre y módulo de cada regla.

El AST elaborado conserva `origins`, pero aplana los imports. Para reconstruir
las aristas de módulos se usa además el **SurfaceAST existente**, obtenido con
`parse_formal`; no se modifica el resolvedor. Los tres consumidores sintéticos
importan respectivamente referencia, general o patrimonial; patrimonial importa
general. Los archivos `*-imports.tsv` registran las rutas canónicas y módulos.
Los nombres de declaraciones son iguales; sus procedencias se conservan separadas.

**Universo de evidencia y método finito.**

Se declara una evidencia sintética conjunta `Σ` de 23 eventos Seeded distintos:
prestaciones, autoridad, legislación, transferencia, una tenencia, una marca de
extinción y dos variantes de evidencia negativa por tiempo/autoridad. Cada evento
está escrito en las [fixtures](../examples/experiments/legal_geometry/general.res).
No se incorporan reglas nuevas ni se construyen respuestas en Rust.

Se toma `E = C_P(Σ)`, con **73 eventos ground**, y `K = P(E)`, ordenado por
inclusión. [universe.tsv](legal-geometry/data/universe.tsv) enumera cada elemento
con ID, polaridad, argumentos, tipo, tiempo y transición; [initial-evidence.tsv](legal-geometry/data/initial-evidence.tsv)
identifica los 23 de entrada. Se comprueba `C_G(E)=E=C_P(E)`. Ambos operadores
utilizan el mismo universo y los mismos estados iniciales.

No se enumeran los `2^73` estados. Se estudian ventanas completas de seeds:

| Ventana | Seeds | Estados | Máximo eventos G/P | Máximo justificaciones G/P |
|---|---:|---:|---:|---:|
| Prestaciones | 7 | 128 | 17 / 20 | 13 / 16 |
| Jerarquía | 8 | 256 | 15 / 15 | 12 / 12 |
| Legislación | 3 | 8 | 8 / 8 | 5 / 5 |
| Transferencia | 1 | 2 | 5 / 5 | 4 / 4 |
| Tenencia | 1 | 2 | 4 / 4 | 3 / 3 |

Los TSV `*-states`, `*-closed`, `*-inclusions` y `*-frontiers` conservan todos
los estados de esas ventanas, sus cierres, relaciones de cobertura de las
entradas y soportes mínimos. Se comprueba extensividad e idempotencia para cada
estado, y monotonicidad para cada cobertura del orden de inclusión; por
transitividad esto cubre todas las parejas comparables de cada ventana.

Para obtener fronteras de **todo `P(E)`**, se instancian las reglas existentes
contra E: 65 cláusulas ground generales y 69 patrimoniales, conservando sus
testigos. Un análisis de hipergrafo propaga antichains de soportes, empezando por
`{e}` para cada evento y combinando soportes de las premisas. Se eliminan sólo
supersets estrictos o idénticos del mismo soporte, no justificaciones jurídicas.

`THEOREM`: toda prueba Horn finita se descompone en esos árboles de premisas;
la propagación exhaustiva de antichains calcula los soportes mínimos de este
hipergrafo finito. `MODEL RESULT`: los 284 soportes G y los 332 P se contrastan
con el **cierre real**, y se vuelve a calcular el cierre al retirar cada elemento
de cada soporte. Ninguna retirada sigue derivando la conclusión. El archivo
[full-event-frontiers.tsv](legal-geometry/data/full-event-frontiers.tsv)
incluye todos esos soportes; no es una enumeración de todas las teorías.

Estos soportes de teoría pueden incluir un Derived ya conocido. Las fronteras
de entrada externa se informan por separado: sólo usan Seeded. En particular,
`{J}` es un soporte legítimo de J en `P(E)`, pero no una hipótesis Seeded ni una
inferencia que invente J. Los `premises` de las fixtures sirven como snapshots
lógicos para este análisis; no se ejecutan sus consultas FOUR.

**Leyes de cierre y geometría.**

`THEOREM`: con oposición evidencial tratada como otro literal, las reglas son
Horn positivas. Su clausura matemática satisface extensividad, monotonicidad e
idempotencia. La aciclicidad, la seguridad de los parámetros y los efectos como
hojas aseguran clausura finita para entradas ground finitas de estas fuentes.
En el universo E cerrado, monotonicidad implica `C(S) ⊆ E` para todo `S ⊆ E`.

`THEOREM`: los estados cerrados forman una retícula completa:

```text
F ∧ G = F ∩ G
F ∨ G = C(F ∪ G)
bottom = C(∅) = ∅
top = E
```

La reflexión `C(S) ⊆ F ⇔ S ⊆ F`, para F cerrado, es una adjunción entre
teorías y estados cerrados. No es una adjunción entre `Verb` y `Rule`, ni entre
Known y FOUR. El orden del cociente se define por inclusión de cierres.

`THEOREM`, **antiintercambio para estas reglas**: si x,y son distintos y están
fuera de C(S), entonces

```text
x ∈ C(S ∪ {y})  ⇒  y ∉ C(S ∪ {x}).
```

Prueba: una derivación nueva de x debe usar y; de lo contrario x ya estaría en
C(S). Por ello existe un camino dirigido del literal de y al literal de x en el
grafo inferencial. La derivación inversa exigiría el camino opuesto. Es imposible
en el DAG inspeccionado, incluso cuando x e y tienen argumentos o tiempos
distintos. El test reconstruye ese DAG desde el AST y verifica un orden
topológico, ausencia de programas asociados y `C(∅)=∅`.

Un cierre finito con ese axioma y vacío cerrado es una geometría convexa
combinatoria en el sentido de [Edelman y Jamison, §2](https://www.researchgate.net/profile/Paul-Edelman/publication/225269790_The_theory_of_convex_geometries/links/54eb2a300cf2f7aa4d5a6ff5/The-theory-of-convex-geometries.pdf).
La prueba de aplicación a estas reglas es el argumento precedente, no una
afirmación del artículo sobre Residuum. `MODEL RESULT`: en 16 bases de teoría,
se comprueban todos los pares distintos de E fuera del cierre: 74 886 parejas
ordenadas G y 72 858 P. No aparece contraejemplo. Esto no enumera todas las bases
de `P(E)`; el alcance general procede del argumento acíclico.

Esto distingue dos problemas: un **estado cerrado completo** tiene una base
mínima única en una geometría convexa; una **conclusión particular** puede tener
varios soportes incomparables. `{O,A}` y `{R,A}` son dos soportes de compliance,
pero generan estados cerrados distintos porque conservan sus seeds. Las 14/10
clases observadas de teoría también tienen cada una un único generador mínimo
dentro de la ventana, registrado en `results.json`. No se confunde esa unicidad
con una explicación abductiva única para una conclusión.

**Regiones y fronteras de prestaciones.**

Para una conclusión ground J, `R_J={S | J ∈ C(S)}`. `THEOREM`: es un upset del
orden de inclusión. En el caso finito,
`R_J = ⋃_{H ∈ Min(R_J)} {S | H ⊆ S}`. Sus soportes mínimos forman una antichain;
no tiene por qué haber un único soporte mínimo ni ser cerrada por intersección.

En la ventana de prestaciones, todas las instancias coinciden en sujeto, cosa,
beneficiario, conducta y `@ gTau`. Abreviaturas, siguiendo el orden de bits de la
fixture: O=obligationFiled, R=right, P=prohibition, M=omissionProhibition,
L=permissionFiled, A=Act, N=~Act. La conducta `gConduct` es distinta de transfer.

| J | Frontera mínima de seeds, G y P | Estados de sus 128 |
|---|---|---:|
| obligation | `{O}` | 64 |
| permission | `{L}` | 64 |
| prohibition | `{P}` | 64 |
| compliance | `{O,A}`, `{R,A}` | 48 |
| violation | `{O,N}`, `{P,A}`, `{M,N}` | 68 |
| enforceability | `{R,N}` | 32 |
| inconsistent | `{A,N}` | 32 |
| disputedPerformance | `{O,P,A}`, `{R,P,A}`, `{O,A,N}`, `{R,M,A,N}` | 34 |
| patrimonialLiability | mismas que violation | 68 |

`MODEL RESULT`: ninguna de las seis primeras regiones contiene a otra en esta
ventana. Compliance y violation se intersectan en 34 estados, incluidos
`{O,A,N}`. El permiso no crea por sí solo una obligación ni acredita cumplimiento;
la prohibición no acredita una violación sin evidencia del acto correspondiente.
La oposición sin norma deriva inconsistent por una regla expresa, pero no
violation, law ni consecuencias arbitrarias.

En jerarquía, las cuatro fronteras de annulled tienen una norma alta, una baja y
los dos guards explícitos: higherAuthority y notEarlier. Sus parejas son
permission/prohibition, obligation/prohibition, prohibition/permission y
prohibition/obligation. No se calcula un calendario ni una jerarquía implícita.
La marca no borra los originales. Para law, el soporte mínimo de la ventana
legislativa exige conjuntamente la atribución de poder, el Act de enact y
legislationText. La tenencia y el Act de transfer tienen cada uno un soporte
unitario para sus cadenas correspondientes.

`MODEL RESULT`: una evidencia negativa a `gOtherTau` no produce la violación de
la norma a `gTau`. Una negativa a `gTau` con autoridad distinta sí puede producir
violation, pero no inconsistent respecto del Act original: las reglas exigen
coincidencias distintas. Estos resultados constan en `boundaries.tsv`.

**Comparación G/P y equivalencia inferencial.**

`THEOREM`: como `R_G ⊆ R_P`, para el mismo universo y evidencia:

```text
C_G(S) ⊆ C_P(S)
C_P(C_G(S)) = C_P(S)
C_G(C_P(S)) = C_P(S)
```

La única nueva dependencia es `compliance → patrimonialAsset`; continúa por
`patrimonyTitle → owns → ownershipCapacity`. No retorna a los predicados
regulativos ni a Act. Sus regiones y fronteras permanecen invariantes al añadir
esta regla. El pasivo por violación ya pertenece a la ontología general.

`MODEL RESULT`: en prestaciones, 48 estados adquieren tres eventos nuevos:
patrimonialAsset, owns y capacity, con causa `gNorm`. El máximo pasa de 17 a 20
eventos. Los nuevos eventos son Derived; no ejecutan give ni mutan State. Las
ventanas de autoridad, legislación aislada, transferencia y tenencia no cambian.
En todo E cambian las fronteras de 12 eventos: esos tres predicados para las
cuatro causas `gNorm`, `gHigh`, `gLow`, `gStatute`. No cambia la dependencia
histórica de transferOccurrence. `results.json` enumera los 12 eventos exactos.

Para la equivalencia `S ~ T ⇔ C(S)=C(T)`, añadir la extensión **puede unir
clases**, no refinarlas:

```text
C_G(S)=C_G(T) ⇒ C_P(S)=C_P(T), por C_P ∘ C_G = C_P.
```

Un ejemplo estricto usa S=`{O,A}` y T=`{O,A,H}`, donde H es el patrimonialAsset
del beneficiario con causa gNorm. En G, T contiene consecuencias que S no tiene.
En P, H ya se deriva de S, y ambos cierres coinciden. En la ventana de teoría
`{O,A,compliance,H}`, los 16 estados forman 14 clases G y 10 P. Sus diagramas de
Hasse de **los cierres observados**, no de todos los estados cerrados de E, están
en [general](legal-geometry/general-theory-hasse.svg) y
[patrimonial](legal-geometry/patrimonial-theory-hasse.svg).

En las ventanas exclusivamente Seeded las clases permanecen singleton.
`THEOREM`: ninguna regla tiene un consecuente Seeded, por tanto el cierre retiene
exactamente la proyección original de seeds, y dos entradas distintas no pueden
tener el mismo cierre completo. Sí pueden tener los mismos observables
proyectados, pero esa sería otra equivalencia. Incluso G tiene estados distintos
equivalentes de teoría: `{O,A}` y `{O,A,compliance}`.

**Contraejemplos geométricos.**

Los siguientes usan estos mismos eventos y reglas, sin introducir reglas de
ejemplo. Se verifican en ambos evaluadores G/P:

| Propiedad que no se cumple | Contraejemplo |
|---|---|
| Aditividad de C respecto de unión | compliance está en `C({O,A})`, pero no en `C({O}) ∪ C({A})` |
| Preservar intersecciones de entradas | `{O}` y `{obligation}` son disjuntos; sus cierres comparten obligation y do, mientras `C(∅)=∅` |
| Región de compliance cerrada por intersección | `{O,A}` y `{R,A}` derivan compliance; su intersección `{A}` no |
| Cumplimiento incompatible con violación | `{O,A,N}` deriva ambos, además de disputedPerformance |
| Oposición equivale a explosión | `{A,N}` deriva inconsistent; no deriva violation ni law |
| Retícula de estados cerrados distributiva | tomar `x=C({compliance})`, `y=C({O})`, `z=C({A})`: `x ∧ (y ∨ z)=x`, pero `(x ∧ y) ∨ (x ∧ z)=∅` |
| Intercambio de matroide | obligation está en `C({O})`; O no está en `C({obligation})` |
| Añadir reglas refina siempre las clases | S/T con H del ejemplo anterior quedan identificados en P |

La retícula no distributiva es compatible con antiintercambio. Tampoco se
identifica el join de teorías cerradas con `&`, ni unión de evidencia con suma o
fusion FOUR. Los efectos y los records son otra interpretación operacional.

**Reproducción y límites.**

El único cambio en un archivo existente es registrar un nuevo módulo **de tests**
en `src/bin/residuum/cli/tests.rs`. El harness está en
`src/bin/residuum/cli/legal_geometry_tests.rs`; no implementa reglas jurídicas,
ni modifica parser, elaborador, Rule, Program, close, defaults o runtime.

Ejecutar únicamente los siete tests seleccionados, cada uno en un proceso
separado del runner:

```text
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::legal_geometry::correspondence_and_typed_inventory \
  --test cli::tests::legal_geometry::performance_regions_and_closures \
  --test cli::tests::legal_geometry::authority_regions_and_closures \
  --test cli::tests::legal_geometry::legislation_transfer_and_evidential_boundaries \
  --test cli::tests::legal_geometry::common_universe_quotients_and_lattice_counterexamples \
  --test cli::tests::legal_geometry::full_event_frontiers_without_powerset_enumeration \
  --test cli::tests::legal_geometry::anti_exchange_on_controlled_theory_states
python3 scripts/analyze_legal_geometry.py --render
```

El script Python sólo resume los exports del AST/evaluador y produce grafos;
no ejecuta Residuum ni contiene otra implementación de close. `--render` requiere
Graphviz. La disposición de los dibujos sólo sirve para leer dependencias; no
define coordenadas ni distancias en K. Los imports vendor apuntan al checkout hermano indicado
por el encargo; para reproducir en otro equipo hacen falta esas fuentes con los
SHA anteriores. El script rechaza fuentes distintas.

Los siete tests pasan individualmente con 1 GiB, sin swap, un job de Cargo,
un hilo y los timeouts existentes. No se ejecutó la suite completa.
La verificación final está en
`target/test-diagnostics/20261009T233532352021Z-574764`:

| Test individual | Segundos | Pico RSS (MiB) |
|---|---:|---:|
| Correspondencia e inventario | 1.3669 | 15.57 |
| Prestaciones | 0.8778 | 15.58 |
| Autoridad | 1.2645 | 16.39 |
| Legislación, transferencia y fronteras evidenciales | 1.6096 | 15.64 |
| Universo común y cocientes | 0.5505 | 15.34 |
| Fronteras de P(E) | 0.8566 | 15.56 |
| Antiintercambio | 1.1213 | 15.56 |

La compilación final de la última ampliación del harness consumió 349.59 MiB RSS.
El primer intento de compilación encontró dos accesos incorrectos a un campo
en el test nuevo; se corrigieron en el harness, sin cambios al motor. Ese intento
alcanzó 571.69 MiB RSS. No hubo timeout ni OOM, y ningún límite se alteró.

Limitaciones: no se cuentan todos los estados cerrados de P(E); no se enumeran
todas sus relaciones de inclusión; no se estudia el universo infinito de objetos
o tiempos. Las fronteras completas calculadas corresponden sólo a estos 73
eventos. Provenance se observa por records y se decora con los orígenes públicos;
no se exige que el record contenga físicamente el módulo. Los grafos de ejecución
de State se describen por sus operaciones tipadas, sin fabricar snapshots.

`CONJECTURE`: esta representación puede ayudar a comparar otros módulos
jurídicos mediante sus fronteras y bases de evidencia. No se afirma que cualquier
ontología futura conserve antiintercambio: ciclos inferenciales, retractación o
defaults exigirían un nuevo análisis. No quedó una propiedad pendiente que
necesitara modificar el motor para observarla dentro del alcance declarado.
