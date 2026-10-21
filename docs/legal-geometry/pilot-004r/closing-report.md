# Piloto 004-R: informe de cierre

Estado: **congelado** tras run-013 (commit `47aa573`, rama
`experiment-pilot004-documentary`, sin merge ni push). No se harán más corridas
sobre Brookside, Casa Fiesta y Flowers. Este informe resume:

- las corridas run-001 a run-013;
- las capacidades del sistema que se ejercitaron de hecho;
- sus limitaciones;
- las decisiones de arquitectura que surgieron por el camino.

Las fuentes son los README de cada corrida (`runs/run-NNN/README.md`), los
informes archivados (`runs/run-NNN/run/ledger/trajectory-report.json`), el
contrato (`docs/pilot-004r-generative-contract.md`) y la implementación
(`docs/pilot-004r-implementation.md`).

**Conclusión en una frase.** El núcleo generativo-simbólico funciona de
punta a punta: el LLM redacta reglas Formal sobre LegalGeneral, Residuum las
admite, certifica, atribuye y puntúa, y la trayectoria queda reproducible y
auditable. Pero con tres expedientes que solo discriminan en cinco
posiciones, el piloto no puede demostrar generalización, ni revisión guiada
por refutación, ni efecto del feedback.

## 1. Pregunta y contrato

El contrato (§0) pide permitir que un LLM genere generalizaciones sobre una
ontología fija (`T_k = O ∪ R_k`, con O = LegalGeneral) y observar si la
interacción entre derivación, recalcitrancia y revisión mejora su capacidad
predictiva.

**Métrica.** La métrica principal es la pérdida prequencial
`L^preq = Σ_i ℓ_i(R_k(i))`. Se calcula sobre las posiciones valoradas, en la
proyección (head, subject, conduct, object, beneficiary), con el conjunto
comprometido antes de ver el feedback del expediente i.

**Estados de una hipótesis:**
- **V**: válida (admitida y certificada).
- **C**: compatible (pérdida 0 tras exponerse).
- **P**: prospectiva (comprometida antes de ver el expediente y sin
  discrepancias en él).

**Pronóstico del contrato (§7), escrito antes de las corridas:**
- `L^preq` = 0 en Casa Fiesta y Flowers en cualquier ejecución.
- Entre 0 y 5 en Brookside.
- El efecto del feedback (ΔL) y la revisión efectiva, "no medible".
- El piloto "no puede mostrar revisión guiada por refutación".

Las trece corridas confirman este pronóstico.

## 2. Corpus y lo que podía medir

Recuentos de `reserved/valuations.tsv`; el contenido de las valoraciones no
se reproduce aquí.

| Expediente | Requeridas | Excluidas | Qué discrimina |
|---|---|---|---|
| Brookside | 11 | 0 | B1–B6 ya las deriva O (controles). B7–B11, `right(jant, c, o, usa)`, son provisionales con reservas R-a y R-b. |
| Casa Fiesta | 0 | 0 | Nada: no hay posiciones valoradas. |
| Flowers | 0 | 10 | Nada alcanzable: no tiene semillas, así que ninguna regla deriva algo. |

**Toda la señal del piloto está en B7–B11.**
- La regla `obligation(id,s,c,o,b,…) ⇒ right(id,s,c,o,b,…)`, con el mismo
  sujeto, las deriva.
- La forma con sujeto y beneficiario intercambiados deriva derechos que no
  están valorados. Por eso no se refuta: queda "Unexposed" y solo deja B7–B11
  sin derivar.
- El contraste (`contrast/`) muestra que la orientación de B7–B11 reproduce
  la convención con que formalizamos esos derechos, no una orientación
  jurídica.
- Casa Fiesta y Flowers no pueden puntuar ninguna regla nueva.

## 3. Arquitectura efectiva

**Harness (`src/experiments/generative.rs`).**
- `run_trajectory` sigue el ciclo explore → commit → puntuación → feedback
  → revise en cada expediente.
- Admisión de borradores por el lenguaje admisible (§2) y certificado
  atribuido (`certify_attributed`).
- Verificador independiente de clausuras (`check_closure`).
- Registro de estados V/C/P y puntuación prequencial con las valoraciones
  reservadas. El aprendiz nunca ve esas valoraciones.
- El motor, LegalGeneral y el corpus no se modificaron en ningún momento.

**Bridge (`boundary/src/bin/residuum-trajectory.rs`).**
- Vista y acción en JSON por líneas.
- Es el único componente que lee las valoraciones, y solo para el informe
  final.
