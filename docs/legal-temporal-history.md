# Historia temporal de formalizaciones normativas en Residuum

Rama `experiment-legal-order-geometry`. Continúa `docs/legal-temporal-activation.md`
(F1–F8). Sin cambios en el motor, en `LegalGeneral` ni en fixtures anteriores.
Sin pilotos LLM.

Clasificación de las afirmaciones: **implementada**, **demostrada bajo hipótesis**,
**verificada en dominio finito**, **expresable no implementada**, **pendiente**.

## 0. Resultado

Existe una teoría histórica Formal ejecutable,
`docs/legal-geometry/temporal-history/fixtures/history2.res`, construida solo con
construcciones nativas: programas asociados a verbos, axiomas `I` y la
indexación proposicional existente. Para los tres expedientes del experimento,
la proyección de su cierre sobre la órbita de cada momento coincide con la
ejecución aislada de la teoría de ese momento. La igualdad abarca eventos,
justificaciones con multiplicidad, sustituciones, antecedentes e identidad de
productor (regla, programa, miembro y módulos de origen). Añadir formalizaciones
no altera las proyecciones anteriores. **(verificada en dominio finito;
demostrada bajo hipótesis en §5)**

Además:

- **No hace falta un segundo índice temporal.** El tiempo proposicional del
  expediente basta como coordenada.
- **No hace falta una variante de `LegalGeneral`.** La única ampliación es un
  verbo experimental, `normEffective`, declarado en el vocabulario del
  experimento (§8).
- **No hace falta un transformador externo.** $\mathcal I_t$ es un axioma Formal.
- **El harness no infiere nada.** Todos los cierres los calcula
  `close_program`. El harness carga módulos, proyecta, compara y comprueba las
  condiciones estructurales.

## 1. Contrato

- $O$ = `LegalGeneral`, importado sin cambios.
- $R_k$ es una formalización sin tiempo: un `program` cuyo único parámetro es
  el tiempo proposicional y cuyos miembros son reglas del espacio de
  generalizaciones.
- $\mathcal I_{t_k}(R_k)$ es el **anclaje**: un axioma
  `rule anchorRkAtTk() = I <= appliesRk() @ tk`, donde `appliesRk` es el verbo
  asociado a `program Rk`.
- $T_k = O \cup \mathcal I_{t_k}(R_k)$ es el módulo `stage_k.res`.
- $H_n = O \cup \bigcup_{k \le n} \mathcal I_{t_k}(R_k)$ es el módulo
  `history_n.res`. Importa los módulos de etapa (sus experimentos no se
  importan) y siembra $\biguplus_k S_k$.
- $\mathrm{orb}(t) = \{t, \mathrm{after}(t), \dots\}$, y
  $\pi_t(C)$ = los eventos de $C$ en $\mathrm{orb}(t)$ junto con sus
  justificaciones.

Secuencia: $R_k$ se fija antes de la valoración de $S_k$. La historia contiene
solo las formalizaciones usadas. Las comparaciones van a módulos hipotéticos
aislados (§7.5).

## 2. Comparación de las alternativas nativas

| | A. Reglas ancladas a mano | B. Especialización nativa | C. Programa + anclaje `I` |
|---|---|---|---|
| Representación de $R_k$ sin tiempo | No existe: cada etapa reescribe las reglas | La plantilla existe; cada miembro se especializa | Un objeto: `program Rk(t)` |
| $\mathcal I_t$ | Reescritura manual | `rule bkX = X(tk)`, una por miembro y etapa | Un axioma por etapa |
| Plantilla activa fuera de su tiempo | No hay plantilla | **Sí, si no está capturada** (F6, `cx_active_template`). No, si es una definición | No: los miembros son definiciones |
| Reusar $R_k$ en otra etapa | Copiar reglas con nombres nuevos | Nuevas especializaciones | Otro axioma (`anchorR1AtT2`) |
| Bindings registrados | Sin `t` (es constante) | Sin `t` | `#Rk:shared:t = tk` y `#Rk:useI:x` por variable |
| Provenance de la formalización | Solo el nombre de la regla | El nombre de la especialización | Programa, módulo definidor, miembro, regla, módulo de la regla, y la activación con su anclaje |
| Eventos frente a C | Iguales, sin `appliesRk` | Iguales, sin `appliesRk` | Referencia |

