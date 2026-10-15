# Transformaciones normativas en Residuum: caracterización ontológica

Estado: **investigación, no implementación.** No se modificó el motor, la
ontología ni ninguna fixture existente. Se añadieron sólo cuatro fixtures de
verificación acotada en `docs/legal-geometry/transformations/fixtures/` con
sus salidas en `.../runs/`. Rama `experiment-legal-order-geometry`,
commit base `4da4eb9`.

Marcas de estado, como en los documentos previos:

- **IMPLEMENTADO**: presente en código y con test.
- **DEMOSTRADO**: argumento matemático bajo hipótesis explícitas.
- **VERIFICADO-FINITO**: comprobado por el evaluador en un dominio finito declarado.
- **EXPRESABLE**: expresable por composición, sin procedimiento general.
- **NO REPRESENTABLE**: no representable con las construcciones actuales.
- **INDETERMINADO**: falta evidencia para decidir.

## 0. Resultado principal

1. **El circuito del Piloto 003 sólo podía añadir consecuencias.** Un
   explorador que propone reglas Horn evaluadas con `close_program` no puede
   cambiar un resultado jurídico efectivo que dependa de la ausencia de una
   consecuencia.
   - Ninguna adición Horn retira una conclusión.
   - Ninguna teoría Horn, con cualquier conjunto de reglas, ajusta una
     valoración antimonótona en la evidencia (Proposición 2).
   - Por eso el piloto necesitó una cabeza artificial. No es sólo que se le
     diera la conclusión: con ese lenguaje de transformación no existía otra
     forma de producir una diferencia observable.
2. **La ontología ya contiene un mecanismo no monótono con significado
   jurídico propio: los `default` con `blocked when` y `exception … to …`**
   (`src/defaults.rs`).
   - Con él se obtiene, sin verbo nuevo, la exclusión efectiva de Riggs sobre
     los seis expedientes. Se coloca la regla de referencia
     `literalEntitlementAsset` bajo un default bloqueado por la relación
     existente `purposefulAcquisition` (VERIFICADO-FINITO, §8.3).
   - La atribución testamentaria sigue derivada. Lo que desaparece es su
     producción patrimonial: `patrimonialAsset`, `owns` y `capacity`.
   - Eso coincide con la definición provisional de impedimento: se restringe
     la eficacia sin negar los antecedentes formales.
3. **Ese mecanismo tiene una insuficiencia expresiva exacta: bloquea la
   lambda completa dentro del expediente, no la instancia.**
   - Contraejemplo verificado: un testamento con dos beneficiarios, uno
     homicida y otro no. El bloqueo excluye también al inocente (§8.3).
   - La capacidad que falta es la **derrotabilidad relativa a la instancia
     ground**, es decir, el bloqueo de las instancias de un productor que
     comparten ligaduras con la configuración bloqueante.
   - No falta retractación ni prioridad general.
4. **El resultado efectivo no es una función del cierre Horn solo.** El
   modelo mínimo tiene cuatro capas, descritas en §4:
   - el cierre obligatorio M_T;
   - la selección de defaults Sel_T;
   - el cierre final F_T;
   - la ejecución sobre `State`, opcional y hoy sólo del harness.

   La decisión D sobre la cuestión Q queda fuera del motor. La descomposición
   (C_T, D_T) sólo es suficiente si no hay defaults ni ejecución de `State`.
5. **Con defaults, S ↦ F_T(S) deja de ser monótona y no es un operador de
   clausura** (contraejemplo verificado, §9).
   - La geometría de orden (estados cerrados, fronteras como generadores de
     conjuntos ascendentes, bases extremas, antiintercambio) no se aplica a
     F_T. Se aplica a M_T y a cada cierre de selección fija.
   - La geometría de identificabilidad (particiones de hipótesis por núcleos
     de observación) no requiere monotonía y se conserva.

## 1. Semántica normativa existente

Fuentes:
- `examples/experiments/legal_geometry/ontology_reference/legal_general.res` (LegalGeneral);
- `legal_patrimonial.res` (LegalPatrimonial);
- `examples/legal/modules/legal_monotonic.res` (LegalMonotonicCase, con las mismas declaraciones que LegalGeneral más `complianceAsset`);
- `examples/legal/modules/sales.res`, `sale_defaults.res` y `associated_program.res`;
- `examples/experiments/legal_geometry/riggs_succession_reference.res`.

### 1.1 Capas del lenguaje

