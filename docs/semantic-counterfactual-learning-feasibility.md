# Aprendizaje por recuperación semántica y contraste contrafáctico: viabilidad y arquitectura mínima

Estado: **INFORME DE INVESTIGACIÓN. No se implementó nada.** No se modificó
código, motor, LegalGeneral, contratos congelados ni registros anteriores. No
hubo llamadas pagadas. La única ejecución es una simulación local con el
worker existente, dentro del runner con límites (§6). Sus archivos están en
`docs/legal-geometry/semantic-counterfactual-feasibility/simulation/`.

## 0. Respuesta corta

**Sí, en su núcleo lógico.**

**Ya existe, y es suficiente:**
- **Admitir y ejecutar formalizaciones**: reglas Formal sobre O.
- **Ejecutar mundos contrafácticos aislados**: cada mundo es un conjunto de
  semillas declarado.
- **Calcular diferencias de consecuencias y soportes mínimos exhaustivos.**
- **Encontrar regiones discriminantes entre hipótesis.**
- **Separar la base efectiva** `B = O ∪ (R_k ∖ {h})`.
- **Conservar hipótesis, revisiones, descartes y sus razones** dentro de una
  trayectoria.

No hace falta ningún mecanismo lógico nuevo. La monotonía de LegalGeneral,
más el cálculo de fronteras ya certificado, responde de una vez a todos los
contrafactuales que se formen dentro de un pool de hechos (§4.4).

**No existe en código:**
- descripciones semánticas escritas por el LLM;
- embeddings y recuperación;
- revisión de descripciones;
- una memoria que cruce trayectorias;
- un operador que proponga mundos `S'`;
- una clasificación de insuficiencias.

Todo eso está especificado en contratos (Stage 0, contrato de revisión
semántica, Piloto 004), pero no tiene código.

**Es pequeño como ingeniería** y no toca el motor. Son adaptadores en el
driver y el puente, un almacén de experiencias y un cliente de embeddings.

**Requiere decisiones conceptuales antes de construirlo** (§8). Las tres que
condicionan todo lo demás:
1. Quién formaliza los hechos de un expediente nuevo.
2. Qué intervenciones sobre hechos son admisibles, incluidas las
   dependencias entre semillas.
3. Qué cuenta como insuficiencia "representacional" frente a "deductiva".

## 1. Hipótesis arquitectónica y objetos que hay que mantener separados

| Objeto | Dónde vive hoy | Quién lo produce | ¿Puede cambiar el aprendizaje? |
|---|---|---|---|
| Expediente y sus documentos | `pilot-004/documentary/sources/` (html, pdf, txt y sha256) | fuente externa | no |
| Hechos establecidos `S` (Formal) | `pilot-004r/dossiers.res`: un bloque `experiment … seeds {}` | **nosotros** (curador), con reservas documentadas | no en 004-R (§8, D1) |
| Descripción determinista `d0` | catálogo del Piloto 004 v2 (`frozen/descriptions.tsv`), renderizado por plantilla y certificado | renderizador | no (función de la estructura) |
| Descripción semántica `h` | inexistente (solo contrato) | LLM | sí, con versiones |
| Ontología fija `O` | `legal_general.res` | fija | **nunca** |
| Reglas aprendidas `R_k` | `HypothesisRecord` en `generative.rs` | LLM, admitidas por Residuum | sí: commit, retiro, descarte |
| Consecuencias `C_T(S)` | clausura del motor | Residuum | derivadas, no editables |
| Valoraciones externas | `pilot-004r/reserved/valuations.tsv` | humanos | no; el LLM solo ve discrepancias tras el scoring |
| Mundos contrafácticos `S'` | `contrast/counterfactual_pool.res` (a mano), variantes Riggs del Piloto 003, simulación de §6 | hoy: nosotros | se construyen para investigar y **nunca** reciben valoración implícita |

Dos confusiones a evitar:
- **"Formalización" significa dos cosas.** Puede ser la representación
  Formal de los hechos (`S`) o las reglas (`R_k`). En 004-R el LLM solo
  produjo reglas; los hechos los fijamos nosotros. El paso 3 del ciclo
  ("adaptar una formalización recuperada") reutiliza reglas. El paso 5
  ("insuficiencia representacional") puede pedir cambiar hechos. Son
  operaciones distintas con riesgos distintos (§8, D1).
- **Las semillas son evidencia, no estado del mundo.** `~Act` es
  `EvidentialNot`: evidencia documentada de que algo no ocurrió, no la
  ausencia de `Act`. Esto importa para los contrafactuales (§4.2).

## 2. Inventario de capacidades

Clasificación:
- **C**: implementada y certificada por tests.
- **E**: implementada y ejercitada en corridas archivadas.
- **D**: disponible pero no ejercitada por un LLM.
- **S**: solo especificada.
- **∅**: inexistente.

