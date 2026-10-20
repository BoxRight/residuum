# Piloto 004-R: verificación de Brookside e implementación local del circuito

Complementa `docs/pilot-004r-generative-contract.md` (v1.1).
- Rama: `experiment-pilot004-documentary`.
- No se ha ejecutado ningún LLM ni se han hecho llamadas pagadas.
- No se han tocado el motor, LegalGeneral ni el Piloto 004 v2.

## 1. Decisiones recibidas

| Cuestión | Decisión |
|---|---|
| Derechos B7–B11 | Aprobación provisional, condicionada a verificar cada correspondencia. Véase §2. |
| Contenido del acuerdo como hechos | Sí, distinguiendo el acuerdo documentado de las posiciones que se derivan de él. |
| Orden | Casa Fiesta → Flowers → Brookside, como primera trayectoria prospectiva. |
| Presupuesto | Ocho evaluaciones verificadas por trayectoria. Llamadas, tokens y coste se registran aparte. |
| Implementación | Infraestructura y pruebas locales, sin llamadas pagadas. |
| Casos O1–O3 | Se buscan después de comprobar el circuito. |

## 2. Verificación individual de B7–B11

### 2.1 Qué queda en los hechos y qué se deriva

**En los hechos** ($S_B$, `docs/legal-geometry/pilot-004r/dossiers.res`):
- las dos firmas (`Act … agree … saDocB`);
- cinco entradas documentales `obligationFiled`;
- `prohibition(usa, sue, premisesB, jant)`.

LegalGeneral no tiene un `prohibitionFiled`, así que el pacto de no demandar
entra como `prohibition` sembrada. Es la única entrada en la que el documento
y la posición coinciden, y por eso B6 solo sirve como control.

**Derivado por LegalGeneral** (regla `filedObligation`):
- las cinco `obligation` B1–B5;
- sus efectos `do`;
- el efecto `notDo` de B6.

No se siembra ninguna `right`.

### 2.2 Cláusulas en las que se apoya la exigibilidad

| | Cláusula |
|---|---|
| E1 | «Failure by the United States to enforce this entire Agreement **or any provision thereof** with regard to any deadline or any other provision herein will not be construed a waiver of **the United States' right to enforce other deadlines and provisions** of this Agreement.» |
| E2 | «the United States **reserves the right to file a civil lawsuit to enforce this Agreement**» |
| E3 | «If the United States believes that JANT, LLC has violated this Agreement **or any requirement thereof**, it agrees to notify in writing … JANT, LLC shall have **30 days** from its receipt of the notice to cure and/or respond» |
| E4 | «If **any provision** of this Agreement is determined by any court to be unenforceable, **the other terms** of this Agreement shall nonetheless remain in full force and effect» |

### 2.3 Por qué las cinco posiciones no se tratan como una única cláusula general

- E1 atribuye a Estados Unidos un derecho a exigir el cumplimiento **por estipulación y por plazo**. Dejar de exigir una no renuncia a las demás.
- E3 individualiza el incumplimiento por requisito.
- E4 declara separables las estipulaciones.

Estos tres textos individualizan la exigibilidad por estipulación. La
cláusula de reserva (E2), por sí sola, no bastaría, y no se usa sola.

### 2.4 Paso de formalización que requiere tu aprobación

> «Estados Unidos tiene derecho a exigir la estipulación P de JANT» se
> representa como `right(saB_j, jant, c_P, o_P, usa, private, d20210216)`.

En LegalGeneral, `right` con `Act` da `compliance`, y con `~Act` da
`enforceability`. Esa lectura coincide con «enforce»: si no hay cumplimiento,
la estipulación es exigible.

Lo que **no** representa:
- el aviso escrito previo y el plazo de 30 días para subsanar (E3);
- que la exigencia se haga mediante una demanda civil (E2).

Esto se registra como reserva **R-a**, aplicable a las cinco posiciones.

### 2.5 Correspondencia por posición

| id | Estipulación (texto) | Conducta y objeto | Sujeto, beneficiario, autoridad, vigencia | Resultado |
|---|---|---|---|---|
| B7 | «adopt the Service Animal Policy … attached hereto … as Attachment A» (§11 a) | `adoptPolicy`, `policyA` | `jant`, `usa`, `private`, `d20210216` | Verificada, con R-a |
| B8 | «distribute the Policy to all employees and managers … who have contact with the public» (§11 b) | `distribute`, `policyA` | ídem; los empleados son destinatarios, no titulares | Verificada, con R-a |
| B9 | «post **a copy** of this Policy in a conspicuous location where employees can readily read the Policy» (§11 d) | `post`, `policyA` | ídem | Verificada, con R-a y **R-b**: el objeto del texto es una copia de la política y se representa con el documento `policyA` |
| B10 | «keep the Policy on hand for any store customers who wish to, upon request, read the Policy» (§11 e) | `keepAvailable`, `policyA` | ídem; los clientes son destinatarios, no titulares | Verificada, con R-a |
| B11 | «post in a conspicuous location in the public entryway of the store a sign prominently indicating "Service Animals Welcome"» | `post`, `signB` | ídem | Verificada, con R-a |

