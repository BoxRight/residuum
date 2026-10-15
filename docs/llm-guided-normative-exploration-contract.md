# Contrato ejecutable de la exploración normativa guiada por LLM

Documento compañero de
[la especificación](llm-guided-normative-exploration-specification.md).
Rama `experiment-llm-normative-exploration-spec`, base `main` en `7af8d39`,
2026-10-14. La especificación estaba sin commit al empezar y se conserva sin
cambios. Este documento no implementa el ciclo autónomo del LLM, no añade
primitivas al motor, no toca `main`, `v1`, la ontología ni las fixtures
originales y no ejecuta la suite.

Revisión del mismo día: el autor del proyecto estipuló la decisión `D` para
Riggs (apartado 5.3) y aceptó el diseño de parametrización de la capa
geométrica (apartado 6.1). Ambas cosas se incorporan como decisiones de
diseño. A continuación el autor autorizó implementar las partidas
independientes del controlador: el módulo `src/experiments/exploration.rs`
realiza las partidas 2, 3, 4 y 8 del apartado 6 y está verificado por cuatro
tests individuales (apartado 6.2). El apartado 3.5 añade las sesiones stepwise
que el contrato inicial no había mapeado. El controlador LLM sigue sin
implementarse.

Las capacidades se confirmaron leyendo los caminos actuales del código en ese
commit; los números de línea son los de esa revisión. En esta etapa no se
compiló ni se ejecutó ningún test. Cuando un dato procede de un certificado
anterior se cita el certificado, no se reafirma como ejecutado hoy.

Se usan cuatro estados, como en los contratos previos:

| Marca | Significado |
|---|---|
| **API PÚBLICA** | Función o tipo exportado por `residuum` (`src/lib.rs`) o el binario `residuum`. |
| **HELPER PRIVADO** | Función `fn`, `pub(crate)` o `pub(super)` del motor, o helper de un módulo de tests del harness `src/bin/residuum/cli/*_tests.rs`. Reutilizable sólo copiándolo a un adaptador o exponiéndolo expresamente. |
| **RESULTADO FINITO** | Certificado ya calculado sobre un universo acotado y archivado en `docs/legal-geometry/`. |
| **PROPUESTA** | Sin implementación; decisión de este contrato pendiente de implementación o deliberación. |

## 1. Matriz código → contrato

### 1.1 Carga y elaboración de módulos

| Operación del contrato | Implementación | Estado | Observaciones para el adaptador |
|---|---|---|---|
| Cargar `T` con grafo de módulos y orígenes | `modules::load_formal_module(path) -> Result<Module>` (`src/modules.rs:17`); `Module { name, declarations, origins }` (`src/typed.rs:2`); `DeclarationOrigin { module, file }` (`:10`) | API PÚBLICA | Límite 1 MiB de fuente y 64 archivos por grafo. Los `experiment` importados se descartan por el loader. `Module::origin(name)` resuelve también `RuleRef` y `Verb.program`. |
| Elaborar fuente en memoria | `parse_formal_to_typed(&str)` (`src/lib.rs`), `elab::elaborate_formal` (`src/elab.rs:58`) | API PÚBLICA | Camino usado por los tests estructurales para los 16 controles. Un módulo en memoria no tiene `file` en sus orígenes. |
| Hashes de declaraciones y fixtures | No los produce el loader. El harness Riggs registra SHA-256 de entradas en `docs/legal-geometry/riggs-rule-learning/input-sha256.tsv` | HELPER PRIVADO | El manifiesto debe calcular y archivar hashes de cada `.res` cargado; es lógica de registro, no de inferencia. |
| Inventario tipado de firmas/constantes | `cli::tests::legal_geometry::correspondence_and_typed_inventory` (`src/bin/residuum/cli/legal_geometry_tests.rs:329`); exportación en `docs/legal-geometry/search-space/signatures.json` | HELPER PRIVADO + RESULTADO FINITO | El JSON contiene 11 firmas, 8 tipos, 10 constantes generales y 19 sintéticas; su `origin_file` apunta a una copia vendor de `12hectareas`, no a `examples/`. El manifiesto del apartado 2 re-deriva constantes del `Module` cargado desde `examples/`, no de ese campo. |

### 1.2 Rules, Programs, RuleUses y aplicación parcial

| Operación | Implementación | Estado | Observaciones |
|---|---|---|---|
| Cláusula individual | `typed::Rule { name, parameters: Vec<Arg>, body: RuleBody }` (`src/typed.rs:291`); `RuleBody::{HornClause, ResidualClause}` (`:396`); `Antecedent::{Unit, Event, And}` (`:451`) | API PÚBLICA | Campos públicos: el harness puede construir `Rule` tipadas directamente, como hace `structural_generation_tests::rule` (`:124`). |
| Program y RuleUse | `typed::Program { identity, parameters, rule_uses, defining_module }` (`:366`); `RuleUse { rule: RuleRef, bindings: Vec<ProgramBinding> }` (`:374`) | API PÚBLICA | No existen Programs anidados: los usos apuntan a `Rule`. |
| Aplicación parcial | `programs::specialize_program(program, args: &[String], constants)` (`src/programs.rs:110`) | API PÚBLICA | Sustituye un **prefijo** de parámetros por constantes declaradas con tipo **exactamente igual**. No unifica símbolos ni acepta variables. |
| Compilación a reglas ordinarias | `programs::compile_program(program, module) -> Vec<Rule>` (`:185`) | API PÚBLICA | Renombrado higiénico `#P:shared:x` y `#P:useN:x`; nombres `P::i::rule`. Parámetros compartidos sin binding permanecen binders. |
| Mapeo RuleUse → sustitución | `programs::rule_use_substitution` (`:202`) | HELPER PRIVADO (`pub(crate)`) | Compartido por compilador y resolución de bindings; verifica que el RuleUse identifique la Rule original. |
| Origen de un productor compilado | `programs::program_rule_origin(module, producer)` (`:254`) | API PÚBLICA | Decodifica `P::i::rule` sin tocar `DerivationRecord`. |
| Elaboración de Program desde fuente | `programs::lower_program` (`:8`) | HELPER PRIVADO | Sólo accesible vía `parse_formal`/`load_formal_module`. |

### 1.3 Sesiones forward y cierre completo

| Operación | Implementación | Estado | Observaciones |
|---|---|---|---|
| Cierre completo con asociaciones | `derive::close_program(seeds, module, clock) -> Result<Closure>` (`src/derive.rs:194`) | API PÚBLICA | Rechaza módulos con defaults. **El llamador garantiza terminación**; no hay cota interna de rondas. |
| Oráculo de comparación | `derive::close_program_reference` (`:205`, `#[doc(hidden)]`) | API PÚBLICA oculta | Misma semántica y mismo requisito de terminación. Usado como contraste en tests de investigación. |
| Cierre de reglas sueltas | `derive::close(seeds, rules, clock)` (`:187`) | API PÚBLICA | Sin asociaciones. Es el cierre que la CLI muestra para consultas con `using rules`. |
| Sesión forward sincrónica | `ForwardSession::{new(seeds, rules), for_module(seeds, module)}` (`src/derive/steps.rs:30,33`); `step(clock) -> ForwardStep` (`:62`); `trace(max_steps, clock)` (`:130`); `closure()`, `is_complete()` | API PÚBLICA | `ForwardStep { round, consequences, witnesses, added, derivations, known, complete }`. Un prefijo truncado no es cierre; `complete` sólo cuando `added` está vacío. |
| Derivada discreta | `IncrementalClosure::begin_stepwise_update` → `DerivativeSession` (`:155`) | API PÚBLICA | Fuera del contrato inicial; se cita por completitud. |
| Registro de justificaciones | `Closure { known, derivations: Vec<DerivedEvent> }` (`src/typed.rs:256`); `DerivedEvent { event, derived_at, record }`; `DerivationRecord { rule, antecedents, substitution }` (`:267`) | API PÚBLICA | `derived_at` es el único campo que puede variar entre ejecuciones independientes. |
| Preparación de reglas ordinarias/asociadas | `derive::prepare_programs` (`:214`) | HELPER PRIVADO | Excluye del cierre ordinario las definiciones asociadas. No es invocable por el harness; la sesión `for_module` ya lo aplica. |
| Comprobación conservadora de aciclicidad | `experiments::require_program_acyclic` (`src/experiments.rs:262`), `require_acyclic` (`:332`) | HELPER PRIVADO | Sólo la ejecuta el runner de fuente (`run_experiments`, `run_forward_experiments`). **Una sesión construida por la API Rust no la ejecuta.** |

### 1.4 Consultas residuales con bindings explícitos

| Operación | Implementación | Estado | Observaciones |
|---|---|---|---|
| Datos de scope y bindings | `HornQueryScope::{Rules(Vec<RuleRef>), Program(ProgramRef)}` (`src/typed.rs:110`); `QueryBinding { scope: QueryParameterScope, parameter, ty, value: SubstitutionValue }` (`:100`); `QueryOptions { active, scope, bindings }` (`:80`) | API PÚBLICA | El scope es exacto; no se añaden dependencias. |
| Consulta Horn completa con bindings | `query_program_residual_with_bindings(module, scope, evidence, goal, bindings, limits, allowed)` (`src/residual/bindings.rs:58`) | API PÚBLICA | Bindings vacíos delegan al camino legado. Rechaza defaults. |
| Consulta local a una Rule | `query_residual_with_bindings(module, rule, goal, known, bindings)` (`:28`) | API PÚBLICA | Contrato Unit local documentado en `residual-explicit-bindings-api.md`. |
| Sesión Horn por capas | `begin_residual_query_with_bindings(...) -> QueryWithBindings<ResidualSession>` (`:138`); `ResidualSession::{new, step, trace, result, evidence}` (`src/residual/steps.rs:80-190`) | API PÚBLICA | `ResidualStep { round, branches, pending, metrics, complete }` (`:71`). `result()` sólo tras completar; se certifica con `certify_candidates` → replay forward. |
| Selección de scope | `residual::scoped_rules` (`bindings.rs:108`) | HELPER PRIVADO (`pub(crate)`) | Compartido por API, fuente y trazas. |
| Resolución y preflight de bindings | `resolve` (`bindings.rs:460`), `preflight` (`:543`), `validate_query_options` (`:358`) | HELPER PRIVADO | Errores tipados: duplicado, parámetro desconocido, fuera de scope, tipo incompatible, conflicto con RuleUse especializado. |
| Límites y métricas | `SearchLimits { max_depth 1..=64, max_nodes, max_branches }` (default 32/4096/4096, `src/residual/program.rs:67`); `SearchMetrics` (`:86`) | API PÚBLICA | Cuentan nodos y ramas intermedias; DFS y capas no agotan en el mismo punto. |
| Parámetros no resueltos | `engine::expand_rule` (`src/residual/engine.rs:158`) devuelve `Err("rule ... has unresolved query parameters ...; no complete result")` (`:187`) | HELPER PRIVADO | **No hay enumeración existencial.** El error es terminal y la sesión queda fallida (`steps.rs:139`). |
| Ciclos y presupuesto | `engine::classify_event` (`:109`) → `Err("cyclic residual search ...")`; `engine::visit`/`reserve_branch` (`:77,90`) → `Err("... limit exceeded; no complete result")` | HELPER PRIVADO | Ninguno produce resultado parcial. |

