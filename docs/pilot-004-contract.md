# Contrato del Piloto 004: revisión semántica tras una refutación verificable

Estado: **CONGELADO PARA REVISIÓN** (2026-10-18). Las valoraciones se pedirán
después de que revises los expedientes. No se ha ejecutado el ciclo de
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
| `frozen/dossiers.res` | `9dd5be455c28ba9a9f3d0458d00470d896589dfcf3b420b30f360fb2c0d8cf36` |
| `frozen/glosses.tsv` | `f6fcd209fae789892f26ffb9c033b0849f584e8cabcc5e9833c062e02c626b93` |
| `frozen/catalog_prohibition.res` | `e2ff4cdc9fff83ac302906ee4e8a74fbfe0c574c8efd8b61cf277b3d0cca2638` |
| `frozen/catalog_permission.res` | `eaab4ef6ae532eea342d116e4f92846f0944dccbd84f38b75ccd439f14bdf1cb` |
| `frozen/catalog_obligation.res` | `60ce5a10fc023afd41573bf08c7dddeed9c32b37b0a89c25ccdcb947a8759271` |
| `frozen/descriptions.tsv` | `67ef05bcd6a7b206870e778ae57754d84969c48f5d89b9138eaf5f04d194f44d` |
| `frozen/protocol.tsv` | `6d36af8725d5b20d67373fd7338afc4d3823405a6920faca22501f2714d278b0` |
| `tools/pilot004/generate_catalog.py` | `135309fe77a508405c1eda1f1bdf1c324dc2870361c98e881727c77b2a10de08` |
| LegalGeneral original (sin cambios) | `639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc` |

Al volver a ejecutar el generador se obtienen archivos con hashes idénticos
(COMPROBADO).

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

## 3. Los cuatro expedientes (retrospectivos)

Cada expediente registra un acto
`Act(actN, ana, enterWith, obj, bea, private, dateN) @ tN`: `ana` entró
llevando `obj` en relación con `bea`.
- `tN` es el momento del expediente.
- `dateN` es la fecha del acto y solo aparece como argumento.

| Expediente | Hechos descriptivos | Diferencia con `dossier1` |
|---|---|---|
| `dossier1` | `kind(obj, animal)`, `condition(cal, allergic)`, `copresent(cal, bea)`, `copresent(bea, cal)` | — (base) |
| `dossier2` | `kind(obj, animal)`, `condition(cal, allergic)`, `~copresent(cal, bea)`, `~copresent(bea, cal)` | **relación entre participantes**: hay evidencia explícita de no copresencia; los atributos individuales son idénticos |
| `dossier3` | `kind(obj, food)`, `condition(cal, allergic)`, `copresent(cal, bea)`, `copresent(bea, cal)` | clase de la cosa |
| `dossier4` | los de `dossier1` más `condition(ana, visuallyImpaired)` | condición del sujeto |

**Principio de diseño:** variaciones de un solo factor sobre una base, que
cubren una vez cada extensión. Se fijaron antes de cualquier valoración y no
dependen de qué tratamiento se espere.

**Posiciones observables que habría que valorar** en cada expediente $N$:

| Posición | Consecuencias que deriva LegalGeneral (en 𝒫, no se valoran aparte) |
|---|---|
| `prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ tN` | `violation` (con el `Act`) y `notDo @ after tN` |
| `permission(…)` | — |
| `obligation(…)` | `compliance` (con el `Act`) y `do @ after tN` |

Se valoran solo los tipos de posición. Las consecuencias son deterministas
dadas la posición y el acto, así que valorarlas no añadiría información.

**Lo que muestra la estructura, calculada sin valoraciones** (test
`every_candidate_runs_in_isolation…`, informe
`20261018T234127183212Z-1018144`):
- Las 534 candidatas inducen **19 clases** de predicción sobre los 4
  expedientes.
- **498 candidatas no derivan ninguna posición** en ningún expediente.

El motivo es que los expedientes no contienen evidencia negativa de
condiciones o clases, ni copresencia del sujeto. Los literales negativos de
`condition` y `kind`, y los de `copresent(s, ·)`, nunca se activan.

**Tienes que revisarlo antes de valorar.** Es una propiedad del diseño de los
expedientes, no de las etiquetas. Si se amplían los expedientes con hechos de
ese tipo, sería una nueva versión congelada, decidida por cobertura
estructural.

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
  - Nodos iniciales: las candidatas de un antecedente, ordenadas por
    `sha256("pilot-004/start/" + id)`, tomando las 3 primeras: `p4c365`,
    `p4c002` y `p4c190`. Es una regla anterior a las valoraciones e
    independiente de ellas.
  - En total, 12 episodios por condición.
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

| Test | Informe |
|---|---|
| `pilot_vocabulary_is_a_general_descriptive_signature_with_glosses` | `20261018T234035388723Z-1017159` |
| `pilot_dossiers_are_admissible_and_include_a_relational_contrast` | `20261018T234041167133Z-1017465` |
| `catalogue_is_the_complete_admissible_grammar_with_reproducible_descriptions` | `20261018T234041782707Z-1017485` |
| `every_candidate_runs_in_isolation_with_constant_projected_arguments` | `20261018T234127183212Z-1018144` |

## 12. Integración del explorador

Ver `docs/pilot-004-explorer-integration.md`.