**D. Variante de `LegalGeneral`: no se ha construido.** C conserva las 27 reglas
y todas las firmas de $O$. Ninguna obligación de §VIII exigía reorganizarlas.

**Activación y captura.** La activación por programa no es la captura de F7.
F7 metía en un programa una regla que *ya era ordinaria*, y eso la retiraba
(reemplazo). En C, toda regla del espacio de generalizaciones es una definición
desde que se declara: `generalizations.res` la usa en `program Templates`,
asociado a `templatesDefined`, un verbo que nadie siembra ni deriva y que
existe solo para dejar las plantillas inactivas como reglas generales. Este
idioma ya existía (`InactiveDefinitions` en `temporal_identity_formal.res`).
Al añadir una formalización no se captura ninguna regla que antes estuviera
activa. El test lo comprueba: ninguna regla ordinaria de $H_k$ es una
definición en $H_{k+1}$.

## 3. Elección: C

C es la más fiel:
- $R_k$ existe como objeto sin tiempo, revisable y reutilizable.
- El anclaje es explícito en el lenguaje y deja provenance.
- La activación queda registrada como antecedente de cada consecuencia.

A sirve como referencia de comportamiento. B es correcta solo con plantillas
capturadas.

## 4. Construcción

Directorio `docs/legal-geometry/temporal-history/fixtures/`:

| Módulo | Contenido |
|---|---|
| `vocabulary.res` | Importa `LegalGeneral`. Declara constantes, momentos `t0..t2`, fechas jurídicas `enact0`, `enact1`, `visitDate0`, `saleDate1`, y los verbos sembrados `accompanies`, `propertyA` y `normEffective` |
| `generalizations.res` | Las plantillas `ordinaryBan`, `refinedBan` y `registrationDuty`, con el tiempo como primer parámetro, y `program Templates` |
| `formalization_r0.res` | `program R0(t) { use ordinaryBan }` y `appliesR0()` |
| `formalization_r1.res` | `program R1(t) { use refinedBan; use registrationDuty }` y `appliesR1()` |
| `stage0/1/2.res` | El anclaje ($R_0$ en `t0`; $R_1$ en `t1`; $R_1$ otra vez en `t2`) y el expediente aislado |
| `history1.res`, `history2.res` | $H_1$ y $H_2$ |
| `hypothetical_r1_on_dossier0.res` | $\operatorname{Run}_{O \cup \mathcal I_{t_0}(R_1)}(S_0)$, fuera de la historia |
| `alternatives/` | A y B |
| `counterexamples/` | Los cuatro contraejemplos de §7 |

Expedientes con participantes compartidos (ana, rex, bea, fido):
- **$S_0$ en `t0`:** rex tiene A. $R_0$ prohíbe a rex y a fido; de ahí se derivan
  `notDo`, `violation` y `patrimonialLiability`.
- **$S_1$ en `t1`:** fido tiene ~A acreditado. $R_1$ prohíbe a fido e impone
  `regDuty` a ana por rex, lo que deriva `do`. Una venta deriva `transferred`,
  `owns`, `capacity` y `give`. La obligación también consta archivada
  (`obligationFiled`), así que tiene dos justificaciones.
- **$S_2$ en `t2`:** $R_1$ prohíbe a fido. Un permiso legislativo con fecha
  `enact1`, más `notEarlier(enact1, enact0)`, deriva `annulled`.

Cierres archivados (eventos): $S_0$ = 12, $S_1$ = 17, $S_2$ = 11, $H_1$ = 29,
$H_2$ = 40 = 12 + 17 + 11. A y B tienen 37: los mismos, menos los tres eventos
de activación.

## 5. Lema de localidad: enunciado, demostración y correspondencia con el código

**Hipótesis** (cada una con su comprobación en
`src/bin/residuum/cli/temporal_history_tests.rs`):

- **L1.** Todo momento de expediente es una constante desnuda (`TimeExpr::At`),
  y los anclajes tienen raíces distintas dos a dos, con un anclaje por momento
  (`moment_violations`).
- **L2.** Toda semilla de $S_k$ está indexada exactamente en $t_k$. Las fechas
  jurídicas internas van como argumentos (`seed_violations`).
- **L3.** Toda regla de $O$ combina sus premisas en un único tiempo variable y
  concluye en $t$ o en $\mathrm{after}(t)$. Comprobado sobre las 27 reglas: los
  cuerpos solo usan `@ t`, y las únicas cabezas en `after t` son las de `do`,
  `notDo` y `give`, que ninguna regla usa como premisa
  (`ontology_rules_are_time_local_and_effects_are_terminal`).
