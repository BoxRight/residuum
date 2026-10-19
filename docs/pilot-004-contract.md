# Contrato del Piloto 004: revisión semántica tras una refutación verificable

Estado: **VERSIÓN 2, APROBADA PROVISIONALMENTE PARA LA REVISIÓN DE VALORACIONES** (2026-10-19). Sustituye a la
versión 1 (`e28fbdf`, §13), que se rechazó por baja capacidad discriminante.
La versión 2 solo revisa de forma estructural los expedientes y la selección
de nodos iniciales. La gramática y las 534 candidatas no cambian. Las
valoraciones se pedirán después de que revises esta versión. No se ha ejecutado el ciclo de
revisión, no se ha elegido proveedor de embeddings ni modelo, y no se ha hecho
ninguna llamada pagada.

## 0. Pregunta y alcance

> ¿Puede un LLM revisar la descripción semántica de una configuración tras una
> refutación verificable, y usar esa revisión para seleccionar una
> generalización que prediga mejor un expediente nuevo?

**Fuera de alcance:** inferencia causal, aprendizaje jurídico general y
recuperación de una justificación desconocida.

**Éxito,** según los criterios del usuario:

| Criterio | Qué tiene que pasar |
|---|---|
| Revisión real | Una refutación cambia una descripción y modifica de forma verificable la ordenación de candidatas. |
| Consecuencia efectiva | La candidata seleccionada produce posiciones existentes mediante Residuum. |
| Generalización prospectiva | La revisión mejora, o al menos permite contrastar, la predicción sobre el expediente reservado. |

**Evidencia:**
- Las 4 rotaciones comparten expedientes y valoraciones. **No son
  observaciones estadísticamente independientes**, y no se presentan como
  tales.
- Las muestras de revisión miden **estabilidad**, no ventaja estadística.

## 1. Artefactos congelados

Directorio `docs/legal-geometry/pilot-004/`. Los hashes están en
`MANIFEST.sha256` y se comprueban con `sha256sum -c`.

| Artefacto | SHA-256 |
|---|---|
| `frozen/vocabulary.res` | `f1b87199d802f1fa112a98949ec220f00fad743208ffd60161d9fa0901df42a5` |
| `frozen/dossiers.res` (v2) | `ecc58e13257273db2d5a9fc9ce857c4ecfdf17cd53db334c088819b599b7ba74` |
| `frozen/glosses.tsv` | `f6fcd209fae789892f26ffb9c033b0849f584e8cabcc5e9833c062e02c626b93` |
| `frozen/catalog_prohibition.res` | `e2ff4cdc9fff83ac302906ee4e8a74fbfe0c574c8efd8b61cf277b3d0cca2638` |
| `frozen/catalog_permission.res` | `eaab4ef6ae532eea342d116e4f92846f0944dccbd84f38b75ccd439f14bdf1cb` |
| `frozen/catalog_obligation.res` | `60ce5a10fc023afd41573bf08c7dddeed9c32b37b0a89c25ccdcb947a8759271` |
| `frozen/descriptions.tsv` | `67ef05bcd6a7b206870e778ae57754d84969c48f5d89b9138eaf5f04d194f44d` |
| `frozen/protocol.tsv` (v2) | `fe371aa0305de32b3865fb51417b41ff1e55f3668bccc3721d164b727753865d` |
| `tools/pilot004/generate_catalog.py` | `135309fe77a508405c1eda1f1bdf1c324dc2870361c98e881727c77b2a10de08` |
| `tools/pilot004/design_dossiers.py` (v2) | `a079f0995128da29ade0852826ed3dc30ac61b41d028095f760592829c857804` |
| LegalGeneral original (sin cambios) | `639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc` |

Al volver a ejecutar el generador de candidatas y el diseñador de
expedientes se obtienen archivos con hashes idénticos (COMPROBADO para los
dos).

## 2. Dominio

$T$ = LegalGeneral (original) más `Pilot004Vocabulary`.

El vocabulario solo contiene firma: tres extensiones descriptivas generales,
sin reglas ni términos deónticos o evaluativos.