- Traduce la navegación a peticiones del worker existente.

**Worker (`src/experiments/worker.rs`).**
- Ofrece `inspect_theory`, sesiones forward, residual y active-residual,
  `minimal_supports` y `compare_partition`.
- Trabaja sobre los expedientes ya presentados.

**Driver (`tools/pilot004r/driver.py`).**
- Proveedor Anthropic o simulado.
- Techos de coste y de llamadas.
- Ledger JSONL completo (`turns.jsonl`).
- Cada cambio de protocolo va detrás de un flag del manifest, de modo que las
  corridas anteriores se siguen reproduciendo.

## 4. Resultados, run-001 a run-013

Todas las corridas usan la condición `feedback`. Las doce primeras usan
`claude-sonnet-5-5` (thinking adaptativo, esfuerzo `medium`) y run-013 usa
`claude-fable-5-1`. "Forma correcta" significa `obligation ⇒ right` con el
mismo sujeto; "forma invertida", con sujeto y beneficiario intercambiados.

| Run | Cambio (flag) | Pérdida por expediente | L | Coste US$ | Conducta |
|---|---|---|---|---|---|
| 001 | Línea base; orden CF→F→B | 0 / 0 / 5 | 5 | 0.05 | 6 `end_phase` sin proponer nada. |
| 002 | Cerrar explore exige proponer o abstenerse | 0 / 0 / 5 | 5 | 0.06 | Tres abstenciones; ninguna regla, ni siquiera tras el feedback. |
| 003 | Objetivo de explorar; orden B→CF→F | 5 / 0 / 0 | 5 | 0.25 | Forma correcta creada en revise, tras el feedback. |
| 004 | Navegación disponible (60 acciones) | 5 / 0 / 0 | 5 | 0.15 | Forma correcta en revise; 0 navegaciones. |
| 005 | `question_first` (pregunta y consulta obligatorias) | 5 / 0 / 0 | 5 | 0.32 | 3 preguntas, 2 navegaciones forzadas, forma correcta en revise. |
| 006 | `draft_contract` (borradores sin penalización, `discard`) | 5 / 0 / 0 | 5 | 0.15 | Abstenciones; forma correcta en revise. |
| 007 | `findings_report` | 5 / 0 / 0 | 5 | 0.15 | 0 hallazgos. |
| 008 | `planning` | 5 / 0 / 0 | 5 | 0.18 | Planes completos; todas las operaciones planeadas sin ejecutar. |
| 009 | `investigation_split` | 5 / 0 / 0 | 5 | 0.18 | Primer borrador antes del feedback, sin evaluar ni comprometer; 1 de 10 operaciones ejecutadas. |
| 010 | `auto_evaluate` | 5 / 0 / 0 | 5 | 0.43 | Forma invertida comprometida antes del feedback y corregida en revise. En CF, primera revisión por consecuencias: h4 → h5 evita las obligaciones de `agree`. |
| 011 | `attribution` + trazas de commit | 0 / 0 / 0 | **0** | 0.28 | Forma correcta como primer borrador, comprometida antes del feedback: **P(1,5)**. Primer `discard`. |
| 012 | `supports` (G(h,S,B) y soportes mínimos) | 5 / 0 / 0 | 5 | 0.39 | Repite run-010: invertida, luego corregida. Todos los soportes triviales. |
| 013 | Modelo `claude-fable-5-1` | 0 / 0 / 0 | **0** | 2.93 | Forma correcta: **P(1,5)**. Alternativa redactada y descartada por su consecuencia; 1 navegación voluntaria; 4 hallazgos. |

En run-001 y run-002 el orden de las columnas es CF / F / B; desde run-003 es
B / CF / F. El coste total es de unos US$5.5 en corridas, más US$0.11 de la
sonda `probes/probe-001`.

**Lectura por tramos:**

- **001–009: el aprendiz no compromete nada antes del feedback.**
  - La pérdida es siempre 5. La regla correcta aparece después, en Brookside
    revise, y queda en C(1,5), nunca en P.
  - La sonda probe-001 descarta que el modelo no sepa formular preguntas:
    sin herramientas, las tres réplicas nombran los derechos que faltan y
    planean consultas.
  - Las palancas de protocolo (pregunta obligatoria, borradores sin
    penalización, hallazgos, planificación, separar incertidumbre de
    investigación) cambian el discurso, pero no la conducta: el aprendiz
    planea operaciones y no las ejecuta.
