# Verificación de expedientes y explicaciones tipadas

Rama: `experiment-legal-order-geometry`. Sin commit ni merge. Se preservaron los cambios previos de la rama.

Compilación y tres tests individuales mediante `scripts/test_safe.py`. No se ejecutó la suite completa. 1 GiB total, swap 0, un job de Cargo, un hilo; compilación 45 s, cada test 15 s y parada preventiva a 768 MiB. Ningún límite fue modificado.

| Etapa | Exit | Tiempo (s) | Pico RSS (MiB) | Pico cgroup (MiB) |
|---|---:|---:|---:|---:|
| compile | 0 | 5.4002 | 578.83 | 724.30 |
| closures | 0 | 0.3486 | 15.36 | 10.30 |
| assessments | 0 | 0.2044 | 15.33 | 10.05 |
| invariance | 0 | 0.6771 | 15.36 | 10.25 |

Todos terminaron; sin timeout ni eventos OOM. Se archivaron [solicitudes, scopes, salida y resultados](verification-results.json).

Comandos (el basename del binario puede variar después de recompilar):

```sh
python3 scripts/test_safe.py --format
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 python3 scripts/test_safe.py --test-target cli --compile-only
python3 scripts/test_safe.py --binary target/debug/deps/residuum-f5f0059bb1094ae1 --test cli::tests::legal_geometry::typed_explanations::concrete_dossiers_use_complete_unchanged_reference_closures_and_provenance
python3 scripts/test_safe.py --binary target/debug/deps/residuum-f5f0059bb1094ae1 --test cli::tests::legal_geometry::typed_explanations::explanations_remain_three_way_and_only_counterexamples_exclude_candidates
python3 scripts/test_safe.py --binary target/debug/deps/residuum-f5f0059bb1094ae1 --test cli::tests::legal_geometry::typed_explanations::explanatory_assessments_do_not_change_known_theory_or_proof_multiplicity
```

Los 111 archivos protegidos conservan sus hashes [anteriores](protected-before.json). Entre los archivos preexistentes de Rust sólo se editó legal_geometry_tests.rs para registrar el nuevo módulo de tests. No se modificó el motor ni las declaraciones jurídicas.

Las constantes nuevas únicamente instancian tipos existentes en la fixture. No se añadieron productores de compliance ni se reemplazaron reglas; se verificó la igualdad de todas las declaraciones importadas y de sus orígenes con el módulo original cargado separadamente.

[Hashes de fuentes y entradas](input-sha256.txt). Los cierres y justificaciones completos están en los TSV del mismo directorio. La comparación entre ejecuciones conserva multiplicidad y excluye sólo el sello concreto derivedAt.

Las evaluaciones compatibles no son valoraciones jurídicas aprobadas. La objeción externa permanece abierta; las hipótesis no se convierten en Known.

Referencias conservadas:

```text
main 7b14cb8ac86d07c8d101dc36ca9f1161580f6c22
v1   f6a83e33fa1edb606748a47394e2bde8b4ce635d
```