**Estipulaciones sin posición `right` valorada:**
- la entrega de la política a nuevos empleados (§11 c);
- los formatos de impresión;
- las confirmaciones escritas;
- la redistribución anual;
- la formación.

No tienen objeto individualizado o no tienen posición `obligation`
representada (contrato mínimo, §2.1). Al no haber obligación, no se valora su
`right`.

**Pretensiones que no se valoran**, porque el documento no las atribuye:
- ninguna pretensión de la demandante ni de terceros;
- ninguna pretensión de JANT frente al pacto de no demandar (B6).

**Estado:** las cinco quedan como `provisional` en
`reserved/valuations.tsv` hasta que apruebes R-a, R-b y el paso de §2.4.
Ninguna se ha descartado.

## 3. Lo implementado

### 3.1 Archivos

| Archivo | Contenido |
|---|---|
| `src/experiments/generative.rs` | Módulo nuevo del arnés: verificador, certificado, protocolo de trayectoria. |
| `src/experiments.rs` | Una línea: `pub mod generative;`. |
| `src/bin/residuum/cli/pilot004r_tests.rs` | 11 tests deterministas. |
| `src/bin/residuum/cli/tests.rs` | Registro del módulo de tests. |
| `docs/legal-geometry/pilot-004r/vocabulary.res` | Solo constantes de instancia; importa LegalGeneral sin cambios. |
| `docs/legal-geometry/pilot-004r/dossiers.res` | $S_B$, $S_{CF}$. $S_F=\varnothing$. |
| `docs/legal-geometry/pilot-004r/reserved/valuations.tsv` | Valoraciones reservadas. Solo las lee el evaluador. |

No se han modificado el motor (`derive`, `elab`, `formal`, `typed`, `residual`),
`proposal.rs`, `exploration.rs` ni `worker.rs`. Se reutilizan:
- `proposal::admit`, para el parseo, la frontera «solo reglas», la elaboración y el perfil;
- `exploration::install`;
- `theory_preserved`;
- `close_program`.

### 3.2 Contrato de certificación

**`check_closure(module, seeds, closure)`** es un verificador independiente. No
usa el emparejador ni el bucle de cierre del motor. Comprueba cuatro
obligaciones:

| Obligación | Qué exige |
|---|---|
| `sound` | Cada justificación instancia exactamente su regla con la sustitución registrada, y está fundada en las semillas. La fundamentación se comprueba con un punto fijo independiente del orden de los registros. |
| `supported` | Toda semilla es conocida, y todo evento conocido que no es semilla tiene justificación. |
| `event_complete` | Ninguna instancia de regla sobre Known produce un evento fuera de Known. |
| `justification_complete` | Toda instancia de regla sobre Known aparece como justificación. Los tests confirman que el motor registra todas las instancias. |

Un fragmento no cubierto (defaults, programas, cláusulas residuales, patrones
con records) da un error, que se trata como estado `pending`, nunca como fallo.

**`certify_attributed(O, O⊕R, R, S)`** devuelve `holds()` solo si se cumplen
todas estas condiciones:
1. las dos clausuras pasan `check_closure`;
2. `O` no cambia y las únicas adiciones son las reglas de `R`;
3. hay monotonía de eventos y de justificaciones: nada de `O` se pierde;
4. cada justificación nueva usa una regla de `R` o un antecedente nuevo (atribución).

Se informan aparte:
- los eventos nuevos, distinguiendo `direct` (regla aprendida) de `chained` (regla de `O` sobre consecuencias nuevas);
- las justificaciones redundantes de eventos que `O` ya derivaba;
- las reglas de `O` activadas.

La atribución por provenance es la condición 4. La corrección de las
consecuencias la aseguran las condiciones 1–3, de modo que la provenance no
basta por sí sola.

### 3.3 Interfaces del ciclo

Son interfaces en proceso. El aprendiz implementa `Learner::act(&LearnerView) → (LearnerAction, Usage)`.

| Interfaz | Contrato |
|---|---|
| Exploración: `LearnerView` | Muestra para los expedientes presentados: semillas, clausura de $O\cup R_k$ y justificaciones con regla, origen (`O` o hipótesis), antecedentes y sustitución. También sus hipótesis con la admisión y las evaluaciones, el compromiso vigente, el feedback recibido y los presupuestos restantes. |
| `Propose{text, parent, rationale}` | Pasa por `admit`, más A6: las constantes deben ser de `O` o de expedientes ya presentados, y se marca `case_specific`. También detecta colisión de nombres. Un rechazo no es una refutación. |
| `Evaluate{hypothesis, dossier}` | `certify_attributed` de $O\cup R_k\cup h$ sobre un expediente ya presentado. Consume una de las 8 evaluaciones verificadas. Devuelve el estado, los eventos nuevos, las reglas de `O` activadas y las posiciones a las que contribuye. **Nunca devuelve pérdida ni valoraciones.** |
| `Commit{hypotheses}` | Fija $R_{k+1}$. Es una puerta obligatoria: $O\cup R_{k+1}$ debe certificar en todos los expedientes presentados. No consume presupuesto del aprendiz. |
| Evaluación prospectiva | Al cerrar la exploración, el evaluador puntúa la teoría comprometida (prequential) y registra las exposiciones. |
| Feedback | Con `Feedback`: posiciones que faltan, posiciones excluidas derivadas con las hipótesis responsables, y recuentos. Con `NoFeedback`: solo «evaluado». |
| Revisión | Fase posterior con los mismos actos. Después se repuntúan los expedientes vistos (compatibilidad, no prospectiva). |
| `revision_effect(feedback, control)` | La pérdida prequential del control menos la del feedback, en los expedientes posteriores a la primera discrepancia. Si no hay expedientes posteriores, da `NotMeasurable`. |
| Registro | Ledger por acción. Contadores separados de turnos, intentos, admitidas, evaluaciones verificadas, acciones rechazadas, compromisos, rechazos de compromiso, llamadas, tokens de entrada y salida, y coste. |

