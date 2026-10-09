# Corrección y escala después de la integración incremental

Fecha: 2026-10-09. Rama: `experiment-peano-after`, worktree
`/home/maitreya/residuum-peano`. La integración quedó registrada primero en
`3bc5082e4786f075cd8e82bbdaabc866dd50dae3`, sobre `c7d2895`.
`main` y `v1^{commit}` conservan `8025bc6b4afee3415c652a5830576ef18aaf8cb5`.

Esta etapa agrega fixtures, verificadores y mediciones. No cambia el evaluador,
matching, canonicalización, índices, deduplicación ni la política de activación.
No añade otra optimización. Complementa
[la integración](program-semi-naive-integration.md).

## Corrección: cierres finitos y provenance completa

Las definiciones e inputs viven en los archivos `.res`; Rust compara y observa.

- [program_incremental_cycles.res](../examples/experiments/program_incremental_cycles.res)
  importa los programas Root/Branch existentes, incluida la aplicación parcial
  `RootOpen(stamp)`. Añade un ciclo finito `echo → cycleEcho → echo`.
- [program_incremental_scaling.res](../examples/experiments/program_incremental_scaling.res)
  importa la suma y multiplicación recursivas. Los inputs son spines y solicitudes;
  ningún resultado se entrega como evidencia.
- [program_proof_scaling.res](../examples/experiments/program_proof_scaling.res)
  mantiene dos eventos finales y produce 1, 4 o 16 justificaciones mediante
  RuleUses distintos del mismo productor. Cada RuleUse conserva su identidad.

Se compara el cierre frío incremental con `close_program_reference`, y cada
cierre intermedio de actualizaciones con la recomputación de la referencia.
El verificador compara:

```text
Event completo, incluidos tipo, tiempo, polaridad y transition
+ Rule + antecedentes ordenados + sustitución canónica
+ multiplicidad de justificaciones
```

Sólo excluye orden de descubrimiento y el valor concreto de `derivedAt` entre
ejecuciones. Dentro de una ejecución comprueba que las derivaciones y sellos
anteriores permanecen intactos. Los orígenes Program/Rule/Module se resuelven
con el mismo módulo importado. Los testigos deben pertenecer al cierre.
Repetir los seeds no genera eventos, derivaciones, activaciones ni sellos.

| Regresión nueva | Alcance y resultado |
|---|---|
| Ciclos, activaciones concurrentes y pruebas tardías | Cinco órdenes de incorporación; 25 eventos y las mismas 22 justificaciones |
| Sumas concurrentes | 25 solicitudes independientes de `2 + 2`; tres órdenes por lotes; 245 eventos / 200 justificaciones |
| Multiplicación que activa suma recursiva | `0×0`, `1×1`, `2×2`, `4×4`; tres órdenes por lotes; 106 eventos / 82 justificaciones; resultados 0, 1, 4, 16 |
| RuleUses distintos, evento compartido | Dos eventos finales; exactamente 1, 4 o 16 justificaciones distintas, sin perder sus orígenes |

Los lotes prueban tanto solicitudes anteriores a la evidencia como evidencia
anterior a las solicitudes, y activaciones de varios pedidos en el mismo
frontier. «Concurrentes» describe activaciones lógicas; el harness sigue usando
un único hilo.

En el ciclo se conserva una justificación de `echo` desde su programa y otra
desde `cycleEcho`, para cada persona. Una nueva justificación no es un nuevo
evento ni requiere expandir infinitos árboles de prueba.

También se repitieron las **448 comparaciones finitas existentes** del scheduler
integrado. Pasaron. Los tres ensayos del producto pequeño comparan las mismas
223 identidades de evento y las mismas 244 justificaciones completas.

Estos son resultados de modelos/ejecuciones finitos, no una demostración de
equivalencia universal o terminación de cualquier programa.

## Rendimiento: tres repeticiones seriales

Cada caso/evaluador se ejecutó en un proceso distinto mediante `test_safe.py`,
usando el mismo binario ya compilado. El profiler se mantiene activado en ambos.
Se realizaron **60 mediciones**: diez casos, dos evaluadores y tres repeticiones.

La tabla presenta medianas de tiempo de cierre instrumentado. Este tiempo
incluye preparación del comportamiento dentro de `close_program`, pero excluye
carga/parsing/elaboración del módulo, listados del harness y arranque del runner.
Los informes guardan por separado el tiempo total del test y pico RSS/cgroup.

| Caso | Eventos | RuleUses activados | Justificaciones | Referencia ms | Incremental ms |
|---|---:|---:|---:|---:|---:|
| Una suma `0+0` | 25 | 5 | 4 | 0.902 | 0.609 |
| Una suma `2+2` | 29 | 13 | 8 | 2.817 | 1.078 |
| Una suma `4+4` | 33 | 21 | 12 | 7.025 | 1.778 |
| Cuatro solicitudes `2+2` | 56 | 52 | 32 | 15.559 | 4.148 |
| Nueve solicitudes `2+2` | 101 | 117 | 72 | 57.258 | 14.237 |
| 25 solicitudes `2+2` | 245 | 325 | 200 | 349.872 | 77.488 |
| Cuatro multiplicaciones independientes | 106 | 117 | 82 | 223.031 | 17.831 |
| Un RuleUse, evento compartido | 2 | 1 | 1 | 0.208 | 0.325 |
| Cuatro RuleUses, evento compartido | 2 | 4 | 4 | 0.250 | 0.232 |
| 16 RuleUses, evento compartido | 2 | 16 | 16 | 0.419 | 0.362 |