| Capa | Construcción | Semántica operacional | Código |
|---|---|---|---|
| Evidencia | verbo `seeded`, polaridad `Positive` o `EvidentialNot` | Sólo entra como semilla. `~P` es otro evento: la identidad lógica incluye la polaridad. No hay negación por ausencia. | `typed.rs`: `Event::logical_key`, `Polarity`; `elab.rs:787-796` |
| Conclusión | verbo `derived` | Sólo la producen reglas; nunca se abduce (`Verb::is_abducible`). Puede tener polaridad `EvidentialNot` si la deriva una regla. | `typed.rs` `Verb`; `elab.rs:787-796` |
| Efecto | verbo `effect` con `StateTransform` (`+=`, `-=` sobre conjuntos) | Las reglas derivan un *evento* de efecto con metadatos de transición, `@ after t`. Ejecutarlo es otra operación. | `runtime.rs:119-179` |
| Regla | cláusula Horn `A1 & … & An <= C` | Monótona en Known. | `derive.rs:32-77` |
| Programa | paquete de usos de reglas con parámetros compartidos | Se compila a reglas ordinarias `P::i::r`. | `programs.rs:184-234` |
| Asociación | `seeded verb sells(...) = program givingProgram` | Cada instancia positiva activa la definición. | `derive.rs:192-303` |
| Default | `supernormal X [blocked when B]`, `exception X to D [when C] [blocked when B]` | Selección no monótona de lambdas completas sobre el cierre obligatorio. | `defaults.rs:69-158`, `experiments/default_selection.rs:21-99` |
| Residual | `A <= (B ⊸ C)` | En inferencia directa es la cláusula Horn `A & B <= C`. Las consultas residuales devuelven requisitos pendientes fuera de Known. | `typed.rs` `ResidualClause::unresiduate`; `residual/` |

**Hechos estructurales:**

- **Las reglas nunca leen `State`.** `match_antecedents` sólo consulta eventos
  de Known, y ninguna construcción consulta el `Store`. `State` queda aguas
  abajo de la inferencia.
- **Ningún procedimiento de biblioteca ni del CLI ejecuta los efectos de un
  cierre.** `execute_effect` sólo se llama desde tests (`lib.rs` y
  `cli/effect_tests.rs`). El CLI declara "`run` Forward closure and goals; no
  state execution" (`src/bin/residuum/cli.rs:17-18`).
- **Los defaults se rechazan en `close_program`**: "close_program requires a
  module without defaults" (`src/derive.rs:217-226`). También se rechazan en
  las consultas residuales y en `run_experiments`. Sólo los admite
  `run_default_experiments`.

### 1.2 Construcciones jurídicas de LegalGeneral

Firmas en `legal_general.res`. Las líneas de las reglas se citan a la derecha.

**Prestaciones y operaciones (efectos).**
- `give(subject, object, recipient)` mueve la cosa del patrimonio del sujeto
  al del destinatario (`assets -=` y `+=`, l. 81-83).
- `do(subject, conduct, indirectObject?, beneficiary)` y `notDo(...)` añaden
  un `Relation{debtor, thing, conduct, creditor, mode}` a `obligations` del
  deudor y a `rights` del acreedor, con modo `Doing` o `Abstaining`
  (l. 85-115). El comentario de l. 77-79 dice que constituyen relaciones y no
  son evidencia de conducta histórica.
- **Productores:**
  - `obligationCreatesRelation` (obligation → do @ after t, l. 119);
  - `prohibitionCreatesAbstention` (prohibition → notDo @ after t, l. 123);
  - `historicalTransferEffect` (transferred → give @ after t, l. 127);
  - en compraventa, `givingProgram` (I → give @ after t), activado por `sells`.
- **Consumidores:** ninguna regla. Los efectos son hojas del grafo
  inferencial, y su único consumidor posible es `execute_effect`.
- **Sobre `State`:** sólo si un harness los ejecuta en un orden que él elige.

**Evidencia.**
- `Act(id, subject, conduct, object, beneficiary, authority, effective)` es
  `seeded` (l. 42), y `~Act` es evidencia negativa explícita.
- **Consumidores:** `obligationCompliance`, `rightCompliance`, `prohibitedAct`
  (positiva); `obligationViolation`, `rightEnforceability`,
  `prohibitedOmission` (negativa); `conflictingActEvidence` (ambas, l. 178);
  `transferOccurrence` (Act con `transfer`, l. 201); `enactLaw` (Act con
  `enact`, l. 225).

**Calificaciones normativas.**
- **Sembradas:** `right`, `obligationFiled`, `prohibition`,
  `omissionProhibition` y `permissionFiled` (l. 43-47).
- **Derivadas:** `obligation`, que producen `filedObligation` y
  `statutoryObligation`, y `permission`, que produce `filedPermission`
  (l. 131, 137, 231). En compraventa se suman `plainTransferObligation`,
  `paidRetentionObligation` y `reciprocalTransferPermission`.
- `right` **sólo es sembrado**: ninguna regla deriva un derecho
  proposicional. El derecho constituido por `do` existe sólo como `Relation`
  en `State`, y ninguna regla puede leerlo.

**Consecuencias.**
- `compliance`: obligación o derecho + Act (l. 145, 150).
- `violation`: obligación + ~Act; prohibición + Act; prohibición de omisión + ~Act (l. 155, 165, 170).
- `enforceability`: derecho + ~Act (l. 160).
- `disputedPerformance`: compliance y violation a la vez (l. 183).
- `inconsistent`: Act y ~Act a la vez (l. 178).
- `patrimonialLiability`: desde violation (l. 189).
- `patrimonialAsset`: desde `patrimonialHolding` (l. 193); en LegalPatrimonial, también desde compliance.
- `owns`: desde patrimonialAsset (l. 197) o desde transferred (l. 205).
- `capacity`: desde owns (l. 209).
- `law`: desde `enactLaw`.
- `annulled`: desde los cuatro conflictos por jerarquía (l. 239-264).

Todas son conclusiones `Derived` positivas.

**Mecanismos aparentemente restrictivos, y por qué no restringen:**

