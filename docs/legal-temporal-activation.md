# Activación temporal y aprendizaje normativo en Residuum

Rama `experiment-legal-order-geometry`. Investigación, no implementación: no se
modifica el motor, la ontología ni las fixtures originales; no se ejecuta ningún
piloto LLM; no se introducen defaults como solución. Las fixtures nuevas son
independientes y viven en `docs/legal-geometry/temporal-activation/`.

Clasificación de cada afirmación:

- **IMPLEMENTADO**: lo hace el código tal como está, con cita.
- **DEMOSTRADO-BH**: demostrado bajo las hipótesis enunciadas.
- **VERIFICADO-FINITO**: comprobado con el motor real en un dominio finito,
  con salida archivada.
- **EXPRESABLE-NI**: expresable con el lenguaje actual, no implementado ni
  verificado aquí.
- **NO-REPRESENTABLE**: el lenguaje actual no lo permite (con prueba).
- **INDETERMINADO**: no resuelto.

## 0. Resultado principal

1. **Los operadores temporales propiamente dichos sólo indexan.** `@ t`,
   `@ after t` y el constructor `after(·)` forman un álgebra libre de términos.
   Hay constantes de tipo `PropositionTime` y un sucesor sintáctico inyectivo.
   El emparejamiento es sintáctico y unilateral. No hay orden, ni comparación,
   ni intervalos, ni vigencia de reglas, ni tiempo de consulta. Una regla sin
   constante temporal se aplica en todos los índices (IMPLEMENTADO;
   VERIFICADO-FINITO, F1). `after` expresa la adyacencia de un paso, no la
   posterioridad (F1).

2. **La selección positiva sí existe, pero no es temporal.** `Verb.program`
   hace que un programa se active sólo ante instancias conocidas de su verbo.
   Los argumentos y el índice de la instancia se toman como bindings. La
   granularidad es la instancia ground, y el disparador puede derivarse de
   propiedades relacionales del expediente (IMPLEMENTADO; VERIFICADO-FINITO,
   F3). La selección por tiempo se obtiene haciendo que el disparador dependa
   de datos de orden explícitos, como `notEarlier` en LegalGeneral (F8).

3. **Ninguna regla añadida puede hacer que otra deje de producir una
   consecuencia** (Proposición 1, DEMOSTRADO-BH; VERIFICADO-FINITO, F2). Con
   reglas ordinarias y asociaciones fijas, la clausura es monótona en la
   evidencia y en las reglas.

   Lema de la coordenada (§2.4): una revisión conservadora en la que la
   consecuencia anterior deje de valer exige una de dos cosas:

   - que la consecuencia lleve en su clave lógica la coordenada de la revisión
     (época de aprendizaje o vigencia jurídica);
   - una operación de teoría no aditiva.

4. **Hay dos construcciones existentes que logran a la vez revisión por
   clases, conservación histórica y granularidad por instancia**, sin
   defaults y sin negación por ausencia en el motor:

   - **F4, teoría única por épocas.**
     - Cada generalización se escribe desde T0 con una guarda
       `unrevisedThrough(g, e)`. La consecuencia lleva la época `e`.
     - El aprendiz extiende la vigencia de las generalizaciones no revisadas
       añadiendo certificados.
     - Resultados: T0 ⊆ T1 ⊆ T2 ⊆ T3; las cierres crecen; cada época queda
       fija desde que se aprende; una sola ejecución reconstruye todas las
       épocas (VERIFICADO-FINITO; DEMOSTRADO-BH en §8.3).
   - **F8, captura de la regla en un programa más vigencia.**
     - La regla anterior pasa a activarse por dos rutas: fechas anteriores a
       la revisión, o evidencia explícita de ¬A.
     - No es una adición: capturar una regla existente en un programa la
       retira del conjunto de reglas ordinarias (Proposición 2,
       VERIFICADO-FINITO, F7).

5. **Ambas tienen dos condiciones que no son de la temporalidad.**
   - **(a) Quién decide la desactivación.** En F4 no la produce la nueva regla,
     sino la construcción de la teoría: el aprendiz no extiende un
     certificado. En F8 la produce un reemplazo.
   - **(b) Evidencia negativa explícita.** El refinamiento "perro sin A"
     necesita `~propertyA(dog)`. Sin ella el expediente queda indeterminado:
     no prohibido, pero tampoco permitido. Es la misma disciplina que los
     polinomios, donde un coeficiente ausente no es cero (§10).

6. **Los defaults no conservan la historia.** Con un bloqueador en t1 para un
   perro, desaparecen las consecuencias del lambda en t0 y para los demás
   perros (VERIFICADO-FINITO, F5). Fallan la persistencia D y la
   granularidad.

7. **Capacidad mínima que falta**, en el sentido estricto pedido: que una
   generalización nueva, añadida como regla, suspenda la activación de una
   anterior sólo para las instancias ground que cubre y sólo a partir de un
   índice, sin evidencia negativa explícita. Es una derrota acotada por
   instancia e índice. Los operadores existentes no la dan:
   - las reglas y los programas son monótonos (Proposición 1);
   - los defaults bloquean por lambda en todos los índices (F5);
   - la captura por programa es un reemplazo global de la regla (F7);
   - `after` no ordena (Proposición 3).

   Si se aceptan las condiciones 5(a) y 5(b), el lenguaje actual basta y no
   falta ninguna operación del motor; lo que falta es contrato (§9). No se
   diseña la operación (§12).

## 1. Inventario de operadores temporales reales

### 1.1 Tabla