| Operación | Estado | Evidencia |
|---|---|---|
| Expedientes como bloques `experiment … seeds` tipados; solo verbos `seeded` | C, E | `src/elab.rs:361-366`; `dossiers.res`; manifiesto del worker (`dossiers` en JSON) |
| Admisión de hipótesis (perfil `L_adm`, alcance, aciclicidad) | C, E | `proposal.rs` (`check_scope` :340); `profile_004r()` (`generative.rs:1123`); runs 001-013 |
| Evaluación certificada contra base `B` y atribución | C, E | `State::evaluate`, `certify_attributed` (`generative.rs`); flag `attribution` (run-011 en adelante) |
| Evaluación certificada del worker (O ∪ un candidato) | C, E | `worker.rs:840`. **Límite observado en §6:** rechaza con `out_of_certificate_scope` un candidato cuya cabeza ya aparece en la clausura de referencia |
| Clausura hacia adelante con teoría O u O ∪ {c} | C, E | `BeginForward{dossier, theory}` (`worker.rs:611`) |
| Clausura incremental y diferencia **aditiva** | C | `src/derive/incremental.rs`, `program_incremental.rs`; `docs/incremental-close.md`. Eliminar o reemplazar hechos: **pendiente** (`discrete-derivative.md:40-46`) |
| Historia temporal `H_n = O ∪ ⋃ I_tk(R_k)` y proyección | C | `temporal_history_tests.rs` (8 tests). Es arnés de test, no hay motor temporal |
| Procedencia de reglas, commit traces | C, E | `rule_origins` (:1384), `CommitTrace` (:916), `install(..., origin)` |
| Consultas residuales (abducción **aditiva** de semillas) | C, E | `src/residual/{program,active}.rs`; `begin_residual` en runs 008-013 |
| Soportes mínimos exhaustivos | C, E | `exploration::frontiers` (:417); `goal_supports` en `generative.rs` con hojas = semillas del expediente y comprobación `checked` (run-012, run-013) |
| Comparación de particiones, discriminantes, `covers` | C, E (pocas veces) | `worker.rs:986`; `exploration.rs:500-626`; `contrast/divergence-map.md` |
| Mapas de divergencia sobre un pool contrafáctico | C (test) | `pilot004r_contrast_tests.rs`; `contrast/counterfactual_pool.res` |
| Familias de mundos variantes como expedientes | C, E | Piloto 003: seis expedientes Riggs (`historicalRiggs`, `killingByThirdParty`, `killingWithoutAcquisitionPurpose`…) y evaluación offline `candidate_evaluation.py` |
| Claims verificables | C, E con defecto conocido | `claims.py`. `contains_event` compara solo el verbo y marca como `mismatch` claims verdaderos (closing-report §6.1). Hay que corregirlo antes de reutilizarlo |
| Registro de hipótesis, revisiones (`parent`), descartes y hallazgos | C, E | `HypothesisRecord`; `turns.jsonl`; `trajectory-report.json` |
| Memoria **entre** trayectorias | ∅ | cada corrida abre un ledger nuevo; nada lee corridas previas |
| Adaptadores del LLM (Anthropic) y explorador | C, E | `tools/normative_explorer/` (pilotos 001-003); `tools/pilot004r/driver.py` (runs 001-013) |
| `d0` determinista (render de plantilla) | C | `generate_catalog.py:90-117`; `pilot004_tests.rs:230,454`. Rinde candidatos del catálogo; no rinde expedientes ni experiencias |
| Render canónico de eventos | C | `generative::formal_event` (run-012) |
| Identidad canónica `id(Γ)=SHA256(…)` | S | Stage 0 |
| Descripciones semánticas `h` del LLM | S | `legal-semantic-retrieval-stage0.md` §2; `legal-semantic-revision-contract.md` |
| Embeddings, caché, `retrieve(z,k)` | S | Stage 0 §6 ("nuevo; proveedor ABIERTO") |
| Navegación por coseno | S (congelada, nunca ejecutada) | `pilot-004/frozen/protocol.tsv` |
| Revisión de descripciones (condiciones C y D) | S | contrato de revisión semántica §7-8 |
| Operador que proponga `S'` desde `S` | ∅ | ni el worker ni el puente aceptan semillas nuevas en tiempo de ejecución |
| Chequeo de consistencia de semillas | ∅ por diseño | `Act ∧ ~Act` solo deriva la marca `inconsistent` (`legal_general.res:178`) |
| Clasificación de insuficiencias | ∅ | existen sus ingredientes (§5.3), no la clasificación |

## 3. Por qué el ciclo encaja con lo que ya existe

La arquitectura de 004-R ya separa lo que el ciclo necesita separar:
- el LLM propone texto;
- Residuum admite, cierra, atribuye y certifica;
- las valoraciones solo vuelven como discrepancias después del scoring
  prequencial.

El ciclo propuesto añade dos cosas a esa separación:
1. **Entrada recuperada.** El LLM recibe, además del expediente, registros
   de experiencias previas.
2. **Investigación contrafáctica.** El LLM puede pedir que se ejecuten
   mundos `S'`, no solo reglas `h`.

Ninguna de las dos toca la semántica del motor.
- La primera es contexto en la vista del aprendiz.
- La segunda pide más expedientes sobre la misma teoría, y el worker y el
  arnés ya trabajan con varios expedientes.