| Firma | Significado introducido |
|---|---|
| `copresent(first : Person, second : Person)` | Las dos personas están en el mismo lugar en el momento del expediente. Se registra en ambas orientaciones. |
| `condition(person : Person, condition : PersonCondition)` | El expediente registra una condición de la persona. |
| `kind(thing : Thing, kind : ThingKind)` | El expediente registra la clase de la cosa. |

Las constantes son instancias:
- `PersonCondition`: `allergic`, `visuallyImpaired`;
- `ThingKind`: `animal`, `food`;
- `Conduct`: `enterWith`;
- norma hipotética: `hypNorm : Fact`, `hypEnacted : PropositionTime`.

**Glosas** (`frozen/glosses.tsv`):
- Describen las construcciones generales de LegalGeneral y del vocabulario y
  sus argumentos, sin tratamientos.
- Las redactó el asistente a partir de las firmas y comentarios de
  LegalGeneral. Quedan sujetas a tu revisión, y si cambian se congelan como
  versión nueva.

**Quedan fuera** de este piloto: edad, certificaciones, tenencia, relaciones
negociales, `constitutes` y `certifies`, la conexión `before`/`notEarlier`,
el vocabulario opaco y las composiciones con argumentos ficticios.

## 3. Los cuatro expedientes (retrospectivos, versión 2)

**Criterio estructural declarado antes de la búsqueda, sin etiquetas**
(`tools/pilot004/design_dossiers.py`, semilla 4004, 60 reinicios × 6000
iteraciones de búsqueda local):

1. **Condiciones obligatorias:**
   - admisibilidad K1–K5;
   - un anclaje `Act` por expediente;
   - copresencia registrada en ambas orientaciones;
   - **contraste relacional**: `dossier1` y `dossier2` tienen atributos
     individuales idénticos y difieren solo en copresencia;
   - **cobertura de literales**: cada literal que puede aparecer solo se
     satisface en al menos un expediente y falla en al menos otro.
2. **Objetivo, en este orden:**
   - maximizar el número de patrones distintos de predicción del catálogo;
   - minimizar las candidatas inertes;
   - minimizar el número de hechos sembrados.

El objetivo es simétrico sobre las candidatas: no favorece a ninguna. El
emparejador de la búsqueda solo propone el diseño, y las cifras congeladas
las certifica `close_program`. En las 534 candidatas, los patrones del
emparejador coinciden con los del motor.

**Reparto:** `ana` es el sujeto y `bea` la beneficiaria; `cal`, `dan` y la
cosa `obj` completan el reparto. El anclaje común es
`Act(actN, ana, enterWith, obj, bea, private, dateN) @ tN`.

| Expediente | Hechos descriptivos (la copresencia se registra en ambas orientaciones) |
|---|---|
| `dossier1` | `condition(ana, allergic)`, `~condition(ana, visuallyImpaired)`, `~condition(bea, allergic)`, `condition(bea, visuallyImpaired)`, `kind(obj, animal)`, `~kind(obj, food)`, `copresent(ana, bea)` |
| `dossier2` | igual que `dossier1`, salvo `~copresent(ana, bea)` → **contraste relacional** |
| `dossier3` | `~condition(ana, allergic)`, `condition(ana, visuallyImpaired)`, `~condition(bea, allergic)`, `~condition(bea, visuallyImpaired)`, `kind(obj, animal)`, `kind(obj, food)`, `copresent(ana, bea)`, `copresent(ana, cal)`, `~copresent(ana, dan)`, `~copresent(bea, cal)`, `copresent(bea, dan)` |
| `dossier4` | `~condition(ana, allergic)`, `condition(ana, visuallyImpaired)`, `condition(bea, allergic)`, `~condition(bea, visuallyImpaired)`, `condition(cal, allergic)`, `~kind(obj, animal)`, `~kind(obj, food)`, `~copresent(ana, bea)`, `copresent(ana, cal)`, `~copresent(ana, dan)`, `copresent(bea, cal)`, `~copresent(bea, dan)` |

