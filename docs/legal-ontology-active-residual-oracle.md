# Oráculo jurídico de la consulta residual activa

La API activa de biblioteca recupera las fronteras mínimas de los casos
controlados examinados, incluidas sus condiciones de activación. El contraste
con la ontología completa mantiene la limitación de parámetros no determinados:
corregir activación no equivale a resolver existenciales o demostrar completitud.

Trabajo en `experiment-legal-order-geometry`, sin merge. No cambian el motor,
el cierre, defaults, la API Horn ni las declaraciones jurídicas. La biblioteca
exporta query_active_program_residual con su tipo de resultado independiente.
La adopción por la CLI sigue separada, según la
[revisión de integración](residual-active-query-integration-review.md).

## Fuentes y experimento

Se conservan los SHA-256 de las fuentes:

| Fuente | SHA-256 |
|---|---|
| examples/legal/modules/legal_monotonic.res | 24655c98ede7b128b8cd82df3a4473c114428b987bb0fe6a6d370c2b37f5575e |
| vendor legal_general.res de 12hectareas | 639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc |
| vendor legal_patrimonial.res de 12hectareas | f7daf4d5eb937a7fde8534dba5618acc50d5aaa5b05ff86c9656892eb3a03fbf |

Los datos siguen siendo sintéticos, no expedientes de 12hectareas. La fixture
[active_evidence_check.res](../examples/experiments/legal_geometry/active_evidence_check.res)
importa general.res y referencia la regla real conflictingActEvidence, sin
reescribirla:

```
Act(...) @ t & ~Act(...) @ t <= inconsistent(...) @ t
```

Para el tercer modelo añade únicamente una solicitud experimental Seeded,
inspectEvidence, asociada al Program EvidenceCheck. Sus ocho parámetros se
vinculan a los ocho parámetros de la regla importada mediante RuleUse. La
regla conserva literalmente su AST tipado, comprobado contra G y P.

**Este tercer modelo no es la ontología original con otro nombre.** Poner su
regla en una asociación cambia deliberadamente su disponibilidad: ahora necesita
una invocación. No se impone ese requisito globalmente a la ontología ni se
añade una norma jurídica. Se contrasta cada consulta con el operador de cierre
correspondiente a su propio módulo.

## Oráculo independiente del buscador residual

Para evidencia K, catálogo de semillas D y objetivo J se calcula:

\[
\mathcal H(K,J)=\min_{\subseteq}\{H\subseteq D\setminus K:
J\in Close_{\Pi}(K\cup H)\}.
\]

El primer oráculo enumera todos los subconjuntos de D, con a lo sumo tres hechos.
No utiliza query, sus árboles de prueba, su vista con guardas ni sus resultados
para escoger candidatos. Ejecuta close_program_reference, el evaluador anterior,
y minimiza las entradas suficientes por inclusión. Es independiente del
buscador abductivo; comparte el núcleo de matching/instanciación forward, por
lo que no certifica independientemente la corrección de todo el motor.

La consulta devuelve sus fórmulas y pruebas completas. Sólo el análisis convierte
sus hojas en conjuntos y extrae soportes mínimos para compararlas con el oráculo.
No se normaliza Tensor como conjunto ni se elimina provenance en el motor.
Suficiencia y recuperación de la frontera se comprueban por separado.

## Fronteras controladas

Los eventos exactos A y N son Act y ~Act con mismos argumentos, autoridad
privada, effective=gDate e índice gTau. T es la solicitud inspectEvidence con
esos mismos valores. J es el inconsistent ordinario correspondiente.

| Modelo | Catálogo D | K examinados | Frontera desde K vacío |
|---|---|---:|---|
| LegalGeneral original | A, N | 4 | {A,N} |
| LegalPatrimonial original | A, N | 4 | {A,N} |
| EvidenceCheck activado | A, N, T | 8 | {A,N,T} |

En total, 16 consultas y 45 cierres para enumerar completaciones sin redundancia
con K. En todos los casos coinciden las fronteras independientes y las extraídas
de las respuestas activas; cada respuesta es suficiente bajo su cierre real.

Por ejemplo, con K={A,N}, las ontologías originales devuelven I y el módulo
activado devuelve T. Cuando K={A,N,T}, el módulo activado devuelve I. No se
confunde evidencia vacía con Unit ni se obtiene N por ausencia de A.

El mismo caso K={A,N} da I en la consulta Horn histórica del módulo activado,
pues esa consulta considera la cláusula explícita sin guardas. Pero no alcanza
J en el cierre ejecutable sin T. Esto es un contraejemplo mínimo a identificar
la suficiencia Horn con suficiencia de Program: eliminar cualquiera de A/N
impide la prueba Horn; aportar T completa la prueba ejecutable.

