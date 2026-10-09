# Sustituciones tipadas canónicas en la rama experimental

Rama `experiment-peano-after`, sobre `a7822ef` y los cambios de profiling,
indexación y contrato incremental anteriores. `main` y el commit del tag `v1`
siguen en `8025bc6b4afee3415c652a5830576ef18aaf8cb5`.

## Representación y frontera de normalización

El tipo del parámetro elaborado determina la representación almacenada:

| Tipo del parámetro | Valor canónico del binding |
|---|---|
| `PropositionTime` | `SubstitutionValue::Time(TimeExpr)` |
| Los demás tipos | `SubstitutionValue::Term(Term)` |

`Term(Const("tau"))` y `Time(At("tau"))` se identifican **sólo cuando el
parámetro es temporal**. Lo mismo vale para constructores `after` anidados.
`Var("t")` y `Const("t")` siguen siendo valores diferentes, igual que
`TimeExpr::Var("t")` y `TimeExpr::At("t")`. No se consulta el contexto fuente
para reinterpretar nombres después de elaborar.

Los records y optionals siguen siendo términos. Sus campos temporales usan
la codificación estructural existente; al extraer un campo hacia un parámetro
`PropositionTime`, el binding se convierte en `Time`. `none` no puede convertirse
en un tiempo. Una constante de entidad no se convierte en tiempo por su nombre.
Esta operación selecciona representación; no sustituye la comprobación de
tipos realizada por el elaborador ni agrega un validador de entidades runtime.

`SubstitutionValue::canonical_for_type` es idempotente en los valores admitidos.
`derive::bind_typed` la aplica al insertar bindings y al comparar restricciones
ya presentes. Matching ordinario e incremental comparten esa inserción;
el pivote incremental no conserva una variante distinta. Las restricciones
iniciales de `match_antecedents_with` también se normalizan antes de un match
`I`. La activación de `Verb.program` usa el mismo binder tipado.

El matching residual conserva el tipo de los parámetros y reutiliza el mismo
binder. El elaborador de aplicaciones de Rule y el compilador de RuleUses
construyen un único mapa canónico: una pasada sustituye tanto argumentos como
índices temporales. La conversión estructural existente transporta el valor a
la posición correspondiente, sin evaluar `after`.

No se normaliza únicamente `DerivationRecord`, no se cambia su igualdad y no
se añade deduplicación hash. Una justificación sólo colapsa con otra cuando
evento, regla, testigos ordenados y sustitución canónica son iguales. Reglas,
programas o testigos distintos conservan justificaciones distintas.

## Resultados comprobados

| Comprobación | Resultado |
|---|---|
| Contraejemplo temporal mínimo | Una justificación, ambos pivotes y cuatro órdenes de actualización |
| Constructores del contraejemplo | Profundidades 0–5, con el tiempo dentro de un record |
| Universo de seis hechos | 448 comparaciones exactas de eventos y provenance |
| Producto pequeño `partialProduct` | 223 eventos y 244 justificaciones en ambos caminos |
| Dos programas y dos testigos para una conclusión | Se conservan las cuatro justificaciones |
| Cierre indexado contra recorrido lineal | Coinciden closure, provenance y sellos de la ejecución |
| Identidad temporal, partial application, records, imports | Regresiones individuales pasan |
| `explain` | Conserva reglas, programas, módulos, testigos y sustituciones |

La comparación del adaptador sólo elimina su guard redundante cuando reproduce
un testigo que el cuerpo original ya contenía. **No transforma bindings ni
fusiona justificaciones**. Ignora el orden de descubrimiento y el valor concreto
de `derivedAt`, conforme al contrato previo, y compara multiplicidad.

Estos son resultados de modelos finitos y regresiones del motor; no una prueba
de equivalencia general de un futuro scheduler semi-naive. El contraejemplo
anterior queda corregido en el punto de almacenamiento, no escondido por una
comparación más débil.

## Verificación y alcance

Se ejecutaron 27 tests distintos, cada uno en un proceso separado mediante
`python3 scripts/test_safe.py`: 19 CLI y 8 de biblioteca. Todos pasaron. Se
conservaron 1 GiB, cero swap, un job de Cargo, un hilo y 15 s por test.
Pico máximo de cgroup observado: 642.95 MiB; pico RSS de tests: 27.00 MiB.
No hubo timeout, crecimiento anormal ni OOM en estas verificaciones.

Informes locales:

- `target/test-diagnostics/20261009T173356983961Z-248924`: contraejemplo corregido.
- `target/test-diagnostics/20261009T173420774684Z-251606`: 448 comparaciones,
  producto y separación de justificaciones.
- `target/test-diagnostics/20261009T174205775450Z-259129`: canonicalización,
  conversiones, sustitución y equivalencia indexada/lineal.
- `target/test-diagnostics/20261009T174256241323Z-260591`: identidad temporal,
  constructores, captures, partial application e imports.
- `target/test-diagnostics/20261009T174308092451Z-260820`: activaciones, provenance
  del producto, β, imports y explain.
- `target/test-diagnostics/20261009T174333348053Z-261120`: regresiones residuales.

Una selección previa escribió `temporal_constructor` en vez de
`temporal_constructors`; el runner la rechazó antes de ejecutar tests. Se
corrigió el nombre y se reutilizó el binario compilado.

No se ejecutó la suite completa ni el producto mayor. No se integró el
scheduler semi-naive en `close_program`, no se modificó la política de
activación, y no se tocaron parser, defaults, State ni la semántica de `after`.
La reparación y los experimentos anteriores permanecen sin commit.