| Construcción | Lo que hace | Lo que no hace |
|---|---|---|
| `prohibition` | Produce `violation` (con Act) y `notDo` (relación de abstención) | No impide derivar `do` ni `obligation`. `notDo` no cancela `do`: ambos añaden relaciones distintas (`Doing`/`Abstaining`) |
| `annulled(inferior, superior, …)` | Marca | "jamás retracta el inferior" (l. 238). Ninguna regla la consume |
| `inconsistent`, `disputedPerformance` | Marcas | "No dan prioridad a violation, no bloquean reglas" (l. 175-177) |
| `extinguished(id, …)` | Hecho sembrado | Ninguna regla lo consume (grep: sólo aparece como semilla). Es inerte |
| `~P` | Evidencia distinta de P | No retracta P. `~P` y `P` pueden coexistir |

**Estructura patrimonial.**
- `Person{patrimony}` y `Patrimony{assets : Set Thing, rights : Set Relation, obligations : Set Relation}`.
- `Relation{debtor, thing?, conduct?, creditor, mode}`.
- La parte proposicional (`patrimonialAsset`, `owns`, `patrimonialLiability`)
  y la parte de `State` (`assets`, `rights`, `obligations`) **no están
  conectadas por ninguna regla**. LegalPatrimonial lo declara así: "Son hechos
  Derived, no mutaciones de un Store" (`legal_patrimonial.res`).

### 1.3 Riggs

`riggs_succession_reference.res` contiene estas reglas:

| Regla | Línea | Produce | A partir de |
|---|---|---|---|
| `literalTestamentaryEntitlement` | 24 | `testamentaryEntitlement` | disposición operativa, muerte y supervivencia |
| `literalEntitlementAsset` | 32 | `patrimonialAsset` | `testamentaryEntitlement` |
| `acquisitionCausalLink` | 36 | `acquisitionLinkedToWrong` | — |
| `acquisitionPurposeLink` | 43 | `purposefulAcquisition` | — |

A partir de `patrimonialAsset`, LegalGeneral deriva `owns` y luego `capacity`.

**En Riggs no hay ninguna transición de `State`.** La adquisición es
proposicional: `patrimonialAsset` → `owns` → `capacity`. La valoración
histórica de exclusión se refiere, por tanto, a esas consecuencias derivadas.

## 2. Qué es una consecuencia jurídica efectiva

| Nivel | Objeto en Residuum | Estado |
|---|---|---|
| Derivabilidad | q ∈ F_T(S) | IMPLEMENTADO |
| Constitución de relación jurídica | Proposicional: `obligation`/`permission` derivadas. En `State`: `Relation` añadida por `do`/`notDo` | Proposicional IMPLEMENTADO; en `State`, sólo al ejecutar el efecto (harness) |
| Ejecución sobre `State` | `execute_effect(e, verb, σ)`, función pura sobre un único evento | IMPLEMENTADO como función. **No hay planificador**: el orden y la elección de qué efectos ejecutar son del harness |
| Modificación patrimonial | Proposicional: `patrimonialAsset`/`owns`. En `State`: `assets` vía `give` | Las dos capas no se conectan |
| Validez | No hay predicado. `annulled` es marca | NO REPRESENTABLE como condición de producción |
| Aplicabilidad | Aplicabilidad de un default (`when`) | IMPLEMENTADO para defaults |
| Eficacia | Selección de la lambda productora: no bloqueada ni derrotada | IMPLEMENTADO por lambda; NO REPRESENTABLE por instancia |
| Resultado normativo respecto de Q | Proyección del harness (`Study::goal`, `predictions`) | Externo. No hay función universal (`legal-recalcitrance-mathematical-model.md:722`) |

**Mecanismos que condicionan, restringen, sustituyen o impiden la producción
de efectos.** Existen tres, todos en la capa de selección:

1. **Condición de aplicabilidad.** Un default `exception … when C` sólo se
   aplica si C empareja en el cierre obligatorio.
2. **Bloqueo.** Un emparejamiento de `blocked when B` en el cierre obligatorio
   saca la lambda de la selección. La lambda puede ser una regla o un
   programa.
3. **Prioridad.** Una excepción seleccionada derrota su objetivo y la cadena
   de prioridades de éste. Una excepción bloqueada no derrota.

Algoritmo en `src/defaults.rs:69-158` y `176-282`, y en
`experiments/default_selection.rs:21-99`. Sus condiciones exactas:

- Las condiciones y los bloqueos sólo pueden usar parámetros de la regla o el
  programa objetivo, o constantes (`elab.rs:656-713`).
- Ven sólo el cierre obligatorio M_T(S), nunca consecuencias de otros
  defaults.
- No hay iteración: "no Reiter iteration" (`default_selection.rs:18-20`).
- El bloqueo es global sobre la lambda: "any witness blocks the entire
  parametrized rule lambda" (`defaults.rs:24-31`). El test
  `default_blocking_is_global_and_preserves_every_witness` está en
  `lib.rs:2747`.
- La consistencia queda `NotChecked`.
- La selección no se combina con asociaciones `Verb.program` sobre reglas
  controladas (`default_selection.rs:51-67`).

## 3. Transformaciones normativas

Sea T = (R, Δ, Π, Eff):
- R, reglas obligatorias;
- Δ, defaults;
- Π, programas y asociaciones;
- Eff, verbos de efecto.