## 4. Contrafactuales

### 4.1 Representación de una intervención

Una intervención admisible es una pareja finita de eventos sembrados:

```
δ = (D, A)        S' = (S ∖ D) ∪ A
```

donde `D ⊆ S` y `A` está formado por eventos tipados que cumplen:
- usan verbos `seeded` de O;
- usan constantes ya declaradas, o nuevas constantes de sortes existentes,
  como `actJ : Fact` en `counterfactual_pool.res`;
- tienen polaridad positiva o, solo para `Act`, `EvidentialNot`.

`S'` es un expediente más. No hace falta un operador de borrado en el motor:
`S'` se declara completo y se cierra desde cero. Así se hizo en el Piloto 003
(seis variantes Riggs) y en la simulación de §6. La clausura incremental
solo sabe añadir. Eliminar y reemplazar quedan resueltos porque se declara el
mundo, no porque se edite.

### 4.2 Tres intervenciones que no son equivalentes

| Intervención | Forma | Lectura |
|---|---|---|
| **Añadir evidencia** | `A ≠ ∅, D = ∅` | "si además constara X" |
| **Retirar evidencia** | `D ≠ ∅, A = ∅` | "si no constara X" (ignorancia, no negación) |
| **Evidencia contraria** | `D = {Act(...)}, A = {~Act(...)}` | "si constara que X no ocurrió" |

LegalGeneral no lee la ausencia en ninguna regla: no hay negación por
fallo. Por eso retirar un `Act` nunca produce `violation`, y solo `~Act`
puede producirla. La simulación lo muestra:
- `cf_nonperf` deriva una `violation` únicamente cuando además existe la
  obligación escrita;
- `cf_no_agreeU` solo pierde consecuencias.

### 4.3 Conservar el resto, evitar mundos inadmisibles

**Ceteris paribus sintáctico.** `S ∖ D` se conserva literalmente, y el
registro de la intervención es `(D, A)`. Es todo lo que el motor puede
garantizar.

**Admisibilidad que ya está garantizada:**
- tipado y verbos `seeded` (elaboración);
- perfil de reglas (sin cambio).

**Admisibilidad que no está garantizada, y requiere decisión** (§8, D2):
- **Contradicción explícita.** `Act` y `~Act` del mismo acto en `S'`
  derivan `inconsistent`. Es una marca y no un rechazo, de modo que el mundo
  se ejecuta igual. Habría que decidir si esos mundos se rechazan o se
  estudian.
- **Dependencias entre semillas.** El acuerdo `Act(saCF_c, …, agree, …)` y
  una `obligationFiled(saCF_c, …)` vienen del mismo documento. Retirar el
  acuerdo y conservar la obligación escrita produce un mundo bien tipado,
  pero quizá documentalmente imposible. O no expresa esas dependencias, y
  ampliarla para ello está fuera de alcance. La alternativa es declarar
  grupos de semillas que solo se intervienen juntos: un metadato del
  expediente, no lógica.
- **Sembrar posiciones normativas.** `right`, `prohibition` y
  `omissionProhibition` son `seeded` en LegalGeneral, así que un `S'` podría
  sembrar directamente la posición que se investiga. Eso cortocircuita la
  teoría y produce soportes triviales (ver la advertencia de
  `divergence-map.md` sobre "seeded right"). Política recomendada: las
  intervenciones solo tocan `Act`, `~Act` y los documentos (`…Filed`), no
  las posiciones proyectadas.

### 4.4 Ejecución aislada y comparación, sin un mecanismo nuevo

**Aislamiento.** Cada mundo es una entrada distinta en el mapa `dossiers`
del worker, o un `experiment` en un módulo Formal. Cada sesión de clausura
es independiente.

**Comparación de consecuencias.** `C_T(S') ∖ C_T(S)` y `C_T(S) ∖ C_T(S')`
son diferencias de conjuntos sobre las clausuras que el worker ya devuelve.
No hay función de biblioteca que lo haga, pero son dos líneas de adaptador
(`analyze.py`).

**Comparación de soportes: el resultado clave.** Sea `T` monótona (sin
defaults ni programas, que es el caso de LegalGeneral más reglas Horn) y sea
`P` un pool de semillas.

- **Dentro del pool.** Para todo `S' ⊆ P` y todo evento `e`, se cumple
  `e ∈ C_T(S')` si y solo si existe un soporte mínimo `σ` de `e` sobre `P`
  con `σ ⊆ S'`.
- **Consecuencia práctica.** Calcular `frontiers` una vez sobre `P` (ya
  certificado y exhaustivo, run-012 README) responde por inclusión a los
  `2^|P|` mundos contenidos en el pool, sin cerrar ninguno.
- **Qué intervenciones cubre.** Retirar es tomar un subconjunto. Reemplazar
  `Act` por `~Act` es tomar un subconjunto de un pool que contiene ambos.