### 1.5 Consultas residuales activas

| Operación | Implementación | Estado | Observaciones |
|---|---|---|---|
| Consulta activa completa | `query_active_program_residual{,_with_abducibles}` (`src/residual/active.rs:70,85`); con bindings y scope: `query_active_program_residual_with_bindings` (`bindings.rs:235`), `..._in_scope_with_bindings` (`:251`) | API PÚBLICA | Resultado `ActiveProgramResidualQuery { goal, required, explanations: Vec<CertifiedActiveExplanation>, metrics, failure }` (`active.rs:45`). |
| Sesión activa por capas | `begin_active_residual_query_with_bindings` (`bindings.rs:164`), `begin_active_residual_query_in_scope_with_bindings` (`:175`) → `ActiveResidualSession::{step(clock), trace, result}` (`steps.rs:310-390`) | API PÚBLICA | Al completar la búsqueda, `step` llama a `active::certify`. |
| Certificación | `active::certify` (`active.rs:98`) | HELPER PRIVADO (`pub(super)`) | **Por cada rama candidata ejecuta `close_program` completo** sobre evidencia + hojas hipotéticas del módulo original (`:106-110`). Un paso residual puede costar un cierre entero por rama. |
| Proyección guardada | `active::guarded_view` (`:195`); `ResidualActivation { verb, behavior, rule, trigger, substitution, rule_origin, program_origin, search_rule }` (`:19`) | HELPER PRIVADO | Sólo productores guardados; nunca cláusulas compiladas sin guarda. |
| Fallos | `AbductionFailure::{NotAbducible, NotForwardSupported}` (`program.rs:95`) | API PÚBLICA | Presente sólo si `explanations` está vacío. Errores de búsqueda van por `Err`. |

En las fixtures de este contrato (`LegalGeneral`, `GeometryGeneral`,
`RiggsSuccessionReference`) no hay declaraciones `Verb.program`; la proyección
guardada coincide con el conjunto de reglas ordinarias. El valor de la sesión
activa en el primer episodio es su certificación mediante `close_program`,
no una semántica de activación.

### 1.6 Certificación y provenance

| Operación | Implementación | Estado | Observaciones |
|---|---|---|---|
| Teoría hipotética `T ⊕ Γ` por adición | `Study::theory(mask)` (`src/bin/residuum/cli/legal_geometry_riggs_rule_learning_tests.rs:129`): clona el `Module`, añade la declaración Derived de la cabeza y una cláusula, copia sus orígenes y verifica que todas las declaraciones originales y sus orígenes sean idénticos | HELPER PRIVADO | Es el único camino de instalación ya certificado. Se basa en operaciones públicas (`Module.declarations.push`, `origins.insert`). |
| Cierre hipotético | `Study::run` (`:158`) → `legal_geometry_tests::close` (`:75`) → `close_program` | HELPER PRIVADO sobre API PÚBLICA | |
| Conservación de la referencia | `Study::certify` (`:165`): proyecta fuera la cabeza nueva, compara `full_events` y `proof_identity` con la base y con un cierre fresco, ignora sólo `derivedAt`; `audit_proofs` (`legal_geometry_tests.rs:307`); `proof_identity` (`legal_geometry_riggs_tests.rs:206`); `audit_reference` (`:220`) | HELPER PRIVADO | 384 comparaciones archivadas: RESULTADO FINITO (`riggs-rule-learning/verification.md`). |
| Orígenes de reglas y programas | `Module::origins`, `program_rule_origin`, `ResidualActivation.{rule_origin, program_origin}` | API PÚBLICA | |
| Pruebas residuales | `ResidualProof::{Unit, Evidence, Hypothesis, Tensor, Rule{rule, goal, substitution, antecedent}}` (`program.rs:47`); `RequirementFormula` (`:25`) | API PÚBLICA | Testigos ordenados y sustituciones canónicas. |

### 1.7 Soportes mínimos, comparación de cobertura y particiones

| Operación | Implementación | Estado | Observaciones |
|---|---|---|---|
| Soportes mínimos sobre hipergrafo ground | `legal_geometry_tests::finite_frontiers(m, e, allowed)` e `insert_minimal` (`legal_geometry_tests.rs:748,740`) | HELPER PRIVADO | **Asserta `e.len() == 73`**: está fijado al universo del estudio. Reutilizarlo exige generalizar el tamaño del universo. La especificación lo situaba en `legal_geometry_frontier_tests.rs`; ese archivo contiene los tests que lo consumen (`:28,150`). |
| Soportes Seeded en Riggs | `grounding`, `saturation`, `minima` (`riggs_rule_learning_tests.rs:480,467,498`) | HELPER PRIVADO | Enumeración de 64 subconjuntos de seis seeds; RESULTADO FINITO en `minimal-seed-supports.tsv`. |
| Completaciones exhaustivas activas | `exhaustive_completions(m, domain, known, goal)` (`legal_geometry_active_frontier_tests.rs:13`) | HELPER PRIVADO | Oráculo por fuerza bruta sobre un dominio declarado. |
| Bases extremas | `extreme_points`, `verify_basis` (`legal_geometry_extreme_tests.rs:10,37`) | HELPER PRIVADO + RESULTADO FINITO | |
| Vector de predicciones por candidata | `predictions(study) -> Vec<Vec<bool>>` (`riggs_rule_learning_tests.rs:292`) | HELPER PRIVADO | Es la única comparación de cobertura implementada: pertenencia de la cabeza por expediente. |
| Comparación de cobertura `Cov(Γ1) ⊆ Cov(Γ2)` | Inclusión de vectores de predicciones sobre un dominio finito | PROPUESTA | Lógica nueva trivial sobre `close_program`; no existe subsunción simbólica (contrato previo §7). |
| Particiones de hipótesis | `Partition::{of, le, meet, join}`, `all_partitions` (`legal_geometry_partition_tests.rs:21-91`, `const N = 9`); `kernel`, `meet`, `finer` (`legal_geometry_conversation_tests.rs:389,412,422`, `pub(super)`) | HELPER PRIVADO | `Partition` tiene tamaño fijo en compilación. `kernel/meet/finer` sobre `Vec<usize>` sí son reutilizables con carrier variable, pero sólo dentro del binario de tests. |
| Casos discriminantes | `scores`, `best` (`conversation_tests.rs:438,467`); `submit` (`:322`) | HELPER PRIVADO + RESULTADO FINITO | Minimax 8/8 en Riggs (`questions.tsv`). |
| Generación estructural certificada | `structural_generation_tests::classify(m, r, connected)` (`legal_geometry_structural_generation_tests.rs:255`), `term_check` (`:183`), `rule` (`:124`) | HELPER PRIVADO + RESULTADO FINITO | `MAX_ATOMS = 4`, 16 binders, `ALPHABET` de 12 literales y cabeza `compliance` **fijados en constantes**. Los códigos emitidos (`B_ATOMS`, `B_VARIABLES`, `S_DUPLICATE_BINDER`, `S_UNBOUND_HEAD`, `P_HEAD_INTERFACE`, `S_UNKNOWN_VERB`, `P_BODY_VOCABULARY`, `P_POLARITY`, `S_ARITY`, `S_TYPE`, `S_UNKNOWN_TERM`, `B_AFTER_DEPTH`, `S_DISCONNECTED`) son la base del validador propuesto. |

### 1.8 CLI y runner

| Operación | Implementación | Estado | Observaciones |
|---|---|---|---|
| Comandos | `residuum <run|query|explain> <file.res> [--steps N] [--trace]` (`src/bin/residuum/cli.rs:20`); `StepOptions::parse` (`cli/steps.rs:8`); `execute_file_steps` (`:44`) | API PÚBLICA (binario) | `--steps` en 1..=4096; `--trace` por defecto 64. **Cada invocación crea una sesión nueva**; no existe reanudación. |
| Consulta de fuente con scope/bindings | `experiments::run_configured_query` (`src/experiments.rs:160`) | HELPER PRIVADO vía `run_experiments` (API PÚBLICA) | Activa: `require_program_acyclic` + consulta completa + `close_program` del módulo original. Horn: `require_acyclic(rules)` + `close(input, rules)` **sólo con las reglas del scope**. Límites siempre `SearchLimits::default()`. |
| Runner con límites | `python3 scripts/test_safe.py --fixture F --cli query [--steps N] [--trace]`; `--test-target {lib,cli} --test NOMBRE` | API PÚBLICA (script) | Fixture ≤ 64 KiB; 1 GiB, swap 0, un trabajo, un hilo; cgroup obligatorio. |

### 1.9 Sin implementación