Una transformación es T ↦ T′ con T′ elaborable y de cierre finito. Respecto de
(S, Q) cambia el resultado si N_{T′}(S, Q) ≠ N_T(S, Q) (§4).

| Operación | Forma en Residuum | Clasificación | Evidencia |
|---|---|---|---|
| Ampliar condiciones de producción | Añadir una regla Horn con cabeza existente | **Expresable y ejecutable.** Monótona: F_T ⊆ F_{T′} | `derive.rs`; teorema de persistencia (`legal-recalcitrance-mathematical-model.md:702-707`) |
| Restringir la aplicación de una regla: sustitución | Reemplazar r por una especialización r′ (copia privada) | **Expresable por composición** (sustitución de módulo); no es operación del motor. Ejecutado una vez en el estudio de compliance | `recalcitrance_revisions.res`; `legal-recalcitrance-justificatory-functions.md:247-263` |
| Restringir la aplicación de una regla: bloqueo | Poner r bajo `supernormal … blocked when B` | **Expresable por composición y ejecutable** con `run_default_experiments`, por lambda | §8.3 |
| Constituir una posición normativa nueva | Regla con cabeza `obligation`/`permission` (Derived), o `do`/`notDo` (efecto) | **Expresable y ejecutable.** `right` no es derivable | §1.2 |
| Modificar una relación jurídica existente | Proposicional: imposible (sin retractación). En `State`: sólo con un efecto `-=` sobre `rights`/`obligations`, que no existe | **No representable sin ampliar la ontología** (verbo de efecto nuevo); `extinguished` es inerte | §1.2 |
| Condicionar una operación patrimonial | Conjunción en el productor del efecto o de la obligación, o selección de paquete por default | **Implementado y verificado** (compraventa con retención) | `sales.res`; `program_default_tests.rs` |
| Impedir o limitar un efecto | Bloquear la lambda productora del evento de efecto o de la consecuencia | **Expresable por composición**, por lambda. **No representable por instancia.** Suprimir un efecto ya derivado: no representable; sólo el harness puede no ejecutarlo | §8.3; `defaults.rs:24-31` |
| Sustituir una regla o un programa | `exception X to D when C` | **Implementado y verificado** (RetentionSale ↦ NormalSale), por lambda | `sale_defaults.res`; `default_tests.rs`, `program_default_tests.rs` |
| Resolver consecuencias concurrentes | `disputedPerformance`, `inconsistent`, `annulled` | **Sólo registro.** La resolución es una evaluación externa | `legal_general.res:175-177, 238` |

## 4. Modelo matemático mínimo

Sea E el universo ground y S ⊆ E una entrada sembrada finita.

1. **Cierre obligatorio.** M_T(S) = C_{R_m}(S), donde R_m son las reglas no
   controladas por defaults. Es el menor punto fijo Horn: extensivo, monótono
   en S y en R_m, e idempotente (DEMOSTRADO bajo cierre finito;
   `legal-recalcitrance-mathematical-model.md:242-259`).
2. **Selección.** Sel_T(S) = sel(Δ, M_T(S)) ∈ 𝒫(Λ), con Λ el conjunto de
   lambdas controladas. sel aplica aplicabilidad, bloqueo y prioridad. Es una
   función, no monótona, de M_T(S).
3. **Cierre final.** F_T(S) = C_{R_m ∪ R(Sel_T(S))}(S), reiniciado desde las
   semillas (`default_selection.rs:79-91`).
4. **Procedencia.** 𝒫_T(S) = (registros de derivación de F_T(S), registros de
   la extensión: aplicables, bloqueados con su evidencia y sustitución,
   derrotados, seleccionados).
5. **Ejecución, opcional.** Para σ_0 ∈ Store y un orden π de eventos de efecto
   de F_T(S), X(σ_0, π) es la composición parcial de `execute_effect`. **El
   motor no fija π ni qué efectos ejecutar.**
6. **Decisión.** N_T(S, Q) = D_Q(F_T(S), 𝒫_T(S) [, X]).

Las capas 1 a 4 las ejecuta el motor; 5 y 6 corresponden al harness.

Sin defaults, Sel es vacío y F = M. Sin ejecución, X no interviene. Sólo
entonces la descomposición (C_T, D_T) es suficiente.

**Requisitos sobre D**, si se introduce:
- Dominio: (F, 𝒫), y X si Q se refiere a `State`.
- Codominio Y_Q finito y declarado.
- Entradas observables: pertenencia de eventos ground y registros de
  procedencia. **Nunca** las etiquetas y\*.
- Completitud: D sólo se aplica a cierres completos.
- Determinismo, trazabilidad (devuelve los eventos y registros usados) y
  separación (no escribe en Known ni en la procedencia).

Toda política (por ejemplo, "exclusión si hay atribución sin activo") es una
estipulación y debe declararse como tal.

**Ejemplo de D para la adquisición sucesoria** (propuesta, no adoptada).
Q = (b, o, w), es decir, "¿adquiere b el objeto o en virtud de w?", con
Y_Q = {Adquisición, Exclusión, Sin atribución}:

- **Adquisición** si `owns(b, o, w, ·, ·)` ∈ F.
- **Exclusión** si `testamentaryEntitlement(w, ·, b, o, ·, ·)` ∈ F,
  `owns(b, o, w, ·, ·)` ∉ F, y 𝒫 registra un bloqueo o una derrota de un
  productor de `patrimonialAsset`.