- **Precedente.** Es exactamente lo que hizo el mapa de divergencia h1/h2.
- **Límite.** Con defaults (Riggs) `F_T` deja de ser monótona. Hay un
  contraejemplo en `legal-normative-transformations.md` §9.3, y `frontiers`
  rechaza esos módulos.

**Regiones discriminantes.** `compare_partition(h, h', goal)` devuelve la
partición del pool por derivabilidad bajo cada hipótesis y las regiones que
las separan. Está ejercitado en §6.

**Precaución heredada del worker.** Sus hojas son todos los eventos de
verbos `seeded` del universo conjunto, incluidas instancias derivadas como
`right(cfA1, …)`. Aparecen como regiones espurias en §6. El arnés de 004-R ya
corrigió esto en `goal_supports`, restringiendo las hojas a las semillas
reales. La versión a reutilizar es esa.

### 4.5 Diferencias de hechos, de formalización y de teoría

Con dos mundos (`S`, `S'`), dos hipótesis (`h`, `h'`) y una base `B` fija,
las cuatro clausuras `C_{B∪x}(y)` forman un diseño 2×2:

- **Diferencia de hechos:** `C_{B∪h}(S') △ C_{B∪h}(S)`.
- **Diferencia de formalización:** `C_{B∪h'}(S) △ C_{B∪h}(S)`.
- **Interacción:** lo que cambia con el mundo bajo `h` y no bajo `h'`. Es
  exactamente lo que `compare_partition` resume como región discriminante.
- **Diferencia de teoría:** cambiar `B` (commits). La evaluación con base
  explícita (run-011) ya la mantiene separada.

### 4.6 Qué se puede usar sin valoraciones reservadas

Todo lo anterior: clausuras, diferencias, soportes, particiones y
`inconsistent`. Ninguna de esas operaciones lee `reserved/`.

Las valoraciones solo entran por el scorer prequencial, después del commit,
y sobre expedientes documentados. Un mundo `S'` no tiene valoración. Darle
una sería una valoración humana nueva, y el sistema no debe inferirla de la
del expediente real.

### 4.7 Cuándo un contraste sería evidencia causal

Una diferencia `e ∈ C_T(S') ∖ C_T(S)` prueba **dependencia contrafáctica
dentro de T**, y nada más.

- **Lo que sí prueba.** Bajo T, la diferencia `(D, A)` basta para que se
  derive `e`.
- **Los soportes mínimos son su forma exacta.** Cada soporte mínimo es un
  conjunto suficiente en el que cada elemento es necesario: la estructura
  INUS de Mackie, relativa a T.

Para leerla como evidencia causal **sobre el derecho o los hechos** harían
falta cinco condiciones, y hoy no se cumple ninguna por construcción:

1. **T como modelo estructural aceptado.** La corrección de T la deciden las
   valoraciones externas, no la clausura. Un contraste bajo una hipótesis no
   confirmada solo dice qué implicaría esa hipótesis.
2. **Semillas exógenas e intervenibles por separado.** Esto falla con
   semillas que dependen del mismo documento (§4.3), y obliga a declarar las
   dependencias.
3. **Modularidad.** La intervención no cambia reglas. Esto sí se cumple: `T`
   queda fija mientras se comparan mundos.
4. **Semántica de la intervención.** Retirar evidencia no equivale a afirmar
   que algo no ocurrió (§4.2). Una lectura causal de "si no hubiera ocurrido
   X" exige `~X` y además que `~X` sea admisible.
5. **Dominio de la causalidad.** Lo que se contrasta es la dependencia de
   posiciones normativas respecto de hechos documentados: una relación de
   fundamentación jurídica. No es causación física entre hechos. Cuando la
   causación es un hecho del caso, como `deathCausedBy` en Riggs, entra como
   semilla y no se descubre.

Conclusión: los contrastes pueden **orientar** la investigación y
**justificar** revisiones relativas a T. No pueden presentarse como
descubrimiento causal sin las condiciones 1, 2 y 4.

## 5. Recuperación semántica y memoria

### 5.1 La unidad recuperable

| Unidad | Qué aporta | Qué se pierde o confunde |
|---|---|---|
| Descripción del expediente | encuentra casos parecidos en hechos | no dice qué formalización funcionó ni por qué. Confunde parecido narrativo con parecido estructural: Brookside y Casa Fiesta son ambos acuerdos ADA, pero sus semillas no tienen verbos en común aparte de los dos `agree` |
| Descripción de la formalización | encuentra reglas parecidas | pierde el contexto de aplicación. Dos reglas iguales pueden haber fracasado en un caso y funcionado en otro |
| Descripción conjunta | ambas cosas | mezcla en un vector dos señales que deberían ponderarse por separado. Una revisión de la parte interpretativa mueve también la parte de hechos |
| **Registro de experiencia** (expediente, formalización, consecuencias, contrastes, revisiones, resultado) | lo que hace falta para reutilizar y para desconfiar | es la unidad correcta para **almacenar**, pero no tiene un único texto que embeber |

**Recomendación:**
- **Unidad almacenada**: el registro de experiencia, inmutable y
  direccionado por contenido (identidad canónica de Stage 0).
