# Integración incremental de Program en la rama experimental

Rama: `experiment-peano-after`. Checkpoint previo: `c7d2895`, que conserva
canonicalización, índice de Known, profiling y el contrato incremental.
`main` y el commit del tag `v1` siguen en `8025bc6`; no se modificaron.

## Alcance y API

`close_program` usa ahora `IncrementalProgramClosure`. El evaluador anterior
permanece como `close_program_reference`, un oráculo experimental con la misma
preparación de módulos y comportamientos asociados.

```rust
let mut state = IncrementalProgramClosure::new(seeds, &module, &mut clock)?;
let change = state.update(additions, &mut clock);
let closure = state.closure();
```

El módulo resuelto queda fijo. No se agregan/remueven declaraciones, imports,
defaults ni reglas desde esta API. Se añaden hechos, y las instancias positivas
conocidas de Seeded/Derived activan los comportamientos ya declarados. Effect
no recibe una política nueva y no se ejecuta State.

`ProgramClosureDelta` distingue:

- `known`: eventos nuevos, incluidos seeds añadidos;
- `derivations`: nuevas justificaciones, incluso de eventos ya conocidos;
- `activations`: RuleUses activados, con BehaviorRef, regla, trigger y bindings;
- `stats` y `full_initializations`: observaciones del trabajo realizado.

El estado interno conserva activaciones pendientes de inicializar. Esto es
trabajo independiente del delta de eventos; no se representa falsamente como
un hecho nuevo.

## Scheduler y provenance

1. Los seeds únicos entran en Known y su índice existente.
2. Los triggers nuevos descubren activaciones. La declaración del verbo, sin
   una instancia positiva conocida, no activa nada.
3. Cada activación nueva evalúa todos sus RuleUses inicialmente contra **todo
   Known**, con bindings canónicos precargados. Incluye cuerpos `I` y evidencia
   anterior al trigger.
4. Las activaciones ya inicializadas y reglas ordinarias buscan testigos con
   al menos un evento del frontier. Cada hoja del antecedente puede ser pivote;
   las otras usan Known. Se restaura el orden original de antecedentes.
5. Se conservan todas las justificaciones distintas. Sólo un evento lógico
   realmente nuevo entra en el siguiente frontier; una justificación nueva
   de un evento existente no reactiva automáticamente sus consumidores.
6. El trabajo termina cuando no quedan eventos nuevos ni inicializaciones
   pendientes. `update`/`close_program` sólo retornan un cierre terminado.

Las reglas ordinarias se inicializan una vez contra Known; así se preservan
reglas ground con antecedente `I`, incluso sin seeds. Todos los RuleUses
participan en un único cierre junto al entorno, no en cierres independientes.

Se reutiliza la canonicalización de `c7d2895`. Identidad de Rule/Program/Module,
constantes frente a parámetros, tiempos, sustituciones y testigos ordenados
permanecen en la provenance. Se conserva el criterio previo para incluir el
trigger de activación: no se repite si el cuerpo ya lo cita.

No se cambia Eq de DerivationRecord, ni se añade deduplicación hash. El vector
de justificaciones y sus comparaciones existentes siguen siendo el criterio
de retención. El índice por verbo, polaridad y aridad es el del checkpoint
previo y ahora también restringe el acceso al frontier.

## Argumento de programación del trabajo

Bajo las condiciones de este fragmento positivo, con módulo fijo y cierre
terminante, un testigo habilitado para una regla ya inicializada que no estaba
disponible antes debe usar algún evento nuevo. Enumerar cada posición como
pivote cubre esas combinaciones. Una regla recién activada no cumple esa
premisa, por lo que su primera evaluación debe usar Known completo. Una
justificación nueva no modifica la evidencia usada por los matches: las
justificaciones son registros de inferencias inmediatas, no árboles de todas
las pruebas transitivas de cada premisa.

Éste es el argumento del scheduler, no una afirmación de completitud residual,
terminación universal ni equivalencia matemática demostrada de todas las
extensiones futuras. La evidencia experimental se delimita abajo.

## Equivalencia comprobada

La comparación exige los mismos eventos completos, incluyendo tipos y
metadata de transición, y las mismas justificaciones con multiplicidad:

```text
Event + Rule + ordered antecedents + canonical substitution
```

Sólo se ignoran orden de descubrimiento y valor concreto de `derivedAt`.
Actualizaciones conservan intactos los records y sellos anteriores. En el
oráculo guardado se mantiene únicamente la corrección de su guard redundante;
no normaliza bindings ni fusiona justificaciones al comparar.

| Verificación | Resultado |
|---|---|
| Universo finito de seis hechos, scheduler integrado | 448 comparaciones exactas |
| Modelo anterior de reglas guardadas | Sus 448 comparaciones siguen pasando |
| Temporal mínimo, constantes y constructores | Una justificación, ambos pivotes y órdenes de actualización |
| Dos Program distintos y dos testigos | Se conservan cuatro justificaciones |
| Inicializaciones pendientes sin ΔEvents | Se recupera sólo la justificación pendiente sobre Known existente |
| Nueva prueba de un evento antiguo | Se registra sin duplicar pruebas de sus consumidores |
| Seeds duplicados / actualización vacía | No producen trabajo ni nuevos sellos |
| Imports, partial application, records, explain, aislamiento | Regresiones individuales pasan |
| Producto pequeño, frío y actualización | 223 eventos y las mismas 244 justificaciones |