- **L4.** Todo miembro compilado de un programa asociado solo menciona tiempos
  en la órbita del parámetro temporal compartido (`member_violations`).
- **L5.** Fuera de $O$ no hay reglas ordinarias con tiempo variable: las
  plantillas son definiciones (`active_template_violations`).
- **L6.** Las instancias de activación solo las producen los anclajes; no
  aparecen en semillas ni en otras reglas (`activation_violations`).
- **L7.** Los nombres de símbolo son únicos en el grafo de módulos: el resolver
  rechaza los duplicados (`src/modules.rs`, `duplicate symbol`).

**Semántica utilizada** (`src/derive.rs`):
- `prepare_programs` (l. 212–303) retira de las reglas ordinarias las que usa
  un programa asociado a un verbo.
- `activation_bindings` (l. 341–359) liga los parámetros del programa a los
  argumentos del disparador, y **el último al tiempo del disparador**.
- `instantiate_program` empareja los miembros con esos bindings fijados.
- `finish_program_match` añade el disparador como antecedente.
- El emparejamiento temporal es sintáctico (F1).

**Lema.** Bajo L1–L7, para todo $k \le n$:

$$\pi_{t_k}\, C_{H_n}\Big(\biguplus_j S_j\Big) = C_{T_k}(S_k),$$

como multiconjuntos de justificaciones (evento, productor, antecedentes,
sustitución), y como conjuntos de eventos.

**Demostración.** Por inducción sobre el orden de derivación, todo evento de
$C_{H_n}$ tiene como raíz algún $t_j$, y toda derivación tiene antecedentes de
la misma raíz que su conclusión:

- Las semillas cumplen la condición por L2.
- Los anclajes no tienen antecedentes y concluyen en $t_j$.
- Una instancia de una regla de $O$ tiene todos sus antecedentes en un mismo
  $\tau$ y concluye en $\tau$ o en $\mathrm{after}(\tau)$ (L3), así que conserva
  la raíz.
- Un miembro activado tiene un disparador `appliesR(...) @ t_j` (L6), y
  `activation_bindings` fija el tiempo compartido en $t_j$. Por L4, todos sus
  tiempos están en $\mathrm{orb}(t_j)$, y el disparador mismo es antecedente.
- No hay más reglas ordinarias con tiempo variable (L5).

Como los momentos son constantes desnudas distintas (L1), $\mathrm{orb}(t_j)$
es exactamente el conjunto de términos de raíz $t_j$, y las órbitas son
disjuntas. Por tanto, las derivaciones con conclusión en $\mathrm{orb}(t_k)$
solo usan premisas de $\mathrm{orb}(t_k)$:
- semillas de $S_k$;
- reglas de $O$;
- miembros activados por disparadores de raíz $t_k$, es decir, por el único
  anclaje en $t_k$ (L1, L6), que es el de $R_k$.

Ese es exactamente el conjunto de premisas y reglas de $T_k$ sobre $S_k$.
Ambas clausuras son el menor punto fijo de las mismas instancias. Los
productores coinciden en nombre por L7, y en origen porque son las mismas
declaraciones de los mismos archivos. ∎

**Estabilidad.** $H_k$ es un submódulo de $H_n$ y $\biguplus_{j \le k} S_j$ es
un subconjunto de las semillas. Por el lema, ambos coinciden en
$\pi_{t_j}$ para $j \le k$. **(demostrada bajo hipótesis)**

**Correspondencia de la proyección.** `in_orbit` implementa
$\mathrm{orb}$; `project` toma los eventos conocidos y las derivaciones cuya
conclusión cae en la órbita; `justification` define la clave de comparación.

## 6. Certificación

Cada test se ejecuta individualmente con
`scripts/test_safe.py --test-target cli --test cli::tests::temporal_history::<nombre>`.
Los ocho pasan; los identificadores de informe están en
`runs/certification-tests.txt`.