- **Sin atribución** si `testamentaryEntitlement` ∉ F.
- Otro caso: **Indeterminado**.

Lee sólo consecuencias existentes y procedencia. La partición del codominio
es estipulada.

## 5. Impedimento

**Definición provisional:** «condición jurídica que restringe la eficacia de
una consecuencia o transición identificada, sin negar necesariamente que sus
antecedentes formales se satisfacen».

**Relación implementada:**

  Blocks_T(Γ, λ, S) ⇔ existe d ∈ Δ que controla λ, d es aplicable en M_T(S) y
  Γ ⊆ M_T(S) es un emparejamiento del bloqueo de d.

Consecuencias:
- λ ∉ Sel_T(S).
- Toda consecuencia c que sólo produce λ falta de F_T(S).
- Las instancias de los antecedentes de λ pueden seguir en F_T(S).
- La procedencia conserva Γ y su sustitución.

**Correspondencia con la definición provisional.** Es fiel en lo esencial: en
Riggs, `testamentaryEntitlement` sigue derivada y desaparecen
`patrimonialAsset`, `owns` y `capacity` (§8.3). Requiere cuatro precisiones,
todas comprobadas:

1. **Lo que se restringe es un productor (una lambda), no una transición de
   `State`.** Una transición queda impedida sólo si su evento de efecto deja
   de derivarse. Un efecto ya derivado no puede impedirse dentro del motor.
2. **La granularidad es la lambda dentro del expediente, no la instancia.**
   Contraejemplo §8.3: bloquea también al beneficiario inocente. La relación
   que la definición presupone, Blocks(Γ, τ, S) con τ una instancia ground,
   **no es representable**.
3. **El impedimento no es un objeto del cierre**, sino un registro de la
   selección. No hay evento "impedido" en Known, y D debe leer 𝒫.
4. **La "eficacia" no es un predicado aparte.** Es la derivabilidad de las
   consecuencias aguas abajo del productor seleccionado.

**Definición propuesta, más fiel al código:**

> Un impedimento de T es un par (d, λ) donde d es un default que controla la
> lambda λ productora de una consecuencia jurídica existente. Su bloqueo es
> una configuración Γ, expresada en relaciones existentes del vocabulario y
> con las ligaduras de λ, cuya presencia en el cierre obligatorio excluye λ
> de la selección, conservando los antecedentes de λ.

La versión **por instancia** exige una semántica de selección sobre
instancias ground. Es la capacidad que falta.

## 6. Límites expresivos

**Proposición 1 (persistencia; DEMOSTRADO,** `legal-recalcitrance-mathematical-model.md:702-707`**).**
Si T no tiene defaults y T′ = T ∪ {r} con r Horn, entonces F_T(S) ⊆ F_{T′}(S).

**Proposición 2 (valoraciones antimonótonas; DEMOSTRADO aquí).**

*Enunciado.* Sea T cualquier teoría sin defaults. No es necesario que T
extienda a la de referencia: vale para cualquier conjunto de reglas Horn. Sea
D_q(F) = [q ∈ F]. Si S ⊆ S′, y\*(S) = 1 e y\*(S′) = 0, ninguna T ajusta las
dos valoraciones.

*Prueba.* M_T es monótona en S, así que q ∈ F_T(S) implica q ∈ F_T(S′).

*Consecuencia para Riggs.* Las semillas de `causalLinkUnestablished` están
estrictamente incluidas en las de `historicalRiggs` (comprobado). Si una
valoración externa concediera la adquisición en el primero y la negara en el
segundo, ninguna teoría Horn con decisión por pertenencia de `owns` podría
ajustarlas.

*Escapes.* Sólo hay dos:
1. evidencia negativa explícita: exigir `~intentionalKilling` para adquirir
   desplaza la carga de la prueba, y deja sin adquisición a los expedientes
   sin esa evidencia;
2. una capa no monótona: defaults, o una D no monótona en F.

**Proposición 3 (bloqueo y fuerza del bloqueador; DEMOSTRADO aquí, sólo para
defaults `supernormal` sin excepciones).**

*Enunciado.* Para S fijo, reforzar la condición de bloqueo (B ∧ B′ en lugar
de B) amplía la selección, y por tanto F_T(S).

*Prueba.* Un emparejamiento de B ∧ B′ contiene uno de B. Cada lambda
bloqueada con B ∧ B′ lo está con B, así que Sel crece. Por último, C es
monótona en las reglas.

*Límite.* Con excepciones no se ha probado: un bloqueo puede retirar una
derrota.

**Límites del lenguaje** (comprobados en código o fixture):
- No hay retractación de Known.
- No hay negación por ausencia.
- Ninguna regla lee `State`, y no hay planificador de efectos.
- `right` no es derivable.
- No existe ningún efecto que extinga relaciones; `extinguished` es inerte.
- Los bloqueos sólo usan los parámetros del objetivo. Se sortea envolviendo
  la regla en un programa con parámetros adicionales (§8.3).
- Los defaults ven sólo M_T y no se iteran.
- Los defaults son incompatibles con `close_program`, las consultas
  residuales y `Verb.program` sobre reglas controladas.

