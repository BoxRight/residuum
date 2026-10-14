# Verificación individual del aprendizaje argumentativo de Riggs

Todos los comandos se ejecutaron mediante `python3 scripts/test_safe.py`. 1 GiB total, swap 0, un trabajo Cargo, un hilo. Compilación 45 s, tests 15 s; límites intactos.

| Etapa | Exit | Tiempo | Pico RSS |
|---|---:|---:|---:|
| cargo fmt | 0 | 0.6342 s | 57.10 MiB |
| compile only | 0 | 6.9877 s | 586.85 MiB |
| cli::tests::legal_geometry::riggs::conversation::external_answers_update_argument_versions_without_conflating_premises | 0 | 0.4894 s | 15.55 MiB |
| cli::tests::legal_geometry::riggs::conversation::observation_partitions_meet_and_minimax_questions_respect_pending_answers | 0 | 0.4491 s | 15.37 MiB |
| cli::tests::legal_geometry::riggs::conversation::conversations_preserve_complete_legal_closures_and_sourced_objections | 0 | 0.7566 s | 15.07 MiB |
| cli::tests::legal_geometry::conversation::seventh_external_valuation_updates_versions_meet_and_minimax_questions | 0 | 0.4489 s | 15.32 MiB |
| cli::tests::legal_geometry::riggs::sourced_arguments_expose_tension_and_a_discriminating_question | 0 | 0.1838 s | 15.33 MiB |
| cargo fmt | 0 | 0.6343 s | 57.62 MiB |
| compile only | 0 | 2.0374 s | 316.68 MiB |
| cli::tests::legal_geometry::riggs::conversation::observation_partitions_meet_and_minimax_questions_respect_pending_answers | 0 | 0.5124 s | 15.32 MiB |

Los requests exactos, salidas, scopes y métricas se conservan en `verification-results.json`. La primera compilación se reutilizó para los cinco tests individuales. Después se amplió sólo el test de particiones para archivar y verificar las preguntas de los dos controles de evidencia pendiente; se recompiló y repitió ese test. Los otros cuerpos y entradas permanecieron iguales. No se ejecutó la suite completa.

La auditoría mecánica compara los 157 archivos originales capturados antes del cambio. Sólo cambiaron tres visibilidades de helpers en el test conversacional previo y la inscripción del nuevo submódulo dentro de los tests de Riggs. Motor, todas las fuentes jurídicas y todas las fixtures previas permanecen byte por byte iguales. `main` y `v1` conservan sus hashes.

Se verificaron 42 comparaciones independientes de cierre. Las dos continuaciones hipotéticas se verifican en el test de particiones/versiones; cada valoración del guion está cubierta por la comprobación de conservación del contexto fijo.

No hubo commit ni merge.