| Test | Obligación de §VIII |
|---|---|
| `ontology_rules_are_time_local_and_effects_are_terminal` | L3; 12 (efectos en `after(t)`) |
| `history_satisfies_the_locality_conditions` | 1 (participantes compartidos), L1–L6, semillas = unión |
| `projection_of_the_history_equals_each_isolated_run` | 7, 8 (multiplicidad), cobertura total por órbitas |
| `adding_formalizations_preserves_the_historical_projections` | 5, 6, estabilidad, ausencia de captura |
| `consequences_effects_and_provenance_of_the_history` | 2, 3, 4, 8 (provenance y justificaciones alternativas), regresión de `notDo` |
| `native_alternatives_reproduce_the_events_of_the_program_history` | Comparación de A, B y C |
| `counterexamples_are_detected_and_break_the_equivalence` | 9, 10, 11, premisa de F4 |
| `hypothetical_runs_stay_outside_the_history` | Separación de historia e hipótesis |

**Contrato de identidad de productor.** El productor de una derivación se
identifica por:
- el nombre de la regla, único en el grafo de módulos;
- si es una regla ordinaria, su origen (módulo y archivo canónico);
- si es un miembro compilado, `program_rule_origin`: programa, módulo
  definidor, índice de miembro, regla y origen de la regla.

La etiqueta `derivedAt` se excluye de la comparación. La sustitución se compara
ordenada por binder.

**Regresión de `notDo` (F4).** En $H_2$, `notDo(ana, enterWith, rex, host)`
existe solo en `after(t0)`. En `after(t1)` y `after(t2)` no existe, aunque rex
está en los tres expedientes. La premisa del hallazgo de F4 (un mismo momento
evaluado por varias formalizaciones) se reproduce en `cx_double_anchor` (§7.4),
y el detector la rechaza.

## 7. Contraejemplos mínimos

1. **Plantilla activa** (`cx_active_template`, F6): `looseBan` no está
   capturada. Especializarla en `t0` no la confina, y aparece
   `prohibition(…rex…) @ t1`. Lo detecta `active_template_violations`.
2. **Tiempos independientes** (`cx_mixed_times`): el miembro
   `mixedBan(t, u, …)` tiene `normEffective(dogBan, e) @ u` sin ligar. En la
   historia, el dato de `t0` produce la prohibición de fido en `t1`; en la
   ejecución aislada no se produce. Lo detectan `member_violations` y
   `cross_moment_derivations`.
3. **Órbitas que se cortan** (`cx_after_stage`, `cx_after_history`): un
   expediente cuyo momento es `after t0`. Los dos `notDo` del expediente 0 caen
   en su órbita. Lo detecta `moment_violations`. Una regla de $O$ que use
   `@ after t` produce exactamente esta colisión si un momento es un término
   `after` de otro. Por eso numerar los momentos con Peano es incompatible con
   este contrato.
4. **Dos formalizaciones en un momento** (`cx_double_anchor`, premisa de F4):
   $R_1$ anclada también en `t0`. La proyección no coincide con ninguna de las
   dos ejecuciones aisladas, y el `notDo` de rex persiste aunque $R_1$ no
   prohíbe a rex. Lo detecta `moment_violations`.
5. **Evaluación hipotética** (`hypothetical_r1_on_dossier0`): $R_1$ sobre
   $S_0$ no produce nada normativo, porque $S_0$ no tiene ~A ni el dato de
   `regDuty`. El módulo no lo importa ninguna historia.

## 8. Semántica jurídica: `effective`, `notEarlier`, `higherAuthority` y anulación

- Las posiciones toman `effective` del dato explícito
  `normEffective(norm, e) @ t_k`, que declara la vigencia de la norma
  formalizada. Sin ese dato no se produce ninguna posición: no se inventan
  fechas.
- El test comprueba que ninguna posición lleva un momento de expediente como
  último argumento.
- `notEarlier` y `higherAuthority` comparan fechas y autoridades sembradas como
  datos. En $S_2$, `annulled(dogBan, permitFido, legislative, enact1)` se deriva
  con la regla original de $O$.
- Las fechas de actos (`visitDate0`, `saleDate1`) son argumentos distintos de la
  vigencia y del momento.

**Pendiente.** `normEffective` es una convención del experimento, no una figura
de $O$. Una generalización aprendida que formaliza una justificación
desconocida no tiene una vigencia determinada por el contrato. Habrá que
decidir quién aporta ese dato (las valoraciones, el orden jurídico del
expediente u otra fuente) antes de que el explorador produzca posiciones
nuevas.

## 9. F1–F8 bajo este contrato