**Estados de hipótesis:**
- `Valid`;
- `Prospective(n, e)`;
- `Compatible(n, e)`;
- `Refuted(dossier, valoraciones)`;
- `Unexposed`;
- `CaseSpecific`;
- `SupersededBy`;
- `Pending`.

### 3.4 Pendiente, sin implementar

- Puente con el protocolo existente: `wire.rs`, el controlador Python y los adaptadores de proveedor. Con eso un LLM real podría actuar como `Learner`. No se ha hecho nada en esa capa.
- Herramientas geométricas (`minimal_supports`, `partition`, `discriminants`) dentro de `LearnerView`. Siguen disponibles en el worker existente, pero no están conectadas a la trayectoria.
- Vocabulario opaco y prueba de reconocimiento.
- La ejecución con un modelo real, que necesita tu autorización y un presupuesto.

## 4. Pruebas

Se ejecutan con `scripts/test_safe.py --test-target cli`, una a una, con
1 GiB, un trabajo de Cargo y un hilo.

**Tests del circuito 004-R:** 11 de 11 pasan (informe `20261020T041544631154Z-2211671`).

| Test | Qué certifica |
|---|---|
| `install_admits_rule_with_existing_seeded_head_right` | `install` añade una regla con cabeza `right` (sembrada) y conserva `O`. No permite redeclarar la firma (`S_INSTALL`). |
| `conservative_certificate_covers_right_on_brookside` | En $S_B$, `obligation ⇒ right` cumple el certificado conservativo y el atribuido: 5 eventos nuevos directos y ninguna regla de `O` activada. |
| `conservative_certificate_rejects_chained_consequences_attributed_accepts` | Con un `~Act` sintético, el certificado conservativo falla porque aparece un `enforceability` nuevo. El atribuido lo acepta como `chained` por `rightEnforceability`. |
| `checker_detects_tampered_closures` | El verificador acepta la clausura limpia y detecta un evento borrado, una sustitución alterada, un evento fabricado y una justificación eliminada. |
| `attributed_certificate_rejects_unattributed_or_lossy_extensions` | Rechaza adiciones no declaradas y extensiones que pierden reglas de `O`. |
| `corpus_baseline_without_hypotheses` | Sin hipótesis, las pérdidas son CF 0, F 0 y B 5: faltan exactamente B7–B11, y los controles B1–B6 se cumplen. |
| `prospective_trajectory_casa_fiesta_flowers_brookside` | La regla generada en CF y comprometida antes de B queda `Valid`, `Prospective(1,5)` y `Compatible(1,5)`, con pérdida total 0. No puede refutarse. |
| `feedback_and_control_share_budget_and_isolate_valuations` | Mismo presupuesto en ambas condiciones. Con feedback, la regla aparece en la revisión tras B y queda `Compatible`, no prospectiva. El efecto de revisión es `NotMeasurable`. Ninguna vista contiene etiquetas, fuentes ni identificadores de valoración. |
| `budget_counts_verified_evaluations_and_usage_separately` | La novena evaluación se rechaza, igual que una evaluación sobre un expediente no presentado. Uso y coste se cuentan aparte. |
| `admissible_language_rejections_are_not_refutations` | Se rechazan cabeza `Act`, cabeza `obligationFiled`, declaraciones, constantes no vistas y colisiones de nombre. Todas quedan `Pending`. |
| `synthetic_refutation_is_delivered_and_revised_without_the_solution` | Es el único con expedientes sintéticos, ajenos al corpus. La discrepancia se entrega (posición excluida y responsable `h1`, sin texto de reglas). La revisión guionada `h2` es prospectiva en x2. El efecto medido es Δ = 1 frente al control, que sigue refutado en x2. |

**Congelación del v2:**
- Los cuatro tests de `pilot004::` pasan con la biblioteca modificada.
- `MANIFEST.sha256` del Piloto 004 verifica sin fallos.

**Limitación declarada:** con Casa Fiesta, Flowers y Brookside, la capacidad
de revisión ante recalcitrancia no es medible (contrato §3.4). El circuito sí
lo es: el test sintético certifica que una refutación existente se entrega,
se atribuye y se puede revisar sin que el arnés entregue la solución.