El test de inicialización pendiente construye un checkpoint **sólo en tests**:
Known ya contiene el cierre completo y falta una justificación de un RuleUse.
Con frontier vacío se inicializan activaciones y se recupera esa justificación,
sin introducir eventos. No se añade una API pública para manipular provenance.

## Medición con los mismos límites

Producto pequeño, ambos evaluadores terminaron y coincidieron exactamente:

| Observación | Referencia indexada | Incremental integrado |
|---|---:|---:|
| Eventos / justificaciones | 223 / 244 | 223 / 244 |
| Rondas | 36 | 36 |
| RuleUses activados | 226 | 226 |
| Intentos de matching | 93 306 | 5 483 |
| Rechazos por estructura | 81 277 | 4 803 |
| Candidatos de justificación | 6 246 | 297 |
| Candidatos ya registrados | 6 002 | 53 |
| Comparaciones de deduplicación | 607 805 | 31 577 |
| Matching | 702.094 ms | 41.998 ms |
| Tiempo del cierre instrumentado | 802.431 ms | 62.489 ms |

Son observaciones de una ejecución, no cotas generales ni una promesa de
aceleración. Todavía se recorren reglas/activaciones en cada ronda; no se
añadió un nuevo índice de reglas ni una política de deduplicación.

Producto mayor `principalProduct`:

| Evaluador | Estado | Test / RSS | Resultado completo |
|---|---|---|---|
| Incremental | Punto fijo alcanzado | 2.6474 s / 55.54 MiB | 1 149 eventos, 1 262 justificaciones |
| Referencia | Timeout | 15.0141 s / 32.63 MiB | No disponible |

El cierre incremental instrumentado tardó 2.0585 s, con 106 rondas, 1 188
RuleUses activados y 211 891 intentos de matching. Se verificaron exactamente
los siete coeficientes `[5, 10, 18, 26, 9, 12, 0]`, del exponente 0 al 6. No se
sembró ningún coeficiente de salida.

La referencia terminó por límite, no por error lógico ni OOM. Su último
registro completo fue la ronda 48 con 847 eventos y 960 justificaciones:
esos valores son **parciales**. No se declara equivalencia completa del caso
mayor ni se compara su provenance final inexistente. No se aumentó timeout
y no se ejecutaron tests posteriores a ese límite.

## Recursos y reproducción

21 tests distintos pasaron individualmente mediante `scripts/test_safe.py`,
además de la ejecución diagnóstica de referencia que alcanzó timeout. No se
ejecutó la suite completa. Se conservaron 1 GiB total, cero swap, un trabajo
Cargo, un hilo, 15 s por test y el umbral preventivo del runner.

Dos compilaciones iniciales alcanzaron el umbral preventivo de 768 MiB; el
runner las detuvo antes de ejecutar tests, sin OOM. Se evitó duplicar los
cuerpos grandes de los schedulers para cada tipo de reloj: el reloj existente
se pasa como `&mut dyn DerivationClock`. El evaluador fijo conserva su API
genérica mediante wrappers; ambos cuerpos comparten el mismo contrato de
sellado. No se cambiaron límites ni flags de compilación. El máximo de cgroup
en las verificaciones exitosas fue 700.42 MiB.

Informes locales principales:

- `target/test-diagnostics/20261009T185102568846Z-325667`: quince regresiones CLI.
- `target/test-diagnostics/20261009T185232499055Z-327663`: inicializaciones,
  evaluador fijo, medición pequeña y referencia indexada/lineal.
- `target/test-diagnostics/20261009T185347913326Z-329701`: mayor incremental completo.
- `target/test-diagnostics/20261009T185459545136Z-331116`: errores y asociaciones CLI.
- `target/test-diagnostics/20261009T185500168632Z-331134`: formato final.
- `target/test-diagnostics/20261009T185505500851Z-331288`: referencia mayor limitada.
- `target/test-diagnostics/20261009T183951419082Z-310124` y
  `target/test-diagnostics/20261009T184306048097Z-315207`: compilaciones detenidas.

Reproducción de cada medición mayor, por separado y deteniéndose tras un límite:

```sh
RESIDUUM_PROFILE_PRODUCT=principalProduct python3 scripts/test_safe.py --test-target lib --test derive::profile_tests::profile_semi_naive_polynomial_product_fixture
RESIDUUM_PROFILE_PRODUCT=principalProduct python3 scripts/test_safe.py --test-target lib --test derive::profile_tests::profile_polynomial_product_fixture
```

No se añadió Differential Dataflow, derivada composicional, nuevas primitivas,
sintaxis, consistency, prioridades, retractación ni cambios al runtime de State.
La integración se registró posteriormente en `3bc5082`, sobre `c7d2895`.
La siguiente etapa separa [corrección y escala](program-incremental-correctness-and-scaling.md)
sin añadir optimizaciones al evaluador.