- **Vistas embebidas**: varias por registro, con índices separados:
  `z_hechos` sobre la descripción del expediente y `z_form` sobre la
  descripción de la formalización con su resultado. La recuperación
  devuelve registros, nunca vectores sueltos.

Así se puede decidir después (D4) si se recupera por hechos, por
formalización o por ambos, sin rehacer la memoria.

### 5.2 Descripción determinista frente a interpretación revisable

Hay dos componentes, como ya fijaba el contrato del Piloto 004 (`d0`, luego
`h_k`):
- **`d0` estructural.** Función pura del Formal: semillas, reglas, conteos de
  la clausura y verbos leídos y escritos por cada regla. Se renderiza con
  `formal_event` o con una plantilla como la de `generate_catalog.py`. Es
  reproducible, no se revisa y cambia solo si cambia el Formal.
- **`h` interpretativa.** Texto del LLM con versión, autor y claims
  auditables con procedencia (`formal | consecuencia | valoración |
  hipótesis`).

Revisar `h` crea una nueva versión, un nuevo `z` y una recuperación
distinta. No toca `R_k` ni `S`. La relación inversa también se mantiene: un
cambio de `R_k` crea una nueva experiencia, no edita la anterior.

### 5.3 Contra la "colección de respuestas prefijadas"

- **El registro guarda trayectorias, no veredictos.** Una formalización
  recuperada es una hipótesis que hay que **readmitir y reevaluar** en el
  expediente nuevo. Su éxito pasado no se transfiere: en §6, h1 tuvo pérdida
  0 en Brookside y no añade nada en Casa Fiesta.
- **Se conservan los fracasos.** Hipótesis descartadas, refutadas o
  retiradas, cada una con su razón y con los contrastes que la debilitaron.
  Esto ya se registra (`discarded`, `retired_at`, `parent`).
- **Disciplina prequencial de la memoria.** Lo que se escribe sobre el
  expediente `k` antes de su scoring no puede contener su valoración. La
  versión posterior al feedback solo queda disponible para expedientes
  posteriores. Sin esta regla, la memoria filtra valoraciones reservadas.
- **Los nombres de constantes llevan semántica.** `jant`, `usa` o
  `casaFiesta` cargan significado en el embedding. Para medir si la
  recuperación aprovecha estructura o nombres sirve el control de
  vocabulario opaco del contrato de 004-R §8, que está definido y no se ha
  ejecutado.

### 5.4 Ingredientes que ya existen para clasificar insuficiencias

| Tipo | Señal disponible hoy |
|---|---|
| **Representacional:** los hechos no contienen lo que la regla lee | `h` no añade nada en `S` (`added_events = []`). Los verbos del cuerpo de `h` no aparecen en `C_B(S)`. `goal_in_universe = false`. Expediente sin semillas (Flowers) |
| **Deductiva:** representable, pero la teoría no deriva | una posición expresable cuyos soportes mínimos bajo `B ∪ h` están vacíos sobre el pool, mientras un pool ampliado sí la soporta. Es un residuo aditivo |
| **Frente a valoraciones externas** | discrepancias del scorer (faltantes, excesos, `unvalued_derived`), solo después del commit |

La clasificación no existe como función. Además, la frontera entre las dos
primeras depende de D1: si el LLM no puede tocar `S`, "representacional"
significa "fuera de su alcance".

## 6. Recorrido de extremo a extremo

### 6.1 Qué se ejecutó y qué no

**Ejecutado de verdad** (worker existente, `scripts/test_safe.py
--worker-manifest/--worker-requests`, 1 GiB sin swap, 15 s):
- 95 peticiones, todas `complete`;
- dos ejecuciones byte a byte idénticas: `output.jsonl`, sha256
  `c2e85f27…baad3`;
- informe del runner en `simulation/runner-report/`.

El manifiesto declara seis expedientes. Dos son reales (`brookside`,
`casaFiesta`, copiados de `dossiers.res`) y cuatro son mundos contrafácticos
de Casa Fiesta. Todos usan solo constantes ya declaradas en `vocabulary.res`.

Hipótesis:
- **h1 `correlativeRight`** y **h3 `settledPolicyAdoption`/`settledPosting`**,
  con el texto exacto que escribió Fable 5.1 en run-013. **No las inventé
  para la simulación.**

Operaciones:
- para cada mundo, la clausura bajo O, O ∪ {h1} y O ∪ {h3}
  (`begin_forward` con `theory`);
- evaluaciones certificadas del worker;
- `minimal_supports` y `compare_partition` para cinco objetivos.

**Solo conceptual:**
- la descripción semántica;
- el embedding;
- la recuperación;
- la elección de h3 por el LLM tras recuperar h1;
- el texto de las revisiones.

Ninguno existe en código. Abajo están marcados con **[simulado]**.

### 6.2 Expediente 1: Brookside, una experiencia ya almacenada

Esta experiencia se construye a partir de lo que ya ocurrió en run-013:

