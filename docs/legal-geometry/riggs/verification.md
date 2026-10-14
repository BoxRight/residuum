# Verificación individual del experimento Riggs

Runner: `python3 scripts/test_safe.py`. Cgroup 1 GiB, swap 0, un trabajo de Cargo, un hilo; timeout de compilación 45 s y tests 15 s. No suite completa.

| Etapa | Resultado | Tiempo | Pico RSS |
|---|---|---:|---:|
| cargo fmt | exit=0 | 0.5095 s | 57.21 MiB |
| compile only | exit=0 | 4.0213 s | 521.47 MiB |
| cli::tests::legal_geometry::riggs::literal_reference_and_typed_counterfactuals_preserve_complete_provenance | exit=0 | 0.4073 s | 15.36 MiB |
| cli::tests::legal_geometry::riggs::sourced_arguments_expose_tension_and_a_discriminating_question | exit=0 | 0.1836 s | 15.17 MiB |
| cli::tests::legal_geometry::riggs::argument_assessment_leaves_theory_known_and_justification_multiplicity_fixed | exit=0 | 0.4285 s | 15.45 MiB |
| cargo fmt | exit=0 | 0.5306 s | 57.44 MiB |
| compile only | exit=0 | 2.8127 s | 405.84 MiB |
| cli::tests::legal_geometry::riggs::literal_reference_and_typed_counterfactuals_preserve_complete_provenance | exit=0 | 0.5114 s | 15.45 MiB |

La primera compilación y los tres tests pasaron. Después se añadió únicamente la auditoría de la tabla de orígenes al primer test; se recompiló y se repitió ese test. Los otros dos cuerpos y sus entradas permanecieron iguales.

Cada request exacto, resultado, salida y comprobación de cgroup está archivado en `verification-results.json`.

Los hashes de las entradas finales están en `input-sha256.txt`. `protected-before.json` conserva los hashes anteriores de las fuentes existentes y las referencias main/v1. Sólo la inscripción del nuevo submódulo de tests cambió entre esas fuentes.

No se creó commit ni se realizó merge.
