# Integración local de la rama del explorador

Fecha: 2026-10-18. Operación autorizada por el usuario como paso separado y
verificable. Es **local**: no hay push, `main` y `v1` no se han tocado, y no
se ha ejecutado el explorador ni ninguna llamada pagada.

## 1. Commits

| Papel | Commit |
|---|---|
| Origen | `experiment-llm-normative-exploration-spec` = `8edcc303b264878fdc0f64624ddd35bf89aa3045` («Stage I after pilot 003») |
| Destino antes de integrar | `experiment-legal-order-geometry` = `e28fbdf279eb5b8995cd0e80d94b99c3877008ca` (contrato del Piloto 004 congelado) |
| Base común | `4da4eb9f5398740ba530a7ddb212ce50f2ea0255` |
| Copia de seguridad | rama local `backup/geometry-before-explorer-integration` → `e28fbdf` |
| Integración | `ec241dd` (`git merge --no-ff` local) |

## 2. Diferencias comprobadas antes de integrar

| | Cambios desde la base |
|---|---|
| Origen | 271 archivos, +40.376 / −400 líneas |
| Destino | 198 archivos, +13.583 líneas |

- **Solapamiento:** un solo archivo, `src/bin/residuum/cli/tests.rs`, con
  registros de módulos de test en zonas distintas.
- **Conflictos:** ninguno según `git merge-tree --write-tree`, que salió con
  código 0.

**Archivos del motor que trae el origen:** idénticos a `main`, blob a blob:
- `derive.rs`, `derive/incremental.rs`, `derive/program_incremental.rs`,
  `derive/steps.rs`;
- `elab.rs`, `formal.rs`, `lib.rs`, `modules.rs`, `programs.rs`;
- `residual.rs`, `residual/{active,bindings,engine,program,query,steps}.rs`;
- `surface.rs`, `typed.rs`.

La integración, por tanto, no introduce cambios en el motor que no estén ya
aceptados en `main`. El motor de esta rama pasa de la versión de la base a la
de `main`.

**Ontologías:**
- `examples/legal/modules/legal_general.res`: el origen trae la versión de
  `main` (la partición literal).
- `examples/experiments/legal_geometry/ontology_reference/legal_general.res`,
  la que usan la historia temporal y el Piloto 004: **sin cambios**. Su hash
  coincide con `MANIFEST.sha256` del piloto.

**Difieren de `main`** (son propios del explorador):
- `src/experiments.rs`, que registra `worker`, `proposal`, `exploration` y
  `default_steps`;
- `scripts/test_safe.py`, que añade `--interactive-manifest`,
  `--worker-manifest`, `--python-test` y el objetivo `boundary`;
- `Cargo.toml`, que añade el workspace con el crate `boundary`.

**Se reutilizan:** `tools/normative_explorer/` (ledger encadenado,
presupuestos, canal, adaptadores de proveedor, reproducción), el trabajador
Rust y las etapas de admisión de propuestas.

## 3. Compilación con los límites del repositorio

Se mantienen 1 GiB sin swap, un trabajo de Cargo y un hilo. Los límites no se
han cambiado.

| Objetivo | Resultado |
|---|---|
| `cli` | compila (712 MiB) |
| `lib`, con la caché incremental caliente | se detiene por `near_memory_limit` |
| `lib`, tras borrar solo `target/debug/incremental/residuum-*` | compila desde cero (712,6 MiB, informe `20261018T234522641486Z-1022273`) |

El comportamiento del objetivo `lib` coincide con el que ya documentaba el
explorador (`docs/legal-geometry/exploration/verification.md`: 747 MiB desde
cero, 806 MiB con la caché caliente). El margen del objetivo de biblioteca
sigue siendo pequeño. Los tests nuevos del piloto viven en el binario CLI.

## 4. Pruebas ejecutadas tras integrar

Cada test se ejecutó por separado con `scripts/test_safe.py`. El detalle
está en `docs/legal-geometry/pilot-004/runs/integration-tests.txt`.

| Conjunto | Tests | Resultado |
|---|---|---|
| CLI de esta rama: historia temporal (8), dominio semántico (4), orden temporal (4) y Piloto 004 (4) | 20 | 20 PASS |
| CLI de la geometría jurídica anterior (`legal_geometry*`) | 64 | 64 PASS |
| CLI del explorador y de `main` (`query_binding_syntax`, `legal_partition`) | 12 | 12 PASS |
| Biblioteca del explorador (`experiments::{worker,proposal,exploration,step,query_binding}`, `residual::{steps,bindings}`, `derive::steps`) | 60 | 60 PASS |
| `boundary`: generador de fixtures grabados (`process_script_fixture_is_written_with_in_memory_expectation`) | 1 | PASS |
| Python `tools/normative_explorer/tests` (`--python-test`) | 67 | 55 PASS en la primera pasada; los 12 restantes, PASS al repetirlos tras el generador |

Los 12 fallos de la primera pasada tenían una única causa: faltaba el
fixture grabado en `target/exploration/process/`, que genera previamente el
test de `boundary` (los propios tests lo dicen en su mensaje de error). No es
un efecto de la integración.

**Artefactos reescritos por los tests antiguos.** Algunos tests de
`legal_geometry*` escriben TSV en `docs/legal-geometry/`. Tras ejecutarlos,
14 archivos cambiaron. Al comparar normalizando las rutas absolutas de origen
(`file: Some("…")`), 13 solo difieren en esas rutas: se generaron en otras
copias del repositorio (`/home/maitreya/residuum`,
`/home/maitreya/12hectareas/vendor/residuum`). El restante,
`riggs-conversation/legal-contexts.tsv`, difiere además porque el volcado
`Debug` de `Experiment` incluye el campo `query_options`, que añade el motor
de `main`. No hay cambios semánticos. Los artefactos se han restaurado a su
versión archivada y no forman parte de la integración.

**Hashes del Piloto 004:** `sha256sum -c docs/legal-geometry/pilot-004/MANIFEST.sha256`
sigue dando OK tras la integración.

## 5. Resultado

Integración local completada y comprobada. No se ha ejecutado el explorador
(ni `run_episode`, ni el trabajador en modo interactivo, ni proveedores) y no
se ha hecho ninguna llamada pagada.