```
experiencia E1 (registro):
  expediente     brookside, S = 8 semillas (2 Act agree, 5 obligationFiled, 1 prohibition)
  d0             "8 seeds: Act×2 (agree), obligationFiled×5 (jant→usa), prohibition×1 (usa→jant);
                  O derives obligation×5"                                        [render, simulado]
  formalización  h1 correlativeRight: obligation(id,s,c,o,b,au,e) ⇒ right(id,s,c,o,b,au,e)
  consecuencias  O+h1 añade 5 right(saB_j, jant, …, usa)                         [ejecutado]
  resultado      pérdida 0 en Brookside (run-013, B7-B11)
  descartes      h2 prohibitionRight, con la razón escrita por el modelo (orientación vs rightCompliance)
  h (LLM)        "Settlement where one party files written undertakings to the other; the
                  undertakings were mirrored as same-party rights."              [simulado]
```

### 6.3 Expediente 2: Casa Fiesta

**Qué recibe el LLM:**
- la vista actual de 004-R: fuente de LegalGeneral, constantes, semillas de
  Casa Fiesta y su clausura bajo `B`, hipótesis, presupuestos;
- **además**, los `k` registros recuperados **[simulado]**. Con una
  descripción del tipo "acuerdo con el gobierno federal; la empresa adopta
  una política y coloca un aviso", E1 sería el vecino más próximo por
  descripción de hechos. Por estructura no lo es: Casa Fiesta solo tiene
  `Act` y ninguna `obligationFiled`. §5.1 lo predice.

**Paso 3: reutilizar h1. [ejecutado]**

```
O    sobre casaFiesta: 4 semillas, 0 derivados
O+h1 sobre casaFiesta: añade 0
```

El parecido en el embedding no garantizó nada. h1 lee `obligation`, y en
Casa Fiesta O no deriva ninguna `obligation`. **Insuficiencia
representacional:** los hechos dicen que la empresa *hizo* cosas (`Act`) y
no que *se obligó* por escrito.

**Paso 3 bis: adaptar. [la regla es real, de run-013; la elección tras
recuperar es simulada]** La adaptación es h3: si ambas partes firmaron el
mismo documento y una realizó una conducta, esa conducta estaba obligada.

```
O+h3 sobre casaFiesta: añade 4
  + obligation(cfA1, casaFiesta, adoptPolicy, policyCF, usa, …)
  + obligation(cfA2, casaFiesta, post, signCF, usa, …)
  + compliance(cfA1, cfA1, casaFiesta, policyCF, usa, …)
  + compliance(cfA2, cfA2, casaFiesta, signCF, usa, …)
O+h3 sobre brookside: añade 0      (Brookside no tiene Act de ejecución)
```

La transferencia falla en los dos sentidos: h1 no sirve en Casa Fiesta y h3
no sirve en Brookside.

**Pasos 6 y 7: contrafactuales. [ejecutados]** Cada mundo es `(S ∖ D) ∪ A`
sobre Casa Fiesta:

| Mundo | δ | O | O+h1 | O+h3 |
|---|---|---|---|---|
| `cf_filed` | `A = {obligationFiled(saCF_c, casaFiesta, adoptPolicy, policyCF, usa, …)}` | `+obligation(saCF_c…)`, `+compliance(saCF_c, cfA1…)` | además `+right(saCF_c…)` | sus 4 de siempre, **además** de lo que deriva O |
| `cf_no_agreeU` | `D = {Act(saCF_u, usa, agree, …)}` | sin cambio | sin cambio | **pierde sus 4** consecuencias |
| `cf_nonperf` | `Act(cfA1…)` → `~Act(cfA1…)` | sin cambio | sin cambio | **pierde** `obligation(cfA1…)` y `compliance(cfA1…)`; ninguna `violation` |
| `cf_filed_nonperf` | ambas | `+obligation(saCF_c…)`, `+violation(saCF_c, cfA1…)`, `+patrimonialLiability` | además `+right`, `+enforceability(saCF_c, cfA1…)` | igual que O para cfA1 |

Soportes y regiones discriminantes, calculados con `minimal_supports` y
`compare_partition` sobre el pool conjunto de los seis mundos:

| Objetivo | Soportes bajo h1 | Soportes bajo h3 | Discrimina |
|---|---|---|---|
| `obligation(cfA1…)` | ninguno | {Act cfA1, Act agree c, Act agree u} | sí: solo h3 |
| `compliance(saCF_c, cfA1…)` | {Act cfA1, obligationFiled saCF_c} | igual | no: lo deriva O |
| `compliance(cfA1, cfA1…)` | solo la región espuria con `right(cfA1…)` como hoja (§4.4) | {Act cfA1, ambos agree} | sí: solo h3 |
| `violation(saCF_c, cfA1…)` | {~Act cfA1, obligationFiled saCF_c} | igual | no: lo deriva O |
| `enforceability(saCF_c, cfA1…)` | {~Act cfA1, obligationFiled} | solo la región con `right` sembrado | sí: solo h1 |

Las evaluaciones certificadas del worker aceptan h1 en los tres mundos
probados. Rechazan h3 en `brookside` y en `cf_filed` con
`out_of_certificate_scope: [obligation]`, porque la clausura de referencia
ya contiene `obligation`. Para certificar contrafácticos con cabezas ya
pobladas hay que usar la evaluación con base explícita del arnés 004-R, no la
del worker.