| Resultado | Situación |
|---|---|
| F1: `after` como álgebra libre, emparejamiento sintáctico | Aplicable: es la base de L1 y de la identidad por momento |
| F2: añadir reglas es monótono | Aplicable: estabilidad |
| F3: activación positiva por `Verb.program` | Aplicable: mecanismo de C |
| F4: épocas, `evaluatedAt`, certificados `unrevisedThrough` | **Ya no necesario**: el momento del expediente es la coordenada. La condición (a) se reduce a qué formalización se ancla en cada momento |
| F4: hallazgo de `notDo` | Su premisa queda excluida por L1; regresión y contraejemplo en §6 y §7.4 |
| F4: `unknownA` indeterminado sin evidencia | Sigue vigente, pero pertenece al aprendizaje, no a la infraestructura temporal |
| F5: los defaults bloquean globalmente | Aplicable como exclusión: `close_program` rechaza módulos con defaults (`prepare_programs`, l. 224) |
| F6: la plantilla especializada sigue activa | Aplicable: es el contraejemplo 7.1; la corrección es capturar las plantillas |
| F7: la captura es un reemplazo | Aplicable como exclusión: en C no hay captura posterior a la declaración |
| F8: vigencia por `precedes` | Ya no necesario: no se distingue por fecha jurídica para separar versiones |

## 10. Papel del harness

El test en Rust solo hace esto:
- carga módulos;
- toma las semillas de los experimentos;
- llama a `close_program`;
- proyecta por órbita y compara multiconjuntos;
- comprueba L1–L7 sobre el AST tipado.

`definitions` replica la regla de `prepare_programs` solo para comprobar L5:
no decide qué reglas se ejecutan. La activación, la producción de
consecuencias, los efectos y la anulación ocurren en Formal.

## 11. Inventario, limitaciones y pendientes

**Capacidades existentes que se usan**
- Programas asociados a verbos con parámetro temporal.
- Axiomas `I`.
- Captura por asociación.
- Especialización parcial de reglas.
- `close_program`.
- `program_rule_origin`.
- Unicidad de símbolos en el grafo de módulos.
- `--forward` y `--cli explain` del runner seguro.

**Limitaciones verificadas**
- El runner por defecto (`run_experiments`) rechaza los módulos con `program`.
  Las consultas residuales con programas no están integradas en ese runner, así
  que la historia se ejecuta con `--forward` y `explain`.
- `explain` muestra la provenance solo del objetivo de cada experimento. La
  provenance completa se certifica a través de la API en el test.
- Los experimentos no se importan, así que la historia repite literalmente las
  semillas. El test comprueba que son la unión exacta de las de las etapas.
  Generar estos módulos corresponderá al harness.
- El grafo de módulos admite 64 KiB de fuente; $H_2$ ocupa unos 24 KB, de los
  que `LegalGeneral` es la mayor parte. Cada etapa añade alrededor de 1 KB más
  sus semillas.
- `--forward` y el trabajador exigen dependencias acíclicas, incluidas las
  aristas de activación.
- Las justificaciones de C incluyen el disparador como antecedente, así que
  solo son comparables entre ejecuciones de C. A y B solo se comparan por
  eventos.

**Pendiente**
1. La fuente de `effective` para generalizaciones aprendidas (§8).
2. Generar los módulos de etapa, historia e hipótesis desde un registro de la
   secuencia $R_k \to \widehat N_k \to y_k^* \to R_{k+1}$, con las valoraciones
   asociadas y la pérdida. **(expresable no implementada)**
3. $D_Q$ comparando módulo el identificador de norma, y la pérdida
   retrospectiva y prospectiva sobre ejecuciones hipotéticas.
   **(expresable no implementada)**
4. Que el trabajador del explorador (otra rama) acepte módulos con programas y
   anclajes. **(indeterminado)**
5. La equivalencia está verificada en un dominio de tres expedientes. El lema
   la extiende a cualquier historia que cumpla L1–L7, y los detectores son
   exactamente esas hipótesis.

## Archivos

- `docs/legal-geometry/temporal-history/fixtures/`: los módulos de §4, más
  `alternatives/` y `counterexamples/`.
- `docs/legal-geometry/temporal-history/runs/`: salidas `--forward` y
  `explain`, `result.json`, `report-id.txt`, y `certification-tests.txt`.
- `src/bin/residuum/cli/temporal_history_tests.rs`: la certificación.
  Registrada en `src/bin/residuum/cli/tests.rs` (una declaración `mod`).