Las series de una suma mantienen una solicitud y las mismas 20 evidencias de
spines; varía la profundidad. La serie de múltiples sumas mantiene operandos
`2+2` y esos mismos spines; varía sólo la cantidad de solicitudes con identidades
distintas. La serie de justificaciones mantiene dos eventos finales, aunque
RuleUses y justificaciones crecen juntos. No son tres variables independientes
y estos datos no permiten inferir una cota asintótica multivariable.

| Solicitudes `2+2` | Probes referencia → incremental | Duplicados referencia → incremental |
|---|---:|---:|
| 4 | 5 408 → 1 060 | 144 → 0 |
| 9 | 20 943 → 4 230 | 324 → 0 |
| 25 | 136 175 → 28 150 | 900 → 0 |

El crecimiento observado sigue siendo superlineal al añadir solicitudes,
incluso cuando no se producen candidatos duplicados. En las 25 solicitudes,
matching ocupa una mediana de 69.247 ms de los 77.488 ms; deduplicación ocupa
1.378 ms. El scheduler todavía recorre reglas activadas, y el índice por header
no restringe por bindings de argumentos. Esta observación orienta futuras
investigaciones; aquí no se modifica ninguno de esos mecanismos.

Los casos mínimos por debajo de un milisegundo son sensibles a ruido y al
overhead del profiler. No se promete una mejora en todos los casos: el caso de
un RuleUse resulta ligeramente más lento con el scheduler incremental.

## Repetición de productos polinómicos

Se ejecutaron seis tests adicionales en procesos separados: tres comparaciones
pequeñas y tres productos mayores incrementales. No se volvió a ejecutar la
referencia mayor que anteriormente agotó 15 segundos.

| Caso | Estado | Eventos / justificaciones | Mediana cierre | Intervalo de tres ejecuciones |
|---|---|---:|---:|---:|
| Pequeño, referencia | Punto fijo | 223 / 244 | 783.553 ms | 772.515–800.734 ms |
| Pequeño, incremental | Punto fijo equivalente | 223 / 244 | 64.062 ms | 63.604–68.276 ms |
| Mayor, incremental | Punto fijo | 1 149 / 1 262 | 2 019.221 ms | 1 951.611–2 088.705 ms |

El producto mayor mantiene 106 rondas y 1 188 RuleUses activados. En todas las
repeticiones se comprobaron los siete coeficientes `[5,10,18,26,9,12,0]` sin
sembrar coeficientes de salida. El test completo tardó 2.545–2.688 segundos;
pico RSS máximo: 55.37 MiB. No existe aquí un cierre completo de la referencia
mayor: **no se afirma equivalencia de su provenance final**.

En los tres ensayos pequeños los contadores fueron estables:
probes `93 306 → 5 483`, candidatos `6 246 → 297`, duplicados `6 002 → 53`.
La deduplicación lineal no se cambió.

## Recursos, informes y reproducción

Se mantuvieron 1 GiB total, sin swap, un trabajo Cargo, un hilo y 15 s por test,
con el umbral preventivo del runner. No se ejecutó la suite completa ni tests
fuera del runner. Ninguna medición llegó a un límite.

La primera compilación con todos los verificadores añadidos al target library
alcanzó el umbral preventivo; ningún test se ejecutó y no hubo OOM. Se separaron
los verificadores de corrección al target CLI existente, dejando sólo el test
instrumentado en library. Se compilaron ambos con los límites originales;
la compilación library posterior llegó a 695.03 MiB de pico cgroup, por debajo
del umbral. No se cambió código productivo ni flags para lograrlo.

Informes locales:

- `target/test-diagnostics/20261009T212148336072Z-451292`: compilación detenida.
- `target/test-diagnostics/20261009T212245294069Z-452522`: ciclos.
- `target/test-diagnostics/20261009T212723368456Z-459284`: sumas y multiplicidad.
- `target/test-diagnostics/20261009T213627393612Z-470980`: multiplicaciones y 448 comparaciones.
- `target/program-scaling-results.json`: 60 perfiles, recursos y paths de cada informe.
- `target/polynomial-program-repeat-results.json`: seis tests de productos y sus perfiles.

Compilar y probar individualmente, desde el worktree experimental:

```sh
python3 scripts/test_safe.py --test-target cli --test cli::tests::program_validation::finite_cycles_concurrent_activations_and_late_proofs_preserve_equivalence
python3 scripts/test_safe.py --test-target cli --test cli::tests::program_validation::recursive_addition_concurrent_requests_preserve_all_twenty_five_answers_and_proofs
python3 scripts/test_safe.py --test-target cli --test cli::tests::program_validation::recursive_multiplication_and_addition_continuations_preserve_independent_results
python3 scripts/test_safe.py --test-target cli --test cli::tests::program_validation::distinct_rule_uses_preserve_multiple_proofs_without_extra_events
python3 scripts/test_safe.py --test derive::profile_tests::bounded_program_scaling_observes_work_without_changing_evaluation
```

Después, indicar el binario library que informó el runner (sin recompilar):

```sh
python3 scripts/benchmark_program_scaling.py --binary target/debug/deps/residuum-cfe44311058885c9 --repetitions 3
```

El script ejecuta exclusivamente el test instrumentado individual mediante
`test_safe.py`, en serie, y se detiene ante cualquier fallo/límite. Los perfiles
pequeños/grandes pueden repetirse individualmente con los tests existentes:

```sh
python3 scripts/test_safe.py --binary target/debug/deps/residuum-cfe44311058885c9 --test derive::profile_tests::semi_naive_product_preserves_exact_provenance_and_reports_matching_cost
RESIDUUM_PROFILE_PRODUCT=principalProduct python3 scripts/test_safe.py --binary target/debug/deps/residuum-cfe44311058885c9 --test derive::profile_tests::profile_semi_naive_polynomial_product_fixture
```

Las equivalencias finitas y las mediciones de escala quedan separadas. Ninguna
dependió de completar el producto mayor en la referencia lenta.