| Objeto | Estado |
|---|---|
| Política LLM, bucle de episodios, aislamiento A/B/C, métricas | PROPUESTA (apartados 3 a 6). |
| Validador estructural parametrizado por perfil y cabeza | PROPUESTA; adaptación de `classify`. |
| Comparador de cobertura, particiones con carrier dinámico, soportes mínimos con universo variable | PROPUESTA; adaptación de helpers privados. |
| Decisión efectiva `D` para exclusión sucesoria | PENDIENTE DE DELIBERACIÓN (apartado 5). |
| Subsunción simbólica, operadores mínimos completos, Candidate Elimination | No existen; no se asumen. |

## 2. Manifiesto propuesto del perfil finito `P1`

Todo el apartado es **PROPUESTA**. Las firmas se copian del `Module` elaborado
y coinciden con `docs/legal-geometry/search-space/signatures.json`. `P1` es un
perfil general; cada problema añade un **manifiesto de problema** separado que
declara sus constantes, verbos y salidas específicos, identificados como tales.

### 2.1 Vocabulario y firmas

Doce literales firmados, con canal y polaridad admitida:

```text
Efectos (Effect), positivos:
  give(subject : Person, object : Thing, recipient : Person) @ t
  do(subject : Person, conduct : Conduct, indirectObject : Thing?, beneficiary : Person) @ t
  notDo(subject : Person, conduct : Conduct, indirectObject : Thing?, beneficiary : Person) @ t

Evidencia histórica (Seeded), ambas polaridades:
  Act(id : Fact, subject : Person, conduct : Conduct, object : Thing,
      beneficiary : Person, authority : Authority, effective : PropositionTime) @ t
  ~Act(...)   // Polarity::EvidentialNot, misma firma

Calificaciones normativas, positivas, siete argumentos iguales a Act:
  obligationFiled      Seeded
  obligation           Derived
  right                Seeded
  permissionFiled      Seeded
  permission           Derived
  prohibition          Seeded
  omissionProhibition  Seeded
```

Obligación y permiso conservan sus dos canales; no se funden. `~Act` es el
único literal negativo del perfil general. Otros literales negativos requieren
listarse en el manifiesto de problema (Riggs: `~intentionalKilling`,
`~acquisitionPurposeRecorded`, ya presentes en su catálogo existente).

Los nombres de argumento son posiciones. `Person` no distingue acreedor,
deudor ni beneficiario; dos variables `Person` distintas pueden coincidir en el
matching y no expresan desigualdad. Ninguna comprobación del perfil introduce
desigualdad, exclusividad, prioridad ni suficiencia jurídica.

### 2.2 Interfaces de salida e intermedias

| Interfaz | Contenido en `P1` | Uso |
|---|---|---|
| Cabezas intermedias `H_int` | `obligation`, `permission` (Derived del vocabulario) | Cabeza de una cláusula cuya conclusión puede aparecer en el cuerpo de otra cláusula de la misma configuración o alimentar reglas de `T`. |
| Cabezas de salida `H_out` | `compliance`, `violation`, `enforceability` (Derived de `LegalGeneral`); `give`, `do`, `notDo` (Effect) | Cabeza de la cláusula final. `compliance` es una opción, no la única. |
| Interfaz observable `Q` | Subconjunto de `H_out ∪ H_int ∪` salidas del problema, declarado por episodio | Lo que `D` observa. |
| Cabezas excluidas | Todo verbo Seeded (`Act`, `~Act`, `obligationFiled`, `right`, `permissionFiled`, `prohibition`, `omissionProhibition`) | Política de investigación: no se aprende evidencia histórica. |

Las firmas de `compliance`, `violation` y `enforceability` son
`(norm : Fact, act : Fact, subject : Person, object : Thing, beneficiary :
Person, authority : Authority, effective : PropositionTime) @ t`, según el
inventario elaborado.

Manifiesto de problema Riggs (identificado como específico):

| Elemento | Declaración | Procedencia |
|---|---|---|
| Teoría `T` | `RiggsSuccessionReference` = `LegalGeneral` + 4 reglas | `examples/experiments/legal_geometry/riggs_succession_reference.res` |
| Literales de cuerpo adicionales | `operativeDisposition`, `deathRecorded`, `survivorRecorded`, `intentionalKilling`, `~intentionalKilling`, `deathCausedBy`, `acquisitionPurposeRecorded`, `~acquisitionPurposeRecorded` (Seeded); `testamentaryEntitlement`, `acquisitionLinkedToWrong`, `purposefulAcquisition`, `patrimonialAsset` (Derived) | Misma fixture y `LegalGeneral` |
| Salida experimental | `hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective)` | Declaración Derived de `riggs_candidate_rules.res`, no un primitivo universal |
| Constantes | `francisPalmer`, `elmerPalmer`, `otherKiller`, `residuaryInterest`, `palmerWill`, `palmerDeath`, `killingEvidence`, `will1880`, `death1882`, `dossierTime` | `riggs_cases.res` |
| Expedientes `A` | Los seis `experiment` de `riggs_cases.res` y, para dominio acotado, los 64 subconjuntos de los seis seeds históricos | `legal_geometry_riggs_tests.rs:10` |

Las seis plantillas `OPTIONS` y la obligatoriedad de `E` del catálogo Riggs
(`riggs_rule_learning_tests.rs:7-17`) **no** forman parte de la gramática.
Son un catálogo anterior que sirve de control: un candidato generado bajo `P1`
que coincida con una de las 64 reglas debe registrarse como tal.

### 2.3 Constantes y bindings admitidos

| Conjunto | Miembros | Tipo |
|---|---|---|
| `C_general` | `Doing`, `Abstaining` : PerformanceMode; `transfer`, `enact`, `use` : Conduct; `private`, `legislative`, `judicial`, `administrative`, `constitutional` : Authority | Elaboradas de `LegalGeneral` |
| `C_sint` | `gSubject`, `gBeneficiary`, `gLegislator` : Person; `gObject` : Thing; `gConduct` : Conduct; `gNorm`, `gAct`, `gHigh`, `gLow`, `gGrant`, `gEnact`, `gStatute`, `gTransfer`, `gHolding` : Fact; `gTau`, `gOtherTau`, `gDate`, `gEarlyDate`, `gLateDate` : PropositionTime | `GeometryGeneral` (`general.res`) |
| `C_problema` | Declaradas por el manifiesto de problema | Riggs: las diez anteriores |

Términos admitidos por posición:

```text
Term[T] ::= variable tipada T
          | constante de C con tipo exactamente T (sin subtipos: las fuentes no los declaran)
          | none          sólo en posiciones Thing? (indirectObject de do/notDo)
          | after(Term[PropositionTime])   sólo en argumentos PropositionTime, profundidad ≤ 2
Time    ::= variable PropositionTime | constante PropositionTime | after(variable | constante)
```

El índice `@` se limita a `t` o `after t` porque el parser fuente no admite
`@ after(after t)` (certificado del contrato estructural, §5). Un AST con índice
más profundo se registra como no expresable en fuente y se rechaza en `P1`.
No se admiten records en candidatos (`d_r = 0`). El LLM no puede introducir
constantes ni verbos nuevos; cualquier nombre fuera del manifiesto produce
`S_UNKNOWN_TERM` o `S_UNKNOWN_VERB`.

Bindings en RuleUses (perfil `P1-program`, apartado 2.5): constantes de `C`
o parámetros compartidos del Program; los omitidos quedan locales, como en
`compile_program`.

### 2.4 Límites

| Parámetro | Cota `P1` | Código de rechazo |
|---|---:|---|
| Cláusulas nuevas por configuración `r` | 2 (una con cabeza en `H_int`, una con cabeza en `H_out ∪ Q`; o una sola) | `B_CLAUSES` |
| Átomos por cuerpo `a` | 4 | `B_ATOMS` |
| Binders por cláusula `v` | 16 | `B_VARIABLES` |
| RuleUses por Program `u` | 4, sólo en `P1-program` | `B_RULE_USES` |
| Profundidad de `after` en argumentos `d_t` | 2 | `B_AFTER_DEPTH` |
| Profundidad de records `d_r` | 0 | `B_RECORD_DEPTH` |
| Nodos `I` por cuerpo | 1, y sólo si el cuerpo no tiene átomos | `B_UNIT_NODES` |
| Nodos `&` por cuerpo | `a − 1` (ninguno si `a ≤ 1`) | `B_AND_NODES` |
| Nodos totales del árbol del cuerpo | `max(1, 2a − 1) ≤ 7` | derivado de los dos anteriores |
| Módulos autorizados `M` | el grafo de `T` más un módulo privado por candidato | `S_MODULE` |
| Profundidad de la configuración | una cláusula intermedia puede alimentar una final; no hay cadenas más largas dentro del candidato | `B_CLAUSES` |

Representación de cuerpos: secuencia ordenada de átomos; `I` representa la
secuencia vacía; `&` asociado a la izquierda. Formulaciones con otra asociación
o con `I` redundante se rechazan por `B_UNIT_NODES`/`B_AND_NODES` en lugar de
normalizarse en silencio. El texto original del candidato se conserva siempre.
Dos candidatos con la misma secuencia pero distinto orden de antecedentes son
candidatos distintos; dos candidatos con igual cobertura siguen siendo
registros distintos.

Canonización para indexación: las variables se renombran `x1..xn` por orden de
primera aparición en cuerpo y después cabeza; los nombres originales se guardan
en el registro. El índice canónico sirve para detectar reenvíos idénticos, no
para fusionar candidatos.

Con firmas, constantes y cotas finitas, el conjunto de AST admisibles es finito
por el argumento del contrato previo (§5), ahora con los nodos `I`/`&` acotados.
No se reclama cota práctica ni terminación de cada cierre hipotético.

### 2.5 Instalación y activación permitidas