| Construcción | Código | Qué representa formalmente | Operaciones | Estado |
|---|---|---|---|---|
| Índice proposicional `@ t` | `TimeExpr::{At, Var}` (`src/typed.rs:504`); superficie `src/formal.rs:294` | Componente de la clave lógica del evento (`logical_key`, `typed.rs:433`) | Igualdad sintáctica; ligadura de un parámetro de tipo `PropositionTime` | IMPLEMENTADO |
| `@ after t` | `TimeExpr::After` (`typed.rs:504`); superficie un nivel, `formal.rs:294-300`; elaboración `proposition_time`, `src/elab.rs:1042` | Sucesor sintáctico libre e inyectivo | Construir; descomponer por patrón `after(x)`. No hay inversa general, comparación ni suma primitiva | IMPLEMENTADO |
| `after(·)` como término | `Constructor::After : PropositionTime → PropositionTime` (`typed.rs:514-521`); `to_term`/`from_term` | El mismo sucesor como argumento ordinario (por ejemplo, `effective`) | Anidamiento arbitrario en argumentos y bindings de `RuleUse` | IMPLEMENTADO (`docs/temporal-constructors-experiment.md`) |
| `PropositionTime` como argumento | `effective : PropositionTime` en LegalGeneral (`legal_general.res:42-76`) | Dato de vigencia del acto o la norma. Según el comentario de la ontología, "no se identifica con DerivationTime" ni con `tau` | Ninguna, salvo igualdad | IMPLEMENTADO |
| Orden como dato | `seeded verb notEarlier(later, earlier)` (`legal_general.res:53`); consumido por `permissionAnnulsProhibition` (`:239`) | Relación extensional aportada por el expediente | Ninguna inferencia propia, sin transitividad | IMPLEMENTADO |
| `DerivationTime` | `typed.rs:222`; `DerivationClock`; sello `d1, d2…` | Orden de producción dentro de una ejecución | Sólo se registra; no se compara entre ejecuciones (`docs/peano-polynomial-addition-experiment.md`) | IMPLEMENTADO |
| `StateTime` / transición de efecto | `typed.rs:164`; `effect_transition`, `elab.rs:1056`: entrada `t`, salida `after t` | Tipado de una transición de State | No se ejecuta en la clausura | IMPLEMENTADO (sólo tipo) |
| Regla con constante temporal | `check_rule_event` (`elab.rs:964`): el tiempo de un evento de regla es un parámetro o una constante `PropositionTime` | Regla fijada a un índice | La regla sólo empareja eventos con ese índice | IMPLEMENTADO; F6 |
| Especialización temporal | `rule r' = r(t0)`, `SurfaceRuleBody::Application` (`elab.rs:808`) | Beta-reducción del parámetro temporal | Añade una regla; no restringe la original | IMPLEMENTADO; F6 |
| Activación por programa | `Verb.program`; `prepare_programs` (`src/derive.rs:212`), `activation_bindings` (`:341`), `finish_program_match` (`:361`) | Instanciación de las reglas de P con los argumentos y el índice del disparador | Selección positiva por instancia | IMPLEMENTADO; F3 |
| Defaults | `evaluate_defaults` (`src/defaults.rs:69`); `BlockingScope` (`:27`) | Bloqueo y selección de lambdas sobre la clausura obligatoria | Bloqueo global por lambda | IMPLEMENTADO; F5 |
| Evidencia negativa `~V` | `Polarity::EvidentialNot` (`typed.rs:464`); restricción `elab.rs:789-796` | Hecho positivo de no ocurrencia: no es negación por ausencia | Se empareja como cualquier literal | IMPLEMENTADO; F4, F7, F8 |
| Clausura incremental y por pasos | `IncrementalClosure`, `IncrementalProgramClosure` (`src/derive/incremental.rs:25`: "Rules are copied once and cannot change") | Mantenimiento sólo-añadir con reglas fijas | Añadir semillas; nunca reglas | IMPLEMENTADO |
| Aritmética de Peano y polinomios | Fixtures `examples/experiments/peano_*.res`; tests `src/bin/residuum/cli/peano_polynomial*_tests.rs` | `zeroTime` como Z y `after` como S; suma, producto y polinomios por reglas recursivas | Recursión sobre espinas finitas | VERIFICADO (3 tests, §10); rechazado por el runner de fixtures y el trabajador |

### 1.2 Respuestas por construcción

**`@ t` y `after`.**

- Bindings: `bind_time` (`src/derive.rs:648`) liga un parámetro temporal a
  cualquier `TimeExpr` y desciende `after/after`. En cualquier otro caso exige
  igualdad.
- Matching: `match_event` compara verbo, polaridad y aridad, luego argumentos
  y luego tiempo.
- Activación: el tiempo condiciona la activación sólo como un argumento más
  del patrón. Una regla de parámetro `t` vale en cada índice.
- Vigencia, anterioridad, posterioridad, intervalos: no los representa.
  `after(after(t0))` no empareja con `after(x)` para x = t0 (F1, `afterSteps`),
  y entre constantes distintas no hay relación alguna.
- Selección por tiempo de consulta: no existe. Un experimento tiene semillas y
  un objetivo, y la clausura se calcula sobre todos los índices a la vez.
- Reconstrucción de momentos distintos: sí. La clausura contiene a la vez las
  consecuencias de todos los índices, con provenance (F1).

**Argumentos `PropositionTime`** (`effective`).

- Datos sin semántica. La vigencia se compara sólo mediante hechos de orden
  aportados (`notEarlier`).
- En LegalGeneral, `annulled` "añade una marca, jamás retracta el inferior"
  (comentario en `legal_general.res`, junto a la regla de la línea 239).

**Activación por programa.**

- Es el único mecanismo que condiciona la *activación* de reglas: las reglas
  miembro no se disparan como reglas ordinarias (`definitions`,
  `derive.rs:246,258,293`) y sólo existen instanciadas por disparadores.
- La activación es monótona: más hechos producen más activaciones.

**Tiempo jurídico frente a tiempo de aprendizaje.**

- El motor sólo conoce el índice proposicional, los argumentos temporales y
  el sello de derivación.
- El tiempo de aprendizaje, es decir la cronología de experiencias y de
  teorías, no tiene representación en el motor. Vive en el harness (ledger,
  secuencia de pilotos), salvo que se cosifique como argumento, como hace F4
  con las épocas.
- No son equivalentes. F4 y F8 muestran que requieren tratamientos distintos
  (§5.4). No requieren mecanismos del motor distintos: ambos se codifican con
  los mismos medios.

## 2. Formalización correspondiente al código

### 2.1 Términos, eventos, reglas

- 𝕋 = álgebra libre sobre las constantes K_time con un símbolo unario `after`.
  Los términos con variables sólo aparecen en patrones; los valores
  emparejados son ground (`bind_term`, `derive.rs:600`).
- Evento: e = (pol, v, ā, τ), con pol ∈ {+, ~} y τ ∈ 𝕋. El tipo de
  proposición no forma parte de la identidad (`typed.rs:433`).
- Regla r = (X, A₁ ∧ … ∧ Aₙ ⇒ B) con X sus parámetros (n = 0 es la unidad I).
  σ es un emparejamiento unilateral; σB debe ser ground.