- **010–012: la evaluación automática convierte borradores en compromisos.**
  - El aprendiz pasa a comprometer antes del feedback, siempre sobre la
    única evaluación que recibe.
  - Brookside depende entonces de qué forma sale primero. La invertida da
    pérdida 5 y revisión posterior (010, 012); la correcta da pérdida 0 y P
    (011).
  - Ni la atribución ni los soportes mínimos cambian ese patrón observable.
- **013: un modelo más capaz mejora la calidad, no la estructura.** Con
  Fable 5.1:
  - redacta una alternativa para ver su consecuencia y la descarta
    razonando con `rightCompliance`;
  - diseña h3 de Casa Fiesta de modo que no genere obligaciones de `agree`;
  - reporta hallazgos.

  Sigue sin hacer evaluaciones voluntarias ni consultas geométricas. Sigue
  dejando sin ejecutar lo que planea, por ejemplo evaluar h3 en Brookside o
  comparar con una segunda regla.

## 5. Capacidades efectivamente ejercitadas

| Capacidad | Ejercitada | Evidencia |
|---|---|---|
| Redacción de reglas Formal admisibles por el LLM | Sí | 001–013. Una sola elaboración rechazada (010 h3, parámetro sin uso), corregida por el aprendiz en el turno siguiente. |
| Admisión, certificado atribuido, verificador de clausuras | Sí | Toda hipótesis admitida quedó certificada. Ningún certificado falló. |
| Puntuación prequencial y estados V/C/P | Sí | C(1,5) en 003–012 y **P(1,5) en 011 y 013**. Ninguna hipótesis refutada. |
| DiscrepancyView y revisión tras discrepancia | Sí | 010 y 012: el `right` invertido queda reemplazado tras ver B7–B11 sin derivar. Es revisión por omisión, no por refutación. |
| Revisión por consecuencias formales | Parcial | 010: de h4 a h5 en Casa Fiesta. 013: descarte de h2 en Brookside. En ambos casos sobre la evaluación automática. |
| Evaluación voluntaria (`evaluate_rule` pedida por el aprendiz) | Casi nunca | Solo run-003 y run-005. Desde run-010 todas las evaluaciones son automáticas. |
| Navegación (forward, residual, inventario) | Mínima | run-005 (forzada por la compuerta) y run-013 (`inspect_theory`, voluntaria). |
| Geometría: `minimal_supports`, `compare_partition` | No por el aprendiz | Planeadas en 004–013 y nunca ejecutadas. Los soportes automáticos de run-012 y run-013 se calcularon y verificaron, sin uso observable. |
| Hallazgos verificables | Solo en 013 | 4 hallazgos; las 3 aserciones con `contains_event` son ciertas según los registros. El verificador las marca `mismatch` por la ambigüedad de formato (§6). |
| Comparación de alternativas antes de comprometer | Casi nunca | 013 en Brookside, por razonamiento y no con `compare_partition`. |

## 6. Limitaciones

**Corpus.**
- Una única señal (B7–B11), provisional y con reservas R-a y R-b.
- Casa Fiesta no tiene valoraciones y Flowers no tiene semillas.
- B7–B11 coinciden con nuestra convención de formalización (`contrast/`),
  así que acertar la orientación prueba alineación con esa convención, no
  aprendizaje jurídico.
- No se puede refutar ninguna regla, ni medir generalización a un
  expediente nuevo.

**Diseño experimental.**
- No se corrió el control G−F, así que el efecto del feedback no está
  medido.
- Hay una sola muestra por configuración y el primer borrador varía entre
  corridas (011 frente a 012). Las diferencias entre 010, 011 y 012 son
  compatibles con el azar del muestreo.
- Las rotaciones y los controles de fuga del contrato (vocabulario opaco,
  reconocimiento de casos) no se ejecutaron.

**Desviaciones respecto del contrato.**
- Presupuestos de 40 evaluaciones y 40 propuestas, en lugar de 8.
- Orden B→CF→F (el "secundario") desde run-003.
- Los cambios de protocolo de 003 a 012 son añadidos de texto o de interfaz,
  todos detrás de flags.
- `docs/pilot-004r-implementation.md` §3.4 todavía dice que el bridge no está
  hecho; el bridge se construyó después.

**Observabilidad del modelo.**
- Los bloques de razonamiento vienen vacíos (solo firma) tanto en Sonnet 5.5
  como en Fable 5.1.
