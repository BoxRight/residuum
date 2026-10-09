# Revisión de compatibilidad antes del merge posterior a v1

Fecha: 2026-10-09. Candidato: `experiment-peano-after`,
`45f58865784e1f88202e7c6e1602d9619330fc82`.
Base: `8025bc6b4afee3415c652a5830576ef18aaf8cb5`, commit del tag `v1`.

## Alcance que se integra

La rama completa incluye siete commits desde v1, no sólo los dos últimos:

| Commit | Cambio |
|---|---|
| `531544f` | Experimento Peano con programas recursivos |
| `4ac81a4` | Constructores temporales tipados y transporte de valores |
| `175e43b` | Suma de polinomios densos |
| `a7822ef` | Producto polinómico recursivo |
| `c7d2895` | Identidad canónica de bindings y contrato incremental |
| `3bc5082` | Scheduler semi-naive para programas activados dinámicamente |
| `45f5886` | Equivalencias finitas y mediciones de escala separadas |

La compatibilidad revisada es la de los programas Formal y sus resultados en
los casos seleccionados. No se promete compatibilidad de código Rust que
dependa de los enums/representaciones públicas de v1: `Term`, `TimeExpr` y
`Value` incorporan variantes nuevas. Variables y constantes temporales ahora
se distinguen explícitamente. Las sustituciones temporales tienen una forma
canónica; no se promete identidad textual del AST o Debug antiguo.

`close_program_reference` se conserva como evaluador indexado de matching
completo y comparte la elaboración y canonicalización actuales. Es el oráculo
anterior al scheduler semi-naive, no una copia byte por byte del motor v1.

## Verificación seleccionada

Se compilaron ambos targets exclusivamente con `python3 scripts/test_safe.py`
y se ejecutaron **34 tests seleccionados, cada uno en su proceso**:

| Área | Cobertura |
|---|---|
| Imports | Equivalencia inline/modular; asociaciones y β; imports transitivos y diamantes; experiments locales; rutas, ciclos, duplicados y símbolos desconocidos |
| Ontología y extracción | Los 16 cierres de Legal conservan eventos y provenance |
| Defaults | Los cuatro casos de venta; selección del Program completo; reglas del entorno; imports pasivos; aislamiento del Program no seleccionado |
| Residual y CLI | Requisitos sin añadir evidencia; negación evidencial y tiempos anidados; β y residuación; vista forward y provenance; outputs de query/explain y errores existentes |
| Effects | Records Doing/Abstaining, coexistencia e idempotencia; ejecución funcional; derivación separada de State y evidencia histórica |
| Programas asociados | Activación Seeded/Derived, espera por evidencia compatible, β, imports, aislamiento y seeds duplicados |
| Referencia de cierre | Producto pequeño: mismos 223 eventos y las mismas 244 justificaciones completas |

No se ejecutó la suite completa. Los nueve tests CNL ignored continúan pausados.
No se amplió timeout, memoria, swap, concurrencia Cargo ni hilos del harness.
Todas las ejecuciones de esta revisión terminaron normalmente; el test más
largo tardó 1.708 s y el pico RSS máximo fue 18.45 MiB.

Informes locales completos, con selección de tests y recursos:

- `target/test-diagnostics/20261009T230010248076Z-541882`: 20 regresiones CLI.
- `target/test-diagnostics/20261009T230032175103Z-542155`: 14 regresiones library.

La rama no modifica los archivos de `examples/legal`, `src/modules.rs`,
`src/defaults.rs`/`src/defaults`, `scripts/test_safe.py`, `Cargo.toml` ni
`Cargo.lock`. El SHA-256 de la ontología de referencia permanece:

```text
examples/legal/modules/legal_monotonic.res
24655c98ede7b128b8cd82df3a4473c114428b987bb0fe6a6d370c2b37f5575e
```

## Límites de la conclusión

La revisión respalda el merge en el alcance comprobado; no demuestra
compatibilidad universal ni completitud residual. Los resultados finitos y
las mediciones están detallados en
[corrección y escala](program-incremental-correctness-and-scaling.md).

No se volvió a ejecutar la referencia del producto mayor, que antes alcanzó
el límite de 15 segundos. Su equivalencia completa sigue pendiente y no es
condición de este merge. El evaluador de referencia se conserva para ampliar
oráculos acotados cuando una aplicación lo necesite.

La integración se hace mediante merge explícito `--no-ff`. El tag anotado `v1`
debe mantener tanto su objeto `f6a83e33fa1edb606748a47394e2bde8b4ce635d`
como su commit `8025bc6b4afee3415c652a5830576ef18aaf8cb5`.
No se publica al remoto ni se asigna todavía un nuevo número de versión.