- Operador inmediato para un conjunto de reglas R:
  Φ_R(Y) = Y ∪ { σB : ∀i σAᵢ ∈ Y }.
- Programas. Para un verbo v asociado a P con parámetros (x̄, t), cada evento
  positivo v(ā)@τ ∈ Y aporta las reglas P[x̄ := ā, t := τ], con el disparador
  como antecedente adicional. Entonces
  Ψ_T(Y) = Φ_{R ∪ ⋃_{v(ā)@τ ∈ Y} P[ā, τ]}(Y),
  donde R excluye los miembros de programas asociados (`derive.rs:293`).
- C_T(S) = mínimo punto fijo de Ψ_T por encima de S. Existe y es finito si el
  grafo de dependencias es acíclico; esa es la condición suficiente que exige
  el runner (`require_acyclic_with_edges`, `src/experiments.rs:238`).

### 2.2 Proposiciones

**Proposición 1 (monotonía).** Sean T = (R, P, asoc) y T′ = (R′, P, asoc) con
R ⊆ R′, mismas asociaciones y sin defaults. Si S ⊆ S′, entonces
C_T(S) ⊆ C_{T′}(S′).

*Prueba.* Ψ es monótona en Y: el emparejamiento sólo pide pertenencia, la
polaridad `~` es un literal más y las activaciones crecen con Y. También es
monótona en R. El punto fijo mínimo hereda la monotonía, por inducción sobre
las etapas Ψ^k. ∎ (DEMOSTRADO-BH; VERIFICADO-FINITO en F2: `removed = []`.)

**Corolario 1.** Ninguna adición de reglas ordinarias o de programas no
asociados hace que deje de derivarse una consecuencia, en ningún índice, ni
siquiera en los posteriores.

**Proposición 2 (captura).** Añadir una asociación `v = program P` cuyo
programa usa una regla ordinaria g existente retira g de R. Por tanto
C_{T ∪ {v, P, …}}(S) ⊉ C_T(S) en general.

*Prueba.* `prepare_programs` inserta los miembros en `definitions` y los
excluye de `ordinary` (`derive.rs:258,293`). Contraejemplo en F7: la
prohibición de rex en t0 desaparece. ∎ (IMPLEMENTADO; VERIFICADO-FINITO.)

**Proposición 3 (`after` no ordena).** Para constantes c ≠ c′, ningún patrón
construido sobre c empareja un evento indexado en c′. El patrón `after(x)`
sólo empareja términos con cabeza `after`, y la ligadura de x es única.
Ninguna regla puede expresar "en algún índice posterior a t" sin datos o sin
recursión.

*Prueba.* Por `bind_time` (`derive.rs:648`), que no tiene ninguna otra regla
de igualdad. ∎ (IMPLEMENTADO; VERIFICADO-FINITO en F1: `twoStepProbe(ana, rex)
@ t0` no se deriva aunque existe `reminded(ana, rex) @ after(after(t0))`.)

**Proposición 4 (selección positiva).** Para P asociado a v:
Active(P, S, ā, τ) ⟺ v(ā)@τ ∈ C_T(S). Las consecuencias de P sólo se producen
con esos bindings, y la provenance registra
`Programa::i::regla`, el disparador y la sustitución.

*Prueba:* por `activation_bindings` y `finish_program_match`. ∎
(IMPLEMENTADO; VERIFICADO-FINITO en F3.)

**Proposición 5 (defaults).** Si alguna sustitución empareja el bloqueador de
un default, el default se bloquea por nombre (`blocked_names`,
`defaults.rs:122-129`). Su lambda queda excluido para todas las instancias y
todos los índices de la ejecución. ∎ (IMPLEMENTADO; VERIFICADO-FINITO en F5.)

### 2.3 Correspondencia con la formulación propuesta

La hipótesis N(S, t, Q) = D_Q(Run_{T_{Active(S,t)}}(S)) tiene esta
correspondencia exacta con el código:

- **Active.** Existe como activación por programa (Proposición 4), pero
  depende de hechos de la clausura, no de un tiempo de consulta. "Activo en
  t" significa que existe el disparador indexado en t.
- **Run.** No hay ejecuciones parciales por conjunto activo. Una única
  clausura C_T(S) contiene la ejecución de todos los programas activados en
  todos los índices.
- **El tiempo de consulta es una proyección del harness.**
  N(S, τ, Q) = D_Q(π_τ C_T(S)), con π_τ la restricción a eventos de índice τ.
- **La época de aprendizaje es una semilla.** En el diseño de F4, una
  evaluación bajo la época e es
  N_e(S, Q) = D_Q(π^arg_e C_{T_n}(S ∪ {evaluatedAt(e)})),
  donde π^arg_e filtra por el argumento `effective = e`.

La formulación más fiel es, por tanto: **un único operador de clausura más
proyecciones**, no una familia de teorías activas. La familia (T_k) sólo
aparece en la secuencia de construcción de la teoría, que realiza el aprendiz.

### 2.4 Lema de la coordenada

Sea T ⊆ T′ en el sentido de la Proposición 1, y sea k una consecuencia con
k ∈ C_T(S). Si se exige k ∉ D(C_{T′}(S)) y D es una proyección monótona,
entonces es imposible. Por tanto, una revisión que haga cesar una consecuencia
mediante crecimiento de la teoría exige una de estas tres cosas:

1. que la consecuencia nueva tenga otra clave lógica, porque lleva la
   coordenada de la revisión: la época en F4, el índice jurídico en F8;
2. que D no sea monótona: negación por ausencia en el harness, como en el
   informe anterior, §2;
3. que T′ no extienda T: captura (F7, F8) o defaults.

*Prueba:* inmediata de la Proposición 1. ∎ (DEMOSTRADO-BH.)

## 3. Temporalidad frente a selección normativa

| Propiedad | Qué exige | Existente | Evidencia |
|---|---|---|---|
| A. Indexación | Consecuencias con tiempos distintos | Sí, siempre | F1 |
| B. Condiciones temporales | La aplicabilidad depende de relaciones temporales | Sólo igualdad de índice, un paso de `after` o datos de orden | F1, F6, F8 |
| C. Selección temporal | Entre varias generalizaciones, sólo algunas se activan para un momento | Sólo positiva: activación por disparador (programa) o guarda positiva | F3, F4, F8 |
| D. Persistencia histórica | Las derivaciones anteriores son recuperables con tiempo, regla y provenance | Sí en Horn y programas dentro de una ejecución; no con defaults ni con captura sin ruta de vigencia | F1, F4, F5, F7, F8 |
| E. Revisión normativa | Un cambio de tratamiento sin reescritura retroactiva | Sólo con la coordenada en la clave (F4) o con reemplazo más vigencia (F8) | F4, F8 |