- La interpretación se apoya en planes, motivos, abstenciones y hallazgos
  escritos, no en el razonamiento interno.

**Interfaz.**
- `contains_event` se verifica contra nombres de verbo, y la descripción no
  lo dice; las aserciones de run-013 son ciertas y aun así salen `mismatch`.
- Hasta run-011, `new_events` y `activated_base_rules` se calculaban contra O
  sola, mientras que `positions_contributed` se calculaba por dependencia.
  Esa inconsistencia la corrigió la atribución.
- Las dos quedan documentadas, no corregidas en el piloto congelado.

**Coste.** Los costes son estimaciones del driver con los precios del
manifest, no la factura.

## 7. Decisiones de arquitectura que surgieron

1. **Flags de manifest para todo cambio de protocolo.** Cada corrida
   anterior se reproduce con el binario actual.
   - Comprobación: los checks locales de 007 a 013 son exactos.
   - 003 a 006 solo difieren en campos que añadieron corridas posteriores,
     todos en cero o vacíos.
2. **Verificación local con proveedor simulado y diff de contrato antes de
   cada corrida pagada.** Cada configuración tiene su `contract.diff`, y su
   `run-NNN-local-check/` reproduce la corrida anterior bajo el flag nuevo.
3. **Un compromiso puede preceder a la evaluación y al feedback**, porque el
   estado P solo existe si se compromete antes del feedback. La evaluación
   automática tras `propose_rule` (run-010) fue la única intervención que
   cambió la conducta: produjo compromisos antes del feedback y P.
4. **Atribución contra una base declarada (run-011).**
   - Cada evaluación se mide contra `B = O ∪ (R_k \ {h})`, con
     `added_events` y justificaciones aportadas, incluidas las que recaen
     sobre eventos que la base ya derivaba.
   - Corrige la inconsistencia entre eventos medidos contra O y posiciones
     medidas por dependencia.
5. **Trazas de commit (run-011).** Registran lo que el aprendiz había visto
   al comprometer: feedback, evaluaciones con su base, navegación y
   hallazgos. Así se puede afirmar, por ejemplo, "comprometida sin feedback,
   con una sola evaluación".
6. **G(h, S, B) y soportes mínimos (run-012).**
   - Orden determinista de los goals:
     1. primero los eventos nuevos;
     2. luego las posiciones proyectadas;
     3. por último el texto Formal canónico, compartido por harness y bridge.
     Esa prioridad es una política de presentación, no geometría.
   - Omisiones registradas por separado: límite de presentación, presupuesto
     y análisis.
   - Hojas = semillas del expediente, no todo verbo `Seeded`, para que un
     `right` derivado no sea su propio soporte.
   - Presupuesto propio, separado del de navegación.
   - Exhaustividad argumentada por monotonía. Suficiencia y minimalidad
     verificadas por clausura. La completitud no se verifica de forma
     independiente.
7. **Preflight de modelo sin generación (run-013).** Consulta la Models API y
   hace `count_tokens` con el sistema y las herramientas exactos, antes de
   gastar.
8. **Higiene del ledger.**
   - La clave y el workspace no se registran, y no hay datos personales.
   - Se hace grep de secretos antes de archivar.
   - Las valoraciones solo aparecen en el informe final.

## 8. Qué deja el piloto para la etapa siguiente

**Lo que 004-R aporta y se puede reutilizar:**
- Un generador de hipótesis Formal admisibles, cada una con:
  - certificado;
  - consecuencias atribuidas a una base declarada;
  - soportes mínimos verificados;
  - historia de compromisos.
- Un protocolo reproducible y auditable para observar a un aprendiz LLM.

**Lo que no resolvió:** el aprendiz no organiza su exploración. Consume la
información que le llega sin pedirla, y no compara alternativas con las
herramientas disponibles.

**Retomar el Piloto 004.** La hipótesis original
(`docs/pilot-004-contract.md`) eran descripciones revisables de las
hipótesis, embeddings y navegación semántica entre candidatos. Esa hipótesis
encaja con esta carencia. La navegación entre hipótesis pasaría a ser una
estructura que el sistema mantiene, en vez de una decisión que se espera del
LLM. Los nodos serían las hipótesis Formal ya certificadas y atribuidas.

Dos condiciones heredadas de este informe:
- **Corpus:** hace falta uno con más posiciones valoradas y discriminantes
  por expediente, o el resultado volverá a ser no medible.
- **Control:** hay que planear desde el inicio el control sin feedback y las
  réplicas por configuración.