**Estructura certificada** (informe `20261019T010013659961Z-1080357`):

| Medida | Versión 1 | Versión 2 |
|---|---|---|
| Clases de predicción de las 534 candidatas | 19 | **43** |
| Candidatas que no derivan ninguna posición | 498 (93,3 %) | **66 (12,4 %)** |
| Candidatas de un antecedente que varían entre expedientes | — | **54 de 54** |

Las 43 clases son 14 patrones activos por cada uno de los 3 tipos de cabeza,
más una única clase inerte común, porque la firma vacía es la misma para las
tres cabezas. Hay 15 patrones de cuerpo en total.

**Decisiones del usuario (2026-10-19) y comprobación de las glosas:**
- **`dossier3` se conserva.** Las glosas congeladas no hacen incompatibles
  las dos clases (COMPROBADO):
  - `kind` dice «The dossier records that the thing is of the kind»: registra
    una clase sin exclusividad;
  - `animal` es «an animal»;
  - `food` es «food or drink», sin la condición de estar preparado para el
    consumo.

  El catálogo ya contiene candidatas con `kind(o, animal) & kind(o, food)`,
  cuyo $d_0$ es «the thing is an animal, and the thing is food or drink». No
  se añade ninguna exclusión semántica.
- **Variedad relacional limitada (registrada, no bloqueante).** El contraste
  relacional `dossier1`/`dossier2` es la copresencia entre sujeto y
  beneficiaria, y `cal` y `dan` solo aparecen en `dossier3` y `dossier4`.
- **Interpretación de las 43 clases.** Son 43 patrones distintos de resultado
  de las candidatas sobre los cuatro expedientes, no 43 justificaciones.
  Garantizan alternativas que se pueden distinguir por observación, pero no
  que las descripciones permitan elegir mejor entre ellas.
- **Información prospectiva.** Las cuatro rotaciones sirven para probar el
  mecanismo, no para demostrar una ventaja general.
- No se busca más optimización estructural.

**Posiciones observables y consecuencias:**
- La **pérdida primaria** se calcula sobre `prohibition`, `permission` y
  `obligation` del acto.
- Las **consecuencias constitutivas** (`violation`, `compliance`, `notDo` y
  `do`) se calculan y se conservan en el cierre y en la provenance, y se
  verifican. El test comprueba en las 2.136 evaluaciones que una prohibición
  produce `violation` y `notDo`, que una obligación produce `compliance` y
  `do`, y que no aparecen en otro caso.
- Las consecuencias no forman parte de la pérdida primaria.

## 4. Gramática y catálogo de candidatas

```text
rule id(a, s, c, o, b, au, e, [x,] t) =
    Act(a, s, c, o, b, au, e) @ t & L1 @ t [& L2 @ t]
    <= K(hypNorm, s, c, o, b, administrative, hypEnacted) @ t
K   ::= prohibition | permission | obligation
L   ::= [~] condition(P, allergic | visuallyImpaired)
      | [~] kind(o, animal | food)
      | [~] copresent(P, P')         con (P, P') ∈ {(s,b), (s,x), (b,x)}
P   ::= s | b | x
```

**Condiciones de admisibilidad:**

| | Condición |
|---|---|
| H1 | Elabora con el elaborador real. |
| H2 | Cabeza segura: $s, c, o, b$ quedan ligadas por el anclaje. |
| H3 | El tercero $x$ solo aparece si un literal `copresent` lo vincula. |
| H4 | No aparece un átomo con las dos polaridades. |
| H5 | Hay un único índice `@ t`, y las fechas solo aparecen como argumentos. |
| H6 | No se usan records ni variables ligadas a campos opcionales; esto evita la laguna de tipado de `Relation.thing`. |

**Copresencia:** se usa un orden canónico de pares, porque los expedientes la
registran en ambas orientaciones.

**El tercero $x$** puede ligarse a `ana` o `bea` si los hechos lo permiten,
porque Formal no tiene desigualdad. Es semántica del motor y no se corrige.