| Perfil | Operación `⊕` | Camino existente | Estado en el primer episodio |
|---|---|---|---|
| `P1-add` | Adición de cláusulas ordinarias en un módulo privado `Candidate<id>` que importa la referencia; la cabeza nueva se declara Derived si no existe | Generación de fuente Formal → `parse_formal_to_typed`/`load_formal_module` → copia tipada con `Study::theory` | **Permitido**; único camino ya certificado (384 comparaciones). |
| `P1-program` | Declarar un `Program` con ≤ 4 RuleUses sobre reglas de `T` y cláusulas del candidato, compilarlo con `compile_program` e instalar las reglas compiladas como ordinarias | `compile_program` + adición | Permitido sólo si el manifiesto del episodio lo activa; sin certificado previo de conservación. |
| Asociación `Verb.program` | Modificar la declaración de un verbo de `T` | Edición de la ontología | **No permitido**: altera `T`. |
| Defaults, `State`, `execute_effect` | — | — | **No permitido** (fuera de `close_program` y de residual). |
| Retirada o sustitución de reglas | — | Sólo copias privadas del antiguo harness de compliance | **No permitido** en `P1`; véase apartado 5. |

Toda cláusula instalada recibe origen `Candidate<id>`; las declaraciones y
orígenes de `T` se comprueban idénticos antes y después, como en `Study::theory`.

### 2.6 Verificaciones estructurales

Orden de comprobación de una propuesta:

1. **Sintaxis**: la fuente generada se analiza con `parse_formal`. Fallo →
   `S_SYNTAX`, con el texto original conservado.
2. **Perfil** (`classify` parametrizado): `B_*`, `P_HEAD_INTERFACE` (cabeza en
   `H_int ∪ H_out ∪ Q`), `P_BODY_VOCABULARY` (átomos en el vocabulario del perfil
   más el del problema más cabezas intermedias del propio candidato),
   `P_POLARITY`, `S_ARITY`, `S_TYPE`, `S_UNKNOWN_TERM`, `S_UNKNOWN_VERB`,
   `S_DUPLICATE_BINDER`, `S_UNBOUND_HEAD` (todo parámetro de la cabeza obtiene
   valor por matching del cuerpo o por constante).
3. **Conexión** (`S_DISCONNECTED`): sólo si el manifiesto del episodio activa el
   perfil conectado; se registra como restricción de búsqueda, no como
   relevancia jurídica.
4. **Elaboración real**: `load_formal_module` del módulo privado. Fallo →
   `S_ELABORATION` con el diagnóstico del elaborador. Compilar no basta para
   admitir; admitir no garantiza completar la evaluación.
5. **Instalación**: `S_INSTALL` si el plan no es uno de los permitidos;
   `S_ORIGIN` si alguna declaración u origen de `T` cambia.

Un rechazo nunca se puede sortear con texto en lenguaje natural: sólo entra a
evaluación lo que pasa los cinco pasos.

## 3. Contratos de herramientas y registro

Todas las herramientas son adaptadores sobre las APIs de 1.x. Ninguna
implementa matching, sustitución, cierre, búsqueda o certificación propios.
Los estados de ejecución son comunes:

| Estado | Significado | Nunca significa |
|---|---|---|
| `complete` | La sesión alcanzó su punto fijo o la búsqueda terminó y fue certificada | Completitud residual general |
| `step_limit` | Se agotó `max_steps`/`max_rounds` de la herramienta; hay frontera `pending` | Refutación ni éxito |
| `search_limit` | `SearchLimits` agotados (`Err` del motor); la sesión queda fallida | Refutación |
| `cyclic` | `Err` por ciclo en la búsqueda residual | Negación de derivabilidad |
| `unresolved_parameters` | `Err` por parámetros no determinados por objetivo ni bindings | Enumeración existencial |
| `binding_error` | `resolve`/`preflight` rechazan los bindings | — |
| `structural_reject` | Códigos del apartado 2.6 | Incompatibilidad normativa |
| `runner_limit` | Timeout o memoria del runner | Etiqueta jurídica |
| `error` | Elaboración u otra `Err` | — |

### 3.1 Herramientas

**`inspect_theory(theory_ref)`** — todas las condiciones.
Entrada: `T` o `candidate_id`. Salida: verbos con firmas y canal, constantes
tipadas, nombres de reglas y Programs con sus orígenes, hashes de archivos.
Llama a `load_formal_module` y recorre `Module.declarations`/`origins`. Sin
estado.

**`forward_observe(dossier_id, theory_ref, max_rounds)`** — todas.
Construye `ForwardSession::for_module(seeds, module)` y llama a
`trace(max_rounds, clock)`. Salida A: `known`, `added` por ronda, `round`,
`complete`, `horn_goal` para cada evento de `Q`. Salida B añade `witnesses`
y `derivations` (regla, antecedentes ordenados, sustitución, origen del
productor vía `Module::origin`/`program_rule_origin`). Estado `complete` o
`step_limit`. Límite de rondas `max_rounds ≤ 4096`, igual que la CLI. Advertencia
registrada: sin la comprobación privada de aciclicidad del runner, un cierre
infinito sólo lo detiene el presupuesto de rondas o el runner.

**`evaluate_candidate(candidate_id, dossier_ids)`** — todas.
Para cada expediente: `close_program(seeds, T ⊕ Γ)` y cierre de la base `T`
reproducido independientemente; proyección fuera de la cabeza nueva y
comparación de eventos completos y `proof_identity` con la base (camino
`Study::certify`); cálculo de `D` sobre el cierre y de `κ_ε` sobre las
valoraciones resueltas. Salida A: resultado de `D` por expediente, estado de
conservación de la referencia, estado de ejecución. B añade derivaciones de la
cabeza candidata. C añade el vector de cobertura y las actualizaciones de
partición. Termina en `complete`, `runner_limit` o `error`; **no se ejecuta si
`D` no está fijado para el episodio**.

**`residual_query(dossier_id, theory_ref, goal, scope, bindings, limits, allowed, max_steps)`** — todas.
Llama a `begin_residual_query_with_bindings` y a `ResidualSession::trace`.
`scope` obligatorio (`using rules` o `using program`); el alcance explícito no
incorpora dependencias. Salida A: `required` (fórmula), `failure`, `metrics`,
estado, y en `step_limit` la lista `pending`. B añade `explanations` con
`ResidualProof`, sustituciones y orígenes. Los parámetros no determinados
producen `unresolved_parameters`; el LLM debe aportar bindings explícitos, no
pedir enumeración.

**`active_query(...)`** — mismos parámetros, con `begin_active_residual_query_in_scope_with_bindings`
y `ActiveResidualSession::trace`. Al completar, `certify` ejecuta un
`close_program` **completo por rama** sobre el módulo original: la herramienta
registra ese coste como transiciones del motor del episodio. Sin comprobación
pública de aciclicidad: la protección es el runner.

**`propose_candidate(formulation, installation_plan, parent_id, move, rationale)`** — todas.
Ejecuta 2.6. No evalúa. Salida: `admitted` con `candidate_id`, índice canónico
y hashes, o `structural_reject` con códigos. Marca si el índice canónico
coincide con un candidato anterior o con una regla del catálogo Riggs.

**`request_valuation(case_spec, expected_discrimination)`** — todas.
Registra una solicitud al evaluador externo. Devuelve `pending`. Nunca
responde el harness ni el LLM. Un caso propuesto debe ser tipado y realizable
con seeds admisibles; si requiere una premisa Derived se marca `derived_premise`.
Las valoraciones de reserva no se exponen durante la búsqueda.

**`minimal_supports(dossier_id, theory_ref, goal)`** — B y C.
Soportes mínimos Seeded del objetivo sobre el dominio acotado declarado.
Reutiliza la lógica de `finite_frontiers`/`insert_minimal` o de `grounding`/`minima`
tras generalizar el tamaño del universo. Devuelve anticadena de soportes con
eventos completos. Sólo con `complete`.

**`compare_coverage(candidate_a, candidate_b, domain_id)`** — C.
Inclusión de vectores de predicciones (`predictions`) sobre un dominio finito
declarado. Devuelve `subset`, `superset`, `equal`, `incomparable` **relativo a
ese dominio**; nunca subsunción general.

**`partition(candidate_ids, observation_ids)`** — C.
`kernel` y `meet` sobre un conjunto fijo de candidatos. Devuelve bloques por
canal (`events`, `provenance`, `assessment`) y estado de cada observación.

**`discriminants(candidate_ids, domain_id)`** — C.
Scores por caso como `scores`/`best`: tamaño de la mayor rama por caso. Casos
candidatos a `request_valuation`.

### 3.2 Hechos del contrato que el explorer recibe por escrito

- Una invocación CLI nueva no reanuda una sesión: sólo el harness Rust
  mantiene `ForwardSession`, `ResidualSession` y `ActiveResidualSession` en
  memoria entre acciones.
- Una certificación activa ejecuta `close_program` completo por rama.
- `step_limit`, `search_limit` y `runner_limit` producen resultados pendientes,
  no refutaciones.
- Las variables no resueltas no se enumeran; se piden bindings.
- `using rules {...}` y `using program P` no incorporan dependencias.
- Residual y activa rechazan defaults.
- Para consultas Horn de fuente, el cierre que muestra la CLI usa sólo las
  reglas del scope; para observar `C_T(S)` completo se usa `forward_observe`.

### 3.3 Registro por acción

Cada acción escribe un registro con estos bloques, en archivos distintos:

| Bloque | Campos | Archivo |
|---|---|---|
| Identidad | `episode`, `condition`, `action_index`, `history_hash` | `actions.jsonl` |
| Propuesta y razón declarada | `tool`, `typed_inputs`, `rationale` (texto del LLM), `expected_discrimination` | `actions.jsonl` |
| Entradas | `dossier_ids`, `bindings` normalizados, `scope`, `goal`, `limits`, `allowed` | `actions.jsonl` |
| Candidato, teoría y versiones | `candidate_id`, `theory_ref`, hashes de módulos, versión de `D`, versión del manifiesto | `actions.jsonl` |
| Estado de ejecución | estado de la tabla de 3.0, `metrics`, rondas/pasos, tiempo de runner, pico de memoria | `evaluator-trace.jsonl` |
| Resultados y certificados | eventos, derivaciones, pruebas, conservación de la referencia, referencia al archivo del certificado | `evaluator-trace.jsonl` |
| Progreso | `inferential`, `geometric`, `normative`, cada uno con `claim`, `scope`, `certificate_ref` o `none` | `progress.jsonl` |