**Paso 8: qué justificaría una revisión** (la decisión es del LLM; aquí solo
qué está disponible sin valoraciones):
- **h3 construye una obligación que solo existe si se cumple.**
  - En `cf_nonperf` la obligación desaparece en lugar de violarse.
  - Bajo h3 no hay ningún mundo del pool con `violation` de esa obligación.
  - El acto es a la vez la norma y su cumplimiento: `compliance(cfA1, cfA1…)`.

  Es una propiedad de la teoría, visible solo por contraste, y un motivo
  serio para revisar. No es una refutación: decidir si es un defecto
  requiere un juicio jurídico externo.
- **h3 exige el acuerdo de ambas partes** (`cf_no_agreeU`). Puede ser
  correcto, pero ahora es explícito y verificable.
- **La formalización de los hechos difiere entre experiencias.** Con la
  obligación escrita (`cf_filed`), O ya hace el trabajo que h3 intentaba
  hacer, con otro identificador de norma (`saCF_c` frente a `cfA1`). Es la
  insuficiencia de §6.3 vista desde el contraste. Resolverla de raíz pide
  re-representar los hechos de Casa Fiesta, y eso es D1, no una revisión de
  reglas.

**Paso 9: qué se almacenaría. [estructura propuesta; nada de esto existe]**

```
experiencia E2:
  expediente      casaFiesta, hash de S, d0 estructural
  recuperados     [E1] con su puntuación y la versión del embedder
  intentos        h1 (reutilizada; añade 0; insuficiencia representacional)
                  h3 (adaptada; añade 4; commit; retirada en revise en run-013)
  contrafácticos  4 mundos (D, A), diferencias, soportes, regiones discriminantes, todos con hash
  diagnóstico     "obligación condicionada a su cumplimiento"; "depende del acuerdo bilateral"
  resultado       sin valoraciones en el corpus (CF 0): ninguna pérdida informativa
  h (LLM)         versión 1 antes del scoring; versión 2 tras los contrastes   [simulado]
```

### 6.4 Expediente 3: Flowers

Las semillas están vacías: no hay ningún átomo representable establecido.
La recuperación podría devolver E1 o E2 por la narrativa **[simulado]**,
pero no hay nada que ejecutar: cualquier regla añade 0 y no hay pool del que
partir. Es el caso límite de insuficiencia representacional.

Lo que se almacenaría es la experiencia negativa ("no representable con O"),
que también sirve para recuperar. Ejecutarlo no añade nada a lo que ya
mostraron run-001 a run-013.

## 7. Mapa de dependencias

| Componente | Clasificación | Nota |
|---|---|---|
| O, motor, admisión, clausura | **Sin cambios** | |
| Evaluación con base `B` y atribución (`generative.rs`) | **Sin cambios** | |
| Soportes mínimos con hojas = semillas (`goal_supports`) | **Sin cambios** | usar este, no el `minimal_supports` del worker (hojas espurias) |
| `compare_partition`, discriminantes, `covers` | **Sin cambios** | |
| Mundos `S'` aislados | **Adaptador** | declararlos como expedientes adicionales. El navigator ya se reconstruye por expediente (`open_navigator`), así que añadirle mundos es local al arnés |
| Herramienta `propose_world{base, remove, add, reason}` | **Adaptador** | validación: verbos `seeded`, constantes declaradas, `D ⊆ S`, política sobre posiciones sembradas |
| Diferencias de clausura entre mundos | **Adaptador** | diferencia de conjuntos (como `analyze.py`) |
| Pool y consultas "qué mundos derivan e" por inclusión de soportes | **Adaptador** | aplica `frontiers` sobre el pool (§4.4) |
| Corregir `contains_event` en `claims.py` | **Adaptador** | requisito para claims sobre contrastes (closing-report §6.1) |
| Registro de experiencia y almacén entre trayectorias | **Extensión general** | se escribe desde `trajectory-report.json` y `turns.jsonl`; el arnés no cambia |
| `d0` estructural de expedientes y experiencias | **Extensión general** | reutiliza `formal_event` |
| Embeddings, caché y `retrieve` por fuerza bruta | **Extensión general** | proveedor por decidir (Stage 0 §7.4) |
| Descripciones `h` del LLM, versiones y claims con procedencia | **Extensión general** | contrato de revisión semántica |
| Inyección de recuperados en la vista del aprendiz | **Adaptador** | detrás de un flag del manifiesto, como los anteriores |
| Clasificación de insuficiencias | **Decisión conceptual previa** (D3) | los ingredientes están en §5.4 |
| Quién formaliza los hechos | **Decisión conceptual previa** (D1) | |
| Dependencias entre semillas y admisibilidad de mundos | **Decisión conceptual previa** (D2) | metadato, no lógica |
| Unidad y vista de recuperación | **Decisión conceptual previa** (D4) | |
| Contraste como evidencia causal | **Decisión conceptual previa** (D5); en parte **no expresable** | las condiciones 1, 2 y 4 de §4.7 no se satisfacen por construcción |
| Contrafactuales sobre teorías con defaults | **No expresable** con `frontiers` | no es monótona; se rechaza. LegalGeneral no tiene defaults |
| Dependencias causales entre hechos del mundo | **No expresable** en O | entran como semillas o no entran |