**Generación** (`tools/pilot004/generate_catalog.py`), exhaustiva y anterior
a las valoraciones:
- 18 cuerpos de un antecedente y 160 de dos, 178 en total;
- multiplicados por 3 cabezas, dan **534 candidatas** (`p4c001`–`p4c534`).

El catálogo no se ha diseñado ni ajustado para que contenga una solución. Si
el espacio compatible resulta vacío, se registra como resultado.

**Verificación** (`catalogue_is_the_complete_admissible_grammar…`):
- una enumeración **independiente** en Rust coincide con el catálogo para
  cada cabeza;
- las formas canónicas (variables renombradas por primera aparición, sin
  nombre de regla) son únicas;
- se cumplen H2–H6.

## 5. Descripciones iniciales deterministas $d_0$

- Columna `d0` de `frozen/descriptions.tsv`, en inglés, por compatibilidad
  léxica con los identificadores.
- Plantilla: «If an act is recorded in which the subject performs a conduct
  with a thing for a beneficiary, and ⟨literal⟩[, and ⟨literal⟩], then
  ⟨posición⟩.»
- Cada literal se redacta a partir de su glosa: «a person X» en la primera
  mención del tercero y «X» en las siguientes.
- El test vuelve a calcular cada $d_0$ en Rust a partir de la regla tipada, y
  coincide con el archivo en las 534 candidatas.
- $d_0$ es la componente formal $f$, fija. La componente revisable empieza
  vacía: $h_0=\varnothing$.
- Texto que se embebe: $d_0$ y después una línea por afirmación de $h_k$.

## 6. Comparación de posiciones y pérdida

- **Posiciones derivadas:** $N_h(S)$ es el conjunto de tipos de posición
  derivados sobre los participantes del acto del expediente, en
  `close_program` de $T\oplus h$.
- **Proyección:** se descartan `id`, `authority` y `effective`, que valen
  `hypNorm`, `administrative` y `hypEnacted` por construcción. Los tests
  comprueban esos valores y que las posiciones solo se refieren a los
  participantes del acto, en las 2.136 evaluaciones.
- **Valoración por expediente:** $P^+,P^-\subseteq$ {prohibition, permission,
  obligation}, con $P^+\cap P^-=\varnothing$. Los tipos no mencionados quedan
  sin valorar.
- **Pérdida:** $\ell=|P^+\setminus N|+|P^-\cap N|$. El cierre es finito, así
  que no hay intervalos.

## 7. Política de navegación (fija para B, C y D)

- **Episodio:** un par (rotación, nodo inicial).
  - Rotaciones: el expediente reservado es $r\in\{1,2,3,4\}$, y los otros
    tres son los observados.
  - Nodos iniciales (versión 2): las candidatas de un antecedente, ordenadas
    por FNV-1a 64 de `"pilot-004/start/" + id`. Se conservan las que derivan
    una posición en al menos 1 y como mucho 3 de los 4 expedientes
    (actividad inferencial mínima, certificada con `close_program`), y se
    toman las 3 primeras: **`p4c188`, `p4c189` y `p4c184`**. FNV-1a sustituye
    a SHA-256 para que el test Rust la recalcule sin dependencias. La regla
    no usa valoraciones.
  - **Limitación registrada (decisión del usuario, 2026-10-19).** Los tres
    nodos tienen cabeza `permission`, de modo que todas las trayectorias
    parten de la misma clase de posición. No se cambia la selección: forzar
    variedad sería una decisión de diseño tomada después de ver el resultado.
- **Trayectoria** (presupuesto de 8 evaluaciones):
  1. Evaluar el nodo actual sobre los expedientes observados.
  2. Si $\ell=0$ en todos, parar y devolverlo.
  3. Si no, tomar como discrepancia el primer expediente observado (en orden
     ascendente) con $\ell>0$.
  4. Según la condición: B no revisa; C revisa $h$ del nodo con la vista
     I2; D reescribe $h$ sin vista.
  5. Siguiente nodo: el de máximo coseno entre $z(\text{actual})$ y $z(u)$
     entre las no evaluadas, con empates resueltos por id ascendente.