La razón declarada del LLM (`rationale`) y la traza del evaluador son
registros distintos; la primera no se interpreta como transcripción del
razonamiento interno y la segunda no contiene texto del LLM. La provenance
jurídica (derivaciones) y la provenance de las valoraciones (`source`, `grounds`,
`scope`, `status`, como en `riggs_rule_observations.tsv`) se guardan en
`provenance.jsonl` y `valuations.jsonl`.

Registro por candidato (`candidates.jsonl`): fuente original y AST, índice
canónico, `generation_source` (herramienta y acción), códigos estructurales,
plan de instalación, `parent_id` y `move`, historial de valoraciones resuelto y
no resuelto, resultados geométricos acotados con su dominio.

### 3.4 Generación, filtrado y refinamiento

Tres eventos distintos del archivo:

| Evento | Efecto | Registro |
|---|---|---|
| `candidate.generated` | `B_{n+1} = B_n ∪ {Γ}`; cambia el carrier; `Γ` entra en `U` (no resuelto) hasta evaluarse contra **toda** la historia de observaciones | `archive.jsonl` |
| `observation.resolved` → `filter.applied` | `V_{n+1} = {Γ ∈ B_{n+1} : pasa todas las observaciones resueltas}`; las pendientes no eliminan | `archive.jsonl` |
| `partition.refined` | `π_{n+1} = π_n ∧ ker(o_new)` **sobre un conjunto fijo declarado** de candidatos ya evaluados en el mismo dominio | `partitions.jsonl` |

Agregar candidatos no es un meet: la partición anterior no está definida sobre
el nuevo elemento hasta extender las observaciones. El registro conserva
`carrier_ids` de cada partición.

### 3.5 Sesiones stepwise: qué se reutiliza y qué no

Las sesiones de `stepwise-evaluation.md` son la base de las herramientas de
3.1. Lo que cada una aporta al explorer y su límite:

| Sesión | Herramienta | Aporta | Límite |
|---|---|---|---|
| `ForwardSession` | `forward_observe` | `T(X)` ronda a ronda sobre instantánea congelada; prefijos acotados de cierres infinitos; testigos con pruebas alternativas | Síncrona: orden y `derivedAt` difieren del cierre completo; sólo los eventos lógicos finales se comparan |
| `ResidualSession` con bindings | `residual_query` | Frontera `pending` explícita por capa de reglas; alternativas retenidas en `ResidualSearchTree`; resultado sólo tras certificación por replay forward | Agotar capas o `SearchLimits` no refuta; parámetros no resueltos son `Err` |
| `ActiveResidualSession` | `active_query` | Productores guardados y metadatos de activación; certificación contra `close_program` original | Cada rama completada ejecuta un cierre completo; requiere entradas forward terminantes |
| `IncrementalClosure::begin_stepwise_update` | `evaluate_candidate` (opcional) | Derivada `Φ(D)` de una adición sobre base cerrada `B`: exactamente qué añade una candidata | Usa matching de instantánea completa; no es el incremental semi-naive |
| `FourSession` | ninguna | — | Ground, ≤ 4 átomos, un tiempo compartido; designado `{T,B}`; ver 5.3 |
| `DefaultSession` | ninguna en `P1-add` | Fases `M1` → selección → `M2`, con aplicables/bloqueados/derrotados | Rechazada por residual, activa y `close_program`; reservada para una futura `⊕` por excepción |

Dos propiedades del trabajo stepwise que el contrato usa directamente: las
sesiones retienen estado en memoria entre llamadas, por lo que el harness
debe ser un proceso Rust y no una secuencia de invocaciones de la CLI; y el
estado `step_limit` distingue formalmente límite de completitud, lo que da
soporte a la regla "los límites producen pendientes, no refutaciones".

## 4. Diseño verificable de las vistas A/B/C

### 4.1 Proyecciones

El harness guarda siempre el registro completo. El explorer recibe
`view_X(record)`:

| Campo | A | B | C |
|---|---|---|---|
| Firmas, constantes, nombres de reglas y orígenes de `T` | sí | sí | sí |
| Resultados de `D`, `horn_goal`, eventos conocidos, estado, `pending`, `metrics`, errores tipados | sí | sí | sí |
| `required` como fórmula | sí | sí | sí |
| `witnesses`, `derivations`, `ResidualProof`, sustituciones, productores y orígenes | omitido | sí | sí |
| `minimal_supports` | no disponible | sí | sí |
| `compare_coverage`, `partition`, `discriminants` | no disponible | no disponible | sí |
| Conservación de la referencia (booleano) | sí | sí | sí |
| Detalle de la comparación de pruebas | omitido | sí | sí |

Omitido significa que el campo no se serializa en la respuesta de A; la
verificación es un test del proyector que compara el JSON entregado con un
esquema sin campos de prueba. Lo que A puede reconstruir con sus propias
consultas no se considera filtración: la ontología es común.

B ya contiene geometría (soportes mínimos). C/B mide el valor de orden,
particiones y casos discriminantes sobre esa base; C/A no aísla una operación.

### 4.2 Igualdad entre condiciones

Fijos por episodio e iguales en A, B y C: modelo y versión, parámetros de
generación, `T` y hashes, manifiesto `P1` y de problema, `D` y su versión,
fuente de valoraciones y protocolo de respuesta, presupuesto de oportunidad.
Los episodios se ejecutan por separado; transcripciones e hipótesis no se
comparten. El evaluador externo recibe las solicitudes con el `episode_id`
anonimizado respecto de la condición y responde con la misma tabla.

### 4.3 Presupuestos

| Presupuesto común de oportunidad | Coste real medido |
|---|---|
| Acciones máximas por episodio | Tokens de entrada/salida |
| Solicitudes de valoración máximas | Transiciones del motor: rondas forward, pasos residuales, cierres completos (incluidos los de `certify` y los de dominio en C) |
| Tamaño máximo de candidato (`P1`) | Tiempo y memoria del runner |
| `max_rounds`, `max_steps`, `SearchLimits` iguales | Trabajo adicional de B/C para calcular soportes, cobertura y particiones, declarado aparte |

Igual número de consultas no implica igual coste; ambas columnas se informan.

### 4.4 Métricas separadas

| Métrica | Definición | Fuente |
|---|---|---|
| Admisibilidad | propuestas, admitidas, rechazadas por código; propuestas que repiten un índice canónico | `candidates.jsonl` |
| Eficiencia exploratoria | acciones, tokens y transiciones hasta el primer éxito verificado de forma independiente; acciones sin progreso | `actions.jsonl`, `progress.jsonl` |
| Discriminación | candidatos supervivientes distintos, clases de cobertura distintas en el dominio acotado, bloques de partición, solicitudes informativas (mayor rama < total) | `partitions.jsonl` |
| Compatibilidad normativa | fracción de observaciones resueltas satisfechas bajo `D` por candidato; pendientes contadas aparte | `valuations.jsonl` |
| Casos valorados fuera del diseño | resultado de `D` en expedientes con valoración independiente obtenida fuera del episodio y no accesible durante la búsqueda | reserva externa |

Ninguna valoración producida por el LLM o por el harness cuenta para la
métrica normativa. Una valoración hipotética propuesta por el explorer se
registra como `hypothetical` y sólo sirve para seleccionar casos a solicitar.
Se informan episodios fallidos y censurados; variantes sintácticas y clases de
cobertura se cuentan por separado.

## 5. Estado de `D` y decisiones que requieren deliberación

### 5.1 Familia de `D` compatible con el código

| `D` | Definición | Implementación | Estado |
|---|---|---|---|
| `D_mem` | `N = {q ∈ Q_ground : q ∈ C_T(S).known}` | `closure.known.contains(goal)` (`experiments.rs`, `horn_goal`); `Study::goal` y `predictions` | IMPLEMENTADA como proyección; usada por todos los experimentos previos |
| `D_prov` | `D_mem` más multiconjunto de `DerivationRecord` de cada `q` | filtro sobre `Closure.derivations` | IMPLEMENTABLE sin lógica nueva; uso de provenance debe declararse |
| `D_struct` | tupla de resultados `D_mem` para varias cabezas de `Q` | varias llamadas a `D_mem` | IMPLEMENTABLE |
| `D_state` | resultado tras ejecutar efectos sobre `State` | `runtime::execute_effect` | FUERA del contrato inicial |

`κ_ε` compara `D(...)` con `y*`. Para `D_mem` con una cabeza:
`κ(z) = [z = y*]`. Para `D_struct` hay que declarar qué tuplas satisfacen `y*`.

### 5.2 Riggs: tres resultados distintos

Con `S_hist` el expediente histórico, `T` la referencia y `T ⊕ r_M` las
teorías aditivas del catálogo:

| Resultado | Fórmula | Valor en `T` | Valor en `T ⊕ r_M`, 16 supervivientes | Certificado |
|---|---|---|---|---|
| Atribución literal | `testamentaryEntitlement(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime ∈ C(S_hist)` | derivado | derivado (monotonía) | `case-consequences.tsv` |
| Impedimento propuesto | `hypotheticalSuccessionImpediment(... mismos argumentos ...) ∈ C(S_hist)` | no declarado en `T`; pasivo en la referencia geométrica | derivado | `versions.tsv`, 384 comparaciones |
| Exclusión efectiva | `y*` = Elmer no adquiere el interés (`riggs_rule_observations.tsv`, pp. 514-515) | sin canal ejecutable | sin canal ejecutable | ninguno |

**DEMOSTRADO (contrato previo §5.5)**: si `⊕` es adición, ningún candidato
hace que la atribución deje de derivarse. Por tanto un `D` que exija
`attr = 0` es insatisfacible en `P1-add`; eso es un límite del perfil, no un
dato jurídico. Bajo la decisión adoptada en 5.3 ese requisito no existe: la
atribución literal se conserva, justificada, y la candidata añade la
consecuencia que prevalece.

### 5.3 Decisión adoptada para Riggs