Implicaciones demostradas:

1. **A ⇏ B.** F1 tiene indexación con una regla sin condición temporal.
2. **B ⇏ C.** Una condición temporal en una regla ordinaria restringe dónde
   produce, pero no selecciona entre generalizaciones: la otra regla sigue
   activa. En F6, `banAtT0` convive con `banTemplate`, que deriva en t1.
3. **C sin B.** F3 selecciona por una propiedad relacional sin ninguna
   condición temporal.
4. **C positiva ⇏ E.** Con selección sólo positiva, añadir G1 activa cosas
   nuevas pero no desactiva G0 (Proposición 1, F2).
5. **E ⇒ coordenada en la clave o T′ no aditiva** (Lema §2.4).
6. **Defaults: cese sin D.** Dan "cese" sin la coordenada y destruyen D (F5).
   Por tanto, no realizan E.
7. **F4 realiza A ∧ C ∧ D ∧ E** con teoría aditiva, bajo las hipótesis de
   §8.3.
8. **F8 realiza A ∧ B ∧ C ∧ D ∧ E** con un reemplazo de la activación de G0.

## 4. Composición con LegalGeneral

Todas las fixtures importan `legal_general.res` sin cambios y usan sus verbos
y reglas reales.

- **Prohibición y violación.** `prohibition` (sembrado, `:45`) producido por
  una regla de la fixture. Con `Act`, `prohibitedAct` (`:165`) deriva
  `violation`; `violationLiability` (`:189`) deriva `patrimonialLiability`; y
  `prohibitionCreatesAbstention` deriva `notDo(…) @ after t`. Todo en el mismo
  índice t que la prohibición (F1, F2, F7).
- **Derecho.** `right` (sembrado, `:43`) producido por G3 (F3, F4). Coexiste
  con `prohibition` del mismo sujeto, conducta, cosa y beneficiario. La
  ontología no tiene una regla de conflicto entre derecho y prohibición: las
  reglas de anulación cubren permiso/prohibición y obligación/prohibición
  (`:239` y siguientes). F4 `s3Dossier` lo muestra en e3.
- **Permiso y anulación.** G1 de F2 produce `permission` de autoridad superior
  con vigencia no anterior. `permissionAnnulsProhibition` deriva
  `annulled(dogBan, exemptA, legislative, enact1)`. **La marca no lleva
  sujeto** (`annulled(inferior, superior, authority, effective)`, `:76`), así
  que es el mismo hecho para rex y para fido. Una D que tratara `annulled`
  como pérdida de efecto excluiría también la prohibición de fido: la
  granularidad es global por identificador de norma, análoga al bloqueo por
  lambda (VERIFICADO-FINITO, F2 `twoDogs`). Además, `permission` excede la
  valoración parcial "Prohibition ∉ N", que no establece permiso.
- **`Act` y `~Act`.** Las reglas de violación y exigibilidad ya usan `~Act`
  como evidencia positiva (`:160`). F4, F7 y F8 usan el mismo patrón para
  `~propertyA`.
- **Configuraciones relacionales compartidas.** Las reglas de las fixtures
  comparten sujeto, conducta, cosa, beneficiario y autoridad mediante
  variables. Las generalizaciones se refieren a propiedades (`propertyA`,
  `serviceDog`, `allergicPresent`), nunca a identificadores. F4 `twoDogs` aplica
  la misma G0a a dos perros del mismo expediente con resultados distintos.
- **`give`, `do`, `notDo`.** Son efectos con transición `t → after t`. Aquí
  sólo aparecen como consecuencias derivadas (`notDo`). No se ejecuta State.

## 5. Caso de control: perros en un restaurante

Las valoraciones son datos hipotéticos del ejemplo. A, B y C son propiedades
expresables, no justificaciones.

### 5.1 Codificación (F4, `f4_t0` … `f4_t3`)

- **Épocas de aprendizaje.** e0 a e3 son constantes `PropositionTime` usadas
  como **argumentos**.
- **Tiempos jurídicos de los expedientes.** sA a sF son **índices**.
- **Evaluación de un expediente bajo una época.** Se siembra
  `evaluatedAt(e)`.
- **Teoría.** Cada generalización g se escribe desde T0 con la guarda
  `unrevisedThrough(g, e)`. Su consecuencia lleva `effective = e`.

  Esto reinterpreta `effective`, que en LegalGeneral es vigencia de la norma,
  como "época bajo la que se evaluó". Es una elección de la fixture, señalada
  aquí; un verbo propio evitaría la confusión.

| Teoría | Añade | Motivo |
|---|---|---|
| T0 | G0 `g0Ban`: perro → prohibición; certificado (g0, e0) | Regla general |
| T1 | G0a `g0aBan`: perro ∧ `~propertyA` → prohibición; certificado (g0a, e1). **No** añade (g0, e1) | ε₁: perro con A, Prohibition ∉ N |
| T2 | Certificado (g0a, e2) | ε₂: alérgico presente, Right ∉ N. Ninguna regla deriva `right`: la valoración ya se cumple y queda como restricción |
| T3 | G3 `g3ServiceRight`: perro ∧ `serviceDog` → derecho; certificados (g0a, e3) y (g3, e3) | ε₃: perro de servicio, Right ∈ N |

### 5.2 Resultados (`summary.json`, VERIFICADO-FINITO)

| Expediente | Bajo T3: consecuencias por época |
|---|---|
| `s1Dossier` (A, evaluado en e0, e1, e3) | e0: prohibición g0. e1 y e3: nada. ε₁ satisfecha; la evaluación e0 se conserva |
| `plainDog` (`~A`, e0 a e3) | e0: g0. e1, e2, e3: g0a |
| `unknownA` (ni A ni `~A`, e0 y e1) | e0: g0. e1: **nada** (indeterminado) |
| `s2Dossier` (`~A`, B; e2 y e3) | Prohibición g0a; ningún derecho. ε₂ sigue satisfecha en e3 |
| `s3Dossier` (`~A`, C; e2 y e3) | e2: g0a. e3: g0a **y** derecho g3. ε₃ satisfecha; prohibición y derecho coexisten |
| `s4Question` (`~A`, B y C; e3) | Prohibición g0a y derecho g3 |
| `twoDogs` (rex con A, fido con `~A`, e1) | Sólo fido prohibido |