- **Si se agota el presupuesto:** se devuelve la evaluada con menor pérdida
  observada; los empates se resuelven por orden de evaluación.
- **Prueba prospectiva:** la salida se evalúa sobre el expediente reservado.
  Su valoración se revela solo después de registrar la predicción.
- **Muestras:** 3 muestras independientes por llamada de revisión en C y D,
  lo que da 3 trayectorias por episodio. B es determinista.
- **Medidas:**
  - pérdida prospectiva;
  - evaluaciones hasta la primera compatible;
  - cambio en la ordenación de vecinos del nodo refutado antes y después de
    revisar (Spearman, RBO, cambio del primer vecino);
  - estabilidad entre muestras;
  - proporción de revisiones que no cambian la selección;
  - rechazos del validador.

## 8. Protocolo de información

**Vista I2 para C,** construida a partir de la discrepancia:
- la configuración formal del nodo y su $d_0$;
- la componente $h$ vigente;
- las glosas;
- las semillas del expediente observado que falló;
- los tipos valorados de ese expediente;
- las posiciones producidas, con productor y antecedentes.

**D** recibe la configuración, $d_0$, $h$ vigente y las glosas, pero **no**
la discrepancia. La longitud está igualada.

**Prohibido en las tres condiciones:**
- el expediente reservado y su valoración;
- las valoraciones de los otros expedientes observados que no son la
  discrepancia;
- el catálogo y las descripciones de otras candidatas;
- la tabla de predicciones;
- el objetivo sintético, si se usa.

**Salida de la revisión:** una lista de afirmaciones
`{texto, procedencia ∈ {formal, consequence, valuation, hypothesis}, referencia}`.

El validador rechaza una revisión que:
- cambie Γ;
- mencione términos ground (`ana`, `bea`, `cal`, `obj`, `act1-4`, `date1-4`,
  `t1-4`, `dossier1-4` o ids `p4c`);
- use verbos ajenos al vocabulario o a LegalGeneral;
- supere las 120 palabras;
- se autovalore.

Toda vista enviada y toda respuesta se registran en el ledger.

## 9. Lo que no se congela todavía

- proveedor de embeddings y modelo de lenguaje;
- texto de las instrucciones;
- presupuesto de coste;
- objetivo sintético para la prueba en seco.

Se decidirán después de certificar el harness sin LLM.

## 10. Requisitos para las valoraciones (cuando las proporciones)

- Por cada expediente: $P^+$ y $P^-$ sobre los tres tipos de posición del
  acto, más la fuente y, opcionalmente, el fundamento.
- Se entregan **después** de esta congelación y **sin consultar** las
  predicciones de las candidatas, que no se han publicado.
- Se registran con su propio hash antes de cualquier ejecución del ciclo.

## 11. Verificación

Tests individuales con `test_safe.py --test-target cli`:

| Test (versión 2) | Informe |
|---|---|
| `pilot_vocabulary_is_a_general_descriptive_signature_with_glosses` | `20261019T010009682914Z-1080154` |
| `pilot_dossiers_are_admissible_and_include_a_relational_contrast` | `20261019T010010400907Z-1080179` |
| `catalogue_is_the_complete_admissible_grammar_with_reproducible_descriptions` | `20261019T010011119911Z-1080201` |
| `every_candidate_runs_in_isolation_with_constant_projected_arguments` | `20261019T010013659961Z-1080357` |

## 12. Integración del explorador

Ver `docs/pilot-004-explorer-integration.md`.

## 13. Historial de versiones

| Versión | Commit | Cambios | Hash de `dossiers.res` | Hash de `protocol.tsv` |
|---|---|---|---|---|
| 1 | `e28fbdf` | inicial; rechazada por baja capacidad discriminante (498 candidatas inertes, 19 clases) y por nodos iniciales inertes | `9dd5be45…` | `6d36af85…` |
| 2 | este commit | expedientes rediseñados con un criterio estructural; nodos iniciales con actividad mínima; consecuencias constitutivas verificadas; gramática, catálogo, vocabulario, glosas y descripciones sin cambios | ver §1 | ver §1 |