Había que fijar, antes del primer episodio Riggs y sin cambiar `y*`, **cómo se
compara el resultado ejecutable con la valoración histórica**. Las
alternativas compatibles con el código eran las siguientes; la elegida es
`K2` con `κ(1,1) = 1`, estipulada por el autor del proyecto el 2026-10-14:

| Opción | `D` y `κ` | Qué puede certificar | Qué requiere |
|---|---|---|---|
| `K1` Condición necesaria | `D_mem` sobre el impedimento; `κ = [imp = 1]` | Compatibilidad con una condición necesaria del fallo; el experimento previo ya lo hizo | Declarar en el manifiesto que el éxito normativo es "condición necesaria", nunca "reproducción del remedio" |
| `K2` Resultado estructurado | `D_struct = (attr, imp)`; `κ` declara qué pares son compatibles con `y*` | Si `κ(1,1) = 1`, equivale a `K1` con registro explícito de la atribución; si `κ(1,1) = 0`, ningún candidato aditivo lo cumple y el episodio sólo puede registrar ese límite | Deliberación sobre `κ(1,1)` |
| `K3` Canal de exclusión declarado | Declarar en `T` o en el manifiesto un verbo Derived "atribución efectiva" cuya derivación dependa de que no haya impedimento | No expresable en Horn monotónico sin negación por ausencia; `~` es evidencial, no por fallo | Nueva semántica: **fuera de este contrato** |
| `K4` Operación distinta de adición | `⊕` = sustitución controlada de `literalTestamentaryEntitlement` en copia privada, o excepción entre defaults | Podría hacer `attr = 0`; el antiguo harness de compliance sustituía productores en copias privadas | Declarar y certificar una nueva operación `⊕`; defaults no son compatibles con residual ni con `close_program` |

**Estipulación `D_Riggs`, versión 1.**

- Objetivo `Q = hypotheticalSuccessionImpediment(...)` con los argumentos del
  expediente histórico. El modelo de recalcitrancia define la experiencia
  relativa a un objetivo y declara que cambiar de objetivo cambia la
  descripción, no los hechos: Riggs se trata como subinclusión respecto del
  impedimento, no como sobreinclusión respecto de la atribución.
- `D = D_struct = (attr, imp)` con `D_mem` en cada componente.
- `κ(attr, imp) = [imp = 1]`. El par `(1,1)` es compatible: la atribución
  literal coexiste, derivada y justificada, con la consecuencia que prevalece.
  La prevalencia es una lectura externa de `κ`, no un cálculo del motor; es un
  trump, no una eliminación, y la adición Horn es la operación correcta para
  expresarlo.
- `attr` se registra siempre en el resultado por separado y nunca cuenta en
  contra. La exclusión efectiva queda **fuera del experimento**.

Consecuencias que deben constar porque `κ` es laxo:

1. El impedimento es una cabeza terminal que ninguna regla de `T` consume.
   Todo lo que depende de la atribución (activo, titularidad, capacidad)
   sigue derivándose para Elmer. Una candidata que deriva el impedimento y
   además sobregenera consecuencias aguas abajo puntúa igual bajo `κ` que una
   que no. Ninguna valoración del dataset puede formularse como "Elmer no
   adquiere el activo": el motor no podría satisfacerla.
2. `κ` tampoco penaliza la anchura: una candidata que derive el impedimento
   en casi cualquier expediente pasa. La laxitud de `D` se compensa fuera de
   `D`: con la región de cada candidata sobre el universo finito y con
   valoraciones pedidas donde las supervivientes discrepan (apartados 4.4 y
   6.1). Por eso la geometría es imprescindible y no opcional bajo esta
   decisión.
3. Si en el futuro se exige que la exclusión se propague, la primitiva
   existente es la excepción entre defaults (`DefaultBody::Exception`,
   `src/defaults.rs`), que derrota sin borrar y registra evidencia y
   sustitución. Queda aplazada: cambia el evaluador que certifica, y el código
   marca como provisionales la consistencia `M1`/`M2` (`NotChecked`) y el
   bloqueo por lambda completa.
4. FOUR no sustituye nada de lo anterior. `experiments/four.rs` es un
   evaluador de referencia sobre a lo sumo cuatro átomos ground con reglas sin
   parámetros, conjunto designado `{T, B}`: un átomo con soporte a favor y en
   contra sigue contando como entailed. Registra la contradicción, no decide
   la prevalencia, y no acepta la teoría de Riggs.

Con `D_Riggs` fijado, la evaluación normativa de Riggs queda autorizada en
cuanto la admisibilidad esté instanciada (apartado 7). Para un problema no
Riggs con `y*` expresado como pertenencia de un evento de `Q`, `D_mem` con
`κ = [z = y*]` está completamente determinado y puede usarse.

`D` queda fijo durante cada episodio e idéntico en A/B/C; el explorer no puede
redefinirlo ni usar provenance como prioridad no declarada.

## 6. Lista mínima de implementación posterior

Ordenada por dependencia. Ninguna partida toca el motor, `State`, `close`,
`Rule`, `Program` ni la API residual.

| # | Partida | Reutiliza | Lógica nueva estrictamente necesaria |
|---|---|---|---|
| 1 | Manifiesto de episodio (JSON): `T`, hashes, `P1`, manifiesto de problema, `Q`, `D` y versión, dominio `A`, presupuestos, condición | `load_formal_module`, `Module::origins`, exportación del inventario | Serialización y hashing. |
| 2 | Validador estructural parametrizado | `classify`, `term_check`, `rule`, `atoms` del harness estructural | **IMPLEMENTADO** en `exploration::{Profile, classify, classify_candidate}`: `Profile::p1()`, `with_problem`, `legacy_compliance`; códigos nuevos `B_UNIT_NODES`, `B_AND_NODES`, `B_CLAUSES`, `S_SYNTAX`. Pendiente: `B_RULE_USES` (`P1-program`) y el camino de fuente `parse_formal` → `S_SYNTAX`/`S_ELABORATION` para propuestas en texto. |
| 3 | Instalador `P1-add` | Patrón `riggs_candidate_rules.res` + `Study::theory`/`with_passive_head` | **IMPLEMENTADO** en `exploration::install` sobre `Rule` tipadas, con `S_INSTALL` y orígenes por candidato. Pendiente: generación del módulo privado desde fuente y `P1-program`. |
| 4 | Evaluador de candidato y certificado de conservación | `close_program`, `audit_proofs`, `proof_identity`, `full_events`, `Study::certify` | **IMPLEMENTADO** en `exploration::{Decision, Criterion, evaluate, certify_addition}` (`D_struct`, `κ` por componente o exacto; conservación de eventos y multiconjunto de justificaciones fuera de las cabezas declaradas). |
| 5 | Adaptadores de sesión (`forward_observe`, `residual_query`, `active_query`) | `ForwardSession`, `begin_residual_query_with_bindings`, `begin_active_residual_query_in_scope_with_bindings`, `trace` | Mapeo de `Err` a los estados de 3.0; serialización por vista. Sin aciclicidad pública: documentar y confiar en el runner. |
| 6 | Proyectores A/B/C y test de esquema | — | Filtrado de campos y test que verifica ausencia de campos de prueba en A. |
| 7 | Registro (`actions`, `evaluator-trace`, `progress`, `candidates`, `archive`, `partitions`, `valuations`, `provenance`) | Formato de las tablas TSV existentes de Riggs | Escritura append-only y verificación de hashes. |
| 8 | Geometría para B/C | `finite_frontiers`/`insert_minimal` (quitar la aserción de 73), `universe`, `kernel`/`meet`/`finer`, `scores`/`best`, `instantiate_from_known` (público) | **IMPLEMENTADO** en `exploration::{universe, joint_universe, seeded_indices, frontiers, region_pool, signature, in_region, covers, kernel, meet, finer, class_count, discriminants}`; equivalencia verificada (6.2). Pendiente: minimax `scores`/`best` para elegir entre discriminantes; los adaptadores de herramienta B/C que lo expongan. |
| 9 | Oráculo acotado de cobertura | `close_program` sobre el dominio `A` declarado | Bucle y conteo. |
| 10 | Controlador de episodio mínimo y política LLM | Todo lo anterior | **No autorizado** por este documento; sólo tras revisar 1-9 y resolver el apartado 5. |

Los helpers del harness viven en el binario de tests de la CLI: reutilizarlos
desde un proceso de harness exige moverlos a un módulo compilado fuera de
`#[cfg(test)]` o duplicarlos con sus tests de equivalencia. El apartado 6.1
fija esa ubicación.

### 6.1 Diseño de la partida 8: geometría parametrizada

Diagnóstico sobre el código en `7af8d39`:

| Helper | Ubicación | Qué lo ata | Cambio |
|---|---|---|---|
| `finite_frontiers(m, e, allowed)` | `legal_geometry_tests.rs:748` | `assert_eq!(e.len(), 73)` | Quitar la aserción; el algoritmo es genérico y usa el público `derive::instantiate_from_known` |
| `universe(m)` | `legal_geometry_tests.rs:93` | Toma `Σ` de los experimentos Seeds del módulo | Recibir `Σ` y el módulo como parámetros |
| `kernel`, `meet`, `finer` | `legal_geometry_conversation_tests.rs:389-422` | Nada; operan sobre `Vec<usize>` | Mover |
| `Partition([u8; N])`, `N = 9` | `legal_geometry_partition_tests.rs:21` | Tamaño en compilación | No se reutiliza; la versión sobre vectores la sustituye |
| `classify(m, r, connected)` | `legal_geometry_structural_generation_tests.rs:255` | Cabeza `compliance`, `ALPHABET` de 12, `MAX_ATOMS`, 16 binders | Recibir un `Profile` |

Lo único nuevo es cómo comparar candidatas sin enumerar estados. Riggs
comparó sobre `2^13` estados; en `P1` la potencia del universo es inviable.

