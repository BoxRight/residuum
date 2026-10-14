# Verificación individual

1 GiB total, sin swap, un job de Cargo y un hilo; stop de memoria a 768 MiB.

| Nombre | Exit status | Tiempo (s) | Pico RSS (MiB) | Resultado |
|---|---:|---:|---:|---|
| list tests | 0 | 0.0220 | 15.48 | passed |
| list ignored tests | 0 | 0.0220 | 15.35 | passed |
| cli::tests::legal_geometry::recalcitrance::recalcitrant_observations_filter_antichains_and_refine_hypothesis_partitions | 0 | 0.3279 | 15.48 | passed |
| cli::tests::legal_geometry::recalcitrance::justificatory_revisions_are_hypothetical_and_certified_by_real_closures | 0 | 1.5048 | 15.55 | passed |
| cli::tests::legal_geometry::recalcitrance::hypothetical_revision_geometry_checks_anti_exchange_without_using_it_as_a_filter | 0 | 1.2813 | 41.63 | passed |

Compilación separada: 3.8693 s; 408.40 MiB RSS; exit 0.

Logs: `target/test-diagnostics/20261011T042328166794Z-85194`, `target/test-diagnostics/20261011T042303072526Z-84625`.

Sólo tres tests seleccionados; no suite completa. Todas las ejecuciones mediante `scripts/test_safe.py`.

Durante preparación se corrigieron dos errores del experimento: la estimación del universo con ~Act (14 eventos, no 13) y los paréntesis requeridos en declaraciones de reglas ground. La geometría exhaustiva usa el dominio inicial de ocho eventos; no se elevó ningún límite de recursos ni se modificó el parser.