Comprobaciones mecánicas (`analyse_runs.py`):

- inclusión de cierres T0 ⊆ T1 ⊆ T2 ⊆ T3 en los siete expedientes;
- para cada época e_j, sus consecuencias son idénticas en todas las teorías
  T_k con k ≥ j.

### 5.3 Lectura

- **No se memorizan expedientes.** Ninguna regla menciona rex, fido ni ningún
  expediente. G0a, G3 y las certificaciones son de clase.
- **Las valoraciones parciales no fuerzan posiciones.** ε₁ sólo exige que no
  haya prohibición en e1: no se deriva ningún permiso. ε₂ se cumple
  vacuamente en e2, pero restringe hipótesis futuras: cualquier G que derive
  `right` no debe cubrir S2.
- **S4 es la pregunta informativa.** Las hipótesis G3 y G3 ∧ ¬B son
  indistinguibles en ε₁–ε₃, pero dan resultados distintos en S4 (§8.4). Para
  expresar G3 ∧ ¬B haría falta evidencia positiva de ausencia de alérgicos,
  por ejemplo un hecho explícito por local y época; no se ha implementado
  (EXPRESABLE-NI).
- **La coexistencia derecho/prohibición en S3** no viola ε₃, que sólo exige el
  derecho. Si debe resolverse, es una decisión de D o de la ontología, no del
  tiempo.

### 5.4 Cronología de experiencias frente a tiempos jurídicos

- **F4 (épocas).** La revisión corrige el conocimiento de la teoría. La
  evaluación e0 de S1 (prohibido) queda como registro de lo que decía T0,
  aunque T1 la desautorice. La valoración ε₁ se trata como dato sobre todas
  las épocas posteriores.
- **F8 (vigencia).** La revisión es un cambio de derecho en tiempo jurídico.
  Un perro con A antes de rev1 sigue prohibido (`aBeforeRevision`), lo cual es
  correcto si hubo cambio normativo e incorrecto si ε₁ revela que G0 siempre
  fue errónea.
- **Elegir entre ambas lecturas** es decidir si una valoración pasada
  restringe todas las épocas (aprendizaje de una norma fija desconocida) o
  sólo su época (cambio normativo). Es una decisión del contrato, no un hecho
  del motor (INDETERMINADO para Residuum).

## 6. Verificaciones acotadas

Todas se ejecutaron con `scripts/test_safe.py --fixture … --forward` o
`--cli run|explain` (1 GiB, sin swap, un trabajo, un hilo). Las salidas,
`result.json` e identificadores de informe están en
`docs/legal-geometry/temporal-activation/runs/`. `analyse_runs.py` recomprueba
28 de 28 expectativas y las inclusiones y estabilidades de F2 y F4 sobre las
salidas archivadas (`summary.json`).

| Nº | Pregunta | Resultado | Fixture |
|---|---|---|---|
| 1 | ¿Una regla anterior sigue derivando en un momento posterior? | **Sí.** Sin ámbito temporal, deriva en t0, t1 y after(t0), con violación y responsabilidad en t1 | F1 `indexing` |
| 2 | ¿`after(t)` basta para seleccionar reglas? | **No.** Es un paso: `after(after(t0))` no empareja con `after t`, t = t0, y no relaciona constantes. Sí basta para adyacencia | F1 `afterSteps`, `indexing` |
| 3 | ¿Puede una regla nueva condicionar la activación de una anterior sin defaults, sustitución externa ni negación por ausencia? | **No** por adición de reglas (Proposición 1, F2). Sí por captura en programa (F7), que es un reemplazo de declaraciones. En F4, el "condicionamiento" lo hace la construcción de la teoría, no la regla nueva | F2, F7, F4 |
| 4 | ¿Una generalización parametrizada se aplica a varios sujetos? | **Sí**, por instancia ground | F3 `selection`, F4 `twoDogs`, F7 `twoDogs` |
| 5 | ¿Son recuperables las consecuencias y provenance anteriores? | **Sí** en Horn y programas, con regla, sustitución y disparador. **No** con defaults | F1, F3 y F6 (`explain`), F4, F5 |
| 6 | ¿Puede evidencia posterior modificar consecuencias en tiempos anteriores? | **Sólo añadir.** `retroFinding` deriva `incidentAtVisit @ t0` desde un informe en after(t0); nada se retira. Con defaults sí retira (F5) | F1 `retroLate`, F5 |
| 7 | ¿Depende el resultado de una teoría versionada externamente? | **No** en F4 ni en F8: una sola teoría y una sola ejecución reconstruyen todas las épocas o fechas. La secuencia T_k sólo existe como historia de construcción | F4, F8 |

Controles negativos incluidos:

- F3 `passive`: sin disparador no hay derecho.
- F5 `noBlocker`: sin A, prohibición en todas partes.
- F8 `historyWithoutOrder`: sin dato de orden, el expediente histórico no
  queda cubierto.
- F4 `unknownA`: sin `~A`, nada en e1.
- Rechazo del escenario polinómico por el runner (§10).

## 7. Comparación con defaults

| Capacidad | Defaults (`evaluate_defaults`) | Temporalidad existente sola (`@`, `after`) | Programas y datos de orden o certificados (F4, F8) |
|---|---|---|---|
| Evitar el bloqueo de todos los beneficiarios | No: bloqueo por lambda (F5: cae fido) | No aplica: no bloquea | **Sí**, por instancia, con `~A` explícita |
| Conservar consecuencias anteriores sin ejecuciones separadas | **No**: el bloqueador en t1 elimina t0 (F5) | Sí, pero no revisa | **Sí** (F4: una ejecución; F8: ruta de vigencia) |
| Reglas sucesivas con ámbitos temporales | No: el bloqueo no tiene índice | Sólo índices constantes o un paso | Sí: certificados por época (F4) o `precedes` (F8) |
| No monotonía | Respecto de la evidencia (F5) | Ninguna: monótona | Ninguna respecto de la evidencia; sólo respecto del índice de época o fecha (F4), o respecto de declaraciones (F7, F8) |
| Especializar una regla sin tocar el motor | Exception defaults; global | `rule r' = r(t0)` sólo añade (F6) | Captura en programa con disparador refinado (F7, F8) |
| Requiere evidencia negativa explícita | No: usa ausencia del bloqueador en la clausura obligatoria | — | **Sí** (`~A`) |