**Separación por fronteras. DEMOSTRADO para cierres monótonos.** Sea `P` la
unión de los soportes mínimos de todas las candidatas para `q`. Si
`R_θ1 ⊄ R_θ2`, existe `B` en la frontera de `θ1` con `B ∉ R_θ2`: si todos los
elementos de la frontera de `θ1` estuvieran en `R_θ2`, por monotonía
`R_θ1 = ∪ ↑B ⊆ R_θ2`. Ese `B` pertenece a `P`. Luego dos candidatas tienen la
misma región si y sólo si coinciden sobre `P`. La partición de hipótesis se
calcula evaluando cada candidata sólo sobre `P`, y el meet con la partición
de observaciones se calcula sobre `P` más los expedientes valorados. Vale para
cualquier cierre monótono, que es lo que el motor garantiza; no requiere
aciclicidad.

Interfaz propuesta (sólo análisis; nunca toca `derive`, `residual`, `typed`
ni `defaults`):

```
pub struct Profile { heads, body_vocabulary, max_atoms, max_binders,
                     max_i_nodes, max_depth, allow_evidential_not_on }
pub fn classify(m, rule, profile) -> BTreeSet<Reason>           // B_*/S_*/P_*
pub fn universe(theory_plus_all_candidates, sigma) -> Vec<Event> // cierre conjunto
pub fn frontiers(m, universe, allowed) -> Vec<Supports>          // el actual, sin assert
pub fn region_pool(frontiers_by_candidate, goal) -> Vec<State>   // P
pub fn signature(candidate, pool, goal) -> Vec<bool>
pub fn kernel / meet / finer                                     // los actuales
pub fn discriminants(partition, pool) -> Vec<State>              // donde discrepan
```

Decisiones fijadas:

- **Universo conjunto.** `E = close(T ⊕ todas las candidatas admitidas, Σ)`.
  Así `E` contiene las cabezas nuevas y está cerrado bajo cada candidata por
  separado; las fronteras de `θ` se calculan sobre `E` y los eventos que `θ` no
  alcanza quedan sin soporte. Exige aciclicidad de la unión, comprobada con el
  helper Horn existente (`require_acyclic`). Si falla, se degrada a universo
  por candidata y comparación por conjuntos de eventos en vez de índices.
- **`allowed` como parámetro.** Las hojas admisibles como evidencia: en `P1`,
  los eventos de verbos Seeded de `E`. Distingue "soporte mínimo" de
  "cualquier evento ya en `K`".
- **Ubicación.** Módulo de biblioteca junto a los evaluadores de referencia de
  `experiments`, que ya tienen el mismo lema: capa de consulta que nunca
  aumenta el cierre. No se tocan motor, ontología ni fixtures. Los tests
  antiguos no se modifican. Se añaden tests individuales de equivalencia que
  comprueban que el módulo nuevo reproduce, sobre el universo de 73 y sobre
  los 13 puntos de Riggs, las fronteras y las clases ya certificadas. Es
  extracción con prueba de equivalencia, no maquinaria paralela; la delegación
  de los tests antiguos queda para una autorización posterior.
- **Coste.** Un punto fijo de fronteras por candidata, sin enumeración
  exponencial. La única variable es el tamaño de las anticadenas, pequeño con
  cuatro átomos por cuerpo.
- **Métricas que habilita (4.4).** Generalidad: inclusión de regiones vía
  fronteras (`θ1 ⊒ θ2` si todo elemento de la frontera de `θ2` contiene uno de
  la de `θ1`). Discriminación: número de clases de la partición de hipótesis
  que separa cada valoración. Ambas se calculan fuera del LLM.

### 6.2 Estado de implementación y verificación

Implementado en `src/experiments/exploration.rs` (biblioteca, hermano de los
evaluadores de referencia; no toca `derive`, `residual`, `typed`, `defaults`,
ontología ni fixtures; `experiments.rs` sólo registra el módulo y sus tests).
Tests en `src/experiments/exploration_tests.rs`, ejecutados uno a uno con
`scripts/test_safe.py --test-target lib` bajo 1 GiB, sin swap, un trabajo y un
hilo, con `CARGO_PROFILE_TEST_DEBUG=0` como en los informes previos. Registro
completo en [`docs/legal-geometry/exploration/verification.md`](legal-geometry/exploration/verification.md).

| Test | Oráculo reproducido | Informe |
|---|---|---|
| `exploration_frontiers_reproduce_certified_general_and_patrimonial_totals` | `goal-frontier-results.json`: universo 73, 23 hojas Seeded; general 73/284 y 61/99; patrimonial 73/332 y 73/114; restricción de fronteras completas a hojas Seeded igual a la frontera externa | `20261014T233835607992Z-1881332` |
| `exploration_riggs_catalog_partition_and_discriminants_match_certificates` | `partitions.tsv`: 16 de 64 retenidas por la valoración histórica, núcleo idéntico; `questions.tsv`: predicciones por caso de las 16 retenidas calculadas por frontera y confirmadas por `close_program`; certificado de conservación en las 64 adiciones; firmas sobre el pool confirmadas por cierre real en cada estado | `20261014T233836418502Z-1881365` |
| `exploration_profile_admits_riggs_catalog_and_reports_bounded_rejections` | 64 reglas del catálogo admitidas con 7 átomos; exactamente 42 admitidas con la cota `P1` de 4 átomos (`C(6,≤3)`), tanto con el perfil directo como con `Profile::p1().with_problem(...)`; códigos `P_HEAD_INTERFACE`, `S_UNBOUND_HEAD`, `B_UNIT_NODES`+`B_AND_NODES`, `P_BODY_VOCABULARY`, `P_POLARITY`, `B_CLAUSES` | `20261014T233834601180Z-1881315` |
| `exploration_install_rejects_collisions_and_non_derived_heads` | `S_INSTALL` por nombre duplicado, verbo no Derived y colisión con `T`; idempotencia del verbo; declaraciones y orígenes de `T` intactos | `20261014T233833387538Z-1881182` |

Diagnóstico del pool de separación en Riggs (`riggs-separation-pool.tsv`,
informe, no oráculo): universo conjunto de 16 eventos con 9 hojas Seeded; pool
de 6 estados; la partición de las 16 retenidas sobre el pool tiene 6 clases
frente a 4 con los seis expedientes declarados, y 5 estados del pool son
discriminantes. Es decir, los seis expedientes de `riggs_cases.res` no agotan
las distinciones que las propias candidatas inducen; el pool las localiza sin
enumerar los 8.192 estados.

Controles añadidos tras revisión, para que un error de herramienta nunca se
convierta en un veredicto sobre una hipótesis (informes del 2026-10-15 en el
registro de verificación):

| Riesgo señalado | Control | Comportamiento |
|---|---|---|
| Componente de `D` inexistente o `y*` de longitud distinta leído como `false` | `Decision::validate`, llamado por `evaluate` | `Err("D: component … does not exist")`; nunca `compatible = false` |
| Certificado de conservación aplicado a una cabeza que `T` ya produce (p. ej. `obligation`), ocultando justo lo que debe examinar | `certify_addition` devuelve `Err` si una cabeza declarada ya aparece en el cierre de referencia | El certificado sólo se emite en su alcance probado: cabezas nuevas y terminales. Una cabeza intermedia exige su propio certificado (límite 3) |
| Módulos de origen no comprobados | `theory_preserved(theory, installed)` | Cada declaración y origen de `T` idéntico; toda adición con origen fuera de los módulos de `T`; si no, `Err("S_ORIGIN …")` |
| `meet`/`finer` truncando por `zip` sobre listas de distinta longitud | Comprobación de carrier; ambas devuelven `Result` | `Err("partition carriers differ")` |
| Particiones sobre hipótesis distintas o en distinto orden | `Partition { hypotheses, kernel }` con `new`, `meet`, `finer`, `classes` | Identificadores obligatorios y sin duplicados; combinar particiones con identificadores distintos es `Err` |

Lo que esto **no** cambia: el par `(attr = 1, imp = 1)` sigue certificando
"esta candidata propone el impedimento", no "Elmer queda excluido" (5.3). Es
una diferencia de objetivo certificado, no un fallo de cálculo.

### 6.3 Camino de fuente para propuestas en texto

Implementado en `src/experiments/proposal.rs` (`SourceProposal`,
`ProposalContext`, `process`, `ProposalStatus`, `ProposalReport`), con la
fixture nueva `examples/experiments/legal_geometry/riggs_problem_manifest.res`,
que sólo importa la referencia y declara la cabeza experimental; ninguna
fixture original se modificó. Etapas, en orden, cada una registrada en
`ProposalReport::stages`:

```text
texto Formal (sólo reglas)
  → parse_formal del módulo compuesto            S_SYNTAX
  → módulo privado Candidate<id>.res que importa la teoría
    y el manifiesto de problema; load_formal_module   S_ELABORATION
  → extracción por origen (módulo == Candidate<id>) S_DECLARATION si declara algo que no es regla
  → theory_preserved contra una línea base con las mismas importaciones   S_ORIGIN
  → classify_candidate con el Profile del episodio  structural_reject (códigos B_*/S_*/P_*)
  → cabezas del candidato frente al cierre de referencia   out_of_certificate_scope
  → evaluate + certify_addition                     evaluation_pending | evaluated_{compatible,incompatible}
```

Lo que distingue el resultado, como pidió la revisión: fuente mal formada;
referencias, tipos o bindings inválidos; rechazo del perfil; configuración
admitida pero fuera del alcance de certificación; evaluación pendiente
(el evaluador no completó: no es refutación); compatibilidad o
incompatibilidad con la valoración. Una mala configuración del harness
(identificador inválido, sin importaciones, teoría que no elabora, `D`
inválido) es un `Err` de `process`, nunca un estado sobre la propuesta. El
texto original se conserva en el informe en todos los casos y el archivo
generado queda en el directorio de trabajo del episodio.

