# Verificación individual de la continuación conversacional

Rama: `experiment-legal-order-geometry`. No se hizo merge ni commit en esta etapa.
Los archivos del motor y del experimento anterior conservan sus hashes del inicio de esta tarea; se preservaron los cambios previos de la rama. Los resultados anteriores no se volvieron a generar.

Compilación y tests exclusivamente mediante `scripts/test_safe.py`, con 1 GiB total, swap 0, un job de Cargo y un hilo. Timeout de compilación 45 s y de cada test 15 s; parada preventiva a 768 MiB, sin modificaciones.

| Etapa | Status | Tiempo (s) | Pico RSS (MiB) | Pico cgroup (MiB) |
|---|---:|---:|---:|---:|
| compile | 0 | 6.7806 | 530.40 | 542.50 |
| versions | 0 | 0.6138 | 15.16 | 10.25 |
| diagnostics | 0 | 0.5929 | 15.36 | 10.27 |
| conservation | 0 | 0.5737 | 15.23 | 11.26 |

Los tres tests alcanzaron su resultado completo; ninguno agotó presupuesto ni tiempo. Sin eventos OOM. No se ejecutó la suite completa.

Comandos usados (el hash del nombre del binario puede variar al recompilar):

```sh
python3 scripts/test_safe.py --format
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 python3 scripts/test_safe.py --test-target cli --compile-only
python3 scripts/test_safe.py --binary target/debug/deps/residuum-f5f0059bb1094ae1 --test cli::tests::legal_geometry::conversation::seventh_external_valuation_updates_versions_meet_and_minimax_questions
python3 scripts/test_safe.py --binary target/debug/deps/residuum-f5f0059bb1094ae1 --test cli::tests::legal_geometry::conversation::empty_version_spaces_have_distinct_diagnostics_without_inventing_functions
python3 scripts/test_safe.py --binary target/debug/deps/residuum-f5f0059bb1094ae1 --test cli::tests::legal_geometry::conversation::conversational_learning_keeps_norm_known_times_and_legal_provenance_intact
```

[Resultados, solicitudes, scopes y salida del runner](verification-results.json) archivados para mantener el registro aunque se limpie target. Los paths originales de diagnósticos figuran también en ese archivo.

Conservación de referencias:

```text
main 7b14cb8ac86d07c8d101dc36ca9f1161580f6c22
v1   f6a83e33fa1edb606748a47394e2bde8b4ce635d
```

El [registro de hashes](input-sha256.txt) fija entradas, harness y fuentes jurídicas. La [instantánea anterior](preserved-inputs-before.json) permite comprobar que los archivos protegidos no cambiaron durante este incremento.