## 7. Espacio de búsqueda y aprendizaje

**Explicación estructural del Piloto 003.**
- Las propuestas del explorador son sólo declaraciones de regla (sintaxis
  Formal entregada al modelo).
- El trabajador evalúa con `close_program` (`src/experiments/proposal.rs:210,
  407`; `exploration.rs:156, 332` en la rama del piloto), que rechaza
  defaults.
- Por las Proposiciones 1 y 2, el único cambio observable posible era añadir
  un evento positivo. De ahí la cabeza artificial.

**Distinciones necesarias:**

| Nivel | Qué fija | Estado |
|---|---|---|
| Admisibilidad de la configuración | Tipado, vocabulario del perfil, ligaduras de cabeza | IMPLEMENTADO para reglas |
| Admisibilidad de la instalación | Elaboración del módulo hipotético; terminación (aciclicidad); restricciones de defaults (§6) | Reglas: IMPLEMENTADO. Defaults: comprobado por elaboración y por `run_default_experiments`, no por el trabajador |
| Comportamiento inferencial | F_T(S), 𝒫_T(S) | IMPLEMENTADO (con defaults sólo en `run_default_experiments`) |
| Comportamiento sobre `State` | X | Sólo harness |
| Adecuación a las valoraciones | κ(D(F, 𝒫), y\*) | Harness. D pendiente |

**Lenguaje de transformación que permitiría aprender sin consecuencia
artificial** (EXPRESABLE hoy; su ejecución en el circuito no está
implementada):

1. Adición Horn con cabeza en verbos `derived` o `effect` existentes, para
   ampliar o constituir.
2. Colocar una lambda existente λ bajo un default `supernormal` bloqueado por
   una configuración Γ de relaciones existentes, para restringir o impedir.
3. Excepción que sustituye un productor bajo una condición, para sustituir.
4. Especialización de una regla existente (copia privada), para restringir
   monótonamente.

El resultado se observaría con una D fija y declarada sobre consecuencias
existentes (`owns`, `patrimonialAsset`, `obligation`, eventos `give`/`do`).

**El problema jurídico se entregaría como Q y valoraciones, no como cabeza.**

Restricciones constitutivas ya codificadas:
- tipado y firmas;
- seguridad de parámetros en la cabeza;
- `EvidentialNot` sólo derivado;
- ámbito de las condiciones de default;
- tipos iguales para excepción y objetivo;
- prohibición de `Verb.program` sobre reglas controladas;
- aciclicidad para la terminación del runner.

Sigue abierto:
- qué lambdas de la referencia son "derrotables" (hoy cualquier regla puede
  ponerse bajo un default, lo que puede no ser admisible jurídicamente);
- la granularidad por instancia;
- D.

**Identificabilidad.** Si varias transformaciones producen el mismo resultado
en los expedientes disponibles, la partición de hipótesis no las separa. Esto
vale para cualquier D (§9).

## 8. Casos de referencia

### 8.1 A. Compraventa

`sales.res`:
- `sale(...)` selecciona un paquete de reciprocidad: `normalSale` →
  `selectedReciprocity(Plain)` y `retentionSale` → `Retention`.
- `reciprocalTransferPermission` deriva `permission(permit, seller, transfer, …)`.
- `plainTransferObligation` deriva `obligation(duty, seller, transfer, …)`.
- `paidRetentionObligation` exige, además, `paymentFor` y
  `Act(payment, buyer, pay, …)`. Es una **operación patrimonial
  condicionada** expresada por conjunción.
- La obligación produce `do(seller, transfer, object, buyer) @ after t` por
  `obligationCreatesRelation`. Al ejecutarse, es una `Relation` en `State`.
- La titularidad pasa sólo con evidencia histórica: `Act(..., transfer, ...)`
  → `transferred` → `owns` del destinatario (`legal_general.res:201, 205`).
- `seeded verb sells(...) = program givingProgram` deriva el evento de efecto
  `give @ after t`, que no se ejecuta.

`sale_defaults.res`:
- `NormalSale` (`supernormal normalSaleProgram blocked when retentionAgreed`).
- `RetentionSale` (`exception retentionSaleProgram to NormalSale when retentionAgreed`).

Es la **sustitución de un programa** por excepción y bloqueo, IMPLEMENTADO y
con tests (`default_tests.rs`, `program_default_tests.rs`; tabla en
`docs/program-defaults.md:78-83`).

Cubre: constituir, condicionar y sustituir. No cubre la ejecución de
`State`, que es del harness.

### 8.2 B. Cumplimiento y violación

- **Monotonía.** Obligación + Act → compliance; obligación + ~Act →
  violation. Con evidencia de ambos signos coexisten (`disputedPerformance`).
  No hay prioridad: es un registro de concurrencia sin resolución.
- **Consecuencias patrimoniales proposicionales.** `violation` →
  `patrimonialLiability`; con LegalPatrimonial, `compliance` →
  `patrimonialAsset`. No hay mutación de `Store`.
- **La prohibición no bloquea.** Produce `violation` y `notDo`, y deja intacta
  la obligación concurrente.
- **Jerarquía.** El conflicto jerárquico produce `annulled` sin retraer.
- **Resolución.** Resolver la concurrencia exige D o una selección por
  defaults.

### 8.3 C. Riggs (verificación acotada)