Ni los defaults son necesarios para la revisión por clases (F4, F8), ni la
temporalidad los reemplaza: `@` y `after` no producen cese. Lo que separa F4 y
F8 de los defaults es que su no monotonía está en la **coordenada** o en la
**construcción de la teoría**, no en la evidencia. Se paga con la exigencia de
evidencia negativa positiva.

## 8. Geometría y aprendizaje

### 8.1 Qué objeto resulta

- **Un único operador de clausura.** Para una teoría sin defaults, C_T es un
  único operador de clausura sobre 𝒫(E): extensivo, monótono e idempotente
  (Proposición 1 y punto fijo). E es el universo de eventos indexados,
  incluidos los que llevan época (IMPLEMENTADO).
- **La familia C_t es derivada.** π_τ ∘ C_T y π^arg_e ∘ C_T son proyecciones
  monótonas, no operadores de clausura: no son extensivas sobre 𝒫(E). No hay
  una familia de teorías activas en el motor.
- **La familia (T_k) es la trayectoria del aprendiz.** En F4 es una cadena por
  inclusión. En F7 y F8 no lo es: la captura retira una regla ordinaria.
- **La selección de reglas** es el conjunto de activaciones de programas
  inducido por C_T(S). Es un subproducto de la misma clausura, no un operador
  separado.

**Conclusión:** una combinación de un único cierre sobre eventos indexados,
proyecciones por índice o época, y una cadena de teorías construida fuera del
motor.

### 8.2 Propiedades conservadas

| Propiedad | F4 (cadena aditiva) | F7, F8 (captura) | Defaults |
|---|---|---|---|
| Monotonía de C en S | Sí | Sí, para T fija | No (F5) |
| Monotonía en la teoría | Sí (T_k ⊆ T_{k+1}) | No (Proposición 2) | No |
| Idempotencia | Sí | Sí | Sí por trozos (informe anterior, §9) |
| Fronteras mínimas de regiones ascendentes | Se conservan para C_{T_k} | Hay que recalcular tras cada captura | No (la región no es ascendente) |
| Antiintercambio | No se garantiza ni se conserva bajo adición: debe recertificarse (`legal-ontology-region-revision.md:192-222`) | Igual | No aplica |
| Equivalencias observacionales | Valen para cualquier D | Valen | Valen |

Las regiones definidas por valoraciones negativas ("Prohibition ∉ N_e") son
**descendentes** en S: añadir `~A` a un expediente con A puede hacer aparecer
la prohibición. No las genera una frontera mínima, sino sus elementos
maximales. No se ha estudiado su estructura (INDETERMINADO).

### 8.3 Estabilidad histórica (DEMOSTRADO-BH; VERIFICADO-FINITO en F4)

Hipótesis:

- **H1.** Toda regla que produce una consecuencia normativa lleva la guarda
  `unrevisedThrough(g, e)` y escribe e en la clave de la consecuencia.
- **H2.** T_k añade certificados sólo para e_k.
- **H3.** Las reglas añadidas en T_k no concluyen verbos que aparezcan en los
  antecedentes de generalizaciones anteriores. Es una estratificación: no
  pueden crear nuevos disparadores ni antecedentes para g de épocas
  anteriores.

Entonces, para j < k: π^arg_{e_j} C_{T_k}(S) = π^arg_{e_j} C_{T_j}(S).

*Prueba.* Toda derivación de una consecuencia de época e_j usa un certificado
(g, e_j). Por H2, ese certificado ya estaba en T_j, y g existía en T_j. Por
H3, los antecedentes de g en C_{T_k}(S) son los mismos que en C_{T_j}(S).
Las consecuencias de reglas nuevas tienen claves con épocas ≥ k (H1). ∎

Sin H3 la estabilidad falla. Por ejemplo, una regla nueva que derive
`accompanies` desde otro hecho crearía nuevas evaluaciones antiguas. No se ha
construido ese contraejemplo (EXPRESABLE-NI).

### 8.4 Particiones de hipótesis y espacio de versiones

- **Valoraciones.** Una valoración parcial (S_i, Q_i, restricción) define el
  conjunto de teorías candidatas cuya proyección en (S_i, e_i) la cumple.
  ε₁ es una restricción negativa, ε₂ negativa y ε₃ positiva.
- **Espacio de versiones.** Su intersección sobre las experiencias es el
  espacio de versiones, y la partición por perfiles de observación en un
  conjunto de preguntas organiza qué expediente preguntar después.
  - En F4, {G3, G3 ∧ ¬B} coinciden en ε₁–ε₃ y se separan en S4, que es la
    pregunta que maximiza la información entre ambas.
  - {G0a con `~A`, abandonar G0} coinciden en S1 y se separan en `plainDog`.
- **Alcance temporal de las valoraciones.** Sólo restringen las épocas que el
  contrato declare (§5.4). Si ε₂ vale sólo en e2, G3 no tiene que respetarla
  en e3.
- **Límites.** No se afirma convergencia a una justificación desconocida. Sin
  una clase de hipótesis finita fijada, un supuesto de realizabilidad y un
  orden de preguntas, no hay garantía. Las propiedades A, B y C sólo entran si
  son vocabulario declarado.

## 9. Consecuencias para el explorador LLM

### 9.1 Reparto

**Lo que puede hacer el motor (sin cambios):**

- programas activados por propiedades relacionales derivadas;
- guardas positivas por época o fecha;
- `~V` como evidencia;
- una clausura única con provenance por programa, regla, sustitución y
  disparador;
- adición de reglas candidatas, como en el piloto 003.

**Lo que tendría que hacer el harness o el contrato:**

1. Fijar la lectura temporal de las valoraciones: época de aprendizaje frente
   a vigencia jurídica (§5.4) y alcance de cada valoración.
2. Gestionar las coordenadas de revisión. En el diseño F4, los certificados
   `unrevisedThrough` son decisiones del aprendiz o del harness, y el LLM
   propondría "no extender g en e_k" como acción tipada distinta de proponer
   una regla. En el diseño F8, el contrato debe admitir la **captura** de una
   regla existente en un programa, que hoy no es una acción del explorador:
   éste sólo propone reglas Horn añadidas.