## 8. Camino mínimo y decisiones pendientes

### 8.1 Camino mínimo, sin rediseñar el motor ni ampliar LegalGeneral

1. **Mundos contrafácticos en el arnés de 004-R**, detrás de un flag:
   - `propose_world` valida `(D, A)` y registra `S'` como expediente
     auxiliar, sin valoración y no puntuable;
   - `evaluate_rule` acepta ese expediente;
   - la respuesta incluye diferencias frente al mundo base y soportes con
     hojas = semillas.

   Todo es reutilización (§7).
2. **Corrección de `contains_event`.**
3. **Almacén de experiencias.**
   - Al cerrar cada trayectoria, un adaptador escribe un registro inmutable
     por expediente.
   - Fuentes: `trajectory-report.json`, los mundos `S'` y sus resultados.
   - Formato: JSONL direccionado por contenido, `d0` incluido.
4. **Descripciones y embeddings.**
   - `h` del LLM con versiones, claims con procedencia y la disciplina
     prequencial de §5.3;
   - un cliente de embeddings con caché;
   - recuperación por fuerza bruta.
5. **Recuperados en la vista**, detrás de un flag. Cada recuperado se
   presenta como hipótesis a readmitir, nunca como respuesta.

Los pasos 1 y 2 se pueden verificar localmente como esta simulación, sin
modelo. Los pasos 3 a 5 solo dependen de D1-D4.

### 8.2 Decisiones que nos corresponden

- **D1. Quién formaliza los hechos de un expediente nuevo.**
  - **Curadores (como en 004-R).** "Representacional" es una insuficiencia
    que el LLM solo puede señalar.
  - **El LLM propone `S`.** Hace falta un control contra ajustar los hechos
    a las reglas, y la comparación entre experiencias se vuelve más frágil.
- **D2. Admisibilidad de intervenciones.**
  - si `inconsistent` rechaza un mundo o lo marca;
  - grupos de semillas que solo se intervienen juntos (dependencias
    documentales);
  - si se prohíbe sembrar posiciones proyectadas.
- **D3. Definición operativa de las tres insuficiencias** sobre las señales
  de §5.4, y cuál se calcula automáticamente.
- **D4. Unidad y vista de recuperación:**
  - registro con `z_hechos` y `z_form`;
  - qué se recupera y cuántos;
  - si se registran los recuperados no usados;
  - el proveedor de embeddings: API o modelo local dentro de 1 GiB.
- **D5. Lenguaje permitido sobre los contrastes.** Recomiendo "dependencia
  bajo T" y no "causa", salvo que se adopten explícitamente las condiciones
  de §4.7.
- **D6. Corpus.** Los tres expedientes actuales no tienen valoraciones en
  Casa Fiesta, y Flowers no es representable. Para el ciclo completo hace
  falta al menos un par de expedientes con valoraciones y estructura
  comparable. Esto no es "más jurisprudencia para generalizar": es la
  condición mínima para que el paso 5 (insuficiencia frente a valoraciones)
  ocurra alguna vez.

## 9. Respuesta a la pregunta

**¿Podemos construir, con lo que Residuum ya tiene, un aprendiz que
recupere formalizaciones de experiencias semejantes, las contraste con
expedientes nuevos, investigue sus insuficiencias mediante contrafactuales
y acumule descripciones semánticas revisables?**

**Resuelto.** Contrastar formalizaciones con expedientes nuevos e investigar
mediante contrafactuales. La simulación ejecutó el ciclo de los pasos 3, 4, 6
y 7 con el motor, el worker y las hipótesis reales de run-013, de forma
reproducible y sin valoraciones reservadas. Los contrafactuales no requieren
lógica nueva: son expedientes declarados, y bajo O monótona un único cálculo
de fronteras sobre un pool los responde todos.

**Falta, como ingeniería acotada.**
- la memoria entre trayectorias;
- `d0` de expedientes;
- las descripciones `h` revisables;
- embeddings y recuperación;
- la herramienta para proponer mundos.

Todo está especificado en contratos existentes y nada toca el motor.

**Requiere decisiones nuestras.** D1-D6, sobre todo:
- quién representa los hechos;
- qué intervenciones son admisibles;
- cómo se clasifican las insuficiencias;
- qué no se puede afirmar sobre causalidad.

**Primer aviso de la simulación.** Recuperar por similitud narrativa
habría propuesto una formalización (h1) que en el expediente nuevo no deriva
nada. El contraste lo detecta de inmediato y localiza el porqué. Es un
argumento a favor del ciclo propuesto, en el que la recuperación propone y
Residuum decide. Y es un argumento en contra de cualquier diseño que trate
la proximidad del embedding como evidencia.