Fixtures en `docs/legal-geometry/transformations/fixtures/` y salidas en
`.../runs/`. Las cuatro ejecuciones terminaron con código 0 y sin incidencias
del runner. Los informes son `20261015T204459897263Z-2815923`,
`…204510148151Z-2816407`, `…204553358370Z-2817109` y
`…204553911123Z-2817205`.

**Referencia literal** (`riggs_reference_owns.res`, sin reglas añadidas;
objetivo `owns(elmerPalmer, residuaryInterest, palmerWill, private, will1880)`).
Se deriva en cinco expedientes, todos salvo `survivalUnestablished`.
`historicalRiggs` incluido.

**Bloqueo** (`riggs_blocked_entitlement_asset.res`).
- `literalEntitlementAsset` se envuelve en un programa con parámetros
  adicionales `act` y `death`. Es necesario, porque el bloqueo sólo admite
  parámetros del objetivo.
- El programa se pone bajo
  `supernormal … blocked when purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t`.
- **Sin verbo nuevo ni cuerpo de regla nuevo.**

Resultado por expediente:

| Expediente | Bloqueo | `owns` |
|---|---|---|
| `historicalRiggs` | sí, con evidencia `purposefulAcquisition(palmerWill, killingEvidence, palmerDeath, elmerPalmer, …)` | **no** |
| `noBeneficiaryKilling`, `killingWithoutAcquisitionPurpose`, `killingByThirdParty`, `causalLinkUnestablished` | no | sí |
| `survivalUnestablished` | no | no (como en la referencia) |

En `historicalRiggs` el cierre final contiene `testamentaryEntitlement`,
`acquisitionLinkedToWrong` y `purposefulAcquisition`, pero no
`patrimonialAsset`, `owns` ni `capacity`.

**Contraejemplo de granularidad** (`two_beneficiaries_*.res`). Un testamento
con dos disposiciones (Elmer y `innocentHeir`); sólo Elmer mata con propósito
registrado.

| Teoría | `owns` de `innocentHeir` | `owns` de Elmer |
|---|---|---|
| Referencia | sí | sí |
| Bloqueo | **no** | no |

El bloqueo de la lambda excluye a quien no cumple la configuración
bloqueante.

**Qué tendría que cambiar para la exclusión efectiva.**
- Q se refiere a `owns`/`patrimonialAsset` del beneficiario; no hay `State`.
- Por la Proposición 2, ninguna adición ni reemplazo Horn la produce si la
  valoración es antimonótona; con los seis expedientes actuales no lo es
  necesariamente, porque sólo una valoración está documentada.
- Con el bloqueo por lambda se obtiene sobre los seis expedientes, pero falla
  con varios beneficiarios.
- **La capacidad que falta es el bloqueo por instancia.**
- El mecanismo no se declara jurídicamente correcto por reproducir la
  decisión histórica: es un candidato de forma admisible.

## 9. Geometría

1. **Adición Horn** (resultados previos). Engrosa las particiones de clausura,
   ex_{C′} ⊆ ex_C y las regiones crecen. El antiintercambio no se conserva y
   debe recertificarse (`legal-ontology-region-revision.md:192-222`).
2. **Reemplazo o especialización.** No hay teorema general. Las particiones de
   observación pueden quedar incomparables
   (`legal-recalcitrance-mathematical-model.md:458-464`).
3. **Defaults. Contraejemplo, VERIFICADO-FINITO aquí.**
   - Con el bloqueo de §8.3, S = semillas de `causalLinkUnestablished` ⊂
     S′ = semillas de `historicalRiggs`, mientras `owns` ∈ F(S) y
     `owns` ∉ F(S′).
   - Por tanto F no es monótona ni operador de clausura sobre 𝒫(E).
   - La región de `owns` no es un conjunto ascendente, así que su frontera
     mínima no la genera.
   - Estados cerrados, bases extremas y antiintercambio no se aplican a F.
   - Se aplican a M_T y a cada C_{R_m ∪ R(σ)} con σ fija. F es, por trozos,
     un cierre sobre las regiones {S : Sel(S) = σ}, pero esas regiones no son
     cerradas bajo unión. No se afirma ninguna estructura de retículo sobre
     ellas.
4. **Equivalencias observacionales y retícula de particiones de hipótesis.**
   Sólo requieren que la observación sea función de la hipótesis; los núcleos
   y los encuentros siguen valiendo para cualquier D y con defaults.
   **No se introduce ningún cociente**: la congruencia no está demostrada y
   hay contraejemplos previos
   (`legal-recalcitrance-mathematical-model.md:500-541`).
5. **Orden de generalidad.** Cov(Γ) ⊆ Cov(Γ′) sigue definido. La condición
   suficiente de subsunción estructural es sólo para Horn positivo; para
   bloqueos vale la Proposición 3, sólo en el caso supernormal.

## 10. Matriz de correspondencia