3. Reevaluar el panel histórico completo tras cada propuesta: todas las
   experiencias, en todas las épocas. Comprobar inclusión y estabilidad
   (§8.3) con el mismo tipo de verificación que `analyse_runs.py`.
4. Exigir y suministrar evidencia negativa explícita (`~A`) en los
   expedientes. Su ausencia produce indeterminación, que debe distinguirse de
   "no prohibido" en D.
5. No dar cabezas ni bloqueos predeterminados. El LLM recibe el vocabulario,
   el panel y las valoraciones, y propone generalizaciones sobre propiedades
   tipadas. Residuum calcula Cov por época.

**Límite actual del trabajador.**

- El trabajador del explorador exige programas acíclicos
  (`require_program_acyclic` en `src/experiments/worker.rs:616` y
  `proposal.rs:209` de la rama del spec), igual que el runner de fixtures.
- Ninguno de los dos admite defaults en sus cierres.
- Las guardas de orden derivadas por recursión de Peano no pueden entrar
  (§10).

**Problemas de investigación abiertos:**

- elegir entre refinamientos indistinguibles en el panel;
- obtener la completitud de la evidencia negativa;
- decidir si hace falta la derrota por instancia e índice (§12).

### 9.2 Lo que no cambia

La evaluación de un candidato sigue siendo C_{T ∪ r}(S), como en la etapa I
del piloto 003. Lo nuevo es que la teoría deja de crecer sólo por reglas, que
la cobertura se mide por época y que el panel histórico forma parte de cada
evaluación.

## 10. Polinomios (aritmética de Peano sobre `after`)

### 10.1 Qué hay

`docs/peano-after-experiment.md`, `peano-multiplication-and-polynomial-boundary.md`,
`peano-polynomial-addition-experiment.md`, `peano-polynomial-product-experiment.md`
y `temporal-constructors-experiment.md`:

- `zeroTime` como Z y `after` como S.
- Suma, producto, suma de polinomios densos y producto (convolución) se
  calculan con programas recursivos y `close_program`. No hay primitiva
  numérica ni tabla de resultados; Rust sólo verifica.
- El índice `@ t` transporta el acumulador. `after(·)` como término permite
  pasar valores construidos a argumentos.

Verificación aquí (informe `20261015T233827548351Z-2940467`, archivado en
`runs/peano-tests/`). Pasan individualmente:

- `polynomial_principal_computes_four_coefficients_from_requests_and_input_only`
- `polynomial_missing_input_is_not_zero_and_absent_request_does_not_activate`
- `polynomial_product_computes_complete_dense_convolution_on_the_verified_domain`

Control negativo: `--fixture examples/experiments/peano_polynomial_scenario.res
--forward` falla con "residual fixture runner supports acyclic rule
dependencies only" (`runs/peano_polynomial_scenario-rejected/`).

### 10.2 Qué aportan a la cuestión temporal

1. **`after` es aritmética, no tiempo.** Los polinomios muestran que 𝕋 es el
   modelo libre de Peano sobre las constantes temporales, y que sobre las
   espinas con raíz `zeroTime` las reglas Horn computan funciones
   aritméticas. Nada de eso da orden entre constantes distintas
   (Proposición 3). Una fecha `t1` no es `after^n(t0)` salvo que se construya
   así.
2. **El orden puede derivarse, pero sólo con recursión.** Una relación
   `precedes(a, b)` sobre numerales unarios se puede escribir como la suma
   (existe c tal que a + c = b) o por descenso estructural simultáneo. Sobre
   espinas finitas y bien formadas, el descenso permitiría derivar
   *positivamente* ambas direcciones, incluida la estricta, sin negación
   (EXPRESABLE-NI). Así, las guardas de vigencia de F8 podrían calcularse en
   vez de suministrarse. Pero:
   - el runner de fixtures y el trabajador del explorador rechazan la
     recursión, así que hoy sólo es posible desde la API (`close_program` en
     tests);
   - exige que las fechas se codifiquen como numerales unarios, que no es la
     codificación de LegalGeneral: ésta usa constantes simbólicas más
     `notEarlier`, y su comentario dice expresamente "no se presupone un orden
     de calendario ni que after sea sucesor discreto".
3. **Ausencia no es cero.** La suma de polinomios exige un coeficiente
   explícito por exponente del dominio declarado, y un coeficiente ausente no
   produce resultado (test `polynomial_missing_input_is_not_zero…`). Es
   exactamente la condición 5(b) de §0: G0a sólo decide cuando hay `~A`, y
   `unknownA` queda sin resultado. En ambos casos el lenguaje evita la
   negación por ausencia haciendo densa la evidencia, a costa de la
   indeterminación cuando falta.
4. **La activación por petición es el patrón de selección.** En los
   polinomios ninguna definición actúa sin un hecho de petición
   (`addPolynomial`, "absent request does not activate"). Es la Proposición 4
   aplicada a la aritmética, el mismo mecanismo que F3 y F7 usan para
   generalizaciones jurídicas.
5. **`DerivationTime` no es tiempo de aprendizaje.** Los informes de
   polinomios lo comparan sólo dentro de una ejecución. No sirve para ordenar
   teorías.

### 10.3 Geometría

Con recursión, el universo de eventos alcanzables es infinito (𝕋 lo es) y la
finitud de C_T(S) depende de precondiciones sobre los datos: espinas finitas,
acíclicas y sin ramas (documentadas en los informes de Peano). La clausura
sigue siendo monótona e idempotente cuando existe, pero las afirmaciones de
fronteras y de enumeración finita de este informe y del anterior suponen
dominios acíclicos y finitos.

## 11. Matriz de correspondencia