Cada cierre certificado se compara además con el cierre de referencia sobre
la misma K ∪ H: eventos completos, tiempos, antecedentes ordenados,
sustituciones canónicas y multiplicidad de registros. Sólo derivedAt se excluye
por proceder de relojes independientes. La evidencia y el módulo permanecen
idénticos antes/después de la consulta.

La consecuencia del módulo activado conserva:

- Program: EvidenceCheck, módulo GeometryActiveEvidenceCheck;
- productor: EvidenceCheck::0::conflictingActEvidence;
- regla de origen: conflictingActEvidence, módulo LegalGeneral;
- testigos ordenados: solicitud, Act, ~Act.

Inconsistent no bloquea, no produce explosión ni elimina evidencia. Aquí se
deriva por la regla jurídica existente, no por una política global del motor.

## Cobertura sobre las ontologías completas

Se reutiliza sin ampliar el universo anterior: 73 eventos ground y 23 semillas
admisibles del estudio geométrico. El oráculo de fronteras propaga soportes sobre
las instancias ground reales del AST, verificadas previamente mediante cierre
y retirada de cada hecho. No enumera los 2^23 subconjuntos ni llama al buscador
residual. Este segundo contraste también es independiente de la búsqueda, no
de las primitivas forward compartidas.

Se consulta cada objetivo con evidencia vacía en G y P: 146 consultas.

| Resultado | General | Patrimonial |
|---|---:|---:|
| Consulta completada, frontera mínima coincide dentro del dominio | 30 | 29 |
| Consulta completada, con propuestas fuera del catálogo finito | 4 | 0 |
| Err por parámetro no determinado por el objetivo | 39 | 44 |
| Total | 73 | 73 |

Todas las respuestas publicadas contienen un cierre hipotético que deriva su
objetivo. En las consultas completadas, sus soportes mínimos dentro del catálogo
coinciden con la frontera. Las cuatro consultas fuera de catálogo proponen
patrimonialHolding para causas no incluidas entre las 23 semillas: son suficientes,
pero no se consideran recuperación de una frontera sobre otro dominio.

Los Err no son respuestas negativas ni conjuntos vacíos de explicaciones. La
columna missing del TSV, para esas filas, cuenta soportes geométricos que no
pudieron contrastarse mediante una respuesta completa; no contabiliza una
incompletitud demostrada de resultados que se hayan devuelto.

Persisten los ejemplos de compliance/violation/enforceability con conduct o
autoridad/fecha del acto no fijados por el head. Se registra el error real;
no se omiten reglas, preligan variables ni cambia la ontología para evitarlos.
Las asociaciones no aparecen en las fuentes jurídicas originales, por lo que
en esta parte los sidecars de activación están vacíos. El tercer modelo controlado
es el que comprueba específicamente el requisito de activación.

## Clasificación de la evidencia científica

- **Argumento general condicional:** una respuesta activa publicada es suficiente
  si termina la certificación original. Es su contrato de construcción, no un
  teorema de completitud ni de terminación.
- **Resultados finitos:** igualdad de las 16 fronteras controladas, certificados
  de provenance y clasificación de las 146 consultas reales.
- **No demostrado:** completitud para toda la ontología, minimalidad de todas
  las respuestas, resolución existencial o adjunción general con el álgebra L.

La geometría sirve como oráculo de corrección y cobertura en un dominio explícito.
No redefine la abducción, no elige explicaciones preferidas y no convierte los
Program en nuevas leyes jurídicas.

## Artefactos y verificación

- [Fronteras y certificados de los 16 casos](legal-geometry/data/active-legal-exhaustive-frontiers.tsv).
- [Clasificación de las 146 consultas](legal-geometry/data/active-legal-ontology-coverage.tsv).
- Tests: src/bin/residuum/cli/legal_geometry_active_frontier_tests.rs.

Pasaron los dos tests individuales mediante test_safe.py --binary, después de
compilar una sola vez el target CLI con DEBUG=0 y un trabajo. Se conservaron
1 GiB, sin swap, umbral preventivo 768 MiB, un hilo y los límites temporales.
No se ejecutó la suite completa.

Informe: `target/test-diagnostics/20261010T062831491496Z-21746`.
Fronteras controladas: 0,5907 s / 15,48 MiB RSS.
Cobertura de ontologías: 0,4487 s / 15,21 MiB RSS.

La integración de la API de biblioteca queda contrastada en este alcance; CLI,
merge y existenciales permanecen fuera de esta etapa.