| Objeto jurídico | Definición matemática | Construcción | Evidencia | Estado |
|---|---|---|---|---|
| Evidencia positiva y negativa | eventos de polaridad distinta | `seeded`, `~` | `typed.rs` `Event`, `elab.rs:787-796` | IMPLEMENTADO |
| Cierre obligatorio | M_T = lfp Horn | `close`, `close_program` | `derive.rs:185-200` | IMPLEMENTADO; DEMOSTRADO bajo cierre finito |
| Persistencia bajo adición | F_T ⊆ F_{T∪r} | — | `legal-recalcitrance-mathematical-model.md:702-707` | DEMOSTRADO |
| Valoración antimonótona | S ⊆ S′, y\*=1→0 | — | §6 | DEMOSTRADO (inexpresable sin capa no monótona) |
| Selección de defaults | Sel = sel(Δ, M) | `evaluate_defaults` | `defaults.rs:69-158` | IMPLEMENTADO |
| Bloqueo por lambda | Blocks(Γ, λ, S) | `blocked when` | `defaults.rs:104-122`; `lib.rs:2747`; §8.3 | IMPLEMENTADO y VERIFICADO-FINITO |
| Bloqueo por instancia | Blocks(Γ, τ, S) | — | contraejemplo §8.3; `defaults.rs:24-25` | NO REPRESENTABLE |
| Sustitución | prioridad de excepción | `exception … to` | `defaults.rs:176-238`; `sale_defaults.res` | IMPLEMENTADO |
| Cierre final | F_T | `run_default_experiments` | `default_selection.rs:21-99` | IMPLEMENTADO |
| No monotonía de F_T | F(S) ⊄ F(S′) para S ⊂ S′ | — | §9; fixtures | VERIFICADO-FINITO |
| Constitución proposicional | `obligation`, `permission` derivadas | reglas | `legal_general.res:131-137, 231` | IMPLEMENTADO |
| Derecho proposicional derivado | — | `right` sólo sembrado | `legal_general.res:43` | NO REPRESENTABLE |
| Relación en `State` | `Relation` en `rights`/`obligations` | `do`, `notDo` | `legal_general.res:85-115`; `runtime.rs` | IMPLEMENTADO como efecto; ejecución sólo en tests |
| Transferencia en `State` | `assets -=`/`+=` | `give` | `legal_general.res:81-83` | Ídem |
| Extinción de relación | — | `extinguished` inerte | grep | NO REPRESENTABLE sin efecto nuevo |
| Planificador de efectos | X(σ_0, π) | — | `cli.rs:17-18`; sólo tests | NO IMPLEMENTADO (harness) |
| Reglas que leen `State` | — | — | `derive.rs` | NO REPRESENTABLE |
| Nulidad o validez | marca `annulled` | regla | `legal_general.res:238-264` | Sólo marca |
| Concurrencia | `disputedPerformance`, `inconsistent` | reglas | `legal_general.res:175-187` | Sólo registro |
| Resultado efectivo | D_Q(F, 𝒫 [, X]) | harness | `legal-relational-search-space-contract.md:326-346` | Pendiente; ejemplo en §4 |
| Impedimento | (d, λ, Γ) | default bloqueado | §5, §8.3 | EXPRESABLE por lambda |
| Monotonía del bloqueador | B∧B′ ⇒ F mayor | — | §6 | DEMOSTRADO (supernormal) |
| Cociente de particiones | — | — | contraejemplos previos | No introducido |

## 11. Obligaciones pendientes, por dependencia

1. **Fijar Q y D** para la adquisición sucesoria. Codominio, entradas
   (F y 𝒫, sin etiquetas), estipulaciones declaradas y completitud. Todo lo
   demás depende de esto.
2. **Decidir la granularidad del bloqueo.** O se acepta la lambda con una
   restricción de dominio declarada (un beneficiario por expediente), o se
   especifica y revisa una semántica de selección por instancia, que sería un
   cambio del motor.
3. **Declarar qué productores de la referencia son derrotables** y con qué
   bloqueadores admisibles. Es una restricción constitutiva, no del explorador.
4. **Definir el lenguaje de transformación** (las formas 1 a 4 de §7) y su
   admisibilidad, incluida la terminación.
5. **Extender el trabajador y el explorador** para evaluar transformaciones
   con defaults. Hoy `close_program` los rechaza. Es implementación, con su
   propia revisión.
6. **Geometría por trozos** para F con defaults. Recertificar por
   transformación; la identificabilidad por particiones de hipótesis se
   conserva.
7. **Segunda valoración independiente,** necesaria para cualquier afirmación
   de aprendizaje.
8. **Capa de `State`**, sólo si alguna Q se refiere a ella (compraventa).
   Requiere un planificador del harness con orden declarado. Riggs no lo
   necesita.

## 12. Respuesta al criterio de éxito

- **Residuum ya puede representar algunas transformaciones normativas
  efectivas** con construcciones existentes: ampliar, constituir, condicionar,
  sustituir y restringir o impedir a nivel de lambda.
- **Con el bloqueo por lambda reproduce la exclusión histórica de Riggs** sin
  consecuencia artificial: es la ausencia de `owns` con la atribución
  conservada.
- **No puede representar el impedimento relativo a la instancia.** Lo muestra
  el contraejemplo verificable de dos beneficiarios.
- **El circuito del explorador tampoco puede hoy evaluar ninguna
  transformación no aditiva.**

Un explorador LLM podría aprender legítimamente, sin consecuencia artificial,
transformaciones de las formas 1 a 4 evaluadas con una D declarada. Para que
lo que aprenda sea un impedimento personal en dominios con varios
beneficiarios, falta la selección por instancia.