| Concepto | Código | Prueba o fixture | Estado |
|---|---|---|---|
| Índice proposicional | `TimeExpr`, `typed.rs:504`; `logical_key`, `:433` | F1 | IMPLEMENTADO |
| Sucesor sin orden | `bind_time`, `derive.rs:648` | F1 `afterSteps`; `legal_abduction_tests.rs` ("after(after(tau)) cannot collapse") | IMPLEMENTADO; VERIFICADO-FINITO |
| Regla sin ámbito temporal | `close_with_programs`, `derive.rs:374` | F1 `indexing` | VERIFICADO-FINITO |
| Regla con constante temporal | `check_rule_event`, `elab.rs:964` | F6 `twoTimes` | VERIFICADO-FINITO |
| La especialización añade, no restringe | Aplicación de regla, `elab.rs:808` | F6 `specializedAtT0` (dos justificaciones) | VERIFICADO-FINITO |
| Monotonía de la clausura (Proposición 1) | `derive.rs:374-480` | F2 (`removed = []`) | DEMOSTRADO-BH; VERIFICADO-FINITO |
| Captura no monótona (Proposición 2) | `prepare_programs`, `derive.rs:246-293` | F7 `history` | IMPLEMENTADO; VERIFICADO-FINITO |
| Selección positiva por instancia (Proposición 4) | `activation_bindings`, `:341`; `finish_program_match`, `:361` | F3 | VERIFICADO-FINITO |
| Disparador por propiedad relacional | Regla `detectService` más `Verb.program` | F3, F7 | VERIFICADO-FINITO |
| Vigencia por dato de orden | `notEarlier` (`legal_general.res:53`); `precedes` en la fixture | F8 | VERIFICADO-FINITO |
| Teoría por épocas, aditiva, con historia estable | F4 más `analyse_runs.py` | `summary.json` (`inclusion`, `history`) | VERIFICADO-FINITO; DEMOSTRADO-BH (§8.3) |
| Evidencia negativa explícita | `Polarity::EvidentialNot`; `elab.rs:789` | F4 `unknownA` frente a `plainDog` | VERIFICADO-FINITO |
| `annulled` sin sujeto | `legal_general.res:76,239` | F2 `twoDogs` | VERIFICADO-FINITO |
| Coexistencia derecho/prohibición | Falta de regla de conflicto en LegalGeneral | F4 `s3Dossier` | VERIFICADO-FINITO |
| Defaults: bloqueo global en lambda e índices | `evaluate_defaults`, `defaults.rs:69-129` | F5 | VERIFICADO-FINITO |
| Adición retroactiva, nunca retirada | Regla de `after t` a `t` | F1 `retroLate` | VERIFICADO-FINITO |
| Sin tiempo de consulta ni de aprendizaje en el motor | `run_forward_experiments`, `experiments.rs:140`; `incremental.rs:25` | Inspección | IMPLEMENTADO (ausencia) |
| Aritmética de Peano | Fixtures `peano_*`; tests de polinomios | 3 tests (§10.1) | VERIFICADO |
| Orden derivado por recursión | — | — | EXPRESABLE-NI |
| Recursión rechazada en el runner y el trabajador | `experiments.rs:238`; `worker.rs:616` (spec) | Escenario polinómico rechazado | IMPLEMENTADO |
| Derrota por instancia e índice sin evidencia negativa | — | Proposiciones 1, 2 y 5; F2, F5, F7 | NO-REPRESENTABLE con los operadores actuales |
| Lectura temporal de las valoraciones | — | — | INDETERMINADO (decisión de contrato) |

## 12. Capacidades, límites y obligaciones pendientes

**Capacidades existentes:**

- indexación;
- adyacencia de un paso;
- reglas fijadas a constantes;
- selección positiva por instancia mediante programas;
- vigencia por datos de orden;
- evidencia negativa explícita;
- provenance completa;
- reconstrucción histórica en una sola ejecución bajo F4 o F8.

**Límites expresivos:**

- No hay orden temporal ni intervalos nativos.
- No hay tiempo de consulta ni de aprendizaje.
- Ninguna regla suspende otra (Proposición 1).
- La captura por programa es global por regla (F7).
- Los defaults son globales por lambda e índice (F5).
- La anulación de LegalGeneral es global por identificador de norma (F2).
- El orden derivado requiere recursión, prohibida en el runner y el
  trabajador.

**Operación mínima que falta.** Sólo la exige quien rechace las condiciones
5(a) y 5(b). Es la derrota de una instancia ground de una generalización
anterior por una posterior, acotada a un índice o época, decidida dentro de la
teoría y sin evidencia negativa explícita.

Las Proposiciones 1, 2 y 5 muestran que no la dan:

- las reglas ni los programas (monótonos);
- la captura (global por regla);
- los defaults (globales por lambda e índice).

La Proposición 3 muestra que tampoco la da `after`. No se diseña aquí.

**Obligaciones pendientes, por dependencia:**

1. Decidir la lectura temporal de las valoraciones: aprendizaje de norma fija
   o cambio normativo, y su alcance (§5.4). Es decisión del usuario.
2. Decidir si se aceptan las condiciones 5(a) y 5(b). Si no, la obligación 7
   pasa a ser necesaria. Es decisión del usuario.
3. Fijar D por época con tres valores: consecuencia, exclusión e
   indeterminado. Así se distingue `unknownA` de "no prohibido".
4. Definir el lenguaje de transformación del explorador: regla, certificado y
   captura como acciones tipadas distintas (§9.1).
5. Reevaluación del panel histórico y comprobación de H1–H3 en el harness.
6. Verificar el contraejemplo sin H3 (§8.3) y estudiar las regiones
   descendentes (§8.2).
7. Sólo si la obligación 2 lo exige: caracterizar formalmente la derrota por
   instancia e índice antes de cualquier diseño.
8. Opcional: orden de Peano derivado en la API, si se acepta la codificación
   unaria de fechas. Exigiría relajar la guarda de aciclicidad del trabajador,
   con un criterio de terminación propio.

## Archivos

- `docs/legal-geometry/temporal-activation/fixtures/`:
  - `f1_indexing.res`
  - `f2_base.res`, `f2_extended.res`
  - `f3_program_activation.res`
  - `f4_t0.res` … `f4_t3.res`, generados con el mismo panel
  - `f5_default_temporal.res`
  - `f6_constant_time.res`
  - `f7_wrapped_generalization.res`
  - `f8_wrapped_with_vigencia.res`
- `runs/<fixture>[-explain|-run]/`: `output.txt`, `result.json`,
  `report-id.txt`; además `peano-tests/` y `peano_polynomial_scenario-rejected/`.
- `gen_f4.py`: regenera `f4_t0.res` … `f4_t3.res` byte a byte.
- `analyse_runs.py` y `summary.json`: `python3 -B analyse_runs.py` sale con 0
  si se cumplen las 28 expectativas, la inclusión de F2 y F4 y la estabilidad
  por época.