Verificado por
`proposal_tests::proposal_source_path_distinguishes_syntax_elaboration_profile_scope_and_evaluation`
(informe `20261015T003143485814Z-1920146`): una regla con `purposefulAcquisition`
llega a `evaluated_compatible` con certificado y orígenes correctos al
recargar el archivo; una regla con `~intentionalKilling` llega a
`evaluated_incompatible` con certificado; `S_SYNTAX` sin etapas; `S_ELABORATION`
por verbo desconocido y por aridad; `structural_reject` por `B_ATOMS` y por
`S_DECLARATION`; `out_of_certificate_scope` para una cabeza
`testamentaryEntitlement`; y tres errores de harness. `evaluation_pending`
está definido pero no se provocó en el test: con dos cláusulas y el perfil
`P1`, los ciclos de activación se rechazan antes como `B_CLAUSES`, así que
sólo lo alcanzarían límites del runner o cierres no terminantes.

### 6.4 Incremento 1 del protocolo de invocación: etapas compartidas

Tras la revisión del [protocolo de invocación](llm-explorer-invocation-protocol.md)
(apartado 5.1), `proposal.rs` se reorganizó en etapas reutilizables sin
cambiar el resultado de `process`:

| Función | Etapas | Cierra |
|---|---|---|
| `Episode::prepare(ctx)` | valida `D`, resuelve importaciones relativas, elabora la línea base `Baseline.res` | ninguno; `Err` de harness |
| `admit(proposal, ctx, episode)` | parseo; **frontera sobre el AST**: más importaciones que las del controlador → `S_IMPORT`, cualquier declaración que no sea regla → `S_DECLARATION`, ambas antes de escribir o resolver archivo alguno; elaboración; extracción por origen; `theory_preserved`; `classify_candidate` | ninguno |
| `Episode::reference(seeds)` | cierre de referencia de la línea base | uno, del verificador |
| `check_scope(admitted, reference)` | cabezas frente al cierre de referencia | ninguno |
| `evaluate_admitted(admitted, ctx, reference)` | `close_program` del módulo privado, `read_decision`, `certify_addition` | uno |
| `EvaluationSession::{begin, step, finish}` | `ForwardSession::for_module` sobre el módulo privado; cada `step` es una ronda síncrona; `finish` sólo lee `D` y certifica si la sesión alcanzó su punto fijo, y en otro caso devuelve `evaluation_pending` con las rondas consumidas | por rondas |

`read_decision` es la única lectura de `D` sobre un cierre y la comparten
`exploration::evaluate`, `evaluate_admitted` y la sesión. No hay segunda
definición de `D`. Una propuesta rechazada no deja módulo privado bajo su
identificador.

**Equivalencia verificada**: `forward_session_matches_close_program_on_riggs_candidates_and_geometry_fixtures`
compara, para 64 candidatas × 6 expedientes, la referencia en los 6 y los 24
experimentos sembrados de `general`, `patrimonial` y `reference`, el conjunto
de eventos completos, el multiconjunto de justificaciones (regla,
antecedentes ordenados, sustitución canónica) y los orígenes de las reglas,
normalizando únicamente `derivedAt`. 414 comparaciones, todas iguales
([`forward-equivalence.tsv`](legal-geometry/exploration/forward-equivalence.tsv)).
Si hubiera una discrepancia, el test la conservaría como contraejemplo en
`target/exploration/forward-equivalence-counterexamples.tsv` y fallaría; no
hay fallback que la presente como equivalencia. Esta equivalencia es finita y
vale para esas teorías y expedientes; el certificado sigue emitiéndose sólo
sobre una sesión completada.

Qué no está implementado: `P1-program` con `B_RULE_USES`, el certificado de
conservación para cabezas intermedias existentes, el minimax de preguntas,
y todo lo que el protocolo de invocación sitúa después del incremento 1
(trabajador, transporte en el runner, controlador, proveedor).
Los tests antiguos del harness no se modificaron ni se volvieron a ejecutar;
la equivalencia se comprueba contra sus certificados archivados.

## 7. Límites que impiden comenzar un episodio completo

1. **`D` para Riggs está fijado** (`D_Riggs` v1, apartado 5.3) pero es laxo
   por construcción: no penaliza sobregeneración aguas abajo ni en anchura.
   Hasta que la partida 8 calcule regiones y discriminantes, un episodio Riggs
   sólo puede informar compatibilidad con `κ`, no generalidad ni
   discriminación.
2. **La admisibilidad está instanciada** para candidatos tipados (6.2) y para
   propuestas en texto Formal (6.3). Lo que falta es el manifiesto de episodio
   serializado (partida 1) que fije `Profile`, importaciones, `D` y
   presupuestos como datos versionados en lugar de valores construidos en
   código.
3. **Instalación `P1-add` sólo certificada para una cláusula con cabeza
   nueva**; dos cláusulas con una cabeza intermedia existente (`obligation`,
   `permission`) pueden alimentar reglas de `T` y cambiar otras consecuencias.
   Requiere su propio certificado de conservación antes de usarse.
4. **Sin comprobación pública de terminación** para sesiones construidas por
   la API Rust; sólo el runner de fuente la aplica. Un candidato con `after`
   puede producir cierres infinitos: la protección es el presupuesto de rondas
   y los límites del runner.
5. **La geometría parametrizada existe y está verificada** (6.2), pero las
   herramientas `minimal_supports`, `compare_coverage`, `compare_partition` y
   `discriminants` del apartado 3.1 que la exponen al explorer, con sus
   proyecciones B/C, aún no están escritas. Los helpers antiguos del harness
   siguen privados y de tamaño fijo; no se tocaron.
6. **No existe fuente de valoraciones independientes para casos nuevos**. Las
   tablas actuales de Riggs contienen una observación histórica documentada y
   valoraciones estipuladas marcadas como hipotéticas; éstas no certifican el
   criterio normativo primario. Hay que fijar el protocolo de valoración y la
   reserva de casos fuera del diseño.
7. **Familiaridad del modelo con Riggs**: el catálogo de 64 reglas y los
   documentos están en el repositorio; Riggs sirve como problema experimental
   publicado, no como descubrimiento oculto.
8. **Sin controlador ni política implementados**; no hay métricas calculadas.

El experimento **no está listo**: faltan el manifiesto de episodio (partida
1), los adaptadores de sesión y de geometría con sus proyecciones A/B/C
(partidas 5, 6, 8 parcial), el registro append-only (7), el oráculo de
cobertura (9), el protocolo de valoraciones (límite 6) y el controlador (10).
Lo que sí está cerrado: `D` para Riggs, la admisibilidad sobre candidatos
tipados y sobre texto Formal, la instalación por adición con certificado de
conservación en su alcance probado, la geometría parametrizada y los
controles que impiden que un error de herramienta se lea como veredicto.
Queda delimitado: la matriz de 1.x fija
qué operaciones existen y por qué camino; el manifiesto `P1` y el de problema
fijan la gramática sin reutilizar el catálogo Riggs ni fijar `compliance` como
única cabeza; los contratos de herramientas y registros fijan estados y
separaciones; las vistas y métricas son verificables por proyección; `D` para
Riggs está estipulado y versionado con sus consecuencias explícitas; y la
parametrización geométrica tiene diseño, lema de separación y ubicación
fijados.

## 8. Procedencia de esta etapa

- Árbol de trabajo: `/tmp/residuum-llm-normative-exploration-spec`, rama
  `experiment-llm-normative-exploration-spec`, `HEAD = 7af8d39`.
- `git status` inicial: sólo
  `docs/llm-guided-normative-exploration-specification.md` sin seguimiento. Se
  conserva sin modificar.
- Archivos inspeccionados: los siete documentos indicados en el encargo,
  `legal-riggs-normative-rule-learning.md`, `src/lib.rs`, `src/typed.rs`,
  `src/programs.rs`, `src/derive.rs`, `src/derive/steps.rs`, `src/modules.rs`,
  `src/experiments.rs`, `src/residual/{bindings,steps,active,program,engine,query}.rs`,
  `src/bin/residuum/cli.rs`, `cli/steps.rs`, y los módulos de tests
  `legal_geometry_{tests,frontier,partition,conversation,structural_generation,riggs,riggs_rule_learning,extreme,active_frontier,explicit_query,query_scope}_tests.rs`;
  fixtures `general.res`, `riggs_succession_reference.res`, `riggs_cases.res`,
  `riggs_candidate_rules.res`, `riggs_rule_observations.tsv`,
  `riggs_rule_followup_valuations.tsv`; manifiesto
  `docs/legal-geometry/search-space/signatures.json`.
- No se compiló, no se ejecutaron tests ni fixtures, no se hizo commit, merge
  ni push. No se buscó literatura.
- Revisión del 2026-10-14: se inspeccionaron además `src/experiments/four.rs`,
  `src/defaults.rs`, `src/experiments/default_steps.rs`,
  `legal_geometry_partition_tests.rs`, `legal_geometry_recalcitrance_tests.rs`
  y los apartados 1.2, 1.3, 3.2 y 5.5 de
  `legal-recalcitrance-mathematical-model.md`. `D_Riggs` v1 procede de la
  estipulación del autor del proyecto en conversación; el diseño de 6.1 fue
  propuesto en esa conversación y aceptado.
- Implementación autorizada después: archivos nuevos
  `src/experiments/exploration.rs`, `src/experiments/exploration_tests.rs`,
  `docs/legal-geometry/exploration/{verification.md,riggs-separation-pool.tsv}`;
  `src/experiments.rs` modificado sólo para registrar el módulo y sus tests.
  Compilación y cuatro tests individuales con `scripts/test_safe.py`
  (informes en 6.2 y en el registro de verificación). Esa etapa se
  confirmó en la rama como `f3091c7`.
- Camino de fuente (6.3): archivos nuevos `src/experiments/proposal.rs`,
  `src/experiments/proposal_tests.rs`,
  `examples/experiments/legal_geometry/riggs_problem_manifest.res`;
  `src/experiments.rs` registra el módulo. Confirmado como `d17a194`.
- Incremento 1 (6.4): etapas separadas, frontera sobre el AST,
  `read_decision` compartida y `EvaluationSession`; tres tests individuales
  del camino de fuente y los cuatro de exploración vueltos a pasar (registro
  de verificación). No se ejecutó la suite ni ningún test antiguo; no se
  hizo merge ni push.
